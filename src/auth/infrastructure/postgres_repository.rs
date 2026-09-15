use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::auth::domain::RevokedTokenRepository;
use crate::users::domain::DomainError;

pub struct PostgresRevokedTokenRepository {
    pool: PgPool,
}

impl PostgresRevokedTokenRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl RevokedTokenRepository for PostgresRevokedTokenRepository {
    async fn revoke(
        &self,
        jti: Uuid,
        expires_at: DateTime<Utc>,
    ) -> Result<(), DomainError> {
        sqlx::query(
            r#"
            INSERT INTO revoked_tokens (jti, expires_at)
            VALUES ($1, $2)
            ON CONFLICT (jti) DO NOTHING
            "#,
        )
        .bind(jti)
        .bind(expires_at)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Repository(e.to_string()))?;

        Ok(())
    }

    async fn is_revoked(&self, jti: Uuid) -> Result<bool, DomainError> {
        let exists = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS(
                SELECT 1
                FROM revoked_tokens
                WHERE jti = $1
            )
            "#,
        )
        .bind(jti)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DomainError::Repository(e.to_string()))?;

        Ok(exists)
    }
}