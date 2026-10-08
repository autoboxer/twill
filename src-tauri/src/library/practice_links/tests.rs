use std::fs::File;
use std::io::Read;

use serde_json::{json, Value};
use tempfile::{tempdir, TempDir};
use zip::ZipArchive;

use super::*;
use crate::library::{
    ConceptDetail, ConceptLibrary, CreateConceptInput, RecordReviewInput, ReviewRating,
};

fn setup() -> (TempDir, LocalDataStore) {
    let directory = tempdir().unwrap();
    let store = LocalDataStore::open(directory.path().join("original")).unwrap();

    (directory, store)
}

fn concept(store: &LocalDataStore, title: &str) -> ConceptDetail {
    let input: CreateConceptInput = serde_json::from_value(json!({ "title": title })).unwrap();

    ConceptLibrary::new(store).create_concept(input).unwrap()
}

fn link(store: &LocalDataStore, first: &str, second: &str, objective: &str) -> LibraryResult<()> {
    PracticeLinkLibrary::new(store).create(CreatePracticeLinkInput {
        concept_id: first.to_owned(),
        related_concept_id: second.to_owned(),
        objective: objective.to_owned(),
    })
}

fn changes(store: &LocalDataStore) -> usize {
    store.changes_after(0, 1000).unwrap().len()
}

#[test]
fn links_are_symmetric_independent_and_revision_checked() {
    let (_directory, store) = setup();
    let first = concept(&store, "Reliable file transfer");
    let second = concept(&store, "Reliable remote commands");
    let library = PracticeLinkLibrary::new(&store);
    let before = changes(&store);

    link(&store, &second.id, &first.id, "  Apply ordered delivery.  ").unwrap();

    let forward = library.links(&first.id).unwrap().remove(0);
    let backward = library.links(&second.id).unwrap().remove(0);

    assert_eq!(forward.id, backward.id);
    assert_eq!(forward.concept_id, second.id);
    assert_eq!(backward.concept_id, first.id);
    assert_eq!(forward.objective, "Apply ordered delivery.");
    assert_eq!(changes(&store), before + 1);
    assert_eq!(
        ConceptLibrary::new(&store).concept(&first.id).unwrap(),
        first
    );
    assert_eq!(
        ConceptLibrary::new(&store).concept(&second.id).unwrap(),
        second
    );

    library
        .update(UpdatePracticeLinkInput {
            id: forward.id.clone(),
            expected_change_id: forward.last_change_id.clone(),
            objective: forward.objective.clone(),
        })
        .unwrap();

    assert_eq!(changes(&store), before + 1);

    library
        .update(UpdatePracticeLinkInput {
            id: backward.id.clone(),
            expected_change_id: backward.last_change_id,
            objective: "Choose transport based on reliability.".to_owned(),
        })
        .unwrap();

    let updated = library.links(&first.id).unwrap().remove(0);

    assert_eq!(updated.id, forward.id);
    assert_ne!(updated.last_change_id, forward.last_change_id);
    assert!(matches!(
        library.remove(RemovePracticeLinkInput {
            id: forward.id.clone(),
            expected_change_id: forward.last_change_id.clone(),
        }),
        Err(LibraryError::PracticeLinkChanged)
    ));
    assert!(matches!(
        library.update(UpdatePracticeLinkInput {
            id: forward.id,
            expected_change_id: forward.last_change_id,
            objective: "Stale text".to_owned(),
        }),
        Err(LibraryError::PracticeLinkChanged)
    ));
    assert_eq!(changes(&store), before + 2);

    library
        .remove(RemovePracticeLinkInput {
            id: updated.id.clone(),
            expected_change_id: updated.last_change_id,
        })
        .unwrap();

    assert!(library.links(&first.id).unwrap().is_empty());
    assert!(library.links(&second.id).unwrap().is_empty());
    assert!(store
        .entity(&updated.id)
        .unwrap()
        .unwrap()
        .deleted_at
        .is_some());

    link(&store, &first.id, &second.id, "A revised connection.").unwrap();

    assert_ne!(library.links(&first.id).unwrap()[0].id, updated.id);
}

#[test]
fn invalid_and_duplicate_links_roll_back_without_changes() {
    let (_directory, store) = setup();
    let first = concept(&store, "First case");
    let second = concept(&store, "Second case");
    let before = changes(&store);

    for objective in [" ".to_owned(), "学".repeat(301)] {
        assert!(link(&store, &first.id, &second.id, &objective).is_err());
    }

    assert!(link(&store, &first.id, &first.id, "Self link.").is_err());
    assert!(link(&store, &first.id, "missing", "Unknown target.").is_err());
    assert_eq!(changes(&store), before);

    link(&store, &first.id, &second.id, &"学".repeat(300)).unwrap();

    assert!(matches!(
        link(&store, &second.id, &first.id, "Duplicate."),
        Err(LibraryError::PracticeLinkExists)
    ));
    assert_eq!(changes(&store), before + 1);
}

