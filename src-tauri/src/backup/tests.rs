use std::fs::{self, File};
use std::io::{Cursor, Read, Write};
use std::path::Path;
use std::sync::Barrier;

use image::{DynamicImage, ImageFormat};
use rusqlite::Connection;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use tempfile::{tempdir, TempDir};
use zip::{CompressionMethod, ZipArchive};

use crate::data::{DataResult, LocalDataStore};
use crate::library::{
    AuthoringDraftKind, AuthoringDraftLibrary, AuthoringMediaLibrary, ConceptContent,
    ConceptLibrary, CreateConceptInput, CreateTemplateInput, GradingMode, RecordReviewInput,
    ReverseReviewInput, ReviewRating, TemplateContent, TemplateLibrary, UpsertAuthoringDraftInput,
};

use super::checksum::{copy_and_digest, finish_digest};
use super::models::{ArchiveManifest, DATABASE_PATH, LIBRARY_PATH, MANIFEST_PATH};
use super::portable::{schema_fingerprint, write_library};
use super::{create_backup, export_library, ArchiveKind, BackupError, LibraryCounts};

fn test_store() -> (TempDir, LocalDataStore) {
    let directory = tempdir().unwrap();
    let store = LocalDataStore::open(directory.path().join("profile")).unwrap();

    (directory, store)
}

fn concept_input(title: &str) -> CreateConceptInput {
    serde_json::from_value(json!({ "title": title })).unwrap()
}

fn png_bytes() -> Vec<u8> {
    let mut output = Cursor::new(Vec::new());

    DynamicImage::new_rgba8(4, 3)
        .write_to(&mut output, ImageFormat::Png)
        .unwrap();

    output.into_inner()
}

fn read_entry(archive: &mut ZipArchive<File>, path: &str) -> Vec<u8> {
    let mut bytes = Vec::new();

    archive
        .by_name(path)
        .unwrap()
        .read_to_end(&mut bytes)
        .unwrap();

    bytes
}

fn read_manifest(archive: &mut ZipArchive<File>) -> ArchiveManifest {
    serde_json::from_slice(&read_entry(archive, MANIFEST_PATH)).unwrap()
}

fn media_path(store: &LocalDataStore) -> std::path::PathBuf {
    let filename = store
        .read_result::<_, crate::data::DataError>(|connection| {
            Ok(connection.query_row(
                "SELECT digest || '.' || file_extension FROM media LIMIT 1",
                [],
                |row| row.get::<_, String>(0),
            )?)
        })
        .unwrap();

    store.media_directory().join(filename)
}

fn assert_no_export_temporaries(directory: &Path) {
    for entry in fs::read_dir(directory).unwrap() {
        let name = entry.unwrap().file_name();

        assert!(!name.to_string_lossy().starts_with(".twill-"), "{name:?}");
    }
}

