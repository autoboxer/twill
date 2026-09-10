use std::io::Cursor;
use std::time::Duration;

use image::{DynamicImage, ImageFormat};
use rusqlite::params;
use serde_json::json;
use tempfile::{tempdir, TempDir};

use super::ConceptLibrary;
use crate::data::{DataResult, EntityKind, LocalDataStore};
use crate::library::models::{AnswerFeedback, ExplainFocus, TemplateMode};
use crate::library::{
    AppearancePreferences, AppearanceTheme, ConceptContent, CreateConceptInput,
    CreateTemplateInput, ExplainSettings, GradingMode, LibraryError, LibraryQuery, MotionPreference,
    PretestOutcome, ProblemSettings, ReadingFont, ReadingTextSize, RecordPretestInput,
    RecordReviewInput, RetrievalFormKind, ReverseReviewInput, ReviewRating, SchedulingState,
    StartupDestination, TemplateContent, TemplateLibrary, TypeAnswerSettings, UpdateConceptInput,
    UpdateSchedulingSettingsInput, UpdateTemplateInput,
};

fn test_store() -> (TempDir, LocalDataStore) {
    let directory = tempdir().unwrap();
    let store = LocalDataStore::open(directory.path()).unwrap();

    (directory, store)
}

fn png_bytes() -> Vec<u8> {
    let mut bytes = Cursor::new(Vec::new());

    DynamicImage::new_rgba8(4, 3)
        .write_to(&mut bytes, ImageFormat::Png)
        .unwrap();

    bytes.into_inner()
}

#[test]
fn organization_catalog_retains_counts_and_order_without_concept_payloads() {
    let (_directory, store) = test_store();
    let library = ConceptLibrary::new(&store);

    assert_eq!(
        serde_json::to_value(library.organizations().unwrap()).unwrap(),
        json!({ "decks": [], "tags": [] })
    );

    let deck = library.create_deck("Zoology".to_owned()).unwrap();
    let empty_deck = library.create_deck("Biology".to_owned()).unwrap();
    let tag = library.create_tag("À réviser".to_owned()).unwrap();
    let removed_deck = library.create_deck("Removed deck".to_owned()).unwrap();
    let removed_tag = library.create_tag("Removed tag".to_owned()).unwrap();
    let create = |title: &str| {
        library
            .create_concept(CreateConceptInput {
                title: title.to_owned(),
                deck_ids: vec![deck.id.clone()],
                tag_ids: vec![tag.id.clone()],
                content: Default::default(),
                include_standard_recall: true,
                template_ids: Vec::new(),
                explain: None,
                problem: None,
                type_answer: None,
            })
            .unwrap()
    };

    let active = create("Active concept");
    let archived = create("Archived concept");
    let deleted = create("Deleted concept");

    library.set_concept_archived(&archived.id, true).unwrap();
    library.delete_concept(&deleted.id).unwrap();
    library.delete_deck(&removed_deck.id).unwrap();
    library.delete_tag(&removed_tag.id).unwrap();

    let catalog = library.organizations().unwrap();

    assert_eq!(catalog.decks[1].active_concept_count, 1);
    assert_eq!(catalog.tags[0].active_concept_count, 1);
    assert_eq!(catalog.decks.len(), 2);
    assert_eq!(catalog.decks[0].id, empty_deck.id);
    assert_eq!(catalog.decks[0].concept_count, 0);
    assert_eq!(catalog.decks[1].id, deck.id);
    assert_eq!(catalog.decks[1].concept_count, 2);
    assert_eq!(catalog.tags.len(), 1);
    assert_eq!(catalog.tags[0].name, "À réviser");
    assert_eq!(catalog.tags[0].concept_count, 2);

    let serialized = serde_json::to_value(catalog).unwrap();

    assert_eq!(serialized.as_object().unwrap().len(), 2);
    assert!(serialized.get("concepts").is_none());
    assert!(!serialized.to_string().contains(&active.id));

    library.delete_concept(&active.id).unwrap();
    library.delete_concept(&archived.id).unwrap();

    let empty_catalog = library.organizations().unwrap();

    assert_eq!(empty_catalog.decks.len(), 2);
    assert!(empty_catalog.decks.iter().all(|item| item.concept_count == 0));
    assert_eq!(empty_catalog.tags[0].concept_count, 0);
}

#[test]
fn concepts_can_be_created_organized_updated_archived_and_deleted() {
    let (_directory, store) = test_store();
    let library = ConceptLibrary::new(&store);
    let first_deck = library.create_deck("Biology".to_owned()).unwrap();
    let second_deck = library.create_deck("Chemistry".to_owned()).unwrap();
    let unused_deck = library.create_deck("Physics".to_owned()).unwrap();
    let first_tag = library.create_tag("Exam one".to_owned()).unwrap();
    let second_tag = library.create_tag("Needs diagrams".to_owned()).unwrap();

    let created = library
        .create_concept(CreateConceptInput {
            title: "  Cell membrane  ".to_owned(),
            deck_ids: vec![first_deck.id.clone()],
            tag_ids: vec![first_tag.id.clone()],
            content: Default::default(),
            include_standard_recall: true,
            template_ids: Vec::new(),
            problem: None,
            explain: None,
            type_answer: None,
        })
        .unwrap();

    assert_eq!(created.title, "Cell membrane");
    assert_eq!(created.decks[0].id, first_deck.id);
    assert_eq!(created.tags[0].id, first_tag.id);
    assert_eq!(created.cards.len(), 1);
    assert_eq!(
        store.entity(&created.cards[0].id).unwrap().unwrap().kind,
        EntityKind::Card
    );

    let updated = library
        .update_concept(UpdateConceptInput {
            id: created.id.clone(),
            title: "Plasma membrane".to_owned(),
            deck_ids: vec![second_deck.id.clone()],
            tag_ids: vec![second_tag.id.clone()],
            content: Default::default(),
            include_standard_recall: true,
            template_ids: Vec::new(),
            problem: None,
            explain: None,
            type_answer: None,
        })
        .unwrap();

    assert_eq!(updated.title, "Plasma membrane");
    assert_eq!(updated.decks[0].id, second_deck.id);
    assert_eq!(updated.tags[0].id, second_tag.id);
    assert_eq!(store.entity(&created.id).unwrap().unwrap().revision, 2);

    let active_snapshot = library.search(LibraryQuery::default()).unwrap();

    assert_eq!(active_snapshot.concepts.len(), 1);
    let active_catalog = library.organizations().unwrap();

    assert_eq!(
        active_catalog
            .decks
            .iter()
            .find(|deck| deck.id == second_deck.id)
            .unwrap()
            .concept_count,
        1
    );
    assert_eq!(
        active_catalog
            .decks
            .iter()
            .find(|deck| deck.id == first_deck.id)
            .unwrap()
            .concept_count,
        0
    );
    assert_eq!(
        active_catalog
            .decks
            .iter()
            .find(|deck| deck.id == unused_deck.id)
            .unwrap()
            .concept_count,
        0
    );

    let archived = library.set_concept_archived(&created.id, true).unwrap();

    assert!(archived.archived);
    assert!(library
        .search(LibraryQuery::default())
        .unwrap()
        .concepts
        .is_empty());
    assert_eq!(
        library.search(LibraryQuery::default()).unwrap().archived_count,
        1
    );
    assert_eq!(
        library
            .search(LibraryQuery {
                include_archived: true,
                ..Default::default()
            })
            .unwrap()
            .concepts.len(),
        1
    );

    let restored = library.set_concept_archived(&created.id, false).unwrap();

    assert!(!restored.archived);

    library.delete_concept(&created.id).unwrap();
    library.delete_concept(&created.id).unwrap();

    assert!(
        library
            .search(LibraryQuery {
                include_archived: true,
                ..Default::default()
            })
            .unwrap()
            .concepts.is_empty()
    );
    assert!(store
        .entity(&created.id)
        .unwrap()
        .unwrap()
        .deleted_at
        .is_some());
    assert!(matches!(
        library.concept(&created.id),
        Err(LibraryError::ConceptNotFound(_))
    ));

    let removed_memberships: i64 = store
        .read_result(|connection| -> DataResult<i64> {
            Ok(connection.query_row(
                "SELECT COUNT(*)
                FROM concept_decks
                WHERE concept_id = ?1
                    AND removed_at IS NOT NULL",
                [&created.id],
                |row| row.get(0),
            )?)
        })
        .unwrap();

    assert_eq!(removed_memberships, 1);
}

#[test]
fn organization_names_are_validated_and_deleted_items_leave_assignments_safe() {
    let (_directory, store) = test_store();
    let library = ConceptLibrary::new(&store);
    let deck = library.create_deck("Languages".to_owned()).unwrap();
    let tag = library.create_tag("Speaking".to_owned()).unwrap();
    let concept = library
        .create_concept(CreateConceptInput {
            title: "Greetings".to_owned(),
            deck_ids: vec![deck.id.clone()],
            tag_ids: vec![tag.id.clone()],
            content: Default::default(),
            include_standard_recall: true,
            template_ids: Vec::new(),
            problem: None,
            explain: None,
            type_answer: None,
        })
        .unwrap();

    assert!(matches!(
        library.create_deck(" languages ".to_owned()),
        Err(LibraryError::DuplicateName { .. })
    ));
    assert!(matches!(
        library.create_tag("   ".to_owned()),
        Err(LibraryError::EmptyValue { .. })
    ));

    let renamed_deck = library.rename_deck(&deck.id, "French".to_owned()).unwrap();
    let renamed_tag = library
        .rename_tag(&tag.id, "Conversation".to_owned())
        .unwrap();

    assert_eq!(renamed_deck.name, "French");
    assert_eq!(renamed_tag.name, "Conversation");

    library.delete_deck(&deck.id).unwrap();
    library.delete_tag(&tag.id).unwrap();

    let detail = library.concept(&concept.id).unwrap();
    let catalog = library.organizations().unwrap();

    assert!(detail.decks.is_empty());
    assert!(detail.tags.is_empty());
    assert!(catalog.decks.is_empty());
    assert!(catalog.tags.is_empty());
    assert!(store
        .entity(&deck.id)
        .unwrap()
        .unwrap()
        .deleted_at
        .is_some());
    assert!(store.entity(&tag.id).unwrap().unwrap().deleted_at.is_some());
}

#[test]
fn invalid_assignments_and_no_op_edits_do_not_create_changes() {
    let (_directory, store) = test_store();
    let library = ConceptLibrary::new(&store);

    let invalid_create = library.create_concept(CreateConceptInput {
        title: "Invalid".to_owned(),
        deck_ids: vec!["missing-deck".to_owned()],
        tag_ids: Vec::new(),
        content: Default::default(),
        include_standard_recall: true,
        template_ids: Vec::new(),
        problem: None,
        explain: None,
        type_answer: None,
    });

    assert!(matches!(
        invalid_create,
        Err(LibraryError::InvalidSelection { .. })
    ));
    assert!(store.changes_after(0, 100).unwrap().is_empty());

    let concept = library
        .create_concept(CreateConceptInput {
            title: "Stable".to_owned(),
            deck_ids: Vec::new(),
            tag_ids: Vec::new(),
            content: Default::default(),
            include_standard_recall: true,
            template_ids: Vec::new(),
            problem: None,
            explain: None,
            type_answer: None,
        })
        .unwrap();
    let revision = store.entity(&concept.id).unwrap().unwrap().revision;

    library
        .update_concept(UpdateConceptInput {
            id: concept.id.clone(),
            title: concept.title,
            deck_ids: Vec::new(),
            tag_ids: Vec::new(),
            content: Default::default(),
            include_standard_recall: true,
            template_ids: Vec::new(),
            problem: None,
            explain: None,
            type_answer: None,
        })
        .unwrap();

    assert_eq!(
        store.entity(&concept.id).unwrap().unwrap().revision,
        revision
    );
}

