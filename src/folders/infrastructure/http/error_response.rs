//! Traducción de `DomainError` a respuestas HTTP.

use axum::http::StatusCode;
use axum::Json;
use serde::Serialize;

use crate::folders::domain::DomainError;

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

pub type HandlerError = (StatusCode, Json<ErrorResponse>);

pub fn map_domain_error(err: DomainError) -> HandlerError {
    let (status, message) = match err {
        DomainError::NotFound => (StatusCode::NOT_FOUND, err.to_string()),
        DomainError::Validation(msg) => (StatusCode::BAD_REQUEST, msg),
        DomainError::Conflict(msg) => (StatusCode::CONFLICT, msg),
        DomainError::Repository(msg) => {
            tracing::error!("error de repositorio: {msg}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error interno del servidor".to_string(),
            )
        }
    };

    (status, Json(ErrorResponse { error: message }))
}
