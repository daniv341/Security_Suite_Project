use std::sync::Arc;
use axum::extract::FromRef;

use crate::users::domain::UserServicePort;
use crate::folders::domain::FolderServicePort;

use crate::users::application::login_service::LoginService;
use crate::shared::auth::JwtService;

pub struct UserState {
    pub user_service: Arc<dyn UserServicePort>,
}

pub struct FolderState {
    pub folder_service: Arc<dyn FolderServicePort>,
}

pub struct LoginState {
    pub login_service: Arc<LoginService>,
    pub jwt_service: Arc<JwtService>,
}

#[derive(Clone)]
pub struct AppState {
    pub user_state: Arc<UserState>,
    pub login_state: Arc<LoginState>,
    pub folder_state: Arc<FolderState>,
}

impl FromRef<AppState> for Arc<UserState> {
    fn from_ref(state: &AppState) -> Self {
        state.user_state.clone()
    }
}

impl FromRef<AppState> for Arc<LoginState> {
    fn from_ref(state: &AppState) -> Self {
        state.login_state.clone()
    }
}

impl FromRef<AppState> for Arc<FolderState> {
    fn from_ref(state: &AppState) -> Self {
        state.folder_state.clone()
    }
}