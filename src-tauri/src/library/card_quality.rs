use rusqlite::{params, Connection, OptionalExtension};
use uuid::Uuid;

use crate::data::{EntityKind, LocalDataStore};
use crate::library::retrieval_forms::parse_retrieval_form_configuration;
use crate::library::{
    CardQualityConcern, CardQualityDisposition, CardQualityEvidence, CardQualityKind,
    CardQualityQueue, CardQualityQueueItem, CardQualitySignal, CardQualitySource,
    CardQualityStatus, CloseCardQualityConcernInput, CreateCardQualityConcernInput,
    DismissCardQualitySignalInput, LibraryError, LibraryResult, NamedItem, RetrievalFormKind,
};

const MAXIMUM_CARD_QUALITY_NOTE_LENGTH: usize = 500;
const REPEATED_DIFFICULTY_AGAIN_COUNT: i64 = 3;
const REPEATED_DIFFICULTY_REVIEW_WINDOW: i64 = 5;

pub struct CardQualityLibrary<'store> {
    store: &'store LocalDataStore,
}

impl<'store> CardQualityLibrary<'store> {
    pub fn new(store: &'store LocalDataStore) -> Self {
        Self { store }
    }

    pub fn queue(&self) -> LibraryResult<CardQualityQueue> {
        self.store.read_result(query_queue)
    }

    pub fn create_concern(
        &self,
        input: CreateCardQualityConcernInput,
    ) -> LibraryResult<CardQualityConcern> {
        let card_id = normalize_id(input.card_id, "card ID")?;
        let note = normalize_note(input.note)?;

        self.store.write_result(|transaction| {
            validate_active_card(transaction, &card_id)?;

            if let Some(existing) = query_open_manual_concern(transaction, &card_id, input.kind)? {
                if existing.note != note {
                    return Err(LibraryError::CardQualityConcernConflict);
                }

                return Ok(existing);
            }

            let entity = transaction.create_entity(EntityKind::CardQualityConcern)?;

            transaction.execute(
                "INSERT INTO card_quality_concerns (
                    entity_id,
                    card_id,
                    source,
                    kind,
                    status,
                    note,
                    observed_through_review_id,
                    status_changed_at,
                    last_change_id
                ) VALUES (?1, ?2, 'manual', ?3, 'open', ?4, NULL, NULL, ?5)",
                params![
                    entity.id,
                    card_id,
                    input.kind.as_str(),
                    note,
                    entity.last_change_id,
                ],
            )?;

            query_concern(transaction, &entity.id)
        })
    }

    pub fn close_concern(
        &self,
        input: CloseCardQualityConcernInput,
    ) -> LibraryResult<CardQualityConcern> {
        let concern_id = normalize_id(input.concern_id, "concern ID")?;
        let next_status = input.disposition.status();

        self.store.write_result(|transaction| {
            let concern = query_concern(transaction, &concern_id)?;

            if concern.source != CardQualitySource::Manual {
                return Err(LibraryError::CardQualityConcernClosed(concern_id));
            }

            if concern.status == next_status {
                return Ok(concern);
            }

            if concern.status != CardQualityStatus::Open {
                return Err(LibraryError::CardQualityConcernClosed(concern_id));
            }

            let entity = transaction.touch_entity(&concern_id)?;

            transaction.execute(
                "UPDATE card_quality_concerns
                SET status = ?1,
                    status_changed_at = ?2,
                    last_change_id = ?3
                WHERE entity_id = ?4",
                params![
                    next_status.as_str(),
                    entity.updated_at,
                    entity.last_change_id,
                    concern_id,
                ],
            )?;

            query_concern(transaction, &entity.id)
        })
    }

    pub fn dismiss_signal(
        &self,
        input: DismissCardQualitySignalInput,
    ) -> LibraryResult<CardQualityConcern> {
        let card_id = normalize_id(input.card_id, "card ID")?;
        let observed_through_review_id =
            normalize_id(input.observed_through_review_id, "observed review ID")?;

        self.store.write_result(|transaction| {
            if let Some(existing) = query_signal_dismissal(
                transaction,
                &card_id,
                input.signal,
                &observed_through_review_id,
            )? {
                return Ok(existing);
            }

            validate_active_card(transaction, &card_id)?;

            let signal = query_review_pattern_items(transaction, Some(&card_id))?
                .into_iter()
                .find(|item| input.signal.matches(item));

            let Some(signal) = signal else {
                return Err(LibraryError::CardQualitySignalNotActive(card_id));
            };

            let evidence = signal
                .evidence
                .ok_or_else(|| LibraryError::CardQualitySignalNotActive(card_id.clone()))?;

            if evidence.observed_through_review_id != observed_through_review_id {
                return Err(LibraryError::CardQualityEvidenceChanged);
            }

            let entity = transaction.create_entity(EntityKind::CardQualityConcern)?;

            transaction.execute(
                "INSERT INTO card_quality_concerns (
                    entity_id,
                    card_id,
                    source,
                    kind,
                    status,
                    note,
                    observed_through_review_id,
                    status_changed_at,
                    last_change_id
                ) VALUES (
                    ?1,
                    ?2,
                    'review_pattern',
                    'too_difficult',
                    'dismissed',
                    '',
                    ?3,
                    ?4,
                    ?5
                )",
                params![
                    entity.id,
                    card_id,
                    observed_through_review_id,
                    entity.created_at,
                    entity.last_change_id,
                ],
            )?;

            query_concern(transaction, &entity.id)
        })
    }
}

