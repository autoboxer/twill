use rusqlite::{Connection, OptionalExtension};
use crate::data::current_timestamp;

use crate::library::{
    LibraryError, LibraryResult, ReviewHistory, ReviewHistoryEntry, ReviewRating,
};

const HISTORY_LIMIT: usize = 10;

pub(super) const CONTENT_VERSION_SQL: &str = "concept_entities.last_change_id || ':' ||
    card_entities.last_change_id || ':' || COALESCE(template_entities.last_change_id, '')";

pub(super) fn query_content_version(
    connection: &Connection,
    card_id: &str,
) -> LibraryResult<String> {
    connection
        .query_row(
            &format!(
                "SELECT {CONTENT_VERSION_SQL}
            FROM cards
            INNER JOIN entities AS card_entities ON card_entities.id = cards.entity_id
            INNER JOIN entities AS concept_entities ON concept_entities.id = cards.concept_id
            LEFT JOIN entities AS template_entities ON template_entities.id = cards.template_id
            WHERE cards.entity_id = ?1
                AND card_entities.deleted_at IS NULL
                AND concept_entities.deleted_at IS NULL
                AND (cards.template_id IS NULL OR template_entities.deleted_at IS NULL)"
            ),
            [card_id],
            |row| row.get(0),
        )
        .optional()?
        .ok_or_else(|| LibraryError::CardNotFound(card_id.to_owned()))
}

pub fn query_review_history(
    connection: &Connection,
    card_id: &str,
) -> LibraryResult<ReviewHistory> {
    let current_version = query_content_version(connection, card_id)?;
    let mut statement = connection.prepare(
        "SELECT reviews.entity_id, reviews.rating, reviews.assisted, reviews.reviewed_at,
            reviews.content_version, review_reversals.reversed_at, reviews.evidence_kind
        FROM reviews
        LEFT JOIN review_reversals ON review_reversals.review_id = reviews.entity_id
        WHERE reviews.card_id = ?1
        ORDER BY reviews.reviewed_at DESC, reviews.entity_id DESC
        LIMIT ?2",
    )?;
    let rows = statement.query_map((card_id, HISTORY_LIMIT as i64 + 1), |row| {
        let version: Option<String> = row.get(4)?;
        let rating = match row.get::<_, i64>(1)? {
            1 => ReviewRating::Again,
            2 => ReviewRating::Hard,
            3 => ReviewRating::Good,
            4 => ReviewRating::Easy,
            _ => return Err(rusqlite::Error::InvalidQuery),
        };

        Ok(ReviewHistoryEntry {
            id: row.get(0)?,
            rating,
            assisted: row.get(2)?,
            reviewed_at: row.get(3)?,
            current_content: version.map(|version| version == current_version),
            reversed_at: row.get(5)?,
            evidence_kind: row.get(6)?,
        })
    })?;
    let mut reviews = rows.collect::<Result<Vec<_>, _>>()?;
    let has_more = reviews.len() > HISTORY_LIMIT;

    reviews.truncate(HISTORY_LIMIT);

    let evidence = super::evidence::query_evidence(
        connection, card_id, &current_version, current_timestamp()?,
    )?;

    Ok(ReviewHistory {
        reviews,
        has_more,
        evidence,
    })
}

#[cfg(test)]
mod tests;