#[test]
fn archived_links_are_visible_and_deleting_a_concept_removes_its_links() {
    let (_directory, store) = setup();
    let first = concept(&store, "First case");
    let second = concept(&store, "Second case");
    let third = concept(&store, "Third case");
    let library = PracticeLinkLibrary::new(&store);
    let concepts = ConceptLibrary::new(&store);

    link(&store, &first.id, &second.id, "Existing connection.").unwrap();
    link(&store, &second.id, &third.id, "Another connection.").unwrap();
    concepts.set_concept_archived(&second.id, true).unwrap();

    assert!(library.links(&first.id).unwrap()[0].archived);
    assert_eq!(library.links(&second.id).unwrap().len(), 2);
    assert!(link(&store, &first.id, &second.id, "Archived target.").is_err());
    assert!(link(&store, &second.id, &first.id, "Archived source.").is_err());

    let old_links = library.links(&second.id).unwrap();

    concepts.delete_concept(&second.id).unwrap();

    assert!(library.links(&first.id).unwrap().is_empty());
    assert!(library.links(&third.id).unwrap().is_empty());
    assert!(library.links(&second.id).is_err());

    for old in old_links {
        assert!(store.entity(&old.id).unwrap().unwrap().deleted_at.is_some());
    }

    store.read_result(super::validate_restored_links).unwrap();
}

#[test]
fn connection_limit_applies_to_both_ends_without_transitive_links() {
    let (_directory, store) = setup();
    let center = concept(&store, "Shared objective");
    let library = PracticeLinkLibrary::new(&store);
    let mut cases = Vec::new();

    for index in 0..32 {
        let case = concept(&store, &format!("Case {index}"));

        link(&store, &case.id, &center.id, "Apply the principle.").unwrap();
        cases.push(case);
    }

    assert_eq!(library.links(&center.id).unwrap().len(), 32);
    assert_eq!(library.links(&cases[0].id).unwrap().len(), 1);

    let extra = concept(&store, "Another case");
    let before = changes(&store);

    assert!(link(&store, &extra.id, &center.id, "Another connection.").is_err());
    assert!(link(&store, &center.id, &extra.id, "Another connection.").is_err());
    assert_eq!(changes(&store), before);
}

