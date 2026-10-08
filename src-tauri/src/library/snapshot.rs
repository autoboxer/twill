use std::collections::{BTreeSet, HashSet};

use rusqlite::{Connection, Row};
use serde::de::DeserializeOwned;

use super::content::{validate_content, ValidatedContent};
use super::media::validate_media_ids;
use super::retrieval_forms::{parse_retrieval_form_configuration, RetrievalFormConfiguration};
use super::service::{validate_retrieval_form_selection, validate_template_selections};
use super::{
    ConceptContent, CssSnippetContent, LibraryError, LibraryResult, RetrievalFormKind,
    TemplateContent,
};

const MAXIMUM_JSON_BYTES: usize = 4_000_000;

pub(crate) fn validate_restored_content(connection: &Connection) -> LibraryResult<()> {
    validate_card_parents(connection)?;
    super::practice_links::validate_restored_links(connection)?;

    let mut statement = connection.prepare(
        "SELECT entity_id, content_json FROM concepts
        INNER JOIN entities ON entities.id = concepts.entity_id
        WHERE entities.deleted_at IS NULL",
    )?;
    let mut rows = statement.query([])?;

    while let Some(row) = rows.next()? {
        let id: String = row.get(0)?;
        let content: ConceptContent = read_json(row, 1)?;
        let validated = validate_content(content)?;

        validate_media_links(connection, &id, &validated.media_ids)?;
        validate_concept_cards(connection, &id, &validated)?;
    }

    let mut statement = connection.prepare(
        "SELECT content_json FROM templates
        INNER JOIN entities ON entities.id = templates.entity_id
        WHERE entities.deleted_at IS NULL",
    )?;
    let mut rows = statement.query([])?;

    while let Some(row) = rows.next()? {
        let content: TemplateContent = read_json(row, 0)?;
        let validated = super::templates::validate_template_content(content.clone())?;

        if serde_json::to_value(content)? != serde_json::to_value(validated)? {
            return Err(LibraryError::InvalidRetrievalForm);
        }
    }

    let mut statement = connection.prepare(
        "SELECT schema_version, source FROM css_snippets
        INNER JOIN entities ON entities.id = css_snippets.entity_id
        WHERE entities.deleted_at IS NULL",
    )?;
    let mut rows = statement.query([])?;

    while let Some(row) = rows.next()? {
        super::css_snippets::validate_content(CssSnippetContent {
            schema_version: row.get(0)?,
            source: row.get(1)?,
        })?;
    }

    Ok(())
}

fn validate_media_links(
    connection: &Connection,
    concept_id: &str,
    expected_ids: &HashSet<String>,
) -> LibraryResult<()> {
    let mut statement = connection.prepare(
        "SELECT media_id FROM concept_media
        WHERE concept_id = ?1 AND removed_at IS NULL",
    )?;
    let mut rows = statement.query([concept_id])?;
    let mut count = 0;

    while let Some(row) = rows.next()? {
        let id: String = row.get(0)?;

        if !expected_ids.contains(&id) {
            return Err(LibraryError::InvalidRetrievalForm);
        }

        count += 1;
    }

    if count != expected_ids.len() {
        return Err(LibraryError::InvalidRetrievalForm);
    }

    validate_media_ids(connection, expected_ids)
}

fn validate_card_parents(connection: &Connection) -> LibraryResult<()> {
    let invalid: bool = connection.query_row(
        "SELECT EXISTS (
            SELECT 1 FROM cards
            INNER JOIN entities AS card_entities ON card_entities.id = cards.entity_id
            WHERE card_entities.deleted_at IS NULL
                AND (
                    card_entities.kind != 'card'
                    OR NOT EXISTS (
                        SELECT 1 FROM concepts
                        INNER JOIN entities ON entities.id = concepts.entity_id
                        WHERE concepts.entity_id = cards.concept_id
                            AND entities.kind = 'concept'
                            AND entities.deleted_at IS NULL
                    )
                )
        )",
        [],
        |row| row.get(0),
    )?;

    if invalid {
        return Err(LibraryError::InvalidRetrievalForm);
    }

    Ok(())
}

fn validate_concept_cards(
    connection: &Connection,
    concept_id: &str,
    content: &ValidatedContent,
) -> LibraryResult<()> {
    let mut cloze_groups = content
        .cloze_group_ids
        .iter()
        .map(String::as_str)
        .collect::<HashSet<_>>();
    let mut occlusion_groups = content
        .image_occlusion_group_ids
        .iter()
        .map(String::as_str)
        .collect::<HashSet<_>>();
    let mut forms = HashSet::new();
    let mut template_ids = BTreeSet::new();
    let mut statement = connection.prepare(
        "SELECT retrieval_kind, configuration_json, template_id FROM cards
        INNER JOIN entities ON entities.id = cards.entity_id
        WHERE concept_id = ?1 AND entities.deleted_at IS NULL",
    )?;
    let mut rows = statement.query([concept_id])?;

    while let Some(row) = rows.next()? {
        let kind = RetrievalFormKind::try_from(row.get::<_, String>(0)?.as_str())?;
        let configuration = parse_retrieval_form_configuration(kind, read_json_text(row, 1)?)?;
        let template_id: Option<String> = row.get(2)?;

        if let Some(id) = &template_id {
            template_ids.insert(id.clone());
        }

        let identity = match configuration {
            RetrievalFormConfiguration::Cloze(settings) => {
                if !cloze_groups.remove(settings.group_id.as_str()) {
                    return Err(LibraryError::InvalidRetrievalForm);
                }

                Some(settings.group_id)
            }
            RetrievalFormConfiguration::ImageOcclusion(settings) => {
                if !occlusion_groups.remove(settings.group_id.as_str()) {
                    return Err(LibraryError::InvalidRetrievalForm);
                }

                Some(settings.group_id)
            }
            RetrievalFormConfiguration::Problem(_) if !content.prompt_has_content => {
                return Err(LibraryError::MissingProblemPrompt);
            }
            _ => template_id,
        };

        if !forms.insert((kind.as_str(), identity)) {
            return Err(LibraryError::InvalidRetrievalForm);
        }
    }

    if !cloze_groups.is_empty() || !occlusion_groups.is_empty() {
        return Err(LibraryError::InvalidRetrievalForm);
    }

    validate_template_selections(connection, &template_ids)?;
    validate_retrieval_form_selection(
        forms.contains(&("recall", None)),
        &template_ids,
        forms.contains(&("explain", None)),
        forms.contains(&("problem", None)),
        forms.contains(&("type_answer", None)),
        !content.cloze_group_ids.is_empty(),
        !content.image_occlusion_group_ids.is_empty(),
    )
}

fn read_json_text<'row>(row: &'row Row<'_>, column: usize) -> LibraryResult<&'row str> {
    let value = row.get_ref(column)?;
    let value = value.as_str().map_err(rusqlite::Error::from)?;

    if value.len() > MAXIMUM_JSON_BYTES {
        return Err(LibraryError::InvalidRetrievalForm);
    }

    Ok(value)
}

fn read_json<T: DeserializeOwned>(row: &Row<'_>, column: usize) -> LibraryResult<T> {
    Ok(serde_json::from_str(read_json_text(row, column)?)?)
}
