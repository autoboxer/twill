use std::fs;

use serde_json::json;
use tempfile::tempdir;

use crate::library::{ConceptLibrary, CreateConceptInput};

use super::*;
use crate::backup::{create_backup, inspect_backup};

#[test]
fn corrupt_startup_keeps_original_data_and_lock_with_bounded_redacted_diagnostics() {
    let root = tempdir().unwrap();
    let profile = root.path().join("private-source-title");

    fs::create_dir(&profile).unwrap();

    let original = b"not a SQLite database: private authored content";

    fs::write(profile.join("twill.sqlite3"), original).unwrap();

    let recovery = StorageRecovery::new(profile.clone());

    for _ in 0..20 {
        assert!(recovery
            .open(|_| panic!("invalid storage cannot be published"))
            .is_err());
    }

    let status = recovery.status().unwrap();
    let serialized = serde_json::to_string(&status).unwrap();

    assert!(!status.ready);
    assert!(!status.restore_pending);
    assert_eq!(status.diagnostics.len(), MAX_DIAGNOSTICS);
    assert_eq!(status.diagnostics[0].category, "sqlite");
    assert_eq!(status.diagnostics[0].code, Some(26));
    assert!(!serialized.contains("private"));
    assert!(!serialized.contains(root.path().to_str().unwrap()));
    assert_eq!(fs::read(profile.join("twill.sqlite3")).unwrap(), original);
    assert!(LocalDataStore::open(&profile).is_err());

    drop(recovery);

    assert!(LocalDataStore::lock_profile(&profile).is_ok());
}

#[test]
fn recovery_can_restore_corrupt_storage_then_publish_a_working_store() {
    let root = tempdir().unwrap();
    let source = LocalDataStore::open(root.path().join("source")).unwrap();
    let input: CreateConceptInput =
        serde_json::from_value(json!({ "title": "Recovered" })).unwrap();

    ConceptLibrary::new(&source).create_concept(input).unwrap();

    let archive = root.path().join("backup.twill");

    create_backup(&source, &archive).unwrap();

    let profile = root.path().join("target");

    fs::create_dir(&profile).unwrap();
    fs::write(profile.join("twill.sqlite3"), b"corrupt").unwrap();

    let recovery = StorageRecovery::new(profile);

    assert!(recovery.open(|_| unreachable!()).is_err());

    let preview = inspect_backup(&archive).unwrap();

    recovery.prepare(&archive, &preview.fingerprint).unwrap();

    assert!(recovery.status().unwrap().restore_pending);

    let mut restored = None;

    recovery.open(|store| restored = Some(store)).unwrap();

    assert!(recovery.status().unwrap().ready);
    assert!(!recovery.status().unwrap().restore_pending);
    assert_eq!(
        ConceptLibrary::new(restored.as_ref().unwrap())
            .search(Default::default())
            .unwrap()
            .concepts[0]
            .title,
        "Recovered"
    );

    recovery
        .open(|_| panic!("ready storage must not be republished"))
        .unwrap();
}

#[test]
fn a_busy_profile_cannot_be_restored_and_can_be_retried_after_its_owner_exits() {
    let root = tempdir().unwrap();
    let profile = root.path().join("profile");
    let store = LocalDataStore::open(&profile).unwrap();
    let archive = root.path().join("backup.twill");

    create_backup(&store, &archive).unwrap();

    let recovery = StorageRecovery::new(profile.clone());
    let preview = inspect_backup(&archive).unwrap();

    assert!(recovery.open(|_| unreachable!()).is_err());
    assert!(recovery.prepare(&archive, &preview.fingerprint).is_err());
    assert!(!profile.join(".twill-restore").exists());

    drop(store);

    let mut published = None;

    recovery.open(|store| published = Some(store)).unwrap();

    assert!(recovery.status().unwrap().ready);
    assert!(published.is_some());
}
