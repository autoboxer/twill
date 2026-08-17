use std::collections::{BTreeSet, HashMap, HashSet};

use rusqlite::{params, Connection, OptionalExtension};

use crate::data::{
    current_timestamp, EntityKind, EntityMetadata, LocalDataStore, WriteTransaction,
};
use crate::library::content::validate_content;
use crate::library::media::{
    active_concept_media_ids, query_concept_media, validate_media_ids,
};
use crate::library::study::{
    create_recall_card, query_scheduling_settings, query_study_preferences,
    query_study_queue, record_review, update_grading_mode, update_scheduling_settings,
};
use crate::library::{
    CardSummary, ConceptDetail, ConceptSummary, CreateConceptInput, GradingMode,
    LibraryError, LibraryResult, LibrarySnapshot, MediaSummary, NamedItem,
    OrganizationSummary, RecordReviewInput, RetrievalFormKind, ReviewOutcome,
    SchedulingSettings, SchedulingState, StudyPreferences, StudyQueue, UpdateConceptInput,
    UpdateSchedulingSettingsInput,
};

const MAXIMUM_CONCEPT_TITLE_LENGTH: usize = 200;
const MAXIMUM_ORGANIZATION_NAME_LENGTH: usize = 80;

pub struct ConceptLibrary<'store> {
    store: &'store LocalDataStore,
}

#[derive(Clone, Copy)]
enum OrganizationKind {
    Deck,
    Tag,
}

impl<'store> ConceptLibrary<'store> {
    pub fn new(store: &'store LocalDataStore) -> Self {
        Self { store }
    }

    pub fn snapshot(&self, include_archived: bool) -> LibraryResult<LibrarySnapshot> {
        self.store
            .read_result(|connection| query_snapshot(connection, include_archived))
    }

    pub fn concept(&self, id: &str) -> LibraryResult<ConceptDetail> {
        self.store
            .read_result(|connection| query_concept(connection, id))
    }

    pub fn study_queue(&self) -> LibraryResult<StudyQueue> {
        self.study_queue_at(current_timestamp()?)
    }

    pub fn record_review(&self, input: RecordReviewInput) -> LibraryResult<ReviewOutcome> {
        self.record_review_at(input, current_timestamp()?)
    }

    pub fn study_preferences(&self) -> LibraryResult<StudyPreferences> {
        self.store.read_result(query_study_preferences)
    }

    pub fn set_grading_mode(
        &self,
        grading_mode: GradingMode,
    ) -> LibraryResult<StudyPreferences> {
        self.store.write_result(|transaction| {
            update_grading_mode(transaction, grading_mode)
        })
    }

    pub fn scheduling_settings(&self) -> LibraryResult<SchedulingSettings> {
        self.store.read_result(query_scheduling_settings)
    }

    pub fn update_scheduling_settings(
        &self,
        input: UpdateSchedulingSettingsInput,
    ) -> LibraryResult<SchedulingSettings> {
        self.store.write_result(|transaction| {
            update_scheduling_settings(transaction, input)
        })
    }

    pub fn import_image(&self, bytes: &[u8]) -> LibraryResult<MediaSummary> {
        crate::library::media::import_image(self.store, bytes)
    }

    pub fn media_bytes(&self, id: &str) -> LibraryResult<Vec<u8>> {
        crate::library::media::read_media(self.store, id)
    }

    fn study_queue_at(&self, now: i64) -> LibraryResult<StudyQueue> {
        self.store
            .read_result(|connection| query_study_queue(connection, now))
    }

    fn record_review_at(
        &self,
        input: RecordReviewInput,
        now: i64,
    ) -> LibraryResult<ReviewOutcome> {
        let card_id = input.card_id.trim().to_owned();

        self.store.write_result(|transaction| {
            record_review(transaction, &card_id, input.rating, now)
        })
    }

    pub fn create_concept(&self, input: CreateConceptInput) -> LibraryResult<ConceptDetail> {
        let title = normalize_value(
            input.title,
            "Concept title",
            MAXIMUM_CONCEPT_TITLE_LENGTH,
        )?;
        let deck_ids = normalize_ids(input.deck_ids, "deck")?;
        let tag_ids = normalize_ids(input.tag_ids, "tag")?;
        let content = validate_content(input.content)?;
        let template_ids = normalize_template_ids(input.template_ids)?;

        validate_retrieval_form_selection(input.include_standard_recall, &template_ids)?;

        self.store.write_result(|transaction| {
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
                params![
                    entity.id,
                    title,
                    entity.last_change_id,
                    content.serialized
                ],
            )?;

            if input.include_standard_recall {
                create_recall_card(transaction, &entity.id, None)?;
            }

            for template_id in &template_ids {
                create_recall_card(transaction, &entity.id, Some(template_id))?;
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
            apply_media_assignments(
                transaction,
                &entity,
                &HashSet::new(),
                &content.media_ids,
            )?;

            query_concept(transaction, &entity.id)
        })
    }

    pub fn update_concept(&self, input: UpdateConceptInput) -> LibraryResult<ConceptDetail> {
        let id = input.id.trim().to_owned();
        let title = normalize_value(
            input.title,
            "Concept title",
            MAXIMUM_CONCEPT_TITLE_LENGTH,
        )?;
        let deck_ids = normalize_ids(input.deck_ids, "deck")?;
        let tag_ids = normalize_ids(input.tag_ids, "tag")?;
        let content = validate_content(input.content)?;
        let template_ids = normalize_template_ids(input.template_ids)?;

        validate_retrieval_form_selection(input.include_standard_recall, &template_ids)?;

        self.store.write_result(|transaction| {
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
                .any(|card| card.template.is_none());
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
            apply_media_assignments(
                transaction,
                &entity,
                &current_media,
                &content.media_ids,
            )?;
            apply_retrieval_forms(
                transaction,
                &id,
                &current.cards,
                input.include_standard_recall,
                &template_ids,
            )?;

            query_concept(transaction, &id)
        })
    }

    pub fn set_concept_archived(
        &self,
        id: &str,
        archived: bool,
    ) -> LibraryResult<ConceptDetail> {
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

    pub fn create_deck(&self, name: String) -> LibraryResult<OrganizationSummary> {
        self.create_organization(OrganizationKind::Deck, name)
    }

    pub fn rename_deck(&self, id: &str, name: String) -> LibraryResult<OrganizationSummary> {
        self.rename_organization(OrganizationKind::Deck, id, name)
    }

    pub fn delete_deck(&self, id: &str) -> LibraryResult<()> {
        self.delete_organization(OrganizationKind::Deck, id)
    }

    pub fn create_tag(&self, name: String) -> LibraryResult<OrganizationSummary> {
        self.create_organization(OrganizationKind::Tag, name)
    }

    pub fn rename_tag(&self, id: &str, name: String) -> LibraryResult<OrganizationSummary> {
        self.rename_organization(OrganizationKind::Tag, id, name)
    }

    pub fn delete_tag(&self, id: &str) -> LibraryResult<()> {
        self.delete_organization(OrganizationKind::Tag, id)
    }

    fn create_organization(
        &self,
        kind: OrganizationKind,
        name: String,
    ) -> LibraryResult<OrganizationSummary> {
        let name = normalize_value(
            name,
            kind.display_name(),
            MAXIMUM_ORGANIZATION_NAME_LENGTH,
        )?;

        self.store.write_result(|transaction| {
            ensure_unique_name(transaction, kind, &name, None)?;

            let entity = transaction.create_entity(kind.entity_kind())?;
            let sql = format!(
                "INSERT INTO {} (entity_id, name, last_change_id)
                VALUES (?1, ?2, ?3)",
                kind.table()
            );

            transaction.execute(&sql, params![entity.id, name, entity.last_change_id])?;

            query_organization(transaction, kind, &entity.id)
        })
    }

    fn rename_organization(
        &self,
        kind: OrganizationKind,
        id: &str,
        name: String,
    ) -> LibraryResult<OrganizationSummary> {
        let id = id.trim().to_owned();
        let name = normalize_value(
            name,
            kind.display_name(),
            MAXIMUM_ORGANIZATION_NAME_LENGTH,
        )?;

        self.store.write_result(|transaction| {
            let current = query_organization(transaction, kind, &id)?;

            if current.name == name {
                return Ok(current);
            }

            ensure_unique_name(transaction, kind, &name, Some(&id))?;

            let entity = transaction.touch_entity(&id)?;
            let sql = format!(
                "UPDATE {}
                SET name = ?1,
                    last_change_id = ?2
                WHERE entity_id = ?3",
                kind.table()
            );

            transaction.execute(&sql, params![name, entity.last_change_id, id])?;

            query_organization(transaction, kind, &id)
        })
    }

    fn delete_organization(&self, kind: OrganizationKind, id: &str) -> LibraryResult<()> {
        let id = id.trim().to_owned();

        self.store.write_result(|transaction| {
            if !organization_record_exists(transaction, kind, &id)? {
                return Err(LibraryError::OrganizationNotFound {
                    kind: kind.noun(),
                    id,
                });
            }

            transaction.soft_delete_entity(&id)?;

            Ok(())
        })
    }
}

impl OrganizationKind {
    const fn display_name(self) -> &'static str {
        match self {
            Self::Deck => "Deck name",
            Self::Tag => "Tag name",
        }
    }

