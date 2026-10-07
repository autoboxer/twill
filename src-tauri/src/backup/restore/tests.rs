use std::fs;
use std::io::{Cursor, Read, Write};
use std::path::Path;

use image::{DynamicImage, ImageFormat};
use rusqlite::Connection;
use serde_json::json;
use tempfile::{tempdir, TempDir};
use zip::{write::SimpleFileOptions, ZipArchive, ZipWriter};

use crate::data::LocalDataStore;
use crate::library::{
    ConceptContent, ConceptLibrary, CreateConceptInput, CreateTemplateInput, RecordReviewInput,
    ReviewRating, TemplateContent, TemplateLibrary, UpdateConceptInput,
};

use super::*;
use crate::backup::checksum::finish_digest;
use crate::backup::models::ArchiveManifest;
use crate::backup::portable::{library_counts, schema_fingerprint, write_library};
use crate::backup::{create_backup, export_library, inspect_backup};
use sha2::{Digest, Sha256};

fn concept_input(title: &str) -> CreateConceptInput {
    serde_json::from_value(json!({ "title": title })).unwrap()
}

#[test]
fn backup_round_trip_preserves_assistance_media_and_assisted_reviews() {
    let root = tempdir().unwrap();
    let source = LocalDataStore::open(root.path().join("source")).unwrap();
    let library = ConceptLibrary::new(&source);
    let mut image = Cursor::new(Vec::new());

    DynamicImage::new_rgba8(4, 3).write_to(&mut image, ImageFormat::Png).unwrap();

    let media = library.import_image(image.get_ref()).unwrap();
    let mut input = concept_input("Help round trip");
    input.content.assistance.hint = json!({ "type": "doc", "content": [{
        "type": "paragraph", "content": [{ "type": "text", "text": "Think of the denominator." }]
    }] });
    input.content.assistance.reference = json!({ "type": "doc", "content": [{
        "type": "mediaImage", "attrs": { "mediaId": media.id }
    }] });
    let concept = library.create_concept(input).unwrap();
    let review = library.record_review(RecordReviewInput {
        card_id: concept.cards[0].id.clone(),
        rating: ReviewRating::Again,
        assisted: true,
    }).unwrap();
    let archive = root.path().join("help.twill");

    create_backup(&source, &archive).unwrap();

    let preview = inspect_backup(&archive).unwrap();
    let profile = root.path().join("target");
    let target = LocalDataStore::open(&profile).unwrap();

    prepare_restore(&profile, &archive, &preview.fingerprint).unwrap();
    drop(target);

    let restored = open_with_restore(&profile, || Ok(LocalDataStore::open(&profile)?)).unwrap();
    let saved = ConceptLibrary::new(&restored).concept(&concept.id).unwrap();
    let assisted: bool = restored.read_result::<_, crate::data::DataError>(|connection| {
        Ok(connection.query_row("SELECT assisted FROM reviews WHERE entity_id = ?1",
            [&review.review_id], |row| row.get(0))?)
    }).unwrap();

    assert_eq!(saved.content.assistance, concept.content.assistance);
    assert_eq!(saved.media[0].id, media.id);
    assert!(assisted);
    assert_eq!(fs::read_dir(restored.media_directory()).unwrap().count(), 1);
}

fn fixture() -> (TempDir, std::path::PathBuf) {
    fixture_with_input(|_, _, _| {})
}