#[test]
fn rich_content_and_media_references_update_transactionally() {
    let (_directory, store) = test_store();
    let library = ConceptLibrary::new(&store);
    let image_bytes = png_bytes();
    let media = library.import_image(&image_bytes).unwrap();
    let content = ConceptContent {
        schema_version: 1,
        prompt: json!({
            "type": "doc",
            "content": [{
                "type": "paragraph",
                "content": [{
                    "type": "text",
                    "text": "Identify this structure."
                }]
            }]
        }),
        answer: json!({
            "type": "doc",
            "content": [{
                "type": "codeBlock",
                "attrs": { "language": "rust" },
                "content": [{
                    "type": "text",
                    "text": "struct Cell;"
                }]
            }]
        }),
        feedback: AnswerFeedback {
            explanation: json!({
                "type": "doc",
                "content": [{
                    "type": "mediaImage",
                    "attrs": {
                        "mediaId": media.id,
                        "alt": "Why this is a cell",
                        "title": null
                    }
                }]
            }),
            common_mistakes: json!({
                "type": "doc",
                "content": [{
                    "type": "paragraph",
                    "content": [{
                        "type": "text",
                        "text": "Do not confuse the cell wall with the membrane."
                    }]
                }]
            }),
        },
    };
    let concept = library
        .create_concept(CreateConceptInput {
            title: "Rich concept".to_owned(),
            deck_ids: Vec::new(),
            tag_ids: Vec::new(),
            content: content.clone(),
            include_standard_recall: true,
            template_ids: Vec::new(),
            problem: None,
            explain: None,
            type_answer: None,
        })
        .unwrap();

    assert_eq!(concept.content, content);
    assert_eq!(concept.media, vec![media.clone()]);
    assert_eq!(library.media_bytes(&media.id).unwrap(), image_bytes);
    assert!(library.study_queue().unwrap().media.is_empty());

    let revision = store.entity(&concept.id).unwrap().unwrap().revision;
    let invalid_content = ConceptContent {
        schema_version: 1,
        prompt: ConceptContent::default().prompt,
        answer: ConceptContent::default().answer,
        feedback: AnswerFeedback {
            explanation: json!({
                "type": "doc",
                "content": [{
                    "type": "mediaImage",
                    "attrs": {
                        "mediaId": "018f1e2d-3c4b-7a69-8f10-123456789abc",
                        "alt": null,
                        "title": null
                    }
                }]
            }),
            ..Default::default()
        },
    };
    let invalid_update = library.update_concept(UpdateConceptInput {
        id: concept.id.clone(),
        title: concept.title.clone(),
        deck_ids: Vec::new(),
        tag_ids: Vec::new(),
        content: invalid_content,
        include_standard_recall: true,
        template_ids: Vec::new(),
        problem: None,
        explain: None,
        type_answer: None,
    });

    assert!(matches!(
        invalid_update,
        Err(LibraryError::MediaNotFound(_))
    ));
    assert_eq!(
        store.entity(&concept.id).unwrap().unwrap().revision,
        revision
    );

    let updated = library
        .update_concept(UpdateConceptInput {
            id: concept.id.clone(),
            title: concept.title,
            deck_ids: Vec::new(),
            tag_ids: Vec::new(),
            content: Default::default(),
            include_standard_recall: true,
            template_ids: Vec::new(),
            problem: None,
            explain: None,
            type_answer: None,
        })
        .unwrap();
    let removed_references: i64 = store
        .read_result(|connection| -> DataResult<i64> {
            Ok(connection.query_row(
                "SELECT COUNT(*)
                FROM concept_media
                WHERE concept_id = ?1
                    AND removed_at IS NOT NULL",
                [&concept.id],
                |row| row.get(0),
            )?)
        })
        .unwrap();

    assert_eq!(updated.content, ConceptContent::default());
    assert!(updated.media.is_empty());
    assert_eq!(removed_references, 1);
}

#[test]
fn associated_cards_are_visible_and_follow_card_tombstones() {
    let (_directory, store) = test_store();
    let library = ConceptLibrary::new(&store);
    let concept = library
        .create_concept(CreateConceptInput {
            title: "Linked card".to_owned(),
            deck_ids: Vec::new(),
            tag_ids: Vec::new(),
            content: Default::default(),
            include_standard_recall: true,
            template_ids: Vec::new(),
            problem: None,
            explain: None,
            type_answer: None,
        })
        .unwrap();

    let card_id = concept.cards[0].id.clone();

    assert_eq!(library.concept(&concept.id).unwrap().cards[0].id, card_id);
    assert_eq!(
        library.search(LibraryQuery::default()).unwrap().concepts[0].card_count,
        1
    );

    let duplicate_card = store.write(|transaction| {
        let card = transaction.create_entity(EntityKind::Card)?;

        transaction.execute(
            "INSERT INTO cards (
                entity_id,
                concept_id,
                retrieval_kind,
                template_id,
                last_change_id
            ) VALUES (?1, ?2, 'recall', NULL, ?3)",
            params![card.id, concept.id, card.last_change_id],
        )?;

        Ok(card)
    });

    assert!(duplicate_card.is_err());

    store
        .write(|transaction| transaction.soft_delete_entity(&card_id))
        .unwrap();

    assert!(library.concept(&concept.id).unwrap().cards.is_empty());
    assert_eq!(
        library.search(LibraryQuery::default()).unwrap().concepts[0].card_count,
        0
    );

    let second_card = store
        .write(|transaction| {
            let card = transaction.create_entity(EntityKind::Card)?;

            transaction.execute(
                "INSERT INTO cards (
                    entity_id,
                    concept_id,
                    retrieval_kind,
                    template_id,
                    last_change_id
                ) VALUES (?1, ?2, 'recall', NULL, ?3)",
                params![card.id, concept.id, card.last_change_id],
            )?;

            Ok(card)
        })
        .unwrap();

    library.delete_concept(&concept.id).unwrap();

    assert!(store
        .entity(&second_card.id)
        .unwrap()
        .unwrap()
        .deleted_at
        .is_some());
}

#[test]
fn retrieval_forms_are_selected_without_duplicates_and_schedule_independently() {
    let (_directory, store) = test_store();
    let library = ConceptLibrary::new(&store);
    let templates = TemplateLibrary::new(&store);
    let missing_forms = library.create_concept(CreateConceptInput {
        title: "No forms".to_owned(),
        deck_ids: Vec::new(),
        tag_ids: Vec::new(),
        content: Default::default(),
        include_standard_recall: false,
        template_ids: Vec::new(),
        problem: None,
        explain: None,
        type_answer: None,
    });

    assert!(matches!(
        missing_forms,
        Err(LibraryError::MissingRetrievalForm)
    ));
    assert!(library
        .search(LibraryQuery::default())
        .unwrap()
        .concepts
        .is_empty());

    let mut custom_template_content = TemplateContent::default();

    custom_template_content.mode = TemplateMode::Custom;

    let first_template = templates
        .create_template(CreateTemplateInput {
            name: "Answer first".to_owned(),
            content: custom_template_content,
        })
        .unwrap();
    let second_template = templates
        .create_template(CreateTemplateInput {
            name: "Prompt focused".to_owned(),
            content: TemplateContent::default(),
        })
        .unwrap();
    let media = library.import_image(&png_bytes()).unwrap();
    let concept = library
        .create_concept(CreateConceptInput {
            title: "Complementary practice".to_owned(),
            deck_ids: Vec::new(),
            tag_ids: Vec::new(),
            content: ConceptContent {
                schema_version: 1,
                prompt: json!({
                    "type": "doc",
                    "content": [{
                        "type": "mediaImage",
                        "attrs": {
                            "mediaId": media.id,
                            "alt": "Practice image",
                            "title": null
                        }
                    }]
                }),
                answer: ConceptContent::default().answer,
                feedback: Default::default(),
            },
            include_standard_recall: false,
            template_ids: vec![
                second_template.id.clone(),
                first_template.id.clone(),
                first_template.id.clone(),
            ],
            problem: None,
            explain: None,
            type_answer: None,
        })
        .unwrap();

    assert_eq!(concept.cards.len(), 2);
    assert_eq!(
        concept
            .cards
            .iter()
            .map(|card| card.template.as_ref().unwrap().name.as_str())
            .collect::<Vec<_>>(),
        vec!["Answer first", "Prompt focused"]
    );

    let reviewed_card = concept.cards[0].clone();
    let waiting_card = concept.cards[1].clone();
    let review_time = reviewed_card.due_at.max(waiting_card.due_at);
    let initial_queue = library.study_queue_at(review_time).unwrap();

    assert_eq!(initial_queue.media, vec![media]);

    library
        .record_review_at(
            RecordReviewInput {
                card_id: reviewed_card.id.clone(),
                rating: ReviewRating::Good,
            },
            review_time,
        )
        .unwrap();

    let queue = library.study_queue_at(review_time).unwrap();

    assert_eq!(queue.cards.len(), 1);
    assert!(queue.media.is_empty());
    assert_eq!(queue.cards[0].id, waiting_card.id);
    assert_eq!(
        queue.cards[0].template.as_ref().unwrap().id,
        second_template.id
    );

    let updated_template = templates
        .update_template(UpdateTemplateInput {
            id: second_template.id.clone(),
            name: "Focused prompt".to_owned(),
            content: second_template.content,
        })
        .unwrap();
    let updated_queue = library.study_queue_at(review_time).unwrap();

    assert_eq!(
        updated_queue.cards[0].template.as_ref().unwrap().name,
        updated_template.name
    );
}

#[test]
fn type_answer_settings_are_validated_and_normalized() {
    let (_directory, store) = test_store();
    let library = ConceptLibrary::new(&store);
    let create_type_answer = |answers| CreateConceptInput {
        title: "Typed response".to_owned(),
        deck_ids: Vec::new(),
        tag_ids: Vec::new(),
        content: Default::default(),
        include_standard_recall: false,
        template_ids: Vec::new(),
        problem: None,
        explain: None,
        type_answer: Some(TypeAnswerSettings {
            accepted_answers: answers,
        }),
    };

    assert!(matches!(
        library.create_concept(create_type_answer(Vec::new())),
        Err(LibraryError::MissingAcceptedAnswer)
    ));
    assert!(matches!(
        library.create_concept(create_type_answer(vec!["answer".to_owned(); 21])),
        Err(LibraryError::TooManyAcceptedAnswers { maximum: 20 })
    ));
    assert!(matches!(
        library.create_concept(create_type_answer(vec![
            "First answer".to_owned(),
            " first   ANSWER ".to_owned(),
        ])),
        Err(LibraryError::DuplicateAcceptedAnswer)
    ));
    assert!(matches!(
        library.create_concept(create_type_answer(vec!["x".repeat(501)])),
        Err(LibraryError::ValueTooLong {
            field: "Accepted answer",
            maximum: 500,
        })
    ));
    assert!(library
        .search(LibraryQuery::default())
        .unwrap()
        .concepts
        .is_empty());

    let concept = library
        .create_concept(create_type_answer(vec![
            "  García   Márquez  ".to_owned(),
            "C++".to_owned(),
        ]))
        .unwrap();
    let card = &concept.cards[0];

    assert_eq!(concept.cards.len(), 1);
    assert_eq!(card.retrieval_kind, RetrievalFormKind::TypeAnswer);
    assert_eq!(card.template, None);
    assert_eq!(
        card.type_answer.as_ref().unwrap().accepted_answers,
        vec!["García Márquez", "C++"]
    );
    let study_card = library.study_queue().unwrap().cards[0].clone();

    assert_eq!(study_card.id, card.id);
    assert_eq!(study_card.retrieval_kind, RetrievalFormKind::TypeAnswer);
    assert_eq!(study_card.type_answer, card.type_answer);
}

