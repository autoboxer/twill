use rusqlite::{params, Connection, OptionalExtension};

use crate::data::{EntityKind, LocalDataStore, WriteTransaction};
use crate::library::{
    CreatePracticeLinkInput, LibraryError, LibraryResult, LinkedPracticeCase, LinkedPracticeInput,
    PracticeLink, RemovePracticeLinkInput,
    UpdatePracticeLinkInput,
};

const MAXIMUM_LINKS_PER_CONCEPT: usize = 32;

pub struct PracticeLinkLibrary<'store> {
    store: &'store LocalDataStore,
}

impl<'store> PracticeLinkLibrary<'store> {
    pub fn new(store: &'store LocalDataStore) -> Self {
        Self { store }
    }

    pub fn links(&self, concept_id: &str) -> LibraryResult<Vec<PracticeLink>> {
        self.store.read_result(|connection| {
            concept_archived(connection, concept_id)?;
            query_links(connection, concept_id)
        })
    }

    pub fn practice(&self, input: LinkedPracticeInput) -> LibraryResult<LinkedPracticeCase> {
        self.store.read_result(|connection| {
            if concept_archived(connection, &input.concept_id)? {
                return Err(invalid("Restore the current concept before linked practice."));
            }

            let link = query_links(connection, &input.concept_id)?
                .into_iter()
                .find(|link| link.id == input.link_id)
                .ok_or_else(|| LibraryError::PracticeLinkNotFound(input.link_id.clone()))?;

            if link.archived {
                return Err(invalid("Restore this archived case before practicing it."));
            }

            let source_last_change_id = connection.query_row(
                "SELECT last_change_id FROM entities WHERE id = ?1",
                [&input.concept_id],
                |row| row.get(0),
            )?;
            let concept = crate::library::service::query_concept(connection, &link.concept_id)?;
            let template_ids = concept
                .cards
                .iter()
                .filter_map(|card| card.template.as_ref().map(|template| template.id.clone()))
                .collect();
            let templates = crate::library::study::query_templates(connection, &template_ids)?;

            Ok(LinkedPracticeCase {
                link,
                source_last_change_id,
                concept,
                templates,
            })
        })
    }

    pub fn create(&self, input: CreatePracticeLinkInput) -> LibraryResult<()> {
        let objective = normalize_objective(input.objective)?;
        let mut ids = [input.concept_id.trim(), input.related_concept_id.trim()];

        if ids[0] == ids[1] {
            return Err(invalid("Choose a different concept to link."));
        }

        ids.sort_unstable();

        self.store.write_result(|transaction| {
            for id in ids {
                if concept_archived(transaction, id)? {
                    return Err(invalid("Restore archived concepts before adding a link."));
                }
            }

            let exists: bool = transaction.query_row(
                "SELECT EXISTS (
                    SELECT 1 FROM practice_links
                    INNER JOIN entities ON entities.id = practice_links.entity_id
                    WHERE first_concept_id = ?1 AND second_concept_id = ?2
                        AND entities.deleted_at IS NULL
                )",
                ids,
                |row| row.get(0),
            )?;

            if exists {
                return Err(LibraryError::PracticeLinkExists);
            }

            for id in ids {
                if query_links(transaction, id)?.len() >= MAXIMUM_LINKS_PER_CONCEPT {
                    return Err(invalid("A concept can have up to 32 practice links."));
                }
            }

            let entity = transaction.create_entity(EntityKind::PracticeLink)?;

            transaction.execute(
                "INSERT INTO practice_links (
                    entity_id, first_concept_id, second_concept_id, objective, last_change_id
                ) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![entity.id, ids[0], ids[1], objective, entity.last_change_id],
            )?;

            Ok(())
        })
    }

    pub fn update(&self, input: UpdatePracticeLinkInput) -> LibraryResult<()> {
        let objective = normalize_objective(input.objective)?;

        self.store.write_result(|transaction| {
            let current = link_objective(transaction, &input.id, &input.expected_change_id)?;

            if current == objective {
                return Ok(());
            }

            let entity = transaction.touch_entity(&input.id)?;

            transaction.execute(
                "UPDATE practice_links SET objective = ?1, last_change_id = ?2
                WHERE entity_id = ?3",
                params![objective, entity.last_change_id, input.id],
            )?;

            Ok(())
        })
    }

    pub fn remove(&self, input: RemovePracticeLinkInput) -> LibraryResult<()> {
        self.store.write_result(|transaction| {
            link_objective(transaction, &input.id, &input.expected_change_id)?;
            transaction.soft_delete_entity(&input.id)?;

            Ok(())
        })
    }
}