#[test]
fn backup_captures_uncheckpointed_content_media_and_all_learning_state() {
    let (directory, store) = test_store();

    store
        .read_result::<_, crate::data::DataError>(|connection| {
            connection.pragma_update(None, "wal_autocheckpoint", 0)?;

            Ok(())
        })
        .unwrap();

    let library = ConceptLibrary::new(&store);
    let deck = library.create_deck("网络 / Networking".to_owned()).unwrap();
    let tag = library.create_tag("Chapter 1".to_owned()).unwrap();
    let template = TemplateLibrary::new(&store)
        .create_template(CreateTemplateInput {
            name: "Custom view".to_owned(),
            content: TemplateContent::default(),
        })
        .unwrap();
    let image = library.import_image(&png_bytes()).unwrap();
    let mut input = concept_input("TCP café — 🧵");

    input.deck_ids = vec![deck.id.clone()];
    input.tag_ids = vec![tag.id.clone()];
    input.template_ids = vec![template.id.clone()];
    input.content.prompt = json!({
        "type": "doc", "content": [{
            "type": "mediaImage", "attrs": {
                "mediaId": image.id, "alt": "Diagram", "title": null
            }
        }]
    });

    let concept = library.create_concept(input).unwrap();
    let review = library
        .record_review(RecordReviewInput {
            assisted: false,
            card_id: concept.cards[0].id.clone(),
            rating: ReviewRating::Good,
        })
        .unwrap();

    library
        .reverse_review(ReverseReviewInput {
            review_id: review.review_id.clone(),
        })
        .unwrap();
    library.set_grading_mode(GradingMode::Advanced).unwrap();
    library.set_concept_archived(&concept.id, true).unwrap();

    let draft = AuthoringDraftLibrary::new(&store)
        .upsert_draft(UpsertAuthoringDraftInput {
            kind: AuthoringDraftKind::Concept,
            target_id: None,
            schema_version: 1,
            base_change_id: None,
            payload: json!({ "title": "Saved draft", "content": ConceptContent::default() }),
            media_ids: vec![image.id.clone()],
            media_session_id: None,
        })
        .unwrap();

    assert!(
        fs::metadata(store.data_directory().join("twill.sqlite3-wal"))
            .unwrap()
            .len()
            > 0
    );

    let path = directory.path().join("library.twill");
    let summary = create_backup(&store, &path).unwrap();
    let mut archive = ZipArchive::new(File::open(&path).unwrap()).unwrap();
    let manifest = read_manifest(&mut archive);

    assert_eq!(summary.kind, ArchiveKind::Backup);
    assert_eq!(summary.byte_size, fs::metadata(&path).unwrap().len());
    assert_eq!(summary.created_at, manifest.created_at);
    assert_eq!(
        summary.counts,
        LibraryCounts {
            concepts: 1,
            cards: 2,
            reviews: 1,
            media: 1
        }
    );
    assert_eq!(manifest.format, "twill");
    assert_eq!(manifest.format_version, 1);
    assert_eq!(manifest.schema_version, 1);
    assert_eq!(manifest.kind, ArchiveKind::Backup);
    assert_eq!(manifest.counts, summary.counts);
    assert_eq!(manifest.files.len() + 1, archive.len());
    assert!(archive.by_name("twill.sqlite3-wal").is_err());
    assert!(archive.by_name("twill.lock").is_err());

    for entry in &manifest.files {
        let bytes = read_entry(&mut archive, &entry.path);

        assert_eq!(entry.byte_size, bytes.len() as u64);
        assert_eq!(entry.sha256, finish_digest(Sha256::new_with_prefix(&bytes)));
    }

    let media = manifest
        .files
        .iter()
        .find(|entry| entry.path.starts_with("media/"))
        .unwrap();

    assert_eq!(read_entry(&mut archive, &media.path), png_bytes());
    assert_eq!(
        archive.by_name(&media.path).unwrap().compression(),
        CompressionMethod::Stored
    );

    let snapshot_path = directory.path().join("standalone.sqlite3");

    fs::write(&snapshot_path, read_entry(&mut archive, DATABASE_PATH)).unwrap();

    let snapshot = Connection::open(&snapshot_path).unwrap();
    let stored_title: String = snapshot
        .query_row(
            "SELECT title FROM concepts WHERE entity_id = ?1",
            [&concept.id],
            |row| row.get(0),
        )
        .unwrap();

    assert_eq!(stored_title, "TCP café — 🧵");
    assert_eq!(
        snapshot
            .query_row("SELECT count(*) FROM templates", [], |row| row
                .get::<_, i64>(0))
            .unwrap(),
        1
    );
    assert_eq!(
        snapshot
            .query_row("SELECT count(*) FROM decks", [], |row| row.get::<_, i64>(0))
            .unwrap(),
        1
    );
    assert_eq!(
        snapshot
            .query_row("SELECT count(*) FROM tags", [], |row| row.get::<_, i64>(0))
            .unwrap(),
        1
    );
    assert_eq!(
        snapshot
            .query_row("SELECT count(*) FROM review_reversals", [], |row| row
                .get::<_, i64>(0))
            .unwrap(),
        1
    );
    assert_eq!(
        snapshot
            .query_row("SELECT revision FROM authoring_drafts", [], |row| row
                .get::<_, String>(0))
            .unwrap(),
        draft.revision
    );
    assert_eq!(
        snapshot
            .query_row("SELECT grading_mode FROM device_preferences", [], |row| row
                .get::<_, String>(0))
            .unwrap(),
        "advanced"
    );
    assert_eq!(
        schema_fingerprint(&snapshot).unwrap(),
        manifest.schema_fingerprint
    );
    assert_eq!(
        snapshot
            .query_row("PRAGMA integrity_check", [], |row| row.get::<_, String>(0))
            .unwrap(),
        "ok"
    );
    assert!(!snapshot
        .prepare("PRAGMA foreign_key_check")
        .unwrap()
        .exists([])
        .unwrap());

    library
        .create_concept(concept_input("Created after backup"))
        .unwrap();

    assert_eq!(
        snapshot
            .query_row("SELECT count(*) FROM concepts", [], |row| row
                .get::<_, i64>(0))
            .unwrap(),
        1
    );
    assert_eq!(
        library.device_preferences().unwrap().grading_mode,
        GradingMode::Advanced
    );
    assert_no_export_temporaries(directory.path());
}

