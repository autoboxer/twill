use std::collections::HashSet;

use ammonia::Builder;
use rusqlite::{params, Connection, OptionalExtension};

use crate::data::{EntityKind, LocalDataStore};
use crate::library::{
    CreateTemplateInput, LibraryError, LibraryResult, TemplateBlock, TemplateCatalog,
    TemplateContent, TemplateDetail, TemplateSummary, UpdateTemplateInput,
    TEMPLATE_SCHEMA_VERSION,
};

const MAXIMUM_TEMPLATE_NAME_LENGTH: usize = 80;
const MAXIMUM_TEMPLATE_BLOCKS_PER_SIDE: usize = 50;
const MAXIMUM_TEMPLATE_TEXT_LENGTH: usize = 1_000;
const MAXIMUM_TEMPLATE_HTML_BYTES: usize = 200_000;
const MAXIMUM_TEMPLATE_CSS_BYTES: usize = 100_000;

const ALLOWED_TEMPLATE_TAGS: [&str; 45] = [
    "address",
    "article",
    "aside",
    "b",
    "blockquote",
    "br",
    "code",
    "dd",
    "div",
    "dl",
    "dt",
    "em",
    "figcaption",
    "figure",
    "footer",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "header",
    "hr",
    "i",
    "li",
    "main",
    "mark",
    "ol",
    "p",
    "pre",
    "s",
    "section",
    "small",
    "span",
    "strong",
    "sub",
    "sup",
    "table",
    "tbody",
    "td",
    "tfoot",
    "th",
    "thead",
    "tr",
    "u",
];

const ALLOWED_TEMPLATE_ATTRIBUTES: [&str; 8] = [
    "aria-hidden",
    "aria-label",
    "class",
    "dir",
    "id",
    "lang",
    "role",
    "title",
];

const CLEAN_CONTENT_TAGS: [&str; 8] = [
    "iframe",
    "math",
    "object",
    "script",
    "style",
    "svg",
    "template",
    "textarea",
];

pub struct TemplateLibrary<'store> {
    store: &'store LocalDataStore,
}

impl<'store> TemplateLibrary<'store> {
    pub fn new(store: &'store LocalDataStore) -> Self {
        Self { store }
    }

    pub fn catalog(&self) -> LibraryResult<TemplateCatalog> {
        self.store.read_result(query_catalog)
    }

    pub fn template(&self, id: &str) -> LibraryResult<TemplateDetail> {
        let id = id.trim();

        self.store
            .read_result(|connection| query_template(connection, id))
    }

    pub fn create_template(
        &self,
        input: CreateTemplateInput,
    ) -> LibraryResult<TemplateDetail> {
        let name = normalize_name(input.name)?;
        let content = validate_template_content(input.content)?;
        let serialized = serde_json::to_string(&content)?;

        self.store.write_result(|transaction| {
            ensure_unique_template_name(transaction, &name, None)?;

            let entity = transaction.create_entity(EntityKind::Template)?;

            transaction.execute(
                "INSERT INTO templates (
                    entity_id,
                    name,
                    content_json,
                    last_change_id
                ) VALUES (?1, ?2, ?3, ?4)",
                params![entity.id, name, serialized, entity.last_change_id],
            )?;

            query_template(transaction, &entity.id)
        })
    }

    pub fn update_template(
        &self,
        input: UpdateTemplateInput,
    ) -> LibraryResult<TemplateDetail> {
        let id = input.id.trim().to_owned();
        let name = normalize_name(input.name)?;
        let content = validate_template_content(input.content)?;
        let serialized = serde_json::to_string(&content)?;

        self.store.write_result(|transaction| {
            let current = query_template(transaction, &id)?;

            if current.name == name && current.content == content {
                return Ok(current);
            }

            ensure_unique_template_name(transaction, &name, Some(&id))?;

            let entity = transaction.touch_entity(&id)?;

            transaction.execute(
                "UPDATE templates
                SET name = ?1,
                    content_json = ?2,
                    last_change_id = ?3
                WHERE entity_id = ?4",
                params![name, serialized, entity.last_change_id, id],
            )?;

            query_template(transaction, &id)
        })
    }

    pub fn delete_template(&self, id: &str) -> LibraryResult<()> {
        let id = id.trim().to_owned();

        self.store.write_result(|transaction| {
            if !template_record_exists(transaction, &id)? {
                return Err(LibraryError::TemplateNotFound(id));
            }

            let retrieval_form_count = active_retrieval_form_count(transaction, &id)?;

            if retrieval_form_count > 0 {
                return Err(LibraryError::TemplateInUse {
                    retrieval_form_count,
                });
            }

            transaction.soft_delete_entity(&id)?;

            Ok(())
        })
    }

    pub fn prepare_content(content: TemplateContent) -> LibraryResult<TemplateContent> {
        validate_template_content(content)
    }
}

