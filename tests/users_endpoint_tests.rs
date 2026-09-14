mod common;

use common::{build_app, dispatch, json_request, auth_request, empty_request};

use axum::http::StatusCode;
use serde_json::json;
use uuid::Uuid;

use security_suite::shared::auth::JwtService;

// POST

#[tokio::test]
async fn post_users_crea_el_usuario_y_no_expone_password_hash() {
    let app = build_app();

    let (status, body) = dispatch(
        &app,
        json_request(
            "POST",
            "/api/v1/users",
            json!({
                "username": "johndoe",
                "email": "john@example.com",
                "password": "supersecret123"
            }),
        ),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["username"], "johndoe");
    assert_eq!(body["email"], "john@example.com");
    assert!(body.get("password_hash").is_none());
    assert!(body["id"].is_string());
    assert!(body["created_at"].is_string());
}

#[tokio::test]
async fn post_users_falla_con_password_corta() {
    let app = build_app();

    let (status, body) = dispatch(
        &app,
        json_request(
            "POST",
            "/api/v1/users",
            json!({"username": "johndoe", "email": "john@example.com", "password": "123"}),
        ),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body["error"].is_string());
}

#[tokio::test]
async fn post_users_falla_con_email_invalido() {
    let app = build_app();

    let (status, _body) = dispatch(
        &app,
        json_request(
            "POST",
            "/api/v1/users",
            json!({"username": "johndoe", "email": "no-es-un-email", "password": "supersecret123"}),
        ),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn post_users_falla_con_username_corto() {
    let app = build_app();

    let (status, _body) = dispatch(
        &app,
        json_request(
            "POST",
            "/api/v1/users",
            json!({"username": "jo", "email": "john@example.com", "password": "supersecret123"}),
        ),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn post_users_falla_con_email_duplicado() {
    let app = build_app();

    let (first_status, _) = dispatch(
        &app,
        json_request(
            "POST",
            "/api/v1/users",
            json!({"username": "johndoe", "email": "john@example.com", "password": "supersecret123"}),
        ),
    )
    .await;
    assert_eq!(first_status, StatusCode::CREATED);

    let (status, body) = dispatch(
        &app,
        json_request(
            "POST",
            "/api/v1/users",
            json!({"username": "janedoe", "email": "john@example.com", "password": "otraclave123"}),
        ),
    )
    .await;

    assert_eq!(status, StatusCode::CONFLICT);
    assert!(body["error"].is_string());
}


// GET

#[tokio::test]
async fn get_user_devuelve_404_si_no_existe() {
    let app = build_app();
    let token = JwtService::new("test-secret", 3600).create_token(&Uuid::new_v4().to_string()).unwrap();

    let (status, _body) = dispatch(
        &app,
        auth_request("GET", &format!("/api/v1/users/{}", Uuid::new_v4()), &token)
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn get_user_devuelve_el_usuario_creado() {
    let app = build_app();

    let (_, created) = dispatch(
        &app,
        json_request(
            "POST",
            "/api/v1/users",
            json!({"username": "johndoe", "email": "john@example.com", "password": "supersecret123"}),
        ),
    )
    .await;

    let id = created["id"].as_str().unwrap();
    let jwt_service = JwtService::new("test-secret", 3600);
    let token = jwt_service.create_token(id).unwrap();

    let (status, body) = dispatch(&app, auth_request("GET", &format!("/api/v1/users/{id}"), &token)).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["id"], id);
    assert_eq!(body["username"], "johndoe");
}


// PUT

#[tokio::test]
async fn put_user_actualiza_username_y_email() {
    let app = build_app();

    let (_, created) = dispatch(
        &app,
        json_request(
            "POST",
            "/api/v1/users",
            json!({"username": "johndoe", "email": "john@example.com", "password": "supersecret123"}),
        ),
    )
    .await;
    let id = created["id"].as_str().unwrap();

    let (status, body) = dispatch(
        &app,
        json_request(
            "PUT",
            &format!("/api/v1/users/{id}"),
            json!({"username": "john_updated", "email": "john2@example.com"}),
        ),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["username"], "john_updated");
    assert_eq!(body["email"], "john2@example.com");
}

#[tokio::test]
async fn put_user_falla_con_email_duplicado() {
    let app = build_app();

    dispatch(
        &app,
        json_request(
            "POST",
            "/api/v1/users",
            json!({"username": "johndoe", "email": "john@example.com", "password": "supersecret123"}),
        ),
    )
    .await;

    let (_, jane) = dispatch(
        &app,
        json_request(
            "POST",
            "/api/v1/users",
            json!({"username": "janedoe", "email": "jane@example.com", "password": "supersecret123"}),
        ),
    )
    .await;
    let jane_id = jane["id"].as_str().unwrap();

    let (status, _body) = dispatch(
        &app,
        json_request(
            "PUT",
            &format!("/api/v1/users/{jane_id}"),
            json!({"email": "john@example.com"}),
        ),
    )
    .await;

    assert_eq!(status, StatusCode::CONFLICT);
}


// DELETE

#[tokio::test]
async fn delete_user_elimina_la_cuenta() {
    let app = build_app();

    let (_, created) = dispatch(
        &app,
        json_request(
            "POST",
            "/api/v1/users",
            json!({"username": "johndoe", "email": "john@example.com", "password": "supersecret123"}),
        ),
    )
    .await;
    let id = created["id"].as_str().unwrap();
    let jwt_service = JwtService::new("test-secret", 3600);
    let token = jwt_service.create_token(id).unwrap();

    let (delete_status, _) = dispatch(&app, empty_request("DELETE", &format!("/api/v1/users/{id}"))).await;
    assert_eq!(delete_status, StatusCode::NO_CONTENT);

    let (get_status, _) = dispatch(&app, auth_request("GET", &format!("/api/v1/users/{id}"), &token)).await;
    assert_eq!(get_status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_user_falla_si_no_existe() {
    let app = build_app();

    let (status, _body) = dispatch(
        &app,
        empty_request("DELETE", &format!("/api/v1/users/{}", Uuid::new_v4())),
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}


// GETALL

#[tokio::test]
async fn get_users_pagina_los_resultados() {
    let app = build_app();
    let mut id = String::new();

    for i in 0..5 {
        let (status, body) = dispatch(
            &app,
            json_request(
                "POST",
                "/api/v1/users",
                json!({
                    "username": format!("user{i}"),
                    "email": format!("user{i}@example.com"),
                    "password": "supersecret123"
                }),
            ),
        )
        .await;

        assert_eq!(status, StatusCode::CREATED);
        id = body["id"].as_str().unwrap().to_string();
    }

    let jwt_service = JwtService::new("test-secret", 3600);
    let token = jwt_service.create_token(&id).unwrap();

    let (status, body) = dispatch(&app, auth_request("GET", "/api/v1/users/?page=1&page_size=2", &token)).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["items"].as_array().unwrap().len(), 2);
    assert_eq!(body["page"], 1);
    assert_eq!(body["page_size"], 2);
    assert_eq!(body["total_items"], 5);
    assert_eq!(body["total_pages"], 3);
}

#[tokio::test]
async fn get_users_usa_valores_por_defecto_si_no_se_especifican() {
    let app = build_app();

    let (_, created) = dispatch(
        &app,
        json_request(
            "POST",
            "/api/v1/users",
            json!({"username": "johndoe", "email": "john@example.com", "password": "supersecret123"}),
        ),
    )
    .await;

    let id = created["id"].as_str().unwrap();
    let jwt_service = JwtService::new("test-secret", 3600);
    let token = jwt_service.create_token(id).unwrap();

    let (status, body) = dispatch(&app, auth_request("GET", "/api/v1/users/", &token)).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["page"], 1);
    assert_eq!(body["page_size"], 20);
    assert_eq!(body["total_items"], 1);
}
