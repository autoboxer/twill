use std::collections::{BTreeMap, BTreeSet};

use rusqlite::{named_params, params_from_iter, Connection};

use crate::library::media::query_media_for_concepts;
use crate::library::mixed_practice::mix_due_cards;
use crate::library::models::TemplateMode;
use crate::library::retrieval_forms::parse_retrieval_form_configuration;
use crate::library::search_query::search_expression;
use crate::library::{
    LibraryError, LibraryResult, RetrievalFormKind, SchedulingState, StudyCard, StudyConcept,
    StudyQuery, StudyQueue, StudyTemplate,
};

const ID_BATCH_SIZE: usize = 500;

pub fn query_study_queue(connection: &Connection, now: i64) -> LibraryResult<StudyQueue> {
    query_selected_study_queue(connection, now, &StudyQuery::default())
}

pub fn query_selected_study_queue(
    connection: &Connection,
    now: i64,
    input: &StudyQuery,
) -> LibraryResult<StudyQueue> {
    if input.card_limit == Some(0) {
        return Err(LibraryError::InvalidContent {
            field: "Card limit",
            message: "must be a positive whole number".into(),
        });
    }

    let expression = search_expression(&input.query)?;
    let card_type = input.card_type.map(RetrievalFormKind::as_str);
    let state = input.state.map(SchedulingState::as_str);
    let mixed_practice_enabled = connection.query_row(
        "SELECT mixed_practice_enabled
        FROM device_preferences
        WHERE singleton = 1",
        [],
        |row| row.get(0),
    )?;
    let scope = "FROM cards
        INNER JOIN card_scheduling ON card_scheduling.card_id = cards.entity_id
        INNER JOIN entities AS card_entities ON card_entities.id = cards.entity_id
        INNER JOIN concepts ON concepts.entity_id = cards.concept_id
        INNER JOIN entities AS concept_entities ON concept_entities.id = concepts.entity_id
        LEFT JOIN entities AS template_entities ON template_entities.id = cards.template_id
        WHERE card_entities.deleted_at IS NULL
            AND concept_entities.deleted_at IS NULL
            AND concepts.archived_at IS NULL
            AND (cards.template_id IS NULL OR template_entities.deleted_at IS NULL)
            AND (:deck IS NULL OR EXISTS (
                SELECT 1 FROM concept_decks
                INNER JOIN entities AS decks ON decks.id = concept_decks.deck_id
                WHERE concept_decks.concept_id = concepts.entity_id
                    AND concept_decks.deck_id = :deck
                    AND concept_decks.removed_at IS NULL
                    AND decks.deleted_at IS NULL
            ))
            AND (:tag IS NULL OR EXISTS (
                SELECT 1 FROM concept_tags
                INNER JOIN entities AS tags ON tags.id = concept_tags.tag_id
                WHERE concept_tags.concept_id = concepts.entity_id
                    AND concept_tags.tag_id = :tag
                    AND concept_tags.removed_at IS NULL
                    AND tags.deleted_at IS NULL
            ))
            AND (:search = '' OR concepts.rowid IN (
                SELECT rowid FROM concept_search WHERE concept_search MATCH :search
            ))
            AND (:kind IS NULL OR cards.retrieval_kind = :kind)
            AND (:state IS NULL OR card_scheduling.state = :state)";
    let filters = named_params! {
        ":deck": input.deck_id,
        ":tag": input.tag_id,
        ":search": expression,
        ":kind": card_type,
        ":state": state,
        ":now": now,
    };
    let (total_cards, due_cards, next_due_at) = connection.query_row(
        &format!(
            "SELECT COUNT(*),
                COUNT(*) FILTER (WHERE card_scheduling.due_at <= :now),
                MIN(card_scheduling.due_at) FILTER (WHERE card_scheduling.due_at > :now)
            {scope}"
        ),
        filters,
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    )?;
    let mut statement = connection.prepare(&format!(
        "SELECT
            cards.entity_id,
            concepts.entity_id,
            cards.retrieval_kind,
            cards.configuration_json,
            cards.template_id,
            card_scheduling.state,
            card_scheduling.due_at,
            card_scheduling.state = 'new'
                AND NOT EXISTS (
                    SELECT 1 FROM reviews
                    INNER JOIN cards AS reviewed_cards ON reviewed_cards.entity_id = reviews.card_id
                    WHERE reviewed_cards.concept_id = concepts.entity_id
                )
                AND NOT EXISTS (
                    SELECT 1 FROM pretests WHERE pretests.concept_id = concepts.entity_id
                ) AS pretest_eligible
        {scope}
            AND card_scheduling.due_at <= :now
        ORDER BY card_scheduling.due_at, cards.entity_id
        LIMIT :limit"
    ))?;
    let rows = statement.query_map(
        named_params! {
            ":deck": input.deck_id,
            ":tag": input.tag_id,
            ":search": expression,
            ":kind": card_type,
            ":state": state,
            ":now": now,
            ":limit": input.card_limit.map(i64::from).unwrap_or(-1),
        },
        |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, Option<String>>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, i64>(6)?,
                row.get::<_, bool>(7)?,
            ))
        },
    )?;
    let card_rows = rows.collect::<Result<Vec<_>, _>>()?;

    drop(statement);

    let cards = card_rows
        .into_iter()
        .map(|row| {
            let (
                id,
                concept_id,
                retrieval_kind,
                configuration,
                template_id,
                state,
                due_at,
                pretest_eligible,
            ) = row;
            let retrieval_kind = RetrievalFormKind::try_from(retrieval_kind.as_str())?;
            let parsed = parse_retrieval_form_configuration(retrieval_kind, &configuration)?;

            Ok(StudyCard {
                id,
                concept_id,
                retrieval_kind,
                explain: parsed.explain().cloned(),
                problem: parsed.problem().cloned(),
                cloze: parsed.cloze().cloned(),
                image_occlusion: parsed.image_occlusion().cloned(),
                type_answer: parsed.type_answer().cloned(),
                template_id,
                scheduling_state: SchedulingState::try_from(state.as_str())?,
                due_at,
                pretest_eligible,
            })
        })
        .collect::<LibraryResult<Vec<_>>>()?;
    let concept_ids = cards.iter().map(|card| card.concept_id.clone()).collect();
    let template_ids = cards
        .iter()
        .filter_map(|card| card.template_id.clone())
        .collect();
    let concepts = query_concepts(connection, &concept_ids)?;
    let templates = query_templates(connection, &template_ids)?;

    if concepts.len() != concept_ids.len() || templates.len() != template_ids.len() {
        return Err(LibraryError::InvalidRetrievalForm);
    }

    let cards = if mixed_practice_enabled {
        let concept_tags = query_active_concept_tags(connection, &concept_ids)?;

        mix_due_cards(cards, &concept_tags)
    } else {
        cards
    };
    let custom_template_ids = templates
        .iter()
        .filter(|template| template.content.mode == TemplateMode::Custom)
        .map(|template| &template.id)
        .collect::<BTreeSet<_>>();
    let media_concept_ids = cards
        .iter()
        .filter(|card| {
            card.template_id
                .as_ref()
                .is_some_and(|id| custom_template_ids.contains(id))
                || matches!(
                    card.retrieval_kind,
                    RetrievalFormKind::Problem | RetrievalFormKind::ImageOcclusion
                )
        })
        .map(|card| card.concept_id.clone())
        .collect::<BTreeSet<_>>();
    let mut media = BTreeMap::new();

    for batch in media_concept_ids
        .into_iter()
        .collect::<Vec<_>>()
        .chunks(ID_BATCH_SIZE)
    {
        for item in query_media_for_concepts(connection, &batch.iter().cloned().collect())? {
            media.insert(item.id.clone(), item);
        }
    }

    Ok(StudyQueue {
        cards,
        concepts,
        templates,
        media: media.into_values().collect(),
        next_due_at,
        total_cards,
        due_cards,
        mixed_practice_enabled,
    })
}

