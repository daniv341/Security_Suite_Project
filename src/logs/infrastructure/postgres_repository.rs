//! Adaptador de salida: implementación concreta de `LogRepository`
//! usando PostgreSQL a través de SQLx (consultas verificadas en
//! tiempo de ejecución, sin requerir `DATABASE_URL` en tiempo de
//! compilación).
//!
//! Requiere una tabla `logs` con columnas
//! `(id, name, created_at, updated_at)` — ver `print_next_steps` del
//! script para un ejemplo de migración.

use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::shared::pagination::Pagination;
use crate::logs::domain::{DomainError, Log, LogRepository, ActionLog, ResourceLog};

pub struct PostgresLogRepository {
    pool: PgPool,
}

impl PostgresLogRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl LogRepository for PostgresLogRepository {
    async fn create(&self, log: &Log) -> Result<Log, DomainError> {
        sqlx::query_as::<_, Log>(
            r#"
            INSERT INTO logs (id, user_id, action, resource, resource_id, details, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, name, created_at, updated_at
            "#,
        )
        .bind(log.id)
        .bind(&log.user_id)
        .bind(&log.action)
        .bind(&log.resource)
        .bind(&log.resource_id)
        .bind(&log.details)
        .bind(log.created_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DomainError::Repository(e.to_string()))
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Log>, DomainError> {
        sqlx::query_as::<_, Log>(
            r#"SELECT id, user_id, action, resource, resource_id, details, created_at FROM logs WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::Repository(e.to_string()))
    }

    async fn find_all_by_user_id(&self, user_id: Uuid, pagination: Pagination) -> Result<Vec<Log>, DomainError> {
        sqlx::query_as::<_, Log>(
            r#"
            SELECT id, user_id, action, resource, resource_id, details, created_at
            FROM logs
            WHERE user_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(user_id)
        .bind(pagination.limit())
        .bind(pagination.offset())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Repository(e.to_string()))
    }

    async fn find_all_by_resource(&self, resource: &ResourceLog, pagination: Pagination) -> Result<Vec<Log>, DomainError> {
        sqlx::query_as::<_, Log>(
            r#"
            SELECT id, user_id, action, resource, resource_id, details, created_at
            FROM logs
            WHERE resource = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(resource)
        .bind(pagination.limit())
        .bind(pagination.offset())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Repository(e.to_string()))
    }

    async fn find_all_by_action(&self, action: &ActionLog, pagination: Pagination) -> Result<Vec<Log>, DomainError> {
        sqlx::query_as::<_, Log>(
            r#"
            SELECT id, user_id, action, resource, resource_id, details, created_at
            FROM logs
            WHERE action = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(action)
        .bind(pagination.limit())
        .bind(pagination.offset())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Repository(e.to_string()))
    }

    async fn count_all(&self) -> Result<i64, DomainError> {
        let (total,): (i64,) = sqlx::query_as(r#"SELECT COUNT(*) FROM logs"#)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DomainError::Repository(e.to_string()))?;

        Ok(total)
    }
}
