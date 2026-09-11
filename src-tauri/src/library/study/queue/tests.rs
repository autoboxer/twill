use std::collections::BTreeSet;

use serde_json::json;

use super::ID_BATCH_SIZE;
use crate::data::LocalDataStore;
use crate::library::{
    ConceptLibrary, CreateTemplateInput, LibraryError, LibraryQuery, RecordReviewInput,
    RetrievalFormKind, ReviewRating, SchedulingState, StudyQuery, TemplateLibrary,
};

#[test]
fn queue_shares_documents_and_templates_without_losing_form_configuration() {
    let directory = tempfile::tempdir().unwrap();
    let store = LocalDataStore::open(directory.path()).unwrap();
    let library = ConceptLibrary::new(&store);
    let template = TemplateLibrary::new(&store)
        .create_template(CreateTemplateInput {
            name: "Shared template".into(),
            content: Default::default(),
        })
        .unwrap();
    let prompt = json!({ "type": "doc", "content": [{ "type": "paragraph", "content": [
        { "type": "text", "text": "Shared content. ".repeat(500) }
    ] }] });
    let first = library
        .create_concept(
            serde_json::from_value(json!({
                "title": "First", "templateIds": [template.id],
                "content": { "schemaVersion": 1, "prompt": prompt, "answer": prompt },
                "typeAnswer": { "acceptedAnswers": ["Exact answer"] },
                "explain": { "focus": "why", "keyPoints": ["Key point"] },
                "problem": { "checkpoints": ["Checkpoint"] }
            }))
            .unwrap(),
        )
        .unwrap();
    let second = library
        .create_concept(
            serde_json::from_value(json!({
                "title": "Second", "templateIds": [template.id]
            }))
            .unwrap(),
        )
        .unwrap();
    let queue = library.study_queue().unwrap();

    assert_eq!(queue.cards.len(), first.cards.len() + second.cards.len());
    assert_eq!(queue.concepts.len(), 2);
    assert_eq!(queue.templates.len(), 1);
    assert_eq!(queue.templates[0].id, template.id);
    assert_eq!(
        queue
            .concepts
            .iter()
            .find(|concept| concept.id == first.id)
            .unwrap()
            .content,
        first.content
    );

    for card in &first.cards {
        let loaded = queue
            .cards
            .iter()
            .find(|candidate| candidate.id == card.id)
            .unwrap();
        assert_eq!(loaded.concept_id, first.id);
        assert_eq!(
            loaded.template_id,
            card.template.as_ref().map(|template| template.id.clone())
        );
        assert_eq!(loaded.type_answer, card.type_answer);
        assert_eq!(loaded.explain, card.explain);
        assert_eq!(loaded.problem, card.problem);
    }

    let wire = serde_json::to_value(&queue).unwrap();
    assert!(wire["cards"]
        .as_array()
        .unwrap()
        .iter()
        .all(|card| card.get("content").is_none() && card.get("template").is_none()));
    let shared_bytes = serde_json::to_vec(&queue.concepts).unwrap().len();
    let old_document_bytes = serde_json::to_vec(&first.content).unwrap().len() * first.cards.len();
    assert!(shared_bytes * 3 < old_document_bytes);

    for card in &first.cards {
        library
            .record_review(RecordReviewInput {
                card_id: card.id.clone(),
                rating: ReviewRating::Good,
            })
            .unwrap();
    }
    let remaining = library.study_queue().unwrap();
    assert_eq!(remaining.concepts.len(), 1);
    assert_eq!(remaining.concepts[0].id, second.id);
    assert_eq!(remaining.templates.len(), 1);

    library.set_concept_archived(&second.id, true).unwrap();
    let empty = library.study_queue().unwrap();
    assert!(empty.cards.is_empty());
    assert!(empty.concepts.is_empty());
    assert!(empty.templates.is_empty());
    assert!(empty.next_due_at.is_some());
    assert_eq!(empty.total_cards, first.cards.len() as i64);
}

