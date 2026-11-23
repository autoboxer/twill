use serde_json::json;
use tempfile::{tempdir, TempDir};

use super::*;
use crate::data::{current_timestamp, LocalDataStore};
use crate::library::study::{record_review, reverse_review};
use crate::library::{ConceptDetail, ConceptLibrary, PracticeLinkLibrary, ReviewRating};

fn setup() -> (TempDir, LocalDataStore, ConceptDetail, String, i64) {
    let directory = tempdir().unwrap();
    let store = LocalDataStore::open(directory.path()).unwrap();
    let concept = concept(&store, "Evidence");
    let version = store
        .read_result(|connection| query_content_version(connection, &concept.cards[0].id))
        .unwrap();
    let now = current_timestamp().unwrap();

    (directory, store, concept, version, now)
}

fn concept(store: &LocalDataStore, title: &str) -> ConceptDetail {
    ConceptLibrary::new(store)
        .create_concept(serde_json::from_value(json!({ "title": title })).unwrap())
        .unwrap()
}

fn expose(
    store: &LocalDataStore,
    concept: &ConceptDetail,
    version: Option<&str>,
    now: i64,
) -> String {
    store
        .write_result(|transaction| {
            record_exposure(
                transaction,
                RecordExposureInput {
                    uncertainty_marked: false,
                    concept_id: concept.id.clone(),
                    card_id: version.map(|_| concept.cards[0].id.clone()),
                    content_version: version.map(str::to_owned),
                },
                now,
            )
        })
        .unwrap()
}

fn grade(
    store: &LocalDataStore,
    concept: &ConceptDetail,
    version: &str,
    exposure: Option<&str>,
    now: i64,
) -> crate::library::ReviewOutcome {
    store
        .write_result(|transaction| {
            record_review(
                transaction,
                &concept.cards[0].id,
                ReviewRating::Good,
                Some(false),
                Some(version),
                exposure,
                now,
            )
        })
        .unwrap()
}

fn evidence(
    store: &LocalDataStore,
    concept: &ConceptDetail,
    version: &str,
    now: i64,
) -> LearningEvidence {
    store
        .read_result(|connection| query_evidence(connection, &concept.cards[0].id, version, now))
        .unwrap()
}

#[test]
fn elapsed_boundary_uses_exposure_not_calendar_day_or_review_gap() {
    let (_directory, store, concept, version, now) = setup();
    let first = expose(&store, &concept, Some(&version), now);
    grade(&store, &concept, &version, Some(&first), now);
    let later = now + 10 * MILLISECONDS_PER_DAY;

    expose(&store, &concept, None, later);

    for (elapsed, expected) in [
        (MILLISECONDS_PER_DAY - 1, "recent"),
        (MILLISECONDS_PER_DAY, "delayed"),
    ] {
        let kind = store
            .read_result(|connection| {
                recall_kind(
                    connection,
                    &concept.cards[0].id,
                    &concept.id,
                    &version,
                    later + elapsed,
                )
            })
            .unwrap();
        assert_eq!(kind, expected);
    }

    let delayed = expose(
        &store,
        &concept,
        Some(&version),
        later + MILLISECONDS_PER_DAY,
    );
    grade(
        &store,
        &concept,
        &version,
        Some(&delayed),
        later + MILLISECONDS_PER_DAY,
    );
    let summary = evidence(&store, &concept, &version, later + MILLISECONDS_PER_DAY);
    assert_eq!(summary.first, 1);
    assert_eq!(summary.delayed_total, 1);
    assert_eq!(summary.delayed_remembered, 1);
    assert_eq!(
        evidence(
            &store,
            &concept,
            &version,
            later + 32 * MILLISECONDS_PER_DAY
        ),
        LearningEvidence::default()
    );
}

