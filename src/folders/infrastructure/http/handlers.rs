//! Adaptador de entrada: handlers de Axum para `/api/v1/folders`.

use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use uuid::Uuid;


use super::dto::{
    CreateFolderRequest, PaginatedFoldersResponse, PaginationQuery, UpdateFolderRequest,
    FolderResponse,
};
use crate::users::infrastructure::http::handlers::AppState;
use crate::users::infrastructure::http::auth::AuthenticatedUser;
use super::error_response::{map_domain_error, HandlerError};


pub async fn create_folder(
    State(state): State<Arc<AppState>>, // state inyectado con Arc<dyn FolderServicePort>
    auth: AuthenticatedUser,
    Json(payload): Json<CreateFolderRequest>, // payload inyectado con Json<CreateFolderRequest>
) -> Result<(StatusCode, Json<FolderResponse>), HandlerError> {
    let folder = state.folder_service
        .create_folder(auth.user_id, payload.name) // llama al servicio junto con los datos del payload
        .await
        .map_err(map_domain_error)?;

    Ok((StatusCode::CREATED, Json(folder.into())))
}

pub async fn get_folder(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>, // path inyectado con Path<Uuid>
    auth: AuthenticatedUser,
) -> Result<Json<FolderResponse>, HandlerError> {
    let folder = state.folder_service.get_folder(auth.user_id, id) // llama al servicio junto con el id que viene en el path
        .await.map_err(map_domain_error)?;
    Ok(Json(folder.into()))
}

pub async fn get_all_folders(
    State(state): State<Arc<AppState>>,
    Query(pagination): Query<PaginationQuery>,
    auth: AuthenticatedUser,
) -> Result<Json<PaginatedFoldersResponse>, HandlerError> {
    let result = state.folder_service
        .get_all_folders(auth.user_id, pagination.page, pagination.page_size) // llama al servicio junto con los datos de paginación que vienen en la query
        .await
        .map_err(map_domain_error)?;

    Ok(Json(result.map(FolderResponse::from)))
}


pub async fn update_folder(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    auth: AuthenticatedUser,
    Json(payload): Json<UpdateFolderRequest>,
) -> Result<Json<FolderResponse>, HandlerError> {
    let folder = state.folder_service
        .update_folder(auth.user_id, id, payload.name) // llama al servicio junto con el id que viene en el path y los datos del payload
        .await
        .map_err(map_domain_error)?;

    Ok(Json(folder.into()))
}

pub async fn delete_folder(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    auth: AuthenticatedUser,
) -> Result<StatusCode, HandlerError> {
    state.folder_service.delete_folder(auth.user_id, id) // llama al servicio junto con el id que viene en el path
    .await.map_err(map_domain_error)?;
    Ok(StatusCode::NO_CONTENT)
}