    const fn entity_kind(self) -> EntityKind {
        match self {
            Self::Deck => EntityKind::Deck,
            Self::Tag => EntityKind::Tag,
        }
    }

    const fn noun(self) -> &'static str {
        match self {
            Self::Deck => "deck",
            Self::Tag => "tag",
        }
    }

    const fn table(self) -> &'static str {
        match self {
            Self::Deck => "decks",
            Self::Tag => "tags",
        }
    }

    const fn membership_table(self) -> &'static str {
        match self {
            Self::Deck => "concept_decks",
            Self::Tag => "concept_tags",
        }
    }

    const fn membership_column(self) -> &'static str {
        match self {
            Self::Deck => "deck_id",
            Self::Tag => "tag_id",
        }
    }
}

fn normalize_value(
    value: String,
    field: &'static str,
    maximum: usize,
) -> LibraryResult<String> {
    let value = value.trim().to_owned();

    if value.is_empty() {
        return Err(LibraryError::EmptyValue { field });
    }

    if value.chars().count() > maximum {
        return Err(LibraryError::ValueTooLong { field, maximum });
    }

    Ok(value)
}

fn normalize_ids(ids: Vec<String>, kind: &'static str) -> LibraryResult<HashSet<String>> {
    ids.into_iter()
        .map(|id| {
            let id = id.trim().to_owned();

            if id.is_empty() {
                return Err(LibraryError::InvalidSelection { kind, id });
            }

            Ok(id)
        })
        .collect()
}

fn normalize_template_ids(ids: Vec<String>) -> LibraryResult<BTreeSet<String>> {
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

fn validate_retrieval_form_selection(
    include_standard_recall: bool,
    template_ids: &BTreeSet<String>,
) -> LibraryResult<()> {
    if !include_standard_recall && template_ids.is_empty() {
        return Err(LibraryError::MissingRetrievalForm);
    }

    Ok(())
}

fn item_ids(items: &[NamedItem]) -> HashSet<String> {
    items.iter().map(|item| item.id.clone()).collect()
}

fn query_snapshot(
    connection: &Connection,
    include_archived: bool,
) -> LibraryResult<LibrarySnapshot> {
    let mut statement = connection.prepare(
        "SELECT
            concepts.entity_id,
            concepts.title,
            entities.created_at,
            entities.updated_at,
            concepts.archived_at,
            (
                SELECT COUNT(*)
                FROM cards
                INNER JOIN entities AS card_entities
                    ON card_entities.id = cards.entity_id
                WHERE cards.concept_id = concepts.entity_id
                    AND card_entities.deleted_at IS NULL
            )
        FROM concepts
        INNER JOIN entities ON entities.id = concepts.entity_id
        WHERE entities.deleted_at IS NULL
            AND (?1 OR concepts.archived_at IS NULL)
        ORDER BY
            concepts.archived_at IS NOT NULL,
            concepts.title COLLATE NOCASE,
            concepts.entity_id",
    )?;
    let concepts = statement.query_map([include_archived], |row| {
        Ok(ConceptSummary {
            id: row.get(0)?,
            title: row.get(1)?,
            created_at: row.get(2)?,
            updated_at: row.get(3)?,
            archived: row.get::<_, Option<i64>>(4)?.is_some(),
            decks: Vec::new(),
            tags: Vec::new(),
            card_count: row.get(5)?,
        })
    })?;
    let mut concepts: Vec<_> = concepts.collect::<Result<_, _>>()?;

    attach_assignments(connection, &mut concepts, OrganizationKind::Deck)?;
    attach_assignments(connection, &mut concepts, OrganizationKind::Tag)?;

    let archived_count = connection.query_row(
        "SELECT COUNT(*)
        FROM concepts
        INNER JOIN entities ON entities.id = concepts.entity_id
        WHERE entities.deleted_at IS NULL
            AND concepts.archived_at IS NOT NULL",
        [],
        |row| row.get(0),
    )?;

    Ok(LibrarySnapshot {
        concepts,
        decks: query_organizations(connection, OrganizationKind::Deck)?,
        tags: query_organizations(connection, OrganizationKind::Tag)?,
        archived_count,
    })
}

fn attach_assignments(
    connection: &Connection,
    concepts: &mut [ConceptSummary],
    kind: OrganizationKind,
) -> LibraryResult<()> {
    let concept_indexes: HashMap<_, _> = concepts
        .iter()
        .enumerate()
        .map(|(index, concept)| (concept.id.clone(), index))
        .collect();
    let sql = format!(
        "SELECT
            memberships.concept_id,
            items.entity_id,
            items.name
        FROM {} AS memberships
        INNER JOIN {} AS items
            ON items.entity_id = memberships.{}
        INNER JOIN entities AS item_entities
            ON item_entities.id = items.entity_id
        WHERE memberships.removed_at IS NULL
            AND item_entities.deleted_at IS NULL
        ORDER BY items.name COLLATE NOCASE, items.entity_id",
        kind.membership_table(),
        kind.table(),
        kind.membership_column()
    );
    let mut statement = connection.prepare(&sql)?;
    let assignments = statement.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            NamedItem {
                id: row.get(1)?,
                name: row.get(2)?,
            },
        ))
    })?;

    for assignment in assignments {
        let (concept_id, item) = assignment?;

        if let Some(index) = concept_indexes.get(&concept_id) {
            match kind {
                OrganizationKind::Deck => concepts[*index].decks.push(item),
                OrganizationKind::Tag => concepts[*index].tags.push(item),
            }
        }
    }

    Ok(())
}

fn query_concept(connection: &Connection, id: &str) -> LibraryResult<ConceptDetail> {
    let concept = connection
        .query_row(
            "SELECT
                concepts.entity_id,
                concepts.title,
                entities.created_at,
                entities.updated_at,
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
                    row.get::<_, Option<i64>>(4)?,
                    row.get::<_, String>(5)?,
                ))
            },
        )
        .optional()?;
    let Some((id, title, created_at, updated_at, archived_at, content)) = concept else {
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
        archived: archived_at.is_some(),
    })
}

fn query_concept_assignments(
    connection: &Connection,
    concept_id: &str,
    kind: OrganizationKind,
) -> LibraryResult<Vec<NamedItem>> {
    let sql = format!(
        "SELECT items.entity_id, items.name
        FROM {} AS memberships
        INNER JOIN {} AS items
            ON items.entity_id = memberships.{}
        INNER JOIN entities AS item_entities
            ON item_entities.id = items.entity_id
        WHERE memberships.concept_id = ?1
            AND memberships.removed_at IS NULL
            AND item_entities.deleted_at IS NULL
        ORDER BY items.name COLLATE NOCASE, items.entity_id",
        kind.membership_table(),
        kind.table(),
        kind.membership_column()
    );
    let mut statement = connection.prepare(&sql)?;
    let items = statement.query_map([concept_id], |row| {
        Ok(NamedItem {
            id: row.get(0)?,
            name: row.get(1)?,
        })
    })?;

    Ok(items.collect::<Result<_, _>>()?)
}