fn normalize_name(name: String) -> LibraryResult<String> {
    let name = name.trim().to_owned();

    if name.is_empty() {
        return Err(LibraryError::EmptyValue {
            field: "Template name",
        });
    }

    if name.chars().count() > MAXIMUM_TEMPLATE_NAME_LENGTH {
        return Err(LibraryError::ValueTooLong {
            field: "Template name",
            maximum: MAXIMUM_TEMPLATE_NAME_LENGTH,
        });
    }

    Ok(name)
}

fn validate_template_content(
    mut content: TemplateContent,
) -> LibraryResult<TemplateContent> {
    if content.schema_version != TEMPLATE_SCHEMA_VERSION {
        return Err(invalid_template(
            "Template",
            "uses an unsupported schema version",
        ));
    }

    validate_visual_side(&content.visual.front.blocks, "Visual front")?;
    validate_visual_side(&content.visual.answer.blocks, "Visual answer")?;

    content.custom.front_html =
        sanitize_template_html(&content.custom.front_html, "Custom front HTML")?;
    content.custom.answer_html =
        sanitize_template_html(&content.custom.answer_html, "Custom answer HTML")?;
    content.custom.css = validate_template_css(&content.custom.css)?;

    Ok(content)
}

fn validate_visual_side(
    blocks: &[TemplateBlock],
    name: &'static str,
) -> LibraryResult<()> {
    if blocks.is_empty() {
        return Err(invalid_template(name, "must contain at least one block"));
    }

    if blocks.len() > MAXIMUM_TEMPLATE_BLOCKS_PER_SIDE {
        return Err(invalid_template(name, "contains too many blocks"));
    }

    let mut unique_fields = HashSet::new();

    for block in blocks {
        match block {
            TemplateBlock::Field { field } => {
                if !unique_fields.insert(field) {
                    return Err(invalid_template(
                        name,
                        "cannot contain a field more than once",
                    ));
                }
            }
            TemplateBlock::Text { text } => {
                if text.trim().is_empty() {
                    return Err(invalid_template(name, "contains an empty text block"));
                }

                if text.chars().count() > MAXIMUM_TEMPLATE_TEXT_LENGTH {
                    return Err(invalid_template(
                        name,
                        "contains a text block that is too long",
                    ));
                }
            }
        }
    }

    if unique_fields.is_empty() {
        return Err(invalid_template(name, "must contain at least one field"));
    }

    Ok(())
}

fn sanitize_template_html(
    source: &str,
    field: &'static str,
) -> LibraryResult<String> {
    if source.len() > MAXIMUM_TEMPLATE_HTML_BYTES {
        return Err(invalid_template(field, "is too large"));
    }

    if source.contains('\0') {
        return Err(invalid_template(field, "contains an invalid character"));
    }

    let mut sanitizer = Builder::empty();

    sanitizer
        .add_tags(&ALLOWED_TEMPLATE_TAGS)
        .add_generic_attributes(&ALLOWED_TEMPLATE_ATTRIBUTES)
        .add_clean_content_tags(&CLEAN_CONTENT_TAGS)
        .strip_comments(true);

    let sanitized = sanitizer.clean(source.trim()).to_string();

    validate_field_tokens(&sanitized, field)?;

    Ok(sanitized)
}