fn normalize_id(value: String, field: &'static str) -> LibraryResult<String> {
    let value = value.trim().to_owned();

    if Uuid::parse_str(&value).is_err() {
        return Err(invalid_concern(format!("{field} is not valid")));
    }

    Ok(value)
}

fn normalize_note(note: String) -> LibraryResult<String> {
    let note = note.trim().to_owned();

    if note.chars().count() > MAXIMUM_CARD_QUALITY_NOTE_LENGTH {
        return Err(LibraryError::ValueTooLong {
            field: "Card issue note",
            maximum: MAXIMUM_CARD_QUALITY_NOTE_LENGTH,
        });
    }

    if note.contains('\0') {
        return Err(invalid_concern(
            "note contains an invalid character".to_owned(),
        ));
    }

    Ok(note)
}

fn validate_active_card(connection: &Connection, card_id: &str) -> LibraryResult<()> {
    let active: bool = connection.query_row(
        "SELECT EXISTS (
            SELECT 1
            FROM cards
            INNER JOIN entities AS card_entities
                ON card_entities.id = cards.entity_id
            INNER JOIN concepts
                ON concepts.entity_id = cards.concept_id
            INNER JOIN entities AS concept_entities
                ON concept_entities.id = concepts.entity_id
            LEFT JOIN entities AS template_entities
                ON template_entities.id = cards.template_id
            WHERE cards.entity_id = ?1
                AND card_entities.deleted_at IS NULL
                AND concept_entities.deleted_at IS NULL
                AND concepts.archived_at IS NULL
                AND (
                    cards.template_id IS NULL
                    OR template_entities.deleted_at IS NULL
                )
        )",
        [card_id],
        |row| row.get(0),
    )?;

    if !active {
        return Err(LibraryError::CardNotFound(card_id.to_owned()));
    }

    Ok(())
}

fn query_queue(connection: &Connection) -> LibraryResult<CardQualityQueue> {
    let mut items = query_manual_queue_items(connection)?;

    items.extend(query_review_pattern_items(connection, None)?);
    items.sort_by(|left, right| {
        right
            .noticed_at
            .cmp(&left.noticed_at)
            .then_with(|| left.concept_title.cmp(&right.concept_title))
            .then_with(|| left.card_id.cmp(&right.card_id))
            .then_with(|| left.kind.as_str().cmp(right.kind.as_str()))
    });

    Ok(CardQualityQueue { items })
}

fn query_manual_queue_items(connection: &Connection) -> LibraryResult<Vec<CardQualityQueueItem>> {
    let mut statement = connection.prepare(
        "SELECT
            card_quality_concerns.entity_id,
            cards.entity_id,
            concepts.entity_id,
            concepts.title,
            cards.retrieval_kind,
            card_quality_concerns.kind,
            card_quality_concerns.note,
            concern_entities.created_at,
            cards.configuration_json,
            templates.entity_id AS template_id,
            templates.name AS template_name
        FROM card_quality_concerns
        INNER JOIN entities AS concern_entities
            ON concern_entities.id = card_quality_concerns.entity_id
        INNER JOIN cards
            ON cards.entity_id = card_quality_concerns.card_id
        INNER JOIN entities AS card_entities
            ON card_entities.id = cards.entity_id
        INNER JOIN concepts
            ON concepts.entity_id = cards.concept_id
        INNER JOIN entities AS concept_entities
            ON concept_entities.id = concepts.entity_id
        LEFT JOIN entities AS template_entities
            ON template_entities.id = cards.template_id
        LEFT JOIN templates
            ON templates.entity_id = cards.template_id
        WHERE card_quality_concerns.source = 'manual'
            AND card_quality_concerns.status = 'open'
            AND concern_entities.deleted_at IS NULL
            AND card_entities.deleted_at IS NULL
            AND concept_entities.deleted_at IS NULL
            AND concepts.archived_at IS NULL
            AND (
                cards.template_id IS NULL
                OR template_entities.deleted_at IS NULL
            )
        ORDER BY concern_entities.created_at, card_quality_concerns.entity_id",
    )?;
    let items = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, i64>(7)?,
                row.get::<_, String>(8)?,
                read_template(row)?,
            ))
        })?
        .map(|row| {
            let (
                concern_id,
                card_id,
                concept_id,
                concept_title,
                retrieval_kind,
                kind,
                note,
                noticed_at,
                configuration,
                template,
            ) = row?;
            let retrieval_kind = RetrievalFormKind::try_from(retrieval_kind.as_str())?;
            let parsed = parse_retrieval_form_configuration(retrieval_kind, &configuration)?;

            Ok(CardQualityQueueItem {
                concern_id: Some(concern_id),
                card_id,
                concept_id,
                concept_title,
                retrieval_kind,
                template,
                cloze: parsed.cloze,
                image_occlusion: parsed.image_occlusion,
                source: CardQualitySource::Manual,
                kind: CardQualityKind::try_from(kind.as_str())?,
                note,
                noticed_at,
                evidence: None,
            })
        })
        .collect::<LibraryResult<Vec<_>>>()?;

    Ok(items)
}

