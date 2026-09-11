use rusqlite::params;
use serde_json::{json, Value};

use super::*;
use crate::data::{DataResult, LocalDataStore};
use crate::library::{CreateConceptInput, LibraryError, RecordReviewInput, ReviewRating, UpdateConceptInput};

fn input(title: &str) -> CreateConceptInput {
    serde_json::from_value(json!({ "title": title })).unwrap()
}

fn document(text: &str) -> Value {
    json!({ "type": "doc", "content": [{ "type": "paragraph", "content": [
        { "type": "text", "text": text }
    ] }] })
}

fn search(library: &ConceptLibrary<'_>, query: &str) -> LibraryPage {
    library
        .search(LibraryQuery {
            query: query.to_owned(),
            ..Default::default()
        })
        .unwrap()
}

#[test]
fn search_indexes_readable_content_and_treats_query_syntax_as_text() {
    let directory = tempfile::tempdir().unwrap();
    let store = LocalDataStore::open(directory.path()).unwrap();
    let library = ConceptLibrary::new(&store);
    let mut create = input("Café biology");
    create.content.prompt = json!({ "type": "doc", "content": [{ "type": "paragraph", "content": [
        { "type": "text", "text": "photo" },
        { "type": "text", "text": "synthesis", "marks": [{ "type": "bold" }] },
        { "type": "hardBreak" },
        { "type": "text", "text": "研究 mitochondria OR NEAR" }
    ] }] });
    create.content.answer = document("chlorophyll absorbs sunlight");
    create.content.feedback.explanation = document("pigment");
    create.content.feedback.common_mistakes = document("chloroplast");
    create.type_answer =
        Some(serde_json::from_value(json!({ "acceptedAnswers": ["ATP"] })).unwrap());
    create.explain =
        Some(serde_json::from_value(json!({ "focus": "why", "keyPoints": ["photons"] })).unwrap());
    create.problem =
        Some(serde_json::from_value(json!({ "checkpoints": ["balance electrons"] })).unwrap());
    let concept = library.create_concept(create).unwrap();

    for query in [
        "cafe",
        "CAFE",
        "cafe\u{301}",
        "photosyn",
        "cafe sunlight",
        "研究",
        "chlorophyll",
        "pigment",
        "chloroplast",
        "atp",
        "photons",
        "electrons",
        "OR",
        "NEAR",
        "\"mitochondria\"",
        "mitochondria*",
        "mito\0",
    ] {
        assert_eq!(
            search(&library, query).concepts[0].id,
            concept.id,
            "{query}"
        );
    }

    for query in [
        "photosynthesis missing",
        "schemaVersion",
        "paragraph",
        "acceptedAnswers",
        "bold",
        &concept.id,
        "\"",
        "***",
        "() : -",
        "sunlight OR nonexistent",
    ] {
        assert_eq!(search(&library, query).total_count, 0, "{query}");
    }

    assert!(matches!(
        library.search(LibraryQuery {
            query: "x".repeat(251),
            ..Default::default()
        }),
        Err(LibraryError::ValueTooLong { .. })
    ));
    assert_eq!(search(&library, " \n ").total_count, 1);
}

#[test]
fn rich_text_projection_keeps_inline_words_and_excludes_metadata() {
    let mut text = String::new();
    append_document_text(
        &json!({ "type": "doc", "content": [
        { "type": "paragraph", "content": [{ "type": "text", "text": "alpha" }] },
        { "type": "codeBlock", "content": [{ "type": "text", "text": "beta()" }] },
        { "type": "inlineMath", "attrs": { "latex": "gamma = 3" } },
        { "type": "mediaImage", "attrs": { "mediaId": "secret-id", "alt": "delta diagram", "title": "epsilon" } }
    ] }),
        &mut text,
    );
    assert_eq!(
        text.split_whitespace().collect::<Vec<_>>(),
        vec!["alpha", "beta()", "gamma", "=", "3", "delta", "diagram", "epsilon"]
    );
}