#[test]
fn batched_content_reads_keep_complete_membership_and_mixed_order() {
    let directory = tempfile::tempdir().unwrap();
    let store = LocalDataStore::open(directory.path()).unwrap();
    let library = ConceptLibrary::new(&store);
    let tag = library.create_tag("Shared tag".into()).unwrap();

    for index in 0..=ID_BATCH_SIZE {
        library
            .create_concept(
                serde_json::from_value(json!({
                    "title": format!("Concept {index}"), "tagIds": [tag.id],
                }))
                .unwrap(),
            )
            .unwrap();
    }

    let ordered = library.study_queue().unwrap();
    assert_eq!(ordered.cards.len(), ID_BATCH_SIZE + 1);
    assert_eq!(ordered.concepts.len(), ID_BATCH_SIZE + 1);
    assert!(ordered
        .cards
        .windows(2)
        .all(|pair| { (pair[0].due_at, &pair[0].id) <= (pair[1].due_at, &pair[1].id) }));
    library.set_mixed_practice_enabled(true).unwrap();
    let mixed = library.study_queue().unwrap();
    let card_ids = |queue: &crate::library::StudyQueue| {
        queue
            .cards
            .iter()
            .map(|card| &card.id)
            .cloned()
            .collect::<BTreeSet<_>>()
    };

    assert_eq!(card_ids(&mixed), card_ids(&ordered));
    assert_eq!(mixed.concepts, ordered.concepts);
    assert_eq!(mixed, library.study_queue().unwrap());
    assert!(mixed.mixed_practice_enabled);
}

#[test]
fn focused_filters_match_one_due_card_and_keep_scoped_counts() {
    let directory = tempfile::tempdir().unwrap();
    let store = LocalDataStore::open(directory.path()).unwrap();
    let library = ConceptLibrary::new(&store);
    let deck = library.create_deck("Deck".into()).unwrap();
    let tag = library.create_tag("Tag".into()).unwrap();
    let concept = library
        .create_concept(
            serde_json::from_value(json!({
                "title": "Café biology",
                "deckIds": [deck.id],
                "tagIds": [tag.id],
                "typeAnswer": { "acceptedAnswers": ["Mitochondrion"] }
            }))
            .unwrap(),
        )
        .unwrap();
    let recall = concept
        .cards
        .iter()
        .find(|card| card.retrieval_kind == RetrievalFormKind::Recall)
        .unwrap();
    library
        .record_review(RecordReviewInput {
            card_id: recall.id.clone(),
            rating: ReviewRating::Good,
        })
        .unwrap();
    let input = StudyQuery {
        query: "cafe mito".into(),
        deck_id: Some(deck.id.clone()),
        tag_id: Some(tag.id.clone()),
        card_type: Some(RetrievalFormKind::TypeAnswer),
        state: Some(SchedulingState::New),
        ..Default::default()
    };
    let queue = library.selected_study_queue(input.clone()).unwrap();

    assert_eq!(queue.cards.len(), 1);
    assert_eq!(
        (queue.total_cards, queue.due_cards, queue.next_due_at),
        (1, 1, None)
    );
    assert_eq!(queue.cards[0].retrieval_kind, RetrievalFormKind::TypeAnswer);
    assert!(!queue.cards[0].pretest_eligible);

    let mismatched = library
        .selected_study_queue(StudyQuery {
            state: Some(SchedulingState::Review),
            ..input.clone()
        })
        .unwrap();
    assert_eq!(mismatched.total_cards, 0);
    assert!(mismatched.cards.is_empty());

    let future = library
        .selected_study_queue(StudyQuery {
            card_type: Some(RetrievalFormKind::Recall),
            state: Some(SchedulingState::Review),
            ..input.clone()
        })
        .unwrap();
    assert_eq!((future.total_cards, future.due_cards), (1, 0));
    assert!(future.cards.is_empty());
    assert!(future.next_due_at.is_some());

    library.set_concept_archived(&concept.id, true).unwrap();
    assert_eq!(
        library
            .selected_study_queue(input.clone())
            .unwrap()
            .total_cards,
        0
    );
    library.set_concept_archived(&concept.id, false).unwrap();
    library.delete_tag(&tag.id).unwrap();
    assert_eq!(library.selected_study_queue(input).unwrap().total_cards, 0);
}