#[test]
fn undo_excludes_grade_but_preserves_exposure_and_allows_correction_once() {
    let (_directory, store, concept, version, now) = setup();
    let first = expose(&store, &concept, Some(&version), now);
    let original = grade(&store, &concept, &version, Some(&first), now);
    store
        .write_result(|transaction| reverse_review(transaction, &original.review_id, now + 1))
        .unwrap();
    assert_eq!(
        evidence(&store, &concept, &version, now + 1),
        LearningEvidence::default()
    );

    let corrected = grade(&store, &concept, &version, Some(&first), now + 2);
    assert_eq!(evidence(&store, &concept, &version, now + 2).first, 1);
    assert!(store
        .write_result(|transaction| record_review(
            transaction,
            &concept.cards[0].id,
            ReviewRating::Good,
            Some(false),
            Some(&version),
            Some(&first),
            corrected.due_at
        ))
        .is_err());

    let later = corrected.due_at.max(now + 3 * MILLISECONDS_PER_DAY);
    let delayed = expose(&store, &concept, Some(&version), later);
    let review = grade(&store, &concept, &version, Some(&delayed), later);
    store
        .write_result(|transaction| reverse_review(transaction, &review.review_id, later + 1))
        .unwrap();
    let retry = expose(&store, &concept, Some(&version), later + 2);
    grade(&store, &concept, &version, Some(&retry), later + 2);
    let summary = evidence(&store, &concept, &version, later + 2);
    assert_eq!(summary.delayed_total, 0);
    assert_eq!(summary.recent, 1);
}

#[test]
fn linked_exposure_and_reopen_preserve_recent_context() {
    let (directory, store, source, version, now) = setup();
    let related = concept(&store, "Linked case");
    let unrelated = concept(&store, "Unrelated case");
    PracticeLinkLibrary::new(&store)
        .create(
            serde_json::from_value(json!({
                "conceptId": source.id, "relatedConceptId": related.id, "objective": "Compare cases"
            }))
            .unwrap(),
        )
        .unwrap();
    let first = expose(&store, &source, Some(&version), now);
    grade(&store, &source, &version, Some(&first), now);
    let later = now + 10 * MILLISECONDS_PER_DAY;
    expose(&store, &unrelated, None, later);
    assert_eq!(
        store
            .read_result(|connection| recall_kind(
                connection,
                &source.cards[0].id,
                &source.id,
                &version,
                later
            ))
            .unwrap(),
        "delayed"
    );
    expose(&store, &related, None, later);
    drop(store);

    let store = LocalDataStore::open(directory.path()).unwrap();
    let exposure = expose(&store, &source, Some(&version), later + 1);
    grade(&store, &source, &version, Some(&exposure), later + 1);
    assert_eq!(evidence(&store, &source, &version, later + 1).recent, 1);
}

#[test]
fn help_missing_context_and_changed_versions_never_inflate_delayed_evidence() {
    let (_directory, store, concept, version, now) = setup();
    let first = grade(&store, &concept, &version, None, now);
    let later = first.due_at.max(now + 3 * MILLISECONDS_PER_DAY);
    let exposure = expose(&store, &concept, Some(&version), later);
    store
        .write_result(|transaction| {
            record_review(
                transaction,
                &concept.cards[0].id,
                ReviewRating::Good,
                Some(true),
                Some(&version),
                Some(&exposure),
                later,
            )
        })
        .unwrap();
    let summary = evidence(&store, &concept, &version, later);
    assert_eq!(summary.unknown, 1);
    assert_eq!(summary.helped, 1);
    assert_eq!(summary.delayed_total, 0);

    ConceptLibrary::new(&store)
        .update_concept(
            serde_json::from_value(json!({
                "id": concept.id, "title": "Changed target"
            }))
            .unwrap(),
        )
        .unwrap();
    let current = store
        .read_result(|connection| query_content_version(connection, &concept.cards[0].id))
        .unwrap();
    assert_eq!(
        evidence(&store, &concept, &current, later),
        LearningEvidence::default()
    );
    assert_eq!(
        store
            .read_result(|connection| recall_kind(
                connection,
                &concept.cards[0].id,
                &concept.id,
                &current,
                later + 10 * MILLISECONDS_PER_DAY
            ))
            .unwrap(),
        "first"
    );
    assert!(store
        .write_result(|transaction| record_exposure(
            transaction,
            RecordExposureInput {
                uncertainty_marked: false,
                concept_id: concept.id,
                card_id: Some(concept.cards[0].id.clone()),
                content_version: Some(version),
            },
            later
        ))
        .is_err());
}