fn query_review_pattern_items(
    connection: &Connection,
    card_id: Option<&str>,
) -> LibraryResult<Vec<CardQualityQueueItem>> {
    let mut statement = connection.prepare(
        "WITH ranked_effective_reviews AS (
            SELECT
                effective_reviews.card_id,
                effective_reviews.entity_id,
                effective_reviews.rating,
                effective_reviews.reviewed_at,
                ROW_NUMBER() OVER (
                    PARTITION BY effective_reviews.card_id
                    ORDER BY effective_reviews.reviewed_at DESC,
                        effective_reviews.entity_id DESC
                ) AS review_position
            FROM effective_reviews
            WHERE (?3 IS NULL OR effective_reviews.card_id = ?3)
        ),
        recent_patterns AS (
            SELECT
                card_id,
                COUNT(*) AS reviews_considered,
                SUM(CASE WHEN rating = 1 THEN 1 ELSE 0 END) AS again_count,
                MAX(
                    CASE WHEN review_position = 1 THEN entity_id END
                ) AS observed_through_review_id,
                MAX(
                    CASE WHEN review_position = 1 THEN reviewed_at END
                ) AS noticed_at
            FROM ranked_effective_reviews
            WHERE review_position <= ?1
            GROUP BY card_id
            HAVING SUM(CASE WHEN rating = 1 THEN 1 ELSE 0 END) >= ?2
        ),
        ranked_dismissals AS (
            SELECT
                card_quality_concerns.card_id,
                card_quality_concerns.observed_through_review_id,
                ROW_NUMBER() OVER (
                    PARTITION BY card_quality_concerns.card_id
                    ORDER BY observed_reviews.reviewed_at DESC,
                        observed_reviews.entity_id DESC,
                        card_quality_concerns.entity_id DESC
                ) AS dismissal_position
            FROM card_quality_concerns
            INNER JOIN entities AS concern_entities
                ON concern_entities.id = card_quality_concerns.entity_id
            INNER JOIN reviews AS observed_reviews
                ON observed_reviews.entity_id =
                    card_quality_concerns.observed_through_review_id
            WHERE card_quality_concerns.source = 'review_pattern'
                AND card_quality_concerns.status = 'dismissed'
                AND concern_entities.deleted_at IS NULL
        )
        SELECT
            cards.entity_id,
            concepts.entity_id,
            concepts.title,
            cards.retrieval_kind,
            recent_patterns.reviews_considered,
            recent_patterns.again_count,
            recent_patterns.observed_through_review_id,
            recent_patterns.noticed_at,
            cards.configuration_json,
            templates.entity_id AS template_id,
            templates.name AS template_name
        FROM recent_patterns
        INNER JOIN cards
            ON cards.entity_id = recent_patterns.card_id
        INNER JOIN entities AS card_entities
            ON card_entities.id = cards.entity_id
        INNER JOIN concepts
            ON concepts.entity_id = cards.concept_id
        INNER JOIN entities AS concept_entities
            ON concept_entities.id = concepts.entity_id
        LEFT JOIN entities AS template_entities
            ON template_entities.id = cards.template_id
        LEFT JOIN templates
            ON templates.entity_id = cards.template_id
        LEFT JOIN ranked_dismissals
            ON ranked_dismissals.card_id = cards.entity_id
            AND ranked_dismissals.dismissal_position = 1
        LEFT JOIN reviews AS dismissed_through_review
            ON dismissed_through_review.entity_id =
                ranked_dismissals.observed_through_review_id
        WHERE (?3 IS NULL OR cards.entity_id = ?3)
            AND card_entities.deleted_at IS NULL
            AND concept_entities.deleted_at IS NULL
            AND concepts.archived_at IS NULL
            AND (
                cards.template_id IS NULL
                OR template_entities.deleted_at IS NULL
            )
            AND (
                ranked_dismissals.card_id IS NULL
                OR (
                    SELECT COUNT(*)
                    FROM effective_reviews AS fresh_reviews
                    WHERE fresh_reviews.card_id = cards.entity_id
                        AND fresh_reviews.rating = 1
                        AND (
                            fresh_reviews.reviewed_at >
                                dismissed_through_review.reviewed_at
                            OR (
                                fresh_reviews.reviewed_at =
                                    dismissed_through_review.reviewed_at
                                AND fresh_reviews.entity_id >
                                    dismissed_through_review.entity_id
                            )
                        )
                ) >= ?2
            )
        ORDER BY recent_patterns.noticed_at DESC, cards.entity_id",
    )?;
    let items = statement
        .query_map(
            params![
                REPEATED_DIFFICULTY_REVIEW_WINDOW,
                REPEATED_DIFFICULTY_AGAIN_COUNT,
                card_id,
            ],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, i64>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, i64>(7)?,
                    row.get::<_, String>(8)?,
                    read_template(row)?,
                ))
            },
        )?
        .map(|row| {
            let (
                card_id,
                concept_id,
                concept_title,
                retrieval_kind,
                reviews_considered,
                again_count,
                observed_through_review_id,
                noticed_at,
                configuration,
                template,
            ) = row?;
            let retrieval_kind = RetrievalFormKind::try_from(retrieval_kind.as_str())?;
            let parsed = parse_retrieval_form_configuration(retrieval_kind, &configuration)?;

            Ok(CardQualityQueueItem {
                concern_id: None,
                card_id,
                concept_id,
                concept_title,
                retrieval_kind,
                template,
                cloze: parsed.cloze,
                image_occlusion: parsed.image_occlusion,
                source: CardQualitySource::ReviewPattern,
                kind: CardQualityKind::TooDifficult,
                note: String::new(),
                noticed_at,
                evidence: Some(CardQualityEvidence {
                    reviews_considered,
                    again_count,
                    observed_through_review_id,
                }),
            })
        })
        .collect::<LibraryResult<Vec<_>>>()?;

    Ok(items)
}

