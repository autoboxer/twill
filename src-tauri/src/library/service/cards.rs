use std::collections::{BTreeSet, HashSet};

use rusqlite::{params, Connection};

use crate::data::{EntityKind, WriteTransaction};
use crate::library::retrieval_forms::{
    parse_retrieval_form_configuration, RetrievalFormConfiguration,
};
use crate::library::{
    CardSummary, ClozeSettings, ExplainSettings, ImageOcclusionSettings, LibraryError,
    LibraryResult, NamedItem, ProblemSettings, RetrievalFormKind, SchedulingState,
    TypeAnswerSettings,
};

pub(super) fn create_card(
    transaction: &WriteTransaction<'_>,
    concept_id: &str,
    template_id: Option<&str>,
    form: RetrievalFormConfiguration,
) -> LibraryResult<()> {
    let configuration = form.to_json()?;
    let entity = transaction.create_entity(EntityKind::Card)?;

    transaction.execute(
        "INSERT INTO cards (
            entity_id,
            concept_id,
            retrieval_kind,
            configuration_json,
            template_id,
            last_change_id
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            entity.id,
            concept_id,
            form.kind().as_str(),
            configuration,
            template_id,
            entity.last_change_id
        ],
    )?;
    transaction.execute(
        "INSERT INTO card_scheduling (
            card_id,
            state,
            due_at,
            stability,
            difficulty,
            last_reviewed_at,
            last_review_id,
            review_count,
            lapse_count
        ) VALUES (?1, 'new', ?2, NULL, NULL, NULL, NULL, 0, 0)",
        params![entity.id, entity.created_at],
    )?;

    Ok(())
}

pub(super) fn normalize_template_ids(ids: Vec<String>) -> LibraryResult<BTreeSet<String>> {
    ids.into_iter()
        .map(|id| {
            let id = id.trim().to_owned();

            if id.is_empty() {
                return Err(LibraryError::InvalidSelection {
                    kind: "template",
                    id,
                });
            }

            Ok(id)
        })
        .collect()
}

pub(super) fn validate_retrieval_form_selection(
    include_standard_recall: bool,
    template_ids: &BTreeSet<String>,
    include_explain: bool,
    include_problem: bool,
    include_type_answer: bool,
    include_cloze: bool,
    include_image_occlusion: bool,
) -> LibraryResult<()> {
    if !include_standard_recall
        && !include_explain
        && !include_problem
        && !include_type_answer
        && !include_cloze
        && !include_image_occlusion
        && template_ids.is_empty()
    {
        return Err(LibraryError::MissingRetrievalForm);
    }

    Ok(())
}

pub(super) fn validate_template_selections(
    connection: &Connection,
    ids: &BTreeSet<String>,
) -> LibraryResult<()> {
    for id in ids {
        let exists: bool = connection.query_row(
            "SELECT EXISTS (
                SELECT 1
                FROM templates
                INNER JOIN entities ON entities.id = templates.entity_id
                WHERE templates.entity_id = ?1
                    AND entities.deleted_at IS NULL
            )",
            [id],
            |row| row.get(0),
        )?;

        if !exists {
            return Err(LibraryError::InvalidSelection {
                kind: "template",
                id: id.clone(),
            });
        }
    }

    Ok(())
}

