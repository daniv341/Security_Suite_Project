//! Errores de negocio del dominio de `folders`.

use thiserror::Error;

// errores de negocio para folders
#[derive(Debug, Error)]
pub enum DomainError {
    #[error("Folder no encontrado")]
    NotFound, // notfound significa que no se encontro

    #[error("Error de validación: {0}")]
    Validation(String), // validation significa input no es valido

    #[error("Conflicto: {0}")]
    Conflict(String), //conflict significa que ya existe un folder

    #[error("Error de repositorio: {0}")]
    Repository(String), //respositoy significa que hubo un error en la capa de repository
}
