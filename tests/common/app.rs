use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Router;
use serde_json::Value;
use tower::ServiceExt;

use security_suite::shared::auth::JwtService;
use security_suite::shared::state::{AppState, FolderState, LoginState, UserState};

use security_suite::users::application::login_service::LoginService;
use security_suite::users::application::service::UserService;
use security_suite::users::domain::UserServicePort;
use security_suite::users::infrastructure::http::routes::{login_routes, user_routes};

use security_suite::folders::domain::FolderServicePort;
use security_suite::folders::infrastructure::http::routes::folders_routes;

use super::mock::{MockFolderService, MockUserRepository};

pub fn build_app() -> Router {
    let user_repository = Arc::new(MockUserRepository::new());

    let user_service: Arc<dyn UserServicePort> =
        Arc::new(UserService::new(user_repository.clone()));

    let jwt_service = Arc::new(JwtService::new("test-secret", 3600));

    let login_service = Arc::new(LoginService::new(
        user_repository,
        jwt_service.clone(),
    ));

    let user_state = Arc::new(UserState {
        user_service,
    });

    let login_state = Arc::new(LoginState {
        login_service,
        jwt_service,
    });

    let folder_service: Arc<dyn FolderServicePort> = Arc::new(MockFolderService::new());

    let folder_state = Arc::new(FolderState {
        folder_service,
    });

    let app_state = AppState {
        user_state,
        login_state,
        folder_state,
    };

    user_routes().merge(login_routes()).merge(folders_routes()).with_state(app_state)
}

pub async fn dispatch(
    app: &Router,
    request: Request<Body>,
) -> (StatusCode, Value) {
    let response = app.clone()
        .oneshot(request)
        .await
        .expect("el router no pudo procesar la request");

    let status = response.status();

    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("no se pudo leer el body de la respuesta");

    let body = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes)
            .unwrap_or(Value::Null)
    };

    (status, body)
}

pub fn json_request(
    method: &str,
    uri: &str,
    body: Value,
) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::to_vec(&body).unwrap(),
        ))
        .unwrap()
}

pub fn auth_request( // usado para GET y DELETE y rutas de users que no requieran un usuario en el body
    method: &str,
    uri: &str,
    token: &str,
) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap()
}

pub fn auth_json_request( // usado para POST y PUT
    method: &str,
    uri: &str,
    body: Value,
    token: &str,
) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap()
}

pub fn empty_request(
    method: &str,
    uri: &str,
) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .body(Body::empty())
        .unwrap()
}


pub async fn create_authenticated_user(app: &Router) -> String {
    let (status, _) = dispatch(
        app,
        json_request(
            "POST",
            "/api/v1/users",
            serde_json::json!({
                "username": "testuser",
                "email": "test@example.com",
                "password": "supersecret123"
            }),
        ),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);

    let (status, body) = dispatch(
        app,
        json_request(
            "POST",
            "/api/v1/users/login",
            serde_json::json!({
                "email": "test@example.com",
                "password": "supersecret123"
            }),
        ),
    )
    .await;

    assert_eq!(status, StatusCode::OK);

    body["access_token"].as_str().expect("el login no devolvió access_token").to_string()
}