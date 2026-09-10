use std::sync::Arc;
use uuid::Uuid;

use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{
        header::AUTHORIZATION,
        request::Parts,
        StatusCode,
    },
};

use crate::users::infrastructure::http::handlers::AppState;

pub struct AuthenticatedUser {
    pub user_id: Uuid,
}

#[async_trait]
impl FromRequestParts<Arc<AppState>> for AuthenticatedUser {
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get(AUTHORIZATION)
            .ok_or((
                StatusCode::UNAUTHORIZED,
                "Missing Authorization header",
            ))?;

        let auth_header = auth_header
            .to_str()
            .map_err(|_| {
                (
                    StatusCode::UNAUTHORIZED,
                    "Invalid Authorization header",
                )
            })?;

        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or((
                StatusCode::UNAUTHORIZED,
                "Invalid Authorization scheme",
            ))?;

        let claims = state
            .jwt_service
            .validate_token(token)
            .map_err(|_| {
                (
                    StatusCode::UNAUTHORIZED,
                    "Invalid or expired token",
                )
            })?;

        let user_id = Uuid::parse_str(&claims.sub)
            .map_err(|_| {
                (
                    StatusCode::UNAUTHORIZED,
                    "Invalid user ID",
                )
            })?;

        Ok(Self {
            user_id
        })
    }
}