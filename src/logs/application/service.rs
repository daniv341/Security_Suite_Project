//! Capa de aplicación: implementa `LogServicePort`.
//! Acá va la lógica de negocio (validaciones, orquestación del
//! repositorio) — lo equivalente a `UserService` para `User`.

use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use crate::shared::pagination::{PaginatedResult, Pagination};
use crate::logs::domain::{DomainError, Log, LogRepository, LogServicePort, ActionLog, ResourceLog};

/// Longitud mínima aceptada para `details`. Ajustá o quitá según haga
/// falta.
const MIN_DETAILS_LEN: usize = 3;

pub struct LogService {
    repository: Arc<dyn LogRepository>,
}

impl LogService {
    pub fn new(repository: Arc<dyn LogRepository>) -> Self {
        Self { repository }
    }

    fn validate_details(details: &str) -> Result<(), DomainError> {
        if details.trim().len() < MIN_DETAILS_LEN {
            return Err(DomainError::Validation(format!(
                "Los detalles deben tener al menos {} caracteres",
                MIN_DETAILS_LEN
            )));
        }
        Ok(())
    }
}

#[async_trait]
impl LogServicePort for LogService {
    async fn create_log(
        &self, 
        user_id: Option<Uuid>, 
        action: ActionLog, 
        resource: ResourceLog, 
        resource_id: Option<Uuid>, 
        details: String
    ) -> Result<Log, DomainError> {
        Self::validate_details(&details)?;

        let log = Log::new(user_id, action, resource, resource_id, details);
        self.repository.create(&log).await
    }

    async fn get_log(&self, id: Uuid) -> Result<Log, DomainError> {
        self.repository
            .find_by_id(id)
            .await?
            .ok_or(DomainError::NotFound)
    }

    async fn get_all_logs_by_user_id(
        &self,
        user_id: Uuid,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<PaginatedResult<Log>, DomainError> {
        let pagination = Pagination::new(page, page_size);
        let logs = self.repository.find_all_by_user_id(user_id, pagination).await?;
        let total_logs = self.repository.count_all().await?;
        Ok(PaginatedResult::new(logs, pagination, total_logs))
    }

    async fn get_all_logs_by_resource(
        &self,
        resource: &ResourceLog,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<PaginatedResult<Log>, DomainError> {
        let pagination = Pagination::new(page, page_size);
        let logs = self.repository.find_all_by_resource(resource, pagination).await?;
        let total_logs = self.repository.count_all().await?;
        Ok(PaginatedResult::new(logs, pagination, total_logs))
    }

    async fn get_all_logs_by_action(
        &self,
        action: &ActionLog,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<PaginatedResult<Log>, DomainError> {
        let pagination = Pagination::new(page, page_size);
        let logs = self.repository.find_all_by_action(action, pagination).await?;
        let total_logs = self.repository.count_all().await?;
        Ok(PaginatedResult::new(logs, pagination, total_logs))
    }
}
