use rusqlite::{params, Connection};
use serde_json::Value;

use super::assignments::attach_assignments;
use super::organizations::OrganizationKind;
use super::ConceptLibrary;
use crate::data::{current_timestamp, WriteTransaction};
use crate::library::{
    ConceptDetail, ConceptSummary, LibraryCardMatch, LibraryCardState, LibraryError, LibraryPage,
    LibraryQuery, LibraryResult, LibrarySort, NamedItem, RetrievalFormKind,
};

const PAGE_SIZE: i64 = 50;
const MAXIMUM_QUERY_LENGTH: usize = 250;
const MAXIMUM_EXCERPT_LENGTH: usize = 280;

impl ConceptLibrary<'_> {
    pub fn search(&self, input: LibraryQuery) -> LibraryResult<LibraryPage> {
        let query = input.query.trim();

        if query.chars().count() > MAXIMUM_QUERY_LENGTH {
            return Err(LibraryError::ValueTooLong {
                field: "Search",
                maximum: MAXIMUM_QUERY_LENGTH,
            });
        }

        // Quote user input so FTS operators and punctuation cannot change the query grammar
        let expression = query
            .split(|character: char| character.is_whitespace() || character == '\0')
            .filter(|term| !term.is_empty())
            .map(|term| format!("\"{}\"*", term.replace('"', "\"\"")))
            .collect::<Vec<_>>()
            .join(" AND ");

        self.store
            .read_result(|connection| query_page(connection, &input, &expression))
    }
}

fn query_page(
    connection: &Connection,
    input: &LibraryQuery,
    expression: &str,
) -> LibraryResult<LibraryPage> {
    let searching = !expression.is_empty();
    let now = current_timestamp()?;
    let card_type = input.card_type.map(RetrievalFormKind::as_str);
    let state = input.state.map(LibraryCardState::as_str);
    let matching_card = "matching_card.entity_id = (
            SELECT cards.entity_id FROM cards
            INNER JOIN entities AS card_entities ON card_entities.id = cards.entity_id
            INNER JOIN card_scheduling ON card_scheduling.card_id = cards.entity_id
            WHERE cards.concept_id = concepts.entity_id
                AND (?5 IS NOT NULL OR ?6 IS NOT NULL)
                AND card_entities.deleted_at IS NULL
                AND (?5 IS NULL OR cards.retrieval_kind = ?5)
                AND (?6 IS NULL OR card_scheduling.state = ?6 OR (
                    ?6 = 'due' AND card_scheduling.due_at <= ?7
                    AND concepts.archived_at IS NULL
                ))
            ORDER BY card_entities.created_at, cards.entity_id
            LIMIT 1
        )";
    let search_join = if searching {
        "INNER JOIN concept_search ON concept_search.rowid = concepts.rowid"
    } else {
        ""
    };
    let search_condition = if searching {
        "AND concept_search MATCH ?4"
    } else {
        "AND ?4 = ''"
    };
    let from = format!(
        "FROM concepts
        INNER JOIN entities ON entities.id = concepts.entity_id
        {search_join}
        LEFT JOIN cards AS matching_card ON {matching_card}
        LEFT JOIN templates ON templates.entity_id = matching_card.template_id
        WHERE entities.deleted_at IS NULL
            AND (?1 OR concepts.archived_at IS NULL)
            AND (?2 IS NULL OR EXISTS (
                SELECT 1 FROM concept_decks
                INNER JOIN entities AS decks ON decks.id = concept_decks.deck_id
                WHERE concept_decks.concept_id = concepts.entity_id
                    AND concept_decks.deck_id = ?2
                    AND concept_decks.removed_at IS NULL
                    AND decks.deleted_at IS NULL
            ))
            AND (?3 IS NULL OR EXISTS (
                SELECT 1 FROM concept_tags
                INNER JOIN entities AS tags ON tags.id = concept_tags.tag_id
                WHERE concept_tags.concept_id = concepts.entity_id
                    AND concept_tags.tag_id = ?3
                    AND concept_tags.removed_at IS NULL
                    AND tags.deleted_at IS NULL
            ))
            {search_condition}
            AND ((?5 IS NULL AND ?6 IS NULL) OR matching_card.entity_id IS NOT NULL)"
    );
    let filters = params![
        input.include_archived,
        input.deck_id,
        input.tag_id,
        expression,
        card_type,
        state,
        now
    ];
    let total_count: i64 =
        connection.query_row(&format!("SELECT COUNT(*) {from}"), filters, |row| {
            row.get(0)
        })?;
    let last_page = ((total_count - 1).max(0) / PAGE_SIZE) + 1;
    let page = i64::from(input.page).clamp(1, last_page);
    let ranking = match input.sort {
        LibrarySort::Relevance if searching => "bm25(concept_search, 5.0, 1.0),",
        LibrarySort::Updated => "entities.updated_at DESC,",
        _ => "",
    };
    let excerpt = if searching {
        "snippet(concept_search, 1, '', '', '…', 24)"
    } else {
        "NULL"
    };
    let mut statement = connection.prepare(&format!(
        "SELECT
            concepts.entity_id,
            concepts.title,
            entities.created_at,
            entities.updated_at,
            concepts.archived_at,
            (
                SELECT COUNT(*) FROM cards
                INNER JOIN entities AS card_entities ON card_entities.id = cards.entity_id
                WHERE cards.concept_id = concepts.entity_id
                    AND card_entities.deleted_at IS NULL
            ),
            {excerpt},
            matching_card.entity_id,
            matching_card.retrieval_kind,
            templates.entity_id,
            templates.name
        {from}
        ORDER BY {ranking}
            concepts.archived_at IS NOT NULL,
            concepts.title COLLATE NOCASE,
            concepts.entity_id
        LIMIT ?8 OFFSET ?9"
    ))?;
    let rows = statement.query_map(
        params![
            input.include_archived,
            input.deck_id,
            input.tag_id,
            expression,
            card_type,
            state,
            now,
            PAGE_SIZE,
            (page - 1) * PAGE_SIZE
        ],
        |row| {
            let summary = ConceptSummary {
                id: row.get(0)?,
                title: row.get(1)?,
                created_at: row.get(2)?,
                updated_at: row.get(3)?,
                archived: row.get::<_, Option<i64>>(4)?.is_some(),
                decks: Vec::new(),
                tags: Vec::new(),
                card_count: row.get(5)?,
                excerpt: row.get::<_, Option<String>>(6)?.map(bounded_excerpt),
                matching_form: None,
            };

            Ok((
                summary,
                row.get::<_, Option<String>>(7)?,
                row.get::<_, Option<String>>(8)?,
                row.get::<_, Option<String>>(9)?,
                row.get::<_, Option<String>>(10)?,
            ))
        },
    )?;
    let mut concepts = Vec::new();

    for row in rows {
        let (mut summary, card_id, kind, template_id, template_name) = row?;

        if let (Some(id), Some(kind)) = (card_id, kind) {
            summary.matching_form = Some(LibraryCardMatch {
                id,
                retrieval_kind: RetrievalFormKind::try_from(kind.as_str())?,
                template: template_id
                    .zip(template_name)
                    .map(|(id, name)| NamedItem { id, name }),
            });
        }

        concepts.push(summary);
    }

    attach_assignments(connection, &mut concepts, OrganizationKind::Deck)?;
    attach_assignments(connection, &mut concepts, OrganizationKind::Tag)?;

    let (concept_count, archived_count) = connection.query_row(
        "SELECT COUNT(*) FILTER (WHERE ?1 OR concepts.archived_at IS NULL),
            COUNT(*) FILTER (WHERE concepts.archived_at IS NOT NULL)
        FROM concepts
        INNER JOIN entities ON entities.id = concepts.entity_id
        WHERE entities.deleted_at IS NULL",
        [input.include_archived],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;

    Ok(LibraryPage {
        concepts,
        total_count,
        concept_count,
        archived_count,
        page,
        page_size: PAGE_SIZE,
    })
}

