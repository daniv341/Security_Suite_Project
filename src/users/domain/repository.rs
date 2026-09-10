//! Puerto de salida (driven port): define cómo la capa de aplicación
//! persiste y consulta usuarios, sin conocer si detrás hay PostgreSQL,
//! otra base de datos o un mock en memoria.

use async_trait::async_trait;
use uuid::Uuid;

use crate::shared::pagination::Pagination;

use super::{DomainError, User};

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create(&self, user: &User) -> Result<User, DomainError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, DomainError>;
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, DomainError>;

    /// Devuelve una página de usuarios según `pagination`.
    async fn find_all(&self, pagination: Pagination) -> Result<Vec<User>, DomainError>;

    /// Cantidad total de usuarios, usada para calcular `total_pages`.
    async fn count_all(&self) -> Result<i64, DomainError>;

    async fn update(&self, user: &User) -> Result<User, DomainError>;
    async fn delete(&self, id: Uuid) -> Result<(), DomainError>;
}
