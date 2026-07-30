use serde::{Deserialize, Serialize};

pub const FORMAT_VERSION: i64 = 1;
pub const MANIFEST_TABLE: &str = "lf_backup_manifest";

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupSummary {
    pub backup_id: String,
    pub created_at: String,
    pub app_version: String,
    pub source_platform: String,
    pub format_version: i64,
    pub database_schema: i64,
    pub profile_count: u64,
    pub palette_count: u64,
    pub track_count: u64,
    pub file_size: u64,
    pub integrity_ok: bool,
    pub audio_included: bool,
    pub path: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupInspection {
    pub summary: BackupSummary,
    pub source_path: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RestorePrepared {
    pub summary: BackupSummary,
    pub emergency_backup_path: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreResult {
    pub success: bool,
    pub summary: Option<BackupSummary>,
    pub emergency_backup_path: String,
    pub error: String,
}

#[derive(Debug)]
pub(crate) struct PackageData {
    pub summary: BackupSummary,
    pub config_json: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PendingRestore {
    pub format_version: i64,
    pub expected_backup_id: String,
    pub source_path: String,
    pub emergency_backup_path: String,
}
