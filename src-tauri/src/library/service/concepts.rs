use std::collections::{BTreeSet, HashSet};

use rusqlite::{params, Connection, OptionalExtension};

use super::assignments::{
    apply_assignments, apply_media_assignments, query_concept_assignments,
};
use super::cards::{
    active_card_ids, apply_retrieval_forms, create_card, normalize_template_ids, query_cards,
    validate_retrieval_form_selection, validate_template_selections,
};
use super::organizations::{normalize_ids, validate_selections, OrganizationKind};
use super::search::index_concept;
use super::{normalize_value, ConceptLibrary};
use crate::data::{EntityKind, WriteTransaction};
use crate::library::content::validate_content;
use crate::library::media::{active_concept_media_ids, query_concept_media, validate_media_ids};
use crate::library::retrieval_forms::{
    normalize_explain, normalize_problem, normalize_type_answer, RetrievalFormConfiguration,
};
use crate::library::{
    ClozeSettings, ConceptDetail, CreateConceptInput, ImageOcclusionSettings, LibraryError,
    LibraryResult, NamedItem, RetrievalFormKind, UpdateConceptInput,
};

const MAXIMUM_CONCEPT_TITLE_LENGTH: usize = 200;

impl ConceptLibrary<'_> {
    pub fn create_concept(&self, input: CreateConceptInput) -> LibraryResult<ConceptDetail> {
        self.store
            .write_result(|transaction| create_concept(transaction, input))
    }

    pub fn update_concept(&self, input: UpdateConceptInput) -> LibraryResult<ConceptDetail> {
        self.store
            .write_result(|transaction| update_concept(transaction, input))
    }

    pub fn set_concept_archived(&self, id: &str, archived: bool) -> LibraryResult<ConceptDetail> {
        let id = id.trim().to_owned();

        self.store.write_result(|transaction| {
            let current = query_concept(transaction, &id)?;

            if current.archived == archived {
                return Ok(current);
            }

            let entity = transaction.touch_entity(&id)?;
            let archived_at = archived.then_some(entity.updated_at);

            transaction.execute(
                "UPDATE concepts
                SET archived_at = ?1,
                    last_change_id = ?2
                WHERE entity_id = ?3",
                params![archived_at, entity.last_change_id, id],
            )?;

            query_concept(transaction, &id)
        })
    }

    pub fn delete_concept(&self, id: &str) -> LibraryResult<()> {
        let id = id.trim().to_owned();

        self.store.write_result(|transaction| {
            if !concept_record_exists(transaction, &id)? {
                return Err(LibraryError::ConceptNotFound(id));
            }

            for card_id in active_card_ids(transaction, &id)? {
                transaction.soft_delete_entity(&card_id)?;
            }

            transaction.soft_delete_entity(&id)?;

            Ok(())
        })
    }
}

