//! Puerto de entrada (driving port): define los casos de uso que
//! ofrece el módulo `logs`. Los adaptadores de entrada
//! (handlers HTTP, etc.) solo dependen de esta interfaz.

use async_trait::async_trait;
use uuid::Uuid;

use crate::shared::pagination::PaginatedResult;

use super::{DomainError, Log, ResourceLog, ActionLog};

#[async_trait]
pub trait LogServicePort: Send + Sync {
    async fn create_log(
        &self, 
        user_id: Option<Uuid>, 
        action: ActionLog, 
        resource: ResourceLog, 
        resource_id: Option<Uuid>, 
        details: String
    ) -> Result<Log, DomainError>;

    async fn get_log(&self, id: Uuid) -> Result<Log, DomainError>;

    /// `page`/`page_size` opcionales: `None` aplica los valores por
    /// defecto de `shared::pagination`.
    async fn get_all_logs_by_user_id(
        &self,
        user_id: Uuid,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<PaginatedResult<Log>, DomainError>;

    async fn get_all_logs_by_resource(
        &self,
        resource: &ResourceLog,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<PaginatedResult<Log>, DomainError>;

    async fn get_all_logs_by_action(
        &self,
        action: &ActionLog,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<PaginatedResult<Log>, DomainError>;
}