#[test]
fn rating_changes_only_the_attempted_card() {
    let (_directory, store) = setup();
    let first = concept(&store, "File transfer");
    let second = concept(&store, "Remote commands");
    let library = ConceptLibrary::new(&store);

    link(
        &store,
        &first.id,
        &second.id,
        "Use reliable ordered delivery.",
    )
    .unwrap();
    library
        .record_review(RecordReviewInput {
            card_id: first.cards[0].id.clone(),
            rating: ReviewRating::Good,
            assisted: false,
        })
        .unwrap();

    assert_eq!(library.concept(&second.id).unwrap(), second);
    assert_eq!(library.concept(&first.id).unwrap().cards[0].review_count, 1);
    assert_eq!(
        PracticeLinkLibrary::new(&store)
            .links(&first.id)
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn linked_practice_loads_independent_cards_and_templates_without_scheduling_writes() {
    let (_directory, store) = setup();
    let source = concept(&store, "Transport choice");
    let template = crate::library::TemplateLibrary::new(&store)
        .create_template(crate::library::CreateTemplateInput {
            name: "Application question".to_owned(),
            content: Default::default(),
        })
        .unwrap();
    let input = serde_json::from_value(json!({
        "title": "File transfer",
        "templateIds": [template.id],
        "explain": {"focus": "why", "keyPoints": ["Ordered delivery."]}
    }))
    .unwrap();
    let concepts = ConceptLibrary::new(&store);
    let target = concepts.create_concept(input).unwrap();

    link(&store, &source.id, &target.id, "Choose based on ordering.").unwrap();
    concepts.record_review(RecordReviewInput {
        card_id: target.cards[0].id.clone(),
        rating: ReviewRating::Good,
        assisted: false,
    }).unwrap();

    let target = concepts.concept(&target.id).unwrap();
    let before = changes(&store);
    let links = PracticeLinkLibrary::new(&store);
    let linked = links.links(&source.id).unwrap().remove(0);
    let practice = links.practice(LinkedPracticeInput {
        concept_id: source.id.clone(),
        link_id: linked.id.clone(),
    }).unwrap();

    assert_eq!(practice.source_last_change_id, source.last_change_id);
    assert_eq!(practice.link, linked);
    assert_eq!(practice.concept, target);
    assert_eq!(practice.templates.len(), 1);
    assert_eq!(practice.templates[0].content, template.content);
    assert!(practice.concept.cards.iter().any(|card| {
        card.review_count == 1 && card.due_at > crate::data::current_timestamp().unwrap()
    }));
    assert_eq!(changes(&store), before);
    assert_eq!(concepts.concept(&source.id).unwrap(), source);
    assert_eq!(concepts.concept(&target.id).unwrap(), target);
}

#[test]
fn linked_practice_rejects_unrelated_removed_archived_and_deleted_targets() {
    let (_directory, store) = setup();
    let source = concept(&store, "First case");
    let target = concept(&store, "Second case");
    let unrelated = concept(&store, "Unrelated case");
    let links = PracticeLinkLibrary::new(&store);
    let concepts = ConceptLibrary::new(&store);

    link(&store, &source.id, &target.id, "Apply a shared principle.").unwrap();

    let linked = links.links(&source.id).unwrap().remove(0);
    let load = |id: &str| links.practice(LinkedPracticeInput {
        concept_id: id.to_owned(),
        link_id: linked.id.clone(),
    });

    assert!(matches!(load(&unrelated.id), Err(LibraryError::PracticeLinkNotFound(_))));
    concepts.set_concept_archived(&target.id, true).unwrap();
    assert!(load(&source.id).is_err());
    assert!(load(&target.id).is_err());
    concepts.set_concept_archived(&target.id, false).unwrap();
    assert_eq!(load(&target.id).unwrap().concept.id, source.id);
    links.remove(RemovePracticeLinkInput {
        id: linked.id.clone(),
        expected_change_id: linked.last_change_id.clone(),
    }).unwrap();
    assert!(matches!(load(&source.id), Err(LibraryError::PracticeLinkNotFound(_))));
    link(&store, &source.id, &target.id, "A new connection.").unwrap();
    concepts.delete_concept(&target.id).unwrap();
    assert!(links.links(&source.id).unwrap().is_empty());
    assert!(load(&source.id).is_err());
}

#[test]
fn links_and_removal_history_survive_backup_and_portable_export() {
    let (directory, store) = setup();
    let first = concept(&store, "File transfer");
    let second = concept(&store, "Remote commands");
    let library = PracticeLinkLibrary::new(&store);

    link(&store, &first.id, &second.id, "Old objective.").unwrap();

    let removed = library.links(&first.id).unwrap().remove(0);

    library
        .remove(RemovePracticeLinkInput {
            id: removed.id.clone(),
            expected_change_id: removed.last_change_id,
        })
        .unwrap();
    link(
        &store,
        &first.id,
        &second.id,
        "Choose reliability when order matters.",
    )
    .unwrap();
    ConceptLibrary::new(&store)
        .set_concept_archived(&second.id, true)
        .unwrap();

    let expected = library.links(&first.id).unwrap();
    let path = directory.path().join("backup.twill");

    crate::backup::create_backup(&store, &path).unwrap();
    crate::backup::inspect_backup(&path).unwrap();

    let mut archive = ZipArchive::new(File::open(path).unwrap()).unwrap();
    let mut bytes = Vec::new();

    archive
        .by_name("twill.sqlite3")
        .unwrap()
        .read_to_end(&mut bytes)
        .unwrap();

    let restored_directory = directory.path().join("restored");

    std::fs::create_dir(&restored_directory).unwrap();
    std::fs::write(restored_directory.join("twill.sqlite3"), bytes).unwrap();

    let restored = LocalDataStore::open(&restored_directory).unwrap();

    assert_eq!(
        PracticeLinkLibrary::new(&restored)
            .links(&first.id)
            .unwrap(),
        expected
    );
    assert!(restored
        .entity(&removed.id)
        .unwrap()
        .unwrap()
        .deleted_at
        .is_some());

    let export = directory.path().join("export.twill");

    crate::backup::export_library(&store, &export).unwrap();

    let mut archive = ZipArchive::new(File::open(export).unwrap()).unwrap();
    let mut text = String::new();

    archive
        .by_name("library.json")
        .unwrap()
        .read_to_string(&mut text)
        .unwrap();

    let portable: Value = serde_json::from_str(&text).unwrap();

    assert_eq!(
        portable["tables"]["practice_links"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
}
