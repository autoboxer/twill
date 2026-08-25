use std::collections::{HashMap, HashSet};

use serde_json::{Map, Value};
use uuid::Uuid;

use crate::library::{
    ConceptContent, LibraryError, LibraryResult, RICH_CONTENT_SCHEMA_VERSION,
};

const MAXIMUM_DOCUMENT_BYTES: usize = 1_000_000;
const MAXIMUM_DOCUMENT_DEPTH: usize = 32;
const MAXIMUM_DOCUMENT_NODES: usize = 10_000;
const MAXIMUM_DOCUMENT_TEXT: usize = 500_000;
const MAXIMUM_LATEX_LENGTH: usize = 10_000;
const MAXIMUM_LINK_LENGTH: usize = 2_048;
const MAXIMUM_ATTRIBUTE_TEXT_LENGTH: usize = 500;
const MAXIMUM_CLOZE_GROUPS: usize = 100;
const MAXIMUM_IMAGE_OCCLUSION_GROUPS: usize = 100;
const MAXIMUM_IMAGE_OCCLUSION_REGIONS: usize = 500;

pub struct ValidatedContent {
    pub cloze_group_ids: Vec<String>,
    pub content: ConceptContent,
    pub image_occlusion_group_ids: Vec<String>,
    pub media_ids: HashSet<String>,
    pub prompt_has_content: bool,
    pub serialized: String,
}

#[derive(Clone, Copy)]
enum NodeContext {
    Block,
    Code,
    Inline,
    ListItem,
}

struct ValidationState {
    cloze_group_ids: Vec<String>,
    cloze_group_set: HashSet<String>,
    image_occlusion_group_ids: Vec<String>,
    image_occlusion_group_sources: HashMap<String, usize>,
    image_occlusion_region_ids: HashSet<String>,
    media_ids: HashSet<String>,
    node_count: usize,
    text_length: usize,
}

pub fn validate_content(content: ConceptContent) -> LibraryResult<ValidatedContent> {
    if content.schema_version != RICH_CONTENT_SCHEMA_VERSION {
        return Err(invalid_content(
            "Content",
            "uses an unsupported schema version",
        ));
    }

    let serialized = serde_json::to_string(&content)?;

    if serialized.len() > MAXIMUM_DOCUMENT_BYTES * 2 {
        return Err(invalid_content("Content", "is too large"));
    }

    let mut state = ValidationState {
        cloze_group_ids: Vec::new(),
        cloze_group_set: HashSet::new(),
        image_occlusion_group_ids: Vec::new(),
        image_occlusion_group_sources: HashMap::new(),
        image_occlusion_region_ids: HashSet::new(),
        media_ids: HashSet::new(),
        node_count: 0,
        text_length: 0,
    };

    validate_document(&content.prompt, "Prompt", &mut state)?;
    validate_document(&content.answer, "Answer", &mut state)?;

    Ok(ValidatedContent {
        cloze_group_ids: state.cloze_group_ids,
        prompt_has_content: rich_document_has_content(&content.prompt),
        content,
        image_occlusion_group_ids: state.image_occlusion_group_ids,
        media_ids: state.media_ids,
        serialized,
    })
}

fn rich_document_has_content(document: &Value) -> bool {
    let Some(node) = document.as_object() else {
        return false;
    };

    match node.get("type").and_then(Value::as_str) {
        Some("text") => node
            .get("text")
            .and_then(Value::as_str)
            .is_some_and(|text| !text.trim().is_empty()),
        Some("inlineMath" | "blockMath") => node
            .get("attrs")
            .and_then(Value::as_object)
            .and_then(|attributes| attributes.get("latex"))
            .and_then(Value::as_str)
            .is_some_and(|latex| !latex.trim().is_empty()),
        Some("mediaImage") => true,
        _ => node
            .get("content")
            .and_then(Value::as_array)
            .is_some_and(|content| content.iter().any(rich_document_has_content)),
    }
}