#[test]
fn portable_export_contains_decoded_json_and_images_without_a_database() {
    let (directory, store) = test_store();
    let library = ConceptLibrary::new(&store);
    let image = library.import_image(&png_bytes()).unwrap();
    let concept = library
        .create_concept(concept_input("What does TCP guarantee?"))
        .unwrap();
    let path = directory.path().join("export.zip");

    let summary = export_library(&store, &path).unwrap();
    let mut archive = ZipArchive::new(File::open(path).unwrap()).unwrap();
    let manifest = read_manifest(&mut archive);
    let portable: Value = serde_json::from_slice(&read_entry(&mut archive, LIBRARY_PATH)).unwrap();
    let tables = &portable["tables"];

    assert_eq!(summary.kind, ArchiveKind::Export);
    assert_eq!(manifest.kind, ArchiveKind::Export);
    assert!(archive.by_name(DATABASE_PATH).is_err());
    assert_eq!(portable["format"], "twillLibrary");
    assert_eq!(portable["formatVersion"], 1);
    assert_eq!(portable["createdAt"], manifest.created_at);
    assert_eq!(tables["concepts"][0]["entity_id"], concept.id);
    assert_eq!(
        tables["concepts"][0]["content_json"]["prompt"]["type"],
        "doc"
    );
    assert_eq!(tables["cards"][0]["concept_id"], concept.id);
    assert!(tables["cards"][0]["configuration_json"].is_object());
    assert!(tables["scheduler_configurations"][0]["parameters_json"].is_array());
    assert_eq!(tables["media"][0]["entity_id"], image.id);
    assert!(tables["card_scheduling"].is_array());
    assert!(tables["device_preferences"].is_array());
    assert!(tables["authoring_drafts"].is_array());
    assert!(tables.get("concept_search").is_none());
    assert!(tables.get("authoring_media_sessions").is_none());
    assert!(tables.get("authoring_session_media").is_none());
    assert!(tables.get("device_media_cleanup").is_none());
    assert!(!tables
        .as_object()
        .unwrap()
        .keys()
        .any(|name| name.starts_with("concept_search_")));
    assert_eq!(
        manifest
            .files
            .iter()
            .filter(|entry| entry.path.starts_with("media/"))
            .count(),
        1
    );
    assert!(String::from_utf8(read_entry(&mut archive, "README.md"))
        .unwrap()
        .contains("not encrypted"));
    assert_no_export_temporaries(directory.path());
}

