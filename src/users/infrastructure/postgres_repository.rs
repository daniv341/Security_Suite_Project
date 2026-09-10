//! Adaptador de salida: implementación concreta de `UserRepository`
//! usando PostgreSQL a través de SQLx (consultas verificadas en
//! tiempo de ejecución, sin requerir `DATABASE_URL` en tiempo de
//! compilación).

use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::shared::pagination::Pagination;
use crate::users::domain::{DomainError, User, UserRepository};

pub struct PostgresUserRepository {
    pool: PgPool,
}

impl PostgresUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for PostgresUserRepository {
    async fn create(&self, user: &User) -> Result<User, DomainError> {
        sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (id, username, email, password_hash, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, username, email, password_hash, created_at, updated_at
            "#,
        )
        .bind(user.id)
        .bind(&user.username)
        .bind(&user.email)
        .bind(&user.password_hash)
        .bind(user.created_at)
        .bind(user.updated_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(db_err) if db_err.is_unique_violation() => {
                DomainError::Conflict(format!("El email '{}' ya está registrado", user.email))
            }
            other => DomainError::Repository(other.to_string()),
        })
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, DomainError> {
        sqlx::query_as::<_, User>(
            r#"
            SELECT id, username, email, password_hash, created_at, updated_at
            FROM users
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::Repository(e.to_string()))
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>, DomainError> {
        sqlx::query_as::<_, User>(
            r#"
            SELECT id, username, email, password_hash, created_at, updated_at
            FROM users
            WHERE email = $1
            "#,
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::Repository(e.to_string()))
    }

    /// Devuelve una página de usuarios (`ORDER BY created_at DESC`,
    /// con `LIMIT`/`OFFSET` calculados a partir de `pagination`).
    async fn find_all(&self, pagination: Pagination) -> Result<Vec<User>, DomainError> {
        sqlx::query_as::<_, User>(
            r#"
            SELECT id, username, email, password_hash, created_at, updated_at
            FROM users
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(pagination.limit())
        .bind(pagination.offset())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Repository(e.to_string()))
    }

    async fn count_all(&self) -> Result<i64, DomainError> {
        let (total,): (i64,) = sqlx::query_as(r#"SELECT COUNT(*) FROM users"#)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DomainError::Repository(e.to_string()))?;

        Ok(total)
    }

    async fn update(&self, user: &User) -> Result<User, DomainError> {
        sqlx::query_as::<_, User>(
            r#"
            UPDATE users
            SET username = $1, email = $2, updated_at = $3
            WHERE id = $4
            RETURNING id, username, email, password_hash, created_at, updated_at
            "#,
        )
        .bind(&user.username)
        .bind(&user.email)
        .bind(user.updated_at)
        .bind(user.id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(db_err) if db_err.is_unique_violation() => {
                DomainError::Conflict(format!("El email '{}' ya está registrado", user.email))
            }
            other => DomainError::Repository(other.to_string()),
        })?
        .ok_or(DomainError::NotFound)
    }

    async fn delete(&self, id: Uuid) -> Result<(), DomainError> {
        let result = sqlx::query(r#"DELETE FROM users WHERE id = $1"#)
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::Repository(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(DomainError::NotFound);
        }

        Ok(())
    }
}