#[test]
fn search_projection_updates_atomically_and_removes_deleted_content() {
    let directory = tempfile::tempdir().unwrap();
    let store = LocalDataStore::open(directory.path()).unwrap();
    let library = ConceptLibrary::new(&store);
    let mut create = input("Original title");
    create.type_answer =
        Some(serde_json::from_value(json!({ "acceptedAnswers": ["obsolete"] })).unwrap());
    let concept = library.create_concept(create).unwrap();
    let update: UpdateConceptInput = serde_json::from_value(json!({
        "id": concept.id, "title": "Replacement title", "content": {
            "schemaVersion": 1, "prompt": document("newword"), "answer": document("newanswer")
        }
    }))
    .unwrap();

    let result: LibraryResult<()> = store.write_result(|transaction| {
        super::super::concepts::update_concept(transaction, update.clone())?;
        Err(LibraryError::ConceptNotFound("force rollback".into()))
    });
    assert!(result.is_err());
    assert_eq!(search(&library, "obsolete").total_count, 1);
    assert_eq!(search(&library, "newword").total_count, 0);

    library.update_concept(update).unwrap();
    assert_eq!(search(&library, "obsolete").total_count, 0);
    assert_eq!(search(&library, "Original").total_count, 0);
    assert_eq!(search(&library, "newword newanswer").total_count, 1);

    library.set_concept_archived(&concept.id, true).unwrap();
    assert_eq!(search(&library, "newword").total_count, 0);
    let archived = library
        .search(LibraryQuery {
            query: "newword".into(),
            include_archived: true,
            ..Default::default()
        })
        .unwrap();
    assert_eq!(archived.total_count, 1);
    assert_eq!(archived.archived_count, 1);
    library.set_concept_archived(&concept.id, false).unwrap();
    assert_eq!(search(&library, "newword").total_count, 1);
    library.delete_concept(&concept.id).unwrap();
    assert_eq!(search(&library, "newword").total_count, 0);
    store
        .read_result(|connection| -> DataResult<()> {
            let count: i64 =
                connection
                    .query_row("SELECT COUNT(*) FROM concept_search", [], |row| row.get(0))?;
            assert_eq!(count, 0);
            Ok(())
        })
        .unwrap();

    library
        .create_concept(input("Durable search index"))
        .unwrap();
    drop(store);
    let reopened = LocalDataStore::open(directory.path()).unwrap();
    assert_eq!(
        search(&ConceptLibrary::new(&reopened), "Replacement").total_count,
        0
    );
    assert_eq!(
        search(&ConceptLibrary::new(&reopened), "Durable").total_count,
        1
    );
}

