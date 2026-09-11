//! Pruebas de integración a nivel de **endpoint HTTP** para
//! `/api/v1/folders`. Generadas por `scripts/scaffold_object.py`
//! — ajustá según la lógica real de `Folder` (agregá casos para
//! campos nuevos, etc.).
//!
//! No usan ninguna librería nueva: `Router` implementa `tower::Service`
//! directamente (el trait base de `tower`, sin necesitar el feature
//! `"util"` que daría `ServiceExt::oneshot`), y `axum::body::to_bytes`
//! permite leer el body de la respuesta sin agregar `http-body-util`.
//! Las respuestas se comparan como `serde_json::Value` para no
//! depender de que los DTOs deriven `Deserialize`.

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
use security_suite::folders::application::service::FolderService;
use security_suite::folders::domain::{DomainError, Folder, FolderRepository, FolderServicePort};
use security_suite::folders::infrastructure::http::routes::folders_routes;

struct MockFolderRepository {
    folders: Mutex<HashMap<Uuid, Folder>>,
}

impl MockFolderRepository {
    fn new() -> Self {
        Self {
            folders: Mutex::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl FolderRepository for MockFolderRepository {
    async fn create(&self, folder: &Folder) -> Result<Folder, DomainError> {
        let mut folders = self.folders.lock().unwrap();
        folders.insert(folder.id, folder.clone());
        Ok(folder.clone())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Folder>, DomainError> {
        let folders = self.folders.lock().unwrap();
        Ok(folders.get(&id).cloned())
    }

    async fn find_by_name(&self, name: &str) -> Result<Option<Folder>, DomainError> {
        let folders = self.folders.lock().unwrap();
        Ok(folders.values().find(|i| i.name == name).cloned())
    }

    async fn find_all(&self, pagination: Pagination) -> Result<Vec<Folder>, DomainError> {
        let folders = self.folders.lock().unwrap();
        let mut all: Vec<Folder> = folders.values().cloned().collect();
        all.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        let start = pagination.offset() as usize;
        if start >= all.len() {
            return Ok(vec![]);
        }
        let end = (start + pagination.limit() as usize).min(all.len());
        Ok(all[start..end].to_vec())
    }

    async fn count_all(&self) -> Result<i64, DomainError> {
        let folders = self.folders.lock().unwrap();
        Ok(folders.len() as i64)
    }

    async fn update(&self, folder: &Folder) -> Result<Folder, DomainError> {
        let mut folders = self.folders.lock().unwrap();
        folders.insert(folder.id, folder.clone());
        Ok(folder.clone())
    }

    async fn delete(&self, id: Uuid) -> Result<(), DomainError> {
        let mut folders = self.folders.lock().unwrap();
        folders.remove(&id).ok_or(DomainError::NotFound)?;
        Ok(())
    }
}

/// Arma un router nuevo (con su propio repositorio en memoria,
/// aislado del resto de los tests) listo para recibir requests.
fn build_app() -> Router {
    let repository = Arc::new(MockFolderRepository::new());
    let service: Arc<dyn FolderServicePort> = Arc::new(FolderService::new(repository));
    folders_routes(service)
}

/// Equivalente casero a `tower::ServiceExt::oneshot`, sin necesitar el
/// feature `"util"` de `tower`.
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
    Request::builder().method(method).uri(uri).body(Body::empty()).unwrap()
}

#[tokio::test]
async fn post_folders_crea_correctamente() {
    let mut app = build_app();

    let (status, body) = dispatch(
        &mut app,
        json_request("POST", "/api/v1/folders", json!({"name": "Ejemplo"})),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["name"], "Ejemplo");
    assert!(body["id"].is_string());
}

#[tokio::test]
async fn post_folders_falla_si_el_nombre_es_muy_corto() {
    let mut app = build_app();

    let (status, body) = dispatch(
        &mut app,
        json_request("POST", "/api/v1/folders", json!({"name": "ab"})),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body["error"].is_string());
}

#[tokio::test]
async fn post_folders_falla_si_el_nombre_ya_existe() {
    let mut app = build_app();

    dispatch(
        &mut app,
        json_request("POST", "/api/v1/folders", json!({"name": "Duplicado"})),
    )
    .await;

    let (status, _body) = dispatch(
        &mut app,
        json_request("POST", "/api/v1/folders", json!({"name": "Duplicado"})),
    )
    .await;

    assert_eq!(status, StatusCode::CONFLICT);
}

#[tokio::test]
async fn get_folder_devuelve_404_si_no_existe() {
    let mut app = build_app();

    let (status, _body) = dispatch(
        &mut app,
        empty_request("GET", &format!("/api/v1/folders/{}", Uuid::new_v4())),
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn get_folder_devuelve_el_creado() {
    let mut app = build_app();

    let (_, created) = dispatch(
        &mut app,
        json_request("POST", "/api/v1/folders", json!({"name": "Original"})),
    )
    .await;
    let id = created["id"].as_str().unwrap();

    let (status, body) = dispatch(&mut app, empty_request("GET", &format!("/api/v1/folders/{id}"))).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["name"], "Original");
}

#[tokio::test]
async fn put_folder_actualiza_el_nombre() {
    let mut app = build_app();

    let (_, created) = dispatch(
        &mut app,
        json_request("POST", "/api/v1/folders", json!({"name": "Original"})),
    )
    .await;
    let id = created["id"].as_str().unwrap();

    let (status, body) = dispatch(
        &mut app,
        json_request("PUT", &format!("/api/v1/folders/{id}"), json!({"name": "Nuevo"})),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["name"], "Nuevo");
}

#[tokio::test]
async fn delete_folder_elimina_correctamente() {
    let mut app = build_app();

    let (_, created) = dispatch(
        &mut app,
        json_request("POST", "/api/v1/folders", json!({"name": "Borrame"})),
    )
    .await;
    let id = created["id"].as_str().unwrap();

    let (delete_status, _) =
        dispatch(&mut app, empty_request("DELETE", &format!("/api/v1/folders/{id}"))).await;
    assert_eq!(delete_status, StatusCode::NO_CONTENT);

    let (get_status, _) = dispatch(&mut app, empty_request("GET", &format!("/api/v1/folders/{id}"))).await;
    assert_eq!(get_status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_folder_falla_si_no_existe() {
    let mut app = build_app();

    let (status, _body) = dispatch(
        &mut app,
        empty_request("DELETE", &format!("/api/v1/folders/{}", Uuid::new_v4())),
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn get_folders_pagina_los_resultados() {
    let mut app = build_app();

    for i in 0..5 {
        dispatch(
            &mut app,
            json_request("POST", "/api/v1/folders", json!({"name": format!("Item {i}")})),
        )
        .await;
    }

    let (status, body) =
        dispatch(&mut app, empty_request("GET", "/api/v1/folders/?page=1&page_size=2")).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["items"].as_array().unwrap().len(), 2);
    assert_eq!(body["total_items"], 5);
    assert_eq!(body["total_pages"], 3);
}