fn query_open_manual_concern(
    connection: &Connection,
    card_id: &str,
    kind: CardQualityKind,
) -> LibraryResult<Option<CardQualityConcern>> {
    let concern_id = connection
        .query_row(
            "SELECT card_quality_concerns.entity_id
            FROM card_quality_concerns
            INNER JOIN entities
                ON entities.id = card_quality_concerns.entity_id
            WHERE card_quality_concerns.card_id = ?1
                AND card_quality_concerns.source = 'manual'
                AND card_quality_concerns.kind = ?2
                AND card_quality_concerns.status = 'open'
                AND entities.deleted_at IS NULL
            ORDER BY entities.created_at, card_quality_concerns.entity_id
            LIMIT 1",
            params![card_id, kind.as_str()],
            |row| row.get::<_, String>(0),
        )
        .optional()?;

    concern_id
        .map(|concern_id| query_concern(connection, &concern_id))
        .transpose()
}

fn query_signal_dismissal(
    connection: &Connection,
    card_id: &str,
    signal: CardQualitySignal,
    observed_through_review_id: &str,
) -> LibraryResult<Option<CardQualityConcern>> {
    let concern_id = connection
        .query_row(
            "SELECT card_quality_concerns.entity_id
            FROM card_quality_concerns
            INNER JOIN entities
                ON entities.id = card_quality_concerns.entity_id
            INNER JOIN reviews
                ON reviews.entity_id =
                    card_quality_concerns.observed_through_review_id
            WHERE card_quality_concerns.card_id = ?1
                AND card_quality_concerns.source = 'review_pattern'
                AND card_quality_concerns.kind = ?2
                AND card_quality_concerns.status = 'dismissed'
                AND card_quality_concerns.observed_through_review_id = ?3
                AND entities.deleted_at IS NULL
            ORDER BY reviews.reviewed_at DESC,
                reviews.entity_id DESC,
                card_quality_concerns.entity_id DESC
            LIMIT 1",
            params![card_id, signal.kind().as_str(), observed_through_review_id],
            |row| row.get::<_, String>(0),
        )
        .optional()?;

    concern_id
        .map(|concern_id| query_concern(connection, &concern_id))
        .transpose()
}

fn query_concern(connection: &Connection, concern_id: &str) -> LibraryResult<CardQualityConcern> {
    let stored = connection
        .query_row(
            "SELECT
                card_quality_concerns.entity_id,
                card_quality_concerns.card_id,
                card_quality_concerns.source,
                card_quality_concerns.kind,
                card_quality_concerns.status,
                card_quality_concerns.note,
                card_quality_concerns.observed_through_review_id,
                entities.created_at,
                entities.updated_at
            FROM card_quality_concerns
            INNER JOIN entities
                ON entities.id = card_quality_concerns.entity_id
            WHERE card_quality_concerns.entity_id = ?1
                AND entities.deleted_at IS NULL",
            [concern_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, Option<String>>(6)?,
                    row.get::<_, i64>(7)?,
                    row.get::<_, i64>(8)?,
                ))
            },
        )
        .optional()?
        .ok_or_else(|| LibraryError::CardQualityConcernNotFound(concern_id.to_owned()))?;

    Ok(CardQualityConcern {
        id: stored.0,
        card_id: stored.1,
        source: CardQualitySource::try_from(stored.2.as_str())?,
        kind: CardQualityKind::try_from(stored.3.as_str())?,
        status: CardQualityStatus::try_from(stored.4.as_str())?,
        note: stored.5,
        observed_through_review_id: stored.6,
        created_at: stored.7,
        updated_at: stored.8,
    })
}

fn invalid_concern(message: String) -> LibraryError {
    LibraryError::InvalidCardQualityConcern { message }
}

fn read_template(row: &rusqlite::Row<'_>) -> rusqlite::Result<Option<NamedItem>> {
    match (
        row.get::<_, Option<String>>("template_id")?,
        row.get::<_, Option<String>>("template_name")?,
    ) {
        (Some(id), Some(name)) => Ok(Some(NamedItem { id, name })),
        (None, None) => Ok(None),
        _ => Err(rusqlite::Error::InvalidQuery),
    }
}