#[test]
fn shared_prompt_exposure_affects_siblings_without_exposing_its_own_hidden_target() {
    let directory = tempdir().unwrap();
    let store = LocalDataStore::open(directory.path()).unwrap();
    let concept = ConceptLibrary::new(&store)
        .create_concept(
            serde_json::from_value(json!({
                "title": "Related forms",
                "typeAnswer": { "acceptedAnswers": ["TCP"] }
            }))
            .unwrap(),
        )
        .unwrap();
    let now = current_timestamp().unwrap();

    for card in &concept.cards {
        let version = store
            .read_result(|connection| query_content_version(connection, &card.id))
            .unwrap();
        store
            .write_result(|transaction| {
                record_review(
                    transaction,
                    &card.id,
                    ReviewRating::Good,
                    Some(false),
                    Some(&version),
                    None,
                    now,
                )
            })
            .unwrap();
    }

    let later = now + 10 * MILLISECONDS_PER_DAY;
    let context_id = store
        .write_result(|transaction| {
            record_exposure(
                transaction,
                RecordExposureInput {
                    uncertainty_marked: false,
                    concept_id: concept.id.clone(),
                    card_id: Some(concept.cards[0].id.clone()),
                    content_version: None,
                },
                later,
            )
        })
        .unwrap();

    for (index, card) in concept.cards.iter().enumerate() {
        let version = store
            .read_result(|connection| query_content_version(connection, &card.id))
            .unwrap();
        let kind = store
            .read_result(|connection| {
                recall_kind(connection, &card.id, &concept.id, &version, later + 1)
            })
            .unwrap();
        assert_eq!(kind, if index == 0 { "delayed" } else { "recent" });
        assert!(store
            .write_result(|transaction| record_review(
                transaction,
                &card.id,
                ReviewRating::Good,
                Some(false),
                Some(&version),
                Some(&context_id),
                later + 1
            ))
            .is_err());
    }

    assert!(store
        .write_result::<_, LibraryError>(|transaction| {
            transaction.execute(
                "UPDATE practice_exposures SET occurred_at = occurred_at + 1 WHERE entity_id = ?1",
                [&context_id],
            )?;
            Ok(())
        })
        .is_err());
}

#[test]
fn workload_is_whole_active_library_and_undo_restores_unseen_count() {
    let (_directory, store, source, version, now) = setup();
    let _unseen = concept(&store, "Unseen");
    let archived = concept(&store, "Archived");
    ConceptLibrary::new(&store)
        .set_concept_archived(&archived.id, true)
        .unwrap();
    let workload = store
        .read_result(|connection| query_workload(connection, now))
        .unwrap();
    assert_eq!(workload.new, 2);
    assert_eq!(workload.due, 0);
    let review = grade(&store, &source, &version, None, now);
    let workload = store
        .read_result(|connection| query_workload(connection, now))
        .unwrap();
    assert_eq!(workload.new, 1);
    assert_eq!(workload.introduced, 1);
    let later = review.due_at + MILLISECONDS_PER_DAY;
    let workload = store
        .read_result(|connection| query_workload(connection, later))
        .unwrap();
    assert_eq!(workload.due, 1);
    assert_eq!(workload.overdue, 1);
    store
        .write_result(|transaction| reverse_review(transaction, &review.review_id, later))
        .unwrap();
    let workload = store
        .read_result(|connection| query_workload(connection, later))
        .unwrap();
    assert_eq!(workload.due, 0);
    assert_eq!(workload.introduced, 0);
    assert_eq!(workload.new, 2);
}
