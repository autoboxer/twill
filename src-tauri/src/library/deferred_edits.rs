use rusqlite::{params, Connection};
use uuid::Uuid;

use crate::data::{current_timestamp, LocalDataStore};
use crate::library::{
    DeferredConceptEdit, DeferredEditQueue, DeferredEditTargetStatus, LibraryError, LibraryResult,
    QueueDeferredEditInput, UpdateDeferredEditNoteInput,
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

    pub fn update_note(
        &self,
        input: UpdateDeferredEditNoteInput,
    ) -> LibraryResult<DeferredConceptEdit> {
        let concept_id = normalize_id(input.concept_id, "concept ID")?;
        let note = input.note.trim();

        if note.chars().count() > 500 {
            return Err(LibraryError::InvalidDeferredEdit {
                message: "note must be 500 characters or fewer".to_owned(),
            });
        }

        self.store.write_result(|transaction| {
            let updated = transaction.execute(
                "UPDATE deferred_concept_edits SET note = ?1
                WHERE concept_id = ?2 AND position = ?3 AND note = ?4",
                params![note, concept_id, input.position, input.expected_note],
            )?;

            if updated != 1 {
                return Err(LibraryError::DeferredEditChanged);
            }

            query_item(transaction, &concept_id)?.ok_or(LibraryError::DeferredEditChanged)
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
            END,
            deferred_concept_edits.position,
            deferred_concept_edits.note
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
                row.get::<_, i64>(5)?,
                row.get::<_, String>(6)?,
            ))
        })?
        .map(|row| {
            let (
                concept_id,
                concept_title,
                base_change_id,
                queued_at,
                target_status,
                position,
                note,
            ) = row?;

            Ok(DeferredConceptEdit {
                position,
                concept_id,
                concept_title,
                base_change_id,
                queued_at,
                note,
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
        ConceptContent, ConceptLibrary, CreateConceptInput, DeferredEditTargetStatus, LibraryError,
        QueueDeferredEditInput, UpdateConceptInput, UpdateDeferredEditNoteInput,
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

            let queued = edits.queue_concept(queue_input(&first)).unwrap();
            edits
                .update_note(UpdateDeferredEditNoteInput {
                    concept_id: first.id.clone(),
                    position: queued.position,
                    expected_note: String::new(),
                    note: "  Add a TCP/IP example\nClarify café 中 🧵  ".to_owned(),
                })
                .unwrap();
            edits.queue_concept(queue_input(&first)).unwrap();
            edits.queue_concept(queue_input(&second)).unwrap();

            let queue = edits.queue().unwrap();

            assert_eq!(queue.items.len(), 2);
            assert_eq!(queue.items[0].concept_id, first.id);
            assert_eq!(queue.items[1].concept_id, second.id);
            assert_eq!(queue.items[0].position, queued.position);
            assert_eq!(queue.items[0].queued_at, queued.queued_at);
            assert_eq!(queue.items[0].base_change_id, queued.base_change_id);
            assert_eq!(
                queue.items[0].note,
                "Add a TCP/IP example\nClarify café 中 🧵"
            );
            assert_eq!(store.changes_after(0, 100).unwrap(), changes_before);
        }

        let store = LocalDataStore::open(directory.path()).unwrap();
        let queue = DeferredEditLibrary::new(&store).queue().unwrap();

        assert_eq!(queue.items.len(), 2);
        assert_eq!(queue.items[0].concept_title, "First");
        assert_eq!(queue.items[1].concept_title, "Second");
        assert_eq!(
            queue.items[0].note,
            "Add a TCP/IP example\nClarify café 中 🧵"
        );
    }

    #[test]
    fn deferred_notes_are_bounded_can_be_cleared_and_do_not_change_queue_identity() {
        let directory = tempdir().unwrap();
        let store = LocalDataStore::open(directory.path()).unwrap();
        let concept = create_concept(&ConceptLibrary::new(&store), "Notes");
        let edits = DeferredEditLibrary::new(&store);
        let queued = edits.queue_concept(queue_input(&concept)).unwrap();
        let mut input = UpdateDeferredEditNoteInput {
            concept_id: concept.id.clone(),
            position: queued.position,
            expected_note: String::new(),
            note: "🧵".repeat(500),
        };
        let updated = edits.update_note(input.clone()).unwrap();

        assert_eq!(updated.note.chars().count(), 500);
        assert_eq!(updated.position, queued.position);
        input.expected_note = updated.note.clone();
        input.note.push('x');
        assert!(matches!(
            edits.update_note(input.clone()),
            Err(LibraryError::InvalidDeferredEdit { .. })
        ));
        assert_eq!(edits.queue().unwrap().items[0], updated);
        input.note = " \n\t ".to_owned();
        assert!(edits.update_note(input).unwrap().note.is_empty());
    }

    #[test]
    fn stale_notes_and_replaced_or_removed_queue_items_cannot_be_overwritten() {
        let directory = tempdir().unwrap();
        let store = LocalDataStore::open(directory.path()).unwrap();
        let concept = create_concept(&ConceptLibrary::new(&store), "Notes");
        let edits = DeferredEditLibrary::new(&store);
        let queued = edits.queue_concept(queue_input(&concept)).unwrap();
        let input = UpdateDeferredEditNoteInput {
            concept_id: concept.id.clone(),
            position: queued.position,
            expected_note: String::new(),
            note: "First reason".to_owned(),
        };
        let updated = edits.update_note(input.clone()).unwrap();

        assert!(matches!(
            edits.update_note(input.clone()),
            Err(LibraryError::DeferredEditChanged)
        ));
        assert_eq!(edits.queue().unwrap().items[0], updated);
        edits.remove_concept(&concept.id).unwrap();
        assert!(matches!(
            edits.update_note(input.clone()),
            Err(LibraryError::DeferredEditChanged)
        ));
        let replacement = edits.queue_concept(queue_input(&concept)).unwrap();
        assert_ne!(replacement.position, queued.position);
        assert!(matches!(
            edits.update_note(input),
            Err(LibraryError::DeferredEditChanged)
        ));
        assert_eq!(edits.queue().unwrap().items[0], replacement);
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