pub(super) fn query_cards(
    connection: &Connection,
    concept_id: &str,
) -> LibraryResult<Vec<CardSummary>> {
    let mut statement = connection.prepare(
        "SELECT
            cards.entity_id,
            cards.retrieval_kind,
            cards.configuration_json,
            templates.entity_id,
            templates.name,
            card_scheduling.state,
            card_scheduling.due_at,
            card_scheduling.review_count,
            card_scheduling.lapse_count
        FROM cards
        INNER JOIN entities AS card_entities
            ON card_entities.id = cards.entity_id
        INNER JOIN card_scheduling
            ON card_scheduling.card_id = cards.entity_id
        LEFT JOIN templates
            ON templates.entity_id = cards.template_id
        LEFT JOIN entities AS template_entities
            ON template_entities.id = cards.template_id
        WHERE cards.concept_id = ?1
            AND card_entities.deleted_at IS NULL
            AND (
                cards.template_id IS NULL
                OR template_entities.deleted_at IS NULL
            )
        ORDER BY
            cards.template_id IS NOT NULL,
            templates.name COLLATE NOCASE,
            card_entities.created_at,
            cards.entity_id",
    )?;
    let cards = statement.query_map([concept_id], |row| {
        let template_id = row.get::<_, Option<String>>(3)?;
        let template_name = row.get::<_, Option<String>>(4)?;
        let template = match (template_id, template_name) {
            (Some(id), Some(name)) => Some(NamedItem { id, name }),
            (None, None) => None,
            _ => return Err(rusqlite::Error::InvalidQuery),
        };
        let retrieval_kind = row.get::<_, String>(1)?;
        let configuration = row.get::<_, String>(2)?;
        let scheduling_state = row.get::<_, String>(5)?;

        Ok((
            row.get::<_, String>(0)?,
            retrieval_kind,
            configuration,
            template,
            scheduling_state,
            row.get::<_, i64>(6)?,
            row.get::<_, i64>(7)?,
            row.get::<_, i64>(8)?,
        ))
    })?;

    cards
        .map(|card| {
            let (
                id,
                retrieval_kind,
                configuration,
                template,
                scheduling_state,
                due_at,
                review_count,
                lapse_count,
            ) = card?;
            let retrieval_kind = RetrievalFormKind::try_from(retrieval_kind.as_str())?;
            let parsed = parse_retrieval_form_configuration(retrieval_kind, &configuration)?;

            Ok(CardSummary {
                id,
                retrieval_kind,
                explain: parsed.explain().cloned(),
                problem: parsed.problem().cloned(),
                cloze: parsed.cloze().cloned(),
                image_occlusion: parsed.image_occlusion().cloned(),
                type_answer: parsed.type_answer().cloned(),
                template,
                scheduling_state: SchedulingState::try_from(scheduling_state.as_str())?,
                due_at,
                review_count,
                lapse_count,
            })
        })
        .collect()
}

pub(super) fn active_card_ids(
    connection: &Connection,
    concept_id: &str,
) -> LibraryResult<Vec<String>> {
    let mut statement = connection.prepare(
        "SELECT cards.entity_id
        FROM cards
        INNER JOIN entities ON entities.id = cards.entity_id
        WHERE cards.concept_id = ?1
            AND entities.deleted_at IS NULL",
    )?;
    let ids = statement.query_map([concept_id], |row| row.get(0))?;

    Ok(ids.collect::<Result<_, _>>()?)
}