fn validate_field_tokens(source: &str, field: &'static str) -> LibraryResult<()> {
    let mut cursor = 0;
    let mut field_count = 0;

    while let Some(relative_start) = source[cursor..].find("{{") {
        let start = cursor + relative_start;

        if source[cursor..start].contains("}}") {
            return Err(invalid_template(field, "contains malformed field syntax"));
        }

        let Some(relative_end) = source[start + 2..].find("}}") else {
            return Err(invalid_template(field, "contains malformed field syntax"));
        };
        let end = start + 2 + relative_end;
        let token = &source[start..end + 2];

        if field_token_is_inside_tag(source, start) {
            return Err(invalid_template(
                field,
                "can only place fields in document content",
            ));
        }

        let field_name = token[2..token.len() - 2].trim();

        if !matches!(field_name, "title" | "prompt" | "answer") {
            return Err(invalid_template(field, "contains an unknown field"));
        }

        field_count += 1;
        cursor = end + 2;
    }

    if source[cursor..].contains("}}") {
        return Err(invalid_template(field, "contains malformed field syntax"));
    }

    if field_count == 0 {
        return Err(invalid_template(
            field,
            "must contain at least one concept field",
        ));
    }

    Ok(())
}

fn field_token_is_inside_tag(source: &str, offset: usize) -> bool {
    let last_open = source[..offset].rfind('<');
    let last_close = source[..offset].rfind('>');

    matches!((last_open, last_close), (Some(open), Some(close)) if open > close)
        || matches!((last_open, last_close), (Some(_), None))
}

fn validate_template_css(source: &str) -> LibraryResult<String> {
    if source.len() > MAXIMUM_TEMPLATE_CSS_BYTES {
        return Err(invalid_template("CSS", "is too large"));
    }

    if source.contains('\0') {
        return Err(invalid_template("CSS", "contains an invalid character"));
    }

    let normalized = source.to_lowercase();

    if normalized.contains("</style") {
        return Err(invalid_template("CSS", "contains an invalid style tag"));
    }

    if normalized.contains("@import") || normalized.contains("url(") {
        return Err(invalid_template(
            "CSS",
            "cannot load external resources",
        ));
    }

    Ok(source.trim().to_owned())
}

fn query_catalog(connection: &Connection) -> LibraryResult<TemplateCatalog> {
    let mut statement = connection.prepare(
        "SELECT
            templates.entity_id,
            templates.name,
            entities.updated_at,
            templates.content_json,
            (
                SELECT COUNT(*)
                FROM cards
                INNER JOIN entities AS card_entities
                    ON card_entities.id = cards.entity_id
                WHERE cards.template_id = templates.entity_id
                    AND card_entities.deleted_at IS NULL
            )
        FROM templates
        INNER JOIN entities ON entities.id = templates.entity_id
        WHERE entities.deleted_at IS NULL
        ORDER BY templates.name COLLATE NOCASE, templates.entity_id",
    )?;
    let templates = statement.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, i64>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, i64>(4)?,
        ))
    })?;
    let templates = templates
        .map(|template| {
            let (id, name, updated_at, content, retrieval_form_count) = template?;
            let content: TemplateContent = serde_json::from_str(&content)?;

            Ok(TemplateSummary {
                id,
                name,
                updated_at,
                mode: content.mode,
                retrieval_form_count,
            })
        })
        .collect::<LibraryResult<_>>()?;

    Ok(TemplateCatalog { templates })
}

fn query_template(connection: &Connection, id: &str) -> LibraryResult<TemplateDetail> {
    let template = connection
        .query_row(
            "SELECT
                templates.entity_id,
                templates.name,
                entities.created_at,
                entities.updated_at,
                entities.last_change_id,
                templates.content_json
            FROM templates
            INNER JOIN entities ON entities.id = templates.entity_id
            WHERE templates.entity_id = ?1
                AND entities.deleted_at IS NULL",
            [id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                ))
            },
        )
        .optional()?;

    let Some((id, name, created_at, updated_at, last_change_id, content)) = template else {
        return Err(LibraryError::TemplateNotFound(id.to_owned()));
    };

    Ok(TemplateDetail {
        id,
        name,
        created_at,
        updated_at,
        last_change_id,
        content: serde_json::from_str(&content)?,
    })
}