fn generated_fixture() -> (TempDir, std::path::PathBuf) {
    fixture_with_input(|store, input, media_id| {
        input.include_standard_recall = false;
        input.type_answer =
            Some(serde_json::from_value(json!({ "acceptedAnswers": ["café"] })).unwrap());
        input.explain = Some(
            serde_json::from_value(json!({
                "focus": "why", "keyPoints": ["Describe the mechanism"]
            }))
            .unwrap(),
        );
        input.problem =
            Some(serde_json::from_value(json!({ "checkpoints": ["Show the reasoning"] })).unwrap());
        input.content.prompt = json!({ "type": "doc", "content": [
            { "type": "paragraph", "content": [{
                "type": "text", "text": "ATP", "marks": [{
                    "type": "cloze", "attrs": { "groupId": "0192abf7-22b3-7000-8000-000000000001" }
                }]
            }] },
            { "type": "mediaImage", "attrs": {
                "mediaId": media_id, "alt": "Cell diagram", "title": null,
                "occlusionRegions": [{
                    "id": "0192abf7-22b3-7000-8000-000000000002",
                    "groupId": "0192abf7-22b3-7000-8000-000000000003",
                    "x": 0.1, "y": 0.1, "width": 0.2, "height": 0.2
                }]
            } }
        ] });

        for name in ["First template", "Second template"] {
            let template = TemplateLibrary::new(store)
                .create_template(CreateTemplateInput {
                    name: name.to_owned(),
                    content: TemplateContent::default(),
                })
                .unwrap();

            input.template_ids.push(template.id);
        }
    })
}

fn fixture_with_input(
    configure: impl FnOnce(&LocalDataStore, &mut CreateConceptInput, &str),
) -> (TempDir, std::path::PathBuf) {
    let root = tempdir().unwrap();
    let store = LocalDataStore::open(root.path().join("source")).unwrap();
    let library = ConceptLibrary::new(&store);
    let mut image = Cursor::new(Vec::new());

    DynamicImage::new_rgba8(4, 3)
        .write_to(&mut image, ImageFormat::Png)
        .unwrap();

    let media = library.import_image(image.get_ref()).unwrap();
    let mut input = concept_input("Restore café 网络");

    input.content.prompt = json!({
        "type": "doc", "content": [{ "type": "mediaImage", "attrs": {
            "mediaId": media.id, "alt": "Diagram", "title": null
        }}]
    });

    configure(&store, &mut input, &media.id);

    let concept = library.create_concept(input).unwrap();

    library
        .record_review(RecordReviewInput {
            assisted: false,
            card_id: concept.cards[0].id.clone(),
            rating: ReviewRating::Good,
        })
        .unwrap();

    let archive = root.path().join("valid.twill");

    create_backup(&store, &archive).unwrap();

    (root, archive)
}

fn rewrite(source: &Path, destination: &Path, change: impl FnOnce(&mut Vec<(String, Vec<u8>)>)) {
    let mut input = ZipArchive::new(File::open(source).unwrap()).unwrap();
    let mut entries = Vec::new();

    for index in 0..input.len() {
        let mut entry = input.by_index(index).unwrap();
        let name = entry.name().to_owned();
        let mut bytes = Vec::new();

        entry.read_to_end(&mut bytes).unwrap();
        entries.push((name, bytes));
    }

    change(&mut entries);

    let mut output = ZipWriter::new(File::create(destination).unwrap());

    for (name, bytes) in entries {
        output
            .start_file(name, SimpleFileOptions::default())
            .unwrap();
        output.write_all(&bytes).unwrap();
    }

    output.finish().unwrap();
}

fn change_manifest(entries: &mut [(String, Vec<u8>)], change: impl FnOnce(&mut serde_json::Value)) {
    let manifest = entries
        .iter_mut()
        .find(|(name, _)| name == "manifest.json")
        .unwrap();
    let mut value = serde_json::from_slice(&manifest.1).unwrap();

    change(&mut value);
    manifest.1 = serde_json::to_vec(&value).unwrap();
}

fn rewrite_database(source: &Path, destination: &Path, change: impl FnOnce(&Connection)) {
    rewrite(source, destination, |entries| {
        let temporary = tempfile::NamedTempFile::new().unwrap();
        let database = entries
            .iter()
            .find(|(name, _)| name == "twill.sqlite3")
            .unwrap();

        fs::write(temporary.path(), &database.1).unwrap();

        let connection = Connection::open(temporary.path()).unwrap();

        connection
            .pragma_update(None, "journal_mode", "DELETE")
            .unwrap();
        change(&connection);

        let manifest_bytes = &entries
            .iter()
            .find(|(name, _)| name == "manifest.json")
            .unwrap()
            .1;
        let mut manifest: ArchiveManifest = serde_json::from_slice(manifest_bytes).unwrap();
        let mut readable = Vec::new();

        write_library(&connection, &mut readable, manifest.created_at).unwrap();
        manifest.schema_fingerprint = schema_fingerprint(&connection).unwrap();
        manifest.counts = library_counts(&connection).unwrap();

        drop(connection);

        for (name, bytes) in entries.iter_mut() {
            match name.as_str() {
                "twill.sqlite3" => *bytes = fs::read(temporary.path()).unwrap(),
                "library.json" => *bytes = readable.clone(),
                _ => continue,
            }

            let file = manifest
                .files
                .iter_mut()
                .find(|file| file.path == *name)
                .unwrap();
            let mut digest = Sha256::new();

            digest.update(&*bytes);
            file.sha256 = finish_digest(digest);
            file.byte_size = bytes.len() as u64;
        }

        entries
            .iter_mut()
            .find(|(name, _)| name == "manifest.json")
            .unwrap()
            .1 = serde_json::to_vec(&manifest).unwrap();
    });
}

