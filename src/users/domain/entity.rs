//! Entidad `User`: el objeto de negocio en sí, sin traits ni errores.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// `password_hash` nunca debe salir hacia afuera del sistema en texto
/// plano ni en las respuestas HTTP; los adaptadores de entrada
/// (handlers) son responsables de mapear esta entidad a un DTO que
/// omita este campo.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    /// Construye una nueva entidad `User` a partir de un hash de
    /// contraseña ya calculado. El hasheo en sí es responsabilidad de
    /// la capa de aplicación (`UserService`), no del dominio.
    pub fn new(username: String, email: String, password_hash: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            username,
            email,
            password_hash,
            created_at: now,
            updated_at: now,
        }
    }
}
