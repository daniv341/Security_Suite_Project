//! Librería del crate `security-suite`.
//!
//! Organización por objeto de negocio (feature-first): cada módulo de
//! objeto (por ahora, solo `users`) contiene su propia arquitectura
//! hexagonal completa —`domain/`, `application/`, `infrastructure/`—
//! autocontenida. `shared/` guarda lo genérico que cualquier objeto
//! puede reutilizar (por ejemplo, paginación), sin pertenecerle a
//! ninguno en particular.
//!
//! Se expone como librería (además del binario en `main.rs`) para que
//! las pruebas de integración en `tests/` puedan importar y ejercitar
//! la capa de aplicación y de dominio de forma aislada.

pub mod shared;
pub mod users;
pub mod folders;