pub(in crate::library) fn create_concept(
    transaction: &WriteTransaction<'_>,
    input: CreateConceptInput,
) -> LibraryResult<ConceptDetail> {
    let title = normalize_value(input.title, "Concept title", MAXIMUM_CONCEPT_TITLE_LENGTH)?;
    let deck_ids = normalize_ids(input.deck_ids, "deck")?;
    let tag_ids = normalize_ids(input.tag_ids, "tag")?;
    let content = validate_content(input.content)?;
    let template_ids = normalize_template_ids(input.template_ids)?;
    let explain = input.explain.map(normalize_explain).transpose()?;
    let problem = input.problem.map(normalize_problem).transpose()?;
    let type_answer = input.type_answer.map(normalize_type_answer).transpose()?;

    if problem.is_some() && !content.prompt_has_content {
        return Err(LibraryError::MissingProblemPrompt);
    }

    validate_retrieval_form_selection(
        input.include_standard_recall,
        &template_ids,
        explain.is_some(),
        problem.is_some(),
        type_answer.is_some(),
        !content.cloze_group_ids.is_empty(),
        !content.image_occlusion_group_ids.is_empty(),
    )?;

    validate_selections(transaction, OrganizationKind::Deck, &deck_ids)?;
    validate_selections(transaction, OrganizationKind::Tag, &tag_ids)?;
    validate_media_ids(transaction, &content.media_ids)?;
    validate_template_selections(transaction, &template_ids)?;

    let entity = transaction.create_entity(EntityKind::Concept)?;

    transaction.execute(
        "INSERT INTO concepts (
            entity_id,
            title,
            archived_at,
            last_change_id,
            content_json
        ) VALUES (?1, ?2, NULL, ?3, ?4)",
        params![entity.id, title, entity.last_change_id, content.serialized],
    )?;

    if input.include_standard_recall {
        create_card(
            transaction,
            &entity.id,
            None,
            RetrievalFormConfiguration::Recall,
        )?;
    }

    if let Some(settings) = &type_answer {
        create_card(
            transaction,
            &entity.id,
            None,
            RetrievalFormConfiguration::TypeAnswer(settings.clone()),
        )?;
    }

    if let Some(settings) = &explain {
        create_card(
            transaction,
            &entity.id,
            None,
            RetrievalFormConfiguration::Explain(settings.clone()),
        )?;
    }

    if let Some(settings) = &problem {
        create_card(
            transaction,
            &entity.id,
            None,
            RetrievalFormConfiguration::Problem(settings.clone()),
        )?;
    }

    for group_id in &content.cloze_group_ids {
        create_card(
            transaction,
            &entity.id,
            None,
            RetrievalFormConfiguration::Cloze(ClozeSettings {
                group_id: group_id.clone(),
            }),
        )?;
    }

    for group_id in &content.image_occlusion_group_ids {
        create_card(
            transaction,
            &entity.id,
            None,
            RetrievalFormConfiguration::ImageOcclusion(ImageOcclusionSettings {
                group_id: group_id.clone(),
            }),
        )?;
    }

    for template_id in &template_ids {
        create_card(
            transaction,
            &entity.id,
            Some(template_id),
            RetrievalFormConfiguration::Recall,
        )?;
    }

    apply_assignments(
        transaction,
        OrganizationKind::Deck,
        &entity,
        &HashSet::new(),
        &deck_ids,
    )?;
    apply_assignments(
        transaction,
        OrganizationKind::Tag,
        &entity,
        &HashSet::new(),
        &tag_ids,
    )?;
    apply_media_assignments(transaction, &entity, &HashSet::new(), &content.media_ids)?;

    let concept = query_concept(transaction, &entity.id)?;

    index_concept(transaction, &concept)?;

    Ok(concept)
}

