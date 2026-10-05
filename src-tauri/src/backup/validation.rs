use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::Path;

use rusqlite::{Connection, OpenFlags};
use sha2::{Digest, Sha256};
use zip::ZipArchive;

use crate::data::current_schema_connection;

use super::checksum::{copy_and_digest, finish_digest};
use super::models::{
    ArchiveManifest, ARCHIVE_FORMAT, ARCHIVE_FORMAT_VERSION, DATABASE_PATH, LIBRARY_PATH,
    MANIFEST_PATH,
};
use super::portable::{library_counts, schema_fingerprint, write_library};
use super::{ArchiveKind, BackupError, BackupPreview, BackupResult};

pub(super) const MAX_MANIFEST_BYTES: u64 = 8 * 1024 * 1024;
const MAX_DATABASE_BYTES: u64 = 2 * 1024 * 1024 * 1024;
const MAX_TOTAL_BYTES: u64 = 8 * 1024 * 1024 * 1024;

pub(super) fn check_size_limits(files: &[super::models::ArchiveFile]) -> BackupResult<()> {
    if files.len() + 1 > 100_000 {
        return Err(BackupError::ArchiveLimit);
    }

    let mut total = 0_u64;

    for file in files {
        if path_limit(&file.path).is_none_or(|limit| file.byte_size == 0 || file.byte_size > limit)
        {
            return Err(BackupError::ArchiveLimit);
        }

        total = total
            .checked_add(file.byte_size)
            .filter(|size| *size <= MAX_TOTAL_BYTES)
            .ok_or(BackupError::ArchiveLimit)?;
    }

    Ok(())
}

pub fn inspect_backup(source: impl AsRef<Path>) -> BackupResult<BackupPreview> {
    let staging = tempfile::tempdir()?;

    extract_backup(source.as_ref(), staging.path())
}

pub(super) fn extract_backup(source: &Path, staging: &Path) -> BackupResult<BackupPreview> {
    let mut input = File::open(source)?;

    if input.metadata()?.len() > MAX_TOTAL_BYTES + 64 * 1024 * 1024 {
        return Err(BackupError::InvalidArchive("the archive is too large"));
    }

    super::zip_directory::check_directory(&mut input)?;

    let mut archive = ZipArchive::new(input)?;
    let manifest = {
        let mut entry = archive.by_name(MANIFEST_PATH)?;

        if entry.size() > MAX_MANIFEST_BYTES {
            return Err(BackupError::InvalidArchive("the manifest is too large"));
        }

        let mut bytes = Vec::new();

        entry
            .by_ref()
            .take(MAX_MANIFEST_BYTES + 1)
            .read_to_end(&mut bytes)?;

        if bytes.len() as u64 > MAX_MANIFEST_BYTES {
            return Err(BackupError::InvalidArchive("the manifest is too large"));
        }

        serde_json::from_slice::<ArchiveManifest>(&bytes)
            .map_err(|_| BackupError::InvalidArchive("the manifest is invalid"))?
    };
    let files = check_manifest(&manifest)?;

    if archive.len() != files.len() + 1 {
        return Err(BackupError::InvalidArchive(
            "the file list does not match the archive",
        ));
    }

    fs::create_dir_all(staging.join("media"))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        fs::set_permissions(staging.join("media"), fs::Permissions::from_mode(0o700))?;
    }

    for index in 0..archive.len() {
        let mut entry = archive.by_index(index)?;

        if entry.name_raw() != entry.name().as_bytes()
            || entry.encrypted()
            || entry.unix_mode().is_some_and(|mode| {
                let kind = mode & 0o170000;
                kind != 0 && kind != 0o100000
            })
        {
            return Err(BackupError::InvalidArchive(
                "only regular, unencrypted files are supported",
            ));
        }

        if entry.name() == MANIFEST_PATH {
            continue;
        }

        let expected = files
            .get(entry.name())
            .ok_or(BackupError::InvalidArchive("an unexpected file is present"))?;

        if entry.size() != expected.byte_size {
            return Err(BackupError::InvalidArchive("a file size does not match"));
        }

        let mut output = create_staged_file(&staging.join(&expected.path))?;
        let (size, digest) =
            copy_and_digest(&mut entry, &mut output, expected.byte_size).map_err(|error| {
                match error {
                    BackupError::Integrity(_) => {
                        BackupError::InvalidArchive("a file exceeds its declared size")
                    }
                    other => other,
                }
            })?;

        if size != expected.byte_size || digest != expected.sha256 {
            return Err(BackupError::InvalidArchive(
                "a file checksum does not match",
            ));
        }

        output.sync_all()?;
    }

    let mut output = create_staged_file(&staging.join(MANIFEST_PATH))?;

    serde_json::to_writer(&mut output, &manifest)?;
    output.sync_all()?;

    validate_snapshot(staging, &manifest)
}

