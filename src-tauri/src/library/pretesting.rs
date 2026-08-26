use rusqlite::{params, Connection, OptionalExtension};

use crate::data::{EntityKind, WriteTransaction};
use crate::library::{
    LibraryError, LibraryResult, PretestOutcome, PretestRecord,
};

pub fn record_pretest(
    transaction: &WriteTransaction<'_>,
    card_id: &str,
    outcome: PretestOutcome,
    now: i64,
) -> LibraryResult<PretestRecord> {
    let concept_id = transaction
        .query_row(
            "SELECT cards.concept_id
            FROM cards
            INNER JOIN entities AS card_entities
                ON card_entities.id = cards.entity_id
            INNER JOIN concepts
                ON concepts.entity_id = cards.concept_id
            INNER JOIN entities AS concept_entities
                ON concept_entities.id = concepts.entity_id
            INNER JOIN card_scheduling
                ON card_scheduling.card_id = cards.entity_id
            LEFT JOIN entities AS template_entities
                ON template_entities.id = cards.template_id
            WHERE cards.entity_id = ?1
                AND card_entities.deleted_at IS NULL
                AND concept_entities.deleted_at IS NULL
                AND concepts.archived_at IS NULL
                AND (
                    cards.template_id IS NULL
                    OR template_entities.deleted_at IS NULL
                )",
            [card_id],
            |row| row.get::<_, String>(0),
        )
        .optional()?
        .ok_or_else(|| LibraryError::CardNotFound(card_id.to_owned()))?;

    if let Some(existing) = query_concept_pretest(transaction, &concept_id)? {
        return Ok(existing);
    }

    let eligible = transaction.query_row(
        "SELECT EXISTS (
            SELECT 1
            FROM card_scheduling
            WHERE card_id = ?1
                AND state = 'new'
                AND due_at <= ?2
                AND NOT EXISTS (
                    SELECT 1
                    FROM reviews
                    INNER JOIN cards AS reviewed_cards
                        ON reviewed_cards.entity_id = reviews.card_id
                    WHERE reviewed_cards.concept_id = ?3
                )
        )",
        params![card_id, now, concept_id],
        |row| row.get::<_, bool>(0),
    )?;

    if !eligible {
        return Err(LibraryError::PretestNotEligible(card_id.to_owned()));
    }

    let entity = transaction.create_entity_at(EntityKind::Pretest, now)?;

    transaction.execute(
        "INSERT INTO pretests (
            entity_id,
            concept_id,
            card_id,
            outcome,
            occurred_at,
            last_change_id
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            entity.id,
            concept_id,
            card_id,
            outcome.as_str(),
            now,
            entity.last_change_id,
        ],
    )?;

    Ok(PretestRecord {
        pretest_id: entity.id,
        concept_id,
        card_id: card_id.to_owned(),
        outcome,
        occurred_at: now,
    })
}

fn query_concept_pretest(
    connection: &Connection,
    concept_id: &str,
) -> LibraryResult<Option<PretestRecord>> {
    let stored = connection
        .query_row(
            "SELECT entity_id, concept_id, card_id, outcome, occurred_at
            FROM pretests
            WHERE concept_id = ?1
            ORDER BY occurred_at, entity_id
            LIMIT 1",
            [concept_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, i64>(4)?,
                ))
            },
        )
        .optional()?;

    stored
        .map(|stored| {
            Ok(PretestRecord {
                pretest_id: stored.0,
                concept_id: stored.1,
                card_id: stored.2,
                outcome: PretestOutcome::try_from(stored.3.as_str())?,
                occurred_at: stored.4,
            })
        })
        .transpose()
}

impl PretestOutcome {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Attempted => "attempted",
            Self::Skipped => "skipped",
        }
    }
}

impl TryFrom<&str> for PretestOutcome {
    type Error = LibraryError;

    fn try_from(value: &str) -> LibraryResult<Self> {
        match value {
            "attempted" => Ok(Self::Attempted),
            "skipped" => Ok(Self::Skipped),
            _ => Err(LibraryError::InvalidPretestOutcome(value.to_owned())),
        }
    }
}
