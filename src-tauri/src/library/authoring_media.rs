use std::collections::BTreeSet;

use rusqlite::{params, Connection};
use uuid::Uuid;

use crate::data::{LocalDataStore, WriteTransaction};
use crate::library::authoring_drafts::{validate_media, MAXIMUM_DRAFT_MEDIA};
use crate::library::content::MAXIMUM_DOCUMENT_NODES;
use crate::library::media::release_unreferenced_media;
use crate::library::{LibraryError, LibraryResult};

// Each saved content node can reference one image, in addition to recovered draft media
const MAXIMUM_INITIAL_MEDIA: usize = MAXIMUM_DOCUMENT_NODES + MAXIMUM_DRAFT_MEDIA;

pub struct AuthoringMediaLibrary<'store> {
    store: &'store LocalDataStore,
}

impl<'store> AuthoringMediaLibrary<'store> {
    pub fn new(store: &'store LocalDataStore) -> Self {
        Self { store }
    }

    pub fn begin_session(&self, media_ids: Vec<String>) -> LibraryResult<String> {
        let media_ids = normalize_initial_media(media_ids)?;
        let session_id = Uuid::now_v7().to_string();

        self.store.write_result(|transaction| {
            validate_media(transaction, &media_ids)?;
            transaction.execute(
                "INSERT INTO authoring_media_sessions (id) VALUES (?1)",
                [&session_id],
            )?;
            retain_session_media(transaction, &session_id, &media_ids)?;

            Ok(session_id)
        })
    }

    pub fn end_session(&self, session_id: &str) -> LibraryResult<()> {
        self.store.write_result(|transaction| {
            let mut statement = transaction
                .prepare("SELECT media_id FROM authoring_session_media WHERE session_id = ?1")?;
            let media_ids = statement
                .query_map([session_id], |row| row.get::<_, String>(0))?
                .collect::<Result<Vec<_>, _>>()?;

            transaction.execute(
                "DELETE FROM authoring_media_sessions WHERE id = ?1",
                [session_id],
            )?;

            release_unreferenced_media(transaction, &media_ids)
        })
    }

    pub fn release_abandoned_sessions(&self) -> LibraryResult<()> {
        // Run once at native startup, before any editor can acquire a session
        self.store.write_result::<_, LibraryError>(|transaction| {
            let mut statement =
                transaction.prepare("SELECT DISTINCT media_id FROM authoring_session_media")?;
            let media_ids = statement
                .query_map([], |row| row.get::<_, String>(0))?
                .collect::<Result<Vec<_>, _>>()?;

            transaction.execute("DELETE FROM authoring_media_sessions", [])?;
            release_unreferenced_media(transaction, &media_ids)
        })?;

        self.store.cleanup_media_files()?;

        Ok(())
    }
}

fn normalize_initial_media(media_ids: Vec<String>) -> LibraryResult<BTreeSet<String>> {
    if media_ids.len() > MAXIMUM_INITIAL_MEDIA {
        return Err(LibraryError::InvalidContent {
            field: "Image editing session",
            message: format!("cannot start with more than {MAXIMUM_INITIAL_MEDIA} media items"),
        });
    }

    media_ids
        .into_iter()
        .map(|id| {
            let id = id.trim().to_owned();

            if Uuid::parse_str(&id).is_err() {
                return Err(LibraryError::InvalidContent {
                    field: "Image editing session",
                    message: "contains an invalid media ID".to_owned(),
                });
            }

            Ok(id)
        })
        .collect()
}

pub(super) fn validate_session(connection: &Connection, session_id: &str) -> LibraryResult<()> {
    let exists: bool = connection.query_row(
        "SELECT EXISTS (SELECT 1 FROM authoring_media_sessions WHERE id = ?1)",
        [session_id],
        |row| row.get(0),
    )?;

    if !exists {
        return Err(LibraryError::AuthoringMediaSessionClosed);
    }

    Ok(())
}