#[test]
fn type_answer_edits_keep_schedules_and_readded_forms_start_fresh() {
    let (_directory, store) = test_store();
    let library = ConceptLibrary::new(&store);
    let concept = library
        .create_concept(CreateConceptInput {
            title: "Capital of France".to_owned(),
            deck_ids: Vec::new(),
            tag_ids: Vec::new(),
            content: Default::default(),
            include_standard_recall: false,
            template_ids: Vec::new(),
            problem: None,
            explain: None,
            type_answer: Some(TypeAnswerSettings {
                accepted_answers: vec!["Paris".to_owned()],
            }),
        })
        .unwrap();
    let original_card = concept.cards[0].clone();
    let review = library
        .record_review_at(
            RecordReviewInput {
                card_id: original_card.id.clone(),
                rating: ReviewRating::Good,
            },
            original_card.due_at,
        )
        .unwrap();
    let updated = library
        .update_concept(UpdateConceptInput {
            id: concept.id.clone(),
            title: concept.title.clone(),
            deck_ids: Vec::new(),
            tag_ids: Vec::new(),
            content: concept.content.clone(),
            include_standard_recall: false,
            template_ids: Vec::new(),
            problem: None,
            explain: None,
            type_answer: Some(TypeAnswerSettings {
                accepted_answers: vec!["Paris".to_owned(), "The City of Paris".to_owned()],
            }),
        })
        .unwrap();
    let retained_card = updated.cards[0].clone();

    assert_eq!(retained_card.id, original_card.id);
    assert_eq!(retained_card.review_count, 1);
    assert_eq!(retained_card.due_at, review.due_at);
    assert_eq!(
        retained_card.type_answer.unwrap().accepted_answers,
        vec!["Paris", "The City of Paris"]
    );

    let without_type_answer = library
        .update_concept(UpdateConceptInput {
            id: concept.id.clone(),
            title: concept.title.clone(),
            deck_ids: Vec::new(),
            tag_ids: Vec::new(),
            content: concept.content.clone(),
            include_standard_recall: true,
            template_ids: Vec::new(),
            problem: None,
            explain: None,
            type_answer: None,
        })
        .unwrap();

    assert_eq!(without_type_answer.cards.len(), 1);
    assert_eq!(
        without_type_answer.cards[0].retrieval_kind,
        RetrievalFormKind::Recall
    );
    assert!(store
        .entity(&original_card.id)
        .unwrap()
        .unwrap()
        .deleted_at
        .is_some());

    let readded = library
        .update_concept(UpdateConceptInput {
            id: concept.id,
            title: concept.title,
            deck_ids: Vec::new(),
            tag_ids: Vec::new(),
            content: concept.content,
            include_standard_recall: true,
            template_ids: Vec::new(),
            problem: None,
            explain: None,
            type_answer: Some(TypeAnswerSettings {
                accepted_answers: vec!["Paris".to_owned()],
            }),
        })
        .unwrap();
    let readded_card = readded
        .cards
        .iter()
        .find(|card| card.retrieval_kind == RetrievalFormKind::TypeAnswer)
        .unwrap();

    assert_ne!(readded_card.id, original_card.id);
    assert_eq!(readded_card.scheduling_state, SchedulingState::New);
    assert_eq!(readded_card.review_count, 0);
}

#[test]
fn explain_settings_are_validated_normalized_and_queued() {
    let (_directory, store) = test_store();
    let library = ConceptLibrary::new(&store);
    let create_explain = |key_points| CreateConceptInput {
        title: "Photosynthesis".to_owned(),
        deck_ids: Vec::new(),
        tag_ids: Vec::new(),
        content: Default::default(),
        include_standard_recall: false,
        template_ids: Vec::new(),
        problem: None,
        explain: Some(ExplainSettings {
            focus: ExplainFocus::Why,
            key_points,
        }),
        type_answer: None,
    };

    assert!(matches!(
        library.create_concept(create_explain(Vec::new())),
        Err(LibraryError::MissingExplainKeyPoint)
    ));
    assert!(matches!(
        library.create_concept(create_explain(vec!["point".to_owned(); 13])),
        Err(LibraryError::TooManyExplainKeyPoints { maximum: 12 })
    ));
    assert!(matches!(
        library.create_concept(create_explain(vec![
            "Stored energy".to_owned(),
            " stored   ENERGY ".to_owned(),
        ])),
        Err(LibraryError::DuplicateExplainKeyPoint)
    ));
    assert!(matches!(
        library.create_concept(create_explain(vec!["x".repeat(281)])),
        Err(LibraryError::ValueTooLong {
            field: "Explain key point",
            maximum: 280,
        })
    ));
    assert!(library
        .search(LibraryQuery::default())
        .unwrap()
        .concepts
        .is_empty());

    let concept = library
        .create_concept(create_explain(vec![
            "  Light   energy becomes chemical energy.  ".to_owned(),
            "Carbon dioxide supplies carbon.".to_owned(),
        ]))
        .unwrap();
    let card = &concept.cards[0];

    assert_eq!(concept.cards.len(), 1);
    assert_eq!(card.retrieval_kind, RetrievalFormKind::Explain);
    assert_eq!(card.template, None);
    assert_eq!(
        card.explain.as_ref().unwrap().key_points,
        vec![
            "Light energy becomes chemical energy.",
            "Carbon dioxide supplies carbon.",
        ]
    );
    let study_card = library.study_queue().unwrap().cards[0].clone();

    assert_eq!(study_card.id, card.id);
    assert_eq!(study_card.retrieval_kind, RetrievalFormKind::Explain);
    assert_eq!(study_card.explain, card.explain);
}

#[test]
fn explain_edits_keep_schedules_and_readded_forms_start_fresh() {
    let (_directory, store) = test_store();
    let library = ConceptLibrary::new(&store);
    let concept = library
        .create_concept(CreateConceptInput {
            title: "Seasons".to_owned(),
            deck_ids: Vec::new(),
            tag_ids: Vec::new(),
            content: Default::default(),
            include_standard_recall: false,
            template_ids: Vec::new(),
            problem: None,
            explain: Some(ExplainSettings {
                focus: ExplainFocus::Why,
                key_points: vec!["Earth's axis is tilted.".to_owned()],
            }),
            type_answer: None,
        })
        .unwrap();
    let original_card = concept.cards[0].clone();
    let review = library
        .record_review_at(
            RecordReviewInput {
                card_id: original_card.id.clone(),
                rating: ReviewRating::Good,
            },
            original_card.due_at,
        )
        .unwrap();
    let updated = library
        .update_concept(UpdateConceptInput {
            id: concept.id.clone(),
            title: concept.title.clone(),
            deck_ids: Vec::new(),
            tag_ids: Vec::new(),
            content: concept.content.clone(),
            include_standard_recall: false,
            template_ids: Vec::new(),
            problem: None,
            explain: Some(ExplainSettings {
                focus: ExplainFocus::CauseAndEffect,
                key_points: vec![
                    "Earth's axis is tilted.".to_owned(),
                    "The hemispheres receive different sunlight.".to_owned(),
                ],
            }),
            type_answer: None,
        })
        .unwrap();
    let retained_card = updated.cards[0].clone();

    assert_eq!(retained_card.id, original_card.id);
    assert_eq!(retained_card.review_count, 1);
    assert_eq!(retained_card.due_at, review.due_at);
    assert_eq!(
        retained_card.explain.as_ref().unwrap().focus,
        ExplainFocus::CauseAndEffect
    );

    let without_explain = library
        .update_concept(UpdateConceptInput {
            id: concept.id.clone(),
            title: concept.title.clone(),
            deck_ids: Vec::new(),
            tag_ids: Vec::new(),
            content: concept.content.clone(),
            include_standard_recall: true,
            template_ids: Vec::new(),
            problem: None,
            explain: None,
            type_answer: None,
        })
        .unwrap();

    assert_eq!(without_explain.cards.len(), 1);
    assert_eq!(
        without_explain.cards[0].retrieval_kind,
        RetrievalFormKind::Recall
    );
    assert!(store
        .entity(&original_card.id)
        .unwrap()
        .unwrap()
        .deleted_at
        .is_some());

    let readded = library
        .update_concept(UpdateConceptInput {
            id: concept.id,
            title: concept.title,
            deck_ids: Vec::new(),
            tag_ids: Vec::new(),
            content: concept.content,
            include_standard_recall: true,
            template_ids: Vec::new(),
            problem: None,
            explain: Some(ExplainSettings {
                focus: ExplainFocus::CompareAndContrast,
                key_points: vec!["Summer and winter receive different light.".to_owned()],
            }),
            type_answer: None,
        })
        .unwrap();
    let readded_card = readded
        .cards
        .iter()
        .find(|card| card.retrieval_kind == RetrievalFormKind::Explain)
        .unwrap();

    assert_ne!(readded_card.id, original_card.id);
    assert_eq!(readded_card.scheduling_state, SchedulingState::New);
    assert_eq!(readded_card.review_count, 0);
}

#[test]
fn problem_checkpoints_are_validated_normalized_and_queued() {
    let (_directory, store) = test_store();
    let library = ConceptLibrary::new(&store);
    let media = library.import_image(&png_bytes()).unwrap();
    let content = ConceptContent {
        schema_version: 1,
        prompt: json!({
            "type": "doc",
            "content": [
                {
                    "type": "paragraph",
                    "content": [{
                        "type": "text",
                        "text": "Find the acceleration of the cart."
                    }]
                },
                {
                    "type": "mediaImage",
                    "attrs": {
                        "mediaId": media.id,
                        "alt": "A force diagram",
                        "title": null
                    }
                }
            ]
        }),
        answer: json!({
            "type": "doc",
            "content": [{
                "type": "codeBlock",
                "attrs": { "language": "rust" },
                "content": [{
                    "type": "text",
                    "text": "let acceleration = force / mass;"
                }]
            }]
        }),
        feedback: Default::default(),
    };
    let create_problem = |checkpoints| CreateConceptInput {
        title: "Cart acceleration".to_owned(),
        deck_ids: Vec::new(),
        tag_ids: Vec::new(),
        content: content.clone(),
        include_standard_recall: false,
        template_ids: Vec::new(),
        problem: Some(ProblemSettings { checkpoints }),
        explain: None,
        type_answer: None,
    };

    assert!(matches!(
        library.create_concept(CreateConceptInput {
            title: "Empty problem".to_owned(),
            deck_ids: Vec::new(),
            tag_ids: Vec::new(),
            content: ConceptContent::default(),
            include_standard_recall: false,
            template_ids: Vec::new(),
            problem: Some(ProblemSettings {
                checkpoints: vec!["Solve the problem.".to_owned()],
            }),
            explain: None,
            type_answer: None,
        }),
        Err(LibraryError::MissingProblemPrompt)
    ));

    assert!(matches!(
        library.create_concept(create_problem(Vec::new())),
        Err(LibraryError::MissingProblemCheckpoint)
    ));
    assert!(matches!(
        library.create_concept(create_problem(vec!["checkpoint".to_owned(); 13])),
        Err(LibraryError::TooManyProblemCheckpoints { maximum: 12 })
    ));
    assert!(matches!(
        library.create_concept(create_problem(vec![
            "Choose the equation".to_owned(),
            " choose   THE equation ".to_owned(),
        ])),
        Err(LibraryError::DuplicateProblemCheckpoint)
    ));
    assert!(matches!(
        library.create_concept(create_problem(vec!["x".repeat(281)])),
        Err(LibraryError::ValueTooLong {
            field: "Problem checkpoint",
            maximum: 280,
        })
    ));
    assert!(library
        .search(LibraryQuery::default())
        .unwrap()
        .concepts
        .is_empty());

    let concept = library
        .create_concept(create_problem(vec![
            "  Identify   the net force.  ".to_owned(),
            "Solve F = ma for acceleration.".to_owned(),
        ]))
        .unwrap();
    let card = &concept.cards[0];

    assert_eq!(concept.cards.len(), 1);
    assert_eq!(card.retrieval_kind, RetrievalFormKind::Problem);
    assert_eq!(card.template, None);
    assert_eq!(
        card.problem.as_ref().unwrap().checkpoints,
        vec!["Identify the net force.", "Solve F = ma for acceleration.",]
    );
    let study_card = library.study_queue().unwrap().cards[0].clone();

    assert_eq!(study_card.id, card.id);
    assert_eq!(study_card.retrieval_kind, RetrievalFormKind::Problem);
    assert_eq!(study_card.problem, card.problem);
    assert_eq!(study_card.content, concept.content);
    assert_eq!(library.study_queue().unwrap().media, vec![media]);
}

