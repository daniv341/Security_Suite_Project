//! Entidad `Log`: el objeto de negocio en sí, sin traits ni errores.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "action_log", rename_all = "PascalCase")]
pub enum ActionLog {
    Create,
    Update,
    Delete,
    Get,
    GetAll,
    Login,
    Logout,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "resource_log", rename_all = "PascalCase")]
pub enum ResourceLog {
    Folder,
    File,
    User,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Log {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub action: ActionLog,
    pub resource: ResourceLog,
    pub resource_id: Option<Uuid>,
    pub details: String,
    pub created_at: DateTime<Utc>,
}

impl Log {
    pub fn new(user_id: Option<Uuid>, action: ActionLog, resource: ResourceLog, resource_id: Option<Uuid>, details: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            user_id,
            action,
            resource,
            resource_id,
            details,
            created_at: now,
        }
    }
}
