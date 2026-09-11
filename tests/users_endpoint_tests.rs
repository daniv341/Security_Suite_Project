
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Router;
use serde_json::{json, Value};
use tower::Service;
use uuid::Uuid;

use security_suite::shared::pagination::Pagination;
use security_suite::users::application::service::UserService;
use security_suite::users::domain::{DomainError, User, UserRepository, UserServicePort};
use security_suite::users::infrastructure::http::routes::user_routes;

/// Mismo mock que en `tests/user_service_tests.rs`: repositorio en
/// memoria, sin necesidad de PostgreSQL.
struct MockUserRepository {
    users: Mutex<HashMap<Uuid, User>>,
}

impl MockUserRepository {
    fn new() -> Self {
        Self {
            users: Mutex::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl UserRepository for MockUserRepository {
    async fn create(&self, user: &User) -> Result<User, DomainError> {
        let mut users = self.users.lock().unwrap();
        users.insert(user.id, user.clone());
        Ok(user.clone())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, DomainError> {
        let users = self.users.lock().unwrap();
        Ok(users.get(&id).cloned())
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>, DomainError> {
        let users = self.users.lock().unwrap();
        Ok(users.values().find(|u| u.email == email).cloned())
    }

    async fn find_all(&self, pagination: Pagination) -> Result<Vec<User>, DomainError> {
        let users = self.users.lock().unwrap();
        let mut all: Vec<User> = users.values().cloned().collect();
        all.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        let start = pagination.offset() as usize;
        if start >= all.len() {
            return Ok(vec![]);
        }
        let end = (start + pagination.limit() as usize).min(all.len());
        Ok(all[start..end].to_vec())
    }

    async fn count_all(&self) -> Result<i64, DomainError> {
        let users = self.users.lock().unwrap();
        Ok(users.len() as i64)
    }

    async fn update(&self, user: &User) -> Result<User, DomainError> {
        let mut users = self.users.lock().unwrap();
        users.insert(user.id, user.clone());
        Ok(user.clone())
    }

    async fn delete(&self, id: Uuid) -> Result<(), DomainError> {
        let mut users = self.users.lock().unwrap();
        users.remove(&id).ok_or(DomainError::NotFound)?;
        Ok(())
    }
}

/// Arma un router nuevo (con su propio repositorio en memoria,
/// aislado del resto de los tests) listo para recibir requests.
fn build_app() -> Router {
    let repository = Arc::new(MockUserRepository::new());
    let service: Arc<dyn UserServicePort> = Arc::new(UserService::new(repository));
    user_routes(service)
}

/// Equivalente casero a `tower::ServiceExt::oneshot`, sin necesitar el
/// feature `"util"` de `tower`: espera a que el servicio esté listo
/// (`poll_ready`) y después lo invoca (`call`), que es exactamente lo
/// que hace `oneshot` por dentro.
async fn dispatch(app: &mut Router, request: Request<Body>) -> (StatusCode, Value) {
    std::future::poll_fn(|cx| Service::poll_ready(app, cx))
        .await
        .expect("el router siempre está listo");

    let response = app.call(request).await.expect("Router::call es infalible");
    let status = response.status();

    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("no se pudo leer el body de la respuesta");

    let body = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).expect("la respuesta no es JSON válido")
    };

    (status, body)
}

fn json_request(method: &str, uri: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap()
}

fn empty_request(method: &str, uri: &str) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .body(Body::empty())
        .unwrap()
}

#[tokio::test]
async fn post_users_crea_el_usuario_y_no_expone_password_hash() {
    let mut app = build_app();

    let (status, body) = dispatch(
        &mut app,
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
    let mut app = build_app();

    let (status, body) = dispatch(
        &mut app,
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
    let mut app = build_app();

    let (status, _body) = dispatch(
        &mut app,
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
    let mut app = build_app();

    let (status, _body) = dispatch(
        &mut app,
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
    let mut app = build_app();

    let (first_status, _) = dispatch(
        &mut app,
        json_request(
            "POST",
            "/api/v1/users",
            json!({"username": "johndoe", "email": "john@example.com", "password": "supersecret123"}),
        ),
    )
    .await;
    assert_eq!(first_status, StatusCode::CREATED);

    let (status, body) = dispatch(
        &mut app,
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

#[tokio::test]
async fn get_user_devuelve_404_si_no_existe() {
    let mut app = build_app();

    let (status, _body) = dispatch(
        &mut app,
        empty_request("GET", &format!("/api/v1/users/{}", Uuid::new_v4())),
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn get_user_devuelve_el_usuario_creado() {
    let mut app = build_app();

    let (_, created) = dispatch(
        &mut app,
        json_request(
            "POST",
            "/api/v1/users",
            json!({"username": "johndoe", "email": "john@example.com", "password": "supersecret123"}),
        ),
    )
    .await;
    let id = created["id"].as_str().unwrap();

    let (status, body) = dispatch(&mut app, empty_request("GET", &format!("/api/v1/users/{id}"))).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["id"], id);
    assert_eq!(body["username"], "johndoe");
}

#[tokio::test]
async fn put_user_actualiza_username_y_email() {
    let mut app = build_app();

    let (_, created) = dispatch(
        &mut app,
        json_request(
            "POST",
            "/api/v1/users",
            json!({"username": "johndoe", "email": "john@example.com", "password": "supersecret123"}),
        ),
    )
    .await;
    let id = created["id"].as_str().unwrap();

    let (status, body) = dispatch(
        &mut app,
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
    let mut app = build_app();

    dispatch(
        &mut app,
        json_request(
            "POST",
            "/api/v1/users",
            json!({"username": "johndoe", "email": "john@example.com", "password": "supersecret123"}),
        ),
    )
    .await;

    let (_, jane) = dispatch(
        &mut app,
        json_request(
            "POST",
            "/api/v1/users",
            json!({"username": "janedoe", "email": "jane@example.com", "password": "supersecret123"}),
        ),
    )
    .await;
    let jane_id = jane["id"].as_str().unwrap();

    let (status, _body) = dispatch(
        &mut app,
        json_request(
            "PUT",
            &format!("/api/v1/users/{jane_id}"),
            json!({"email": "john@example.com"}),
        ),
    )
    .await;

    assert_eq!(status, StatusCode::CONFLICT);
}

#[tokio::test]
async fn delete_user_elimina_la_cuenta() {
    let mut app = build_app();

    let (_, created) = dispatch(
        &mut app,
        json_request(
            "POST",
            "/api/v1/users",
            json!({"username": "johndoe", "email": "john@example.com", "password": "supersecret123"}),
        ),
    )
    .await;
    let id = created["id"].as_str().unwrap();

    let (delete_status, _) = dispatch(&mut app, empty_request("DELETE", &format!("/api/v1/users/{id}"))).await;
    assert_eq!(delete_status, StatusCode::NO_CONTENT);

    let (get_status, _) = dispatch(&mut app, empty_request("GET", &format!("/api/v1/users/{id}"))).await;
    assert_eq!(get_status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_user_falla_si_no_existe() {
    let mut app = build_app();

    let (status, _body) = dispatch(
        &mut app,
        empty_request("DELETE", &format!("/api/v1/users/{}", Uuid::new_v4())),
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn get_users_pagina_los_resultados() {
    let mut app = build_app();

    for i in 0..5 {
        dispatch(
            &mut app,
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
    }

    let (status, body) = dispatch(&mut app, empty_request("GET", "/api/v1/users/?page=1&page_size=2")).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["items"].as_array().unwrap().len(), 2);
    assert_eq!(body["page"], 1);
    assert_eq!(body["page_size"], 2);
    assert_eq!(body["total_items"], 5);
    assert_eq!(body["total_pages"], 3);
}

#[tokio::test]
async fn get_users_usa_valores_por_defecto_si_no_se_especifican() {
    let mut app = build_app();

    dispatch(
        &mut app,
        json_request(
            "POST",
            "/api/v1/users",
            json!({"username": "johndoe", "email": "john@example.com", "password": "supersecret123"}),
        ),
    )
    .await;

    let (status, body) = dispatch(&mut app, empty_request("GET", "/api/v1/users/")).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["page"], 1);
    assert_eq!(body["page_size"], 20);
    assert_eq!(body["total_items"], 1);
}