pub(super) fn retain_session_media(
    transaction: &WriteTransaction<'_>,
    session_id: &str,
    media_ids: &BTreeSet<String>,
) -> LibraryResult<()> {
    validate_session(transaction, session_id)?;

    for media_id in media_ids {
        transaction.execute(
            "INSERT OR IGNORE INTO authoring_session_media (session_id, media_id)
            VALUES (?1, ?2)",
            params![session_id, media_id],
        )?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use image::{DynamicImage, ImageFormat};
    use serde_json::json;
    use tempfile::tempdir;

    use super::*;
    use crate::data::{current_timestamp, EntityKind};
    use crate::library::media::{import_image, import_image_with_session, read_media};
    use crate::library::{
        AuthoringDraftKind, AuthoringDraftLibrary, AuthoringDraftLocator, ConceptContent,
        ConceptLibrary, CreateConceptInput, UpsertAuthoringDraftInput,
    };

    fn png_bytes(width: u32) -> Vec<u8> {
        let mut bytes = Cursor::new(Vec::new());

        DynamicImage::new_rgba8(width, 3)
            .write_to(&mut bytes, ImageFormat::Png)
            .unwrap();

        bytes.into_inner()
    }

    fn save_draft(store: &LocalDataStore, session_id: &str, media_ids: Vec<String>) {
        AuthoringDraftLibrary::new(store)
            .upsert_draft(UpsertAuthoringDraftInput {
                kind: AuthoringDraftKind::Concept,
                target_id: None,
                schema_version: 1,
                base_change_id: None,
                payload: json!({ "title": "Working draft" }),
                media_ids,
                media_session_id: Some(session_id.to_owned()),
            })
            .unwrap();
    }

    #[test]
    fn session_retains_the_full_saved_concept_and_draft_union() {
        let directory = tempdir().unwrap();
        let store = LocalDataStore::open(directory.path()).unwrap();
        let media_ids = store
            .write_result::<_, LibraryError>(|transaction| {
                let mut ids = Vec::new();

                // Ownership validation does not need physical image files
                for index in 0..2_001 {
                    let media = transaction.create_entity(EntityKind::Media)?;

                    transaction.execute(
                        "INSERT INTO media (
                            entity_id, digest, mime_type, file_extension,
                            byte_size, width, height, last_change_id
                        ) VALUES (?1, ?2, 'image/png', 'png', 1, 1, 1, ?3)",
                        params![media.id, format!("{index:064x}"), media.last_change_id],
                    )?;
                    ids.push(media.id);
                }

                Ok(ids)
            })
            .unwrap();
        let saved_ids = &media_ids[..1_001];
        let draft_ids = &media_ids[1_001..];
        let prompt = |ids: &[String]| {
            let images = ids
                .iter()
                .map(|id| json!({ "type": "mediaImage", "attrs": { "mediaId": id } }))
                .collect::<Vec<_>>();

            json!({ "type": "doc", "content": images })
        };
        let concepts = ConceptLibrary::new(&store);
        let concept = concepts
            .create_concept(CreateConceptInput {
                title: "Large saved concept".to_owned(),
                content: ConceptContent {
                    prompt: prompt(saved_ids),
                    ..ConceptContent::default()
                },
                deck_ids: vec![],
                tag_ids: vec![],
                include_standard_recall: true,
                template_ids: vec![],
                explain: None,
                problem: None,
                type_answer: None,
            })
            .unwrap();
        let drafts = AuthoringDraftLibrary::new(&store);
        let mut input = UpsertAuthoringDraftInput {
            kind: AuthoringDraftKind::Concept,
            target_id: Some(concept.id.clone()),
            schema_version: 1,
            base_change_id: Some(concept.last_change_id.clone()),
            payload: json!({ "content": ConceptContent {
                prompt: prompt(draft_ids),
                ..ConceptContent::default()
            } }),
            media_ids: draft_ids.to_vec(),
            media_session_id: None,
        };
        let draft = drafts.upsert_draft(input.clone()).unwrap();
        let mut initial_media = concept
            .media
            .iter()
            .map(|media| media.id.clone())
            .chain(draft.media_ids)
            .collect::<Vec<_>>();

        initial_media.push(format!(" {} ", saved_ids[0]));
        initial_media.push(draft_ids[0].clone());

        let sessions = AuthoringMediaLibrary::new(&store);
        let session = sessions.begin_session(initial_media).unwrap();

        let retained: i64 = store
            .read_result::<_, LibraryError>(|connection| {
                connection
                    .query_row(
                        "SELECT COUNT(*) FROM authoring_session_media WHERE session_id = ?1",
                        [&session],
                        |row| row.get(0),
                    )
                    .map_err(Into::into)
            })
            .unwrap();

        assert_eq!(retained, 2_001);

        // A session's larger union does not raise the per-draft limit
        input.media_ids.push(saved_ids[0].clone());
        assert!(matches!(
            drafts.upsert_draft(input),
            Err(LibraryError::InvalidAuthoringDraft { .. })
        ));

        drafts
            .delete_draft(AuthoringDraftLocator {
                kind: AuthoringDraftKind::Concept,
                target_id: Some(concept.id),
            })
            .unwrap();
        assert!(store
            .entity(&draft_ids[0])
            .unwrap()
            .unwrap()
            .deleted_at
            .is_none());

        sessions.end_session(&session).unwrap();

        assert!(store
            .entity(&saved_ids[0])
            .unwrap()
            .unwrap()
            .deleted_at
            .is_none());
        assert!(store
            .entity(&draft_ids[0])
            .unwrap()
            .unwrap()
            .deleted_at
            .is_some());
    }

    #[test]
    fn initial_media_validation_is_bounded_and_rejects_invalid_or_missing_ids() {
        let ids = (0..MAXIMUM_INITIAL_MEDIA)
            .map(|_| Uuid::now_v7().to_string())
            .collect::<Vec<_>>();

        assert_eq!(
            normalize_initial_media(ids.clone()).unwrap().len(),
            MAXIMUM_INITIAL_MEDIA
        );

        let mut oversized = ids;
        oversized.push(Uuid::now_v7().to_string());
        assert!(matches!(
            normalize_initial_media(oversized),
            Err(LibraryError::InvalidContent { .. })
        ));
        assert!(matches!(
            normalize_initial_media(vec!["not a UUID".to_owned()]),
            Err(LibraryError::InvalidContent { .. })
        ));

        let directory = tempdir().unwrap();
        let store = LocalDataStore::open(directory.path()).unwrap();
        let missing = Uuid::now_v7().to_string();

        assert!(matches!(
            AuthoringMediaLibrary::new(&store).begin_session(vec![missing.clone()]),
            Err(LibraryError::MediaNotFound(id)) if id == missing
        ));

        let sessions: i64 = store
            .read_result::<_, LibraryError>(|connection| {
                connection
                    .query_row("SELECT COUNT(*) FROM authoring_media_sessions", [], |row| {
                        row.get(0)
                    })
                    .map_err(Into::into)
            })
            .unwrap();

        assert_eq!(sessions, 0);
    }

    #[test]
    fn imports_and_earlier_snapshots_stay_readable_until_editing_ends() {
        let directory = tempdir().unwrap();
        let store = LocalDataStore::open(directory.path()).unwrap();
        let library = AuthoringMediaLibrary::new(&store);
        let session = library.begin_session(vec![]).unwrap();
        let bytes = png_bytes(2);
        let media = import_image_with_session(&store, &bytes, Some(&session)).unwrap();
        let pasted_media = import_image(&store, &png_bytes(3)).unwrap();

        assert_eq!(read_media(&store, &media.id).unwrap(), bytes);
        save_draft(
            &store,
            &session,
            vec![media.id.clone(), pasted_media.id.clone()],
        );
        save_draft(&store, &session, vec![]);
        assert_eq!(read_media(&store, &media.id).unwrap(), bytes);
        assert!(read_media(&store, &pasted_media.id).is_ok());

        // Undo can restore the same identity after an autosaved removal
        save_draft(&store, &session, vec![media.id.clone()]);
        AuthoringDraftLibrary::new(&store)
            .delete_draft(AuthoringDraftLocator {
                kind: AuthoringDraftKind::Concept,
                target_id: None,
            })
            .unwrap();

        assert_eq!(read_media(&store, &media.id).unwrap(), bytes);
        library.end_session(&session).unwrap();
        assert!(matches!(
            read_media(&store, &media.id),
            Err(LibraryError::MediaNotFound(_))
        ));
        assert!(matches!(
            read_media(&store, &pasted_media.id),
            Err(LibraryError::MediaNotFound(_))
        ));
        let change_count = store.changes_after(0, 100).unwrap().len();

        library.end_session(&session).unwrap();
        assert_eq!(store.changes_after(0, 100).unwrap().len(), change_count);
    }

    #[test]
    fn snapshot_media_and_deduplicated_imports_are_shared_across_sessions() {
        let directory = tempdir().unwrap();
        let store = LocalDataStore::open(directory.path()).unwrap();
        let library = AuthoringMediaLibrary::new(&store);
        let media = import_image(&store, &png_bytes(2)).unwrap();
        let first = library.begin_session(vec![media.id.clone()]).unwrap();
        let second = library.begin_session(vec![]).unwrap();
        let duplicate = import_image_with_session(&store, &png_bytes(2), Some(&second)).unwrap();

        assert_eq!(duplicate.id, media.id);
        assert_eq!(store.changes_after(0, 100).unwrap().len(), 1);
        library.end_session(&first).unwrap();
        assert!(read_media(&store, &media.id).is_ok());
        library.end_session(&second).unwrap();
        assert!(matches!(
            read_media(&store, &media.id),
            Err(LibraryError::MediaNotFound(_))
        ));
    }

    #[test]
    fn closed_sessions_reject_late_imports_before_writing_files() {
        let directory = tempdir().unwrap();
        let store = LocalDataStore::open(directory.path()).unwrap();
        let library = AuthoringMediaLibrary::new(&store);
        let session = library.begin_session(vec![]).unwrap();

        library.end_session(&session).unwrap();

        assert!(matches!(
            import_image_with_session(&store, &png_bytes(2), Some(&session)),
            Err(LibraryError::AuthoringMediaSessionClosed)
        ));
        assert!(!store.media_directory().exists());
        assert!(store.changes_after(0, 100).unwrap().is_empty());
    }

    #[test]
    fn recovery_releases_only_abandoned_ownership_and_preserves_saved_content() {
        let directory = tempdir().unwrap();
        let store = LocalDataStore::open(directory.path()).unwrap();
        let library = AuthoringMediaLibrary::new(&store);
        let session = library.begin_session(vec![]).unwrap();
        let draft_media = import_image_with_session(&store, &png_bytes(2), Some(&session)).unwrap();
        let concept_media =
            import_image_with_session(&store, &png_bytes(3), Some(&session)).unwrap();
        let abandoned = import_image_with_session(&store, &png_bytes(4), Some(&session)).unwrap();
        let unrelated = import_image(&store, &png_bytes(5)).unwrap();

        save_draft(&store, &session, vec![draft_media.id.clone()]);
        store
            .write_result::<_, LibraryError>(|transaction| {
                let concept = transaction.create_entity(EntityKind::Concept)?;

                transaction.execute(
                    "INSERT INTO concepts (entity_id, title, last_change_id)
                VALUES (?1, 'Saved concept', ?2)",
                    params![concept.id, concept.last_change_id],
                )?;
                transaction.execute(
                    "INSERT INTO concept_media (
                    concept_id, media_id, created_at, updated_at, last_change_id
                ) VALUES (?1, ?2, ?3, ?3, ?4)",
                    params![
                        concept.id,
                        concept_media.id,
                        current_timestamp()?,
                        concept.last_change_id
                    ],
                )?;

                Ok(())
            })
            .unwrap();

        drop(store);

        let reopened = LocalDataStore::open(directory.path()).unwrap();
        let library = AuthoringMediaLibrary::new(&reopened);

        library.release_abandoned_sessions().unwrap();
        library.release_abandoned_sessions().unwrap();

        for media in [&draft_media, &concept_media, &unrelated] {
            assert!(read_media(&reopened, &media.id).is_ok());
        }

        assert!(matches!(
            read_media(&reopened, &abandoned.id),
            Err(LibraryError::MediaNotFound(_))
        ));
        assert_eq!(
            std::fs::read_dir(reopened.media_directory())
                .unwrap()
                .count(),
            3
        );
        assert!(matches!(
            import_image_with_session(&reopened, &png_bytes(6), Some(&session)),
            Err(LibraryError::AuthoringMediaSessionClosed)
        ));
    }
}
