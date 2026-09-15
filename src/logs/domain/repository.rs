//! Puerto de salida (driven port): define cómo la capa de aplicación
//! persiste y consulta `Log`, sin conocer si detrás hay
//! PostgreSQL, otra base de datos o un mock en memoria.

use async_trait::async_trait;
use uuid::Uuid;

use crate::shared::pagination::Pagination;

use super::{DomainError, Log, ResourceLog, ActionLog};

#[async_trait]
pub trait LogRepository: Send + Sync {
    async fn create(&self, log: &Log) -> Result<Log, DomainError>;

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Log>, DomainError>;

        /// Devuelve una página de resultados según `pagination`.
    async fn find_all_by_user_id(&self, user_id: Uuid, pagination: Pagination) -> Result<Vec<Log>, DomainError>;

    async fn find_all_by_resource(&self, resource: &ResourceLog, pagination: Pagination) -> Result<Vec<Log>, DomainError>;

    async fn find_all_by_action(&self, action: &ActionLog, pagination: Pagination) -> Result<Vec<Log>, DomainError>;

    /// Cantidad total, usada para calcular `total_pages`.
    async fn count_all(&self) -> Result<i64, DomainError>;
}
