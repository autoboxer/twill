use std::collections::BTreeSet;

use rusqlite::{params, Connection, OptionalExtension};
use uuid::Uuid;

use crate::data::{current_timestamp, LocalDataStore, WriteTransaction};
use crate::library::{
    AuthoringDraft, AuthoringDraftKind, AuthoringDraftLocator,
    AuthoringDraftTargetStatus, LibraryError, LibraryResult,
    UpsertAuthoringDraftInput, AUTHORING_DRAFT_SCHEMA_VERSION,
};

const MAXIMUM_DRAFT_MEDIA: usize = 1_000;
const MAXIMUM_DRAFT_PAYLOAD_BYTES: usize = 5_000_000;
const NEW_TARGET_KEY: &str = "new";

pub struct AuthoringDraftLibrary<'store> {
    store: &'store LocalDataStore,
}

struct DraftIdentity {
    kind: AuthoringDraftKind,
    target_id: Option<String>,
    target_key: String,
}

impl<'store> AuthoringDraftLibrary<'store> {
    pub fn new(store: &'store LocalDataStore) -> Self {
        Self { store }
    }

    pub fn draft(
        &self,
        locator: AuthoringDraftLocator,
    ) -> LibraryResult<Option<AuthoringDraft>> {
        let identity = normalize_identity(locator.kind, locator.target_id)?;

        self.store.read_result(|connection| {
            query_draft(connection, identity.kind, &identity.target_key)
        })
    }

    pub fn upsert_draft(
        &self,
        input: UpsertAuthoringDraftInput,
    ) -> LibraryResult<AuthoringDraft> {
        let identity = normalize_identity(input.kind, input.target_id)?;
        let base_change_id = normalize_base_change_id(
            identity.target_id.as_deref(),
            input.base_change_id,
        )?;
        let payload_json = validate_payload(input.schema_version, &input.payload)?;
        let media_ids = normalize_media_ids(input.media_ids)?;
        let timestamp = current_timestamp()?;

        self.store.write_result(|transaction| {
            validate_target(
                transaction,
                &identity,
                base_change_id.as_deref(),
            )?;
            validate_media(transaction, &media_ids)?;

            let previous_media = query_draft_media(
                transaction,
                identity.kind,
                &identity.target_key,
            )?;

            transaction.execute(
                "INSERT INTO authoring_drafts (
                    kind,
                    target_key,
                    target_id,
                    schema_version,
                    base_change_id,
                    payload_json,
                    created_at,
                    updated_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)
                ON CONFLICT(kind, target_key) DO UPDATE SET
                    schema_version = excluded.schema_version,
                    base_change_id = excluded.base_change_id,
                    payload_json = excluded.payload_json,
                    updated_at = excluded.updated_at",
                params![
                    identity.kind.as_str(),
                    identity.target_key,
                    identity.target_id,
                    input.schema_version,
                    base_change_id,
                    payload_json,
                    timestamp,
                ],
            )?;

            replace_draft_media(transaction, &identity, &media_ids)?;

            let released_media = previous_media
                .difference(&media_ids)
                .cloned()
                .collect::<Vec<_>>();

            release_unreferenced_media(transaction, &released_media)?;

            query_draft(transaction, identity.kind, &identity.target_key)?
                .ok_or_else(|| invalid_draft("could not be read after saving"))
        })
    }

    pub fn delete_draft(
        &self,
        locator: AuthoringDraftLocator,
    ) -> LibraryResult<()> {
        let identity = normalize_identity(locator.kind, locator.target_id)?;

        self.store.write_result(|transaction| {
            let released_media = query_draft_media(
                transaction,
                identity.kind,
                &identity.target_key,
            )?
            .into_iter()
            .collect::<Vec<_>>();

            transaction.execute(
                "DELETE FROM authoring_drafts
                WHERE kind = ?1 AND target_key = ?2",
                params![identity.kind.as_str(), identity.target_key],
            )?;

            release_unreferenced_media(transaction, &released_media)
        })
    }
}

fn normalize_identity(
    kind: AuthoringDraftKind,
    target_id: Option<String>,
) -> LibraryResult<DraftIdentity> {
    let target_id = target_id.map(|id| id.trim().to_owned());

    if let Some(id) = &target_id {
        if id.is_empty() || Uuid::parse_str(id).is_err() {
            return Err(invalid_draft("target ID is not valid"));
        }
    }

    let target_key = target_id
        .clone()
        .unwrap_or_else(|| NEW_TARGET_KEY.to_owned());

    Ok(DraftIdentity {
        kind,
        target_id,
        target_key,
    })
}

