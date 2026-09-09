use rusqlite::params;

use crate::data::LocalDataStore;
use crate::library::authoring_drafts::{delete_draft, validate_save_context};
use crate::library::{
    service, templates, AuthoringDraftKind, AuthoringDraftLocator, ConceptDetail,
    FinalizeConceptInput, FinalizeTemplateInput, LibraryError, LibraryResult, TemplateDetail,
    UpdateConceptInput, UpdateTemplateInput,
};

pub(super) fn finalize_concept(
    store: &LocalDataStore,
    input: FinalizeConceptInput,
) -> LibraryResult<ConceptDetail> {
    store.write_result(|transaction| {
        let context = input.context;
        validate_save_context(transaction, AuthoringDraftKind::Concept, &context)?;

        let saved = match context.target_id.as_ref().filter(|_| !context.save_as_copy) {
            Some(id) => service::update_concept(
                transaction,
                UpdateConceptInput {
                    id: id.clone(),
                    title: input.concept.title,
                    content: input.concept.content,
                    deck_ids: input.concept.deck_ids,
                    tag_ids: input.concept.tag_ids,
                    include_standard_recall: input.concept.include_standard_recall,
                    template_ids: input.concept.template_ids,
                    explain: input.concept.explain,
                    problem: input.concept.problem,
                    type_answer: input.concept.type_answer,
                },
            )?,
            None => service::create_concept(transaction, input.concept)?,
        };

        if let Some(position) = input.deferred_edit_position {
            let target_id = context
                .target_id
                .as_deref()
                .ok_or(LibraryError::DeferredEditChanged)?;
            let removed = transaction.execute(
                "DELETE FROM deferred_concept_edits WHERE concept_id = ?1 AND position = ?2",
                params![target_id.trim(), position],
            )?;

            if removed != 1 {
                return Err(LibraryError::DeferredEditChanged);
            }
        }

        delete_draft(
            transaction,
            AuthoringDraftLocator {
                kind: AuthoringDraftKind::Concept,
                target_id: context.target_id,
            },
        )?;

        Ok(saved)
    })
}

