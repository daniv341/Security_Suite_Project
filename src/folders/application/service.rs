//! Capa de aplicación: implementa `FolderServicePort`.
//! Acá va la lógica de negocio (validaciones, orquestación del
//! repositorio) — lo equivalente a `UserService` para `User`.

use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use uuid::Uuid;

use crate::shared::pagination::{PaginatedResult, Pagination};
use crate::folders::domain::{DomainError, Folder, FolderRepository, FolderServicePort};

/// Longitud mínima aceptada para `name`. Ajustá o quitá según haga
/// falta.
const MIN_NAME_LEN: usize = 3;

// estructura de FolderService
pub struct FolderService {
    repository: Arc<dyn FolderRepository>,
}

// implementación de FolderService
impl FolderService {
    // crea un nuevo FolderService con el repositorio
    pub fn new(repository: Arc<dyn FolderRepository>) -> Self {
        Self { repository }
    }

    // valida el nombre del folder
    fn validate_name(name: &str) -> Result<(), DomainError> {
        if name.trim().len() < MIN_NAME_LEN {
            return Err(DomainError::Validation(format!(
                "El name debe tener al menos {} caracteres",
                MIN_NAME_LEN
            )));
        }
        Ok(())
    }

    // valida el id del usuario
    fn validate_user_id(user_id: &Uuid) -> Result<(), DomainError> {
        if user_id.is_nil() {
            return Err(DomainError::Validation(
                "El ID de usuario no puede ser nulo".to_string(),
            ));
        }
        Ok(())
    }
}

// implementación de FolderServicePort para FolderService
#[async_trait]
impl FolderServicePort for FolderService {
    // crea un folder para un usuario
    async fn create_folder( // parametros de creacion de folder
        &self, 
        user_id: Uuid,
        name: String,
    ) -> Result<Folder, DomainError> { // validaciones previas a la creacion de folder
        Self::validate_name(&name)?;
        Self::validate_user_id(&user_id)?;

        // verifica si ya existe un folder con ese name
        if self.repository.find_by_name(user_id, &name).await?.is_some() {
            return Err(DomainError::Conflict(format!(
                "Ya existe un/a folder con el name '{}'",
                name
            )));
        }

        let folder = Folder::new(user_id, name); // guarda el folder en el repositorio
        self.repository.create(&folder).await // devuelve el folder creado
    }

    // obtiene un folder por id
    async fn get_folder(&self, user_id: Uuid, id: Uuid) -> Result<Folder, DomainError> {
        self.repository
            .find_by_id(user_id, id) // usa el repositorio para buscar el folder por id
            .await?
            .ok_or(DomainError::NotFound)
    }

    async fn get_all_folders(
        &self,
        user_id: Uuid,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<PaginatedResult<Folder>, DomainError> {
        let pagination = Pagination::new(page, page_size);
        let folders = self.repository.find_all(user_id, pagination).await?;
        let total_folders = self.repository.count_all(user_id).await?;
        Ok(PaginatedResult::new(folders, pagination, total_folders))
    }

    // actualiza un folder existente por id y por user_id de forma opcional
    async fn update_folder(&self, user_id: Uuid, id: Uuid, name: Option<String>) -> Result<Folder, DomainError> {
        let mut folder = self
            .repository
            .find_by_id(user_id, id) // primero busca el folder por id
            .await?
            .ok_or(DomainError::NotFound)?;

        if let Some(new_name) = name {
            Self::validate_name(&new_name)?; // primero valida el nuevo nombre

            if let Some(existing) = self.repository.find_by_name(user_id, &new_name).await? { // luego pregunta si ya existe un folder con ese nombre
                if existing.id != folder.id { // luego compara los ids para ver si es el mismo folder o no
                    return Err(DomainError::Conflict(format!(
                        "Ya existe un/a folder con el name '{}'",
                        new_name
                    )));
                }
            }

            folder.name = new_name; // finalmente actualiza el nombre del folder
        }

        folder.updated_at = Utc::now(); // actualiza la fecha de actualización del folder
        self.repository.update(user_id, &folder).await // devuelve el folder actualizado
    }

    // elimina un folder por id
    async fn delete_folder(&self, user_id: Uuid, id: Uuid) -> Result<(), DomainError> {
        self.repository
            .find_by_id(user_id, id) // primero busca el folder por id
            .await?
            .ok_or(DomainError::NotFound)?;

        self.repository.delete(user_id, id).await // finalmente elimina el folder del repositorio
    }
}
