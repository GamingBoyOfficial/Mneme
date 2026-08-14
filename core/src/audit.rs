use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AuditAction {
    Write,
    Read,
    Forget,
    ForgetAll,
    Export,
    Import,
}

impl ToString for AuditAction {
    fn to_string(&self) -> String {
        match self {
            AuditAction::Write => "Write".to_string(),
            AuditAction::Read => "Read".to_string(),
            AuditAction::Forget => "Forget".to_string(),
            AuditAction::ForgetAll => "ForgetAll".to_string(),
            AuditAction::Export => "Export".to_string(),
            AuditAction::Import => "Import".to_string(),
        }
    }
}

impl From<String> for AuditAction {
    fn from(s: String) -> Self {
        match s.as_str() {
            "Write" => AuditAction::Write,
            "Read" => AuditAction::Read,
            "Forget" => AuditAction::Forget,
            "ForgetAll" => AuditAction::ForgetAll,
            "Export" => AuditAction::Export,
            "Import" => AuditAction::Import,
            _ => AuditAction::Write,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub action: AuditAction,
    pub agent_id: String,
    pub user_id: String,
    pub session_id: String,
    pub memory_id: Option<String>,
    pub details: String,
}

impl AuditEvent {
    pub fn new(
        action: AuditAction,
        agent_id: &str,
        user_id: &str,
        session_id: &str,
        memory_id: Option<&str>,
        details: &str,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            action,
            agent_id: agent_id.to_string(),
            user_id: user_id.to_string(),
            session_id: session_id.to_string(),
            memory_id: memory_id.map(|s| s.to_string()),
            details: details.to_string(),
        }
    }
}