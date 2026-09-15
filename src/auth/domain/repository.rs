use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::users::domain::DomainError;

#[async_trait]
pub trait RevokedTokenRepository: Send + Sync {
    async fn revoke(
        &self,
        jti: Uuid,
        expires_at: DateTime<Utc>,
    ) -> Result<(), DomainError>;

    async fn is_revoked(&self, jti: Uuid) -> Result<bool, DomainError>;
}