fn normalize_base_change_id(
    target_id: Option<&str>,
    base_change_id: Option<String>,
) -> LibraryResult<Option<String>> {
    let base_change_id = base_change_id.map(|id| id.trim().to_owned());

    match (target_id, &base_change_id) {
        (None, None) => Ok(None),
        (Some(_), Some(id)) if Uuid::parse_str(id).is_ok() => Ok(base_change_id),
        (None, Some(_)) => Err(invalid_draft(
            "cannot have a base change without a saved target",
        )),
        (Some(_), None) => Err(invalid_draft(
            "requires a base change for a saved target",
        )),
        (Some(_), Some(_)) => Err(invalid_draft("base change ID is not valid")),
    }
}

fn validate_payload(schema_version: u32, payload: &serde_json::Value) -> LibraryResult<String> {
    if schema_version != AUTHORING_DRAFT_SCHEMA_VERSION {
        return Err(invalid_draft("uses an unsupported schema version"));
    }

    if !payload.is_object() {
        return Err(invalid_draft("payload must be an object"));
    }

    let payload_json = serde_json::to_string(payload)?;

    if payload_json.len() > MAXIMUM_DRAFT_PAYLOAD_BYTES {
        return Err(invalid_draft("payload cannot be larger than 5 MB"));
    }

    Ok(payload_json)
}

fn normalize_media_ids(media_ids: Vec<String>) -> LibraryResult<BTreeSet<String>> {
    if media_ids.len() > MAXIMUM_DRAFT_MEDIA {
        return Err(invalid_draft("cannot retain more than 1,000 media items"));
    }

    media_ids
        .into_iter()
        .map(|id| {
            let id = id.trim().to_owned();

            if Uuid::parse_str(&id).is_err() {
                return Err(invalid_draft("contains an invalid media ID"));
            }

            Ok(id)
        })
        .collect()
}

fn validate_target(
    connection: &Connection,
    identity: &DraftIdentity,
    base_change_id: Option<&str>,
) -> LibraryResult<()> {
    let Some(target_id) = &identity.target_id else {
        return Ok(());
    };
    let Some(base_change_id) = base_change_id else {
        return Err(invalid_draft(
            "requires a base change for a saved target",
        ));
    };
    let expected_kind = identity.kind.as_str();
    let target_kind = connection
        .query_row(
            "SELECT kind FROM entities WHERE id = ?1",
            [target_id],
            |row| row.get::<_, String>(0),
        )
        .optional()?;

    if target_kind.as_deref() != Some(expected_kind) {
        return Err(invalid_draft("target was not found"));
    }

    let base_matches_target: bool = connection.query_row(
        "SELECT EXISTS (
            SELECT 1
            FROM change_log
            WHERE id = ?1 AND entity_id = ?2
        )",
        params![base_change_id, target_id],
        |row| row.get(0),
    )?;

    if !base_matches_target {
        return Err(invalid_draft("base change does not belong to its target"));
    }

    Ok(())
}

fn validate_media(
    connection: &Connection,
    media_ids: &BTreeSet<String>,
) -> LibraryResult<()> {
    for media_id in media_ids {
        let exists: bool = connection.query_row(
            "SELECT EXISTS (
                SELECT 1
                FROM media
                INNER JOIN entities ON entities.id = media.entity_id
                WHERE media.entity_id = ?1
                    AND entities.deleted_at IS NULL
            )",
            [media_id],
            |row| row.get(0),
        )?;

        if !exists {
            return Err(LibraryError::MediaNotFound(media_id.clone()));
        }
    }

    Ok(())
}

fn replace_draft_media(
    transaction: &WriteTransaction<'_>,
    identity: &DraftIdentity,
    media_ids: &BTreeSet<String>,
) -> LibraryResult<()> {
    transaction.execute(
        "DELETE FROM authoring_draft_media
        WHERE kind = ?1 AND target_key = ?2",
        params![identity.kind.as_str(), identity.target_key],
    )?;

    for media_id in media_ids {
        transaction.execute(
            "INSERT INTO authoring_draft_media (kind, target_key, media_id)
            VALUES (?1, ?2, ?3)",
            params![identity.kind.as_str(), identity.target_key, media_id],
        )?;
    }

    Ok(())
}