impl CardQualityKind {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Ambiguous => "ambiguous",
            Self::Incorrect => "incorrect",
            Self::Outdated => "outdated",
            Self::TooEasy => "too_easy",
            Self::TooDifficult => "too_difficult",
        }
    }
}

impl TryFrom<&str> for CardQualityKind {
    type Error = LibraryError;

    fn try_from(value: &str) -> LibraryResult<Self> {
        match value {
            "ambiguous" => Ok(Self::Ambiguous),
            "incorrect" => Ok(Self::Incorrect),
            "outdated" => Ok(Self::Outdated),
            "too_easy" => Ok(Self::TooEasy),
            "too_difficult" => Ok(Self::TooDifficult),
            _ => Err(LibraryError::InvalidCardQualityKind(value.to_owned())),
        }
    }
}

impl TryFrom<&str> for CardQualitySource {
    type Error = LibraryError;

    fn try_from(value: &str) -> LibraryResult<Self> {
        match value {
            "manual" => Ok(Self::Manual),
            "review_pattern" => Ok(Self::ReviewPattern),
            _ => Err(LibraryError::InvalidCardQualitySource(value.to_owned())),
        }
    }
}

impl CardQualityStatus {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Resolved => "resolved",
            Self::Dismissed => "dismissed",
        }
    }
}

impl TryFrom<&str> for CardQualityStatus {
    type Error = LibraryError;

    fn try_from(value: &str) -> LibraryResult<Self> {
        match value {
            "open" => Ok(Self::Open),
            "resolved" => Ok(Self::Resolved),
            "dismissed" => Ok(Self::Dismissed),
            _ => Err(LibraryError::InvalidCardQualityStatus(value.to_owned())),
        }
    }
}

impl CardQualityDisposition {
    const fn status(self) -> CardQualityStatus {
        match self {
            Self::Resolved => CardQualityStatus::Resolved,
            Self::Dismissed => CardQualityStatus::Dismissed,
        }
    }
}

impl CardQualitySignal {
    const fn kind(self) -> CardQualityKind {
        match self {
            Self::RepeatedDifficulty => CardQualityKind::TooDifficult,
        }
    }