#[test]
fn browsing_and_search_are_bounded_stable_and_filter_before_pagination() {
    let directory = tempfile::tempdir().unwrap();
    let store = LocalDataStore::open(directory.path()).unwrap();
    let library = ConceptLibrary::new(&store);
    let deck = library.create_deck("Deck".into()).unwrap();
    let tag = library.create_tag("Tag".into()).unwrap();
    for index in 0..105 {
        let mut create = input(&format!("Concept {:03}", index / 2));
        create.content.answer = document("sharedword");
        if index >= 50 {
            create.deck_ids.push(deck.id.clone());
            create.type_answer =
                Some(serde_json::from_value(json!({ "acceptedAnswers": ["sharedword"] })).unwrap());
        }
        if index >= 100 {
            create.tag_ids.push(tag.id.clone());
        }
        library.create_concept(create).unwrap();
    }

    let first = search(&library, "");
    let second = library
        .search(LibraryQuery {
            page: 2,
            ..Default::default()
        })
        .unwrap();
    let last = library
        .search(LibraryQuery {
            page: u32::MAX,
            ..Default::default()
        })
        .unwrap();
    assert_eq!(
        (
            first.concepts.len(),
            second.concepts.len(),
            last.concepts.len()
        ),
        (50, 50, 5)
    );
    assert_eq!(
        (
            first.total_count,
            first.concept_count,
            last.page,
            last.page_size
        ),
        (105, 105, 3, 50)
    );
    let ids = first
        .concepts
        .iter()
        .chain(&second.concepts)
        .chain(&last.concepts)
        .map(|concept| concept.id.clone())
        .collect::<std::collections::HashSet<_>>();
    assert_eq!(ids.len(), 105);
    assert_eq!(search(&library, "").concepts, first.concepts);
    assert_eq!(search(&library, "sharedword").concepts.len(), 50);
    assert_eq!(search(&library, "sharedword").total_count, 105);
    let scoped = library
        .search(LibraryQuery {
            query: "sharedword".into(),
            deck_id: Some(deck.id.clone()),
            tag_id: Some(tag.id.clone()),
            card_type: Some(RetrievalFormKind::TypeAnswer),
            state: Some(LibraryCardState::New),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(scoped.total_count, 5);
    assert!(scoped
        .concepts
        .iter()
        .all(|concept| concept.decks[0].id == deck.id && concept.tags[0].id == tag.id));
    let filtered_page = library
        .search(LibraryQuery {
            card_type: Some(RetrievalFormKind::TypeAnswer),
            state: Some(LibraryCardState::New),
            page: 2,
            ..Default::default()
        })
        .unwrap();
    assert_eq!(
        (filtered_page.total_count, filtered_page.concepts.len()),
        (55, 5)
    );
    for concept in &last.concepts {
        library.delete_concept(&concept.id).unwrap();
    }
    let clamped = library
        .search(LibraryQuery {
            page: 3,
            ..Default::default()
        })
        .unwrap();
    assert_eq!(
        (clamped.page, clamped.total_count, clamped.concepts.len()),
        (2, 100, 50)
    );
    library.delete_deck(&deck.id).unwrap();
    assert_eq!(
        library
            .search(LibraryQuery {
                deck_id: Some(deck.id),
                ..Default::default()
            })
            .unwrap()
            .total_count,
        0
    );

    store
        .read_result(|connection| -> DataResult<()> {
            let mut statement = connection.prepare(
                "EXPLAIN QUERY PLAN SELECT rowid FROM concept_search WHERE concept_search MATCH ?1",
            )?;
            let plan = statement
                .query_map(params!["sharedword"], |row| row.get::<_, String>(3))?
                .collect::<Result<Vec<_>, _>>()?
                .join(" ");
            assert!(plan.contains("VIRTUAL TABLE INDEX"), "{plan}");
            Ok(())
        })
        .unwrap();
}

#[test]
fn title_matches_receive_more_weight_than_body_matches() {
    let directory = tempfile::tempdir().unwrap();
    let store = LocalDataStore::open(directory.path()).unwrap();
    let library = ConceptLibrary::new(&store);
    let mut body_match = input("A body match");
    body_match.content.answer = document("mitochondria");
    library.create_concept(body_match).unwrap();
    let title_match = library.create_concept(input("Mitochondria")).unwrap();
    assert_eq!(search(&library, "mito").concepts[0].id, title_match.id);
}

#[test]
fn card_filters_match_one_active_form_and_due_excludes_archived_and_future_cards() {
    let directory = tempfile::tempdir().unwrap();
    let store = LocalDataStore::open(directory.path()).unwrap();
    let library = ConceptLibrary::new(&store);
    let mut create = input("Mixed forms");
    create.type_answer =
        Some(serde_json::from_value(json!({ "acceptedAnswers": ["textneedle"] })).unwrap());
    let concept = library.create_concept(create).unwrap();
    let recall = concept
        .cards
        .iter()
        .find(|card| card.retrieval_kind == RetrievalFormKind::Recall)
        .unwrap();
    let typed = concept
        .cards
        .iter()
        .find(|card| card.retrieval_kind == RetrievalFormKind::TypeAnswer)
        .unwrap();

    library
        .record_review_at(
            RecordReviewInput {
                card_id: recall.id.clone(),
                rating: ReviewRating::Good,
            },
            recall.due_at,
        )
        .unwrap();

    let query = LibraryQuery {
        card_type: Some(RetrievalFormKind::TypeAnswer),
        state: Some(LibraryCardState::Review),
        ..Default::default()
    };
    assert_eq!(library.search(query.clone()).unwrap().total_count, 0);
    assert_eq!(
        library
            .search(LibraryQuery {
                state: Some(LibraryCardState::Due),
                card_type: Some(RetrievalFormKind::Recall),
                ..query.clone()
            })
            .unwrap()
            .total_count,
        0
    );

    let typed_result = library
        .search(LibraryQuery {
            state: Some(LibraryCardState::New),
            ..query.clone()
        })
        .unwrap();
    assert_eq!(
        typed_result.concepts[0].matching_form.as_ref().unwrap().id,
        typed.id
    );

    // Text remains concept-wide even when a different form satisfies the card predicates
    let due = library
        .search(LibraryQuery {
            query: "textneedle".into(),
            card_type: Some(RetrievalFormKind::Recall),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(
        due.concepts[0].matching_form.as_ref().unwrap().id,
        recall.id
    );
    assert!(due.concepts[0]
        .excerpt
        .as_ref()
        .unwrap()
        .contains("textneedle"));
    assert!(search(&library, "").concepts[0].matching_form.is_none());

    let due_query = LibraryQuery {
        state: Some(LibraryCardState::Due),
        include_archived: true,
        ..Default::default()
    };
    assert_eq!(library.search(due_query.clone()).unwrap().total_count, 1);
    library.set_concept_archived(&concept.id, true).unwrap();
    assert_eq!(library.search(due_query.clone()).unwrap().total_count, 0);
    library.set_concept_archived(&concept.id, false).unwrap();

    let mut due_at = typed.due_at;

    for (state, rating) in [
        (LibraryCardState::Learning, ReviewRating::Again),
        (LibraryCardState::Review, ReviewRating::Good),
        (LibraryCardState::Relearning, ReviewRating::Again),
    ] {
        due_at = library
            .record_review_at(
                RecordReviewInput {
                    card_id: typed.id.clone(),
                    rating,
                },
                due_at,
            )
            .unwrap()
            .due_at;
        assert_eq!(
            library
                .search(LibraryQuery {
                    state: Some(state),
                    ..query.clone()
                })
                .unwrap()
                .total_count,
            1
        );
    }

    library.set_concept_archived(&concept.id, true).unwrap();
    assert_eq!(library.search(due_query.clone()).unwrap().total_count, 0);
    assert_eq!(
        library
            .search(LibraryQuery {
                include_archived: true,
                card_type: Some(RetrievalFormKind::Recall),
                ..Default::default()
            })
            .unwrap()
            .total_count,
        1
    );

    library.set_concept_archived(&concept.id, false).unwrap();
    library
        .update_concept(
            serde_json::from_value(json!({
                "id": concept.id, "title": concept.title, "includeStandardRecall": false,
                "typeAnswer": { "acceptedAnswers": ["textneedle"] }
            }))
            .unwrap(),
        )
        .unwrap();
    assert_eq!(library.search(due_query).unwrap().total_count, 0);
    assert_eq!(
        library
            .search(LibraryQuery {
                card_type: Some(RetrievalFormKind::Recall),
                ..Default::default()
            })
            .unwrap()
            .total_count,
        0
    );
}

#[test]
fn sorting_and_excerpts_are_deterministic_and_bounded() {
    let directory = tempfile::tempdir().unwrap();
    let store = LocalDataStore::open(directory.path()).unwrap();
    let library = ConceptLibrary::new(&store);
    let mut create = input("A body match");
    create.content.answer = document(&format!("{} <needle>", "界".repeat(1000)));
    let body = library.create_concept(create).unwrap();
    let title = library.create_concept(input("Needle")).unwrap();
    store
        .write_result(|transaction| -> DataResult<()> {
            transaction.touch_entity(&body.id)?;
            transaction.execute(
                "UPDATE entities SET updated_at = updated_at + 10000, revision = revision + 1 WHERE id = ?1",
                [&body.id],
            )?;
            Ok(())
        })
        .unwrap();

    for sort in [LibrarySort::Title, LibrarySort::Updated] {
        let result = library
            .search(LibraryQuery {
                query: "needle".into(),
                sort,
                ..Default::default()
            })
            .unwrap();
        assert_eq!(result.concepts[0].id, body.id);
        assert_eq!(result.concepts[1].id, title.id);
    }

    let excerpt = search(&library, "界").concepts[0].excerpt.clone().unwrap();
    assert_eq!(excerpt.chars().count(), MAXIMUM_EXCERPT_LENGTH + 1);
    assert!(excerpt.ends_with('…'));
    assert!(search(&library, "")
        .concepts
        .iter()
        .all(|concept| concept.excerpt.is_none()));
    assert!(serde_json::from_value::<LibraryQuery>(json!({ "state": "unknown" })).is_err());
    assert!(serde_json::from_value::<LibraryQuery>(json!({ "sort": "updated_at DESC" })).is_err());
}
