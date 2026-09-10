use std::collections::HashSet;

use rusqlite::{params, Connection};

use super::{normalize_value, ConceptLibrary};
use crate::data::EntityKind;
use crate::library::{LibraryError, LibraryOrganizations, LibraryResult, OrganizationSummary};

const MAXIMUM_ORGANIZATION_NAME_LENGTH: usize = 80;

#[derive(Clone, Copy)]
pub(super) enum OrganizationKind {
    Deck,
    Tag,
}

impl OrganizationKind {
    pub(super) const fn display_name(self) -> &'static str {
        match self {
            Self::Deck => "Deck name",
            Self::Tag => "Tag name",
        }
    }

    pub(super) const fn entity_kind(self) -> EntityKind {
        match self {
            Self::Deck => EntityKind::Deck,
            Self::Tag => EntityKind::Tag,
        }
    }

    pub(super) const fn noun(self) -> &'static str {
        match self {
            Self::Deck => "deck",
            Self::Tag => "tag",
        }
    }

    pub(super) const fn table(self) -> &'static str {
        match self {
            Self::Deck => "decks",
            Self::Tag => "tags",
        }
    }

    pub(super) const fn membership_table(self) -> &'static str {
        match self {
            Self::Deck => "concept_decks",
            Self::Tag => "concept_tags",
        }
    }

    pub(super) const fn membership_column(self) -> &'static str {
        match self {
            Self::Deck => "deck_id",
            Self::Tag => "tag_id",
        }
    }
}

impl ConceptLibrary<'_> {
    pub fn organizations(&self) -> LibraryResult<LibraryOrganizations> {
        self.store.read_result(|connection| {
            Ok(LibraryOrganizations {
                decks: query_organizations(connection, OrganizationKind::Deck)?,
                tags: query_organizations(connection, OrganizationKind::Tag)?,
            })
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
        let name = normalize_value(name, kind.display_name(), MAXIMUM_ORGANIZATION_NAME_LENGTH)?;

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
        let name = normalize_value(name, kind.display_name(), MAXIMUM_ORGANIZATION_NAME_LENGTH)?;

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

pub(super) fn normalize_ids(
    ids: Vec<String>,
    kind: &'static str,
) -> LibraryResult<HashSet<String>> {
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

pub(super) fn query_organizations(
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
            ),
            COUNT(
                CASE
                    WHEN memberships.concept_id IS NOT NULL
                        AND memberships.removed_at IS NULL
                        AND concept_entities.deleted_at IS NULL
                        AND concepts.archived_at IS NULL
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
        LEFT JOIN concepts ON concepts.entity_id = memberships.concept_id
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
            active_concept_count: row.get(3)?,
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

pub(super) fn validate_selections(
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
