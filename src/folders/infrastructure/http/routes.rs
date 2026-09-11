//! Configuración del `Router` de Axum para `/api/v1/folders`.

use std::time::Duration;

use axum::body::Body;
use axum::http::{Request, Response};
use axum::routing::{get, post};
use axum::Router;
use tower_http::trace::TraceLayer;
use tracing::Span;

use std::sync::Arc;

use crate::users::infrastructure::http::handlers::AppState;

use super::handlers;

pub fn folders_routes(state: Arc<AppState>) -> Router {
    // Log de request resumido: una sola línea por petición (método,
    // path, status, latencia), igual que en `users`.
    let trace_layer = TraceLayer::new_for_http()
        .make_span_with(|request: &Request<Body>| {
            tracing::info_span!(
                "http",
                method = %request.method(),
                path = %request.uri().path(),
            )
        })
        .on_request(|_request: &Request<Body>, _span: &Span| {})
        .on_response(|response: &Response<Body>, latency: Duration, _span: &Span| {
            tracing::info!(
                status = response.status().as_u16(),
                latency_ms = latency.as_millis(),
                "request"
            );
        });

    // configuración de rutas de Axum para /api/v1/folders
    Router::new()
        .route("/api/v1/folders", post(handlers::create_folder))
        .route("/api/v1/folders/", get(handlers::get_all_folders))
        .route(
            "/api/v1/folders/:id",
            get(handlers::get_folder)
                .put(handlers::update_folder)
                .delete(handlers::delete_folder),
        )
        .layer(trace_layer)
        .with_state(state)
}