fn query_cards(connection: &Connection, concept_id: &str) -> LibraryResult<Vec<CardSummary>> {
    let mut statement = connection.prepare(
        "SELECT
            cards.entity_id,
            cards.retrieval_kind,
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
        let template_id = row.get::<_, Option<String>>(2)?;
        let template_name = row.get::<_, Option<String>>(3)?;
        let template = match (template_id, template_name) {
            (Some(id), Some(name)) => Some(NamedItem { id, name }),
            (None, None) => None,
            _ => return Err(rusqlite::Error::InvalidQuery),
        };
        let retrieval_kind = row.get::<_, String>(1)?;
        let scheduling_state = row.get::<_, String>(4)?;

        Ok((
            row.get::<_, String>(0)?,
            retrieval_kind,
            template,
            scheduling_state,
            row.get::<_, i64>(5)?,
            row.get::<_, i64>(6)?,
            row.get::<_, i64>(7)?,
        ))
    })?;

    cards
        .map(|card| {
            let (
                id,
                retrieval_kind,
                template,
                scheduling_state,
                due_at,
                review_count,
                lapse_count,
            ) = card?;

            Ok(CardSummary {
                id,
                retrieval_kind: RetrievalFormKind::try_from(retrieval_kind.as_str())?,
                template,
                scheduling_state: SchedulingState::try_from(scheduling_state.as_str())?,
                due_at,
                review_count,
                lapse_count,
            })
        })
        .collect()
}

fn active_card_ids(connection: &Connection, concept_id: &str) -> LibraryResult<Vec<String>> {
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

fn query_organizations(
    connection: &Connection,
    kind: OrganizationKind,
) -> LibraryResult<Vec<OrganizationSummary>> {
    let sql = format!(
        "SELECT
            items.entity_id,
            items.name,
            COUNT(
                CASE
                    WHEN memberships.concept_id IS NOT NULL
                        AND memberships.removed_at IS NULL
                        AND concept_entities.deleted_at IS NULL
                    THEN 1
                END
            )
        FROM {} AS items
        INNER JOIN entities AS item_entities
            ON item_entities.id = items.entity_id
        LEFT JOIN {} AS memberships
            ON memberships.{} = items.entity_id
        LEFT JOIN entities AS concept_entities
            ON concept_entities.id = memberships.concept_id
        WHERE item_entities.deleted_at IS NULL
        GROUP BY items.entity_id, items.name
        ORDER BY items.name COLLATE NOCASE, items.entity_id",
        kind.table(),
        kind.membership_table(),
        kind.membership_column()
    );
    let mut statement = connection.prepare(&sql)?;
    let items = statement.query_map([], |row| {
        Ok(OrganizationSummary {
            id: row.get(0)?,
            name: row.get(1)?,
            concept_count: row.get(2)?,
        })
    })?;

    Ok(items.collect::<Result<_, _>>()?)
}

fn query_organization(
    connection: &Connection,
    kind: OrganizationKind,
    id: &str,
) -> LibraryResult<OrganizationSummary> {
    query_organizations(connection, kind)?
        .into_iter()
        .find(|item| item.id == id)
        .ok_or_else(|| LibraryError::OrganizationNotFound {
            kind: kind.noun(),
            id: id.to_owned(),
        })
}

fn validate_selections(
    connection: &Connection,
    kind: OrganizationKind,
    ids: &HashSet<String>,
) -> LibraryResult<()> {
    let sql = format!(
        "SELECT EXISTS (
            SELECT 1
            FROM {} AS items
            INNER JOIN entities ON entities.id = items.entity_id
            WHERE items.entity_id = ?1
                AND entities.deleted_at IS NULL
        )",
        kind.table()
    );

    for id in ids {
        let exists: bool = connection.query_row(&sql, [id], |row| row.get(0))?;

        if !exists {
            return Err(LibraryError::InvalidSelection {
                kind: kind.noun(),
                id: id.clone(),
            });
        }
    }

    Ok(())
}

fn validate_template_selections(
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

fn ensure_unique_name(
    connection: &Connection,
    kind: OrganizationKind,
    name: &str,
    excluded_id: Option<&str>,
) -> LibraryResult<()> {
    let sql = format!(
        "SELECT items.entity_id, items.name
        FROM {} AS items
        INNER JOIN entities ON entities.id = items.entity_id
        WHERE entities.deleted_at IS NULL",
        kind.table()
    );
    let mut statement = connection.prepare(&sql)?;
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
                kind: kind.noun(),
                name: name.to_owned(),
            });
        }
    }

    Ok(())
}

fn apply_assignments(
    transaction: &WriteTransaction<'_>,
    kind: OrganizationKind,
    concept: &EntityMetadata,
    current_ids: &HashSet<String>,
    desired_ids: &HashSet<String>,
) -> LibraryResult<()> {
    let add_sql = format!(
        "INSERT INTO {} (
            concept_id,
            {},
            created_at,
            updated_at,
            removed_at,
            last_change_id
        ) VALUES (?1, ?2, ?3, ?3, NULL, ?4)
        ON CONFLICT (concept_id, {}) DO UPDATE SET
            updated_at = excluded.updated_at,
            removed_at = NULL,
            last_change_id = excluded.last_change_id",
        kind.membership_table(),
        kind.membership_column(),
        kind.membership_column()
    );
    let remove_sql = format!(
        "UPDATE {}
        SET updated_at = ?1,
            removed_at = ?1,
            last_change_id = ?2
        WHERE concept_id = ?3
            AND {} = ?4
            AND removed_at IS NULL",
        kind.membership_table(),
        kind.membership_column()
    );

    for id in desired_ids.difference(current_ids) {
        transaction.execute(
            &add_sql,
            params![
                concept.id,
                id,
                concept.updated_at,
                concept.last_change_id
            ],
        )?;
    }

    for id in current_ids.difference(desired_ids) {
        transaction.execute(
            &remove_sql,
            params![
                concept.updated_at,
                concept.last_change_id,
                concept.id,
                id
            ],
        )?;
    }

    Ok(())
}

fn apply_media_assignments(
    transaction: &WriteTransaction<'_>,
    concept: &EntityMetadata,
    current_ids: &HashSet<String>,
    desired_ids: &HashSet<String>,
) -> LibraryResult<()> {
    for id in desired_ids.difference(current_ids) {
        transaction.execute(
            "INSERT INTO concept_media (
                concept_id,
                media_id,
                created_at,
                updated_at,
                removed_at,
                last_change_id
            ) VALUES (?1, ?2, ?3, ?3, NULL, ?4)
            ON CONFLICT (concept_id, media_id) DO UPDATE SET
                updated_at = excluded.updated_at,
                removed_at = NULL,
                last_change_id = excluded.last_change_id",
            params![
                concept.id,
                id,
                concept.updated_at,
                concept.last_change_id
            ],
        )?;
    }

    for id in current_ids.difference(desired_ids) {
        transaction.execute(
            "UPDATE concept_media
            SET updated_at = ?1,
                removed_at = ?1,
                last_change_id = ?2
            WHERE concept_id = ?3
                AND media_id = ?4
                AND removed_at IS NULL",
            params![
                concept.updated_at,
                concept.last_change_id,
                concept.id,
                id
            ],
        )?;
    }

    Ok(())
}

