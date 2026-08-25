use rusqlite::{params, Connection};
use uuid::Uuid;

use crate::data::{current_timestamp, LocalDataStore};
use crate::library::{
    DeferredConceptEdit, DeferredEditQueue, DeferredEditTargetStatus, LibraryError,
    LibraryResult, QueueDeferredEditInput,
};

pub struct DeferredEditLibrary<'store> {
    store: &'store LocalDataStore,
}

impl<'store> DeferredEditLibrary<'store> {
    pub fn new(store: &'store LocalDataStore) -> Self {
        Self { store }
    }

    pub fn queue(&self) -> LibraryResult<DeferredEditQueue> {
        self.store.read_result(query_queue)
    }

    pub fn queue_concept(
        &self,
        input: QueueDeferredEditInput,
    ) -> LibraryResult<DeferredConceptEdit> {
        let concept_id = normalize_id(input.concept_id, "concept ID")?;
        let base_change_id = normalize_id(input.base_change_id, "base change ID")?;
        let queued_at = current_timestamp()?;

        self.store.write_result(|transaction| {
            validate_target(transaction, &concept_id, &base_change_id)?;

            transaction.execute(
                "INSERT INTO deferred_concept_edits (
                    concept_id,
                    base_change_id,
                    queued_at
                ) VALUES (?1, ?2, ?3)
                ON CONFLICT(concept_id) DO NOTHING",
                params![concept_id, base_change_id, queued_at],
            )?;

            query_item(transaction, &concept_id)?.ok_or_else(|| {
                LibraryError::InvalidDeferredEdit {
                    message: "could not be read after queueing".to_owned(),
                }
            })
        })
    }

    pub fn remove_concept(&self, concept_id: &str) -> LibraryResult<()> {
        let concept_id = normalize_id(concept_id.to_owned(), "concept ID")?;

        self.store.write_result(|transaction| {
            transaction.execute(
                "DELETE FROM deferred_concept_edits WHERE concept_id = ?1",
                [&concept_id],
            )?;

            Ok(())
        })
    }
}

fn validate_target(
    connection: &Connection,
    concept_id: &str,
    base_change_id: &str,
) -> LibraryResult<()> {
    let concept_exists: bool = connection.query_row(
        "SELECT EXISTS (
            SELECT 1 FROM concepts WHERE entity_id = ?1
        )",
        [concept_id],
        |row| row.get(0),
    )?;

    if !concept_exists {
        return Err(LibraryError::ConceptNotFound(concept_id.to_owned()));
    }

    let change_belongs_to_concept: bool = connection.query_row(
        "SELECT EXISTS (
            SELECT 1
            FROM change_log
            WHERE id = ?1
                AND entity_id = ?2
        )",
        params![base_change_id, concept_id],
        |row| row.get(0),
    )?;

    if !change_belongs_to_concept {
        return Err(LibraryError::InvalidDeferredEdit {
            message: "base change does not belong to its concept".to_owned(),
        });
    }

    Ok(())
}

fn query_queue(connection: &Connection) -> LibraryResult<DeferredEditQueue> {
    let mut statement = connection.prepare(
        "SELECT
            deferred_concept_edits.concept_id,
            concepts.title,
            deferred_concept_edits.base_change_id,
            deferred_concept_edits.queued_at,
            CASE
                WHEN entities.deleted_at IS NOT NULL THEN 'missing'
                WHEN concepts.archived_at IS NOT NULL THEN 'archived'
                WHEN concepts.last_change_id != deferred_concept_edits.base_change_id
                    THEN 'changed'
                ELSE 'current'
            END
        FROM deferred_concept_edits
        INNER JOIN concepts
            ON concepts.entity_id = deferred_concept_edits.concept_id
        INNER JOIN entities
            ON entities.id = concepts.entity_id
        ORDER BY deferred_concept_edits.position",
    )?;
    let items = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, String>(4)?,
            ))
        })?
        .map(|row| {
            let (concept_id, concept_title, base_change_id, queued_at, target_status) = row?;

            Ok(DeferredConceptEdit {
                concept_id,
                concept_title,
                base_change_id,
                queued_at,
                target_status: parse_target_status(&target_status)?,
            })
        })
        .collect::<LibraryResult<Vec<_>>>()?;

    Ok(DeferredEditQueue { items })
}

fn query_item(
    connection: &Connection,
    concept_id: &str,
) -> LibraryResult<Option<DeferredConceptEdit>> {
    Ok(query_queue(connection)?
        .items
        .into_iter()
        .find(|item| item.concept_id == concept_id))
}

