use std::collections::HashMap;
use std::sync::Mutex;

use async_trait::async_trait;
use uuid::Uuid;

use security_suite::folders::domain;
use security_suite::shared::pagination::{PaginatedResult, Pagination};
use security_suite::users::domain;

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
    async fn create(&self, user: &User) -> Result<User, DomainError> {
        let mut users = self.users.lock().unwrap();
        users.insert(user.id, user.clone());
        Ok(user.clone())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, DomainError> {
        let users = self.users.lock().unwrap();
        Ok(users.get(&id).cloned())
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>, DomainError> {
        let users = self.users.lock().unwrap();
        Ok(users.values().find(|u| u.email == email).cloned())
    }

    async fn find_all(&self, pagination: Pagination) -> Result<Vec<User>, DomainError> {
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

    async fn count_all(&self) -> Result<i64, DomainError> {
        let users = self.users.lock().unwrap();
        Ok(users.len() as i64)
    }

    async fn update(&self, user: &User) -> Result<User, DomainError> {
        let mut users = self.users.lock().unwrap();
        users.insert(user.id, user.clone());
        Ok(user.clone())
    }

    async fn delete(&self, id: Uuid) -> Result<(), DomainError> {
        let mut users = self.users.lock().unwrap();
        users.remove(&id).ok_or(DomainError::NotFound)?;
        Ok(())
    }
}

pub struct MockFolderService;

#[async_trait]
impl FolderServicePort for MockFolderService {
    async fn create_folder(
        &self,
        _user_id: Uuid,
        _name: String,
    ) -> Result<Folder, DomainError> {
        unimplemented!()
    }

    async fn get_folder(
        &self,
        _user_id: Uuid,
        _id: Uuid,
    ) -> Result<Folder, DomainError> {
        unimplemented!()
    }

    async fn get_all_folders(
        &self,
        _user_id: Uuid,
        _page: Option<i64>,
        _page_size: Option<i64>,
    ) -> Result<PaginatedResult<Folder>, DomainError> {
        unimplemented!()
    }

    async fn update_folder(
        &self,
        _user_id: Uuid,
        _id: Uuid,
        _name: Option<String>,
    ) -> Result<Folder, DomainError> {
        unimplemented!()
    }

    async fn delete_folder(
        &self,
        _user_id: Uuid,
        _id: Uuid,
    ) -> Result<(), DomainError> {
        unimplemented!()
    }
}