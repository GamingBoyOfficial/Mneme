use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct AccessScope {
    pub agent_id: String,
    pub user_id: String,
    pub session_id: String,
    pub permission: Permission,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Permission {
    ReadWrite,
    ReadOnly,
    WriteOnly,
}

impl Default for Permission {
    fn default() -> Self {
        Permission::ReadWrite
    }
}