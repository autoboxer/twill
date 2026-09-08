use std::collections::{BTreeSet, HashSet};
use std::fmt::Write as FormatWrite;
use std::fs::{self, OpenOptions};
use std::io::{Cursor, Write as IoWrite};
use std::path::{Path, PathBuf};

use image::{ImageFormat, ImageReader};
use rusqlite::{params, Connection, OptionalExtension};
use sha2::{Digest, Sha256};

use crate::data::{DataError, EntityKind, LocalDataStore, WriteTransaction};
use crate::library::authoring_media::{retain_session_media, validate_session};
use crate::library::{LibraryError, LibraryResult, MediaSummary};

const MAXIMUM_IMAGE_BYTES: usize = 20 * 1024 * 1024;
const MAXIMUM_IMAGE_MEGABYTES: usize = 20;
const MAXIMUM_IMAGE_PIXELS: u64 = 100_000_000;

struct ImageMetadata {
    digest: String,
    extension: &'static str,
    height: i64,
    mime_type: &'static str,
    width: i64,
}

struct MediaRecord {
    id: String,
    digest: String,
    extension: String,
    mime_type: String,
    byte_size: i64,
    width: i64,
    height: i64,
}

pub fn import_image(store: &LocalDataStore, bytes: &[u8]) -> LibraryResult<MediaSummary> {
    import_image_with_session(store, bytes, None)
}

pub fn import_image_with_session(
    store: &LocalDataStore,
    bytes: &[u8],
    session_id: Option<&str>,
) -> LibraryResult<MediaSummary> {
    let metadata = validate_image(bytes)?;
    let byte_size = i64::try_from(bytes.len()).map_err(|_| LibraryError::ImageTooLarge {
        maximum_megabytes: MAXIMUM_IMAGE_MEGABYTES,
    })?;

    // Record cleanup before writing so an interrupted import remains recoverable
    store.write_result::<_, LibraryError>(|transaction| {
        if let Some(session_id) = session_id {
            validate_session(transaction, session_id)?;
        }

        queue_media_cleanup(transaction, &metadata.digest, metadata.extension)
    })?;

    store.write_result(|transaction| {
        if let Some(session_id) = session_id {
            validate_session(transaction, session_id)?;
        }

        write_media_file(
            &store.media_directory(),
            &metadata.digest,
            metadata.extension,
            bytes,
        )?;

        let media = match query_media_by_digest(transaction, &metadata.digest)? {
            Some(existing) => existing.summary(),
            None => insert_media(transaction, &metadata, byte_size)?,
        };

        if let Some(session_id) = session_id {
            retain_session_media(
                transaction,
                session_id,
                &BTreeSet::from([media.id.clone()]),
            )?;
        }

        transaction.execute(
            "DELETE FROM device_media_cleanup WHERE digest = ?1",
            [&metadata.digest],
        )?;

        Ok(media)
    })
}

fn insert_media(
    transaction: &WriteTransaction<'_>,
    metadata: &ImageMetadata,
    byte_size: i64,
) -> LibraryResult<MediaSummary> {
    let entity = transaction.create_entity(EntityKind::Media)?;

    transaction.execute(
        "INSERT INTO media (
            entity_id,
            digest,
            mime_type,
            file_extension,
            byte_size,
            width,
            height,
            last_change_id
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            entity.id,
            metadata.digest,
            metadata.mime_type,
            metadata.extension,
            byte_size,
            metadata.width,
            metadata.height,
            entity.last_change_id
        ],
    )?;

    Ok(query_media(transaction, &entity.id)?.summary())
}

pub fn read_media(store: &LocalDataStore, id: &str) -> LibraryResult<Vec<u8>> {
    let id = id.trim();
    let record = store.read_result(|connection| query_media(connection, id))?;
    let path = media_path(&store.media_directory(), &record.digest, &record.extension);
    let bytes = fs::read(path).map_err(DataError::from)?;
    let actual_digest = digest(&bytes);

    if actual_digest != record.digest {
        return Err(LibraryError::MediaIntegrity {
            expected_digest: record.digest,
        });
    }

    Ok(bytes)
}

