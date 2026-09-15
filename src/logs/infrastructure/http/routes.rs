//! Configuración del `Router` de Axum para `/api/v1/logs`.

use std::time::Duration;

use axum::body::Body;
use axum::http::{Request, Response};
use axum::routing::get;
use axum::Router;
use tower_http::trace::TraceLayer;
use tracing::Span;

use crate::shared::state::AppState;
use super::handlers;

pub fn logs_routes() -> Router<AppState> {
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

    Router::<AppState>::new()
        .route("/api/v1/logs/user_id/", get(handlers::get_all_logs_by_user_id))
        .route("/api/v1/logs/resource/", get(handlers::get_all_logs_by_resource))
        .route("/api/v1/logs/action/", get(handlers::get_all_logs_by_action))
        .route(
            "/api/v1/logs/:id",
            get(handlers::get_log)
        )
        .layer(trace_layer)
}
