//! Adaptador de entrada: handlers de Axum para el recurso `/api/v1/users`.
//!
//! Estos handlers solo conocen el puerto `UserServicePort`; no saben
//! nada de PostgreSQL ni de SQLx. Son responsables de:
//! - Deserializar el body/query params de la petición HTTP.
//! - Invocar el caso de uso correspondiente.
//! - Mapear la entidad de dominio a un DTO de respuesta (ver `dto`).
//! - Traducir `DomainError` a códigos HTTP (ver `error_response`).

use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use uuid::Uuid;

use crate::users::domain::UserServicePort;

use super::dto::{
    CreateUserRequest, PaginatedUsersResponse, PaginationQuery, UpdateUserRequest, UserResponse,
};
use super::dto::{LoginRequest, LoginResponse};

use crate::shared::auth::JwtService;
use super::auth::AuthenticatedUser;
use super::error_response::{map_domain_error, HandlerError};

/// Tipo compartido inyectado como estado de Axum. Al depender del
/// trait (`dyn UserServicePort`) y no de `UserService` concreto, los
/// handlers permanecen desacoplados de la implementación de la
/// capa de aplicación.
use crate::users::application::login_service::LoginService;

pub struct AppState {
    pub user_service: Arc<dyn UserServicePort>,
    pub login_service: Arc<LoginService>,
    pub jwt_service: Arc<JwtService>,
}

/// `POST /api/v1/users` — Registra un nuevo usuario.
pub async fn create_user(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<UserResponse>), HandlerError> {
    let user = state
        .user_service
        .register_user(payload.username, payload.email, payload.password)
        .await
        .map_err(map_domain_error)?;

    Ok((StatusCode::CREATED, Json(user.into())))
}

/// `GET /api/v1/users/:id` — Obtiene el perfil de un usuario por id.
pub async fn get_user(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    _auth: AuthenticatedUser, // con esta linea el endpoint queda protegido y requiere JWT
) -> Result<Json<UserResponse>, HandlerError> {
    let user = state.user_service.get_user(id).await.map_err(map_domain_error)?;
    Ok(Json(user.into()))
}

/// `GET /api/v1/users/?page=&page_size=` — Lista usuarios paginados.
pub async fn get_all_users(
    State(state): State<Arc<AppState>>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<PaginatedUsersResponse>, HandlerError> {
    let result = state
        .user_service
        .get_all_users(pagination.page, pagination.page_size)
        .await
        .map_err(map_domain_error)?;

    Ok(Json(result.map(UserResponse::from)))
}

/// `PUT /api/v1/users/:id` — Actualiza username y/o email.
pub async fn update_user(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateUserRequest>,
) -> Result<Json<UserResponse>, HandlerError> {
    let user = state
        .user_service
        .update_user(id, payload.username, payload.email)
        .await
        .map_err(map_domain_error)?;

    Ok(Json(user.into()))
}

/// `DELETE /api/v1/users/:id` — Elimina la cuenta de un usuario.
pub async fn delete_user(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, HandlerError> {
    state.user_service.delete_user(id).await.map_err(map_domain_error)?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, HandlerError> {
    let token = state
        .login_service
        .login(&payload.email, &payload.password)
        .await
        .map_err(map_domain_error)?;

    Ok(Json(LoginResponse {
        access_token: token,
        token_type: "Bearer".to_string(),
    }))
}