pub fn query_concept_media(
    connection: &Connection,
    concept_id: &str,
) -> LibraryResult<Vec<MediaSummary>> {
    let mut statement = connection.prepare(
        "SELECT
            media.entity_id,
            media.digest,
            media.file_extension,
            media.mime_type,
            media.byte_size,
            media.width,
            media.height
        FROM concept_media
        INNER JOIN media ON media.entity_id = concept_media.media_id
        INNER JOIN entities ON entities.id = media.entity_id
        WHERE concept_media.concept_id = ?1
            AND concept_media.removed_at IS NULL
            AND entities.deleted_at IS NULL
        ORDER BY entities.created_at, media.entity_id",
    )?;
    let media = statement.query_map([concept_id], media_record_from_row)?;

    media
        .map(|record| record.map(|record| record.summary()))
        .collect::<Result<_, _>>()
        .map_err(Into::into)
}

pub fn query_media_for_concepts(
    connection: &Connection,
    concept_ids: &BTreeSet<String>,
) -> LibraryResult<Vec<MediaSummary>> {
    if concept_ids.is_empty() {
        return Ok(Vec::new());
    }

    let placeholders = vec!["?"; concept_ids.len()].join(", ");
    let sql = format!(
        "SELECT DISTINCT
            media.entity_id,
            media.digest,
            media.file_extension,
            media.mime_type,
            media.byte_size,
            media.width,
            media.height
        FROM concept_media
        INNER JOIN media ON media.entity_id = concept_media.media_id
        INNER JOIN entities ON entities.id = media.entity_id
        WHERE concept_media.concept_id IN ({placeholders})
            AND concept_media.removed_at IS NULL
            AND entities.deleted_at IS NULL
        ORDER BY media.entity_id"
    );
    let mut statement = connection.prepare(&sql)?;
    let media = statement.query_map(
        rusqlite::params_from_iter(concept_ids),
        media_record_from_row,
    )?;

    media
        .map(|record| record.map(|record| record.summary()))
        .collect::<Result<_, _>>()
        .map_err(Into::into)
}

pub fn active_concept_media_ids(
    connection: &Connection,
    concept_id: &str,
) -> LibraryResult<HashSet<String>> {
    let mut statement = connection.prepare(
        "SELECT media_id
        FROM concept_media
        WHERE concept_id = ?1
            AND removed_at IS NULL",
    )?;
    let ids = statement.query_map([concept_id], |row| row.get(0))?;

    Ok(ids.collect::<Result<_, _>>()?)
}

pub fn validate_media_ids(
    connection: &Connection,
    media_ids: &HashSet<String>,
) -> LibraryResult<()> {
    for id in media_ids {
        if query_optional_media(connection, id)?.is_none() {
            return Err(LibraryError::MediaNotFound(id.clone()));
        }
    }

    Ok(())
}

