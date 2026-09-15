//! Capa de aplicación: implementa el caso de uso `UserServicePort`
//! definido en el dominio. Aquí vive la lógica de hasheo de
//! contraseñas (Argon2) y la orquestación de llamadas al repositorio.

use std::sync::Arc;

use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;
use async_trait::async_trait;
use chrono::Utc;
use uuid::Uuid;

use crate::shared::pagination::{PaginatedResult, Pagination};
use crate::users::domain::{DomainError, User, UserRepository, UserServicePort};

/// Longitud mínima aceptada para contraseñas nuevas.
const MIN_PASSWORD_LEN: usize = 8;
/// Longitud mínima aceptada para nombres de usuario.
const MIN_USERNAME_LEN: usize = 3;

pub struct UserService {
    repository: Arc<dyn UserRepository>,
}

impl UserService {
    pub fn new(repository: Arc<dyn UserRepository>) -> Self {
        Self { repository }
    }

    /// Hashea una contraseña en texto plano usando Argon2id con una
    /// sal aleatoria distinta en cada llamada.
    fn hash_password(password: &str) -> Result<String, DomainError> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        argon2
            .hash_password(password.as_bytes(), &salt)
            .map(|hash| hash.to_string())
            .map_err(|e| DomainError::Hashing(e.to_string()))
    }

    /// Verifica una contraseña en texto plano contra un hash Argon2
    /// previamente almacenado. Se expone como utilidad reutilizable
    /// (por ejemplo, para un futuro endpoint de login).
    #[allow(dead_code)]
    pub fn verify_password(password: &str, password_hash: &str) -> Result<bool, DomainError> {
        let parsed_hash =
            PasswordHash::new(password_hash).map_err(|e| DomainError::Hashing(e.to_string()))?;

        Ok(Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }

    fn validate_username(username: &str) -> Result<(), DomainError> {
        let trimmed = username.trim();
        if trimmed.len() < MIN_USERNAME_LEN {
            return Err(DomainError::Validation(format!(
                "El nombre de usuario debe tener al menos {} caracteres",
                MIN_USERNAME_LEN
            )));
        }
        Ok(())
    }

    fn validate_email(email: &str) -> Result<(), DomainError> {
        let trimmed = email.trim();
        let is_valid = trimmed.len() >= 5
            && trimmed.contains('@')
            && trimmed.contains('.')
            && !trimmed.starts_with('@')
            && !trimmed.ends_with('@');

        if !is_valid {
            return Err(DomainError::Validation("El email no es válido".to_string()));
        }
        Ok(())
    }

    fn validate_password(password: &str) -> Result<(), DomainError> {
        if password.len() < MIN_PASSWORD_LEN {
            return Err(DomainError::Validation(format!(
                "La contraseña debe tener al menos {} caracteres",
                MIN_PASSWORD_LEN
            )));
        }
        Ok(())
    }
}

#[async_trait]
impl UserServicePort for UserService {
    async fn register_user(
        &self,
        username: String,
        email: String,
        password: String,
    ) -> Result<User, DomainError> {
        Self::validate_username(&username)?;
        Self::validate_email(&email)?;
        Self::validate_password(&password)?;

        if self.repository.find_by_email(&email).await?.is_some() {
            return Err(DomainError::Conflict(format!(
                "El email '{}' ya está registrado",
                email
            )));
        }

        let password_hash = Self::hash_password(&password)?;
        let user = User::new(username, email, password_hash);

        self.repository.create(&user).await
    }

    async fn get_user(&self, id: Uuid) -> Result<User, DomainError> {
        self.repository
            .find_by_id(id)
            .await?
            .ok_or(DomainError::NotFound)
    }

    async fn get_all_users(
        &self,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<PaginatedResult<User>, DomainError> {
        let pagination = Pagination::new(page, page_size);
        let users = self.repository.find_all(pagination).await?;
        let total_users = self.repository.count_all().await?;
        Ok(PaginatedResult::new(users, pagination, total_users))
    }

    async fn update_user(
        &self,
        id: Uuid,
        username: Option<String>,
        email: Option<String>,
    ) -> Result<User, DomainError> {
        let mut user = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or(DomainError::NotFound)?;

        if let Some(new_username) = username {
            Self::validate_username(&new_username)?;
            user.username = new_username;
        }

        if let Some(new_email) = email {
            Self::validate_email(&new_email)?;

            if let Some(existing) = self.repository.find_by_email(&new_email).await? {
                if existing.id != user.id {
                    return Err(DomainError::Conflict(format!(
                        "El email '{}' ya está registrado",
                        new_email
                    )));
                }
            }

            user.email = new_email;
        }

        user.updated_at = Utc::now();
        self.repository.update(&user).await
    }

    async fn delete_user(&self, id: Uuid) -> Result<(), DomainError> {
        self.repository
            .find_by_id(id)
            .await?
            .ok_or(DomainError::NotFound)?;

        self.repository.delete(id).await
    }
}
