//! Módulo del objeto `Log`: arquitectura hexagonal completa y
//! autocontenida (dominio, aplicación e infraestructura).
//!
//! Generado por `scripts/scaffold_object.py` a partir de la plantilla
//! genérica (id, name, created_at, updated_at) — ajustá según la
//! lógica real de `Log`.

pub mod application;
pub mod domain;
pub mod infrastructure;