#[test]
fn problem_edits_keep_schedules_and_readded_forms_start_fresh() {
    let (_directory, store) = test_store();
    let library = ConceptLibrary::new(&store);
    let content = ConceptContent {
        schema_version: 1,
        prompt: json!({
            "type": "doc",
            "content": [{
                "type": "paragraph",
                "content": [{
                    "type": "text",
                    "text": "How long does a falling object take to reach the ground?"
                }]
            }]
        }),
        answer: ConceptContent::default().answer,
        feedback: Default::default(),
    };
    let concept = library
        .create_concept(CreateConceptInput {
            title: "Falling object".to_owned(),
            deck_ids: Vec::new(),
            tag_ids: Vec::new(),
            content,
            include_standard_recall: false,
            template_ids: Vec::new(),
            problem: Some(ProblemSettings {
                checkpoints: vec!["Choose a kinematic equation.".to_owned()],
            }),
            explain: None,
            type_answer: None,
        })
        .unwrap();
    let original_card = concept.cards[0].clone();
    let review = library
        .record_review_at(
            RecordReviewInput {
                card_id: original_card.id.clone(),
                rating: ReviewRating::Good,
            },
            original_card.due_at,
        )
        .unwrap();
    let updated = library
        .update_concept(UpdateConceptInput {
            id: concept.id.clone(),
            title: concept.title.clone(),
            deck_ids: Vec::new(),
            tag_ids: Vec::new(),
            content: concept.content.clone(),
            include_standard_recall: false,
            template_ids: Vec::new(),
            problem: Some(ProblemSettings {
                checkpoints: vec![
                    "Choose a kinematic equation.".to_owned(),
                    "Substitute the known values.".to_owned(),
                ],
            }),
            explain: None,
            type_answer: None,
        })
        .unwrap();
    let retained_card = updated.cards[0].clone();

    assert_eq!(retained_card.id, original_card.id);
    assert_eq!(retained_card.review_count, 1);
    assert_eq!(retained_card.due_at, review.due_at);
    assert_eq!(retained_card.problem.as_ref().unwrap().checkpoints.len(), 2);

    let without_problem = library
        .update_concept(UpdateConceptInput {
            id: concept.id.clone(),
            title: concept.title.clone(),
            deck_ids: Vec::new(),
            tag_ids: Vec::new(),
            content: concept.content.clone(),
            include_standard_recall: true,
            template_ids: Vec::new(),
            problem: None,
            explain: None,
            type_answer: None,
        })
        .unwrap();

    assert_eq!(without_problem.cards.len(), 1);
    assert_eq!(
        without_problem.cards[0].retrieval_kind,
        RetrievalFormKind::Recall
    );
    assert!(store
        .entity(&original_card.id)
        .unwrap()
        .unwrap()
        .deleted_at
        .is_some());

    let readded = library
        .update_concept(UpdateConceptInput {
            id: concept.id,
            title: concept.title,
            deck_ids: Vec::new(),
            tag_ids: Vec::new(),
            content: concept.content,
            include_standard_recall: true,
            template_ids: Vec::new(),
            problem: Some(ProblemSettings {
                checkpoints: vec!["Check the units.".to_owned()],
            }),
            explain: None,
            type_answer: None,
        })
        .unwrap();
    let readded_card = readded
        .cards
        .iter()
        .find(|card| card.retrieval_kind == RetrievalFormKind::Problem)
        .unwrap();

    assert_ne!(readded_card.id, original_card.id);
    assert_eq!(readded_card.scheduling_state, SchedulingState::New);
    assert_eq!(readded_card.review_count, 0);
}

#[test]
fn cloze_groups_schedule_independently_and_reconcile_by_identity() {
    let (_directory, store) = test_store();
    let library = ConceptLibrary::new(&store);
    let first_group = "018f1e2d-3c4b-7a69-8f10-123456789ab1";
    let second_group = "018f1e2d-3c4b-7a69-8f10-123456789ab2";
    let third_group = "018f1e2d-3c4b-7a69-8f10-123456789ab3";
    let cloze_text = |text: &str, group_id: &str| {
        json!({
            "type": "text",
            "text": text,
            "marks": [{
                "type": "cloze",
                "attrs": { "groupId": group_id }
            }]
        })
    };
    let prompt = |nodes| {
        json!({
            "type": "doc",
            "content": [{
                "type": "paragraph",
                "content": nodes
            }]
        })
    };
    let initial_content = ConceptContent {
        schema_version: 1,
        prompt: prompt(vec![
            json!({ "type": "text", "text": "The " }),
            cloze_text("inner mitochondrial membrane", first_group),
            json!({ "type": "text", "text": " produces " }),
            cloze_text("ATP", first_group),
            json!({ "type": "text", "text": " during " }),
            cloze_text("respiration", second_group),
        ]),
        answer: ConceptContent::default().answer,
        feedback: Default::default(),
    };
    let concept = library
        .create_concept(CreateConceptInput {
            title: "Cellular respiration".to_owned(),
            deck_ids: Vec::new(),
            tag_ids: Vec::new(),
            content: initial_content,
            include_standard_recall: false,
            template_ids: Vec::new(),
            problem: None,
            explain: None,
            type_answer: None,
        })
        .unwrap();
    let first_card = concept
        .cards
        .iter()
        .find(|card| card.cloze.as_ref().unwrap().group_id == first_group)
        .unwrap()
        .clone();
    let second_card = concept
        .cards
        .iter()
        .find(|card| card.cloze.as_ref().unwrap().group_id == second_group)
        .unwrap()
        .clone();

    assert_eq!(concept.cards.len(), 2);
    assert!(concept.cards.iter().all(|card| {
        card.retrieval_kind == RetrievalFormKind::Cloze
            && card.type_answer.is_none()
            && card.template.is_none()
    }));

    let review = library
        .record_review_at(
            RecordReviewInput {
                card_id: first_card.id.clone(),
                rating: ReviewRating::Good,
            },
            first_card.due_at,
        )
        .unwrap();
    let updated_content = ConceptContent {
        schema_version: 1,
        prompt: prompt(vec![
            json!({ "type": "text", "text": "Most " }),
            cloze_text("ATP", first_group),
            json!({ "type": "text", "text": " is made by " }),
            cloze_text("oxidative phosphorylation", third_group),
        ]),
        answer: concept.content.answer.clone(),
        feedback: concept.content.feedback.clone(),
    };
    let updated = library
        .update_concept(UpdateConceptInput {
            id: concept.id.clone(),
            title: concept.title.clone(),
            deck_ids: Vec::new(),
            tag_ids: Vec::new(),
            content: updated_content,
            include_standard_recall: false,
            template_ids: Vec::new(),
            problem: None,
            explain: None,
            type_answer: None,
        })
        .unwrap();
    let retained_card = updated
        .cards
        .iter()
        .find(|card| card.cloze.as_ref().unwrap().group_id == first_group)
        .unwrap();
    let new_card = updated
        .cards
        .iter()
        .find(|card| card.cloze.as_ref().unwrap().group_id == third_group)
        .unwrap();

    assert_eq!(updated.cards.len(), 2);
    assert_eq!(retained_card.id, first_card.id);
    assert_eq!(retained_card.review_count, 1);
    assert_eq!(retained_card.due_at, review.due_at);
    assert_eq!(new_card.scheduling_state, SchedulingState::New);
    assert!(store
        .entity(&second_card.id)
        .unwrap()
        .unwrap()
        .deleted_at
        .is_some());

    let readded_content = ConceptContent {
        schema_version: 1,
        prompt: prompt(vec![
            cloze_text("ATP", first_group),
            json!({ "type": "text", "text": " supports " }),
            cloze_text("cellular work", second_group),
        ]),
        answer: concept.content.answer,
        feedback: concept.content.feedback,
    };
    let readded = library
        .update_concept(UpdateConceptInput {
            id: concept.id,
            title: concept.title,
            deck_ids: Vec::new(),
            tag_ids: Vec::new(),
            content: readded_content,
            include_standard_recall: false,
            template_ids: Vec::new(),
            problem: None,
            explain: None,
            type_answer: None,
        })
        .unwrap();
    let readded_card = readded
        .cards
        .iter()
        .find(|card| card.cloze.as_ref().unwrap().group_id == second_group)
        .unwrap();

    assert_ne!(readded_card.id, second_card.id);
    assert_eq!(readded_card.scheduling_state, SchedulingState::New);
    assert_eq!(readded_card.review_count, 0);
}

