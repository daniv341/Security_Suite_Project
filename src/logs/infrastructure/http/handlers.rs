use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::Json;
use uuid::Uuid;

use crate::logs::domain::{ActionLog, ResourceLog};
use crate::shared::state::LogState;

use super::dto::{
    PaginatedLogsResponse, PaginationQuery, LogResponse,
};
use super::error_response::{map_domain_error, HandlerError};

pub async fn get_log(
    State(state): State<Arc<LogState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<LogResponse>, HandlerError> {
    let log = state.log_service.get_log(id).await.map_err(map_domain_error)?;
    Ok(Json(log.into()))
}

pub async fn get_all_logs_by_user_id(
    State(state): State<Arc<LogState>>,
    Path(user_id): Path<Uuid>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<PaginatedLogsResponse>, HandlerError> {
    let result = state
        .log_service
        .get_all_logs_by_user_id(user_id, pagination.page, pagination.page_size)
        .await
        .map_err(map_domain_error)?;

    Ok(Json(result.map(LogResponse::from)))
}

pub async fn get_all_logs_by_resource(
    State(state): State<Arc<LogState>>,
    Path(resource): Path<ResourceLog>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<PaginatedLogsResponse>, HandlerError> {
    let result = state
        .log_service
        .get_all_logs_by_resource(&resource, pagination.page, pagination.page_size)
        .await
        .map_err(map_domain_error)?;

    Ok(Json(result.map(LogResponse::from)))
}

pub async fn get_all_logs_by_action(
    State(state): State<Arc<LogState>>,
    Path(action): Path<ActionLog>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<PaginatedLogsResponse>, HandlerError> {
    let result = state
        .log_service
        .get_all_logs_by_action(&action, pagination.page, pagination.page_size)
        .await
        .map_err(map_domain_error)?;

    Ok(Json(result.map(LogResponse::from)))
}