fn query_active_concept_tags(
    connection: &Connection,
    concept_ids: &BTreeSet<String>,
) -> LibraryResult<BTreeMap<String, BTreeSet<String>>> {
    let mut concept_tags = BTreeMap::<String, BTreeSet<String>>::new();

    for batch in concept_ids.iter().collect::<Vec<_>>().chunks(ID_BATCH_SIZE) {
        let placeholders = vec!["?"; batch.len()].join(", ");
        let mut statement = connection.prepare(&format!(
            "SELECT
            concept_tags.concept_id,
            concept_tags.tag_id
        FROM concept_tags
        INNER JOIN entities AS tag_entities
            ON tag_entities.id = concept_tags.tag_id
        WHERE concept_tags.removed_at IS NULL
            AND concept_tags.concept_id IN ({placeholders})
            AND tag_entities.deleted_at IS NULL
        ORDER BY concept_tags.concept_id, concept_tags.tag_id",
        ))?;
        let rows = statement.query_map(params_from_iter(batch), |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;

        for row in rows {
            let (concept_id, tag_id) = row?;

            concept_tags.entry(concept_id).or_default().insert(tag_id);
        }
    }

    Ok(concept_tags)
}

fn query_concepts(
    connection: &Connection,
    ids: &BTreeSet<String>,
) -> LibraryResult<Vec<StudyConcept>> {
    let mut concepts = Vec::with_capacity(ids.len());

    for batch in ids.iter().collect::<Vec<_>>().chunks(ID_BATCH_SIZE) {
        let placeholders = vec!["?"; batch.len()].join(", ");
        let mut statement = connection.prepare(&format!(
            "SELECT entity_id, last_change_id, title, content_json
            FROM concepts WHERE entity_id IN ({placeholders})
            ORDER BY entity_id"
        ))?;
        let rows = statement.query_map(params_from_iter(batch), |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get::<_, String>(3)?,
            ))
        })?;

        for row in rows {
            let (id, last_change_id, title, content) = row?;

            concepts.push(StudyConcept {
                id,
                last_change_id,
                title,
                content: serde_json::from_str(&content)?,
            });
        }
    }

    Ok(concepts)
}

fn query_templates(
    connection: &Connection,
    ids: &BTreeSet<String>,
) -> LibraryResult<Vec<StudyTemplate>> {
    let mut templates = Vec::with_capacity(ids.len());

    for batch in ids.iter().collect::<Vec<_>>().chunks(ID_BATCH_SIZE) {
        let placeholders = vec!["?"; batch.len()].join(", ");
        let mut statement = connection.prepare(&format!(
            "SELECT entity_id, name, content_json
            FROM templates WHERE entity_id IN ({placeholders})
            ORDER BY entity_id"
        ))?;
        let rows = statement.query_map(params_from_iter(batch), |row| {
            Ok((row.get(0)?, row.get(1)?, row.get::<_, String>(2)?))
        })?;

        for row in rows {
            let (id, name, content) = row?;

            templates.push(StudyTemplate {
                id,
                name,
                content: serde_json::from_str(&content)?,
            });
        }
    }

    Ok(templates)
}

#[cfg(test)]
mod tests;