#[test]
fn image_occlusion_groups_schedule_independently_and_include_source_media() {
    let (_directory, store) = test_store();
    let library = ConceptLibrary::new(&store);
    let media = library.import_image(&png_bytes()).unwrap();
    let first_group = "018f1e2d-3c4b-7a69-8f10-123456789ab1";
    let second_group = "018f1e2d-3c4b-7a69-8f10-123456789ab2";
    let third_group = "018f1e2d-3c4b-7a69-8f10-123456789ab3";
    let first_region = "018f1e2d-3c4b-7a69-8f10-123456789ab4";
    let grouped_region = "018f1e2d-3c4b-7a69-8f10-123456789ab5";
    let second_region = "018f1e2d-3c4b-7a69-8f10-123456789ab6";
    let third_region = "018f1e2d-3c4b-7a69-8f10-123456789ab7";
    let replacement_region = "018f1e2d-3c4b-7a69-8f10-123456789ab8";
    let region = |id: &str, group_id: &str, x: f64, y: f64| {
        json!({
            "id": id,
            "groupId": group_id,
            "x": x,
            "y": y,
            "width": 0.2,
            "height": 0.2
        })
    };
    let prompt = |regions| {
        json!({
            "type": "doc",
            "content": [{
                "type": "mediaImage",
                "attrs": {
                    "mediaId": media.id,
                    "alt": "Cell diagram",
                    "title": null,
                    "occlusionRegions": regions
                }
            }]
        })
    };
    let initial_content = ConceptContent {
        schema_version: 1,
        prompt: prompt(vec![
            region(first_region, first_group, 0.05, 0.1),
            region(grouped_region, first_group, 0.35, 0.1),
            region(second_region, second_group, 0.1, 0.55),
        ]),
        answer: ConceptContent::default().answer,
        feedback: Default::default(),
    };
    let concept = library
        .create_concept(CreateConceptInput {
            title: "Cell diagram".to_owned(),
            deck_ids: Vec::new(),
            tag_ids: Vec::new(),
            content: initial_content,
            include_standard_recall: false,
            template_ids: Vec::new(),
            problem: None,
            explain: None,
            type_answer: None,
        })
        .unwrap();
    let first_card = concept
        .cards
        .iter()
        .find(|card| card.image_occlusion.as_ref().unwrap().group_id == first_group)
        .unwrap()
        .clone();
    let second_card = concept
        .cards
        .iter()
        .find(|card| card.image_occlusion.as_ref().unwrap().group_id == second_group)
        .unwrap()
        .clone();
    let latest_initial_due_at = concept.cards.iter().map(|card| card.due_at).max().unwrap();
    let queue = library.study_queue_at(latest_initial_due_at).unwrap();

    assert_eq!(concept.cards.len(), 2);
    assert!(concept.cards.iter().all(|card| {
        card.retrieval_kind == RetrievalFormKind::ImageOcclusion
            && card.cloze.is_none()
            && card.type_answer.is_none()
            && card.template.is_none()
    }));
    assert_eq!(queue.media, vec![media.clone()]);
    assert_eq!(queue.cards.len(), 2);
    assert!(queue
        .cards
        .iter()
        .all(|card| card.image_occlusion.is_some()));

    let review = library
        .record_review_at(
            RecordReviewInput {
                card_id: first_card.id.clone(),
                rating: ReviewRating::Good,
            },
            first_card.due_at,
        )
        .unwrap();
    let updated_content = ConceptContent {
        schema_version: 1,
        prompt: prompt(vec![
            region(first_region, first_group, 0.15, 0.15),
            region(third_region, third_group, 0.6, 0.55),
        ]),
        answer: concept.content.answer.clone(),
        feedback: concept.content.feedback.clone(),
    };
    let updated = library
        .update_concept(UpdateConceptInput {
            id: concept.id.clone(),
            title: concept.title.clone(),
            deck_ids: Vec::new(),
            tag_ids: Vec::new(),
            content: updated_content,
            include_standard_recall: false,
            template_ids: Vec::new(),
            problem: None,
            explain: None,
            type_answer: None,
        })
        .unwrap();
    let retained_card = updated
        .cards
        .iter()
        .find(|card| card.image_occlusion.as_ref().unwrap().group_id == first_group)
        .unwrap();
    let new_card = updated
        .cards
        .iter()
        .find(|card| card.image_occlusion.as_ref().unwrap().group_id == third_group)
        .unwrap();

    assert_eq!(updated.cards.len(), 2);
    assert_eq!(retained_card.id, first_card.id);
    assert_eq!(retained_card.review_count, 1);
    assert_eq!(retained_card.due_at, review.due_at);
    assert_eq!(new_card.scheduling_state, SchedulingState::New);
    assert!(store
        .entity(&second_card.id)
        .unwrap()
        .unwrap()
        .deleted_at
        .is_some());

    let readded_content = ConceptContent {
        schema_version: 1,
        prompt: prompt(vec![
            region(first_region, first_group, 0.15, 0.15),
            region(replacement_region, second_group, 0.1, 0.55),
        ]),
        answer: concept.content.answer,
        feedback: concept.content.feedback,
    };
    let readded = library
        .update_concept(UpdateConceptInput {
            id: concept.id,
            title: concept.title,
            deck_ids: Vec::new(),
            tag_ids: Vec::new(),
            content: readded_content,
            include_standard_recall: false,
            template_ids: Vec::new(),
            problem: None,
            explain: None,
            type_answer: None,
        })
        .unwrap();
    let readded_card = readded
        .cards
        .iter()
        .find(|card| card.image_occlusion.as_ref().unwrap().group_id == second_group)
        .unwrap();

    assert_ne!(readded_card.id, second_card.id);
    assert_eq!(readded_card.scheduling_state, SchedulingState::New);
    assert_eq!(readded_card.review_count, 0);
}

#[test]
fn changing_retrieval_forms_retains_selected_schedules_and_protects_templates_in_use() {
    let (_directory, store) = test_store();
    let library = ConceptLibrary::new(&store);
    let templates = TemplateLibrary::new(&store);
    let removed_template = templates
        .create_template(CreateTemplateInput {
            name: "Temporary".to_owned(),
            content: TemplateContent::default(),
        })
        .unwrap();
    let retained_template = templates
        .create_template(CreateTemplateInput {
            name: "Retained".to_owned(),
            content: TemplateContent::default(),
        })
        .unwrap();
    let concept = library
        .create_concept(CreateConceptInput {
            title: "Managed forms".to_owned(),
            deck_ids: Vec::new(),
            tag_ids: Vec::new(),
            content: Default::default(),
            include_standard_recall: false,
            template_ids: vec![removed_template.id.clone(), retained_template.id.clone()],
            problem: None,
            explain: None,
            type_answer: None,
        })
        .unwrap();
    let removed_card_id = concept
        .cards
        .iter()
        .find(|card| card.template.as_ref().unwrap().id == removed_template.id)
        .unwrap()
        .id
        .clone();
    let retained_card_id = concept
        .cards
        .iter()
        .find(|card| card.template.as_ref().unwrap().id == retained_template.id)
        .unwrap()
        .id
        .clone();

    let updated = library
        .update_concept(UpdateConceptInput {
            id: concept.id.clone(),
            title: concept.title,
            deck_ids: Vec::new(),
            tag_ids: Vec::new(),
            content: concept.content,
            include_standard_recall: true,
            template_ids: vec![retained_template.id.clone()],
            problem: None,
            explain: None,
            type_answer: None,
        })
        .unwrap();

    assert_eq!(updated.cards.len(), 2);
    assert!(updated.cards.iter().any(|card| card.template.is_none()));
    assert!(updated.cards.iter().any(|card| card.id == retained_card_id));
    assert!(store
        .entity(&removed_card_id)
        .unwrap()
        .unwrap()
        .deleted_at
        .is_some());
    assert_eq!(
        templates
            .catalog()
            .unwrap()
            .templates
            .into_iter()
            .find(|template| template.id == retained_template.id)
            .unwrap()
            .retrieval_form_count,
        1
    );
    assert!(matches!(
        templates.delete_template(&retained_template.id),
        Err(LibraryError::TemplateInUse {
            retrieval_form_count: 1,
            ..
        })
    ));
    assert!(store
        .write(|transaction| transaction.soft_delete_entity(&retained_template.id))
        .is_err());

    library
        .update_concept(UpdateConceptInput {
            id: concept.id,
            title: updated.title,
            deck_ids: Vec::new(),
            tag_ids: Vec::new(),
            content: updated.content,
            include_standard_recall: true,
            template_ids: Vec::new(),
            problem: None,
            explain: None,
            type_answer: None,
        })
        .unwrap();

    templates.delete_template(&removed_template.id).unwrap();
    templates.delete_template(&retained_template.id).unwrap();
}

#[test]
fn study_cards_include_only_active_unarchived_recall_cards() {
    let (_directory, store) = test_store();
    let library = ConceptLibrary::new(&store);
    let content = ConceptContent {
        schema_version: 1,
        prompt: json!({
            "type": "doc",
            "content": [{
                "type": "paragraph",
                "content": [{ "type": "text", "text": "A prompt" }]
            }]
        }),
        answer: json!({
            "type": "doc",
            "content": [{
                "type": "paragraph",
                "content": [{ "type": "text", "text": "An answer" }]
            }]
        }),
        feedback: Default::default(),
    };
    let active = library
        .create_concept(CreateConceptInput {
            title: "Active concept".to_owned(),
            deck_ids: Vec::new(),
            tag_ids: Vec::new(),
            content: content.clone(),
            include_standard_recall: true,
            template_ids: Vec::new(),
            problem: None,
            explain: None,
            type_answer: None,
        })
        .unwrap();
    let archived = library
        .create_concept(CreateConceptInput {
            title: "Archived concept".to_owned(),
            deck_ids: Vec::new(),
            tag_ids: Vec::new(),
            content: Default::default(),
            include_standard_recall: true,
            template_ids: Vec::new(),
            problem: None,
            explain: None,
            type_answer: None,
        })
        .unwrap();

    library.set_concept_archived(&archived.id, true).unwrap();

    let cards = library.study_queue().unwrap().cards;

    assert_eq!(cards.len(), 1);
    assert_eq!(cards[0].id, active.cards[0].id);
    assert_eq!(cards[0].concept_id, active.id);
    assert_eq!(cards[0].concept_title, active.title);
    assert_eq!(cards[0].content, content);

    store
        .write(|transaction| transaction.soft_delete_entity(&cards[0].id))
        .unwrap();

    assert!(library.study_queue().unwrap().cards.is_empty());
}

#[test]
fn mixed_practice_reorders_due_cards_without_changing_eligibility() {
    let (_directory, store) = test_store();
    let library = ConceptLibrary::new(&store);
    let shared_tag = library.create_tag("Related ideas".to_owned()).unwrap();
    let first = library
        .create_concept(CreateConceptInput {
            title: "First concept".to_owned(),
            deck_ids: Vec::new(),
            tag_ids: vec![shared_tag.id.clone()],
            content: Default::default(),
            include_standard_recall: true,
            template_ids: Vec::new(),
            problem: None,
            explain: None,
            type_answer: Some(TypeAnswerSettings {
                accepted_answers: vec!["First".to_owned()],
            }),
        })
        .unwrap();

    std::thread::sleep(Duration::from_millis(2));

    let unrelated = library
        .create_concept(CreateConceptInput {
            title: "Unrelated concept".to_owned(),
            deck_ids: Vec::new(),
            tag_ids: Vec::new(),
            content: Default::default(),
            include_standard_recall: true,
            template_ids: Vec::new(),
            problem: None,
            explain: None,
            type_answer: None,
        })
        .unwrap();

    std::thread::sleep(Duration::from_millis(2));

    let contrast = library
        .create_concept(CreateConceptInput {
            title: "Contrasting concept".to_owned(),
            deck_ids: Vec::new(),
            tag_ids: vec![shared_tag.id],
            content: Default::default(),
            include_standard_recall: true,
            template_ids: Vec::new(),
            problem: None,
            explain: None,
            type_answer: None,
        })
        .unwrap();

    std::thread::sleep(Duration::from_millis(2));

    let future = library
        .create_concept(CreateConceptInput {
            title: "Not due".to_owned(),
            deck_ids: Vec::new(),
            tag_ids: Vec::new(),
            content: Default::default(),
            include_standard_recall: true,
            template_ids: Vec::new(),
            problem: None,
            explain: None,
            type_answer: None,
        })
        .unwrap();
    let first_recall = first
        .cards
        .iter()
        .find(|card| card.retrieval_kind == RetrievalFormKind::Recall)
        .unwrap();
    let first_type_answer = first
        .cards
        .iter()
        .find(|card| card.retrieval_kind == RetrievalFormKind::TypeAnswer)
        .unwrap();
    let due_order = library.study_queue_at(contrast.cards[0].due_at).unwrap();
    let first_form_id = due_order.cards[0].id.as_str();
    let second_form_id = due_order.cards[1].id.as_str();

    assert!(!due_order.mixed_practice_enabled);
    assert_eq!(due_order.total_cards, 5);
    assert_eq!(due_order.next_due_at, Some(future.cards[0].due_at));
    assert_eq!(due_order.cards[0].concept_id, first.id);
    assert_eq!(due_order.cards[1].concept_id, first.id);
    assert!([first_recall.id.as_str(), first_type_answer.id.as_str()].contains(&first_form_id));
    assert!([first_recall.id.as_str(), first_type_answer.id.as_str()].contains(&second_form_id));
    assert_eq!(
        due_order
            .cards
            .iter()
            .map(|card| card.id.as_str())
            .collect::<Vec<_>>(),
        vec![
            first_form_id,
            second_form_id,
            unrelated.cards[0].id.as_str(),
            contrast.cards[0].id.as_str(),
        ]
    );

    library.set_mixed_practice_enabled(true).unwrap();

    let mixed = library.study_queue_at(contrast.cards[0].due_at).unwrap();
    let repeated = library.study_queue_at(contrast.cards[0].due_at).unwrap();

    assert!(mixed.mixed_practice_enabled);
    assert_eq!(mixed.total_cards, 5);
    assert_eq!(mixed.next_due_at, Some(future.cards[0].due_at));
    assert_eq!(mixed, repeated);
    assert_eq!(
        mixed
            .cards
            .iter()
            .map(|card| card.id.as_str())
            .collect::<Vec<_>>(),
        vec![
            first_form_id,
            contrast.cards[0].id.as_str(),
            second_form_id,
            unrelated.cards[0].id.as_str(),
        ]
    );
}

