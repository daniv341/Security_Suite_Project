//! Capa de dominio del objeto `User`: entidad, errores y puertos.
//! No depende de Axum, SQLx ni de ningún detalle de infraestructura.

mod entity;
mod error;
mod repository;
mod service;

pub use entity::User;
pub use error::DomainError;
pub use repository::UserRepository;
pub use service::UserServicePort;
