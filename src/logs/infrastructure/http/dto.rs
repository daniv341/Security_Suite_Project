//! DTOs (request/response) del adaptador HTTP de `logs`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::shared::pagination::PaginatedResult;
use crate::logs::domain::{Log, ActionLog, ResourceLog};

/// Query params de `GET /api/v1/logs/?page=&page_size=`.
#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct LogResponse {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub action: ActionLog,
    pub resource: ResourceLog,
    pub resource_id: Option<Uuid>,
    pub details: String,
    pub created_at: DateTime<Utc>,
}

impl From<Log> for LogResponse {
    fn from(log: Log) -> Self {
        Self {
            id: log.id,
            user_id: log.user_id,
            action: log.action,
            resource: log.resource,
            resource_id: log.resource_id,
            details: log. details,
            created_at: log.created_at,
        }
    }
}

pub type PaginatedLogsResponse = PaginatedResult<LogResponse>;