#[test]
fn portable_serialization_bounds_text_before_decoding() {
    let cases = [
        ("content_json", "?".repeat(5_000_001)),
        (
            "content_json",
            serde_json::to_string(&json!({ "padding": "x".repeat(5_000_000) })).unwrap(),
        ),
        ("title", "é".repeat(2_500_001)),
    ];

    for (column, value) in cases {
        let connection = Connection::open_in_memory().unwrap();

        connection
            .execute(&format!("CREATE TABLE fixture ({column} TEXT)"), [])
            .unwrap();
        connection
            .execute("INSERT INTO fixture VALUES (?1)", [&value])
            .unwrap();

        assert!(matches!(
            write_library(&connection, &mut std::io::sink(), 0),
            Err(BackupError::Integrity(_))
        ));
    }
}

#[test]
fn oversized_deleted_content_never_publishes_an_archive_or_changes_the_library() {
    let (directory, store) = test_store();
    let library = ConceptLibrary::new(&store);
    let deleted = library.create_concept(concept_input("Deleted")).unwrap();

    library.delete_concept(&deleted.id).unwrap();
    library
        .create_concept(concept_input("Keep this concept"))
        .unwrap();

    let oversized = serde_json::to_string(&json!({ "padding": "x".repeat(5_000_000) })).unwrap();

    store
        .read_result::<_, crate::data::DataError>(|connection| {
            connection.set_db_config(
                rusqlite::config::DbConfig::SQLITE_DBCONFIG_ENABLE_TRIGGER,
                false,
            )?;
            connection.execute(
                "UPDATE concepts SET content_json = ?1 WHERE entity_id = ?2",
                rusqlite::params![oversized, deleted.id],
            )?;
            connection.set_db_config(
                rusqlite::config::DbConfig::SQLITE_DBCONFIG_ENABLE_TRIGGER,
                true,
            )?;

            Ok(())
        })
        .unwrap();

    for kind in [ArchiveKind::Backup, ArchiveKind::Export] {
        let destination = directory.path().join(match kind {
            ArchiveKind::Backup => "invalid.twill",
            ArchiveKind::Export => "invalid.zip",
        });
        let result = match kind {
            ArchiveKind::Backup => create_backup(&store, &destination),
            ArchiveKind::Export => export_library(&store, &destination),
        };

        assert!(matches!(result, Err(BackupError::Integrity(_))));
        assert!(!destination.exists());
        assert_eq!(library.study_queue().unwrap().cards.len(), 1);
        assert_eq!(
            store
                .read_result::<_, crate::data::DataError>(|connection| {
                    Ok(connection.query_row(
                        "SELECT content_json FROM concepts WHERE entity_id = ?1",
                        [&deleted.id],
                        |row| row.get::<_, String>(0),
                    )?)
                })
                .unwrap(),
            oversized
        );
        assert_no_export_temporaries(directory.path());
    }
}

#[test]
fn backup_and_export_preserve_maximum_sized_authoring_drafts() {
    let (directory, store) = test_store();
    let payload = json!({ "body": "x".repeat(5_000_000 - "{\"body\":\"\"}".len()) });

    assert_eq!(serde_json::to_vec(&payload).unwrap().len(), 5_000_000);

    AuthoringDraftLibrary::new(&store)
        .upsert_draft(UpsertAuthoringDraftInput {
            kind: AuthoringDraftKind::Concept,
            target_id: None,
            schema_version: 1,
            base_change_id: None,
            payload: payload.clone(),
            media_ids: vec![],
            media_session_id: None,
        })
        .unwrap();

    for kind in [ArchiveKind::Backup, ArchiveKind::Export] {
        let destination = directory.path().join(match kind {
            ArchiveKind::Backup => "draft.twill",
            ArchiveKind::Export => "draft.zip",
        });

        match kind {
            ArchiveKind::Backup => create_backup(&store, &destination),
            ArchiveKind::Export => export_library(&store, &destination),
        }
        .unwrap();

        let mut archive = ZipArchive::new(File::open(destination).unwrap()).unwrap();
        let portable: Value =
            serde_json::from_slice(&read_entry(&mut archive, LIBRARY_PATH)).unwrap();

        assert_eq!(
            portable["tables"]["authoring_drafts"][0]["payload_json"],
            payload
        );
        assert_no_export_temporaries(directory.path());
    }
}

