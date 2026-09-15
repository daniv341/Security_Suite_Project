//! Pruebas de integración a nivel de **endpoint HTTP** para
//! `/api/v1/logs`. Generadas por `scripts/scaffold_object.py`
//! — ajustá según la lógica real de `Log` (agregá casos para
//! campos nuevos, etc.).
//!
//! Reutilizan los helpers compartidos de `tests/common/` (los mismos
//! que usa `users_endpoint_tests.rs`): `build_app`, `dispatch`,
//! `json_request`, `empty_request`. Para que esto compile hace falta
//! primero:
//!   1. Agregar `MockLogRepository` a `tests/common/mock.rs`.
//!   2. Cablear `LogState` + `logs_routes()` dentro de
//!      `build_app()` en `tests/common/app.rs`.
//! (el script imprime el código exacto de ambos pasos al terminar).

mod common;

use common::{build_app, dispatch, empty_request, json_request};

use axum::http::StatusCode;
use serde_json::json;
use uuid::Uuid;

#[tokio::test]
async fn post_logs_crea_correctamente() {
    let app = build_app();

    let (status, body) = dispatch(
        &app,
        json_request("POST", "/api/v1/logs", json!({"name": "Ejemplo"})),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["name"], "Ejemplo");
    assert!(body["id"].is_string());
}

#[tokio::test]
async fn post_logs_falla_si_el_nombre_es_muy_corto() {
    let app = build_app();

    let (status, body) = dispatch(
        &app,
        json_request("POST", "/api/v1/logs", json!({"name": "ab"})),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body["error"].is_string());
}

#[tokio::test]
async fn post_logs_falla_si_el_nombre_ya_existe() {
    let app = build_app();

    dispatch(
        &app,
        json_request("POST", "/api/v1/logs", json!({"name": "Duplicado"})),
    )
    .await;

    let (status, _body) = dispatch(
        &app,
        json_request("POST", "/api/v1/logs", json!({"name": "Duplicado"})),
    )
    .await;

    assert_eq!(status, StatusCode::CONFLICT);
}

#[tokio::test]
async fn get_log_devuelve_404_si_no_existe() {
    let app = build_app();

    let (status, _body) = dispatch(
        &app,
        empty_request("GET", &format!("/api/v1/logs/{}", Uuid::new_v4())),
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn get_log_devuelve_el_creado() {
    let app = build_app();

    let (_, created) = dispatch(
        &app,
        json_request("POST", "/api/v1/logs", json!({"name": "Original"})),
    )
    .await;
    let id = created["id"].as_str().unwrap();

    let (status, body) = dispatch(&app, empty_request("GET", &format!("/api/v1/logs/{id}"))).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["name"], "Original");
}

#[tokio::test]
async fn put_log_actualiza_el_nombre() {
    let app = build_app();

    let (_, created) = dispatch(
        &app,
        json_request("POST", "/api/v1/logs", json!({"name": "Original"})),
    )
    .await;
    let id = created["id"].as_str().unwrap();

    let (status, body) = dispatch(
        &app,
        json_request("PUT", &format!("/api/v1/logs/{id}"), json!({"name": "Nuevo"})),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["name"], "Nuevo");
}

#[tokio::test]
async fn delete_log_elimina_correctamente() {
    let app = build_app();

    let (_, created) = dispatch(
        &app,
        json_request("POST", "/api/v1/logs", json!({"name": "Borrame"})),
    )
    .await;
    let id = created["id"].as_str().unwrap();

    let (delete_status, _) =
        dispatch(&app, empty_request("DELETE", &format!("/api/v1/logs/{id}"))).await;
    assert_eq!(delete_status, StatusCode::NO_CONTENT);

    let (get_status, _) = dispatch(&app, empty_request("GET", &format!("/api/v1/logs/{id}"))).await;
    assert_eq!(get_status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_log_falla_si_no_existe() {
    let app = build_app();

    let (status, _body) = dispatch(
        &app,
        empty_request("DELETE", &format!("/api/v1/logs/{}", Uuid::new_v4())),
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn get_logs_pagina_los_resultados() {
    let app = build_app();
    let mut id = String::new();

    for i in 0..5 {
        let (status, body) = dispatch(
            &app,
            json_request("POST", "/api/v1/logs", json!({"name": format!("Item {i}")})),
        )
        .await;

        assert_eq!(status, StatusCode::CREATED);
        id = body["id"].as_str().unwrap().to_string();
    }

    let (status, body) =
        dispatch(&app, empty_request("GET", "/api/v1/logs/?page=1&page_size=2")).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["items"].as_array().unwrap().len(), 2);
    assert_eq!(body["total_items"], 5);
    assert_eq!(body["total_pages"], 3);
}
