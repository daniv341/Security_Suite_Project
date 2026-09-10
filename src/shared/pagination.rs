//! Value objects de paginación, reutilizables por cualquier entidad
//! del dominio que necesite listar resultados de a páginas.

use serde::Serialize;

/// Tamaño de página por defecto cuando el cliente no especifica uno.
pub const DEFAULT_PAGE_SIZE: i64 = 20;
/// Tamaño de página máximo permitido, para evitar consultas que
/// devuelvan toda la tabla de una sola vez.
pub const MAX_PAGE_SIZE: i64 = 100;

/// Parámetros de paginación ya validados y normalizados: página >= 1,
/// tamaño de página entre 1 y `MAX_PAGE_SIZE`.
#[derive(Debug, Clone, Copy)]
pub struct Pagination {
    page: i64,
    page_size: i64,
}

impl Pagination {
    /// Construye una paginación válida a partir de valores opcionales
    /// provistos por el cliente, aplicando valores por defecto y
    /// límites razonables (nunca falla ni devuelve error: siempre
    /// normaliza a algo válido).
    pub fn new(page: Option<i64>, page_size: Option<i64>) -> Self {
        let page = page.unwrap_or(1).max(1);
        let page_size = page_size
            .unwrap_or(DEFAULT_PAGE_SIZE)
            .clamp(1, MAX_PAGE_SIZE);

        Self { page, page_size }
    }

    pub fn page(&self) -> i64 {
        self.page
    }

    pub fn page_size(&self) -> i64 {
        self.page_size
    }

    /// `LIMIT` a usar en la consulta SQL.
    pub fn limit(&self) -> i64 {
        self.page_size
    }

    /// `OFFSET` a usar en la consulta SQL.
    pub fn offset(&self) -> i64 {
        (self.page - 1) * self.page_size
    }
}

/// Resultado paginado genérico: una página de `items` más los
/// metadatos necesarios para que el cliente sepa cómo seguir
/// navegando.
#[derive(Debug, Clone, Serialize)]
pub struct PaginatedResult<T> {
    pub items: Vec<T>,
    pub page: i64,
    pub page_size: i64,
    pub total_items: i64,
    pub total_pages: i64,
}

impl<T> PaginatedResult<T> {
    pub fn new(items: Vec<T>, pagination: Pagination, total_items: i64) -> Self {
        let total_pages = if total_items == 0 {
            0
        } else {
            (total_items + pagination.page_size() - 1) / pagination.page_size()
        };

        Self {
            items,
            page: pagination.page(),
            page_size: pagination.page_size(),
            total_items,
            total_pages,
        }
    }

    /// Transforma cada item preservando los metadatos de paginación.
    /// Útil para mapear entidades de dominio a DTOs de respuesta sin
    /// tener que reconstruir el resto de los campos a mano.
    pub fn map<U>(self, f: impl FnMut(T) -> U) -> PaginatedResult<U> {
        PaginatedResult {
            items: self.items.into_iter().map(f).collect(),
            page: self.page,
            page_size: self.page_size,
            total_items: self.total_items,
            total_pages: self.total_pages,
        }
    }
}
