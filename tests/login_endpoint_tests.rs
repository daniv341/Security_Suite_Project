mod common;

use axum::http::StatusCode;
use serde_json::json;

use common::app::{build_app, dispatch, json_request};

#[tokio::test]
async fn login_exitoso_devuelve_token() {
    let app = build_app();

    let (status, _) = dispatch(
        &app,
        json_request(
            "POST",
            "/api/v1/users",
            json!({
                "username": "testuser",
                "email": "test@example.com",
                "password": "supersecret123"
            }),
        ),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);

    let (status, body) = dispatch(
        &app,
        json_request(
            "POST",
            "/api/v1/users/login",
            json!({
                "email": "test@example.com",
                "password": "supersecret123"
            }),
        ),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert!(body["access_token"].as_str().is_some());
    assert_eq!(body["token_type"], "Bearer");
}

#[tokio::test]
async fn login_falla_con_password_incorrecta() {
    let app = build_app();

    let (status, _) = dispatch(
        &app,
        json_request(
            "POST",
            "/api/v1/users",
            json!({
                "username": "testuser",
                "email": "test@example.com",
                "password": "supersecret123"
            }),
        ),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);

    let (status, body) = dispatch(
        &app,
        json_request(
            "POST",
            "/api/v1/users/login",
            json!({
                "email": "test@example.com",
                "password": "passwordincorrecta"
            }),
        ),
    )
    .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert!(body["error"].is_string());
}

#[tokio::test]
async fn login_falla_con_email_inexistente() {
    let app = build_app();

    let (status, body) = dispatch(
        &app,
        json_request(
            "POST",
            "/api/v1/users/login",
            json!({
                "email": "noexiste@example.com",
                "password": "supersecret123"
            }),
        ),
    )
    .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
   assert!(body["error"].is_string());
}

#[tokio::test]
async fn login_falla_sin_email() {
    let app = build_app();

    let (status, _) = dispatch(
        &app,
        json_request(
            "POST",
            "/api/v1/users/login",
            json!({
                "password": "supersecret123"
            }),
        ),
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn login_falla_sin_password() {
    let app = build_app();

    let (status, _) = dispatch(
        &app,
        json_request(
            "POST",
            "/api/v1/users/login",
            json!({
                "email": "test@example.com"
            }),
        ),
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}