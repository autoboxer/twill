use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use rusqlite::{Connection, OpenFlags, MAIN_DB};
use tempfile::{Builder, NamedTempFile};
use zip::{write::SimpleFileOptions, CompressionMethod, ZipWriter};

use crate::data::{current_timestamp, LocalDataStore};

use super::checksum::copy_and_digest;
use super::models::{
    ArchiveFile, ArchiveManifest, ARCHIVE_FORMAT, ARCHIVE_FORMAT_VERSION, DATABASE_PATH,
    LIBRARY_PATH, MANIFEST_PATH,
};
use super::portable::{library_counts, schema_fingerprint, write_library};
use super::{ArchiveKind, ArchiveSummary, BackupError, BackupResult};

const MAX_MEDIA_BYTES: u64 = 20 * 1024 * 1024;

const EXPORT_README: &str = "# Twill library export\n\n\
This archive is not encrypted. Keep it private.\n\n\
library.json contains table rows with their original identifiers and relations.\n\
Columns ending in _json contain decoded JSON, not JSON encoded inside strings.\n\
The media directory contains all active images, named by SHA-256 and extension.\n\
Deleted records retain tombstones; their image files may no longer exist.\n\n\
Backups also contain twill.sqlite3 for exact restoration. Portable exports do not.\n\
Saved drafts, preferences, templates, and learning history are included.\n\
An active Study session, unsaved responses, and editor undo history are not saved.\n";

struct MediaAsset {
    path: String,
    byte_size: u64,
    sha256: String,
}

pub fn create_backup(
    store: &LocalDataStore,
    destination: impl AsRef<Path>,
) -> BackupResult<ArchiveSummary> {
    create_archive(store, destination.as_ref(), ArchiveKind::Backup)
}

pub fn export_library(
    store: &LocalDataStore,
    destination: impl AsRef<Path>,
) -> BackupResult<ArchiveSummary> {
    create_archive(store, destination.as_ref(), ArchiveKind::Export)
}

fn create_archive(
    store: &LocalDataStore,
    destination: &Path,
    kind: ArchiveKind,
) -> BackupResult<ArchiveSummary> {
    let destination = prepare_destination(store, destination)?;
    let parent = destination
        .parent()
        .ok_or(BackupError::InvalidDestination)?;
    let staging = Builder::new().prefix(".twill-export-").tempdir_in(parent)?;
    let database_path = staging.path().join(DATABASE_PATH);

    // Capture media under the same boundary as the SQLite snapshot
    let (created_at, media) = store.read_result(|connection| {
        connection.backup(MAIN_DB, &database_path, None)?;

        let snapshot = open_snapshot(&database_path)?;
        let media = stage_media(&snapshot, &store.media_directory(), staging.path())?;

        Ok::<_, BackupError>((current_timestamp()?, media))
    })?;

    let snapshot = open_snapshot(&database_path)?;
    let schema_version = snapshot.query_row("PRAGMA user_version", [], |row| row.get(0))?;
    let counts = library_counts(&snapshot)?;
    let fingerprint = schema_fingerprint(&snapshot)?;
    let mut library = BufWriter::new(File::create(staging.path().join(LIBRARY_PATH))?);

    write_library(&snapshot, &mut library, created_at)?;
    library.flush()?;

    drop(library);
    drop(snapshot);

    fs::write(staging.path().join("README.md"), EXPORT_README)?;

    let mut output = Builder::new()
        .prefix(".twill-archive-")
        .tempfile_in(parent)?;
    let mut archive = ZipWriter::new(output.as_file_mut());
    let mut files = Vec::with_capacity(media.len() + 3);

    if kind == ArchiveKind::Backup {
        files.push(append_file(
            &mut archive,
            staging.path(),
            DATABASE_PATH,
            None,
        )?);
    }

    files.push(append_file(
        &mut archive,
        staging.path(),
        LIBRARY_PATH,
        None,
    )?);
    files.push(append_file(
        &mut archive,
        staging.path(),
        "README.md",
        None,
    )?);

    for asset in &media {
        files.push(append_file(
            &mut archive,
            staging.path(),
            &asset.path,
            Some(asset),
        )?);
    }

    let manifest = ArchiveManifest {
        format: ARCHIVE_FORMAT.to_owned(),
        format_version: ARCHIVE_FORMAT_VERSION,
        kind,
        app_version: env!("CARGO_PKG_VERSION").to_owned(),
        created_at,
        schema_version,
        schema_fingerprint: fingerprint,
        counts: counts.clone(),
        files,
    };

    super::validation::check_size_limits(&manifest.files)?;

    let manifest_bytes = serde_json::to_vec_pretty(&manifest)?;

    if manifest_bytes.len() as u64 > super::validation::MAX_MANIFEST_BYTES {
        return Err(BackupError::ArchiveLimit);
    }

    archive.start_file(MANIFEST_PATH, file_options(CompressionMethod::Deflated, 0))?;
    archive.write_all(&manifest_bytes)?;
    archive.finish()?.sync_all()?;

    let byte_size = output.as_file().metadata()?.len();

    publish_archive(output, &destination)?;

    Ok(ArchiveSummary {
        kind,
        created_at,
        byte_size,
        counts,
    })
}

