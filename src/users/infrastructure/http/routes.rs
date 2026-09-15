//! Configuración del `Router` de Axum para el recurso `/api/v1/users`.

use std::time::Duration;

use axum::body::Body;
use axum::http::{Request, Response};
use axum::routing::{get, post};
use axum::Router;
use tower_http::trace::TraceLayer;
use tracing::Span;

use crate::shared::state::AppState;
use super::handlers;

pub fn user_routes() -> Router<AppState> {
    // Log de request resumido: una sola línea por petición con
    // método, path, status y latencia. Se desactiva el evento de
    // "started processing request" que trae `TraceLayer` por defecto
    // (solo se loguea al finalizar, en `on_response`).
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
        .route("/api/v1/users", post(handlers::create_user))
        .route("/api/v1/users/", get(handlers::get_all_users))
        .route(
            "/api/v1/users/:id",
            get(handlers::get_user)
                .put(handlers::update_user)
                .delete(handlers::delete_user),
        )
        .layer(trace_layer)
}

pub fn login_routes() -> Router<AppState> {
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
        .route("/api/v1/users/login", post(handlers::login))
        .route("/api/v1/users/logout", post(handlers::logout))
        .layer(trace_layer)
}