pub(super) fn finalize_template(
    store: &LocalDataStore,
    input: FinalizeTemplateInput,
) -> LibraryResult<TemplateDetail> {
    store.write_result(|transaction| {
        let context = input.context;
        validate_save_context(transaction, AuthoringDraftKind::Template, &context)?;

        let saved = match context.target_id.as_ref().filter(|_| !context.save_as_copy) {
            Some(id) => templates::update_template(
                transaction,
                UpdateTemplateInput {
                    id: id.clone(),
                    name: input.template.name,
                    content: input.template.content,
                },
            )?,
            None => templates::create_template(transaction, input.template)?,
        };

        delete_draft(
            transaction,
            AuthoringDraftLocator {
                kind: AuthoringDraftKind::Template,
                target_id: context.target_id,
            },
        )?;

        Ok(saved)
    })
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use image::{DynamicImage, ImageFormat};
    use serde_json::json;
    use tempfile::tempdir;

    use super::{finalize_concept, finalize_template};
    use crate::data::LocalDataStore;
    use crate::library::{
        AuthoringDraft, AuthoringDraftKind, AuthoringDraftLibrary, AuthoringDraftLocator,
        AuthoringSaveContext, ConceptLibrary, CreateConceptInput, CreateTemplateInput,
        DeferredEditLibrary, FinalizeConceptInput, FinalizeTemplateInput, LibraryError,
        QueueDeferredEditInput, TemplateContent, TemplateLibrary, UpdateTemplateInput,
        UpsertAuthoringDraftInput,
    };

    fn concept_input(title: &str) -> CreateConceptInput {
        serde_json::from_value(json!({ "title": title })).unwrap()
    }

    fn save_draft(
        store: &LocalDataStore,
        kind: AuthoringDraftKind,
        target: Option<(&str, &str)>,
    ) -> AuthoringDraft {
        AuthoringDraftLibrary::new(store)
            .upsert_draft(UpsertAuthoringDraftInput {
                kind,
                target_id: target.map(|(id, _)| id.to_owned()),
                base_change_id: target.map(|(_, change)| change.to_owned()),
                schema_version: 1,
                payload: json!({ "title": "Unfinished work" }),
                media_ids: vec![],
                media_session_id: None,
            })
            .unwrap()
    }

    fn context(draft: &AuthoringDraft) -> AuthoringSaveContext {
        AuthoringSaveContext {
            target_id: draft.target_id.clone(),
            expected_change_id: draft.base_change_id.clone(),
            expected_draft_revision: Some(draft.revision.clone()),
            save_as_copy: false,
        }
    }

    fn concept_save(draft: &AuthoringDraft, title: &str) -> FinalizeConceptInput {
        FinalizeConceptInput {
            context: context(draft),
            concept: concept_input(title),
            deferred_edit_position: None,
        }
    }

    fn template_save(draft: &AuthoringDraft, name: &str) -> FinalizeTemplateInput {
        FinalizeTemplateInput {
            context: context(draft),
            template: CreateTemplateInput {
                name: name.to_owned(),
                content: TemplateContent::default(),
            },
        }
    }

    fn count(store: &LocalDataStore, table: &str) -> i64 {
        store
            .read_result::<_, LibraryError>(|connection| {
                Ok(
                    connection.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                        row.get(0)
                    })?,
                )
            })
            .unwrap()
    }

    fn stored_draft(store: &LocalDataStore, draft: &AuthoringDraft) -> Option<AuthoringDraft> {
        AuthoringDraftLibrary::new(store)
            .draft(AuthoringDraftLocator {
                kind: draft.kind,
                target_id: draft.target_id.clone(),
            })
            .unwrap()
    }

    fn fail_draft_deletion(store: &LocalDataStore) {
        store
            .write(|transaction| {
                transaction.execute_batch(
                    "CREATE TRIGGER fail_finalization BEFORE DELETE ON authoring_drafts
                BEGIN SELECT RAISE(ABORT, 'injected cleanup failure'); END;",
                )?;
                Ok(())
            })
            .unwrap();
    }

    fn allow_draft_deletion(store: &LocalDataStore) {
        store
            .write(|transaction| {
                transaction.execute_batch("DROP TRIGGER fail_finalization")?;
                Ok(())
            })
            .unwrap();
    }

    #[test]
    fn saved_concept_and_draft_consumption_survive_reopen_without_duplicates() {
        let directory = tempdir().unwrap();
        let store = LocalDataStore::open(directory.path()).unwrap();
        let mut bytes = Cursor::new(Vec::new());
        DynamicImage::new_rgba8(4, 3)
            .write_to(&mut bytes, ImageFormat::Png)
            .unwrap();
        let library = ConceptLibrary::new(&store);
        let media = library.import_image(bytes.get_ref()).unwrap();
        let draft = AuthoringDraftLibrary::new(&store)
            .upsert_draft(UpsertAuthoringDraftInput {
                kind: AuthoringDraftKind::Concept,
                target_id: None,
                base_change_id: None,
                schema_version: 1,
                payload: json!({ "title": "Saved image" }),
                media_ids: vec![media.id.clone()],
                media_session_id: None,
            })
            .unwrap();
        let mut input = concept_save(&draft, "Saved image");
        input.concept.content.prompt = json!({ "type": "doc", "content": [
            { "type": "mediaImage", "attrs": { "mediaId": media.id, "alt": "Diagram" } }
        ] });
        let saved = finalize_concept(&store, input.clone()).unwrap();

        assert!(stored_draft(&store, &draft).is_none());
        assert_eq!(saved.media[0].id, media.id);
        assert_eq!(saved.cards.len(), 1);
        assert!(matches!(
            finalize_concept(&store, input),
            Err(LibraryError::AuthoringDraftChanged)
        ));
        drop(store);

        let reopened = LocalDataStore::open(directory.path()).unwrap();
        assert!(stored_draft(&reopened, &draft).is_none());
        assert_eq!(count(&reopened, "concepts"), 1);
        assert_eq!(
            ConceptLibrary::new(&reopened)
                .concept(&saved.id)
                .unwrap()
                .title,
            "Saved image"
        );
        assert_eq!(
            ConceptLibrary::new(&reopened)
                .media_bytes(&media.id)
                .unwrap(),
            bytes.into_inner()
        );
    }

    #[test]
    fn failed_concept_finalization_rolls_back_content_cards_changes_and_draft() {
        let directory = tempdir().unwrap();
        let store = LocalDataStore::open(directory.path()).unwrap();
        let draft = save_draft(&store, AuthoringDraftKind::Concept, None);
        let before_changes = count(&store, "change_log");
        fail_draft_deletion(&store);

        assert!(finalize_concept(&store, concept_save(&draft, "Not committed")).is_err());
        assert_eq!(count(&store, "concepts"), 0);
        assert_eq!(count(&store, "cards"), 0);
        assert_eq!(count(&store, "change_log"), before_changes);
        assert_eq!(stored_draft(&store, &draft).unwrap(), draft);
        drop(store);

        let reopened = LocalDataStore::open(directory.path()).unwrap();
        assert_eq!(stored_draft(&reopened, &draft).unwrap(), draft);
        allow_draft_deletion(&reopened);
        finalize_concept(&reopened, concept_save(&draft, "Committed on retry")).unwrap();
        assert_eq!(count(&reopened, "concepts"), 1);
        assert!(stored_draft(&reopened, &draft).is_none());
    }

    #[test]
    fn draft_revisions_reject_stale_missing_and_recreated_drafts() {
        let directory = tempdir().unwrap();
        let store = LocalDataStore::open(directory.path()).unwrap();
        let first = save_draft(&store, AuthoringDraftKind::Concept, None);
        let second = save_draft(&store, AuthoringDraftKind::Concept, None);
        assert_ne!(first.revision, second.revision);
        assert!(matches!(
            finalize_concept(&store, concept_save(&first, "Stale")),
            Err(LibraryError::AuthoringDraftChanged)
        ));

        let mut missing = concept_save(&second, "Unexpected draft");
        missing.context.expected_draft_revision = None;
        assert!(matches!(
            finalize_concept(&store, missing),
            Err(LibraryError::AuthoringDraftChanged)
        ));

        AuthoringDraftLibrary::new(&store)
            .delete_draft(AuthoringDraftLocator {
                kind: AuthoringDraftKind::Concept,
                target_id: None,
            })
            .unwrap();
        let third = save_draft(&store, AuthoringDraftKind::Concept, None);
        assert_ne!(second.revision, third.revision);
        assert!(matches!(
            finalize_concept(&store, concept_save(&second, "Recreated")),
            Err(LibraryError::AuthoringDraftChanged)
        ));
        assert_eq!(stored_draft(&store, &third).unwrap(), third);
        assert_eq!(count(&store, "concepts"), 0);
    }

    #[test]
    fn changed_and_deleted_targets_require_explicit_copy_without_overwriting() {
        for deleted in [false, true] {
            let directory = tempdir().unwrap();
            let store = LocalDataStore::open(directory.path()).unwrap();
            let library = ConceptLibrary::new(&store);
            let concept = library.create_concept(concept_input("Original")).unwrap();
            let draft = save_draft(
                &store,
                AuthoringDraftKind::Concept,
                Some((&concept.id, &concept.last_change_id)),
            );
            if deleted {
                library.delete_concept(&concept.id).unwrap();
            } else {
                library.set_concept_archived(&concept.id, true).unwrap();
            }
            let mut input = concept_save(&draft, "Explicit copy");
            let error = finalize_concept(&store, input.clone()).unwrap_err();
            assert!(matches!(
                (deleted, error),
                (true, LibraryError::AuthoringTargetMissing)
                    | (false, LibraryError::AuthoringTargetChanged)
            ));
            assert!(stored_draft(&store, &draft).is_some());
            input.context.save_as_copy = true;
            let copy = finalize_concept(&store, input).unwrap();
            assert_ne!(copy.id, concept.id);
            assert!(stored_draft(&store, &draft).is_none());
            if !deleted {
                let original = library.concept(&concept.id).unwrap();
                assert_eq!(original.title, "Original");
                assert!(original.archived);
            }
        }
    }

    #[test]
    fn queued_edit_completion_is_atomic_and_cannot_consume_a_replacement() {
        let directory = tempdir().unwrap();
        let store = LocalDataStore::open(directory.path()).unwrap();
        let library = ConceptLibrary::new(&store);
        let concept = library.create_concept(concept_input("Queued")).unwrap();
        let queue = DeferredEditLibrary::new(&store);
        let queue_input = QueueDeferredEditInput {
            concept_id: concept.id.clone(),
            base_change_id: concept.last_change_id.clone(),
        };
        let first = queue.queue_concept(queue_input.clone()).unwrap();
        let draft = save_draft(
            &store,
            AuthoringDraftKind::Concept,
            Some((&concept.id, &concept.last_change_id)),
        );
        let mut input = concept_save(&draft, "Completed edit");
        input.deferred_edit_position = Some(first.position);
        let before_changes = count(&store, "change_log");
        fail_draft_deletion(&store);
        assert!(finalize_concept(&store, input.clone()).is_err());
        assert_eq!(library.concept(&concept.id).unwrap(), concept);
        assert_eq!(queue.queue().unwrap().items[0].position, first.position);
        assert_eq!(count(&store, "change_log"), before_changes);
        allow_draft_deletion(&store);

        queue.remove_concept(&concept.id).unwrap();
        let replacement = queue.queue_concept(queue_input).unwrap();
        assert_ne!(first.position, replacement.position);
        assert!(matches!(
            finalize_concept(&store, input.clone()),
            Err(LibraryError::DeferredEditChanged)
        ));
        assert_eq!(library.concept(&concept.id).unwrap(), concept);
        assert_eq!(stored_draft(&store, &draft).unwrap(), draft);
        input.deferred_edit_position = Some(replacement.position);
        finalize_concept(&store, input).unwrap();
        assert!(queue.queue().unwrap().items.is_empty());
        assert!(stored_draft(&store, &draft).is_none());
        assert_eq!(
            library.concept(&concept.id).unwrap().title,
            "Completed edit"
        );
    }

    #[test]
    fn template_creation_and_updates_roll_back_and_reopen_without_a_draft() {
        let directory = tempdir().unwrap();
        let store = LocalDataStore::open(directory.path()).unwrap();
        let draft = save_draft(&store, AuthoringDraftKind::Template, None);
        fail_draft_deletion(&store);
        assert!(finalize_template(&store, template_save(&draft, "New layout")).is_err());
        assert_eq!(count(&store, "templates"), 0);
        assert_eq!(stored_draft(&store, &draft).unwrap(), draft);
        allow_draft_deletion(&store);
        let saved = finalize_template(&store, template_save(&draft, "New layout")).unwrap();
        let edit = save_draft(
            &store,
            AuthoringDraftKind::Template,
            Some((&saved.id, &saved.last_change_id)),
        );
        fail_draft_deletion(&store);
        assert!(finalize_template(&store, template_save(&edit, "Edited layout")).is_err());
        assert_eq!(
            TemplateLibrary::new(&store).template(&saved.id).unwrap(),
            saved
        );
        assert_eq!(stored_draft(&store, &edit).unwrap(), edit);
        allow_draft_deletion(&store);
        let updated = finalize_template(&store, template_save(&edit, "Edited layout")).unwrap();
        let unchanged = save_draft(
            &store,
            AuthoringDraftKind::Template,
            Some((&updated.id, &updated.last_change_id)),
        );
        let before_changes = count(&store, "change_log");

        assert_eq!(
            finalize_template(&store, template_save(&unchanged, "Edited layout")).unwrap(),
            updated
        );
        assert!(stored_draft(&store, &unchanged).is_none());
        assert_eq!(count(&store, "change_log"), before_changes);
        drop(store);

        let reopened = LocalDataStore::open(directory.path()).unwrap();
        assert_eq!(
            TemplateLibrary::new(&reopened)
                .template(&saved.id)
                .unwrap()
                .name,
            "Edited layout"
        );
        assert!(stored_draft(&reopened, &edit).is_none());
        assert_eq!(count(&reopened, "templates"), 1);
    }

    #[test]
    fn template_conflict_and_invalid_content_preserve_the_draft() {
        let directory = tempdir().unwrap();
        let store = LocalDataStore::open(directory.path()).unwrap();
        let library = TemplateLibrary::new(&store);
        let original = library
            .create_template(CreateTemplateInput {
                name: "Original".to_owned(),
                content: TemplateContent::default(),
            })
            .unwrap();
        let draft = save_draft(
            &store,
            AuthoringDraftKind::Template,
            Some((&original.id, &original.last_change_id)),
        );
        let newer = library
            .update_template(UpdateTemplateInput {
                id: original.id.clone(),
                name: "External edit".to_owned(),
                content: original.content.clone(),
            })
            .unwrap();
        let mut input = template_save(&draft, "Preserved copy");
        assert!(matches!(
            finalize_template(&store, input.clone()),
            Err(LibraryError::AuthoringTargetChanged)
        ));
        input.context.save_as_copy = true;
        input.template.name.clear();
        assert!(finalize_template(&store, input.clone()).is_err());
        assert!(stored_draft(&store, &draft).is_some());
        input.template.name = "Preserved copy".to_owned();
        finalize_template(&store, input).unwrap();
        assert_eq!(library.template(&original.id).unwrap(), newer);
        assert!(stored_draft(&store, &draft).is_none());
    }

    #[test]
    fn unchanged_existing_items_can_save_without_a_draft_or_extra_changes() {
        let directory = tempdir().unwrap();
        let store = LocalDataStore::open(directory.path()).unwrap();
        let library = ConceptLibrary::new(&store);
        let concept = library.create_concept(concept_input("Unchanged")).unwrap();
        let context = AuthoringSaveContext {
            target_id: Some(concept.id.clone()),
            expected_change_id: Some(concept.last_change_id.clone()),
            expected_draft_revision: None,
            save_as_copy: false,
        };
        let before_changes = count(&store, "change_log");
        let result = finalize_concept(
            &store,
            FinalizeConceptInput {
                context: context.clone(),
                concept: concept_input("Unchanged"),
                deferred_edit_position: None,
            },
        )
        .unwrap();
        assert_eq!(result, concept);
        assert_eq!(count(&store, "change_log"), before_changes);

        save_draft(
            &store,
            AuthoringDraftKind::Concept,
            Some((&concept.id, &concept.last_change_id)),
        );
        assert!(matches!(
            finalize_concept(
                &store,
                FinalizeConceptInput {
                    context,
                    concept: concept_input("Unchanged"),
                    deferred_edit_position: None
                }
            ),
            Err(LibraryError::AuthoringDraftChanged)
        ));
    }
}
