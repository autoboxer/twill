use serde_json::json;
use tempfile::tempdir;

use crate::data::{current_timestamp, LocalDataStore};
use crate::library::{ConceptLibrary, LibraryError, RecordExposureInput, ReviewRating};

use super::super::{
    history::query_content_version, record_review, reverse_review, MILLISECONDS_PER_DAY,
};
use super::{query_evidence, record_exposure};

#[test]
fn uncertainty_survives_undo_and_reopen_without_changing_scheduling_or_evidence() {
    let directory = tempdir().unwrap();
    let store = LocalDataStore::open(directory.path()).unwrap();
    let library = ConceptLibrary::new(&store);
    let mut outcomes = Vec::new();
    let now = current_timestamp().unwrap() + MILLISECONDS_PER_DAY;
    let later = now + 10 * MILLISECONDS_PER_DAY;

    for marked in [false, true] {
        let concept = library
            .create_concept(
                serde_json::from_value(json!({
                    "title": if marked { "Marked" } else { "Unmarked" }
                }))
                .unwrap(),
            )
            .unwrap();
        let card_id = concept.cards[0].id.clone();
        let version = store
            .read_result(|connection| query_content_version(connection, &card_id))
            .unwrap();

        store
            .write_result(|transaction| {
                record_review(
                    transaction,
                    &card_id,
                    ReviewRating::Good,
                    Some(false),
                    Some(&version),
                    None,
                    now,
                )
            })
            .unwrap();

        let exposure = store
            .write_result(|transaction| {
                record_exposure(
                    transaction,
                    RecordExposureInput {
                        concept_id: concept.id,
                        card_id: Some(card_id.clone()),
                        content_version: Some(version.clone()),
                        uncertainty_marked: marked,
                    },
                    later,
                )
            })
            .unwrap();
        let review = store
            .write_result(|transaction| {
                record_review(
                    transaction,
                    &card_id,
                    ReviewRating::Good,
                    Some(false),
                    Some(&version),
                    Some(&exposure),
                    later,
                )
            })
            .unwrap();
        let summary = store
            .read_result(|connection| query_evidence(connection, &card_id, &version, later))
            .unwrap();

        assert_eq!(summary.delayed_total, 1);
        assert_eq!(summary.delayed_remembered, 1);
        assert_eq!(
            library.review_history(&card_id).unwrap().reviews[0].uncertainty_marked,
            Some(marked)
        );

        store
            .write_result(|transaction| reverse_review(transaction, &review.review_id, later + 1))
            .unwrap();
        assert_eq!(
            store
                .read_result(|connection| query_evidence(connection, &card_id, &version, later + 1))
                .unwrap()
                .delayed_total,
            0
        );

        let corrected = store
            .write_result(|transaction| {
                record_review(
                    transaction,
                    &card_id,
                    ReviewRating::Hard,
                    Some(false),
                    Some(&version),
                    Some(&exposure),
                    later + 2,
                )
            })
            .unwrap();
        let history = library.review_history(&card_id).unwrap();

        assert_eq!(history.reviews[0].uncertainty_marked, Some(marked));
        assert_eq!(history.reviews[1].uncertainty_marked, Some(marked));
        assert!(history.reviews[1].reversed_at.is_some());
        assert_eq!(history.reviews[2].uncertainty_marked, None);
        outcomes.push((review, corrected, card_id, history));
    }

    assert_eq!(outcomes[0].0.due_at, outcomes[1].0.due_at);
    assert_eq!(
        outcomes[0].0.scheduled_interval_days,
        outcomes[1].0.scheduled_interval_days
    );
    assert_eq!(outcomes[0].1.due_at, outcomes[1].1.due_at);
    assert_eq!(
        outcomes[0].1.scheduled_interval_days,
        outcomes[1].1.scheduled_interval_days
    );
    drop(library);
    drop(store);

    let store = LocalDataStore::open(directory.path()).unwrap();

    for (_, _, card_id, history) in outcomes {
        assert_eq!(
            ConceptLibrary::new(&store)
                .review_history(&card_id)
                .unwrap(),
            history
        );
    }
}

#[test]
fn uncertainty_defaults_to_unmarked_and_cannot_be_attached_to_support_or_prompt_context() {
    let directory = tempdir().unwrap();
    let store = LocalDataStore::open(directory.path()).unwrap();
    let library = ConceptLibrary::new(&store);
    let concept = library
        .create_concept(serde_json::from_value(json!({ "title": "Optional signal" })).unwrap())
        .unwrap();
    let now = current_timestamp().unwrap();
    let version = store
        .read_result(|connection| query_content_version(connection, &concept.cards[0].id))
        .unwrap();
    let default: RecordExposureInput = serde_json::from_value(json!({
        "conceptId": concept.id, "cardId": concept.cards[0].id, "contentVersion": version
    }))
    .unwrap();

    assert!(!default.uncertainty_marked);

    let exposure = store
        .write_result(|transaction| record_exposure(transaction, default, now))
        .unwrap();
    let before = store.changes_after(0, 1000).unwrap();

    for card_id in [None, Some(concept.cards[0].id.clone())] {
        assert!(matches!(
            store.write_result(|transaction| record_exposure(
                transaction,
                RecordExposureInput {
                    concept_id: concept.id.clone(),
                    card_id,
                    content_version: None,
                    uncertainty_marked: true,
                },
                now
            )),
            Err(LibraryError::InvalidContent { .. })
        ));
    }

    assert_eq!(store.changes_after(0, 1000).unwrap(), before);
    assert!(store
        .write_result::<_, LibraryError>(|transaction| {
            transaction.execute(
                "UPDATE practice_exposures SET uncertainty_marked = 1 WHERE entity_id = ?1",
                [&exposure],
            )?;
            Ok(())
        })
        .is_err());
}
