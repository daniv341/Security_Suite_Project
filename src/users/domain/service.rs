//! Puerto de entrada (driving port): define los casos de uso que
//! puede ofrecer el módulo de usuarios. Los adaptadores de entrada
//! (handlers HTTP, CLI, un futuro gRPC, etc.) solo dependen de esta
//! interfaz, nunca de la implementación concreta (`UserService`).

use async_trait::async_trait;
use uuid::Uuid;

use crate::shared::pagination::PaginatedResult;

use super::{DomainError, User};

#[async_trait]
pub trait UserServicePort: Send + Sync {
    async fn register_user(
        &self,
        username: String,
        email: String,
        password: String,
    ) -> Result<User, DomainError>;

    async fn get_user(&self, id: Uuid) -> Result<User, DomainError>;

    /// `page` y `page_size` son opcionales: si no se proveen, se
    /// aplican valores por defecto razonables (ver `shared::pagination`).
    async fn get_all_users(
        &self,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<PaginatedResult<User>, DomainError>;

    async fn update_user(
        &self,
        id: Uuid,
        username: Option<String>,
        email: Option<String>,
    ) -> Result<User, DomainError>;

    async fn delete_user(&self, id: Uuid) -> Result<(), DomainError>;
}