#[test]
fn device_preferences_are_durable_and_stay_out_of_change_tracking() {
    let directory = tempdir().unwrap();

    {
        let store = LocalDataStore::open(directory.path()).unwrap();
        let library = ConceptLibrary::new(&store);
        let defaults = library.device_preferences().unwrap();

        assert_eq!(defaults.grading_mode, GradingMode::Simple);
        assert_eq!(defaults.startup_destination, StartupDestination::Study);
        assert!(!defaults.pretesting_enabled);
        assert!(!defaults.mixed_practice_enabled);
        assert_eq!(
            defaults.appearance,
            AppearancePreferences {
                theme: AppearanceTheme::Aubergine,
                reading_font: ReadingFont::Inter,
                reading_text_size: ReadingTextSize::Medium,
                motion_preference: MotionPreference::System,
            }
        );

        let changes_before = store.changes_after(0, 100).unwrap();
        let preferences = library.set_grading_mode(GradingMode::Advanced).unwrap();

        assert_eq!(preferences.grading_mode, GradingMode::Advanced);
        assert!(!preferences.pretesting_enabled);
        assert!(!preferences.mixed_practice_enabled);
        assert_eq!(preferences.startup_destination, StartupDestination::Study);

        let preferences = library
            .set_startup_destination(StartupDestination::Library)
            .unwrap();

        assert_eq!(preferences.grading_mode, GradingMode::Advanced);
        assert_eq!(preferences.startup_destination, StartupDestination::Library);

        let preferences = library.set_pretesting_enabled(true).unwrap();

        assert!(preferences.pretesting_enabled);

        let preferences = library.set_mixed_practice_enabled(true).unwrap();

        assert!(preferences.mixed_practice_enabled);

        let appearance = AppearancePreferences {
            theme: AppearanceTheme::RosePineDawn,
            reading_font: ReadingFont::SourceSerif4,
            reading_text_size: ReadingTextSize::Large,
            motion_preference: MotionPreference::Reduced,
        };
        let preferences = library
            .set_appearance_preferences(appearance.clone())
            .unwrap();

        assert_eq!(preferences.grading_mode, GradingMode::Advanced);
        assert_eq!(preferences.startup_destination, StartupDestination::Library);
        assert_eq!(preferences.appearance, appearance);
        assert_eq!(store.changes_after(0, 100).unwrap(), changes_before);
    }

    let reopened_store = LocalDataStore::open(directory.path()).unwrap();
    let reopened_library = ConceptLibrary::new(&reopened_store);

    let preferences = reopened_library.device_preferences().unwrap();

    assert_eq!(preferences.grading_mode, GradingMode::Advanced);
    assert!(preferences.pretesting_enabled);
    assert!(preferences.mixed_practice_enabled);
    assert_eq!(preferences.startup_destination, StartupDestination::Library);
    assert_eq!(
        preferences.appearance,
        AppearancePreferences {
            theme: AppearanceTheme::RosePineDawn,
            reading_font: ReadingFont::SourceSerif4,
            reading_text_size: ReadingTextSize::Large,
            motion_preference: MotionPreference::Reduced,
        }
    );
}

#[test]
fn pretests_are_concept_level_events_separate_from_fsrs_history() {
    let (_directory, store) = test_store();
    let library = ConceptLibrary::new(&store);
    let concept = library
        .create_concept(CreateConceptInput {
            title: "Pretest target".to_owned(),
            deck_ids: Vec::new(),
            tag_ids: Vec::new(),
            content: Default::default(),
            include_standard_recall: true,
            template_ids: Vec::new(),
            problem: None,
            explain: None,
            type_answer: Some(TypeAnswerSettings {
                accepted_answers: vec!["Answer".to_owned()],
            }),
        })
        .unwrap();
    let queue = library.study_queue().unwrap();

    assert_eq!(queue.cards.len(), 2);
    assert!(queue.cards.iter().all(|card| card.pretest_eligible));

    let card = queue.cards[0].clone();
    let occurred_at = queue.cards.iter().map(|card| card.due_at).max().unwrap();
    let pretest = library
        .record_pretest_at(
            RecordPretestInput {
                card_id: card.id.clone(),
                outcome: PretestOutcome::Attempted,
            },
            occurred_at,
        )
        .unwrap();

    assert_eq!(pretest.concept_id, concept.id);
    assert_eq!(pretest.card_id, card.id);
    assert_eq!(pretest.outcome, PretestOutcome::Attempted);
    assert_eq!(pretest.occurred_at, occurred_at);
    assert_eq!(
        store.entity(&pretest.pretest_id).unwrap().unwrap().kind,
        EntityKind::Pretest
    );

    let queue_after_pretest = library.study_queue_at(occurred_at).unwrap();

    assert_eq!(queue_after_pretest.cards.len(), 2);
    assert!(queue_after_pretest
        .cards
        .iter()
        .all(|card| !card.pretest_eligible));

    let (review_count, lapse_count, review_rows, pretest_rows): (i64, i64, i64, i64) = store
        .read_result(|connection| -> DataResult<_> {
            Ok((
                connection.query_row(
                    "SELECT SUM(review_count) FROM card_scheduling
                    INNER JOIN cards ON cards.entity_id = card_scheduling.card_id
                    WHERE cards.concept_id = ?1",
                    [&concept.id],
                    |row| row.get(0),
                )?,
                connection.query_row(
                    "SELECT SUM(lapse_count) FROM card_scheduling
                    INNER JOIN cards ON cards.entity_id = card_scheduling.card_id
                    WHERE cards.concept_id = ?1",
                    [&concept.id],
                    |row| row.get(0),
                )?,
                connection.query_row(
                    "SELECT COUNT(*) FROM reviews
                    INNER JOIN cards ON cards.entity_id = reviews.card_id
                    WHERE cards.concept_id = ?1",
                    [&concept.id],
                    |row| row.get(0),
                )?,
                connection.query_row(
                    "SELECT COUNT(*) FROM pretests WHERE concept_id = ?1",
                    [&concept.id],
                    |row| row.get(0),
                )?,
            ))
        })
        .unwrap();

    assert_eq!(
        (review_count, lapse_count, review_rows, pretest_rows),
        (0, 0, 0, 1)
    );

    assert!(store
        .write(|transaction| {
            transaction.execute(
                "UPDATE pretests SET outcome = 'skipped' WHERE entity_id = ?1",
                [&pretest.pretest_id],
            )?;

            Ok(())
        })
        .is_err());
    assert!(store
        .write(|transaction| {
            transaction.execute(
                "DELETE FROM pretests WHERE entity_id = ?1",
                [&pretest.pretest_id],
            )?;

            Ok(())
        })
        .is_err());
    assert!(store
        .write(|transaction| transaction.soft_delete_entity(&pretest.pretest_id))
        .is_err());

    let changes_after_pretest = store.changes_after(0, 100).unwrap();
    let duplicate = library
        .record_pretest_at(
            RecordPretestInput {
                card_id: queue.cards[1].id.clone(),
                outcome: PretestOutcome::Skipped,
            },
            occurred_at + 1,
        )
        .unwrap();

    assert_eq!(duplicate, pretest);
    assert_eq!(store.changes_after(0, 100).unwrap(), changes_after_pretest);

    let skipped_concept = library
        .create_concept(CreateConceptInput {
            title: "Skipped pretest".to_owned(),
            deck_ids: Vec::new(),
            tag_ids: Vec::new(),
            content: Default::default(),
            include_standard_recall: true,
            template_ids: Vec::new(),
            problem: None,
            explain: None,
            type_answer: None,
        })
        .unwrap();
    let skipped_card = library
        .study_queue()
        .unwrap()
        .cards
        .into_iter()
        .find(|card| card.concept_id == skipped_concept.id)
        .unwrap();
    let skipped = library
        .record_pretest_at(
            RecordPretestInput {
                card_id: skipped_card.id,
                outcome: PretestOutcome::Skipped,
            },
            skipped_card.due_at,
        )
        .unwrap();

    assert_eq!(skipped.concept_id, skipped_concept.id);
    assert_eq!(skipped.outcome, PretestOutcome::Skipped);
}

#[test]
fn reviewed_concepts_do_not_become_pretest_eligible_after_undo() {
    let (_directory, store) = test_store();
    let library = ConceptLibrary::new(&store);
    library
        .create_concept(CreateConceptInput {
            title: "Previously studied".to_owned(),
            deck_ids: Vec::new(),
            tag_ids: Vec::new(),
            content: Default::default(),
            include_standard_recall: true,
            template_ids: Vec::new(),
            problem: None,
            explain: None,
            type_answer: None,
        })
        .unwrap();
    let card = library.study_queue().unwrap().cards[0].clone();
    let review = library
        .record_review_at(
            RecordReviewInput {
                card_id: card.id.clone(),
                rating: ReviewRating::Good,
            },
            card.due_at,
        )
        .unwrap();

    library
        .reverse_review_at(
            ReverseReviewInput {
                review_id: review.review_id,
            },
            review.reviewed_at + 1,
        )
        .unwrap();

    let queue = library.study_queue_at(review.reviewed_at + 1).unwrap();

    assert_eq!(queue.cards.len(), 1);
    assert!(!queue.cards[0].pretest_eligible);
    assert!(matches!(
        library.record_pretest_at(
            RecordPretestInput {
                card_id: card.id,
                outcome: PretestOutcome::Attempted,
            },
            review.reviewed_at + 1,
        ),
        Err(LibraryError::PretestNotEligible(_))
    ));
}