fn parse_target_status(value: &str) -> LibraryResult<DeferredEditTargetStatus> {
    match value {
        "current" => Ok(DeferredEditTargetStatus::Current),
        "changed" => Ok(DeferredEditTargetStatus::Changed),
        "archived" => Ok(DeferredEditTargetStatus::Archived),
        "missing" => Ok(DeferredEditTargetStatus::Missing),
        _ => Err(LibraryError::InvalidDeferredEdit {
            message: "target status is not valid".to_owned(),
        }),
    }
}

fn normalize_id(value: String, field: &'static str) -> LibraryResult<String> {
    let value = value.trim().to_owned();

    if Uuid::parse_str(&value).is_err() {
        return Err(LibraryError::InvalidDeferredEdit {
            message: format!("{field} is not valid"),
        });
    }

    Ok(value)
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::DeferredEditLibrary;
    use crate::data::LocalDataStore;
    use crate::library::{
        ConceptContent, ConceptLibrary, CreateConceptInput, DeferredEditTargetStatus,
        QueueDeferredEditInput, UpdateConceptInput,
    };

    fn create_concept(library: &ConceptLibrary<'_>, title: &str) -> crate::library::ConceptDetail {
        library
            .create_concept(CreateConceptInput {
                title: title.to_owned(),
                deck_ids: Vec::new(),
                tag_ids: Vec::new(),
                content: ConceptContent::default(),
                include_standard_recall: true,
                template_ids: Vec::new(),
                problem: None,
                explain: None,
                type_answer: None,
            })
            .unwrap()
    }

    fn queue_input(concept: &crate::library::ConceptDetail) -> QueueDeferredEditInput {
        QueueDeferredEditInput {
            concept_id: concept.id.clone(),
            base_change_id: concept.last_change_id.clone(),
        }
    }

    #[test]
    fn deferred_edits_are_device_local_ordered_and_durable() {
        let directory = tempdir().unwrap();

        {
            let store = LocalDataStore::open(directory.path()).unwrap();
            let concepts = ConceptLibrary::new(&store);
            let edits = DeferredEditLibrary::new(&store);
            let first = create_concept(&concepts, "First");
            let second = create_concept(&concepts, "Second");
            let changes_before = store.changes_after(0, 100).unwrap();

            edits.queue_concept(queue_input(&first)).unwrap();
            edits.queue_concept(queue_input(&first)).unwrap();
            edits.queue_concept(queue_input(&second)).unwrap();

            let queue = edits.queue().unwrap();

            assert_eq!(queue.items.len(), 2);
            assert_eq!(queue.items[0].concept_id, first.id);
            assert_eq!(queue.items[1].concept_id, second.id);
            assert_eq!(store.changes_after(0, 100).unwrap(), changes_before);
        }

        let store = LocalDataStore::open(directory.path()).unwrap();
        let queue = DeferredEditLibrary::new(&store).queue().unwrap();

        assert_eq!(queue.items.len(), 2);
        assert_eq!(queue.items[0].concept_title, "First");
        assert_eq!(queue.items[1].concept_title, "Second");
    }

    #[test]
    fn deferred_edits_report_target_changes_and_can_be_removed() {
        let directory = tempdir().unwrap();
        let store = LocalDataStore::open(directory.path()).unwrap();
        let concepts = ConceptLibrary::new(&store);
        let edits = DeferredEditLibrary::new(&store);
        let current = create_concept(&concepts, "Current");
        let changed = create_concept(&concepts, "Changed");
        let archived = create_concept(&concepts, "Archived");
        let missing = create_concept(&concepts, "Missing");

        for concept in [&current, &changed, &archived, &missing] {
            edits.queue_concept(queue_input(concept)).unwrap();
        }

        concepts
            .update_concept(UpdateConceptInput {
                id: changed.id.clone(),
                title: "Changed after queueing".to_owned(),
                deck_ids: Vec::new(),
                tag_ids: Vec::new(),
                content: changed.content.clone(),
                include_standard_recall: true,
                template_ids: Vec::new(),
                problem: None,
                explain: None,
                type_answer: None,
            })
            .unwrap();
        concepts.set_concept_archived(&archived.id, true).unwrap();
        concepts.delete_concept(&missing.id).unwrap();

        let queue = edits.queue().unwrap();
        let statuses = queue
            .items
            .iter()
            .map(|item| item.target_status)
            .collect::<Vec<_>>();

        assert_eq!(
            statuses,
            vec![
                DeferredEditTargetStatus::Current,
                DeferredEditTargetStatus::Changed,
                DeferredEditTargetStatus::Archived,
                DeferredEditTargetStatus::Missing,
            ]
        );

        edits.remove_concept(&changed.id).unwrap();
        edits.remove_concept(&changed.id).unwrap();

        assert_eq!(edits.queue().unwrap().items.len(), 3);
    }
}
