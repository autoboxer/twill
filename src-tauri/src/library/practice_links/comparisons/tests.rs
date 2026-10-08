use serde_json::json;

use super::*;
use crate::library::practice_links::tests::{changes, concept, link, setup};
use crate::library::{ConceptLibrary, LibraryError};

fn example() -> PracticeComparison {
    PracticeComparison {
        prompt: "Which requirement changes the transport?".to_owned(),
        explanation: "A file needs reliable order. Live video can discard late frames.".to_owned(),
    }
}

#[test]
fn comparisons_are_symmetric_revision_checked_and_independent_of_cards() {
    let (_directory, store) = setup();
    let first = concept(&store, "Complete file");
    let second = concept(&store, "Live video");
    let library = PracticeLinkLibrary::new(&store);

    link(
        &store,
        &first.id,
        &second.id,
        "Distinguish reliability and latency.",
    )
    .unwrap();

    let original = library.links(&first.id).unwrap().remove(0);
    let input = SetPracticeComparisonInput {
        id: original.id.clone(),
        expected_change_id: original.last_change_id.clone(),
        comparison: Some(example()),
    };

    library.set_comparison(input.clone()).unwrap();

    let updated = library.links(&first.id).unwrap().remove(0);
    let reverse = library.links(&second.id).unwrap().remove(0);

    assert_eq!(updated.comparison, reverse.comparison);
    assert_eq!(updated.comparison, Some(example()));
    assert_eq!(updated.id, original.id);
    assert_eq!(updated.objective, original.objective);
    assert_ne!(updated.last_change_id, original.last_change_id);
    assert_eq!(
        ConceptLibrary::new(&store).concept(&first.id).unwrap(),
        first
    );
    assert_eq!(
        ConceptLibrary::new(&store).concept(&second.id).unwrap(),
        second
    );

    let before = changes(&store);

    assert!(matches!(
        library.set_comparison(input),
        Err(LibraryError::PracticeLinkChanged)
    ));
    library
        .set_comparison(SetPracticeComparisonInput {
            id: updated.id.clone(),
            expected_change_id: updated.last_change_id.clone(),
            comparison: Some(example()),
        })
        .unwrap();
    assert_eq!(changes(&store), before);

    library
        .set_comparison(SetPracticeComparisonInput {
            id: updated.id,
            expected_change_id: updated.last_change_id,
            comparison: None,
        })
        .unwrap();

    assert_eq!(library.links(&first.id).unwrap()[0].comparison, None);
    assert_eq!(library.links(&second.id).unwrap().len(), 1);
    assert_eq!(
        ConceptLibrary::new(&store).concept(&first.id).unwrap(),
        first
    );
}

#[test]
fn comparisons_normalize_unicode_and_reject_invalid_input_without_writes() {
    let (_directory, store) = setup();
    let first = concept(&store, "First");
    let second = concept(&store, "Second");
    let library = PracticeLinkLibrary::new(&store);

    link(&store, &first.id, &second.id, "Compare cases.").unwrap();

    let current = library.links(&first.id).unwrap().remove(0);
    let before = changes(&store);

    for comparison in [
        PracticeComparison {
            prompt: " ".to_owned(),
            ..example()
        },
        PracticeComparison {
            explanation: String::new(),
            ..example()
        },
        PracticeComparison {
            prompt: "网".repeat(501),
            ..example()
        },
        PracticeComparison {
            explanation: "🌿".repeat(2_001),
            ..example()
        },
    ] {
        assert!(library
            .set_comparison(SetPracticeComparisonInput {
                id: current.id.clone(),
                expected_change_id: current.last_change_id.clone(),
                comparison: Some(comparison),
            })
            .is_err());
        assert_eq!(changes(&store), before);
    }

    library
        .set_comparison(SetPracticeComparisonInput {
            id: current.id,
            expected_change_id: current.last_change_id,
            comparison: Some(PracticeComparison {
                prompt: format!("  {}  ", "网".repeat(500)),
                explanation: format!("\n{}\n", "🌿".repeat(2_000)),
            }),
        })
        .unwrap();

    let saved = library
        .links(&first.id)
        .unwrap()
        .remove(0)
        .comparison
        .unwrap();

    assert_eq!(saved.prompt, "网".repeat(500));
    assert_eq!(saved.explanation, "🌿".repeat(2_000));
    assert!(serde_json::from_value::<PracticeComparison>(json!({
        "prompt": "Compare.", "explanation": "Expected.", "unknown": true,
    }))
    .is_err());
}