    fn matches(self, item: &CardQualityQueueItem) -> bool {
        match self {
            Self::RepeatedDifficulty => {
                item.source == CardQualitySource::ReviewPattern
                    && item.kind == CardQualityKind::TooDifficult
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use image::{DynamicImage, ImageFormat};
    use serde_json::json;
    use tempfile::tempdir;
    use uuid::Uuid;

    use super::CardQualityLibrary;
    use crate::data::{ChangeOperation, EntityKind, LocalDataStore};
    use crate::library::study::{record_review, reverse_review};
    use crate::library::{
        CardQualityDisposition, CardQualityKind, CardQualitySignal, CardQualitySource,
        CardQualityStatus, CloseCardQualityConcernInput, ConceptContent, ConceptLibrary,
        CreateCardQualityConcernInput, CreateConceptInput, CreateTemplateInput,
        DismissCardQualitySignalInput, LibraryError, ReviewOutcome, ReviewRating, TemplateLibrary,
    };

    fn create_card(library: &ConceptLibrary<'_>, title: &str) -> crate::library::ConceptDetail {
        library
            .create_concept(CreateConceptInput {
                title: title.to_owned(),
                deck_ids: Vec::new(),
                tag_ids: Vec::new(),
                content: Default::default(),
                include_standard_recall: true,
                template_ids: Vec::new(),
                explain: None,
                problem: None,
                type_answer: None,
            })
            .unwrap()
    }

    fn record_rating(
        store: &LocalDataStore,
        card_id: &str,
        rating: ReviewRating,
        reviewed_at: i64,
    ) -> ReviewOutcome {
        store
            .write_result(|transaction| record_review(transaction, card_id, rating, reviewed_at))
            .unwrap()
    }

    fn create_concern_input(
        card_id: &str,
        kind: CardQualityKind,
        note: &str,
    ) -> CreateCardQualityConcernInput {
        CreateCardQualityConcernInput {
            card_id: card_id.to_owned(),
            kind,
            note: note.to_owned(),
        }
    }

    #[test]
    fn manual_concerns_are_sync_ready_and_close_without_changing_content() {
        let directory = tempdir().unwrap();
        let store = LocalDataStore::open(directory.path()).unwrap();
        let concepts = ConceptLibrary::new(&store);
        let quality = CardQualityLibrary::new(&store);
        let concept = create_card(&concepts, "Ambiguous prompt");
        let card = &concept.cards[0];
        let initial_change_count = store.changes_after(0, 100).unwrap().len();

        let ambiguous = quality
            .create_concern(create_concern_input(
                &card.id,
                CardQualityKind::Ambiguous,
                "  The subject is unclear.  ",
            ))
            .unwrap();
        let duplicate = quality
            .create_concern(create_concern_input(
                &card.id,
                CardQualityKind::Ambiguous,
                "The subject is unclear.",
            ))
            .unwrap();
        let incorrect = quality
            .create_concern(create_concern_input(
                &card.id,
                CardQualityKind::Incorrect,
                "",
            ))
            .unwrap();

        assert_eq!(ambiguous, duplicate);
        assert_eq!(ambiguous.note, "The subject is unclear.");
        assert_eq!(ambiguous.source, CardQualitySource::Manual);
        assert_eq!(ambiguous.status, CardQualityStatus::Open);
        assert!(matches!(
            quality.create_concern(create_concern_input(
                &card.id,
                CardQualityKind::Ambiguous,
                "A distinct later observation.",
            )),
            Err(LibraryError::CardQualityConcernConflict)
        ));
        assert_eq!(
            store.entity(&ambiguous.id).unwrap().unwrap().kind,
            EntityKind::CardQualityConcern
        );

        let queue = quality.queue().unwrap();

        assert_eq!(queue.items.len(), 2);
        assert!(queue.items.iter().all(|item| item.card_id == card.id));
        assert!(queue.items.iter().all(|item| item.concept_id == concept.id));
        assert!(queue.items.iter().all(|item| item.evidence.is_none()));

        let resolved = quality
            .close_concern(CloseCardQualityConcernInput {
                concern_id: ambiguous.id.clone(),
                disposition: CardQualityDisposition::Resolved,
            })
            .unwrap();
        let resolved_again = quality
            .close_concern(CloseCardQualityConcernInput {
                concern_id: ambiguous.id.clone(),
                disposition: CardQualityDisposition::Resolved,
            })
            .unwrap();
        let dismissed = quality
            .close_concern(CloseCardQualityConcernInput {
                concern_id: incorrect.id.clone(),
                disposition: CardQualityDisposition::Dismissed,
            })
            .unwrap();

        assert_eq!(resolved.status, CardQualityStatus::Resolved);
        assert_eq!(resolved, resolved_again);
        assert_eq!(dismissed.status, CardQualityStatus::Dismissed);
        assert!(quality.queue().unwrap().items.is_empty());
        assert!(matches!(
            quality.close_concern(CloseCardQualityConcernInput {
                concern_id: ambiguous.id.clone(),
                disposition: CardQualityDisposition::Dismissed,
            }),
            Err(LibraryError::CardQualityConcernClosed(id)) if id == ambiguous.id
        ));

        let changes = store.changes_after(0, 100).unwrap();
        let quality_changes = &changes[initial_change_count..];

        assert_eq!(quality_changes.len(), 4);
        assert_eq!(
            quality_changes
                .iter()
                .map(|change| change.operation)
                .collect::<Vec<_>>(),
            vec![
                ChangeOperation::Create,
                ChangeOperation::Create,
                ChangeOperation::Update,
                ChangeOperation::Update,
            ]
        );
        assert_eq!(
            concepts.concept(&concept.id).unwrap().content,
            concept.content
        );
    }

    #[test]
    fn quality_input_is_bounded_and_inactive_cards_stay_out_of_the_queue() {
        let directory = tempdir().unwrap();
        let store = LocalDataStore::open(directory.path()).unwrap();
        let concepts = ConceptLibrary::new(&store);
        let quality = CardQualityLibrary::new(&store);
        let concept = create_card(&concepts, "Archived concern");
        let card_id = concept.cards[0].id.clone();

        assert!(matches!(
            quality.create_concern(create_concern_input(
                "not-an-id",
                CardQualityKind::Outdated,
                "",
            )),
            Err(LibraryError::InvalidCardQualityConcern { .. })
        ));
        assert!(matches!(
            quality.create_concern(create_concern_input(
                &card_id,
                CardQualityKind::Outdated,
                &"x".repeat(501),
            )),
            Err(LibraryError::ValueTooLong { .. })
        ));
        assert!(matches!(
            quality.create_concern(create_concern_input(
                &card_id,
                CardQualityKind::Outdated,
                "invalid\0note",
            )),
            Err(LibraryError::InvalidCardQualityConcern { .. })
        ));

        quality
            .create_concern(create_concern_input(
                &card_id,
                CardQualityKind::Outdated,
                &"é".repeat(500),
            ))
            .unwrap();

        assert_eq!(quality.queue().unwrap().items.len(), 1);

        concepts.set_concept_archived(&concept.id, true).unwrap();

        assert!(quality.queue().unwrap().items.is_empty());
        assert!(matches!(
            quality.create_concern(create_concern_input(
                &card_id,
                CardQualityKind::TooEasy,
                "",
            )),
            Err(LibraryError::CardNotFound(id)) if id == card_id
        ));

        concepts.set_concept_archived(&concept.id, false).unwrap();

        assert_eq!(quality.queue().unwrap().items.len(), 1);

        concepts.delete_concept(&concept.id).unwrap();

        assert!(quality.queue().unwrap().items.is_empty());
    }

    #[test]
    fn repeated_difficulty_is_transparent_and_ignores_reversed_reviews() {
        let directory = tempdir().unwrap();
        let store = LocalDataStore::open(directory.path()).unwrap();
        let concepts = ConceptLibrary::new(&store);
        let quality = CardQualityLibrary::new(&store);
        let concept = create_card(&concepts, "Difficult retrieval");
        let card = &concept.cards[0];
        let first = record_rating(&store, &card.id, ReviewRating::Again, card.due_at);
        let second = record_rating(&store, &card.id, ReviewRating::Again, first.due_at);

        assert!(quality.queue().unwrap().items.is_empty());

        let third = record_rating(&store, &card.id, ReviewRating::Again, second.due_at);
        let queue = quality.queue().unwrap();

        assert_eq!(queue.items.len(), 1);
        assert_eq!(queue.items[0].source, CardQualitySource::ReviewPattern);
        assert_eq!(queue.items[0].kind, CardQualityKind::TooDifficult);
        assert_eq!(queue.items[0].concern_id, None);
        assert_eq!(
            queue.items[0].evidence,
            Some(crate::library::CardQualityEvidence {
                reviews_considered: 3,
                again_count: 3,
                observed_through_review_id: third.review_id.clone(),
            })
        );

        store
            .write_result(|transaction| {
                reverse_review(transaction, &third.review_id, third.reviewed_at + 1)
            })
            .unwrap();

        assert!(quality.queue().unwrap().items.is_empty());
    }

    #[test]
    fn dismissing_a_review_pattern_requires_fresh_evidence_before_it_returns() {
        let directory = tempdir().unwrap();
        let store = LocalDataStore::open(directory.path()).unwrap();
        let concepts = ConceptLibrary::new(&store);
        let quality = CardQualityLibrary::new(&store);
        let concept = create_card(&concepts, "Repeated trouble");
        let card = &concept.cards[0];
        let mut due_at = card.due_at;

        for _ in 0..3 {
            due_at = record_rating(&store, &card.id, ReviewRating::Again, due_at).due_at;
        }

        let observed_through_review_id = quality.queue().unwrap().items[0]
            .evidence
            .as_ref()
            .unwrap()
            .observed_through_review_id
            .clone();
        let dismissal = quality
            .dismiss_signal(DismissCardQualitySignalInput {
                card_id: card.id.clone(),
                signal: CardQualitySignal::RepeatedDifficulty,
                observed_through_review_id: observed_through_review_id.clone(),
            })
            .unwrap();
        let duplicate = quality
            .dismiss_signal(DismissCardQualitySignalInput {
                card_id: card.id.clone(),
                signal: CardQualitySignal::RepeatedDifficulty,
                observed_through_review_id: observed_through_review_id.clone(),
            })
            .unwrap();

        assert_eq!(dismissal, duplicate);
        assert_eq!(dismissal.source, CardQualitySource::ReviewPattern);
        assert_eq!(dismissal.status, CardQualityStatus::Dismissed);
        assert!(dismissal.observed_through_review_id.is_some());
        assert!(quality.queue().unwrap().items.is_empty());

        for _ in 0..2 {
            due_at = record_rating(&store, &card.id, ReviewRating::Again, due_at).due_at;

            assert!(quality.queue().unwrap().items.is_empty());
        }

        record_rating(&store, &card.id, ReviewRating::Again, due_at);

        let queue = quality.queue().unwrap();

        assert_eq!(queue.items.len(), 1);
        assert_eq!(queue.items[0].source, CardQualitySource::ReviewPattern);
        assert_eq!(queue.items[0].evidence.as_ref().unwrap().again_count, 5);

        let changes_before_retry = store.changes_after(0, 100).unwrap();
        let delayed_retry = quality
            .dismiss_signal(DismissCardQualitySignalInput {
                card_id: card.id.clone(),
                signal: CardQualitySignal::RepeatedDifficulty,
                observed_through_review_id,
            })
            .unwrap();

        assert_eq!(delayed_retry, dismissal);
        assert_eq!(quality.queue().unwrap(), queue);
        assert_eq!(store.changes_after(0, 100).unwrap(), changes_before_retry);

        drop(store);

        let reopened = LocalDataStore::open(directory.path()).unwrap();

        assert_eq!(CardQualityLibrary::new(&reopened).queue().unwrap(), queue);
    }

    #[test]
    fn stale_dismissals_do_not_consume_new_evidence_and_old_patterns_expire() {
        let directory = tempdir().unwrap();
        let store = LocalDataStore::open(directory.path()).unwrap();
        let concepts = ConceptLibrary::new(&store);
        let quality = CardQualityLibrary::new(&store);
        let concept = create_card(&concepts, "Changing evidence");
        let card = &concept.cards[0];
        let mut due_at = card.due_at;

        for _ in 0..3 {
            due_at = record_rating(&store, &card.id, ReviewRating::Again, due_at).due_at;
        }

        let old_evidence = quality.queue().unwrap().items[0].evidence.clone().unwrap();

        due_at = record_rating(&store, &card.id, ReviewRating::Good, due_at).due_at;

        let changes_before_dismissal = store.changes_after(0, 100).unwrap();

        assert!(matches!(
            quality.dismiss_signal(DismissCardQualitySignalInput {
                card_id: card.id.clone(),
                signal: CardQualitySignal::RepeatedDifficulty,
                observed_through_review_id: old_evidence.observed_through_review_id.clone(),
            }),
            Err(LibraryError::CardQualityEvidenceChanged)
        ));
        assert_eq!(
            store.changes_after(0, 100).unwrap(),
            changes_before_dismissal
        );
        assert_eq!(quality.queue().unwrap().items.len(), 1);

        for _ in 0..2 {
            due_at = record_rating(&store, &card.id, ReviewRating::Good, due_at).due_at;
        }

        assert!(quality.queue().unwrap().items.is_empty());
        assert!(matches!(
            quality.dismiss_signal(DismissCardQualitySignalInput {
                card_id: card.id.clone(),
                signal: CardQualitySignal::RepeatedDifficulty,
                observed_through_review_id: old_evidence.observed_through_review_id,
            }),
            Err(LibraryError::CardQualitySignalNotActive(_))
        ));
    }

    #[test]
    fn queue_preserves_template_and_group_identity_for_each_kind_of_concern() {
        let directory = tempdir().unwrap();
        let store = LocalDataStore::open(directory.path()).unwrap();
        let concepts = ConceptLibrary::new(&store);
        let quality = CardQualityLibrary::new(&store);
        let template = TemplateLibrary::new(&store)
            .create_template(CreateTemplateInput {
                name: "Answer first".to_owned(),
                content: Default::default(),
            })
            .unwrap();
        let cloze_group = Uuid::now_v7().to_string();
        let image_group = Uuid::now_v7().to_string();
        let mut bytes = Cursor::new(Vec::new());

        DynamicImage::new_rgba8(4, 3)
            .write_to(&mut bytes, ImageFormat::Png)
            .unwrap();

        let media = concepts.import_image(&bytes.into_inner()).unwrap();
        let concept = concepts
            .create_concept(CreateConceptInput {
                title: "Multiple forms".to_owned(),
                deck_ids: Vec::new(),
                tag_ids: Vec::new(),
                content: ConceptContent {
                    prompt: json!({
                        "type": "doc",
                        "content": [{
                            "type": "paragraph",
                            "content": [{
                                "type": "text", "text": "An important detail",
                                "marks": [{
                                    "type": "cloze", "attrs": { "groupId": cloze_group }
                                }]
                            }]
                        }, {
                            "type": "mediaImage",
                            "attrs": {
                                "mediaId": media.id, "alt": "Diagram",
                                "occlusionRegions": [{
                                    "id": Uuid::now_v7().to_string(),
                                    "groupId": image_group,
                                    "x": 0.1, "y": 0.1, "width": 0.2, "height": 0.2
                                }]
                            }
                        }]
                    }),
                    ..Default::default()
                },
                include_standard_recall: true,
                template_ids: vec![template.id.clone()],
                explain: None,
                problem: None,
                type_answer: None,
            })
            .unwrap();

        assert_eq!(concept.cards.len(), 4);

        for card in &concept.cards {
            quality
                .create_concern(create_concern_input(
                    &card.id,
                    CardQualityKind::Ambiguous,
                    "Check this form",
                ))
                .unwrap();

            let mut due_at = card.due_at;

            for _ in 0..3 {
                due_at = record_rating(&store, &card.id, ReviewRating::Again, due_at).due_at;
            }
        }

        let queue = quality.queue().unwrap();

        assert_eq!(queue.items.len(), 8);

        for item in &queue.items {
            let card = concept
                .cards
                .iter()
                .find(|card| card.id == item.card_id)
                .unwrap();

            assert_eq!(item.retrieval_kind, card.retrieval_kind);
            assert_eq!(item.template, card.template);
            assert_eq!(item.cloze, card.cloze);
            assert_eq!(item.image_occlusion, card.image_occlusion);
        }

        concepts.set_concept_archived(&concept.id, true).unwrap();

        assert!(quality.queue().unwrap().items.is_empty());

        concepts.set_concept_archived(&concept.id, false).unwrap();

        assert_eq!(quality.queue().unwrap(), queue);
    }

    #[test]
    fn concern_history_cannot_be_hard_deleted_or_rewritten() {
        let directory = tempdir().unwrap();
        let store = LocalDataStore::open(directory.path()).unwrap();
        let concepts = ConceptLibrary::new(&store);
        let quality = CardQualityLibrary::new(&store);
        let concept = create_card(&concepts, "Durable concern");
        let concern = quality
            .create_concern(create_concern_input(
                &concept.cards[0].id,
                CardQualityKind::Incorrect,
                "The expected answer is wrong.",
            ))
            .unwrap();

        assert!(store
            .write(|transaction| {
                transaction.execute(
                    "UPDATE card_quality_concerns
                    SET note = 'Rewritten'
                    WHERE entity_id = ?1",
                    [&concern.id],
                )?;

                Ok(())
            })
            .is_err());
        assert!(store
            .write(|transaction| {
                transaction.execute(
                    "DELETE FROM card_quality_concerns WHERE entity_id = ?1",
                    [&concern.id],
                )?;

                Ok(())
            })
            .is_err());
    }
}
