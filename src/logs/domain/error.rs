//! Errores de negocio del dominio de `logs`.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("Log no encontrado")]
    NotFound,

    #[error("Error de validación: {0}")]
    Validation(String),

    #[error("Error de repositorio: {0}")]
    Repository(String),
}