#[test]
fn scheduling_settings_create_immutable_configuration_revisions() {
    let (_directory, store) = test_store();
    let library = ConceptLibrary::new(&store);
    let defaults = library.scheduling_settings().unwrap();

    assert_eq!(defaults.algorithm_version, "6.6.1");
    assert_eq!(defaults.desired_retention, 0.9);
    assert_eq!(defaults.maximum_interval_days, 36_500);

    let original_configuration: (String, String) = store
        .read_result(|connection| -> DataResult<_> {
            Ok(connection.query_row(
                "SELECT configuration_id, parameters_json
                FROM active_scheduler_configuration
                INNER JOIN scheduler_configurations
                    ON scheduler_configurations.id = configuration_id
                WHERE singleton = 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )?)
        })
        .unwrap();
    let updated = library
        .update_scheduling_settings(UpdateSchedulingSettingsInput {
            desired_retention: 0.92,
            maximum_interval_days: 3_650,
        })
        .unwrap();
    let (active_id, parameters_json, configuration_count): (String, String, i64) = store
        .read_result(|connection| -> DataResult<_> {
            let (active_id, parameters_json) = connection.query_row(
                "SELECT configuration_id, parameters_json
                FROM active_scheduler_configuration
                INNER JOIN scheduler_configurations
                    ON scheduler_configurations.id = configuration_id
                WHERE singleton = 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )?;
            let configuration_count = connection.query_row(
                "SELECT COUNT(*) FROM scheduler_configurations",
                [],
                |row| row.get(0),
            )?;

            Ok((active_id, parameters_json, configuration_count))
        })
        .unwrap();

    assert_eq!(updated.desired_retention, 0.92);
    assert_eq!(updated.maximum_interval_days, 3_650);
    assert_ne!(active_id, original_configuration.0);
    assert_eq!(parameters_json, original_configuration.1);
    assert_eq!(configuration_count, 2);
    assert_eq!(
        uuid::Uuid::parse_str(&active_id).unwrap().get_version(),
        Some(uuid::Version::SortRand)
    );

    let unchanged = library
        .update_scheduling_settings(UpdateSchedulingSettingsInput {
            desired_retention: 0.92,
            maximum_interval_days: 3_650,
        })
        .unwrap();
    let configuration_count: i64 = store
        .read_result(|connection| -> DataResult<_> {
            Ok(connection.query_row(
                "SELECT COUNT(*) FROM scheduler_configurations",
                [],
                |row| row.get(0),
            )?)
        })
        .unwrap();

    assert_eq!(unchanged, updated);
    assert_eq!(configuration_count, 2);

    let altered_original: DataResult<()> = store.write(|transaction| {
        transaction.execute(
            "UPDATE scheduler_configurations
            SET desired_retention = 0.91
            WHERE id = ?1",
            [&original_configuration.0],
        )?;

        Ok(())
    });

    assert!(altered_original.is_err());
}

#[test]
fn scheduling_settings_validate_bounds_without_creating_revisions() {
    let (_directory, store) = test_store();
    let library = ConceptLibrary::new(&store);

    for desired_retention in [f64::NAN, 0.79, 0.98] {
        assert!(matches!(
            library.update_scheduling_settings(UpdateSchedulingSettingsInput {
                desired_retention,
                maximum_interval_days: 365,
            }),
            Err(LibraryError::InvalidDesiredRetention { .. })
        ));
    }

    for maximum_interval_days in [0, 36_501] {
        assert!(matches!(
            library.update_scheduling_settings(UpdateSchedulingSettingsInput {
                desired_retention: 0.9,
                maximum_interval_days,
            }),
            Err(LibraryError::InvalidMaximumInterval { .. })
        ));
    }

    let configuration_count: i64 = store
        .read_result(|connection| -> DataResult<_> {
            Ok(connection.query_row(
                "SELECT COUNT(*) FROM scheduler_configurations",
                [],
                |row| row.get(0),
            )?)
        })
        .unwrap();

    assert_eq!(configuration_count, 1);
}

#[test]
fn scheduling_changes_leave_existing_due_dates_and_cap_future_intervals() {
    let (_directory, store) = test_store();
    let library = ConceptLibrary::new(&store);
    let concept = library
        .create_concept(CreateConceptInput {
            title: "Future scheduling settings".to_owned(),
            deck_ids: Vec::new(),
            tag_ids: Vec::new(),
            content: Default::default(),
            include_standard_recall: true,
            template_ids: Vec::new(),
            problem: None,
            explain: None,
            type_answer: None,
        })
        .unwrap();
    let card = library.study_queue().unwrap().cards[0].clone();
    let first_review = library
        .record_review_at(
            RecordReviewInput {
                card_id: card.id.clone(),
                rating: ReviewRating::Good,
            },
            card.due_at,
        )
        .unwrap();

    library
        .update_scheduling_settings(UpdateSchedulingSettingsInput {
            desired_retention: 0.9,
            maximum_interval_days: 1,
        })
        .unwrap();

    let due_at_after_settings: i64 = store
        .read_result(|connection| -> DataResult<_> {
            Ok(connection.query_row(
                "SELECT due_at FROM card_scheduling WHERE card_id = ?1",
                [&card.id],
                |row| row.get(0),
            )?)
        })
        .unwrap();

    assert_eq!(due_at_after_settings, first_review.due_at);

    let second_review = library
        .record_review_at(
            RecordReviewInput {
                card_id: card.id.clone(),
                rating: ReviewRating::Easy,
            },
            first_review.due_at,
        )
        .unwrap();
    let configuration_ids: Vec<String> = store
        .read_result(|connection| -> DataResult<_> {
            let mut statement = connection.prepare(
                "SELECT scheduler_configuration_id
                FROM reviews
                WHERE card_id = ?1
                ORDER BY reviewed_at, entity_id",
            )?;
            let rows = statement.query_map([&card.id], |row| row.get(0))?;

            Ok(rows.collect::<Result<_, _>>()?)
        })
        .unwrap();

    assert_eq!(second_review.scheduled_interval_days, 1.0);
    assert_eq!(second_review.due_at, second_review.reviewed_at + 86_400_000);
    assert_eq!(configuration_ids.len(), 2);
    assert_ne!(configuration_ids[0], configuration_ids[1]);
    assert_eq!(concept.cards[0].id, card.id);
}

#[test]
fn a_failed_scheduling_settings_write_rolls_back_the_revision() {
    let (_directory, store) = test_store();
    let library = ConceptLibrary::new(&store);

    store
        .write(|transaction| {
            transaction.execute_batch(
                "CREATE TRIGGER force_scheduling_settings_failure
                BEFORE UPDATE ON active_scheduler_configuration
                FOR EACH ROW
                BEGIN
                    SELECT RAISE(ABORT, 'forced scheduling settings failure');
                END;",
            )?;

            Ok(())
        })
        .unwrap();

    let result = library.update_scheduling_settings(UpdateSchedulingSettingsInput {
        desired_retention: 0.91,
        maximum_interval_days: 3_650,
    });
    let configuration_count: i64 = store
        .read_result(|connection| -> DataResult<_> {
            Ok(connection.query_row(
                "SELECT COUNT(*) FROM scheduler_configurations",
                [],
                |row| row.get(0),
            )?)
        })
        .unwrap();

    assert!(result.is_err());
    assert_eq!(configuration_count, 1);
    assert_eq!(
        library.scheduling_settings().unwrap().desired_retention,
        0.9
    );
}

#[test]
fn reviews_persist_fsrs_scheduling_and_only_due_cards_are_queued() {
    let (_directory, store) = test_store();
    let library = ConceptLibrary::new(&store);
    library
        .create_concept(CreateConceptInput {
            title: "Scheduled concept".to_owned(),
            deck_ids: Vec::new(),
            tag_ids: Vec::new(),
            content: Default::default(),
            include_standard_recall: true,
            template_ids: Vec::new(),
            problem: None,
            explain: None,
            type_answer: None,
        })
        .unwrap();
    let initial_queue = library.study_queue().unwrap();
    let card = initial_queue.cards[0].clone();

    assert_eq!(initial_queue.total_cards, 1);
    assert_eq!(card.scheduling_state, SchedulingState::New);

    let first_review = library
        .record_review_at(
            RecordReviewInput {
                card_id: card.id.clone(),
                rating: ReviewRating::Good,
            },
            card.due_at,
        )
        .unwrap();

    assert_eq!(first_review.scheduling_state, SchedulingState::Review);
    assert!((first_review.scheduled_interval_days - 2.3065).abs() < 0.0001);
    assert_eq!(
        first_review.due_at,
        first_review.reviewed_at
            + (first_review.scheduled_interval_days * 86_400_000.0).round() as i64
    );

    let waiting_queue = library.study_queue_at(first_review.reviewed_at).unwrap();

    assert!(waiting_queue.cards.is_empty());
    assert_eq!(waiting_queue.next_due_at, Some(first_review.due_at));
    assert_eq!(waiting_queue.total_cards, 1);

    let changes_before_duplicate = store.changes_after(0, 100).unwrap();
    let duplicate = library.record_review_at(
        RecordReviewInput {
            card_id: card.id.clone(),
            rating: ReviewRating::Good,
        },
        first_review.reviewed_at,
    );

    assert!(matches!(duplicate, Err(LibraryError::CardNotDue { .. })));
    assert_eq!(
        store.changes_after(0, 100).unwrap(),
        changes_before_duplicate
    );

    let due_queue = library.study_queue_at(first_review.due_at).unwrap();

    assert_eq!(due_queue.cards.len(), 1);
    assert_eq!(due_queue.cards[0].id, card.id);

    let lapse = library
        .record_review_at(
            RecordReviewInput {
                card_id: card.id.clone(),
                rating: ReviewRating::Again,
            },
            first_review.due_at,
        )
        .unwrap();

    assert_eq!(lapse.scheduling_state, SchedulingState::Relearning);
    assert!(lapse.due_at > lapse.reviewed_at);

    let (review_count, lapse_count, state): (i64, i64, String) = store
        .read_result(|connection| -> DataResult<_> {
            Ok(connection.query_row(
                "SELECT review_count, lapse_count, state
                FROM card_scheduling
                WHERE card_id = ?1",
                [&card.id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )?)
        })
        .unwrap();
    let history: Vec<(i64, i64, String)> = store
        .read_result(|connection| -> DataResult<_> {
            let mut statement = connection.prepare(
                "SELECT rating, elapsed_days, scheduler_configuration_id
                FROM reviews
                WHERE card_id = ?1
                ORDER BY reviewed_at, entity_id",
            )?;
            let rows = statement.query_map([&card.id], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?))
            })?;

            Ok(rows.collect::<Result<_, _>>()?)
        })
        .unwrap();

    assert_eq!(review_count, 2);
    assert_eq!(lapse_count, 1);
    assert_eq!(state, "relearning");
    assert_eq!(
        history,
        vec![
            (3, 0, "fsrs-6.6.1-default-0.90".to_owned()),
            (1, 2, "fsrs-6.6.1-default-0.90".to_owned()),
        ]
    );

    let altered_history: DataResult<()> = store.write(|transaction| {
        transaction.execute(
            "UPDATE reviews SET rating = 4 WHERE entity_id = ?1",
            [&first_review.review_id],
        )?;

        Ok(())
    });
    let removed_history: DataResult<()> = store.write(|transaction| {
        transaction.execute(
            "DELETE FROM reviews WHERE entity_id = ?1",
            [&first_review.review_id],
        )?;

        Ok(())
    });

    assert!(altered_history.is_err());
    assert!(removed_history.is_err());
}