fn apply_retrieval_forms(
    transaction: &WriteTransaction<'_>,
    concept_id: &str,
    current_cards: &[CardSummary],
    include_standard_recall: bool,
    template_ids: &BTreeSet<String>,
) -> LibraryResult<()> {
    for card in current_cards {
        let retained = match &card.template {
            Some(template) => template_ids.contains(&template.id),
            None => include_standard_recall,
        };

        if !retained {
            transaction.soft_delete_entity(&card.id)?;
        }
    }

    let has_standard_recall = current_cards.iter().any(|card| card.template.is_none());

    if include_standard_recall && !has_standard_recall {
        create_recall_card(transaction, concept_id, None)?;
    }

    let current_template_ids = current_cards
        .iter()
        .filter_map(|card| card.template.as_ref().map(|template| template.id.as_str()))
        .collect::<HashSet<_>>();

    for template_id in template_ids {
        if !current_template_ids.contains(template_id.as_str()) {
            create_recall_card(transaction, concept_id, Some(template_id))?;
        }
    }

    Ok(())
}

fn concept_record_exists(connection: &Connection, id: &str) -> LibraryResult<bool> {
    Ok(connection.query_row(
        "SELECT EXISTS (SELECT 1 FROM concepts WHERE entity_id = ?1)",
        [id],
        |row| row.get(0),
    )?)
}

