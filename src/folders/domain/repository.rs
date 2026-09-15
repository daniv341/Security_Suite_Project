use async_trait::async_trait;
use uuid::Uuid;

use crate::shared::pagination::Pagination;

use super::{DomainError, Folder};

#[async_trait]
pub trait FolderRepository: Send + Sync {
    //crea un nuevo folder
    async fn create(&self, folder: &Folder) -> Result<Folder, DomainError>;
    // busca un folder por id
    async fn find_by_id(&self, user_id: Uuid, id: Uuid) -> Result<Option<Folder>, DomainError>;

    // busca un folder por user_id
    async fn find_by_user_id(&self, user_id: Uuid) -> Result<Vec<Folder>, DomainError>;

    // busca un folder por name
    async fn find_by_name(&self, user_id: Uuid, name: &str,) -> Result<Option<Folder>, DomainError>;

    /// Devuelve una página de resultados según `pagination`.
    async fn find_all(&self, user_id: Uuid, pagination: Pagination) -> Result<Vec<Folder>, DomainError>;

    /// Cantidad total, usada para calcular `total_pages`.
    async fn count_all(&self, user_id: Uuid) -> Result<i64, DomainError>;

    // actualiza un folder existente por id
    async fn update(&self, user_id: Uuid, folder: &Folder) -> Result<Folder, DomainError>; // el & es para que se pueda usar el mismo user sin moverlo
    // elimina un folder por id
    async fn delete(&self, user_id:Uuid, id: Uuid) -> Result<(), DomainError>;
}