fn ensure_unique_template_name(
    connection: &Connection,
    name: &str,
    excluded_id: Option<&str>,
) -> LibraryResult<()> {
    let mut statement = connection.prepare(
        "SELECT templates.entity_id, templates.name
        FROM templates
        INNER JOIN entities ON entities.id = templates.entity_id
        WHERE entities.deleted_at IS NULL",
    )?;
    let names = statement.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;
    let normalized_name = name.to_lowercase();

    for item in names {
        let (id, existing_name) = item?;

        if excluded_id == Some(id.as_str()) {
            continue;
        }

        if existing_name.to_lowercase() == normalized_name {
            return Err(LibraryError::DuplicateName {
                kind: "template",
                name: name.to_owned(),
            });
        }
    }

    Ok(())
}

fn template_record_exists(connection: &Connection, id: &str) -> LibraryResult<bool> {
    Ok(connection.query_row(
        "SELECT EXISTS (SELECT 1 FROM templates WHERE entity_id = ?1)",
        [id],
        |row| row.get(0),
    )?)
}

fn active_retrieval_form_count(
    connection: &Connection,
    template_id: &str,
) -> LibraryResult<i64> {
    Ok(connection.query_row(
        "SELECT COUNT(*)
        FROM cards
        INNER JOIN entities ON entities.id = cards.entity_id
        WHERE cards.template_id = ?1
            AND entities.deleted_at IS NULL",
        [template_id],
        |row| row.get(0),
    )?)
}

