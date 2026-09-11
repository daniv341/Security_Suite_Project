//! DTOs (request/response) del adaptador HTTP de `folders`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::shared::pagination::PaginatedResult;
use crate::folders::domain::Folder;

#[derive(Debug, Deserialize)]
pub struct CreateFolderRequest {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateFolderRequest {
    pub name: Option<String>,
}

/// Query params de `GET /api/v1/folders/?page=&page_size=`.
#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct FolderResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Folder> for FolderResponse {
    fn from(folder: Folder) -> Self {
        Self {
            id: folder.id,
            user_id: folder.user_id,
            name: folder.name,
            created_at: folder.created_at,
            updated_at: folder.updated_at,
        }
    }
}

pub type PaginatedFoldersResponse = PaginatedResult<FolderResponse>;
