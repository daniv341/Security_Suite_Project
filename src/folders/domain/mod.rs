//! Capa de dominio del objeto `Folder`: entidad, errores y puertos.
//! No depende de Axum, SQLx ni de ningún detalle de infraestructura.

//mod se usa para declarar submodulos dentro de otro modulo(folder)
mod entity;
mod error;
mod repository;
mod service;

// re exportar elementos para que se puedan usar fuera de este modulo
pub use entity::Folder;
pub use error::DomainError;
pub use repository::FolderRepository;
pub use service::FolderServicePort;
