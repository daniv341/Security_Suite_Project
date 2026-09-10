//! Pruebas de integración del caso de uso `UserService`.
//!
//! Viven fuera de `src/` (carpeta `tests/`, el lugar idiomático en
//! Rust para tests de integración) y se compilan como un crate aparte
//! que enlaza contra la librería `security_suite` (declarada en
//! `src/lib.rs`). Por eso todo lo que se usa acá —`User`, `DomainError`,
//! `UserRepository`, `UserServicePort`, `UserService`— tiene que ser
//! `pub` en el crate, tal como ya lo era.
//!
//! No requieren PostgreSQL: usan un mock manual de `UserRepository`
//! en memoria.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use uuid::Uuid;

use security_suite::shared::pagination::Pagination;
use security_suite::users::application::service::UserService;
use security_suite::users::domain::{DomainError, User, UserRepository, UserServicePort};

/// Mock manual del puerto `UserRepository`, en memoria, usado
/// únicamente para probar la lógica de `UserService` de forma
/// aislada, sin necesidad de una base de datos real.
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

    /// Simula `ORDER BY created_at DESC LIMIT ... OFFSET ...`.
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

fn build_service() -> UserService {
    UserService::new(Arc::new(MockUserRepository::new()))
}

#[tokio::test]
async fn registra_usuario_y_hashea_la_contrasena_correctamente() {
    let service = build_service();

    let user = service
        .register_user(
            "johndoe".to_string(),
            "john@example.com".to_string(),
            "supersecret123".to_string(),
        )
        .await
        .expect("el registro debería ser exitoso");

    // La contraseña nunca debe quedar en texto plano.
    assert_ne!(user.password_hash, "supersecret123");
    assert!(user.password_hash.starts_with("$argon2"));

    // El hash debe poder verificarse contra la contraseña original...
    assert!(UserService::verify_password("supersecret123", &user.password_hash).unwrap());
    // ...y fallar contra una contraseña incorrecta.
    assert!(!UserService::verify_password("otra-clave", &user.password_hash).unwrap());
}

#[tokio::test]
async fn falla_si_la_contrasena_es_demasiado_corta() {
    let service = build_service();

    let result = service
        .register_user(
            "johndoe".to_string(),
            "john@example.com".to_string(),
            "123".to_string(),
        )
        .await;

    assert!(matches!(result, Err(DomainError::Validation(_))));
}

#[tokio::test]
async fn falla_si_el_email_es_invalido() {
    let service = build_service();

    let result = service
        .register_user(
            "johndoe".to_string(),
            "no-es-un-email".to_string(),
            "supersecret123".to_string(),
        )
        .await;

    assert!(matches!(result, Err(DomainError::Validation(_))));
}

#[tokio::test]
async fn falla_si_el_username_es_demasiado_corto() {
    let service = build_service();

    let result = service
        .register_user(
            "jo".to_string(),
            "john@example.com".to_string(),
            "supersecret123".to_string(),
        )
        .await;

    assert!(matches!(result, Err(DomainError::Validation(_))));
}

#[tokio::test]
async fn falla_si_el_email_ya_esta_registrado() {
    let service = build_service();

    service
        .register_user(
            "johndoe".to_string(),
            "john@example.com".to_string(),
            "supersecret123".to_string(),
        )
        .await
        .unwrap();

    let result = service
        .register_user(
            "janedoe".to_string(),
            "john@example.com".to_string(),
            "otraclave123".to_string(),
        )
        .await;

    assert!(matches!(result, Err(DomainError::Conflict(_))));
}

#[tokio::test]
async fn get_user_retorna_not_found_si_no_existe() {
    let service = build_service();
    let result = service.get_user(Uuid::new_v4()).await;
    assert!(matches!(result, Err(DomainError::NotFound)));
}

#[tokio::test]
async fn update_user_actualiza_username_y_email() {
    let service = build_service();

    let user = service
        .register_user(
            "johndoe".to_string(),
            "john@example.com".to_string(),
            "supersecret123".to_string(),
        )
        .await
        .unwrap();

    let updated = service
        .update_user(
            user.id,
            Some("john_updated".to_string()),
            Some("john2@example.com".to_string()),
        )
        .await
        .unwrap();

    assert_eq!(updated.username, "john_updated");
    assert_eq!(updated.email, "john2@example.com");
    // El hash de la contraseña no debe verse alterado por un update de perfil.
    assert_eq!(updated.password_hash, user.password_hash);
}

#[tokio::test]
async fn update_user_falla_con_email_duplicado() {
    let service = build_service();

    service
        .register_user(
            "johndoe".to_string(),
            "john@example.com".to_string(),
            "supersecret123".to_string(),
        )
        .await
        .unwrap();

    let jane = service
        .register_user(
            "janedoe".to_string(),
            "jane@example.com".to_string(),
            "supersecret123".to_string(),
        )
        .await
        .unwrap();

    let result = service
        .update_user(jane.id, None, Some("john@example.com".to_string()))
        .await;

    assert!(matches!(result, Err(DomainError::Conflict(_))));
}

#[tokio::test]
async fn delete_user_elimina_correctamente() {
    let service = build_service();

    let user = service
        .register_user(
            "johndoe".to_string(),
            "john@example.com".to_string(),
            "supersecret123".to_string(),
        )
        .await
        .unwrap();

    service.delete_user(user.id).await.unwrap();

    let result = service.get_user(user.id).await;
    assert!(matches!(result, Err(DomainError::NotFound)));
}

#[tokio::test]
async fn delete_user_falla_si_no_existe() {
    let service = build_service();
    let result = service.delete_user(Uuid::new_v4()).await;
    assert!(matches!(result, Err(DomainError::NotFound)));
}

#[tokio::test]
async fn get_all_users_pagina_los_resultados() {
    let service = build_service();

    for i in 0..5 {
        service
            .register_user(
                format!("user{i}"),
                format!("user{i}@example.com"),
                "supersecret123".to_string(),
            )
            .await
            .unwrap();
    }

    let page1 = service.get_all_users(Some(1), Some(2)).await.unwrap();
    assert_eq!(page1.items.len(), 2);
    assert_eq!(page1.page, 1);
    assert_eq!(page1.page_size, 2);
    assert_eq!(page1.total_items, 5);
    assert_eq!(page1.total_pages, 3);

    let last_page = service.get_all_users(Some(3), Some(2)).await.unwrap();
    assert_eq!(last_page.items.len(), 1);
}

#[tokio::test]
async fn get_all_users_usa_valores_por_defecto_si_no_se_especifican() {
    let service = build_service();

    service
        .register_user(
            "johndoe".to_string(),
            "john@example.com".to_string(),
            "supersecret123".to_string(),
        )
        .await
        .unwrap();

    let result = service.get_all_users(None, None).await.unwrap();
    assert_eq!(result.page, 1);
    assert_eq!(result.page_size, 20);
    assert_eq!(result.total_items, 1);
}