fn organization_record_exists(
    connection: &Connection,
    kind: OrganizationKind,
    id: &str,
) -> LibraryResult<bool> {
    let sql = format!(
        "SELECT EXISTS (SELECT 1 FROM {} WHERE entity_id = ?1)",
        kind.table()
    );

    Ok(connection.query_row(&sql, [id], |row| row.get(0))?)
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use image::{DynamicImage, ImageFormat};
    use rusqlite::params;
    use serde_json::json;
    use tempfile::{tempdir, TempDir};

    use super::ConceptLibrary;
    use crate::data::{DataResult, EntityKind, LocalDataStore};
    use crate::library::models::TemplateMode;
    use crate::library::{
        ConceptContent, CreateConceptInput, CreateTemplateInput, GradingMode, LibraryError,
        RecordReviewInput, ReviewRating, SchedulingState, TemplateContent, TemplateLibrary,
        UpdateConceptInput, UpdateSchedulingSettingsInput, UpdateTemplateInput,
    };

    fn test_store() -> (TempDir, LocalDataStore) {
        let directory = tempdir().unwrap();
        let store = LocalDataStore::open(directory.path()).unwrap();

        (directory, store)
    }

    fn png_bytes() -> Vec<u8> {
        let mut bytes = Cursor::new(Vec::new());

        DynamicImage::new_rgba8(4, 3)
            .write_to(&mut bytes, ImageFormat::Png)
            .unwrap();

        bytes.into_inner()
    }

    #[test]
    fn concepts_can_be_created_organized_updated_archived_and_deleted() {
        let (_directory, store) = test_store();
        let library = ConceptLibrary::new(&store);
        let first_deck = library.create_deck("Biology".to_owned()).unwrap();
        let second_deck = library.create_deck("Chemistry".to_owned()).unwrap();
        let unused_deck = library.create_deck("Physics".to_owned()).unwrap();
        let first_tag = library.create_tag("Exam one".to_owned()).unwrap();
        let second_tag = library.create_tag("Needs diagrams".to_owned()).unwrap();

        let created = library
            .create_concept(CreateConceptInput {
                title: "  Cell membrane  ".to_owned(),
                deck_ids: vec![first_deck.id.clone()],
                tag_ids: vec![first_tag.id.clone()],
                content: Default::default(),
                include_standard_recall: true,
                template_ids: Vec::new(),
            })
            .unwrap();

        assert_eq!(created.title, "Cell membrane");
        assert_eq!(created.decks[0].id, first_deck.id);
        assert_eq!(created.tags[0].id, first_tag.id);
        assert_eq!(created.cards.len(), 1);
        assert_eq!(
            store.entity(&created.cards[0].id).unwrap().unwrap().kind,
            EntityKind::Card
        );

        let updated = library
            .update_concept(UpdateConceptInput {
                id: created.id.clone(),
                title: "Plasma membrane".to_owned(),
                deck_ids: vec![second_deck.id.clone()],
                tag_ids: vec![second_tag.id.clone()],
                content: Default::default(),
                include_standard_recall: true,
                template_ids: Vec::new(),
            })
            .unwrap();

        assert_eq!(updated.title, "Plasma membrane");
        assert_eq!(updated.decks[0].id, second_deck.id);
        assert_eq!(updated.tags[0].id, second_tag.id);
        assert_eq!(store.entity(&created.id).unwrap().unwrap().revision, 2);

        let active_snapshot = library.snapshot(false).unwrap();

        assert_eq!(active_snapshot.concepts.len(), 1);
        assert_eq!(
            active_snapshot
                .decks
                .iter()
                .find(|deck| deck.id == second_deck.id)
                .unwrap()
                .concept_count,
            1
        );
        assert_eq!(
            active_snapshot
                .decks
                .iter()
                .find(|deck| deck.id == first_deck.id)
                .unwrap()
                .concept_count,
            0
        );
        assert_eq!(
            active_snapshot
                .decks
                .iter()
                .find(|deck| deck.id == unused_deck.id)
                .unwrap()
                .concept_count,
            0
        );

        let archived = library.set_concept_archived(&created.id, true).unwrap();

        assert!(archived.archived);
        assert!(library.snapshot(false).unwrap().concepts.is_empty());
        assert_eq!(library.snapshot(false).unwrap().archived_count, 1);
        assert_eq!(library.snapshot(true).unwrap().concepts.len(), 1);

        let restored = library.set_concept_archived(&created.id, false).unwrap();

        assert!(!restored.archived);

        library.delete_concept(&created.id).unwrap();
        library.delete_concept(&created.id).unwrap();

        assert!(library.snapshot(true).unwrap().concepts.is_empty());
        assert!(store
            .entity(&created.id)
            .unwrap()
            .unwrap()
            .deleted_at
            .is_some());
        assert!(matches!(
            library.concept(&created.id),
            Err(LibraryError::ConceptNotFound(_))
        ));

        let removed_memberships: i64 = store
            .read_result(|connection| -> DataResult<i64> {
                Ok(connection.query_row(
                    "SELECT COUNT(*)
                    FROM concept_decks
                    WHERE concept_id = ?1
                        AND removed_at IS NOT NULL",
                    [&created.id],
                    |row| row.get(0),
                )?)
            })
            .unwrap();

        assert_eq!(removed_memberships, 1);
    }

    #[test]
    fn organization_names_are_validated_and_deleted_items_leave_assignments_safe() {
        let (_directory, store) = test_store();
        let library = ConceptLibrary::new(&store);
        let deck = library.create_deck("Languages".to_owned()).unwrap();
        let tag = library.create_tag("Speaking".to_owned()).unwrap();
        let concept = library
            .create_concept(CreateConceptInput {
                title: "Greetings".to_owned(),
                deck_ids: vec![deck.id.clone()],
                tag_ids: vec![tag.id.clone()],
                content: Default::default(),
                include_standard_recall: true,
                template_ids: Vec::new(),
            })
            .unwrap();

        assert!(matches!(
            library.create_deck(" languages ".to_owned()),
            Err(LibraryError::DuplicateName { .. })
        ));
        assert!(matches!(
            library.create_tag("   ".to_owned()),
            Err(LibraryError::EmptyValue { .. })
        ));

        let renamed_deck = library
            .rename_deck(&deck.id, "French".to_owned())
            .unwrap();
        let renamed_tag = library
            .rename_tag(&tag.id, "Conversation".to_owned())
            .unwrap();

        assert_eq!(renamed_deck.name, "French");
        assert_eq!(renamed_tag.name, "Conversation");

        library.delete_deck(&deck.id).unwrap();
        library.delete_tag(&tag.id).unwrap();

        let detail = library.concept(&concept.id).unwrap();
        let snapshot = library.snapshot(false).unwrap();

        assert!(detail.decks.is_empty());
        assert!(detail.tags.is_empty());
        assert!(snapshot.decks.is_empty());
        assert!(snapshot.tags.is_empty());
        assert!(store
            .entity(&deck.id)
            .unwrap()
            .unwrap()
            .deleted_at
            .is_some());
        assert!(store
            .entity(&tag.id)
            .unwrap()
            .unwrap()
            .deleted_at
            .is_some());
    }

    #[test]
    fn invalid_assignments_and_no_op_edits_do_not_create_changes() {
        let (_directory, store) = test_store();
        let library = ConceptLibrary::new(&store);

        let invalid_create = library.create_concept(CreateConceptInput {
            title: "Invalid".to_owned(),
            deck_ids: vec!["missing-deck".to_owned()],
            tag_ids: Vec::new(),
            content: Default::default(),
            include_standard_recall: true,
            template_ids: Vec::new(),
        });

        assert!(matches!(
            invalid_create,
            Err(LibraryError::InvalidSelection { .. })
        ));
        assert!(store.changes_after(0, 100).unwrap().is_empty());

        let concept = library
            .create_concept(CreateConceptInput {
                title: "Stable".to_owned(),
                deck_ids: Vec::new(),
                tag_ids: Vec::new(),
                content: Default::default(),
                include_standard_recall: true,
                template_ids: Vec::new(),
            })
            .unwrap();
        let revision = store.entity(&concept.id).unwrap().unwrap().revision;

        library
            .update_concept(UpdateConceptInput {
                id: concept.id.clone(),
                title: concept.title,
                deck_ids: Vec::new(),
                tag_ids: Vec::new(),
                content: Default::default(),
                include_standard_recall: true,
                template_ids: Vec::new(),
            })
            .unwrap();

        assert_eq!(
            store.entity(&concept.id).unwrap().unwrap().revision,
            revision
        );
    }

    #[test]
    fn rich_content_and_media_references_update_transactionally() {
        let (_directory, store) = test_store();
        let library = ConceptLibrary::new(&store);
        let image_bytes = png_bytes();
        let media = library.import_image(&image_bytes).unwrap();
        let content = ConceptContent {
            schema_version: 1,
            prompt: json!({
                "type": "doc",
                "content": [
                    {
                        "type": "paragraph",
                        "content": [{
                            "type": "text",
                            "text": "Identify this structure."
                        }]
                    },
                    {
                        "type": "mediaImage",
                        "attrs": {
                            "mediaId": media.id,
                            "alt": "A test image",
                            "title": null
                        }
                    }
                ]
            }),
            answer: json!({
                "type": "doc",
                "content": [{
                    "type": "codeBlock",
                    "attrs": { "language": "rust" },
                    "content": [{
                        "type": "text",
                        "text": "struct Cell;"
                    }]
                }]
            }),
        };
        let concept = library
            .create_concept(CreateConceptInput {
                title: "Rich concept".to_owned(),
                deck_ids: Vec::new(),
                tag_ids: Vec::new(),
                content: content.clone(),
                include_standard_recall: true,
                template_ids: Vec::new(),
            })
            .unwrap();

        assert_eq!(concept.content, content);
        assert_eq!(concept.media, vec![media.clone()]);
        assert_eq!(library.media_bytes(&media.id).unwrap(), image_bytes);
        assert!(library.study_queue().unwrap().media.is_empty());

        let revision = store.entity(&concept.id).unwrap().unwrap().revision;
        let invalid_content = ConceptContent {
            schema_version: 1,
            prompt: json!({
                "type": "doc",
                "content": [{
                    "type": "mediaImage",
                    "attrs": {
                        "mediaId": "018f1e2d-3c4b-7a69-8f10-123456789abc",
                        "alt": null,
                        "title": null
                    }
                }]
            }),
            answer: ConceptContent::default().answer,
        };
        let invalid_update = library.update_concept(UpdateConceptInput {
            id: concept.id.clone(),
            title: concept.title.clone(),
            deck_ids: Vec::new(),
            tag_ids: Vec::new(),
            content: invalid_content,
            include_standard_recall: true,
            template_ids: Vec::new(),
        });

        assert!(matches!(
            invalid_update,
            Err(LibraryError::MediaNotFound(_))
        ));
        assert_eq!(store.entity(&concept.id).unwrap().unwrap().revision, revision);

        let updated = library
            .update_concept(UpdateConceptInput {
                id: concept.id.clone(),
                title: concept.title,
                deck_ids: Vec::new(),
                tag_ids: Vec::new(),
                content: Default::default(),
                include_standard_recall: true,
                template_ids: Vec::new(),
            })
            .unwrap();
        let removed_references: i64 = store
            .read_result(|connection| -> DataResult<i64> {
                Ok(connection.query_row(
                    "SELECT COUNT(*)
                    FROM concept_media
                    WHERE concept_id = ?1
                        AND removed_at IS NOT NULL",
                    [&concept.id],
                    |row| row.get(0),
                )?)
            })
            .unwrap();

        assert_eq!(updated.content, ConceptContent::default());
        assert!(updated.media.is_empty());
        assert_eq!(removed_references, 1);
    }

    #[test]
    fn associated_cards_are_visible_and_follow_card_tombstones() {
        let (_directory, store) = test_store();
        let library = ConceptLibrary::new(&store);
        let concept = library
            .create_concept(CreateConceptInput {
                title: "Linked card".to_owned(),
                deck_ids: Vec::new(),
                tag_ids: Vec::new(),
                content: Default::default(),
                include_standard_recall: true,
                template_ids: Vec::new(),
            })
            .unwrap();

        let card_id = concept.cards[0].id.clone();

        assert_eq!(library.concept(&concept.id).unwrap().cards[0].id, card_id);
        assert_eq!(library.snapshot(false).unwrap().concepts[0].card_count, 1);

        let duplicate_card = store.write(|transaction| {
            let card = transaction.create_entity(EntityKind::Card)?;

            transaction.execute(
                "INSERT INTO cards (
                    entity_id,
                    concept_id,
                    retrieval_kind,
                    template_id,
                    last_change_id
                ) VALUES (?1, ?2, 'recall', NULL, ?3)",
                params![card.id, concept.id, card.last_change_id],
            )?;

            Ok(card)
        });

        assert!(duplicate_card.is_err());

        store
            .write(|transaction| transaction.soft_delete_entity(&card_id))
            .unwrap();

        assert!(library.concept(&concept.id).unwrap().cards.is_empty());
        assert_eq!(library.snapshot(false).unwrap().concepts[0].card_count, 0);

        let second_card = store
            .write(|transaction| {
                let card = transaction.create_entity(EntityKind::Card)?;

                transaction.execute(
                    "INSERT INTO cards (
                        entity_id,
                        concept_id,
                        retrieval_kind,
                        template_id,
                        last_change_id
                    ) VALUES (?1, ?2, 'recall', NULL, ?3)",
                    params![card.id, concept.id, card.last_change_id],
                )?;

                Ok(card)
            })
            .unwrap();

        library.delete_concept(&concept.id).unwrap();

        assert!(store
            .entity(&second_card.id)
            .unwrap()
            .unwrap()
            .deleted_at
            .is_some());
    }

    #[test]
    fn retrieval_forms_are_selected_without_duplicates_and_schedule_independently() {
        let (_directory, store) = test_store();
        let library = ConceptLibrary::new(&store);
        let templates = TemplateLibrary::new(&store);
        let missing_forms = library.create_concept(CreateConceptInput {
            title: "No forms".to_owned(),
            deck_ids: Vec::new(),
            tag_ids: Vec::new(),
            content: Default::default(),
            include_standard_recall: false,
            template_ids: Vec::new(),
        });

        assert!(matches!(
            missing_forms,
            Err(LibraryError::MissingRetrievalForm)
        ));
        assert!(library.snapshot(false).unwrap().concepts.is_empty());

        let mut custom_template_content = TemplateContent::default();

        custom_template_content.mode = TemplateMode::Custom;

        let first_template = templates
            .create_template(CreateTemplateInput {
                name: "Answer first".to_owned(),
                content: custom_template_content,
            })
            .unwrap();
        let second_template = templates
            .create_template(CreateTemplateInput {
                name: "Prompt focused".to_owned(),
                content: TemplateContent::default(),
            })
            .unwrap();
        let media = library.import_image(&png_bytes()).unwrap();
        let concept = library
            .create_concept(CreateConceptInput {
                title: "Complementary practice".to_owned(),
                deck_ids: Vec::new(),
                tag_ids: Vec::new(),
                content: ConceptContent {
                    schema_version: 1,
                    prompt: json!({
                        "type": "doc",
                        "content": [{
                            "type": "mediaImage",
                            "attrs": {
                                "mediaId": media.id,
                                "alt": "Practice image",
                                "title": null
                            }
                        }]
                    }),
                    answer: ConceptContent::default().answer,
                },
                include_standard_recall: false,
                template_ids: vec![
                    second_template.id.clone(),
                    first_template.id.clone(),
                    first_template.id.clone(),
                ],
            })
            .unwrap();

        assert_eq!(concept.cards.len(), 2);
        assert_eq!(
            concept
                .cards
                .iter()
                .map(|card| card.template.as_ref().unwrap().name.as_str())
                .collect::<Vec<_>>(),
            vec!["Answer first", "Prompt focused"]
        );

        let reviewed_card = concept.cards[0].clone();
        let waiting_card = concept.cards[1].clone();
        let review_time = reviewed_card.due_at.max(waiting_card.due_at);
        let initial_queue = library.study_queue_at(review_time).unwrap();

        assert_eq!(initial_queue.media, vec![media]);

        library
            .record_review_at(
                RecordReviewInput {
                    card_id: reviewed_card.id.clone(),
                    rating: ReviewRating::Good,
                },
                review_time,
            )
            .unwrap();

        let queue = library.study_queue_at(review_time).unwrap();

        assert_eq!(queue.cards.len(), 1);
        assert!(queue.media.is_empty());
        assert_eq!(queue.cards[0].id, waiting_card.id);
        assert_eq!(
            queue.cards[0].template.as_ref().unwrap().id,
            second_template.id
        );

        let updated_template = templates
            .update_template(UpdateTemplateInput {
                id: second_template.id.clone(),
                name: "Focused prompt".to_owned(),
                content: second_template.content,
            })
            .unwrap();
        let updated_queue = library.study_queue_at(review_time).unwrap();

        assert_eq!(
            updated_queue.cards[0].template.as_ref().unwrap().name,
            updated_template.name
        );
    }

    #[test]
    fn changing_retrieval_forms_retains_selected_schedules_and_protects_templates_in_use() {
        let (_directory, store) = test_store();
        let library = ConceptLibrary::new(&store);
        let templates = TemplateLibrary::new(&store);
        let removed_template = templates
            .create_template(CreateTemplateInput {
                name: "Temporary".to_owned(),
                content: TemplateContent::default(),
            })
            .unwrap();
        let retained_template = templates
            .create_template(CreateTemplateInput {
                name: "Retained".to_owned(),
                content: TemplateContent::default(),
            })
            .unwrap();
        let concept = library
            .create_concept(CreateConceptInput {
                title: "Managed forms".to_owned(),
                deck_ids: Vec::new(),
                tag_ids: Vec::new(),
                content: Default::default(),
                include_standard_recall: false,
                template_ids: vec![
                    removed_template.id.clone(),
                    retained_template.id.clone(),
                ],
            })
            .unwrap();
        let removed_card_id = concept
            .cards
            .iter()
            .find(|card| card.template.as_ref().unwrap().id == removed_template.id)
            .unwrap()
            .id
            .clone();
        let retained_card_id = concept
            .cards
            .iter()
            .find(|card| card.template.as_ref().unwrap().id == retained_template.id)
            .unwrap()
            .id
            .clone();

        let updated = library
            .update_concept(UpdateConceptInput {
                id: concept.id.clone(),
                title: concept.title,
                deck_ids: Vec::new(),
                tag_ids: Vec::new(),
                content: concept.content,
                include_standard_recall: true,
                template_ids: vec![retained_template.id.clone()],
            })
            .unwrap();

        assert_eq!(updated.cards.len(), 2);
        assert!(updated.cards.iter().any(|card| card.template.is_none()));
        assert!(updated.cards.iter().any(|card| card.id == retained_card_id));
        assert!(store
            .entity(&removed_card_id)
            .unwrap()
            .unwrap()
            .deleted_at
            .is_some());
        assert_eq!(
            templates
                .catalog()
                .unwrap()
                .templates
                .into_iter()
                .find(|template| template.id == retained_template.id)
                .unwrap()
                .retrieval_form_count,
            1
        );
        assert!(matches!(
            templates.delete_template(&retained_template.id),
            Err(LibraryError::TemplateInUse {
                retrieval_form_count: 1,
                ..
            })
        ));
        assert!(store
            .write(|transaction| transaction.soft_delete_entity(&retained_template.id))
            .is_err());

        library
            .update_concept(UpdateConceptInput {
                id: concept.id,
                title: updated.title,
                deck_ids: Vec::new(),
                tag_ids: Vec::new(),
                content: updated.content,
                include_standard_recall: true,
                template_ids: Vec::new(),
            })
            .unwrap();

        templates.delete_template(&removed_template.id).unwrap();
        templates.delete_template(&retained_template.id).unwrap();
    }

    #[test]
    fn study_cards_include_only_active_unarchived_recall_cards() {
        let (_directory, store) = test_store();
        let library = ConceptLibrary::new(&store);
        let content = ConceptContent {
            schema_version: 1,
            prompt: json!({
                "type": "doc",
                "content": [{
                    "type": "paragraph",
                    "content": [{ "type": "text", "text": "A prompt" }]
                }]
            }),
            answer: json!({
                "type": "doc",
                "content": [{
                    "type": "paragraph",
                    "content": [{ "type": "text", "text": "An answer" }]
                }]
            }),
        };
        let active = library
            .create_concept(CreateConceptInput {
                title: "Active concept".to_owned(),
                deck_ids: Vec::new(),
                tag_ids: Vec::new(),
                content: content.clone(),
                include_standard_recall: true,
                template_ids: Vec::new(),
            })
            .unwrap();
        let archived = library
            .create_concept(CreateConceptInput {
                title: "Archived concept".to_owned(),
                deck_ids: Vec::new(),
                tag_ids: Vec::new(),
                content: Default::default(),
                include_standard_recall: true,
                template_ids: Vec::new(),
            })
            .unwrap();

        library.set_concept_archived(&archived.id, true).unwrap();

        let cards = library.study_queue().unwrap().cards;

        assert_eq!(cards.len(), 1);
        assert_eq!(cards[0].id, active.cards[0].id);
        assert_eq!(cards[0].concept_id, active.id);
        assert_eq!(cards[0].concept_title, active.title);
        assert_eq!(cards[0].content, content);

        store
            .write(|transaction| transaction.soft_delete_entity(&cards[0].id))
            .unwrap();

        assert!(library.study_queue().unwrap().cards.is_empty());
    }

    #[test]
    fn grading_mode_is_a_durable_device_local_preference() {
        let directory = tempdir().unwrap();

        {
            let store = LocalDataStore::open(directory.path()).unwrap();
            let library = ConceptLibrary::new(&store);

            assert_eq!(
                library.study_preferences().unwrap().grading_mode,
                GradingMode::Simple
            );

            let changes_before = store.changes_after(0, 100).unwrap();
            let preferences = library.set_grading_mode(GradingMode::Advanced).unwrap();

            assert_eq!(preferences.grading_mode, GradingMode::Advanced);
            assert_eq!(store.changes_after(0, 100).unwrap(), changes_before);
        }

        let reopened_store = LocalDataStore::open(directory.path()).unwrap();
        let reopened_library = ConceptLibrary::new(&reopened_store);

        assert_eq!(
            reopened_library
                .study_preferences()
                .unwrap()
                .grading_mode,
            GradingMode::Advanced
        );
    }

    #[test]
    fn scheduling_settings_create_immutable_configuration_revisions() {
        let (_directory, store) = test_store();
        let library = ConceptLibrary::new(&store);
        let defaults = library.scheduling_settings().unwrap();

        assert_eq!(defaults.algorithm_version, "6.6.1");
        assert_eq!(defaults.desired_retention, 0.9);
        assert_eq!(defaults.maximum_interval_days, 36_500);

        let original_configuration: (String, String) = store
            .read_result(|connection| -> DataResult<_> {
                Ok(connection.query_row(
                    "SELECT configuration_id, parameters_json
                    FROM active_scheduler_configuration
                    INNER JOIN scheduler_configurations
                        ON scheduler_configurations.id = configuration_id
                    WHERE singleton = 1",
                    [],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )?)
            })
            .unwrap();
        let updated = library
            .update_scheduling_settings(UpdateSchedulingSettingsInput {
                desired_retention: 0.92,
                maximum_interval_days: 3_650,
            })
            .unwrap();
        let (active_id, parameters_json, configuration_count): (String, String, i64) = store
            .read_result(|connection| -> DataResult<_> {
                let (active_id, parameters_json) = connection.query_row(
                    "SELECT configuration_id, parameters_json
                    FROM active_scheduler_configuration
                    INNER JOIN scheduler_configurations
                        ON scheduler_configurations.id = configuration_id
                    WHERE singleton = 1",
                    [],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )?;
                let configuration_count = connection.query_row(
                    "SELECT COUNT(*) FROM scheduler_configurations",
                    [],
                    |row| row.get(0),
                )?;

                Ok((active_id, parameters_json, configuration_count))
            })
            .unwrap();

        assert_eq!(updated.desired_retention, 0.92);
        assert_eq!(updated.maximum_interval_days, 3_650);
        assert_ne!(active_id, original_configuration.0);
        assert_eq!(parameters_json, original_configuration.1);
        assert_eq!(configuration_count, 2);
        assert_eq!(
            uuid::Uuid::parse_str(&active_id).unwrap().get_version(),
            Some(uuid::Version::SortRand)
        );

        let unchanged = library
            .update_scheduling_settings(UpdateSchedulingSettingsInput {
                desired_retention: 0.92,
                maximum_interval_days: 3_650,
            })
            .unwrap();
        let configuration_count: i64 = store
            .read_result(|connection| -> DataResult<_> {
                Ok(connection.query_row(
                    "SELECT COUNT(*) FROM scheduler_configurations",
                    [],
                    |row| row.get(0),
                )?)
            })
            .unwrap();

        assert_eq!(unchanged, updated);
        assert_eq!(configuration_count, 2);

        let altered_original: DataResult<()> = store.write(|transaction| {
            transaction.execute(
                "UPDATE scheduler_configurations
                SET desired_retention = 0.91
                WHERE id = ?1",
                [&original_configuration.0],
            )?;

            Ok(())
        });

        assert!(altered_original.is_err());
    }

    #[test]
    fn scheduling_settings_validate_bounds_without_creating_revisions() {
        let (_directory, store) = test_store();
        let library = ConceptLibrary::new(&store);

        for desired_retention in [f64::NAN, 0.79, 0.98] {
            assert!(matches!(
                library.update_scheduling_settings(UpdateSchedulingSettingsInput {
                    desired_retention,
                    maximum_interval_days: 365,
                }),
                Err(LibraryError::InvalidDesiredRetention { .. })
            ));
        }

        for maximum_interval_days in [0, 36_501] {
            assert!(matches!(
                library.update_scheduling_settings(UpdateSchedulingSettingsInput {
                    desired_retention: 0.9,
                    maximum_interval_days,
                }),
                Err(LibraryError::InvalidMaximumInterval { .. })
            ));
        }

        let configuration_count: i64 = store
            .read_result(|connection| -> DataResult<_> {
                Ok(connection.query_row(
                    "SELECT COUNT(*) FROM scheduler_configurations",
                    [],
                    |row| row.get(0),
                )?)
            })
            .unwrap();

        assert_eq!(configuration_count, 1);
    }

    #[test]
    fn scheduling_changes_leave_existing_due_dates_and_cap_future_intervals() {
        let (_directory, store) = test_store();
        let library = ConceptLibrary::new(&store);
        let concept = library
            .create_concept(CreateConceptInput {
                title: "Future scheduling settings".to_owned(),
                deck_ids: Vec::new(),
                tag_ids: Vec::new(),
                content: Default::default(),
                include_standard_recall: true,
                template_ids: Vec::new(),
            })
            .unwrap();
        let card = library.study_queue().unwrap().cards[0].clone();
        let first_review = library
            .record_review_at(
                RecordReviewInput {
                    card_id: card.id.clone(),
                    rating: ReviewRating::Good,
                },
                card.due_at,
            )
            .unwrap();

        library
            .update_scheduling_settings(UpdateSchedulingSettingsInput {
                desired_retention: 0.9,
                maximum_interval_days: 1,
            })
            .unwrap();

        let due_at_after_settings: i64 = store
            .read_result(|connection| -> DataResult<_> {
                Ok(connection.query_row(
                    "SELECT due_at FROM card_scheduling WHERE card_id = ?1",
                    [&card.id],
                    |row| row.get(0),
                )?)
            })
            .unwrap();

        assert_eq!(due_at_after_settings, first_review.due_at);

        let second_review = library
            .record_review_at(
                RecordReviewInput {
                    card_id: card.id.clone(),
                    rating: ReviewRating::Easy,
                },
                first_review.due_at,
            )
            .unwrap();
        let configuration_ids: Vec<String> = store
            .read_result(|connection| -> DataResult<_> {
                let mut statement = connection.prepare(
                    "SELECT scheduler_configuration_id
                    FROM reviews
                    WHERE card_id = ?1
                    ORDER BY reviewed_at, entity_id",
                )?;
                let rows = statement.query_map([&card.id], |row| row.get(0))?;

                Ok(rows.collect::<Result<_, _>>()?)
            })
            .unwrap();

        assert_eq!(second_review.scheduled_interval_days, 1.0);
        assert_eq!(
            second_review.due_at,
            second_review.reviewed_at + 86_400_000
        );
        assert_eq!(configuration_ids.len(), 2);
        assert_ne!(configuration_ids[0], configuration_ids[1]);
        assert_eq!(concept.cards[0].id, card.id);
    }

    #[test]
    fn a_failed_scheduling_settings_write_rolls_back_the_revision() {
        let (_directory, store) = test_store();
        let library = ConceptLibrary::new(&store);

        store
            .write(|transaction| {
                transaction.execute_batch(
                    "CREATE TRIGGER force_scheduling_settings_failure
                    BEFORE UPDATE ON active_scheduler_configuration
                    FOR EACH ROW
                    BEGIN
                        SELECT RAISE(ABORT, 'forced scheduling settings failure');
                    END;",
                )?;

                Ok(())
            })
            .unwrap();

        let result = library.update_scheduling_settings(UpdateSchedulingSettingsInput {
            desired_retention: 0.91,
            maximum_interval_days: 3_650,
        });
        let configuration_count: i64 = store
            .read_result(|connection| -> DataResult<_> {
                Ok(connection.query_row(
                    "SELECT COUNT(*) FROM scheduler_configurations",
                    [],
                    |row| row.get(0),
                )?)
            })
            .unwrap();

        assert!(result.is_err());
        assert_eq!(configuration_count, 1);
        assert_eq!(library.scheduling_settings().unwrap().desired_retention, 0.9);
    }

    #[test]
    fn reviews_persist_fsrs_scheduling_and_only_due_cards_are_queued() {
        let (_directory, store) = test_store();
        let library = ConceptLibrary::new(&store);
        library
            .create_concept(CreateConceptInput {
                title: "Scheduled concept".to_owned(),
                deck_ids: Vec::new(),
                tag_ids: Vec::new(),
                content: Default::default(),
                include_standard_recall: true,
                template_ids: Vec::new(),
            })
            .unwrap();
        let initial_queue = library.study_queue().unwrap();
        let card = initial_queue.cards[0].clone();

        assert_eq!(initial_queue.total_cards, 1);
        assert_eq!(card.scheduling_state, SchedulingState::New);

        let first_review = library
            .record_review_at(
                RecordReviewInput {
                    card_id: card.id.clone(),
                    rating: ReviewRating::Good,
                },
                card.due_at,
            )
            .unwrap();

        assert_eq!(first_review.scheduling_state, SchedulingState::Review);
        assert!((first_review.scheduled_interval_days - 2.3065).abs() < 0.0001);
        assert_eq!(
            first_review.due_at,
            first_review.reviewed_at
                + (first_review.scheduled_interval_days * 86_400_000.0).round() as i64
        );

        let waiting_queue = library.study_queue_at(first_review.reviewed_at).unwrap();

        assert!(waiting_queue.cards.is_empty());
        assert_eq!(waiting_queue.next_due_at, Some(first_review.due_at));
        assert_eq!(waiting_queue.total_cards, 1);

        let changes_before_duplicate = store.changes_after(0, 100).unwrap();
        let duplicate = library.record_review_at(
            RecordReviewInput {
                card_id: card.id.clone(),
                rating: ReviewRating::Good,
            },
            first_review.reviewed_at,
        );

        assert!(matches!(duplicate, Err(LibraryError::CardNotDue { .. })));
        assert_eq!(
            store.changes_after(0, 100).unwrap(),
            changes_before_duplicate
        );

        let due_queue = library.study_queue_at(first_review.due_at).unwrap();

        assert_eq!(due_queue.cards.len(), 1);
        assert_eq!(due_queue.cards[0].id, card.id);

        let lapse = library
            .record_review_at(
                RecordReviewInput {
                    card_id: card.id.clone(),
                    rating: ReviewRating::Again,
                },
                first_review.due_at,
            )
            .unwrap();

        assert_eq!(lapse.scheduling_state, SchedulingState::Relearning);
        assert!(lapse.due_at > lapse.reviewed_at);

        let (review_count, lapse_count, state): (i64, i64, String) = store
            .read_result(|connection| -> DataResult<_> {
                Ok(connection.query_row(
                    "SELECT review_count, lapse_count, state
                    FROM card_scheduling
                    WHERE card_id = ?1",
                    [&card.id],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
                )?)
            })
            .unwrap();
        let history: Vec<(i64, i64, String)> = store
            .read_result(|connection| -> DataResult<_> {
                let mut statement = connection.prepare(
                    "SELECT rating, elapsed_days, scheduler_configuration_id
                    FROM reviews
                    WHERE card_id = ?1
                    ORDER BY reviewed_at, entity_id",
                )?;
                let rows = statement.query_map([&card.id], |row| {
                    Ok((row.get(0)?, row.get(1)?, row.get(2)?))
                })?;

                Ok(rows.collect::<Result<_, _>>()?)
            })
            .unwrap();

        assert_eq!(review_count, 2);
        assert_eq!(lapse_count, 1);
        assert_eq!(state, "relearning");
        assert_eq!(
            history,
            vec![
                (3, 0, "fsrs-6.6.1-default-0.90".to_owned()),
                (1, 2, "fsrs-6.6.1-default-0.90".to_owned()),
            ]
        );

        let altered_history: DataResult<()> = store.write(|transaction| {
            transaction.execute(
                "UPDATE reviews SET rating = 4 WHERE entity_id = ?1",
                [&first_review.review_id],
            )?;

            Ok(())
        });
        let removed_history: DataResult<()> = store.write(|transaction| {
            transaction.execute(
                "DELETE FROM reviews WHERE entity_id = ?1",
                [&first_review.review_id],
            )?;

            Ok(())
        });

        assert!(altered_history.is_err());
        assert!(removed_history.is_err());
    }

    #[test]
    fn a_failed_new_card_enters_learning_without_counting_a_lapse() {
        let (_directory, store) = test_store();
        let library = ConceptLibrary::new(&store);

        library
            .create_concept(CreateConceptInput {
                title: "Learning concept".to_owned(),
                deck_ids: Vec::new(),
                tag_ids: Vec::new(),
                content: Default::default(),
                include_standard_recall: true,
                template_ids: Vec::new(),
            })
            .unwrap();

        let card = library.study_queue().unwrap().cards[0].clone();
        let review = library
            .record_review_at(
                RecordReviewInput {
                    card_id: card.id.clone(),
                    rating: ReviewRating::Again,
                },
                card.due_at,
            )
            .unwrap();

        assert_eq!(review.scheduling_state, SchedulingState::Learning);
        assert!((review.scheduled_interval_days - 0.212).abs() < 0.0001);

        let lapse_count: i64 = store
            .read_result(|connection| -> DataResult<_> {
                Ok(connection.query_row(
                    "SELECT lapse_count
                    FROM card_scheduling
                    WHERE card_id = ?1",
                    [&card.id],
                    |row| row.get(0),
                )?)
            })
            .unwrap();

        assert_eq!(lapse_count, 0);
    }

    #[test]
    fn a_failed_review_write_rolls_back_the_event_and_schedule() {
        let (_directory, store) = test_store();
        let library = ConceptLibrary::new(&store);
        let concept = library
            .create_concept(CreateConceptInput {
                title: "Rollback".to_owned(),
                deck_ids: Vec::new(),
                tag_ids: Vec::new(),
                content: Default::default(),
                include_standard_recall: true,
                template_ids: Vec::new(),
            })
            .unwrap();
        let card = library.study_queue().unwrap().cards[0].clone();

        store
            .write(|transaction| {
                transaction.execute_batch(
                    "CREATE TRIGGER force_review_failure
                    AFTER INSERT ON reviews
                    FOR EACH ROW
                    BEGIN
                        SELECT RAISE(ABORT, 'forced review failure');
                    END;",
                )?;

                Ok(())
            })
            .unwrap();

        let changes_before = store.changes_after(0, 100).unwrap();
        let result = library.record_review_at(
            RecordReviewInput {
                card_id: card.id.clone(),
                rating: ReviewRating::Good,
            },
            card.due_at,
        );

        assert!(result.is_err());
        assert_eq!(store.changes_after(0, 100).unwrap(), changes_before);

        let (review_entities, reviews, review_count): (i64, i64, i64) = store
            .read_result(|connection| -> DataResult<_> {
                let review_entities = connection.query_row(
                    "SELECT COUNT(*) FROM entities WHERE kind = 'review'",
                    [],
                    |row| row.get(0),
                )?;
                let reviews = connection.query_row(
                    "SELECT COUNT(*) FROM reviews",
                    [],
                    |row| row.get(0),
                )?;
                let review_count = connection.query_row(
                    "SELECT review_count
                    FROM card_scheduling
                    WHERE card_id = ?1",
                    [&concept.cards[0].id],
                    |row| row.get(0),
                )?;

                Ok((review_entities, reviews, review_count))
            })
            .unwrap();

        assert_eq!((review_entities, reviews, review_count), (0, 0, 0));
    }

    #[test]
    fn concept_rows_cannot_bypass_change_tracking_or_tombstones() {
        let (_directory, store) = test_store();
        let library = ConceptLibrary::new(&store);
        let concept = library
            .create_concept(CreateConceptInput {
                title: "Protected".to_owned(),
                deck_ids: Vec::new(),
                tag_ids: Vec::new(),
                content: Default::default(),
                include_standard_recall: true,
                template_ids: Vec::new(),
            })
            .unwrap();

        let bypassed_update: DataResult<()> = store.write(|transaction| {
            transaction.execute(
                "UPDATE concepts SET title = 'Untracked' WHERE entity_id = ?1",
                [&concept.id],
            )?;

            Ok(())
        });
        let hard_delete: DataResult<()> = store.write(|transaction| {
            transaction.execute(
                "DELETE FROM concepts WHERE entity_id = ?1",
                [&concept.id],
            )?;

            Ok(())
        });

        assert!(bypassed_update.is_err());
        assert!(hard_delete.is_err());
        assert_eq!(library.concept(&concept.id).unwrap().title, "Protected");
    }
}
