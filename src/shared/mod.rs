//! "Shared kernel": utilidades y value objects genéricos que no
//! pertenecen a ningún objeto de negocio en particular y por eso no
//! tiene sentido duplicar dentro de `users/` (ni de cualquier otro
//! módulo de objeto que se agregue después, como `credentials/` o
//! `sessions/`).
//!
//! Si en algún momento `Pagination`/`PaginatedResult<T>` necesitaran
//! reglas específicas para un objeto puntual, ese objeto puede seguir
//! usando lo genérico de acá y sumar su propia especialización dentro
//! de su propia carpeta — no hace falta mover nada de este módulo.

pub mod auth;
pub mod pagination;
pub mod state;