#[test]
fn sessions_use_all_search_results_and_limit_before_mixing() {
    let directory = tempfile::tempdir().unwrap();
    let store = LocalDataStore::open(directory.path()).unwrap();
    let library = ConceptLibrary::new(&store);
    let deck = library.create_deck("Selected".into()).unwrap();
    library
        .create_concept(serde_json::from_value(json!({ "title": "Outside scope" })).unwrap())
        .unwrap();

    for index in 0..55 {
        library
            .create_concept(
                serde_json::from_value(json!({
                    "title": format!("Session match {index}"),
                    "deckIds": [deck.id]
                }))
                .unwrap(),
            )
            .unwrap();
    }

    let page = library
        .search(LibraryQuery {
            query: "Session mat".into(),
            deck_id: Some(deck.id.clone()),
            page: 2,
            ..Default::default()
        })
        .unwrap();
    assert_eq!((page.total_count, page.concepts.len()), (55, 5));

    let input = StudyQuery {
        query: "Session mat".into(),
        deck_id: Some(deck.id),
        ..Default::default()
    };
    let full = library.selected_study_queue(input.clone()).unwrap();
    assert_eq!(full.cards.len(), 55);
    let selected = full
        .cards
        .iter()
        .take(3)
        .map(|card| card.id.clone())
        .collect::<BTreeSet<_>>();
    library.set_mixed_practice_enabled(true).unwrap();
    let limited = library
        .selected_study_queue(StudyQuery {
            card_limit: Some(3),
            ..input.clone()
        })
        .unwrap();

    assert_eq!(
        limited
            .cards
            .iter()
            .map(|card| card.id.clone())
            .collect::<BTreeSet<_>>(),
        selected
    );
    assert_eq!(
        (
            limited.total_cards,
            limited.due_cards,
            limited.concepts.len()
        ),
        (55, 55, 3)
    );
    assert_eq!(
        limited,
        library
            .selected_study_queue(StudyQuery {
                card_limit: Some(3),
                ..input.clone()
            })
            .unwrap()
    );
    assert_eq!(
        library
            .selected_study_queue(StudyQuery {
                card_limit: Some(u32::MAX),
                ..input
            })
            .unwrap()
            .cards
            .len(),
        55
    );
}

#[test]
fn session_inputs_reject_invalid_limits_and_do_not_accept_library_pagination() {
    let directory = tempfile::tempdir().unwrap();
    let store = LocalDataStore::open(directory.path()).unwrap();
    let library = ConceptLibrary::new(&store);

    assert!(matches!(
        library.selected_study_queue(StudyQuery {
            card_limit: Some(0),
            ..Default::default()
        }),
        Err(LibraryError::InvalidContent {
            field: "Card limit",
            ..
        })
    ));
    assert!(matches!(
        library.selected_study_queue(StudyQuery {
            query: "é".repeat(251),
            ..Default::default()
        }),
        Err(LibraryError::ValueTooLong { .. })
    ));

    for input in [
        json!({"cardLimit": -1}),
        json!({"cardLimit": 1.5}),
        json!({"state": "due"}),
        json!({"page": 2}),
        json!({"includeArchived": true}),
    ] {
        assert!(serde_json::from_value::<StudyQuery>(input).is_err());
    }

    for query in ["", "   ", "\" OR *", "é"] {
        let queue = library
            .selected_study_queue(StudyQuery {
                query: query.into(),
                ..Default::default()
            })
            .unwrap();
        assert!(queue.cards.is_empty());
        assert_eq!((queue.total_cards, queue.due_cards), (0, 0));
    }
}
