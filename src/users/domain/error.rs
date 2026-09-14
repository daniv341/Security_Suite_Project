//! Errores de negocio del dominio de usuarios.

use thiserror::Error;

/// Los adaptadores de infraestructura (HTTP, base de datos) traducen
/// estos errores a su propio formato (códigos HTTP, logs, etc.) sin
/// que el dominio conozca esos detalles.
#[derive(Debug, Error)]
pub enum DomainError {
    #[error("Usuario no encontrado")]
    NotFound,

    #[error("Error de validación: {0}")]
    Validation(String),

    #[error("Conflicto: {0}")]
    Conflict(String),

    #[error("Error de repositorio: {0}")]
    Repository(String),

    #[error("Credenciales inválidas")]
    Unauthorized,

    #[error("Error al hashear la contraseña: {0}")]
    Hashing(String),
}
