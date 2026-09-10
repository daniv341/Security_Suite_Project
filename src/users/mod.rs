//! Módulo del objeto `User`: contiene su propia arquitectura hexagonal
//! completa (dominio, aplicación e infraestructura), autocontenida.
//!
//! Si mañana se agrega otro objeto (por ejemplo `credentials` o
//! `sessions`), se replica esta misma forma como `src/credentials/`,
//! con sus propios `domain/`, `application/` e `infrastructure/` —
//! sin tocar nada de `users/`.

pub mod application;
pub mod domain;
pub mod infrastructure;
