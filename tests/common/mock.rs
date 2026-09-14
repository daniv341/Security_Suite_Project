use std::collections::HashMap;
use std::sync::Mutex;

use async_trait::async_trait;
use uuid::Uuid;

use security_suite::folders::domain::{DomainError as FolderError, Folder, FolderServicePort};
use security_suite::shared::pagination::{PaginatedResult, Pagination};
use security_suite::users::domain::{DomainError as UserError, User, UserRepository};

pub struct MockUserRepository {
    users: Mutex<HashMap<Uuid, User>>,
}

impl MockUserRepository {
    pub fn new() -> Self {
        Self {
            users: Mutex::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl UserRepository for MockUserRepository {
    async fn create(&self, user: &User) -> Result<User, UserError> {
        let mut users = self.users.lock().unwrap();
        users.insert(user.id, user.clone());
        Ok(user.clone())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, UserError> {
        let users = self.users.lock().unwrap();
        Ok(users.get(&id).cloned())
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>, UserError> {
        let users = self.users.lock().unwrap();
        Ok(users.values().find(|u| u.email == email).cloned())
    }

    async fn find_all(&self, pagination: Pagination) -> Result<Vec<User>, UserError> {
        let users = self.users.lock().unwrap();
        let mut all: Vec<User> = users.values().cloned().collect();
        all.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        let start = pagination.offset() as usize;
        if start >= all.len() {
            return Ok(vec![]);
        }

        let end = (start + pagination.limit() as usize).min(all.len());

        Ok(all[start..end].to_vec())
    }

    async fn count_all(&self) -> Result<i64, UserError> {
        let users = self.users.lock().unwrap();
        Ok(users.len() as i64)
    }

    async fn update(&self, user: &User) -> Result<User, UserError> {
        let mut users = self.users.lock().unwrap();
        users.insert(user.id, user.clone());
        Ok(user.clone())
    }

    async fn delete(&self, id: Uuid) -> Result<(), UserError> {
        let mut users = self.users.lock().unwrap();
        users.remove(&id).ok_or(UserError::NotFound)?;
        Ok(())
    }
}

pub struct MockFolderService {
    folders: Mutex<HashMap<Uuid, Folder>>,
}

impl MockFolderService {
    pub fn new() -> Self {
        Self {
            folders: Mutex::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl FolderServicePort for MockFolderService {
    async fn create_folder(
        &self, user_id: Uuid, name: String,) -> Result<Folder, FolderError> {
        if name.trim().len() < 3 {
            return Err(FolderError::Validation(
                "El nombre debe tener al menos 3 caracteres".to_string(),
            ));
        }

        let mut folders = self.folders.lock().unwrap();

        if folders.values().any(|folder| folder.user_id == user_id && folder.name == name)
        {
            return Err(FolderError::Conflict(format!("Ya existe un/a folder con el name '{}'", name)));
        }

        let folder = Folder::new(user_id, name);
        folders.insert(folder.id, folder.clone());

        Ok(folder)
    }

    async fn get_folder(&self, user_id: Uuid, id: Uuid) -> Result<Folder, FolderError> {
        let folders = self.folders.lock().unwrap();

        folders.get(&id).filter(|folder| folder.user_id == user_id).cloned().ok_or(FolderError::NotFound)
    }

    async fn get_all_folders(&self, user_id: Uuid, page: Option<i64>, page_size: Option<i64>) -> Result<PaginatedResult<Folder>, FolderError> {
        let pagination = Pagination::new(page, page_size);
        let folders = self.folders.lock().unwrap();

        let mut user_folders: Vec<Folder> = folders.values().filter(|folder| folder.user_id == user_id).cloned().collect();

        user_folders.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        let total_folders = user_folders.len() as i64;

        let start = pagination.offset() as usize;
        let end = (start + pagination.limit() as usize).min(user_folders.len());

        let page_folders = if start >= user_folders.len() {
            vec![]
        } else {
            user_folders[start..end].to_vec()
        };

        Ok(PaginatedResult::new(page_folders, pagination, total_folders))
    }

    async fn update_folder(&self, user_id: Uuid, id: Uuid, name: Option<String>) -> Result<Folder, FolderError> {
        let mut folders = self.folders.lock().unwrap();

        let mut folder = folders.get(&id).filter(|folder| folder.user_id == user_id).cloned().ok_or(FolderError::NotFound)?;

        if let Some(new_name) = name {
            if new_name.trim().len() < 3 {
                return Err(FolderError::Validation(
                    "El nombre debe tener al menos 3 caracteres".to_string(),
                ));
            }

            if folders.values().any(|existing| {
                existing.user_id == user_id && existing.name == new_name && existing.id != id
            }) {
                return Err(FolderError::Conflict(format!(
                    "Ya existe un/a folder con el name '{}'", new_name)));
            }
            folder.name = new_name;
        }

        folder.updated_at = chrono::Utc::now();
        folders.insert(id, folder.clone());

        Ok(folder)
    }

    async fn delete_folder(&self, user_id: Uuid, id: Uuid) -> Result<(), FolderError> {
        let mut folders = self.folders.lock().unwrap();

        let exists = folders.get(&id).map(|folder| folder.user_id == user_id).unwrap_or(false);

        if !exists {
            return Err(FolderError::NotFound);
        }

        folders.remove(&id);
        Ok(())
    }
}