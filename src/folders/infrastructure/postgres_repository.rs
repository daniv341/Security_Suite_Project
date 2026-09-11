use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::shared::pagination::Pagination;
use crate::folders::domain::{DomainError, Folder, FolderRepository};

// repositorio de folders basado en postgre
pub struct PostgresFolderRepository {
    pool: PgPool,
}

// implementación de métodos para el repositorio de folders
impl PostgresFolderRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl FolderRepository for PostgresFolderRepository {
    // crea un folder en la base de datos
    async fn create(&self, folder: &Folder) -> Result<Folder, DomainError> {
        sqlx::query_as::<_, Folder>( // query para mapear el resultado
            r#"
            INSERT INTO folders (id, user_id, name, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, user_id, name, created_at, updated_at
            "#,
        ) // bind de los valores del folder
        .bind(folder.id)
        .bind(&folder.user_id)
        .bind(&folder.name)
        .bind(folder.created_at)
        .bind(folder.updated_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e { // errores de la base de datos
            sqlx::Error::Database(db_err) if db_err.is_unique_violation() => DomainError::Conflict(
                format!("Ya existe un/a folder con el name '{}'", folder.name),
            ),
            other => DomainError::Repository(other.to_string()),
        })
    }

    // obtiene un folder por id de la base de datos
    async fn find_by_id(&self, user_id: Uuid, id: Uuid) -> Result<Option<Folder>, DomainError> {
        sqlx::query_as::<_, Folder>(
            r#"SELECT id, user_id, name, created_at, updated_at FROM folders WHERE id = $1 AND user_id=$2"#,
        )
        .bind(id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::Repository(e.to_string()))
    }

    // obtiene un folder por user_id de la base de datos
    async fn find_by_user_id(&self, user_id: Uuid) -> Result<Vec<Folder>, DomainError> {
        sqlx::query_as::<_, Folder>(
            r#"SELECT id, user_id, name, created_at, updated_at FROM folders WHERE user_id = $1"#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Repository(e.to_string()))
    }

    // obtiene un folder por name de la base de datos
    async fn find_by_name(&self, user_id: Uuid, name: &str) -> Result<Option<Folder>, DomainError> {
        sqlx::query_as::<_, Folder>(
            r#"SELECT id, user_id, name, created_at, updated_at FROM folders WHERE name = $1 AND user_id=$2"#,
        )
        .bind(name)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::Repository(e.to_string()))
    }

    // obtiene todos los folders de la base de datos con paginación
    async fn find_all(&self, user_id: Uuid, pagination: Pagination) -> Result<Vec<Folder>, DomainError> {
        sqlx::query_as::<_, Folder>(
            r#"
            SELECT id, user_id, name, created_at, updated_at
            FROM folders
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

    // cuenta todos los folders de la base de datos
    async fn count_all(&self, user_id: Uuid) -> Result<i64, DomainError> {
        let (total,): (i64,) = sqlx::query_as(r#"SELECT COUNT(*) FROM folders WHERE user_id = $1"#)
            .bind(user_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DomainError::Repository(e.to_string()))?;

        Ok(total)
    }

    // actualiza un folder en la base de datos
    async fn update(&self, user_id: Uuid, folder: &Folder) -> Result<Folder, DomainError> {
        sqlx::query_as::<_, Folder>(
            r#"
            UPDATE folders
            SET name = $1, updated_at = $2
            WHERE id = $3 AND user_id=$4
            RETURNING id, user_id, name, created_at, updated_at
            "#,
        )
        .bind(&folder.name)
        .bind(folder.updated_at)
        .bind(folder.id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(db_err) if db_err.is_unique_violation() => DomainError::Conflict(
                format!("Ya existe un/a folder con el name '{}'", folder.name),
            ),
            other => DomainError::Repository(other.to_string()),
        })?
        .ok_or(DomainError::NotFound)
    }

    // elimina un folder de la base de datos
    async fn delete(&self, user_id: Uuid, id: Uuid) -> Result<(), DomainError> {
        let result = sqlx::query(r#"DELETE FROM folders WHERE id = $1 AND user_id=$2"#)
            .bind(id)
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::Repository(e.to_string()))?;

        // si no elimino ningun folder, devuelve un error de no encontrado
        if result.rows_affected() == 0 {
            return Err(DomainError::NotFound);
        }

        Ok(())
    }
}