fn bounded_excerpt(text: String) -> String {
    let mut characters = text.trim().chars();
    let mut excerpt: String = characters.by_ref().take(MAXIMUM_EXCERPT_LENGTH).collect();

    if characters.next().is_some() {
        excerpt.push('…');
    }

    excerpt
}

pub(super) fn index_concept(
    transaction: &WriteTransaction<'_>,
    concept: &ConceptDetail,
) -> LibraryResult<()> {
    let mut body = String::new();

    for document in [
        &concept.content.prompt,
        &concept.content.answer,
        &concept.content.feedback.explanation,
        &concept.content.feedback.common_mistakes,
    ] {
        append_document_text(document, &mut body);
        body.push('\n');
    }

    for card in &concept.cards {
        let parts = card
            .type_answer
            .as_ref()
            .map(|settings| &settings.accepted_answers)
            .or_else(|| card.explain.as_ref().map(|settings| &settings.key_points))
            .or_else(|| card.problem.as_ref().map(|settings| &settings.checkpoints));

        if let Some(parts) = parts {
            for part in parts {
                body.push_str(part);
                body.push('\n');
            }
        }
    }

    // The index is a local projection, updated inside the concept's existing write transaction
    transaction.execute(
        "INSERT OR REPLACE INTO concept_search (rowid, title, body)
        SELECT rowid, ?1, ?2 FROM concepts WHERE entity_id = ?3",
        params![concept.title, body, concept.id],
    )?;

    Ok(())
}

fn append_document_text(node: &Value, output: &mut String) {
    let kind = node["type"].as_str().unwrap_or_default();

    if kind == "text" {
        output.push_str(node["text"].as_str().unwrap_or_default());
        return;
    }

    if matches!(kind, "inlineMath" | "blockMath") {
        output.push(' ');
        output.push_str(node["attrs"]["latex"].as_str().unwrap_or_default());
        output.push(' ');
    }

    if kind == "mediaImage" {
        for attribute in ["alt", "title"] {
            output.push_str(node["attrs"][attribute].as_str().unwrap_or_default());
            output.push('\n');
        }
    }

    if let Some(children) = node["content"].as_array() {
        for child in children {
            append_document_text(child, output);
        }
    }

    if !matches!(kind, "inlineMath" | "text") {
        output.push('\n');
    }
}

#[cfg(test)]
mod tests;
