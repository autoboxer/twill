use serde_json::json;
use tempfile::tempdir;

use super::*;
use crate::data::LocalDataStore;
use crate::library::study::record_review;
use crate::library::{ConceptLibrary, RecordReviewInput, ReverseReviewInput, StudyQuery};

#[test]
fn review_history_preserves_unknown_context_and_undone_grades() {
    let directory = tempdir().unwrap();
    let store = LocalDataStore::open(directory.path()).unwrap();
    let library = ConceptLibrary::new(&store);
    let concept = library
        .create_concept(
            serde_json::from_value(json!({
                "title": "Unknown context"
            }))
            .unwrap(),
        )
        .unwrap();
    let card_id = &concept.cards[0].id;

    assert!(library.review_history(card_id).unwrap().reviews.is_empty());

    let input: RecordReviewInput = serde_json::from_value(json!({
        "cardId": card_id, "rating": "good"
    }))
    .unwrap();
    let review = library.record_review(input).unwrap();
    let history = library.review_history(card_id).unwrap();

    assert_eq!(history.reviews[0].id, review.review_id);
    assert_eq!(history.reviews[0].rating, ReviewRating::Good);
    assert_eq!(history.reviews[0].assisted, None);
    assert_eq!(history.reviews[0].current_content, None);
    assert_eq!(history.reviews[0].reversed_at, None);
    assert!(!history.has_more);

    let reversal = library
        .reverse_review(ReverseReviewInput {
            review_id: review.review_id,
        })
        .unwrap();
    let history = library.review_history(card_id).unwrap();

    assert_eq!(history.reviews[0].reversed_at, Some(reversal.reversed_at));
    assert_eq!(
        library.concept(&concept.id).unwrap().cards[0].review_count,
        0
    );
    assert!(matches!(
        library.review_history("missing"),
        Err(LibraryError::CardNotFound(_))
    ));

    drop(library);
    drop(store);

    let reopened = LocalDataStore::open(directory.path()).unwrap();
    assert_eq!(
        ConceptLibrary::new(&reopened)
            .review_history(card_id)
            .unwrap(),
        history
    );
}

#[test]
fn review_versions_reject_stale_content_and_identify_earlier_history() {
    let directory = tempdir().unwrap();
    let store = LocalDataStore::open(directory.path()).unwrap();
    let library = ConceptLibrary::new(&store);
    let concept = library
        .create_concept(
            serde_json::from_value(json!({
                "title": "Original question"
            }))
            .unwrap(),
        )
        .unwrap();
    let card_id = &concept.cards[0].id;
    let version = library.study_queue().unwrap().cards[0]
        .content_version
        .clone();
    let changed = library
        .update_concept(
            serde_json::from_value(json!({
                "id": concept.id, "title": "Updated question"
            }))
            .unwrap(),
        )
        .unwrap();
    let input = |version| RecordReviewInput {
        exposure_id: None,
        card_id: card_id.clone(),
        rating: ReviewRating::Hard,
        assisted: Some(true),
        content_version: Some(version),
    };

    assert!(matches!(
        library.record_review(input(version)),
        Err(LibraryError::StudyContentChanged)
    ));
    assert!(library.review_history(card_id).unwrap().reviews.is_empty());
    assert_eq!(library.concept(&concept.id).unwrap().cards, changed.cards);

    let version = library.study_queue().unwrap().cards[0]
        .content_version
        .clone();
    library.record_review(input(version)).unwrap();

    let history = library.review_history(card_id).unwrap();
    assert_eq!(history.reviews[0].assisted, Some(true));
    assert_eq!(history.reviews[0].current_content, Some(true));

    library
        .update_concept(
            serde_json::from_value(json!({
                "id": concept.id, "title": "A further edit"
            }))
            .unwrap(),
        )
        .unwrap();

    assert_eq!(
        library.review_history(card_id).unwrap().reviews[0].current_content,
        Some(false)
    );
    assert_eq!(
        library.concept(&concept.id).unwrap().cards[0].review_count,
        1
    );
}

#[test]
fn template_edits_invalidate_queued_review_versions() {
    let directory = tempdir().unwrap();
    let store = LocalDataStore::open(directory.path()).unwrap();
    let templates = crate::library::TemplateLibrary::new(&store);
    let template = templates
        .create_template(
            serde_json::from_value(json!({
                "name": "Versioned template", "content": crate::library::TemplateContent::default()
            }))
            .unwrap(),
        )
        .unwrap();
    let library = ConceptLibrary::new(&store);
    let concept = library
        .create_concept(
            serde_json::from_value(json!({
                "title": "Template example", "includeStandardRecall": false,
                "templateIds": [template.id]
            }))
            .unwrap(),
        )
        .unwrap();
    let card = library
        .selected_study_queue(StudyQuery::default())
        .unwrap()
        .cards
        .remove(0);

    templates
        .update_template(
            serde_json::from_value(json!({
                "id": template.id, "name": "Renamed template", "content": template.content
            }))
            .unwrap(),
        )
        .unwrap();

    assert!(matches!(
        library.record_review(RecordReviewInput {
            exposure_id: None,
            card_id: card.id.clone(),
            rating: ReviewRating::Good,
            assisted: Some(false),
            content_version: Some(card.content_version),
        }),
        Err(LibraryError::StudyContentChanged)
    ));
    assert_eq!(
        library.concept(&concept.id).unwrap().cards[0].review_count,
        0
    );
}

#[test]
fn history_is_bounded_ordered_and_separate_for_each_card() {
    let directory = tempdir().unwrap();
    let store = LocalDataStore::open(directory.path()).unwrap();
    let library = ConceptLibrary::new(&store);
    let concept = library
        .create_concept(
            serde_json::from_value(json!({
                "title": "Bounded history", "typeAnswer": { "acceptedAnswers": ["TCP"] }
            }))
            .unwrap(),
        )
        .unwrap();
    let card_id = &concept.cards[0].id;
    let mut now = concept.cards[0].due_at;
    let version = store
        .read_result(|connection| query_content_version(connection, card_id))
        .unwrap();
    let mut ids = Vec::new();

    for index in 0..12 {
        let review = store
            .write_result(|transaction| {
                record_review(
                    transaction,
                    card_id,
                    ReviewRating::Again,
                    Some(index % 2 == 0),
                    Some(&version),
                    None,
                    now,
                )
            })
            .unwrap();

        now = review.due_at;
        ids.push(review.review_id);
    }

    let history = library.review_history(card_id).unwrap();
    assert_eq!(history.reviews.len(), 10);
    assert!(history.has_more);
    assert_eq!(
        history
            .reviews
            .iter()
            .map(|review| &review.id)
            .collect::<Vec<_>>(),
        ids.iter().rev().take(10).collect::<Vec<_>>()
    );
    assert!(history
        .reviews
        .iter()
        .all(|review| review.current_content == Some(true)));
    assert!(library
        .review_history(&concept.cards[1].id)
        .unwrap()
        .reviews
        .is_empty());
}