fn validate_document(
    document: &Value,
    field: &'static str,
    state: &mut ValidationState,
) -> LibraryResult<()> {
    if serde_json::to_vec(document)?.len() > MAXIMUM_DOCUMENT_BYTES {
        return Err(invalid_content(field, "is too large"));
    }

    let object = required_object(document, field)?;

    ensure_keys(object, &["type", "content"], field)?;

    if required_string(object, "type", field)? != "doc" {
        return Err(invalid_content(field, "must be a rich-text document"));
    }

    let content = required_array(object, "content", field)?;

    if content.is_empty() {
        return Err(invalid_content(field, "must contain at least one block"));
    }

    for node in content {
        validate_node(node, NodeContext::Block, field, 1, state)?;
    }

    Ok(())
}

fn validate_node(
    node: &Value,
    context: NodeContext,
    field: &'static str,
    depth: usize,
    state: &mut ValidationState,
) -> LibraryResult<()> {
    if depth > MAXIMUM_DOCUMENT_DEPTH {
        return Err(invalid_content(field, "is nested too deeply"));
    }

    state.node_count += 1;

    if state.node_count > MAXIMUM_DOCUMENT_NODES {
        return Err(invalid_content(field, "contains too many nodes"));
    }

    let object = required_object(node, field)?;
    let node_type = required_string(object, "type", field)?;

    if !node_is_allowed(node_type, context) {
        return Err(invalid_content(
            field,
            format!("contains an unsupported {node_type} node"),
        ));
    }

    match node_type {
        "paragraph" => validate_inline_container(object, field, depth, state),
        "heading" => validate_heading(object, field, depth, state),
        "blockquote" => validate_block_container(object, field, depth, state),
        "bulletList" => validate_list(object, false, field, depth, state),
        "orderedList" => validate_list(object, true, field, depth, state),
        "listItem" => validate_list_item(object, field, depth, state),
        "codeBlock" => validate_code_block(object, field, depth, state),
        "text" => validate_text(object, context, field, state),
        "hardBreak" | "horizontalRule" => {
            ensure_keys(object, &["type"], field)
        }
        "inlineMath" | "blockMath" => validate_math(object, field),
        "mediaImage" => validate_media_image(object, field, state),
        _ => Err(invalid_content(field, "contains an unsupported node")),
    }
}

fn node_is_allowed(node_type: &str, context: NodeContext) -> bool {
    match context {
        NodeContext::Block => matches!(
            node_type,
            "paragraph"
                | "heading"
                | "blockquote"
                | "bulletList"
                | "orderedList"
                | "codeBlock"
                | "blockMath"
                | "mediaImage"
                | "horizontalRule"
        ),
        NodeContext::Inline => matches!(node_type, "text" | "hardBreak" | "inlineMath"),
        NodeContext::ListItem => node_type == "listItem",
        NodeContext::Code => node_type == "text",
    }
}

fn validate_inline_container(
    object: &Map<String, Value>,
    field: &'static str,
    depth: usize,
    state: &mut ValidationState,
) -> LibraryResult<()> {
    ensure_keys(object, &["type", "content"], field)?;

    for node in optional_array(object, "content", field)? {
        validate_node(node, NodeContext::Inline, field, depth + 1, state)?;
    }

    Ok(())
}

fn validate_heading(
    object: &Map<String, Value>,
    field: &'static str,
    depth: usize,
    state: &mut ValidationState,
) -> LibraryResult<()> {
    ensure_keys(object, &["type", "attrs", "content"], field)?;

    let attributes = required_attributes(object, field)?;
    ensure_keys(attributes, &["level"], field)?;

    let level = required_integer(attributes, "level", field)?;

    if !(1..=3).contains(&level) {
        return Err(invalid_content(field, "contains an unsupported heading level"));
    }

    for node in optional_array(object, "content", field)? {
        validate_node(node, NodeContext::Inline, field, depth + 1, state)?;
    }

    Ok(())
}

