use async_trait::async_trait;
use uuid::Uuid;

use crate::shared::pagination::PaginatedResult;

use super::{DomainError, Folder};

#[async_trait]
pub trait FolderServicePort: Send + Sync {
    // crea un folder para un usuario
    async fn create_folder(&self, 
        user_id: Uuid,
        name: String,
    ) -> Result<Folder, DomainError>;

    // obtiene todos los folders de un usuaio por id
    async fn get_folder(&self, user_id: Uuid, id: Uuid) -> Result<Folder, DomainError>;

    // obtiene todos los folders de un usuario
    async fn get_all_folders(
        &self,
        user_id: Uuid,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<PaginatedResult<Folder>, DomainError>;

    // actualiza un folder existente por id y por nombre de forma opcional
    async fn update_folder(&self, user_id: Uuid, id: Uuid, name: Option<String>) -> Result<Folder, DomainError>;

    // elimina un folder por id
    async fn delete_folder(&self, user_id: Uuid, id: Uuid) -> Result<(), DomainError>;
}
