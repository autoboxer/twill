use std::collections::{HashMap, HashSet};

use rusqlite::{params, params_from_iter, Connection};

use super::organizations::OrganizationKind;
use crate::data::{EntityMetadata, WriteTransaction};
use crate::library::{ConceptSummary, LibraryResult, NamedItem};

pub(super) fn attach_assignments(
    connection: &Connection,
    concepts: &mut [ConceptSummary],
    kind: OrganizationKind,
) -> LibraryResult<()> {
    if concepts.is_empty() {
        return Ok(());
    }

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
            AND memberships.concept_id IN ({})
        ORDER BY items.name COLLATE NOCASE, items.entity_id",
        kind.membership_table(),
        kind.table(),
        kind.membership_column(),
        vec!["?"; concepts.len()].join(", ")
    );
    let mut statement = connection.prepare(&sql)?;
    let assignments = statement.query_map(
        params_from_iter(concepts.iter().map(|concept| &concept.id)),
        |row| {
            Ok((
                row.get::<_, String>(0)?,
                NamedItem {
                    id: row.get(1)?,
                    name: row.get(2)?,
                },
            ))
        },
    )?;

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

pub(super) fn query_concept_assignments(
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

pub(super) fn apply_assignments(
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
            params![concept.id, id, concept.updated_at, concept.last_change_id],
        )?;
    }

    for id in current_ids.difference(desired_ids) {
        transaction.execute(
            &remove_sql,
            params![concept.updated_at, concept.last_change_id, concept.id, id],
        )?;
    }

    Ok(())
}

pub(super) fn apply_media_assignments(
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
            params![concept.id, id, concept.updated_at, concept.last_change_id],
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
            params![concept.updated_at, concept.last_change_id, concept.id, id],
        )?;
    }

    Ok(())
}