fn open_snapshot(path: &Path) -> BackupResult<Connection> {
    Ok(Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_ONLY,
    )?)
}

fn prepare_destination(store: &LocalDataStore, destination: &Path) -> BackupResult<PathBuf> {
    if !destination.is_absolute() {
        return Err(BackupError::InvalidDestination);
    }

    let name = destination
        .file_name()
        .ok_or(BackupError::InvalidDestination)?;
    let parent = destination
        .parent()
        .ok_or(BackupError::InvalidDestination)?;
    let parent = fs::canonicalize(parent)?;
    let profile = fs::canonicalize(store.data_directory())?;

    if parent.starts_with(profile) {
        return Err(BackupError::InvalidDestination);
    }

    let destination = parent.join(name);

    match fs::symlink_metadata(&destination) {
        Ok(_) => return Err(BackupError::DestinationExists),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }

    Ok(destination)
}

fn stage_media(
    snapshot: &Connection,
    media_directory: &Path,
    staging_directory: &Path,
) -> BackupResult<Vec<MediaAsset>> {
    let mut statement = snapshot.prepare(
        "SELECT media.digest, media.file_extension, media.byte_size
        FROM media
        INNER JOIN entities ON entities.id = media.entity_id
        WHERE entities.deleted_at IS NULL
        ORDER BY media.digest",
    )?;
    let records = statement.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, i64>(2)?,
        ))
    })?;
    let mut assets = Vec::new();

    fs::create_dir(staging_directory.join("media"))?;

    for record in records {
        let (sha256, extension, byte_size) = record?;

        if sha256.len() != 64
            || !sha256
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
            || !matches!(extension.as_str(), "jpg" | "png" | "gif" | "webp")
            || !(1..=MAX_MEDIA_BYTES as i64).contains(&byte_size)
        {
            return Err(BackupError::Integrity("invalid media metadata"));
        }

        let byte_size = byte_size as u64;
        let filename = format!("{sha256}.{extension}");
        let source = media_directory.join(&filename);
        let metadata = fs::symlink_metadata(&source).map_err(|error| {
            if error.kind() == std::io::ErrorKind::NotFound {
                BackupError::Integrity("an image is missing")
            } else {
                BackupError::Io(error)
            }
        })?;

        if !metadata.is_file() || metadata.len() != byte_size {
            return Err(BackupError::Integrity("an image is missing or has changed"));
        }

        let asset = MediaAsset {
            path: format!("media/{filename}"),
            byte_size,
            sha256,
        };
        let mut input = File::open(source)?;
        let mut output = File::create(staging_directory.join(&asset.path))?;
        let (copied, digest) = copy_and_digest(&mut input, &mut output, byte_size)?;

        if copied != byte_size || digest != asset.sha256 {
            return Err(BackupError::Integrity("an image failed its checksum"));
        }

        assets.push(asset);
    }

    Ok(assets)
}

fn append_file(
    archive: &mut ZipWriter<&mut File>,
    staging_directory: &Path,
    path: &str,
    media: Option<&MediaAsset>,
) -> BackupResult<ArchiveFile> {
    let mut input = File::open(staging_directory.join(path))?;
    let size = input.metadata()?.len();
    let compression = if media.is_some() {
        CompressionMethod::Stored
    } else {
        CompressionMethod::Deflated
    };

    archive.start_file(path, file_options(compression, size))?;

    let (byte_size, sha256) = copy_and_digest(&mut input, archive, size)?;

    if byte_size != size
        || media.is_some_and(|asset| asset.byte_size != byte_size || asset.sha256 != sha256)
    {
        return Err(BackupError::Integrity(
            "a staged file changed while exporting",
        ));
    }

    Ok(ArchiveFile {
        path: path.to_owned(),
        byte_size,
        sha256,
    })
}

fn file_options(compression: CompressionMethod, byte_size: u64) -> SimpleFileOptions {
    SimpleFileOptions::default()
        .compression_method(compression)
        .unix_permissions(0o600)
        .large_file(byte_size >= u32::MAX as u64)
}

fn publish_archive(output: NamedTempFile, destination: &Path) -> BackupResult<()> {
    output.persist_noclobber(destination).map_err(|error| {
        if error.error.kind() == std::io::ErrorKind::AlreadyExists {
            BackupError::DestinationExists
        } else {
            BackupError::Io(error.error)
        }
    })?;

    Ok(())
}
