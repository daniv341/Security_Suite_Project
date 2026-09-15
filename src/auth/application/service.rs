use std::sync::Arc;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::auth::domain::RevokedTokenRepository;
use crate::users::domain::DomainError;

pub struct AuthService {
    repository: Arc<dyn RevokedTokenRepository>,
}

impl AuthService {
    pub fn new(repository: Arc<dyn RevokedTokenRepository>) -> Self {
        Self { repository }
    }

    pub async fn revoke_token(
        &self,
        jti: Uuid,
        expires_at: DateTime<Utc>,
    ) -> Result<(), DomainError> {
        self.repository.revoke(jti, expires_at).await
    }

    pub async fn is_token_revoked(
        &self,
        jti: Uuid,
    ) -> Result<bool, DomainError> {
        self.repository.is_revoked(jti).await
    }
}