fn release_unreferenced_media(
    transaction: &WriteTransaction<'_>,
    media_ids: &[String],
) -> LibraryResult<()> {
    for media_id in media_ids {
        let retained: bool = transaction.query_row(
            "SELECT
                EXISTS (
                    SELECT 1
                    FROM authoring_draft_media
                    WHERE media_id = ?1
                )
                OR EXISTS (
                    SELECT 1
                    FROM concept_media
                    INNER JOIN entities AS concept_entities
                        ON concept_entities.id = concept_media.concept_id
                    WHERE concept_media.media_id = ?1
                        AND concept_media.removed_at IS NULL
                        AND concept_entities.deleted_at IS NULL
                )",
            [media_id],
            |row| row.get(0),
        )?;

        if retained {
            continue;
        }

        let media = transaction
            .query_row(
                "SELECT media.digest, media.file_extension
                FROM media
                INNER JOIN entities ON entities.id = media.entity_id
                WHERE media.entity_id = ?1
                    AND entities.deleted_at IS NULL",
                [media_id],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
            )
            .optional()?;
        let Some((digest, extension)) = media else {
            continue;
        };

        transaction.soft_delete_entity(media_id)?;
        transaction.execute(
            "INSERT INTO device_media_cleanup (digest, file_extension)
            VALUES (?1, ?2)
            ON CONFLICT(digest) DO UPDATE SET
                file_extension = excluded.file_extension",
            params![digest, extension],
        )?;
    }

    Ok(())
}

fn query_draft(
    connection: &Connection,
    kind: AuthoringDraftKind,
    target_key: &str,
) -> LibraryResult<Option<AuthoringDraft>> {
    let draft = connection
        .query_row(
            "SELECT
                target_id,
                schema_version,
                base_change_id,
                payload_json,
                created_at,
                updated_at
            FROM authoring_drafts
            WHERE kind = ?1 AND target_key = ?2",
            params![kind.as_str(), target_key],
            |row| {
                Ok((
                    row.get::<_, Option<String>>(0)?,
                    row.get::<_, u32>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, i64>(5)?,
                ))
            },
        )
        .optional()?;

    let Some((target_id, schema_version, base_change_id, payload_json, created_at, updated_at)) =
        draft
    else {
        return Ok(None);
    };
    let media_ids = query_draft_media(connection, kind, target_key)?
        .into_iter()
        .collect();
    let target_status = query_target_status(
        connection,
        kind,
        target_id.as_deref(),
        base_change_id.as_deref(),
    )?;

    Ok(Some(AuthoringDraft {
        kind,
        target_id,
        schema_version,
        base_change_id,
        payload: serde_json::from_str(&payload_json)?,
        media_ids,
        created_at,
        updated_at,
        target_status,
    }))
}

fn query_draft_media(
    connection: &Connection,
    kind: AuthoringDraftKind,
    target_key: &str,
) -> LibraryResult<BTreeSet<String>> {
    let mut statement = connection.prepare(
        "SELECT media_id
        FROM authoring_draft_media
        WHERE kind = ?1 AND target_key = ?2
        ORDER BY media_id",
    )?;
    let media_ids = statement.query_map(params![kind.as_str(), target_key], |row| {
        row.get::<_, String>(0)
    })?;

    media_ids
        .collect::<Result<BTreeSet<_>, _>>()
        .map_err(Into::into)
}

fn query_target_status(
    connection: &Connection,
    kind: AuthoringDraftKind,
    target_id: Option<&str>,
    base_change_id: Option<&str>,
) -> LibraryResult<AuthoringDraftTargetStatus> {
    let Some(target_id) = target_id else {
        return Ok(AuthoringDraftTargetStatus::Current);
    };
    let current = connection
        .query_row(
            "SELECT last_change_id, deleted_at
            FROM entities
            WHERE id = ?1 AND kind = ?2",
            params![target_id, kind.as_str()],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, Option<i64>>(1)?,
                ))
            },
        )
        .optional()?;

    match current {
        None | Some((_, Some(_))) => Ok(AuthoringDraftTargetStatus::Missing),
        Some((current_change_id, None)) if Some(current_change_id.as_str()) == base_change_id => {
            Ok(AuthoringDraftTargetStatus::Current)
        }
        Some((_, None)) => Ok(AuthoringDraftTargetStatus::Changed),
    }
}