#[test]
fn comparison_reads_both_current_cases_without_review_or_change_events() {
    let (_directory, store) = setup();
    let first = concept(&store, "First");
    let second = concept(&store, "Second");
    let library = PracticeLinkLibrary::new(&store);

    link(&store, &first.id, &second.id, "Compare cases.").unwrap();

    let current = library.links(&first.id).unwrap().remove(0);

    library
        .set_comparison(SetPracticeComparisonInput {
            id: current.id.clone(),
            expected_change_id: current.last_change_id,
            comparison: Some(example()),
        })
        .unwrap();

    let before = changes(&store);

    for (source, related) in [(&first, &second), (&second, &first)] {
        let result = library
            .comparison(LinkedPracticeInput {
                concept_id: source.id.clone(),
                link_id: current.id.clone(),
            })
            .unwrap();

        assert_eq!(result.source, *source);
        assert_eq!(result.related, *related);
        assert_eq!(result.link.comparison, Some(example()));
    }

    assert_eq!(changes(&store), before);
    assert_eq!(first.cards[0].review_count, 0);
    assert_eq!(
        ConceptLibrary::new(&store).concept(&first.id).unwrap(),
        first
    );
    assert_eq!(
        ConceptLibrary::new(&store).concept(&second.id).unwrap(),
        second
    );
}

#[test]
fn comparison_rejects_missing_unrelated_archived_and_removed_cases() {
    let (_directory, store) = setup();
    let first = concept(&store, "First");
    let second = concept(&store, "Second");
    let unrelated = concept(&store, "Unrelated");
    let library = PracticeLinkLibrary::new(&store);

    link(&store, &first.id, &second.id, "Compare cases.").unwrap();

    let current = library.links(&first.id).unwrap().remove(0);
    let read = |id: &str| {
        library.comparison(LinkedPracticeInput {
            concept_id: id.to_owned(),
            link_id: current.id.clone(),
        })
    };

    assert!(read(&first.id).is_err());
    library
        .set_comparison(SetPracticeComparisonInput {
            id: current.id.clone(),
            expected_change_id: current.last_change_id,
            comparison: Some(example()),
        })
        .unwrap();
    assert!(read(&unrelated.id).is_err());

    ConceptLibrary::new(&store)
        .set_concept_archived(&second.id, true)
        .unwrap();
    assert!(read(&first.id).is_err());
    assert!(read(&second.id).is_err());
    ConceptLibrary::new(&store)
        .set_concept_archived(&second.id, false)
        .unwrap();
    assert!(read(&first.id).is_ok());

    ConceptLibrary::new(&store)
        .delete_concept(&second.id)
        .unwrap();
    assert!(read(&first.id).is_err());
    assert!(library
        .set_comparison(SetPracticeComparisonInput {
            id: current.id,
            expected_change_id: "stale".to_owned(),
            comparison: None,
        })
        .is_err());
}

#[test]
fn restore_validation_rejects_invalid_comparisons() {
    let (_directory, store) = setup();
    let first = concept(&store, "First");
    let second = concept(&store, "Second");

    link(&store, &first.id, &second.id, "Compare cases.").unwrap();

    let current = PracticeLinkLibrary::new(&store)
        .links(&first.id)
        .unwrap()
        .remove(0);

    for value in [
        json!({"prompt": " ", "explanation": "Expected."}),
        json!({"prompt": "Compare.", "explanation": "Expected.", "unknown": true}),
        json!({"prompt": "Compare.", "explanation": "网".repeat(2_001)}),
    ] {
        store.write_result::<_, LibraryError>(|transaction| {
            let entity = transaction.touch_entity(&current.id)?;

            transaction.execute(
                "UPDATE practice_links SET comparison_json = ?1, last_change_id = ?2 WHERE entity_id = ?3",
                params![value.to_string(), entity.last_change_id, current.id],
            )?;

            Ok(())
        }).unwrap();

        assert!(store
            .read_result(super::super::validate_restored_links)
            .is_err());
    }
}