fn invalid_template(
    field: &'static str,
    message: impl Into<String>,
) -> LibraryError {
    LibraryError::InvalidTemplate {
        field,
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use rusqlite::params;
    use tempfile::tempdir;

    use super::TemplateLibrary;
    use crate::data::{ChangeOperation, DataResult, EntityKind, LocalDataStore};
    use crate::library::{
        CreateTemplateInput, LibraryError, TemplateContent, UpdateTemplateInput,
    };
    use crate::library::models::TemplateMode;

    #[test]
    fn templates_can_be_created_updated_listed_and_deleted() {
        let directory = tempdir().unwrap();
        let store = LocalDataStore::open(directory.path()).unwrap();
        let library = TemplateLibrary::new(&store);

        let created = library
            .create_template(CreateTemplateInput {
                name: "  Basic recall  ".to_owned(),
                content: TemplateContent::default(),
            })
            .unwrap();

        assert_eq!(created.name, "Basic recall");
        assert_eq!(library.catalog().unwrap().templates.len(), 1);
        assert_eq!(library.template(&created.id).unwrap(), created);
        assert_eq!(
            store.entity(&created.id).unwrap().unwrap().kind,
            EntityKind::Template
        );

        let updated = library
            .update_template(UpdateTemplateInput {
                id: created.id.clone(),
                name: "Focused recall".to_owned(),
                content: TemplateContent::default(),
            })
            .unwrap();

        assert_eq!(updated.name, "Focused recall");
        assert_eq!(store.entity(&created.id).unwrap().unwrap().revision, 2);

        let unchanged = library
            .update_template(UpdateTemplateInput {
                id: created.id.clone(),
                name: updated.name.clone(),
                content: updated.content.clone(),
            })
            .unwrap();

        assert_eq!(unchanged, updated);
        assert_eq!(store.entity(&created.id).unwrap().unwrap().revision, 2);

        library.delete_template(&created.id).unwrap();
        library.delete_template(&created.id).unwrap();

        assert!(library.catalog().unwrap().templates.is_empty());
        assert!(matches!(
            library.template(&created.id),
            Err(LibraryError::TemplateNotFound(_))
        ));

        let operations: Vec<_> = store
            .changes_after(0, 10)
            .unwrap()
            .into_iter()
            .map(|change| change.operation)
            .collect();

        assert_eq!(
            operations,
            vec![
                ChangeOperation::Create,
                ChangeOperation::Update,
                ChangeOperation::Delete,
            ]
        );
    }

    #[test]
    fn template_names_and_visual_fields_are_validated() {
        let directory = tempdir().unwrap();
        let store = LocalDataStore::open(directory.path()).unwrap();
        let library = TemplateLibrary::new(&store);
        let first = library
            .create_template(CreateTemplateInput {
                name: "Recall".to_owned(),
                content: TemplateContent::default(),
            })
            .unwrap();

        assert!(matches!(
            library.create_template(CreateTemplateInput {
                name: " recall ".to_owned(),
                content: TemplateContent::default(),
            }),
            Err(LibraryError::DuplicateName { .. })
        ));

        let mut invalid_content = TemplateContent::default();
        invalid_content.visual.front.blocks.clear();

        assert!(matches!(
            library.update_template(UpdateTemplateInput {
                id: first.id,
                name: first.name,
                content: invalid_content,
            }),
            Err(LibraryError::InvalidTemplate { .. })
        ));
    }

    #[test]
    fn custom_html_is_sanitized_and_field_syntax_is_restricted() {
        let directory = tempdir().unwrap();
        let store = LocalDataStore::open(directory.path()).unwrap();
        let library = TemplateLibrary::new(&store);
        let custom_content = |front_html: &str| {
            let mut content = TemplateContent::default();

            content.mode = TemplateMode::Custom;
            content.custom.front_html = front_html.to_owned();
            content.custom.answer_html = "<div class=answer>{{ answer }}</div>".to_owned();
            content.custom.css = ".answer { color: green; }".to_owned();

            content
        };

        let created = library
            .create_template(CreateTemplateInput {
                name: "Custom".to_owned(),
                content: custom_content(
                    "<section class=front onclick=alert(1)>{{ prompt }}<script>alert(1)</script></section>",
                ),
            })
            .unwrap();

        assert_eq!(
            created.content.custom.front_html,
            "<section class=\"front\">{{ prompt }}</section>"
        );

        for invalid_html in [
            "<div>{{missing}}</div>",
            "<div class=\"{{title}}\">Front</div>",
            "<div>{{prompt}</div>",
            "<div>No fields</div>",
        ] {
            assert!(matches!(
                library.create_template(CreateTemplateInput {
                    name: format!("Invalid {invalid_html}"),
                    content: custom_content(invalid_html),
                }),
                Err(LibraryError::InvalidTemplate { .. })
            ));
        }
    }

    #[test]
    fn template_rows_cannot_bypass_change_tracking_or_tombstones() {
        let directory = tempdir().unwrap();
        let store = LocalDataStore::open(directory.path()).unwrap();
        let library = TemplateLibrary::new(&store);
        let template = library
            .create_template(CreateTemplateInput {
                name: "Protected".to_owned(),
                content: TemplateContent::default(),
            })
            .unwrap();

        let update = store.write(|transaction| {
            transaction.execute(
                "UPDATE templates SET name = 'Untracked' WHERE entity_id = ?1",
                [&template.id],
            )?;

            Ok(())
        });
        let delete = store.write(|transaction| {
            transaction.execute(
                "DELETE FROM templates WHERE entity_id = ?1",
                [&template.id],
            )?;

            Ok(())
        });

        assert!(update.is_err());
        assert!(delete.is_err());
        assert_eq!(library.template(&template.id).unwrap().name, "Protected");

        let row_count: i64 = store
            .read_result(|connection| -> DataResult<i64> {
                Ok(connection.query_row(
                    "SELECT COUNT(*) FROM templates WHERE entity_id = ?1",
                    params![template.id],
                    |row| row.get(0),
                )?)
            })
            .unwrap();

        assert_eq!(row_count, 1);
    }
}