fn invalid_draft(message: impl Into<String>) -> LibraryError {
    LibraryError::InvalidAuthoringDraft {
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use image::{DynamicImage, ImageFormat};
    use rusqlite::params;
    use serde_json::json;
    use tempfile::tempdir;

    use super::AuthoringDraftLibrary;
    use crate::data::{current_timestamp, EntityKind, LocalDataStore};
    use crate::library::{
        AuthoringDraftKind, AuthoringDraftLocator, AuthoringDraftTargetStatus,
        ConceptLibrary, LibraryError, UpsertAuthoringDraftInput,
        AUTHORING_DRAFT_SCHEMA_VERSION,
    };

    fn png_bytes() -> Vec<u8> {
        let mut bytes = Cursor::new(Vec::new());

        DynamicImage::new_rgba8(4, 3)
            .write_to(&mut bytes, ImageFormat::Png)
            .unwrap();

        bytes.into_inner()
    }

    fn upsert_input(
        kind: AuthoringDraftKind,
        target_id: Option<&str>,
        base_change_id: Option<&str>,
        payload: serde_json::Value,
        media_ids: Vec<String>,
    ) -> UpsertAuthoringDraftInput {
        UpsertAuthoringDraftInput {
            kind,
            target_id: target_id.map(str::to_owned),
            schema_version: AUTHORING_DRAFT_SCHEMA_VERSION,
            base_change_id: base_change_id.map(str::to_owned),
            payload,
            media_ids,
        }
    }

    fn locator(
        kind: AuthoringDraftKind,
        target_id: Option<&str>,
    ) -> AuthoringDraftLocator {
        AuthoringDraftLocator {
            kind,
            target_id: target_id.map(str::to_owned),
        }
    }

    #[test]
    fn drafts_are_device_local_and_report_changed_or_missing_targets() {
        let directory = tempdir().unwrap();
        let store = LocalDataStore::open(directory.path()).unwrap();
        let target = store
            .write(|transaction| transaction.create_entity(EntityKind::Concept))
            .unwrap();
        let initial_change_count = store.changes_after(0, 100).unwrap().len();
        let library = AuthoringDraftLibrary::new(&store);

        let created = library
            .upsert_draft(upsert_input(
                AuthoringDraftKind::Concept,
                Some(&target.id),
                Some(&target.last_change_id),
                json!({ "title": "First draft" }),
                vec![],
            ))
            .unwrap();

        assert_eq!(created.target_status, AuthoringDraftTargetStatus::Current);
        assert_eq!(store.changes_after(0, 100).unwrap().len(), initial_change_count);

        let updated = library
            .upsert_draft(upsert_input(
                AuthoringDraftKind::Concept,
                Some(&target.id),
                Some(&target.last_change_id),
                json!({ "title": "Updated draft" }),
                vec![],
            ))
            .unwrap();

        assert_eq!(updated.created_at, created.created_at);
        assert_eq!(updated.payload, json!({ "title": "Updated draft" }));
        assert_eq!(store.changes_after(0, 100).unwrap().len(), initial_change_count);

        store
            .write(|transaction| transaction.touch_entity(&target.id).map(|_| ()))
            .unwrap();

        let changed = library
            .draft(locator(AuthoringDraftKind::Concept, Some(&target.id)))
            .unwrap()
            .unwrap();

        assert_eq!(changed.target_status, AuthoringDraftTargetStatus::Changed);

        store
            .write(|transaction| transaction.soft_delete_entity(&target.id).map(|_| ()))
            .unwrap();

        let missing = library
            .draft(locator(AuthoringDraftKind::Concept, Some(&target.id)))
            .unwrap()
            .unwrap();

        assert_eq!(missing.target_status, AuthoringDraftTargetStatus::Missing);

        library
            .delete_draft(locator(AuthoringDraftKind::Concept, Some(&target.id)))
            .unwrap();
        assert_eq!(
            library
                .draft(locator(AuthoringDraftKind::Concept, Some(&target.id)))
                .unwrap(),
            None,
        );
    }

    #[test]
    fn released_media_is_cleaned_only_after_its_last_reference() {
        let directory = tempdir().unwrap();
        let store = LocalDataStore::open(directory.path()).unwrap();
        let media = ConceptLibrary::new(&store)
            .import_image(&png_bytes())
            .unwrap();
        let (digest, extension): (String, String) = store
            .read_result(|connection| {
                connection
                    .query_row(
                        "SELECT digest, file_extension FROM media WHERE entity_id = ?1",
                        [&media.id],
                        |row| Ok((row.get(0)?, row.get(1)?)),
                    )
                    .map_err(LibraryError::from)
            })
            .unwrap();
        let media_path = store
            .media_directory()
            .join(format!("{digest}.{extension}"));
        let library = AuthoringDraftLibrary::new(&store);

        library
            .upsert_draft(upsert_input(
                AuthoringDraftKind::Concept,
                None,
                None,
                json!({ "title": "Concept draft" }),
                vec![media.id.clone()],
            ))
            .unwrap();
        library
            .upsert_draft(upsert_input(
                AuthoringDraftKind::Template,
                None,
                None,
                json!({ "name": "Template draft" }),
                vec![media.id.clone()],
            ))
            .unwrap();

        library
            .delete_draft(locator(AuthoringDraftKind::Concept, None))
            .unwrap();
        assert!(store.entity(&media.id).unwrap().unwrap().deleted_at.is_none());

        library
            .delete_draft(locator(AuthoringDraftKind::Template, None))
            .unwrap();
        assert!(store.entity(&media.id).unwrap().unwrap().deleted_at.is_some());
        assert!(media_path.is_file());

        let queued: bool = store
            .read_result(|connection| {
                connection
                    .query_row(
                        "SELECT EXISTS (
                            SELECT 1 FROM device_media_cleanup WHERE digest = ?1
                        )",
                        [&digest],
                        |row| row.get(0),
                    )
                    .map_err(LibraryError::from)
            })
            .unwrap();

        assert!(queued);
        drop(library);
        drop(store);

        let reopened = LocalDataStore::open(directory.path()).unwrap();

        assert!(!media_path.exists());
        let reimported = ConceptLibrary::new(&reopened)
            .import_image(&png_bytes())
            .unwrap();

        assert_ne!(reimported.id, media.id);
        assert!(media_path.is_file());
    }

    #[test]
    fn active_concept_media_is_not_cleaned_with_a_draft() {
        let directory = tempdir().unwrap();
        let store = LocalDataStore::open(directory.path()).unwrap();
        let media = ConceptLibrary::new(&store)
            .import_image(&png_bytes())
            .unwrap();
        let concept = store
            .write_result(|transaction| {
                let concept = transaction.create_entity(EntityKind::Concept)?;

                transaction.execute(
                    "INSERT INTO concepts (entity_id, title, last_change_id)
                    VALUES (?1, 'Media owner', ?2)",
                    params![concept.id, concept.last_change_id],
                )?;
                transaction.execute(
                    "INSERT INTO concept_media (
                        concept_id,
                        media_id,
                        created_at,
                        updated_at,
                        removed_at,
                        last_change_id
                    ) VALUES (?1, ?2, ?3, ?3, NULL, ?4)",
                    params![
                        concept.id,
                        media.id,
                        current_timestamp()?,
                        concept.last_change_id,
                    ],
                )?;

                Ok::<_, LibraryError>(concept)
            })
            .unwrap();
        let library = AuthoringDraftLibrary::new(&store);

        library
            .upsert_draft(upsert_input(
                AuthoringDraftKind::Concept,
                Some(&concept.id),
                Some(&concept.last_change_id),
                json!({ "title": "Media owner draft" }),
                vec![media.id.clone()],
            ))
            .unwrap();
        library
            .delete_draft(locator(AuthoringDraftKind::Concept, Some(&concept.id)))
            .unwrap();

        assert!(store.entity(&media.id).unwrap().unwrap().deleted_at.is_none());
    }

    #[test]
    fn invalid_draft_shapes_and_target_relationships_are_rejected() {
        let directory = tempdir().unwrap();
        let store = LocalDataStore::open(directory.path()).unwrap();
        let concept = store
            .write(|transaction| transaction.create_entity(EntityKind::Concept))
            .unwrap();
        let template = store
            .write(|transaction| transaction.create_entity(EntityKind::Template))
            .unwrap();
        let library = AuthoringDraftLibrary::new(&store);

        let invalid_version = library.upsert_draft(UpsertAuthoringDraftInput {
            schema_version: 2,
            ..upsert_input(
                AuthoringDraftKind::Concept,
                None,
                None,
                json!({}),
                vec![],
            )
        });
        let invalid_payload = library.upsert_draft(upsert_input(
            AuthoringDraftKind::Concept,
            None,
            None,
            json!(["not", "an", "object"]),
            vec![],
        ));
        let wrong_kind = library.upsert_draft(upsert_input(
            AuthoringDraftKind::Template,
            Some(&concept.id),
            Some(&concept.last_change_id),
            json!({}),
            vec![],
        ));
        let wrong_base = library.upsert_draft(upsert_input(
            AuthoringDraftKind::Concept,
            Some(&concept.id),
            Some(&template.last_change_id),
            json!({}),
            vec![],
        ));

        for result in [invalid_version, invalid_payload, wrong_kind, wrong_base] {
            assert!(matches!(
                result,
                Err(LibraryError::InvalidAuthoringDraft { .. })
            ));
        }
    }
}