fn validate_block_container(
    object: &Map<String, Value>,
    field: &'static str,
    depth: usize,
    state: &mut ValidationState,
) -> LibraryResult<()> {
    ensure_keys(object, &["type", "content"], field)?;

    let content = required_array(object, "content", field)?;

    if content.is_empty() {
        return Err(invalid_content(field, "contains an empty block container"));
    }

    for node in content {
        validate_node(node, NodeContext::Block, field, depth + 1, state)?;
    }

    Ok(())
}

fn validate_list(
    object: &Map<String, Value>,
    ordered: bool,
    field: &'static str,
    depth: usize,
    state: &mut ValidationState,
) -> LibraryResult<()> {
    ensure_keys(object, &["type", "attrs", "content"], field)?;

    if let Some(attributes) = optional_attributes(object, field)? {
        let keys = if ordered {
            &["start", "type"][..]
        } else {
            &["type"][..]
        };

        ensure_keys(attributes, keys, field)?;

        if ordered {
            if let Some(start) = optional_integer(attributes, "start", field)? {
                if !(1..=1_000_000).contains(&start) {
                    return Err(invalid_content(field, "contains an invalid list start"));
                }
            }
        }

        validate_optional_short_string(attributes, "type", 16, field)?;
    }

    let content = required_array(object, "content", field)?;

    if content.is_empty() {
        return Err(invalid_content(field, "contains an empty list"));
    }

    for node in content {
        validate_node(node, NodeContext::ListItem, field, depth + 1, state)?;
    }

    Ok(())
}

fn validate_list_item(
    object: &Map<String, Value>,
    field: &'static str,
    depth: usize,
    state: &mut ValidationState,
) -> LibraryResult<()> {
    ensure_keys(object, &["type", "content"], field)?;

    let content = required_array(object, "content", field)?;

    let first_node_type = content
        .first()
        .and_then(Value::as_object)
        .and_then(|node| node.get("type"))
        .and_then(Value::as_str);

    if first_node_type != Some("paragraph") {
        return Err(invalid_content(field, "contains an invalid list item"));
    }

    for node in content {
        validate_node(node, NodeContext::Block, field, depth + 1, state)?;
    }

    Ok(())
}

fn validate_code_block(
    object: &Map<String, Value>,
    field: &'static str,
    depth: usize,
    state: &mut ValidationState,
) -> LibraryResult<()> {
    ensure_keys(object, &["type", "attrs", "content"], field)?;

    if let Some(attributes) = optional_attributes(object, field)? {
        ensure_keys(attributes, &["language"], field)?;
        validate_optional_short_string(attributes, "language", 40, field)?;
    }

    for node in optional_array(object, "content", field)? {
        validate_node(node, NodeContext::Code, field, depth + 1, state)?;
    }

    Ok(())
}

fn validate_text(
    object: &Map<String, Value>,
    context: NodeContext,
    field: &'static str,
    state: &mut ValidationState,
) -> LibraryResult<()> {
    let allowed_keys = match context {
        NodeContext::Code => &["type", "text"][..],
        _ => &["type", "text", "marks"][..],
    };

    ensure_keys(object, allowed_keys, field)?;

    let text = required_string(object, "text", field)?;

    if text.is_empty() {
        return Err(invalid_content(field, "contains an empty text node"));
    }

    state.text_length += text.chars().count();

    if state.text_length > MAXIMUM_DOCUMENT_TEXT {
        return Err(invalid_content(field, "contains too much text"));
    }

    if !matches!(context, NodeContext::Code) {
        validate_marks(object, text, field, state)?;
    }

    Ok(())
}

