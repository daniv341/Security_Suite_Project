//! Capa de dominio del objeto `Log`: entidad, errores y puertos.
//! No depende de Axum, SQLx ni de ningún detalle de infraestructura.

mod entity;
mod error;
mod repository;
mod service;

pub use entity::Log;
pub use entity::ActionLog;
pub use entity::ResourceLog;
pub use error::DomainError;
pub use repository::LogRepository;
pub use service::LogServicePort;
