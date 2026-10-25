use std::collections::HashSet;

use serde::Deserialize;
use serde_json::{Map, Value};
use uuid::Uuid;

use super::{invalid_content, optional_array};
use crate::library::LibraryResult;

const MAXIMUM_TARGETS: usize = 100;
const MAXIMUM_LABEL_LENGTH: usize = 200;
const MAXIMUM_ALTERNATIVES: usize = 20;
const MAXIMUM_DESCRIPTION_LENGTH: usize = 500;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ImageLabelTarget {
    id: String,
    x: f64,
    y: f64,
    label: String,
    alternatives: Vec<String>,
    description: String,
}

pub(super) fn validate(
    attributes: &Map<String, Value>,
    field: &'static str,
    ids: &mut HashSet<String>,
) -> LibraryResult<()> {
    let targets = optional_array(attributes, "labelTargets", field)?;

    if targets.is_empty() {
        return Ok(());
    }

    if field != "Prompt" {
        return Err(invalid_content(field, "cannot contain image label targets"));
    }

    if ids.len() + targets.len() > MAXIMUM_TARGETS {
        return Err(invalid_content(field, "cannot contain more than 100 image label targets"));
    }

    for value in targets {
        let target: ImageLabelTarget = serde_json::from_value(value.clone())
            .map_err(|_| invalid_content(field, "contains an invalid image label target"))?;

        if !Uuid::parse_str(&target.id).is_ok_and(|id| id.to_string() == target.id)
            || !ids.insert(target.id)
        {
            return Err(invalid_content(field, "contains an invalid or duplicate image label identifier"));
        }

        if !target.x.is_finite() || !target.y.is_finite()
            || !(0.0..=1.0).contains(&target.x) || !(0.0..=1.0).contains(&target.y)
        {
            return Err(invalid_content(field, "contains an image label target outside its image"));
        }

        validate_label(&target.label, field)?;

        if target.description.chars().count() > MAXIMUM_DESCRIPTION_LENGTH {
            return Err(invalid_content(field, "contains an image target description longer than 500 characters"));
        }

        if target.alternatives.len() > MAXIMUM_ALTERNATIVES {
            return Err(invalid_content(field, "contains an image label with more than 20 alternatives"));
        }

        let mut answers = HashSet::from([comparison_key(&target.label)]);

        for alternative in target.alternatives {
            validate_label(&alternative, field)?;

            if !answers.insert(comparison_key(&alternative)) {
                return Err(invalid_content(field, "contains duplicate image label alternatives"));
            }
        }
    }

    Ok(())
}

fn validate_label(label: &str, field: &'static str) -> LibraryResult<()> {
    if label.trim().is_empty() || label.chars().count() > MAXIMUM_LABEL_LENGTH {
        return Err(invalid_content(field, "image labels must contain between 1 and 200 characters"));
    }

    Ok(())
}

fn comparison_key(label: &str) -> String {
    label.split_whitespace().collect::<Vec<_>>().join(" ").to_lowercase()
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::library::{content::validate_content, ConceptContent};

    fn target() -> Value {
        json!({
            "id": Uuid::now_v7().to_string(), "x": 0.25, "y": 0.5,
            "label": "Nucleus", "alternatives": ["Cell nucleus"],
            "description": "The round structure near the center."
        })
    }

    fn content(targets: Vec<Value>) -> ConceptContent {
        let mut content = ConceptContent::default();
        content.prompt = json!({ "type": "doc", "content": [{
            "type": "mediaImage", "attrs": {
                "mediaId": "018f1e2d-3c4b-7a69-8f10-123456789abc", "labelTargets": targets
            }
        }] });
        content
    }

    #[test]
    fn image_labels_preserve_unicode_geometry_and_media_ownership() {
        let mut label = target();
        label["label"] = json!("🌿".repeat(200));
        label["description"] = json!("網".repeat(500));
        label["x"] = json!(0);
        label["y"] = json!(1);
        let input = content(vec![label.clone()]);
        let validated = validate_content(input).unwrap();

        assert_eq!(validated.content.prompt["content"][0]["attrs"]["labelTargets"][0], label);
        assert_eq!(validated.media_ids.len(), 1);
        assert!(validated.image_occlusion_group_ids.is_empty());
    }

    #[test]
    fn image_labels_reject_invalid_answers_coordinates_and_identifiers() {
        for (field, value) in [
            ("id", json!("not-an-id")),
            ("id", json!("018F1E2D-3C4B-7A69-8F10-123456789ABC")),
            ("x", json!(-0.001)), ("y", json!(1.001)), ("x", Value::Null),
            ("x", json!("0.5")), ("label", json!(" \n ")),
            ("label", json!("🌿".repeat(201))),
            ("description", json!("網".repeat(501))),
            ("alternatives", json!(["  NUCLEUS  "])),
            ("alternatives", json!(["Cell  nucleus", "cell nucleus"])),
            ("alternatives", json!([""])),
            ("alternatives", json!(["網".repeat(201)])),
            ("alternatives", json!((0..21).map(|i| i.to_string()).collect::<Vec<_>>())),
            ("unknown", json!(true)),
        ] {
            let mut label = target();
            label[field] = value;
            assert!(validate_content(content(vec![label])).is_err(), "Accepted {field}");
        }

        let label = target();
        assert!(validate_content(content(vec![label.clone(), label])).is_err());
    }

    #[test]
    fn image_label_limits_and_identity_apply_across_the_whole_prompt() {
        let targets = (0..100).map(|_| target()).collect();
        let mut input = content(targets);
        assert!(validate_content(input.clone()).is_ok());
        let second = content(vec![target()]).prompt["content"][0].clone();
        input.prompt["content"].as_array_mut().unwrap().push(second);
        assert!(validate_content(input).is_err());

        let mut input = content(vec![target()]);
        let duplicate = input.prompt["content"][0].clone();
        input.prompt["content"].as_array_mut().unwrap().push(duplicate);
        assert!(validate_content(input).is_err());

        let labeled = content(vec![target()]).prompt;
        for field in ["answer", "hint", "reference", "explanation", "commonMistakes"] {
            let mut input = ConceptContent::default();
            match field {
                "answer" => input.answer = labeled.clone(),
                "hint" => input.assistance.hint = labeled.clone(),
                "reference" => input.assistance.reference = labeled.clone(),
                "explanation" => input.feedback.explanation = labeled.clone(),
                _ => input.feedback.common_mistakes = labeled.clone(),
            }
            assert!(validate_content(input).is_err());
        }
    }
}
