use rusqlite::{params, Connection};

use crate::data::{EntityKind, WriteTransaction};
use crate::library::{CardSummary, LibraryResult, StudyCard};

pub fn create_recall_card(
    transaction: &WriteTransaction<'_>,
    concept_id: &str,
) -> LibraryResult<CardSummary> {
    let entity = transaction.create_entity(EntityKind::Card)?;

    transaction.execute(
        "INSERT INTO cards (entity_id, concept_id, last_change_id)
        VALUES (?1, ?2, ?3)",
        params![entity.id, concept_id, entity.last_change_id],
    )?;

    Ok(CardSummary { id: entity.id })
}

pub fn query_study_cards(connection: &Connection) -> LibraryResult<Vec<StudyCard>> {
    let mut statement = connection.prepare(
        "SELECT
            cards.entity_id,
            concepts.entity_id,
            concepts.title,
            concepts.content_json
        FROM cards
        INNER JOIN entities AS card_entities
            ON card_entities.id = cards.entity_id
        INNER JOIN concepts
            ON concepts.entity_id = cards.concept_id
        INNER JOIN entities AS concept_entities
            ON concept_entities.id = concepts.entity_id
        WHERE card_entities.deleted_at IS NULL
            AND concept_entities.deleted_at IS NULL
            AND concepts.archived_at IS NULL
        ORDER BY card_entities.created_at, cards.entity_id",
    )?;
    let rows = statement.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
        ))
    })?;

    rows.map(|row| {
        let (id, concept_id, concept_title, content) = row?;

        Ok(StudyCard {
            id,
            concept_id,
            concept_title,
            content: serde_json::from_str(&content)?,
        })
    })
    .collect()
}
