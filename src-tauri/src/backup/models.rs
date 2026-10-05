use serde::{Deserialize, Serialize};

pub(super) const ARCHIVE_FORMAT: &str = "twill";
pub(super) const ARCHIVE_FORMAT_VERSION: u32 = 1;
pub(super) const DATABASE_PATH: &str = "twill.sqlite3";
pub(super) const LIBRARY_PATH: &str = "library.json";
pub(super) const MANIFEST_PATH: &str = "manifest.json";

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ArchiveKind {
    Backup,
    Export,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LibraryCounts {
    pub concepts: u64,
    pub cards: u64,
    pub reviews: u64,
    pub media: u64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveSummary {
    pub kind: ArchiveKind,
    pub created_at: i64,
    pub byte_size: u64,
    pub counts: LibraryCounts,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupPreview {
    pub created_at: i64,
    pub app_version: String,
    pub counts: LibraryCounts,
    pub fingerprint: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct ArchiveManifest {
    pub format: String,
    pub format_version: u32,
    pub kind: ArchiveKind,
    pub app_version: String,
    pub created_at: i64,
    pub schema_version: i64,
    pub schema_fingerprint: String,
    pub counts: LibraryCounts,
    pub files: Vec<ArchiveFile>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct ArchiveFile {
    pub path: String,
    pub byte_size: u64,
    pub sha256: String,
}