fn validate_marks(
    object: &Map<String, Value>,
    text: &str,
    field: &'static str,
    state: &mut ValidationState,
) -> LibraryResult<()> {
    let mut has_cloze_mark = false;

    for mark in optional_array(object, "marks", field)? {
        let mark = required_object(mark, field)?;
        let mark_type = required_string(mark, "type", field)?;

        match mark_type {
            "bold" | "italic" | "underline" | "strike" | "code" => {
                ensure_keys(mark, &["type"], field)?;
            }
            "link" => validate_link(mark, field)?,
            "cloze" if has_cloze_mark => {
                return Err(invalid_content(
                    field,
                    "contains overlapping cloze omissions",
                ));
            }
            "cloze" => {
                validate_cloze(mark, text, field, state)?;
                has_cloze_mark = true;
            }
            _ => {
                return Err(invalid_content(
                    field,
                    format!("contains an unsupported {mark_type} mark"),
                ));
            }
        }
    }

    Ok(())
}

fn validate_cloze(
    mark: &Map<String, Value>,
    text: &str,
    field: &'static str,
    state: &mut ValidationState,
) -> LibraryResult<()> {
    if field != "Prompt" {
        return Err(invalid_content(
            field,
            "cannot contain cloze omissions",
        ));
    }

    if text.trim().is_empty() {
        return Err(invalid_content(
            field,
            "contains an empty cloze omission",
        ));
    }

    ensure_keys(mark, &["type", "attrs"], field)?;

    let attributes = required_attributes(mark, field)?;
    ensure_keys(attributes, &["groupId"], field)?;

    let group_id = required_string(attributes, "groupId", field)?;

    if Uuid::parse_str(group_id).is_err() {
        return Err(invalid_content(
            field,
            "contains an invalid cloze group",
        ));
    }

    if state.cloze_group_set.insert(group_id.to_owned()) {
        if state.cloze_group_ids.len() >= MAXIMUM_CLOZE_GROUPS {
            return Err(invalid_content(
                field,
                format!("cannot contain more than {MAXIMUM_CLOZE_GROUPS} cloze groups"),
            ));
        }

        state.cloze_group_ids.push(group_id.to_owned());
    }

    Ok(())
}

fn validate_link(mark: &Map<String, Value>, field: &'static str) -> LibraryResult<()> {
    ensure_keys(mark, &["type", "attrs"], field)?;

    let attributes = required_attributes(mark, field)?;
    ensure_keys(
        attributes,
        &["href", "target", "rel", "class", "title"],
        field,
    )?;

    let href = required_string(attributes, "href", field)?.trim();
    let normalized_href = href.to_ascii_lowercase();
    let allowed_protocol = normalized_href.starts_with("https://")
        || normalized_href.starts_with("http://")
        || normalized_href.starts_with("mailto:");

    if href.is_empty()
        || href.len() > MAXIMUM_LINK_LENGTH
        || href.chars().any(char::is_control)
        || !allowed_protocol
    {
        return Err(invalid_content(field, "contains an unsafe link"));
    }

    validate_optional_short_string(attributes, "target", 32, field)?;
    validate_optional_short_string(attributes, "rel", 100, field)?;
    validate_optional_short_string(attributes, "class", 100, field)?;
    validate_optional_short_string(
        attributes,
        "title",
        MAXIMUM_ATTRIBUTE_TEXT_LENGTH,
        field,
    )?;

    Ok(())
}

fn validate_math(object: &Map<String, Value>, field: &'static str) -> LibraryResult<()> {
    ensure_keys(object, &["type", "attrs"], field)?;

    let attributes = required_attributes(object, field)?;
    ensure_keys(attributes, &["latex"], field)?;

    let latex = required_string(attributes, "latex", field)?;

    if latex.chars().count() > MAXIMUM_LATEX_LENGTH {
        return Err(invalid_content(field, "contains an equation that is too long"));
    }

    Ok(())
}

