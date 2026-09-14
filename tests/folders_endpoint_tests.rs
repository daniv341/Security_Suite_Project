mod common;

use common::{build_app, dispatch, json_request, auth_request, auth_json_request, empty_request, create_authenticated_user};

use axum::http::StatusCode;
use serde_json::json;
use uuid::Uuid;


#[tokio::test]
async fn post_folders_crea_correctamente() {
    let app = build_app();
    let token = create_authenticated_user(&app).await;

    let (status, body) = dispatch(
        &app,
        auth_json_request("POST", "/api/v1/folders", json!({"name": "Ejemplo"}), &token),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["name"], "Ejemplo");
    assert!(body["id"].is_string());
}

#[tokio::test]
async fn post_folders_falla_si_el_nombre_es_muy_corto() {
    let app = build_app();
    let token = create_authenticated_user(&app).await;

    let (status, body) = dispatch(
        &app,
        auth_json_request("POST", "/api/v1/folders", json!({"name": "ab"}), &token),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body["error"].is_string());
}

#[tokio::test]
async fn post_folders_falla_si_el_nombre_ya_existe() {
    let app = build_app();
    let token = create_authenticated_user(&app).await;

    dispatch(
        &app,
        auth_json_request("POST", "/api/v1/folders", json!({"name": "Duplicado"}), &token),
    )
    .await;

    let (status, _body) = dispatch(
        &app,
        auth_json_request("POST", "/api/v1/folders", json!({"name": "Duplicado"}), &token),
    )
    .await;

    assert_eq!(status, StatusCode::CONFLICT);
}

#[tokio::test]
async fn get_folder_devuelve_404_si_no_existe() {
    let app = build_app();
    let token = create_authenticated_user(&app).await;

    let (status, _body) = dispatch(
        &app,
        auth_request("GET", &format!("/api/v1/folders/{}", Uuid::new_v4()), &token),
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn get_folder_devuelve_el_creado() {
    let app = build_app();
    let token = create_authenticated_user(&app).await;

    let (_, created) = dispatch(
        &app,
        auth_json_request("POST", "/api/v1/folders", json!({"name": "Original"}), &token),
    )
    .await;
    let id = created["id"].as_str().unwrap();

    let (status, body) = dispatch(&app, auth_request("GET", &format!("/api/v1/folders/{id}"), &token)).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["name"], "Original");
}

#[tokio::test]
async fn put_folder_actualiza_el_nombre() {
    let app = build_app();
    let token = create_authenticated_user(&app).await;

    let (_, created) = dispatch(
        &app,
        auth_json_request("POST", "/api/v1/folders", json!({"name": "Original"}), &token),
    )
    .await;
    let id = created["id"].as_str().unwrap();

    let (status, body) = dispatch(
        &app,
        auth_json_request("PUT", &format!("/api/v1/folders/{id}"), json!({"name": "Nuevo"}), &token),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["name"], "Nuevo");
}

#[tokio::test]
async fn delete_folder_elimina_correctamente() {
    let app = build_app();
    let token = create_authenticated_user(&app).await;

    let (_, created) = dispatch(
        &app,
        auth_json_request("POST", "/api/v1/folders", json!({"name": "Borrame"}), &token),
    )
    .await;
    let id = created["id"].as_str().unwrap();

    let (delete_status, _) =
        dispatch(&app, auth_request("DELETE", &format!("/api/v1/folders/{id}"), &token)).await;
    assert_eq!(delete_status, StatusCode::NO_CONTENT);

    let (get_status, _) = dispatch(&app, auth_request("GET", &format!("/api/v1/folders/{id}"), &token)).await;
    assert_eq!(get_status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_folder_falla_si_no_existe() {
    let app = build_app();
    let token = create_authenticated_user(&app).await;

    let (status, _body) = dispatch(
        &app,
        auth_request("DELETE", &format!("/api/v1/folders/{}", Uuid::new_v4()), &token),
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn get_folders_pagina_los_resultados() {
    let app = build_app();
    let token = create_authenticated_user(&app).await;

    for i in 0..5 {
            dispatch(
            &app,
            auth_json_request("POST", "/api/v1/folders", json!({"name": format!("Item {i}")}), &token),
        )
        .await;
    }

    let (status, body) =
        dispatch(&app, auth_request("GET", "/api/v1/folders/?page=1&page_size=2", &token)).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["items"].as_array().unwrap().len(), 2);
    assert_eq!(body["total_items"], 5);
    assert_eq!(body["total_pages"], 3);
}
