use serde::{Deserialize, Serialize};
use crate::store::MemoryRecord;
use crate::audit::AuditEvent;
use std::fs;
use anyhow::Result;

#[derive(Debug, Serialize, Deserialize)]
pub struct MnemeArchive {
    pub version: String,
    pub exported_at: chrono::DateTime<chrono::Utc>,
    pub records: Vec<MemoryRecord>,
    pub audit_log: Vec<AuditEvent>,
}

pub fn export(records: Vec<MemoryRecord>, audit_log: Vec<AuditEvent>, path: &str) -> Result<()> {
    let archive = MnemeArchive {
        version: "1.0".to_string(),
        exported_at: chrono::Utc::now(),
        records,
        audit_log,
    };
    let json = serde_json::to_string_pretty(&archive)?;
    fs::write(path, json)?;
    Ok(())
}

pub fn import(path: &str) -> Result<MnemeArchive> {
    let data = fs::read_to_string(path)?;
    let archive: MnemeArchive = serde_json::from_str(&data)?;
    Ok(archive)
}