fn create_staged_file(path: &Path) -> BackupResult<File> {
    let mut options = OpenOptions::new();

    options.write(true).create_new(true);

    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;

        options.mode(0o600);
    }

    Ok(options.open(path)?)
}

pub(super) fn validate_staged(staging: &Path) -> BackupResult<BackupPreview> {
    let path = staging.join(MANIFEST_PATH);

    if !fs::symlink_metadata(&path)?.is_file() || fs::metadata(&path)?.len() > MAX_MANIFEST_BYTES {
        return Err(BackupError::InvalidArchive(
            "the staged manifest is invalid",
        ));
    }

    let manifest: ArchiveManifest = serde_json::from_reader(File::open(path)?)?;
    let files = check_manifest(&manifest)?;

    for file in files.values() {
        let path = staging.join(&file.path);
        let metadata = fs::symlink_metadata(&path)?;

        if !metadata.is_file() || metadata.len() != file.byte_size {
            return Err(BackupError::InvalidArchive("a staged file is invalid"));
        }

        let (_, digest) = copy_and_digest(&mut File::open(path)?, &mut io::sink(), file.byte_size)?;

        if digest != file.sha256 {
            return Err(BackupError::InvalidArchive("a staged file has changed"));
        }
    }

    validate_snapshot(staging, &manifest)
}

fn check_manifest(
    manifest: &ArchiveManifest,
) -> BackupResult<BTreeMap<&str, &super::models::ArchiveFile>> {
    if manifest.format != ARCHIVE_FORMAT
        || manifest.format_version != ARCHIVE_FORMAT_VERSION
        || manifest.schema_version != 1
    {
        return Err(BackupError::IncompatibleArchive);
    }

    if manifest.kind != ArchiveKind::Backup {
        return Err(BackupError::InvalidArchive(
            "a portability export cannot be restored",
        ));
    }

    if manifest.created_at < 0
        || manifest.app_version.is_empty()
        || manifest.app_version.len() > 100
        || !valid_digest(&manifest.schema_fingerprint)
    {
        return Err(BackupError::InvalidArchive(
            "the manifest metadata is invalid",
        ));
    }

    let mut files = BTreeMap::new();

    check_size_limits(&manifest.files)
        .map_err(|_| BackupError::InvalidArchive("the archive exceeds its size limits"))?;

    for file in &manifest.files {
        let limit = path_limit(&file.path)
            .filter(|_| file.path != MANIFEST_PATH)
            .ok_or(BackupError::InvalidArchive(
                "an unsupported file path is present",
            ))?;

        if file.byte_size == 0
            || file.byte_size > limit
            || !valid_digest(&file.sha256)
            || files.insert(file.path.as_str(), file).is_some()
        {
            return Err(BackupError::InvalidArchive("the file list is invalid"));
        }
    }

    if ![DATABASE_PATH, LIBRARY_PATH, "README.md"]
        .iter()
        .all(|name| files.contains_key(name))
    {
        return Err(BackupError::InvalidArchive("a required file is missing"));
    }

    Ok(files)
}