fn validate_media_image(
    object: &Map<String, Value>,
    field: &'static str,
    state: &mut ValidationState,
) -> LibraryResult<()> {
    ensure_keys(object, &["type", "attrs"], field)?;

    let attributes = required_attributes(object, field)?;
    ensure_keys(
        attributes,
        &["mediaId", "alt", "title", "occlusionRegions"],
        field,
    )?;

    let media_id = required_string(attributes, "mediaId", field)?;

    if Uuid::parse_str(media_id).is_err() {
        return Err(invalid_content(field, "contains an invalid image reference"));
    }

    validate_optional_short_string(
        attributes,
        "alt",
        MAXIMUM_ATTRIBUTE_TEXT_LENGTH,
        field,
    )?;
    validate_optional_short_string(
        attributes,
        "title",
        MAXIMUM_ATTRIBUTE_TEXT_LENGTH,
        field,
    )?;
    validate_image_occlusion_regions(attributes, field, state)?;

    state.media_ids.insert(media_id.to_owned());

    Ok(())
}

fn validate_image_occlusion_regions(
    attributes: &Map<String, Value>,
    field: &'static str,
    state: &mut ValidationState,
) -> LibraryResult<()> {
    let regions = optional_array(attributes, "occlusionRegions", field)?;

    if regions.is_empty() {
        return Ok(());
    }

    if field != "Prompt" {
        return Err(invalid_content(
            field,
            "cannot contain image occlusion regions",
        ));
    }

    let image_node_index = state.node_count;

    for region in regions {
        if state.image_occlusion_region_ids.len() >= MAXIMUM_IMAGE_OCCLUSION_REGIONS {
            return Err(invalid_content(
                field,
                format!(
                    "cannot contain more than {MAXIMUM_IMAGE_OCCLUSION_REGIONS} image occlusion regions"
                ),
            ));
        }

        let region = required_object(region, field)?;

        ensure_keys(
            region,
            &["id", "groupId", "x", "y", "width", "height"],
            field,
        )?;

        let region_id = required_string(region, "id", field)?;
        let group_id = required_string(region, "groupId", field)?;

        if Uuid::parse_str(region_id).is_err()
            || !state
                .image_occlusion_region_ids
                .insert(region_id.to_owned())
        {
            return Err(invalid_content(
                field,
                "contains an invalid or duplicate image occlusion region",
            ));
        }

        if Uuid::parse_str(group_id).is_err() {
            return Err(invalid_content(
                field,
                "contains an invalid image occlusion group",
            ));
        }

        match state.image_occlusion_group_sources.get(group_id) {
            Some(source) if *source != image_node_index => {
                return Err(invalid_content(
                    field,
                    "contains an image occlusion group shared by multiple images",
                ));
            }
            Some(_) => {}
            None => {
                if state.image_occlusion_group_ids.len() >= MAXIMUM_IMAGE_OCCLUSION_GROUPS {
                    return Err(invalid_content(
                        field,
                        format!(
                            "cannot contain more than {MAXIMUM_IMAGE_OCCLUSION_GROUPS} image occlusion groups"
                        ),
                    ));
                }

                state
                    .image_occlusion_group_sources
                    .insert(group_id.to_owned(), image_node_index);
                state.image_occlusion_group_ids.push(group_id.to_owned());
            }
        }

        let x = required_number(region, "x", field)?;
        let y = required_number(region, "y", field)?;
        let width = required_number(region, "width", field)?;
        let height = required_number(region, "height", field)?;
        let outside_image = x < 0.0
            || y < 0.0
            || width <= 0.0
            || height <= 0.0
            || x + width > 1.0 + 1e-9
            || y + height > 1.0 + 1e-9;

        if outside_image {
            return Err(invalid_content(
                field,
                "contains an image occlusion region outside its image",
            ));
        }
    }

    Ok(())
}

fn required_object<'value>(
    value: &'value Value,
    field: &'static str,
) -> LibraryResult<&'value Map<String, Value>> {
    value
        .as_object()
        .ok_or_else(|| invalid_content(field, "contains a value that is not an object"))
}

