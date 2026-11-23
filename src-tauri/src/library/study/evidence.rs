use rusqlite::{params, Connection, OptionalExtension};

use crate::data::{EntityKind, WriteTransaction};
use crate::library::{
    LearningEvidence, LibraryError, LibraryResult, RecordExposureInput, StudyWorkload,
};

use super::{history::query_content_version, MILLISECONDS_PER_DAY};

pub fn record_exposure(
    transaction: &WriteTransaction<'_>,
    input: RecordExposureInput,
    now: i64,
) -> LibraryResult<String> {
    let exists: bool = transaction.query_row(
        "SELECT EXISTS (SELECT 1 FROM entities
        WHERE id = ?1 AND kind = 'concept' AND deleted_at IS NULL)",
        [&input.concept_id],
        |row| row.get(0),
    )?;

    if !exists {
        return Err(LibraryError::ConceptNotFound(input.concept_id));
    }

    let recall_kind = match (&input.card_id, &input.content_version) {
        (Some(card_id), Some(version)) => {
            if query_content_version(transaction, card_id)? != *version {
                return Err(LibraryError::StudyContentChanged);
            }

            Some(recall_kind(
                transaction,
                card_id,
                &input.concept_id,
                version,
                now,
            )?)
        }
        (Some(card_id), None) => {
            query_content_version(transaction, card_id)?;
            None
        }
        (None, None) => None,
        _ => return Err(LibraryError::StudyContentChanged),
    };
    let entity = transaction.create_entity_at(EntityKind::PracticeExposure, now)?;

    transaction.execute(
        "INSERT INTO practice_exposures
        (entity_id, concept_id, card_id, content_version, recall_kind, occurred_at, last_change_id)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            entity.id,
            input.concept_id,
            input.card_id,
            input.content_version,
            recall_kind,
            now,
            entity.last_change_id
        ],
    )?;

    Ok(entity.id)
}

fn recall_kind(
    connection: &Connection,
    card_id: &str,
    concept_id: &str,
    version: &str,
    now: i64,
) -> LibraryResult<&'static str> {
    let previous_review: bool = connection.query_row(
        "SELECT EXISTS (SELECT 1 FROM effective_reviews
        WHERE card_id = ?1 AND content_version = ?2)",
        params![card_id, version],
        |row| row.get(0),
    )?;

    if !previous_review {
        return Ok("first");
    }

    // Reversed grades still exposed the answer. Direct linked cases can also cue recall.
    let last_exposure: Option<i64> = connection.query_row(
        "WITH related(id) AS (
            SELECT ?1
            UNION
            SELECT CASE WHEN first_concept_id = ?1 THEN second_concept_id ELSE first_concept_id END
            FROM practice_links
            INNER JOIN entities ON entities.id = practice_links.entity_id
            WHERE entities.deleted_at IS NULL
                AND (first_concept_id = ?1 OR second_concept_id = ?1)
        )
        SELECT MAX(time) FROM (
            SELECT occurred_at AS time FROM practice_exposures
                WHERE concept_id IN related
                    AND (card_id IS NULL OR card_id != ?2 OR recall_kind IS NOT NULL)
            UNION ALL
            SELECT reviewed_at FROM reviews INNER JOIN cards ON cards.entity_id = reviews.card_id
                WHERE cards.concept_id IN related
            UNION ALL
            SELECT occurred_at FROM pretests WHERE concept_id IN related AND outcome = 'attempted'
            UNION ALL
            SELECT updated_at FROM entities WHERE id IN related
            UNION ALL
            SELECT updated_at FROM entities WHERE id = ?2
                OR id = (SELECT template_id FROM cards WHERE entity_id = ?2)
        )",
        params![concept_id, card_id],
        |row| row.get(0),
    )?;

    Ok(
        if last_exposure.is_some_and(|last| now.saturating_sub(last) >= MILLISECONDS_PER_DAY) {
            "delayed"
        } else {
            "recent"
        },
    )
}

pub(super) fn review_context(
    connection: &Connection,
    card_id: &str,
    version: Option<&str>,
    exposure_id: Option<&str>,
    assisted: Option<bool>,
    now: i64,
) -> LibraryResult<Option<String>> {
    let context = if let Some(id) = exposure_id {
        let context: Option<String> = connection
            .query_row(
                "SELECT recall_kind FROM practice_exposures
            WHERE entity_id = ?1 AND card_id = ?2 AND content_version = ?3
                AND recall_kind IS NOT NULL AND occurred_at <= ?4
                AND NOT EXISTS (SELECT 1 FROM effective_reviews WHERE exposure_id = ?1)",
                params![id, card_id, version, now],
                |row| row.get(0),
            )
            .optional()?;

        Some(context.ok_or(LibraryError::StudyContentChanged)?)
    } else {
        None
    };

    Ok(match assisted {
        Some(true) => Some("helped".to_owned()),
        Some(false) => context,
        None => None,
    })
}

pub(super) fn query_evidence(
    connection: &Connection,
    card_id: &str,
    version: &str,
    now: i64,
) -> LibraryResult<LearningEvidence> {
    let mut evidence = LearningEvidence::default();
    let mut statement = connection.prepare(
        "SELECT evidence_kind, rating FROM effective_reviews
        WHERE card_id = ?1 AND content_version = ?2 AND reviewed_at BETWEEN ?3 AND ?4",
    )?;
    let rows = statement.query_map(
        params![
            card_id,
            version,
            now.saturating_sub(30 * MILLISECONDS_PER_DAY),
            now
        ],
        |row| Ok((row.get::<_, Option<String>>(0)?, row.get::<_, i64>(1)?)),
    )?;

    for row in rows {
        let (kind, rating) = row?;

        match kind.as_deref() {
            Some("delayed") => {
                evidence.delayed_total += 1;
                evidence.delayed_remembered += i64::from(rating > 1);
            }
            Some("recent") => evidence.recent += 1,
            Some("first") => evidence.first += 1,
            Some("helped") => evidence.helped += 1,
            _ => evidence.unknown += 1,
        }
    }

    Ok(evidence)
}

pub fn query_workload(connection: &Connection, now: i64) -> LibraryResult<StudyWorkload> {
    Ok(connection.query_row(
        "SELECT
            COALESCE(SUM(s.review_count > 0 AND s.due_at <= ?1), 0),
            COALESCE(SUM(s.review_count > 0 AND s.due_at <= ?2), 0),
            COALESCE(SUM(s.review_count = 0), 0),
            COALESCE(SUM((SELECT MIN(reviewed_at) FROM effective_reviews r
                WHERE r.card_id = c.entity_id) BETWEEN ?3 AND ?1), 0)
        FROM cards c
        INNER JOIN card_scheduling s ON s.card_id = c.entity_id
        INNER JOIN entities ce ON ce.id = c.entity_id
        INNER JOIN concepts ON concepts.entity_id = c.concept_id
        INNER JOIN entities co ON co.id = c.concept_id
        LEFT JOIN entities te ON te.id = c.template_id
        WHERE ce.deleted_at IS NULL AND co.deleted_at IS NULL
            AND concepts.archived_at IS NULL
            AND (c.template_id IS NULL OR te.deleted_at IS NULL)",
        params![
            now,
            now.saturating_sub(MILLISECONDS_PER_DAY),
            now.saturating_sub(7 * MILLISECONDS_PER_DAY)
        ],
        |row| {
            Ok(StudyWorkload {
                due: row.get(0)?,
                overdue: row.get(1)?,
                new: row.get(2)?,
                introduced: row.get(3)?,
            })
        },
    )?)
}

#[cfg(test)]
mod tests;