#[test]
fn empty_library_backup_preserves_initial_settings() {
    let (directory, store) = test_store();
    let path = directory.path().join("empty.twill");

    let summary = create_backup(&store, &path).unwrap();
    let mut archive = ZipArchive::new(File::open(path).unwrap()).unwrap();
    let portable: Value = serde_json::from_slice(&read_entry(&mut archive, LIBRARY_PATH)).unwrap();

    assert_eq!(summary.counts, LibraryCounts::default());
    assert_eq!(portable["tables"]["concepts"], json!([]));
    assert_eq!(
        portable["tables"]["device_preferences"][0]["grading_mode"],
        "simple"
    );
    assert_eq!(
        portable["tables"]["active_scheduler_configuration"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_no_export_temporaries(directory.path());
}

#[test]
fn deleted_media_tombstones_do_not_require_missing_files() {
    let (directory, store) = test_store();
    let library = ConceptLibrary::new(&store);
    let image = library.import_image(&png_bytes()).unwrap();
    let image_path = media_path(&store);
    let mut input = concept_input("Disposable image");

    input.content.prompt = json!({ "type": "doc", "content": [{
        "type": "mediaImage", "attrs": { "mediaId": image.id, "alt": "", "title": null }
    }] });

    let concept = library.create_concept(input).unwrap();
    let media_sessions = AuthoringMediaLibrary::new(&store);
    let session = media_sessions
        .begin_session(vec![image.id.clone()])
        .unwrap();

    library.delete_concept(&concept.id).unwrap();
    media_sessions.end_session(&session).unwrap();
    store.cleanup_media_files().unwrap();

    assert!(!image_path.exists());

    let path = directory.path().join("deleted.twill");
    let summary = create_backup(&store, &path).unwrap();
    let mut archive = ZipArchive::new(File::open(path).unwrap()).unwrap();
    let manifest = read_manifest(&mut archive);
    let portable: Value = serde_json::from_slice(&read_entry(&mut archive, LIBRARY_PATH)).unwrap();

    assert_eq!(summary.counts, LibraryCounts::default());
    assert!(!manifest
        .files
        .iter()
        .any(|entry| entry.path.starts_with("media/")));
    assert_eq!(portable["tables"]["media"][0]["entity_id"], image.id);
    assert!(portable["tables"]["entities"]
        .as_array()
        .unwrap()
        .iter()
        .all(|entity| !entity["deleted_at"].is_null()));
}

#[test]
fn invalid_media_never_publishes_a_backup_or_changes_the_library() {
    for missing in [false, true] {
        let (directory, store) = test_store();
        let library = ConceptLibrary::new(&store);

        library.import_image(&png_bytes()).unwrap();
        library
            .create_concept(concept_input("Keep this concept"))
            .unwrap();

        let image = media_path(&store);

        if missing {
            fs::remove_file(image).unwrap();
        } else {
            let mut bytes = png_bytes();

            bytes[0] ^= 1;
            fs::write(image, bytes).unwrap();
        }

        let destination = directory.path().join("invalid.twill");
        let result = create_backup(&store, &destination);

        assert!(result.is_err());
        assert!(!destination.exists());
        assert_eq!(library.study_queue().unwrap().cards.len(), 1);
        assert_no_export_temporaries(directory.path());
    }
}

#[test]
fn backup_rejects_existing_relative_and_in_profile_destinations() {
    let (directory, store) = test_store();
    let existing = directory.path().join("existing.twill");

    fs::write(&existing, b"Keep this existing file").unwrap();

    assert!(matches!(
        create_backup(&store, &existing),
        Err(BackupError::DestinationExists)
    ));
    assert_eq!(fs::read(existing).unwrap(), b"Keep this existing file");
    assert!(matches!(
        create_backup(&store, "relative.twill"),
        Err(BackupError::InvalidDestination)
    ));
    assert!(matches!(
        create_backup(&store, store.data_directory().join("unsafe.twill")),
        Err(BackupError::InvalidDestination)
    ));
    assert!(!store.data_directory().join("unsafe.twill").exists());
    assert_no_export_temporaries(directory.path());
}

#[test]
fn competing_exports_publish_one_complete_archive_without_overwriting() {
    let (directory, store) = test_store();
    let path = directory.path().join("race.twill");
    let barrier = Barrier::new(2);

    let results = std::thread::scope(|scope| {
        let export = || {
            barrier.wait();
            create_backup(&store, &path)
        };
        let first = scope.spawn(export);
        let second = scope.spawn(export);

        [first.join().unwrap(), second.join().unwrap()]
    });

    assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
    assert_eq!(
        results
            .iter()
            .filter(|result| matches!(result, Err(BackupError::DestinationExists)))
            .count(),
        1
    );
    assert!(ZipArchive::new(File::open(path).unwrap()).is_ok());
    assert_no_export_temporaries(directory.path());
}

#[cfg(unix)]
#[test]
fn archive_paths_and_media_cannot_follow_symlinks() {
    use std::os::unix::fs::{symlink, PermissionsExt};

    let (directory, store) = test_store();
    let alias = directory.path().join("profile-alias");

    symlink(store.data_directory(), &alias).unwrap();

    assert!(matches!(
        create_backup(&store, alias.join("unsafe.twill")),
        Err(BackupError::InvalidDestination)
    ));

    let dangling = directory.path().join("dangling.twill");

    symlink(directory.path().join("missing-file"), &dangling).unwrap();

    assert!(matches!(
        create_backup(&store, &dangling),
        Err(BackupError::DestinationExists)
    ));
    assert!(!directory.path().join("missing-file").exists());

    let valid = directory.path().join("private.twill");

    create_backup(&store, &valid).unwrap();
    assert_eq!(
        fs::metadata(valid).unwrap().permissions().mode() & 0o777,
        0o600
    );

    ConceptLibrary::new(&store)
        .import_image(&png_bytes())
        .unwrap();

    let image = media_path(&store);
    let outside = directory.path().join("outside.png");

    fs::rename(&image, &outside).unwrap();
    symlink(&outside, image).unwrap();

    let result = create_backup(&store, directory.path().join("linked-media.twill"));

    assert!(matches!(result, Err(BackupError::Integrity(_))));
    assert_eq!(fs::read(outside).unwrap(), png_bytes());
    assert_no_export_temporaries(directory.path());
}

#[test]
fn schema_fingerprint_tracks_structure_not_content() {
    let (_directory, store) = test_store();
    let fingerprint = || store.read_result(schema_fingerprint).unwrap();
    let original = fingerprint();

    ConceptLibrary::new(&store)
        .create_concept(concept_input("New content"))
        .unwrap();

    assert_eq!(fingerprint(), original);

    store
        .write(|transaction| -> DataResult<_> {
            transaction.execute("CREATE TABLE changed_shape (id TEXT) STRICT", [])?;

            Ok(())
        })
        .unwrap();

    assert_ne!(fingerprint(), original);
}

#[test]
fn streaming_checksum_stops_on_growth_and_write_failure() {
    struct Unwritable;

    impl Write for Unwritable {
        fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
            Err(std::io::Error::other("fixture write failure"))
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    let mut output = Vec::new();

    assert!(matches!(
        copy_and_digest(&mut Cursor::new(b"too large"), &mut output, 2),
        Err(BackupError::Integrity(_))
    ));
    assert!(output.is_empty());
    assert!(matches!(
        copy_and_digest(&mut Cursor::new(b"content"), &mut Unwritable, 7),
        Err(BackupError::Io(_))
    ));
    assert_eq!(
        copy_and_digest(&mut Cursor::new(b"content"), &mut output, 7).unwrap(),
        (7, finish_digest(Sha256::new_with_prefix(b"content")))
    );
    assert_eq!(output, b"content");
}