fn required_attributes<'value>(
    object: &'value Map<String, Value>,
    field: &'static str,
) -> LibraryResult<&'value Map<String, Value>> {
    object
        .get("attrs")
        .and_then(Value::as_object)
        .ok_or_else(|| invalid_content(field, "contains invalid node attributes"))
}

fn optional_attributes<'value>(
    object: &'value Map<String, Value>,
    field: &'static str,
) -> LibraryResult<Option<&'value Map<String, Value>>> {
    match object.get("attrs") {
        Some(Value::Object(attributes)) => Ok(Some(attributes)),
        Some(_) => Err(invalid_content(field, "contains invalid node attributes")),
        None => Ok(None),
    }
}

fn required_string<'value>(
    object: &'value Map<String, Value>,
    key: &str,
    field: &'static str,
) -> LibraryResult<&'value str> {
    object
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| invalid_content(field, format!("contains an invalid {key} value")))
}

fn required_integer(
    object: &Map<String, Value>,
    key: &str,
    field: &'static str,
) -> LibraryResult<i64> {
    object
        .get(key)
        .and_then(Value::as_i64)
        .ok_or_else(|| invalid_content(field, format!("contains an invalid {key} value")))
}

fn required_number(
    object: &Map<String, Value>,
    key: &str,
    field: &'static str,
) -> LibraryResult<f64> {
    object
        .get(key)
        .and_then(Value::as_f64)
        .ok_or_else(|| invalid_content(field, format!("contains an invalid {key} value")))
}

fn optional_integer(
    object: &Map<String, Value>,
    key: &str,
    field: &'static str,
) -> LibraryResult<Option<i64>> {
    match object.get(key) {
        Some(Value::Number(value)) => value
            .as_i64()
            .map(Some)
            .ok_or_else(|| invalid_content(field, format!("contains an invalid {key} value"))),
        Some(Value::Null) | None => Ok(None),
        Some(_) => Err(invalid_content(
            field,
            format!("contains an invalid {key} value"),
        )),
    }
}

fn required_array<'value>(
    object: &'value Map<String, Value>,
    key: &str,
    field: &'static str,
) -> LibraryResult<&'value [Value]> {
    object
        .get(key)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .ok_or_else(|| invalid_content(field, format!("contains an invalid {key} value")))
}

fn optional_array<'value>(
    object: &'value Map<String, Value>,
    key: &str,
    field: &'static str,
) -> LibraryResult<&'value [Value]> {
    match object.get(key) {
        Some(Value::Array(values)) => Ok(values),
        Some(_) => Err(invalid_content(
            field,
            format!("contains an invalid {key} value"),
        )),
        None => Ok(&[]),
    }
}

fn validate_optional_short_string(
    object: &Map<String, Value>,
    key: &str,
    maximum: usize,
    field: &'static str,
) -> LibraryResult<()> {
    match object.get(key) {
        Some(Value::String(value)) if value.chars().count() <= maximum => Ok(()),
        Some(Value::Null) | None => Ok(()),
        Some(_) => Err(invalid_content(
            field,
            format!("contains an invalid {key} value"),
        )),
    }
}

fn ensure_keys(
    object: &Map<String, Value>,
    allowed: &[&str],
    field: &'static str,
) -> LibraryResult<()> {
    if let Some(key) = object.keys().find(|key| !allowed.contains(&key.as_str())) {
        return Err(invalid_content(
            field,
            format!("contains an unsupported {key} property"),
        ));
    }

    Ok(())
}