pub(super) fn apply_retrieval_forms(
    transaction: &WriteTransaction<'_>,
    concept_id: &str,
    current_cards: &[CardSummary],
    include_standard_recall: bool,
    template_ids: &BTreeSet<String>,
    explain: Option<&ExplainSettings>,
    problem: Option<&ProblemSettings>,
    type_answer: Option<&TypeAnswerSettings>,
    cloze_group_ids: &BTreeSet<String>,
    image_occlusion_group_ids: &BTreeSet<String>,
) -> LibraryResult<()> {
    for card in current_cards {
        let retained = match card.retrieval_kind {
            RetrievalFormKind::Recall => match &card.template {
                Some(template) => template_ids.contains(&template.id),
                None => include_standard_recall,
            },
            RetrievalFormKind::TypeAnswer => type_answer.is_some(),
            RetrievalFormKind::Explain => explain.is_some(),
            RetrievalFormKind::Problem => problem.is_some(),
            RetrievalFormKind::Cloze => card
                .cloze
                .as_ref()
                .is_some_and(|cloze| cloze_group_ids.contains(&cloze.group_id)),
            RetrievalFormKind::ImageOcclusion => card
                .image_occlusion
                .as_ref()
                .is_some_and(|occlusion| image_occlusion_group_ids.contains(&occlusion.group_id)),
        };

        if !retained {
            transaction.soft_delete_entity(&card.id)?;
        }
    }

    let has_standard_recall = current_cards
        .iter()
        .any(|card| card.retrieval_kind == RetrievalFormKind::Recall && card.template.is_none());

    if include_standard_recall && !has_standard_recall {
        create_card(
            transaction,
            concept_id,
            None,
            RetrievalFormConfiguration::Recall,
        )?;
    }

    let current_explain = current_cards
        .iter()
        .find(|card| card.retrieval_kind == RetrievalFormKind::Explain);

    match (current_explain, explain) {
        (None, Some(settings)) => {
            create_card(
                transaction,
                concept_id,
                None,
                RetrievalFormConfiguration::Explain(settings.clone()),
            )?;
        }
        (Some(card), Some(settings)) if card.explain.as_ref() != Some(settings) => {
            let configuration = RetrievalFormConfiguration::Explain(settings.clone()).to_json()?;
            let entity = transaction.touch_entity(&card.id)?;

            transaction.execute(
                "UPDATE cards
                SET configuration_json = ?1,
                    last_change_id = ?2
                WHERE entity_id = ?3",
                params![configuration, entity.last_change_id, card.id],
            )?;
        }
        _ => {}
    }

    let current_problem = current_cards
        .iter()
        .find(|card| card.retrieval_kind == RetrievalFormKind::Problem);

    match (current_problem, problem) {
        (None, Some(settings)) => {
            create_card(
                transaction,
                concept_id,
                None,
                RetrievalFormConfiguration::Problem(settings.clone()),
            )?;
        }
        (Some(card), Some(settings)) if card.problem.as_ref() != Some(settings) => {
            let configuration = RetrievalFormConfiguration::Problem(settings.clone()).to_json()?;
            let entity = transaction.touch_entity(&card.id)?;

            transaction.execute(
                "UPDATE cards
                SET configuration_json = ?1,
                    last_change_id = ?2
                WHERE entity_id = ?3",
                params![configuration, entity.last_change_id, card.id],
            )?;
        }
        _ => {}
    }

    let current_type_answer = current_cards
        .iter()
        .find(|card| card.retrieval_kind == RetrievalFormKind::TypeAnswer);

    match (current_type_answer, type_answer) {
        (None, Some(settings)) => {
            create_card(
                transaction,
                concept_id,
                None,
                RetrievalFormConfiguration::TypeAnswer(settings.clone()),
            )?;
        }
        (Some(card), Some(settings)) if card.type_answer.as_ref() != Some(settings) => {
            let configuration =
                RetrievalFormConfiguration::TypeAnswer(settings.clone()).to_json()?;
            let entity = transaction.touch_entity(&card.id)?;

            transaction.execute(
                "UPDATE cards
                SET configuration_json = ?1,
                    last_change_id = ?2
                WHERE entity_id = ?3",
                params![configuration, entity.last_change_id, card.id],
            )?;
        }
        _ => {}
    }

    let current_cloze_group_ids = current_cards
        .iter()
        .filter_map(|card| card.cloze.as_ref().map(|cloze| cloze.group_id.as_str()))
        .collect::<HashSet<_>>();

    for group_id in cloze_group_ids {
        if !current_cloze_group_ids.contains(group_id.as_str()) {
            create_card(
                transaction,
                concept_id,
                None,
                RetrievalFormConfiguration::Cloze(ClozeSettings {
                    group_id: group_id.clone(),
                }),
            )?;
        }
    }

    let current_image_occlusion_group_ids = current_cards
        .iter()
        .filter_map(|card| {
            card.image_occlusion
                .as_ref()
                .map(|occlusion| occlusion.group_id.as_str())
        })
        .collect::<HashSet<_>>();

    for group_id in image_occlusion_group_ids {
        if !current_image_occlusion_group_ids.contains(group_id.as_str()) {
            create_card(
                transaction,
                concept_id,
                None,
                RetrievalFormConfiguration::ImageOcclusion(ImageOcclusionSettings {
                    group_id: group_id.clone(),
                }),
            )?;
        }
    }

    let current_template_ids = current_cards
        .iter()
        .filter_map(|card| card.template.as_ref().map(|template| template.id.as_str()))
        .collect::<HashSet<_>>();

    for template_id in template_ids {
        if !current_template_ids.contains(template_id.as_str()) {
            create_card(
                transaction,
                concept_id,
                Some(template_id),
                RetrievalFormConfiguration::Recall,
            )?;
        }
    }

    Ok(())
}