fn query_links(connection: &Connection, concept_id: &str) -> LibraryResult<Vec<PracticeLink>> {
    let mut statement = connection.prepare(
        "SELECT links.entity_id, concepts.entity_id, concepts.title,
            concepts.archived_at IS NOT NULL, links.objective, entities.last_change_id
        FROM practice_links AS links
        INNER JOIN entities ON entities.id = links.entity_id
        INNER JOIN concepts ON concepts.entity_id = CASE
            WHEN links.first_concept_id = ?1 THEN links.second_concept_id
            ELSE links.first_concept_id END
        INNER JOIN entities AS targets ON targets.id = concepts.entity_id
        WHERE (links.first_concept_id = ?1 OR links.second_concept_id = ?1)
            AND entities.deleted_at IS NULL AND targets.deleted_at IS NULL
        ORDER BY concepts.title COLLATE NOCASE, links.entity_id",
    )?;
    let links = statement.query_map([concept_id], |row| {
        Ok(PracticeLink {
            id: row.get(0)?,
            concept_id: row.get(1)?,
            title: row.get(2)?,
            archived: row.get(3)?,
            objective: row.get(4)?,
            last_change_id: row.get(5)?,
        })
    })?;

    Ok(links.collect::<Result<_, _>>()?)
}

fn concept_archived(connection: &Connection, id: &str) -> LibraryResult<bool> {
    connection
        .query_row(
            "SELECT archived_at IS NOT NULL FROM concepts
            INNER JOIN entities ON entities.id = concepts.entity_id
            WHERE concepts.entity_id = ?1 AND entities.kind = 'concept'
                AND entities.deleted_at IS NULL",
            [id],
            |row| row.get(0),
        )
        .optional()?
        .ok_or_else(|| LibraryError::ConceptNotFound(id.to_owned()))
}

fn link_objective(
    connection: &Connection,
    id: &str,
    expected_change_id: &str,
) -> LibraryResult<String> {
    let current = connection
        .query_row(
            "SELECT objective, entities.last_change_id FROM practice_links
            INNER JOIN entities ON entities.id = practice_links.entity_id
            WHERE entity_id = ?1 AND entities.deleted_at IS NULL",
            [id],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
        )
        .optional()?;
    let Some((objective, change_id)) = current else {
        return Err(LibraryError::PracticeLinkNotFound(id.to_owned()));
    };

    if change_id != expected_change_id {
        return Err(LibraryError::PracticeLinkChanged);
    }

    Ok(objective)
}

fn normalize_objective(objective: String) -> LibraryResult<String> {
    let objective = objective.trim().to_owned();

    if objective.is_empty() || objective.chars().count() > 300 {
        return Err(invalid(
            "Describe the learning objective in 1 to 300 characters.",
        ));
    }

    Ok(objective)
}

fn invalid(message: &'static str) -> LibraryError {
    LibraryError::InvalidPracticeLink { message }
}

pub(super) fn remove_concept_links(
    transaction: &WriteTransaction<'_>,
    concept_id: &str,
) -> LibraryResult<()> {
    for link in query_links(transaction, concept_id)? {
        transaction.soft_delete_entity(&link.id)?;
    }

    Ok(())
}

pub(super) fn validate_restored_links(connection: &Connection) -> LibraryResult<()> {
    let mut statement = connection.prepare(
        "SELECT links.first_concept_id, links.second_concept_id, links.objective,
            entities.kind, entities.last_change_id = links.last_change_id
        FROM practice_links AS links
        INNER JOIN entities ON entities.id = links.entity_id
        WHERE entities.deleted_at IS NULL",
    )?;
    let mut rows = statement.query([])?;
    let mut pairs = std::collections::HashSet::new();
    let mut counts = std::collections::HashMap::new();

    while let Some(row) = rows.next()? {
        let first: String = row.get(0)?;
        let second: String = row.get(1)?;
        let objective: String = row.get(2)?;
        let kind: String = row.get(3)?;
        let matching_change: bool = row.get(4)?;

        if kind != "practice_link"
            || !matching_change
            || first >= second
            || normalize_objective(objective.clone())? != objective
            || !pairs.insert((first.clone(), second.clone()))
        {
            return Err(invalid("The backup contains an invalid practice link."));
        }

        concept_archived(connection, &first)?;
        concept_archived(connection, &second)?;

        for id in [first, second] {
            let count = counts.entry(id).or_insert(0);

            *count += 1;

            if *count > MAXIMUM_LINKS_PER_CONCEPT {
                return Err(invalid("The backup contains too many links for a concept."));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests;