#[test]
fn validated_restore_replaces_content_history_and_media_only_after_reopening() {
    let (root, archive) = fixture();
    let profile = root.path().join("target");
    let store = LocalDataStore::open(&profile).unwrap();

    ConceptLibrary::new(&store)
        .create_concept(concept_input("Original"))
        .unwrap();
    fs::create_dir_all(store.media_directory()).unwrap();
    fs::write(store.media_directory().join("old-image.png"), b"old image").unwrap();

    let preview = inspect_backup(&archive).unwrap();

    assert_eq!(preview.counts.concepts, 1);
    assert_eq!(preview.counts.media, 1);
    assert_eq!(preview.counts.reviews, 1);

    prepare_restore(&profile, &archive, &preview.fingerprint).unwrap();

    assert!(restore_pending(&profile).unwrap());
    assert_eq!(
        ConceptLibrary::new(&store)
            .search(Default::default())
            .unwrap()
            .concepts[0]
            .title,
        "Original"
    );

    drop(store);

    let restored = open_with_restore(&profile, || Ok(LocalDataStore::open(&profile)?)).unwrap();
    let concepts = ConceptLibrary::new(&restored)
        .search(Default::default())
        .unwrap()
        .concepts;

    assert_eq!(concepts[0].title, "Restore café 网络");
    assert_eq!(
        restored
            .read_result::<_, crate::data::DataError>(|connection| {
                Ok(
                    connection.query_row("SELECT count(*) FROM reviews", [], |row| {
                        row.get::<_, i64>(0)
                    })?,
                )
            })
            .unwrap(),
        1
    );
    assert_eq!(fs::read_dir(restored.media_directory()).unwrap().count(), 1);
    assert!(!restored.media_directory().join("old-image.png").exists());
    assert!(!restore_pending(&profile).unwrap());

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        assert_eq!(
            fs::metadata(profile.join("twill.sqlite3"))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
        assert_eq!(
            fs::metadata(restored.media_directory())
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o700
        );
        let image = fs::read_dir(restored.media_directory())
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path();
        assert_eq!(
            fs::metadata(image).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }
}

#[test]
fn bad_metadata_checksums_and_unsafe_paths_never_prepare_a_replacement() {
    let (root, archive) = fixture();
    let profile = root.path().join("target");
    let store = LocalDataStore::open(&profile).unwrap();
    let before = fs::read(profile.join("twill.sqlite3")).unwrap();

    for case in 0..10 {
        let invalid = root.path().join(format!("invalid-{case}.twill"));

        rewrite(&archive, &invalid, |entries| match case {
            0 => change_manifest(entries, |value| value["formatVersion"] = json!(2)),
            1 => change_manifest(entries, |value| value["schemaVersion"] = json!(2)),
            2 => change_manifest(entries, |value| value["counts"]["concepts"] = json!(100)),
            3 => change_manifest(entries, |value| value["unknown"] = json!(true)),
            4 => change_manifest(entries, |value| {
                let first = value["files"][0].clone();
                value["files"].as_array_mut().unwrap().push(first);
            }),
            5 => change_manifest(entries, |value| {
                value["files"][0]["byteSize"] = json!(u64::MAX)
            }),
            6 => entries.push(("../escaped".to_owned(), b"escape".to_vec())),
            7 => entries.push(("media\\escape.png".to_owned(), b"escape".to_vec())),
            8 => entries
                .iter_mut()
                .find(|(name, _)| name == "README.md")
                .unwrap()
                .1
                .push(b'!'),
            9 => entries.retain(|(name, _)| !name.starts_with("media/")),
            _ => unreachable!(),
        });

        assert!(
            prepare_restore(&profile, &invalid, "unused").is_err(),
            "case {case}"
        );
        assert!(!restore_pending(&profile).unwrap());
        assert_eq!(fs::read(profile.join("twill.sqlite3")).unwrap(), before);
        assert!(!root.path().join("escaped").exists());
    }

    drop(store);
}

#[test]
fn schema_changes_foreign_key_errors_and_constraint_violations_are_rejected() {
    let (root, archive) = fixture();

    for case in 0..3 {
        let invalid = root.path().join(format!("database-{case}.twill"));

        rewrite_database(&archive, &invalid, |connection| {
            match case {
            0 => connection.execute_batch("ALTER TABLE concepts ADD COLUMN unexpected TEXT;").unwrap(),
            1 => connection.execute_batch(
                "PRAGMA foreign_keys = OFF; UPDATE active_scheduler_configuration SET configuration_id = 'missing';",
            ).unwrap(),
            2 => connection.execute_batch(
                "PRAGMA ignore_check_constraints = ON; UPDATE device_preferences SET grading_mode = 'invalid';",
            ).unwrap(),
            _ => unreachable!(),
        }
        });

        let error = inspect_backup(invalid).expect_err(&format!("invalid database case {case}"));

        if case == 0 {
            assert!(matches!(error, BackupError::IncompatibleArchive));
        } else {
            assert!(matches!(error, BackupError::InvalidArchive(_)), "{error}");
        }
    }
}

#[test]
fn invalid_rich_content_and_card_configuration_are_rejected_even_with_valid_checksums() {
    let (root, archive) = fixture();

    for case in 0..2 {
        let invalid = root.path().join(format!("content-{case}.twill"));

        rewrite_database(&archive, &invalid, |connection| {
            connection
                .set_db_config(
                    rusqlite::config::DbConfig::SQLITE_DBCONFIG_ENABLE_TRIGGER,
                    false,
                )
                .unwrap();
            let sql = if case == 0 {
                "UPDATE concepts SET content_json = json_set(content_json, '$.schemaVersion', 99)"
            } else {
                "UPDATE cards SET configuration_json = '{\"unexpected\": true}'"
            };

            connection.execute_batch(sql).unwrap();
        });

        assert!(matches!(
            inspect_backup(invalid),
            Err(BackupError::InvalidArchive(_))
        ));
    }
}

#[test]
fn oversized_card_configuration_is_rejected_before_parsing() {
    let (root, archive) = fixture();
    let invalid = root.path().join("oversized-configuration.twill");

    rewrite_database(&archive, &invalid, |connection| {
        connection
            .set_db_config(
                rusqlite::config::DbConfig::SQLITE_DBCONFIG_ENABLE_TRIGGER,
                false,
            )
            .unwrap();
        connection
            .execute(
                "UPDATE cards SET configuration_json = ?1",
                [format!("{}{{}}", " ".repeat(4_000_000))],
            )
            .unwrap();
    });

    assert!(matches!(
        inspect_backup(invalid),
        Err(BackupError::InvalidArchive(_))
    ));
}

#[test]
fn restored_media_links_must_exactly_match_document_media() {
    let (root, archive) = fixture();
    let invalid = root.path().join("extra-media-link.twill");

    rewrite_database(&archive, &invalid, |connection| {
        connection
            .set_db_config(
                rusqlite::config::DbConfig::SQLITE_DBCONFIG_ENABLE_TRIGGER,
                false,
            )
            .unwrap();
        connection
            .execute(
                "UPDATE concepts SET content_json = json_set(content_json, '$.prompt', json(?1))",
                [serde_json::to_string(&ConceptContent::default().prompt).unwrap()],
            )
            .unwrap();
    });

    assert!(matches!(
        inspect_backup(invalid),
        Err(BackupError::InvalidArchive(_))
    ));
}

#[test]
fn generated_retrieval_cards_must_match_the_concept() {
    let (root, archive) = generated_fixture();
    let cases = [
        ("unknown-cloze-group", "UPDATE cards SET configuration_json = '{\"groupId\":\"0192abf7-22b3-7000-8000-000000000099\"}' WHERE retrieval_kind = 'cloze'"),
        ("unknown-occlusion-group", "UPDATE cards SET configuration_json = '{\"groupId\":\"0192abf7-22b3-7000-8000-000000000099\"}' WHERE retrieval_kind = 'image_occlusion'"),
        ("missing-cloze-card", "UPDATE entities SET deleted_at = updated_at WHERE id IN (SELECT entity_id FROM cards WHERE retrieval_kind = 'cloze')"),
        ("missing-occlusion-card", "UPDATE entities SET deleted_at = updated_at WHERE id IN (SELECT entity_id FROM cards WHERE retrieval_kind = 'image_occlusion')"),
        ("duplicate-cloze-card", "UPDATE cards SET retrieval_kind = 'cloze', configuration_json = (SELECT configuration_json FROM cards WHERE retrieval_kind = 'cloze') WHERE retrieval_kind = 'type_answer'"),
        ("duplicate-occlusion-card", "UPDATE cards SET retrieval_kind = 'image_occlusion', configuration_json = (SELECT configuration_json FROM cards WHERE retrieval_kind = 'image_occlusion') WHERE retrieval_kind = 'type_answer'"),
    ];

    for (name, sql) in cases {
        let invalid = root.path().join(format!("{name}.twill"));

        rewrite_database(&archive, &invalid, |connection| {
            connection
                .set_db_config(
                    rusqlite::config::DbConfig::SQLITE_DBCONFIG_ENABLE_TRIGGER,
                    false,
                )
                .unwrap();
            connection.execute_batch(sql).unwrap();
        });

        let error = inspect_backup(invalid).expect_err(name);

        assert!(
            matches!(error, BackupError::InvalidArchive(_)),
            "{name}: {error}"
        );
    }
}

#[test]
fn active_retrieval_forms_require_valid_parents_and_selections() {
    let (root, archive) = generated_fixture();
    let cases = [
        ("no-active-cards", "UPDATE entities SET deleted_at = updated_at WHERE kind = 'card'"),
        ("deleted-concept", "UPDATE entities SET deleted_at = updated_at WHERE kind = 'concept'"),
        ("deleted-template", "UPDATE entities SET deleted_at = updated_at WHERE kind = 'template'"),
        ("duplicate-template-card", "UPDATE cards SET retrieval_kind = 'recall', configuration_json = '{}', template_id = (SELECT template_id FROM cards WHERE template_id IS NOT NULL LIMIT 1) WHERE retrieval_kind = 'type_answer'"),
        ("duplicate-explain-card", "UPDATE cards SET retrieval_kind = 'explain', configuration_json = (SELECT configuration_json FROM cards WHERE retrieval_kind = 'explain') WHERE retrieval_kind = 'type_answer'"),
    ];

    for (name, sql) in cases {
        let invalid = root.path().join(format!("{name}.twill"));

        rewrite_database(&archive, &invalid, |connection| {
            connection
                .set_db_config(
                    rusqlite::config::DbConfig::SQLITE_DBCONFIG_ENABLE_TRIGGER,
                    false,
                )
                .unwrap();
            connection.execute_batch(sql).unwrap();
        });

        let error = inspect_backup(invalid).expect_err(name);

        assert!(
            matches!(error, BackupError::InvalidArchive(_)),
            "{name}: {error}"
        );
    }
}

#[test]
fn problem_cards_require_nonempty_prompts_after_restore() {
    let (root, archive) = fixture();
    let invalid = root.path().join("empty-problem.twill");

    rewrite_database(&archive, &invalid, |connection| {
        connection
            .set_db_config(
                rusqlite::config::DbConfig::SQLITE_DBCONFIG_ENABLE_TRIGGER,
                false,
            )
            .unwrap();
        connection
            .execute(
                "UPDATE concepts SET content_json = json_set(content_json, '$.prompt', json(?1))",
                [serde_json::to_string(&ConceptContent::default().prompt).unwrap()],
            )
            .unwrap();
        connection
            .execute_batch(
                "UPDATE concept_media SET removed_at = updated_at;
                UPDATE cards SET retrieval_kind = 'problem', configuration_json = '{\"checkpoints\":[\"Show the reasoning\"]}';",
            )
            .unwrap();
    });

    assert!(matches!(
        inspect_backup(invalid),
        Err(BackupError::InvalidArchive(_))
    ));
}

#[test]
fn valid_retrieval_forms_and_historical_links_restore() {
    let (root, archive) = generated_fixture();

    assert_eq!(inspect_backup(&archive).unwrap().counts.cards, 7);

    let store = LocalDataStore::open(root.path().join("source")).unwrap();
    let library = ConceptLibrary::new(&store);
    let concept = library
        .search(Default::default())
        .unwrap()
        .concepts
        .remove(0);
    let concept = library.concept(&concept.id).unwrap();
    let updated = library
        .update_concept(serde_json::from_value::<UpdateConceptInput>(json!({
            "id": concept.id,
            "title": concept.title,
            "content": ConceptContent::default(),
            "includeStandardRecall": false,
            "templateIds": concept.cards.iter().filter_map(|card| card.template.as_ref().map(|template| &template.id)).collect::<Vec<_>>(),
            "typeAnswer": { "acceptedAnswers": ["café"] },
            "explain": { "focus": "why", "keyPoints": ["Describe the mechanism"] }
        })).unwrap())
        .unwrap();

    library.set_concept_archived(&updated.id, true).unwrap();

    let deleted = library
        .create_concept(concept_input("Deleted concept"))
        .unwrap();

    library.delete_concept(&deleted.id).unwrap();

    let unused = TemplateLibrary::new(&store)
        .create_template(CreateTemplateInput {
            name: "Deleted template".to_owned(),
            content: TemplateContent::default(),
        })
        .unwrap();

    TemplateLibrary::new(&store)
        .delete_template(&unused.id)
        .unwrap();

    let historical = root.path().join("historical.twill");

    create_backup(&store, &historical).unwrap();

    let preview = inspect_backup(&historical).unwrap();

    assert_eq!(preview.counts.concepts, 1);
    assert_eq!(preview.counts.cards, 4);
    assert_eq!(preview.counts.reviews, 1);
}

#[test]
fn duplicate_zip_entries_symlinks_and_oversized_metadata_are_rejected() {
    let (root, archive) = fixture();
    let bytes = fs::read(&archive).unwrap();
    let end = bytes.len() - 22;

    assert_eq!(&bytes[end..end + 4], b"PK\x05\x06");

    let count = u16::from_le_bytes(bytes[end + 10..end + 12].try_into().unwrap());
    let size = u32::from_le_bytes(bytes[end + 12..end + 16].try_into().unwrap());
    let start = u32::from_le_bytes(bytes[end + 16..end + 20].try_into().unwrap()) as usize;
    let field = |offset| {
        u16::from_le_bytes(
            bytes[start + offset..start + offset + 2]
                .try_into()
                .unwrap(),
        ) as usize
    };
    let entry_size = 46 + field(28) + field(30) + field(32);
    let mut duplicate = bytes[..end].to_vec();

    duplicate.extend_from_slice(&bytes[start..start + entry_size]);
    duplicate.extend_from_slice(&bytes[end..]);

    let end = end + entry_size;

    duplicate[end + 8..end + 10].copy_from_slice(&(count + 1).to_le_bytes());
    duplicate[end + 10..end + 12].copy_from_slice(&(count + 1).to_le_bytes());
    duplicate[end + 12..end + 16].copy_from_slice(&(size + entry_size as u32).to_le_bytes());

    let path = root.path().join("duplicate.twill");

    fs::write(&path, duplicate).unwrap();

    assert!(matches!(
        inspect_backup(path),
        Err(BackupError::InvalidArchive(_))
    ));

    let mut symlink = bytes.clone();

    symlink[start + 5] = 3;
    symlink[start + 38..start + 42].copy_from_slice(&(0o120777_u32 << 16).to_le_bytes());

    let path = root.path().join("symlink.twill");

    fs::write(&path, symlink).unwrap();

    assert!(matches!(
        inspect_backup(path),
        Err(BackupError::InvalidArchive(_))
    ));

    let mut oversized = bytes;

    oversized[start + 28..start + 30].copy_from_slice(&u16::MAX.to_le_bytes());

    let path = root.path().join("oversized.twill");

    fs::write(&path, oversized).unwrap();

    assert!(matches!(
        inspect_backup(path),
        Err(BackupError::InvalidArchive(_))
    ));
}

#[test]
fn backup_and_restore_share_expanded_file_size_limits() {
    let oversized = crate::backup::models::ArchiveFile {
        path: "twill.sqlite3".to_owned(),
        byte_size: 2 * 1024 * 1024 * 1024 + 1,
        sha256: "0".repeat(64),
    };

    assert!(matches!(
        crate::backup::validation::check_size_limits(&[oversized]),
        Err(BackupError::ArchiveLimit)
    ));
}

#[test]
fn portable_exports_changed_previews_and_competing_restores_are_rejected() {
    let (root, archive) = fixture();
    let profile = root.path().join("target");
    let store = LocalDataStore::open(&profile).unwrap();
    let export = root.path().join("portable.zip");

    export_library(&store, &export).unwrap();

    assert!(matches!(
        inspect_backup(export),
        Err(BackupError::InvalidArchive(_))
    ));
    assert!(prepare_restore(&profile, &archive, "changed").is_err());

    let preview = inspect_backup(&archive).unwrap();

    prepare_restore(&profile, &archive, &preview.fingerprint).unwrap();

    assert!(matches!(
        prepare_restore(&profile, &archive, &preview.fingerprint),
        Err(BackupError::RestorePending)
    ));

    cancel_restore(&profile).unwrap();

    assert!(!restore_pending(&profile).unwrap());
    assert_eq!(
        ConceptLibrary::new(&store)
            .search(Default::default())
            .unwrap()
            .concepts
            .len(),
        0
    );
}

#[test]
fn staged_tampering_and_failed_new_profile_open_preserve_the_original() {
    let (root, archive) = fixture();
    let profile = root.path().join("target");
    let store = LocalDataStore::open(&profile).unwrap();

    ConceptLibrary::new(&store)
        .create_concept(concept_input("Original"))
        .unwrap();

    drop(store);

    let original = fs::read(profile.join("twill.sqlite3")).unwrap();
    let preview = inspect_backup(&archive).unwrap();

    prepare_restore(&profile, &archive, &preview.fingerprint).unwrap();
    fs::write(
        profile.join(PENDING_DIRECTORY).join("new/library.json"),
        b"changed",
    )
    .unwrap();

    assert!(open_with_restore(&profile, || Ok(())).is_err());
    assert_eq!(fs::read(profile.join("twill.sqlite3")).unwrap(), original);

    cancel_restore(&profile).unwrap();
    prepare_restore(&profile, &archive, &preview.fingerprint).unwrap();

    assert!(open_with_restore::<()>(&profile, || Err(
        io::Error::other("injected open failure").into()
    ))
    .is_err());
    assert_eq!(fs::read(profile.join("twill.sqlite3")).unwrap(), original);
    assert!(!restore_pending(&profile).unwrap());

    let reopened = LocalDataStore::open(&profile).unwrap();

    assert_eq!(
        ConceptLibrary::new(&reopened)
            .search(Default::default())
            .unwrap()
            .concepts[0]
            .title,
        "Original"
    );
}

#[test]
fn each_interrupted_move_rolls_back_database_sidecars_and_media() {
    let (root, archive) = fixture();
    let preview = inspect_backup(&archive).unwrap();

    for completed in 0..=6 {
        let profile = root.path().join(format!("interrupted-{completed}"));
        let store = LocalDataStore::open(&profile).unwrap();

        drop(store);

        fs::write(profile.join("twill.sqlite3-wal"), b"original WAL").unwrap();
        fs::write(profile.join("twill.sqlite3-shm"), b"original SHM").unwrap();
        fs::create_dir(profile.join("media")).unwrap();
        fs::write(profile.join("media/original.png"), b"original image").unwrap();

        let original = fs::read(profile.join("twill.sqlite3")).unwrap();

        prepare_restore(&profile, &archive, &preview.fingerprint).unwrap();

        let pending = profile.join(PENDING_DIRECTORY);
        let journal = Journal {
            version: 1,
            phase: Phase::Installing,
            originals: PROFILE_ITEMS
                .iter()
                .map(|name| (*name).to_owned())
                .collect(),
        };

        write_journal(&pending, &journal).unwrap();

        for (index, name) in PROFILE_ITEMS.iter().enumerate() {
            if index < completed {
                move_item(&profile.join(name), &pending.join("old").join(name)).unwrap();
            }
        }

        for (index, name) in ["twill.sqlite3", "media"].iter().enumerate() {
            if index + 4 < completed {
                move_item(&pending.join("new").join(name), &profile.join(name)).unwrap();
            }
        }

        assert!(matches!(
            open_with_restore(&profile, || Ok(())),
            Err(BackupError::RestoreInterrupted)
        ));
        assert_eq!(
            fs::read(profile.join("twill.sqlite3")).unwrap(),
            original,
            "step {completed}"
        );
        assert_eq!(
            fs::read(profile.join("twill.sqlite3-wal")).unwrap(),
            b"original WAL"
        );
        assert_eq!(
            fs::read(profile.join("twill.sqlite3-shm")).unwrap(),
            b"original SHM"
        );
        assert_eq!(
            fs::read(profile.join("media/original.png")).unwrap(),
            b"original image"
        );
        assert!(!restore_pending(&profile).unwrap());
    }
}

#[test]
fn completed_restore_cleanup_is_not_needed_to_open_the_new_profile() {
    let (root, archive) = fixture();
    let profile = root.path().join("target");
    let store = LocalDataStore::open(&profile).unwrap();

    drop(store);

    let preview = inspect_backup(&archive).unwrap();

    prepare_restore(&profile, &archive, &preview.fingerprint).unwrap();

    let pending = profile.join(PENDING_DIRECTORY);

    move_item(
        &profile.join("twill.sqlite3"),
        &pending.join("old/twill.sqlite3"),
    )
    .unwrap();
    move_item(
        &pending.join("new/twill.sqlite3"),
        &profile.join("twill.sqlite3"),
    )
    .unwrap();
    move_item(&pending.join("new/media"), &profile.join("media")).unwrap();
    write_journal(
        &pending,
        &Journal {
            version: 1,
            phase: Phase::Committed,
            originals: vec!["twill.sqlite3".to_owned()],
        },
    )
    .unwrap();

    let store = open_with_restore(&profile, || Ok(LocalDataStore::open(&profile)?)).unwrap();

    assert_eq!(
        ConceptLibrary::new(&store)
            .search(Default::default())
            .unwrap()
            .concepts[0]
            .title,
        "Restore café 网络"
    );
    assert!(!restore_pending(&profile).unwrap());
}

#[cfg(unix)]
#[test]
fn symlinks_are_not_followed_during_installation_or_cleanup() {
    use std::os::unix::fs::symlink;

    let (root, archive) = fixture();
    let profile = root.path().join("target");
    let store = LocalDataStore::open(&profile).unwrap();

    drop(store);

    let external = root.path().join("external");

    fs::create_dir(&external).unwrap();
    fs::write(external.join("keep"), b"keep").unwrap();
    symlink(&external, profile.join("media")).unwrap();

    let preview = inspect_backup(&archive).unwrap();

    prepare_restore(&profile, &archive, &preview.fingerprint).unwrap();

    assert!(open_with_restore(&profile, || Ok(())).is_err());
    assert_eq!(fs::read(external.join("keep")).unwrap(), b"keep");

    cancel_restore(&profile).unwrap();

    let cleanup = profile.join("owned-cleanup");

    fs::create_dir(&cleanup).unwrap();
    symlink(&external, cleanup.join("link")).unwrap();

    assert!(remove_tree(&cleanup).is_err());
    assert_eq!(fs::read(external.join("keep")).unwrap(), b"keep");
}
