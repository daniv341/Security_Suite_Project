use std::sync::Arc;


use crate::shared::auth::JwtService;
use crate::users::application::service::UserService;
use crate::users::domain::{DomainError, UserRepository};

// estructura del servicio de login
pub struct LoginService {
    repository: Arc<dyn UserRepository>,
    jwt_service: Arc<JwtService>,
}

// implementa el servicio
impl LoginService {
    pub fn new(
        repository: Arc<dyn UserRepository>,
        jwt_service: Arc<JwtService>,
    ) -> Self {
        Self {
            repository,
            jwt_service,
        }
    }

    // recibe el mail y el password y devuelve el JWT si las credenciales son correctas
    pub async fn login(
        &self,
        email: &str,
        password: &str,
    ) -> Result<String, DomainError> {
        let user = self
            .repository
            .find_by_email(email)
            .await?
            .ok_or_else(|| DomainError::Unauthorized)?;

        let password_valid =
            UserService::verify_password(password, &user.password_hash)?;

        if !password_valid {
            return Err(DomainError::Unauthorized);
        }

        let token = self
            .jwt_service
            .create_token(&user.id.to_string())
            .map_err(|e| DomainError::Validation(e.to_string()))?;

        Ok(token)
    }
}