#[test]
fn reversing_a_review_restores_the_schedule_and_is_idempotent() {
    let (_directory, store) = test_store();
    let library = ConceptLibrary::new(&store);

    library
        .create_concept(CreateConceptInput {
            title: "Review reversal".to_owned(),
            deck_ids: Vec::new(),
            tag_ids: Vec::new(),
            content: Default::default(),
            include_standard_recall: true,
            template_ids: Vec::new(),
            problem: None,
            explain: None,
            type_answer: None,
        })
        .unwrap();

    let card = library.study_queue().unwrap().cards[0].clone();
    let review = library
        .record_review_at(
            RecordReviewInput {
                card_id: card.id.clone(),
                rating: ReviewRating::Good,
            },
            card.due_at,
        )
        .unwrap();
    let reversal = library
        .reverse_review_at(
            ReverseReviewInput {
                review_id: review.review_id.clone(),
            },
            review.reviewed_at + 1,
        )
        .unwrap();
    let restored_schedule: (
        String,
        i64,
        Option<f64>,
        Option<f64>,
        Option<i64>,
        Option<String>,
        Option<String>,
        i64,
        i64,
    ) = store
        .read_result(|connection| -> DataResult<_> {
            Ok(connection.query_row(
                "SELECT
                    state,
                    due_at,
                    stability,
                    difficulty,
                    last_reviewed_at,
                    last_review_id,
                    last_reversal_id,
                    review_count,
                    lapse_count
                FROM card_scheduling
                WHERE card_id = ?1",
                [&card.id],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                        row.get(5)?,
                        row.get(6)?,
                        row.get(7)?,
                        row.get(8)?,
                    ))
                },
            )?)
        })
        .unwrap();

    assert_eq!(reversal.review_id, review.review_id);
    assert_eq!(reversal.card_id, card.id);
    assert_eq!(
        restored_schedule,
        (
            "new".to_owned(),
            card.due_at,
            None,
            None,
            None,
            None,
            Some(reversal.reversal_id.clone()),
            0,
            0,
        )
    );

    let changes_after_reversal = store.changes_after(0, 100).unwrap();
    let repeated = library
        .reverse_review_at(
            ReverseReviewInput {
                review_id: review.review_id.clone(),
            },
            reversal.reversed_at + 1,
        )
        .unwrap();

    assert_eq!(repeated, reversal);
    assert_eq!(store.changes_after(0, 100).unwrap(), changes_after_reversal);

    let replacement = library
        .record_review_at(
            RecordReviewInput {
                card_id: card.id.clone(),
                rating: ReviewRating::Easy,
            },
            reversal.reversed_at,
        )
        .unwrap();
    let history_counts: (i64, i64, i64, Option<String>) = store
        .read_result(|connection| -> DataResult<_> {
            Ok(connection.query_row(
                "SELECT
                    (SELECT COUNT(*) FROM reviews WHERE card_id = ?1),
                    (SELECT COUNT(*) FROM review_reversals WHERE card_id = ?1),
                    review_count,
                    last_reversal_id
                FROM card_scheduling
                WHERE card_id = ?1",
                [&card.id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )?)
        })
        .unwrap();

    assert_ne!(replacement.review_id, review.review_id);
    assert_eq!(history_counts, (2, 1, 1, None));
}

#[test]
fn only_the_latest_effective_review_can_be_reversed() {
    let (_directory, store) = test_store();
    let library = ConceptLibrary::new(&store);

    library
        .create_concept(CreateConceptInput {
            title: "Review history".to_owned(),
            deck_ids: Vec::new(),
            tag_ids: Vec::new(),
            content: Default::default(),
            include_standard_recall: true,
            template_ids: Vec::new(),
            problem: None,
            explain: None,
            type_answer: None,
        })
        .unwrap();

    let card = library.study_queue().unwrap().cards[0].clone();
    let first_review = library
        .record_review_at(
            RecordReviewInput {
                card_id: card.id.clone(),
                rating: ReviewRating::Good,
            },
            card.due_at,
        )
        .unwrap();
    let second_review = library
        .record_review_at(
            RecordReviewInput {
                card_id: card.id.clone(),
                rating: ReviewRating::Again,
            },
            first_review.due_at,
        )
        .unwrap();
    let changes_before_invalid_reversal = store.changes_after(0, 100).unwrap();
    let invalid_reversal = library.reverse_review_at(
        ReverseReviewInput {
            review_id: first_review.review_id.clone(),
        },
        second_review.reviewed_at + 1,
    );

    assert!(matches!(
        invalid_reversal,
        Err(LibraryError::ReviewNotReversible)
    ));
    assert_eq!(
        store.changes_after(0, 100).unwrap(),
        changes_before_invalid_reversal
    );

    let reversal = library
        .reverse_review_at(
            ReverseReviewInput {
                review_id: second_review.review_id,
            },
            second_review.reviewed_at + 1,
        )
        .unwrap();
    let restored: (String, i64, Option<String>, Option<String>, i64, i64) = store
        .read_result(|connection| -> DataResult<_> {
            Ok(connection.query_row(
                "SELECT
                    state,
                    due_at,
                    last_review_id,
                    last_reversal_id,
                    review_count,
                    lapse_count
                FROM card_scheduling
                WHERE card_id = ?1",
                [&card.id],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                        row.get(5)?,
                    ))
                },
            )?)
        })
        .unwrap();

    assert_eq!(
        restored,
        (
            "review".to_owned(),
            first_review.due_at,
            Some(first_review.review_id),
            Some(reversal.reversal_id),
            1,
            0,
        )
    );
}

#[test]
fn a_failed_reversal_rolls_back_the_event_and_schedule() {
    let (_directory, store) = test_store();
    let library = ConceptLibrary::new(&store);

    library
        .create_concept(CreateConceptInput {
            title: "Reversal rollback".to_owned(),
            deck_ids: Vec::new(),
            tag_ids: Vec::new(),
            content: Default::default(),
            include_standard_recall: true,
            template_ids: Vec::new(),
            problem: None,
            explain: None,
            type_answer: None,
        })
        .unwrap();

    let card = library.study_queue().unwrap().cards[0].clone();
    let review = library
        .record_review_at(
            RecordReviewInput {
                card_id: card.id.clone(),
                rating: ReviewRating::Good,
            },
            card.due_at,
        )
        .unwrap();

    store
        .write(|transaction| {
            transaction.execute_batch(
                "CREATE TRIGGER force_reversal_failure
                AFTER INSERT ON review_reversals
                FOR EACH ROW
                BEGIN
                    SELECT RAISE(ABORT, 'forced reversal failure');
                END;",
            )?;

            Ok(())
        })
        .unwrap();

    let changes_before = store.changes_after(0, 100).unwrap();
    let result = library.reverse_review_at(
        ReverseReviewInput {
            review_id: review.review_id.clone(),
        },
        review.reviewed_at + 1,
    );

    assert!(result.is_err());
    assert_eq!(store.changes_after(0, 100).unwrap(), changes_before);

    let persisted: (i64, i64, Option<String>, i64) = store
        .read_result(|connection| -> DataResult<_> {
            let reversal_entities = connection.query_row(
                "SELECT COUNT(*) FROM entities WHERE kind = 'review_reversal'",
                [],
                |row| row.get(0),
            )?;
            let reversals =
                connection.query_row("SELECT COUNT(*) FROM review_reversals", [], |row| {
                    row.get(0)
                })?;
            let schedule = connection.query_row(
                "SELECT review_count, last_reversal_id
                FROM card_scheduling
                WHERE card_id = ?1",
                [&card.id],
                |row| Ok((row.get::<_, i64>(0)?, row.get::<_, Option<String>>(1)?)),
            )?;

            Ok((reversal_entities, reversals, schedule.1, schedule.0))
        })
        .unwrap();

    assert_eq!(persisted, (0, 0, None, 1));
}

#[test]
fn a_failed_new_card_enters_learning_without_counting_a_lapse() {
    let (_directory, store) = test_store();
    let library = ConceptLibrary::new(&store);

    library
        .create_concept(CreateConceptInput {
            title: "Learning concept".to_owned(),
            deck_ids: Vec::new(),
            tag_ids: Vec::new(),
            content: Default::default(),
            include_standard_recall: true,
            template_ids: Vec::new(),
            problem: None,
            explain: None,
            type_answer: None,
        })
        .unwrap();

    let card = library.study_queue().unwrap().cards[0].clone();
    let review = library
        .record_review_at(
            RecordReviewInput {
                card_id: card.id.clone(),
                rating: ReviewRating::Again,
            },
            card.due_at,
        )
        .unwrap();

    assert_eq!(review.scheduling_state, SchedulingState::Learning);
    assert!((review.scheduled_interval_days - 0.212).abs() < 0.0001);

    let lapse_count: i64 = store
        .read_result(|connection| -> DataResult<_> {
            Ok(connection.query_row(
                "SELECT lapse_count
                FROM card_scheduling
                WHERE card_id = ?1",
                [&card.id],
                |row| row.get(0),
            )?)
        })
        .unwrap();

    assert_eq!(lapse_count, 0);
}

#[test]
fn a_failed_review_write_rolls_back_the_event_and_schedule() {
    let (_directory, store) = test_store();
    let library = ConceptLibrary::new(&store);
    let concept = library
        .create_concept(CreateConceptInput {
            title: "Rollback".to_owned(),
            deck_ids: Vec::new(),
            tag_ids: Vec::new(),
            content: Default::default(),
            include_standard_recall: true,
            template_ids: Vec::new(),
            problem: None,
            explain: None,
            type_answer: None,
        })
        .unwrap();
    let card = library.study_queue().unwrap().cards[0].clone();

    store
        .write(|transaction| {
            transaction.execute_batch(
                "CREATE TRIGGER force_review_failure
                AFTER INSERT ON reviews
                FOR EACH ROW
                BEGIN
                    SELECT RAISE(ABORT, 'forced review failure');
                END;",
            )?;

            Ok(())
        })
        .unwrap();

    let changes_before = store.changes_after(0, 100).unwrap();
    let result = library.record_review_at(
        RecordReviewInput {
            card_id: card.id.clone(),
            rating: ReviewRating::Good,
        },
        card.due_at,
    );

    assert!(result.is_err());
    assert_eq!(store.changes_after(0, 100).unwrap(), changes_before);

    let (review_entities, reviews, review_count): (i64, i64, i64) = store
        .read_result(|connection| -> DataResult<_> {
            let review_entities = connection.query_row(
                "SELECT COUNT(*) FROM entities WHERE kind = 'review'",
                [],
                |row| row.get(0),
            )?;
            let reviews =
                connection.query_row("SELECT COUNT(*) FROM reviews", [], |row| row.get(0))?;
            let review_count = connection.query_row(
                "SELECT review_count
                FROM card_scheduling
                WHERE card_id = ?1",
                [&concept.cards[0].id],
                |row| row.get(0),
            )?;

            Ok((review_entities, reviews, review_count))
        })
        .unwrap();

    assert_eq!((review_entities, reviews, review_count), (0, 0, 0));
}

#[test]
fn concept_rows_cannot_bypass_change_tracking_or_tombstones() {
    let (_directory, store) = test_store();
    let library = ConceptLibrary::new(&store);
    let concept = library
        .create_concept(CreateConceptInput {
            title: "Protected".to_owned(),
            deck_ids: Vec::new(),
            tag_ids: Vec::new(),
            content: Default::default(),
            include_standard_recall: true,
            template_ids: Vec::new(),
            problem: None,
            explain: None,
            type_answer: None,
        })
        .unwrap();

    let bypassed_update: DataResult<()> = store.write(|transaction| {
        transaction.execute(
            "UPDATE concepts SET title = 'Untracked' WHERE entity_id = ?1",
            [&concept.id],
        )?;

        Ok(())
    });
    let hard_delete: DataResult<()> = store.write(|transaction| {
        transaction.execute("DELETE FROM concepts WHERE entity_id = ?1", [&concept.id])?;

        Ok(())
    });

    assert!(bypassed_update.is_err());
    assert!(hard_delete.is_err());
    assert_eq!(library.concept(&concept.id).unwrap().title, "Protected");
}
