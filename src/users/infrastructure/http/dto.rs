//! DTOs (request/response) del adaptador HTTP de usuarios.
//!
//! Separados de los handlers para que estos últimos se queden solo
//! con la orquestación (deserializar, llamar al caso de uso, mapear
//! errores), y no con la forma de los datos de entrada/salida.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::shared::pagination::PaginatedResult;
use crate::users::domain::User;

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub username: Option<String>,
    pub email: Option<String>,
}

/// Query params de `GET /api/v1/users/?page=1&page_size=20`. Ambos
/// opcionales: si se omiten, `UserService` aplica valores por defecto.
#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

/// DTO de salida: representación pública de un usuario. Omite de
/// forma deliberada `password_hash`.
#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub token_type: String,
}

/// Respuesta paginada de `GET /api/v1/users/`.
pub type PaginatedUsersResponse = PaginatedResult<UserResponse>;