pub(super) fn release_unreferenced_media(
    transaction: &WriteTransaction<'_>,
    media_ids: &[String],
) -> LibraryResult<()> {
    for media_id in media_ids {
        let retained: bool = transaction.query_row(
            "SELECT
                EXISTS (
                    SELECT 1 FROM authoring_session_media WHERE media_id = ?1
                )
                OR EXISTS (
                    SELECT 1 FROM authoring_draft_media WHERE media_id = ?1
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

        let Some(media) = query_optional_media(transaction, media_id)? else {
            continue;
        };

        transaction.soft_delete_entity(media_id)?;
        queue_media_cleanup(transaction, &media.digest, &media.extension)?;
    }

    Ok(())
}

fn queue_media_cleanup(
    transaction: &WriteTransaction<'_>,
    digest: &str,
    extension: &str,
) -> LibraryResult<()> {
    transaction.execute(
        "INSERT INTO device_media_cleanup (digest, file_extension)
        VALUES (?1, ?2)
        ON CONFLICT(digest) DO UPDATE SET file_extension = excluded.file_extension",
        params![digest, extension],
    )?;

    Ok(())
}

fn validate_image(bytes: &[u8]) -> LibraryResult<ImageMetadata> {
    if bytes.is_empty() || bytes.len() > MAXIMUM_IMAGE_BYTES {
        return Err(LibraryError::ImageTooLarge {
            maximum_megabytes: MAXIMUM_IMAGE_MEGABYTES,
        });
    }

    let format = image::guess_format(bytes).map_err(|_| LibraryError::UnsupportedImage)?;
    let (mime_type, extension) = match format {
        ImageFormat::Gif => ("image/gif", "gif"),
        ImageFormat::Jpeg => ("image/jpeg", "jpg"),
        ImageFormat::Png => ("image/png", "png"),
        ImageFormat::WebP => ("image/webp", "webp"),
        _ => return Err(LibraryError::UnsupportedImage),
    };
    let reader = ImageReader::with_format(Cursor::new(bytes), format);
    let (width, height) = reader
        .into_dimensions()
        .map_err(|_| LibraryError::UnsupportedImage)?;
    let pixels = u64::from(width) * u64::from(height);

    if width == 0 || height == 0 || pixels > MAXIMUM_IMAGE_PIXELS {
        return Err(LibraryError::ImageDimensionsTooLarge);
    }

    Ok(ImageMetadata {
        digest: digest(bytes),
        extension,
        height: i64::from(height),
        mime_type,
        width: i64::from(width),
    })
}

fn write_media_file(
    directory: &Path,
    digest: &str,
    extension: &str,
    bytes: &[u8],
) -> LibraryResult<()> {
    fs::create_dir_all(directory).map_err(DataError::from)?;

    let destination = media_path(directory, digest, extension);

    if destination.exists() {
        return verify_existing_file(&destination, bytes, digest);
    }

    let temporary_path = directory.join(format!(".{digest}.{extension}.tmp"));
    let result = write_temporary_file(&temporary_path, &destination, bytes);

    if result.is_err() {
        let _ = fs::remove_file(&temporary_path);
    }

    result?;

    Ok(())
}

fn write_temporary_file(
    temporary_path: &Path,
    destination: &Path,
    bytes: &[u8],
) -> Result<(), DataError> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(temporary_path)?;

    file.write_all(bytes)?;
    file.sync_all()?;
    drop(file);

    if destination.exists() {
        fs::remove_file(temporary_path)?;
        return Ok(());
    }

    fs::rename(temporary_path, destination)?;

    Ok(())
}

fn verify_existing_file(
    path: &Path,
    expected_bytes: &[u8],
    expected_digest: &str,
) -> LibraryResult<()> {
    let existing_bytes = fs::read(path).map_err(DataError::from)?;

    if existing_bytes != expected_bytes {
        return Err(LibraryError::MediaIntegrity {
            expected_digest: expected_digest.to_owned(),
        });
    }

    Ok(())
}

fn query_media(connection: &Connection, id: &str) -> LibraryResult<MediaRecord> {
    query_optional_media(connection, id)?
        .ok_or_else(|| LibraryError::MediaNotFound(id.to_owned()))
}

fn query_optional_media(
    connection: &Connection,
    id: &str,
) -> LibraryResult<Option<MediaRecord>> {
    Ok(connection
        .query_row(
            "SELECT
                media.entity_id,
                media.digest,
                media.file_extension,
                media.mime_type,
                media.byte_size,
                media.width,
                media.height
            FROM media
            INNER JOIN entities ON entities.id = media.entity_id
            WHERE media.entity_id = ?1
                AND entities.deleted_at IS NULL",
            [id],
            media_record_from_row,
        )
        .optional()?)
}

fn query_media_by_digest(
    connection: &Connection,
    digest: &str,
) -> LibraryResult<Option<MediaRecord>> {
    Ok(connection
        .query_row(
            "SELECT
                media.entity_id,
                media.digest,
                media.file_extension,
                media.mime_type,
                media.byte_size,
                media.width,
                media.height
            FROM media
            INNER JOIN entities ON entities.id = media.entity_id
            WHERE media.digest = ?1
                AND entities.deleted_at IS NULL",
            [digest],
            media_record_from_row,
        )
        .optional()?)
}

fn media_record_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<MediaRecord> {
    Ok(MediaRecord {
        id: row.get(0)?,
        digest: row.get(1)?,
        extension: row.get(2)?,
        mime_type: row.get(3)?,
        byte_size: row.get(4)?,
        width: row.get(5)?,
        height: row.get(6)?,
    })
}

fn media_path(directory: &Path, digest: &str, extension: &str) -> PathBuf {
    directory.join(format!("{digest}.{extension}"))
}

fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes).iter().fold(
        String::with_capacity(64),
        |mut digest, byte| {
            write!(&mut digest, "{byte:02x}").expect("writing to a string cannot fail");
            digest
        },
    )
}