pub(in crate::library) fn update_concept(
    transaction: &WriteTransaction<'_>,
    input: UpdateConceptInput,
) -> LibraryResult<ConceptDetail> {
    let id = input.id.trim().to_owned();
    let title = normalize_value(input.title, "Concept title", MAXIMUM_CONCEPT_TITLE_LENGTH)?;
    let deck_ids = normalize_ids(input.deck_ids, "deck")?;
    let tag_ids = normalize_ids(input.tag_ids, "tag")?;
    let content = validate_content(input.content)?;
    let template_ids = normalize_template_ids(input.template_ids)?;
    let explain = input.explain.map(normalize_explain).transpose()?;
    let problem = input.problem.map(normalize_problem).transpose()?;
    let type_answer = input.type_answer.map(normalize_type_answer).transpose()?;

    if problem.is_some() && !content.prompt_has_content {
        return Err(LibraryError::MissingProblemPrompt);
    }

    validate_retrieval_form_selection(
        input.include_standard_recall,
        &template_ids,
        explain.is_some(),
        problem.is_some(),
        type_answer.is_some(),
        !content.cloze_group_ids.is_empty(),
        !content.image_occlusion_group_ids.is_empty(),
    )?;

    let current = query_concept(transaction, &id)?;

    validate_selections(transaction, OrganizationKind::Deck, &deck_ids)?;
    validate_selections(transaction, OrganizationKind::Tag, &tag_ids)?;
    validate_media_ids(transaction, &content.media_ids)?;
    validate_template_selections(transaction, &template_ids)?;

    let current_decks = item_ids(&current.decks);
    let current_tags = item_ids(&current.tags);
    let current_media = active_concept_media_ids(transaction, &id)?;
    let current_include_standard_recall = current
        .cards
        .iter()
        .any(|card| card.retrieval_kind == RetrievalFormKind::Recall && card.template.is_none());
    let current_type_answer = current
        .cards
        .iter()
        .find_map(|card| card.type_answer.clone());
    let current_explain = current.cards.iter().find_map(|card| card.explain.clone());
    let current_problem = current.cards.iter().find_map(|card| card.problem.clone());
    let current_cloze_group_ids = current
        .cards
        .iter()
        .filter_map(|card| card.cloze.as_ref().map(|cloze| cloze.group_id.clone()))
        .collect::<BTreeSet<_>>();
    let cloze_group_ids = content
        .cloze_group_ids
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    let current_image_occlusion_group_ids = current
        .cards
        .iter()
        .filter_map(|card| {
            card.image_occlusion
                .as_ref()
                .map(|occlusion| occlusion.group_id.clone())
        })
        .collect::<BTreeSet<_>>();
    let image_occlusion_group_ids = content
        .image_occlusion_group_ids
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    let current_template_ids = current
        .cards
        .iter()
        .filter_map(|card| card.template.as_ref().map(|template| template.id.clone()))
        .collect::<BTreeSet<_>>();

    if current.title == title
        && current_decks == deck_ids
        && current_tags == tag_ids
        && current.content == content.content
        && current_media == content.media_ids
        && current_include_standard_recall == input.include_standard_recall
        && current_explain == explain
        && current_problem == problem
        && current_type_answer == type_answer
        && current_cloze_group_ids == cloze_group_ids
        && current_image_occlusion_group_ids == image_occlusion_group_ids
        && current_template_ids == template_ids
    {
        return Ok(current);
    }

    let entity = transaction.touch_entity(&id)?;

    transaction.execute(
        "UPDATE concepts
        SET title = ?1,
            content_json = ?2,
            last_change_id = ?3
        WHERE entity_id = ?4",
        params![title, content.serialized, entity.last_change_id, id],
    )?;

    apply_assignments(
        transaction,
        OrganizationKind::Deck,
        &entity,
        &current_decks,
        &deck_ids,
    )?;
    apply_assignments(
        transaction,
        OrganizationKind::Tag,
        &entity,
        &current_tags,
        &tag_ids,
    )?;
    apply_media_assignments(transaction, &entity, &current_media, &content.media_ids)?;
    apply_retrieval_forms(
        transaction,
        &id,
        &current.cards,
        input.include_standard_recall,
        &template_ids,
        explain.as_ref(),
        problem.as_ref(),
        type_answer.as_ref(),
        &cloze_group_ids,
        &image_occlusion_group_ids,
    )?;

    let concept = query_concept(transaction, &id)?;

    index_concept(transaction, &concept)?;

    Ok(concept)
}

fn item_ids(items: &[NamedItem]) -> HashSet<String> {
    items.iter().map(|item| item.id.clone()).collect()
}

pub(super) fn query_concept(connection: &Connection, id: &str) -> LibraryResult<ConceptDetail> {
    let concept = connection
        .query_row(
            "SELECT
                concepts.entity_id,
                concepts.title,
                entities.created_at,
                entities.updated_at,
                entities.last_change_id,
                concepts.archived_at,
                concepts.content_json
            FROM concepts
            INNER JOIN entities ON entities.id = concepts.entity_id
            WHERE concepts.entity_id = ?1
                AND entities.deleted_at IS NULL",
            [id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, Option<i64>>(5)?,
                    row.get::<_, String>(6)?,
                ))
            },
        )
        .optional()?;
    let Some((id, title, created_at, updated_at, last_change_id, archived_at, content)) = concept
    else {
        return Err(LibraryError::ConceptNotFound(id.to_owned()));
    };

    Ok(ConceptDetail {
        decks: query_concept_assignments(connection, &id, OrganizationKind::Deck)?,
        tags: query_concept_assignments(connection, &id, OrganizationKind::Tag)?,
        cards: query_cards(connection, &id)?,
        content: serde_json::from_str(&content)?,
        media: query_concept_media(connection, &id)?,
        id,
        title,
        created_at,
        updated_at,
        last_change_id,
        archived: archived_at.is_some(),
    })
}

fn concept_record_exists(connection: &Connection, id: &str) -> LibraryResult<bool> {
    Ok(connection.query_row(
        "SELECT EXISTS (SELECT 1 FROM concepts WHERE entity_id = ?1)",
        [id],
        |row| row.get(0),
    )?)
}