pub(super) fn path_limit(path: &str) -> Option<u64> {
    match path {
        DATABASE_PATH | LIBRARY_PATH => Some(MAX_DATABASE_BYTES),
        MANIFEST_PATH => Some(MAX_MANIFEST_BYTES),
        "README.md" => Some(64 * 1024),
        _ => {
            let name = path.strip_prefix("media/")?;
            let (digest, extension) = name.rsplit_once('.')?;

            (valid_digest(digest) && matches!(extension, "png" | "jpg" | "gif" | "webp"))
                .then_some(20 * 1024 * 1024)
        }
    }
}

fn valid_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn validate_snapshot(staging: &Path, manifest: &ArchiveManifest) -> BackupResult<BackupPreview> {
    // Open the isolated copy writable so SQLite also checks CHECK constraints
    let connection = Connection::open_with_flags(
        staging.join(DATABASE_PATH),
        OpenFlags::SQLITE_OPEN_READ_WRITE,
    )?;

    connection.pragma_update(None, "query_only", true)?;
    connection.pragma_update(None, "trusted_schema", false)?;

    let version: i64 = connection.query_row("PRAGMA user_version", [], |row| row.get(0))?;
    let fingerprint = schema_fingerprint(&connection)?;

    if version != 1
        || fingerprint != manifest.schema_fingerprint
        || fingerprint != schema_fingerprint(&current_schema_connection()?)?
    {
        return Err(BackupError::IncompatibleArchive);
    }

    let integrity: String = connection.query_row("PRAGMA integrity_check", [], |row| row.get(0))?;

    if integrity != "ok" || connection.prepare("PRAGMA foreign_key_check")?.exists([])? {
        return Err(BackupError::InvalidArchive(
            "the database failed its integrity check",
        ));
    }

    if library_counts(&connection)? != manifest.counts {
        return Err(BackupError::InvalidArchive(
            "the library counts do not match",
        ));
    }

    crate::library::validate_restored_content(&connection)
        .map_err(|_| BackupError::InvalidArchive("the authored content is invalid"))?;

    let files = check_manifest(manifest)?;
    validate_media(&connection, &files)?;

    let mut readable = DigestWriter(Sha256::new());

    write_library(&connection, &mut readable, manifest.created_at).map_err(
        |error| match error {
            BackupError::Integrity(_) | BackupError::Json(_) => {
                BackupError::InvalidArchive("the readable library contains invalid data")
            }
            other => other,
        },
    )?;

    if finish_digest(readable.0) != files[LIBRARY_PATH].sha256 {
        return Err(BackupError::InvalidArchive(
            "the readable library does not match the database",
        ));
    }

    let mut digest = Sha256::new();

    digest.update(serde_json::to_vec(manifest)?);

    Ok(BackupPreview {
        created_at: manifest.created_at,
        app_version: manifest.app_version.clone(),
        counts: manifest.counts.clone(),
        fingerprint: finish_digest(digest),
    })
}

fn validate_media(
    connection: &Connection,
    files: &BTreeMap<&str, &super::models::ArchiveFile>,
) -> BackupResult<()> {
    let mut expected_media = BTreeSet::new();
    let mut statement = connection.prepare(
        "SELECT digest, file_extension, byte_size FROM media
        INNER JOIN entities ON entities.id = media.entity_id
        WHERE entities.deleted_at IS NULL",
    )?;
    let media = statement.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, i64>(2)?,
        ))
    })?;

    for asset in media {
        let (digest, extension, size) = asset?;
        let path = format!("media/{digest}.{extension}");
        let file = files
            .get(path.as_str())
            .ok_or(BackupError::InvalidArchive("a required image is missing"))?;

        if file.sha256 != digest || file.byte_size != size as u64 {
            return Err(BackupError::InvalidArchive(
                "an image does not match the database",
            ));
        }

        expected_media.insert(path);
    }

    let actual_media = files
        .keys()
        .filter(|path| path.starts_with("media/"))
        .map(|path| (*path).to_owned())
        .collect::<BTreeSet<_>>();

    if actual_media != expected_media {
        return Err(BackupError::InvalidArchive(
            "the image list does not match the database",
        ));
    }

    Ok(())
}

struct DigestWriter(Sha256);

impl Write for DigestWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0.update(bytes);

        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