impl MediaRecord {
    fn summary(&self) -> MediaSummary {
        MediaSummary {
            id: self.id.clone(),
            mime_type: self.mime_type.clone(),
            byte_size: self.byte_size,
            width: self.width,
            height: self.height,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, io::Cursor};

    use image::{DynamicImage, ImageFormat};
    use tempfile::tempdir;

    use super::{digest, import_image, import_image_with_session, media_path, read_media};
    use crate::data::{DataResult, LocalDataStore};
    use crate::library::{AuthoringMediaLibrary, LibraryError};

    fn png_bytes() -> Vec<u8> {
        let mut bytes = Cursor::new(Vec::new());

        DynamicImage::new_rgba8(2, 3)
            .write_to(&mut bytes, ImageFormat::Png)
            .unwrap();

        bytes.into_inner()
    }

    #[test]
    fn imported_images_are_validated_stored_and_deduplicated() {
        let directory = tempdir().unwrap();
        let store = LocalDataStore::open(directory.path()).unwrap();
        let bytes = png_bytes();

        let imported = import_image(&store, &bytes).unwrap();
        let duplicate = import_image(&store, &bytes).unwrap();

        assert_eq!(imported.id, duplicate.id);
        assert_eq!(imported.mime_type, "image/png");
        assert_eq!((imported.width, imported.height), (2, 3));
        assert_eq!(read_media(&store, &imported.id).unwrap(), bytes);
        assert_eq!(store.changes_after(0, 100).unwrap().len(), 1);

        let hard_delete: DataResult<()> = store.write(|transaction| {
            transaction.execute(
                "DELETE FROM media WHERE entity_id = ?1",
                [&imported.id],
            )?;

            Ok(())
        });

        assert!(hard_delete.is_err());
    }

    #[test]
    fn failed_import_transactions_leave_durable_cleanup_for_written_files() {
        let directory = tempdir().unwrap();
        let store = LocalDataStore::open(directory.path()).unwrap();
        let session = AuthoringMediaLibrary::new(&store).begin_session(vec![]).unwrap();
        let bytes = png_bytes();
        let path = media_path(&store.media_directory(), &digest(&bytes), "png");

        store.write(|transaction| {
            transaction.execute_batch(
                "CREATE TEMP TRIGGER reject_media BEFORE INSERT ON authoring_session_media
                BEGIN SELECT RAISE(ABORT, 'Simulated ownership failure'); END;",
            )?;

            Ok(())
        }).unwrap();

        assert!(import_image_with_session(&store, &bytes, Some(&session)).is_err());
        assert!(path.is_file());
        assert!(store.changes_after(0, 100).unwrap().is_empty());
        drop(store);

        let reopened = LocalDataStore::open(directory.path()).unwrap();

        assert!(!path.exists());
        assert!(import_image(&reopened, &bytes).is_ok());
    }

    #[test]
    fn recovery_cleans_partial_imports_without_deleting_active_duplicates() {
        let directory = tempdir().unwrap();
        let store = LocalDataStore::open(directory.path()).unwrap();
        let bytes = png_bytes();
        let media = import_image(&store, &bytes).unwrap();
        let digest = digest(&bytes);
        let temporary_path = store.media_directory().join(format!(".{digest}.png.tmp"));

        store.write_result::<_, LibraryError>(|transaction| {
            super::queue_media_cleanup(transaction, &digest, "png")
        }).unwrap();
        fs::write(&temporary_path, b"Interrupted file write").unwrap();
        drop(store);

        let reopened = LocalDataStore::open(directory.path()).unwrap();

        assert!(!temporary_path.exists());
        assert_eq!(read_media(&reopened, &media.id).unwrap(), bytes);
    }

    #[test]
    fn corrupt_managed_media_reports_the_expected_digest() {
        let directory = tempdir().unwrap();
        let store = LocalDataStore::open(directory.path()).unwrap();
        let bytes = png_bytes();
        let expected_digest = digest(&bytes);
        let imported = import_image(&store, &bytes).unwrap();
        let path = media_path(&store.media_directory(), &expected_digest, "png");

        fs::write(path, b"corrupt image data").unwrap();

        assert!(matches!(
            read_media(&store, &imported.id),
            Err(LibraryError::MediaIntegrity {
                expected_digest: actual_digest,
            }) if actual_digest == expected_digest
        ));
    }

    #[test]
    fn unsupported_and_oversized_files_are_rejected() {
        let directory = tempdir().unwrap();
        let store = LocalDataStore::open(directory.path()).unwrap();

        assert!(matches!(
            import_image(&store, b"not an image"),
            Err(LibraryError::UnsupportedImage)
        ));
        assert!(matches!(
            import_image(&store, &vec![0; 20 * 1024 * 1024 + 1]),
            Err(LibraryError::ImageTooLarge { .. })
        ));
        assert!(store.changes_after(0, 100).unwrap().is_empty());
    }
}