fn invalid_content(
    field: &'static str,
    message: impl Into<String>,
) -> LibraryError {
    LibraryError::InvalidContent {
        field,
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{json, Value};

    use super::{rich_document_has_content, validate_content};
    use crate::library::{ConceptContent, LibraryError};

    #[test]
    fn meaningful_content_distinguishes_empty_structure_from_problem_material() {
        let empty = ConceptContent::default().prompt;
        let whitespace = json!({
            "type": "doc",
            "content": [{
                "type": "paragraph",
                "content": [{ "type": "text", "text": " \n " }]
            }]
        });
        let equation = json!({
            "type": "doc",
            "content": [{
                "type": "paragraph",
                "content": [{
                    "type": "inlineMath",
                    "attrs": { "latex": "F = ma" }
                }]
            }]
        });
        let image = json!({
            "type": "doc",
            "content": [{
                "type": "mediaImage",
                "attrs": { "mediaId": "018f1e2d-3c4b-7a69-8f10-123456789abc" }
            }]
        });

        assert!(!rich_document_has_content(&empty));
        assert!(!rich_document_has_content(&whitespace));
        assert!(rich_document_has_content(&equation));
        assert!(rich_document_has_content(&image));
    }

    #[test]
    fn allowlisted_documents_collect_media_references() {
        let media_id = "018f1e2d-3c4b-7a69-8f10-123456789abc";
        let cloze_group_id = "018f1e2d-3c4b-7a69-8f10-123456789abd";
        let occlusion_group_id = "018f1e2d-3c4b-7a69-8f10-123456789abe";
        let occlusion_region_id = "018f1e2d-3c4b-7a69-8f10-123456789abf";
        let content = ConceptContent {
            schema_version: 1,
            prompt: json!({
                "type": "doc",
                "content": [
                    {
                        "type": "heading",
                        "attrs": { "level": 2 },
                        "content": [{
                            "type": "text",
                            "text": "Cell membrane",
                            "marks": [{
                                "type": "link",
                                "attrs": {
                                    "href": "https://example.com/cells",
                                    "target": "_blank",
                                    "rel": "noopener noreferrer nofollow",
                                    "class": null,
                                    "title": null
                                }
                            }, {
                                "type": "cloze",
                                "attrs": {
                                    "groupId": cloze_group_id
                                }
                            }]
                        }]
                    },
                    {
                        "type": "mediaImage",
                        "attrs": {
                            "mediaId": media_id,
                            "alt": "Cell membrane diagram",
                            "title": null,
                            "occlusionRegions": [{
                                "id": occlusion_region_id,
                                "groupId": occlusion_group_id,
                                "x": 0.1,
                                "y": 0.2,
                                "width": 0.3,
                                "height": 0.25
                            }]
                        }
                    }
                ]
            }),
            answer: json!({
                "type": "doc",
                "content": [{
                    "type": "paragraph",
                    "content": [{
                        "type": "inlineMath",
                        "attrs": { "latex": "E = mc^2" }
                    }]
                }]
            }),
        };

        let validated = validate_content(content).unwrap();

        assert_eq!(validated.cloze_group_ids, vec![cloze_group_id]);
        assert_eq!(
            validated.image_occlusion_group_ids,
            vec![occlusion_group_id]
        );
        assert_eq!(validated.media_ids, [media_id.to_owned()].into());
        assert!(validated.serialized.contains("Cell membrane"));
    }

    #[test]
    fn cloze_marks_require_valid_prompt_groups() {
        let group_id = "018f1e2d-3c4b-7a69-8f10-123456789abc";
        let marked_text = |text: &str, id: &str| {
            json!({
                "type": "text",
                "text": text,
                "marks": [{
                    "type": "cloze",
                    "attrs": { "groupId": id }
                }]
            })
        };
        let document = |node| {
            json!({
                "type": "doc",
                "content": [{
                    "type": "paragraph",
                    "content": [node]
                }]
            })
        };

        let answer_mark = ConceptContent {
            schema_version: 1,
            prompt: ConceptContent::default().prompt,
            answer: document(marked_text("Answer", group_id)),
        };
        let invalid_group = ConceptContent {
            schema_version: 1,
            prompt: document(marked_text("Prompt", "not-a-uuid")),
            answer: ConceptContent::default().answer,
        };
        let empty_omission = ConceptContent {
            schema_version: 1,
            prompt: document(marked_text("   ", group_id)),
            answer: ConceptContent::default().answer,
        };

        for content in [answer_mark, invalid_group, empty_omission] {
            assert!(matches!(
                validate_content(content),
                Err(LibraryError::InvalidContent { .. })
            ));
        }
    }

    #[test]
    fn image_occlusion_regions_require_valid_prompt_geometry() {
        let first_media = "018f1e2d-3c4b-7a69-8f10-123456789ab1";
        let second_media = "018f1e2d-3c4b-7a69-8f10-123456789ab2";
        let first_region = "018f1e2d-3c4b-7a69-8f10-123456789ab3";
        let second_region = "018f1e2d-3c4b-7a69-8f10-123456789ab4";
        let group_id = "018f1e2d-3c4b-7a69-8f10-123456789ab5";
        let document = |nodes: Vec<Value>| json!({ "type": "doc", "content": nodes });
        let image = |media_id: &str, regions: Vec<Value>| {
            json!({
                "type": "mediaImage",
                "attrs": {
                    "mediaId": media_id,
                    "alt": null,
                    "title": null,
                    "occlusionRegions": regions
                }
            })
        };
        let region = |id: &str, group: &str, x: f64, width: f64| {
            json!({
                "id": id,
                "groupId": group,
                "x": x,
                "y": 0.2,
                "width": width,
                "height": 0.25
            })
        };
        let invalid_contents = [
            ConceptContent {
                schema_version: 1,
                prompt: ConceptContent::default().prompt,
                answer: document(vec![image(
                    first_media,
                    vec![region(first_region, group_id, 0.1, 0.3)],
                )]),
            },
            ConceptContent {
                schema_version: 1,
                prompt: document(vec![image(
                    first_media,
                    vec![region(first_region, "not-a-uuid", 0.1, 0.3)],
                )]),
                answer: ConceptContent::default().answer,
            },
            ConceptContent {
                schema_version: 1,
                prompt: document(vec![image(
                    first_media,
                    vec![region(first_region, group_id, 0.1, 0.0)],
                )]),
                answer: ConceptContent::default().answer,
            },
            ConceptContent {
                schema_version: 1,
                prompt: document(vec![image(
                    first_media,
                    vec![region(first_region, group_id, 0.8, 0.3)],
                )]),
                answer: ConceptContent::default().answer,
            },
            ConceptContent {
                schema_version: 1,
                prompt: document(vec![image(
                    first_media,
                    vec![
                        region(first_region, group_id, 0.1, 0.3),
                        region(first_region, group_id, 0.5, 0.3),
                    ],
                )]),
                answer: ConceptContent::default().answer,
            },
            ConceptContent {
                schema_version: 1,
                prompt: document(vec![
                    image(
                        first_media,
                        vec![region(first_region, group_id, 0.1, 0.3)],
                    ),
                    image(
                        second_media,
                        vec![region(second_region, group_id, 0.2, 0.3)],
                    ),
                ]),
                answer: ConceptContent::default().answer,
            },
        ];

        for content in invalid_contents {
            assert!(matches!(
                validate_content(content),
                Err(LibraryError::InvalidContent { .. })
            ));
        }
    }

    #[test]
    fn executable_or_unrecognized_content_is_rejected() {
        let content = ConceptContent {
            schema_version: 1,
            prompt: json!({
                "type": "doc",
                "content": [{
                    "type": "paragraph",
                    "content": [{
                        "type": "text",
                        "text": "Unsafe",
                        "marks": [{
                            "type": "link",
                            "attrs": {
                                "href": "javascript:alert(1)",
                                "target": null,
                                "rel": null,
                                "class": null,
                                "title": null
                            }
                        }]
                    }]
                }]
            }),
            answer: ConceptContent::default().answer,
        };

        assert!(matches!(
            validate_content(content),
            Err(LibraryError::InvalidContent { .. })
        ));
    }
}
