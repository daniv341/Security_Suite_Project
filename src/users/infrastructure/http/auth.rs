use std::sync::Arc;
use uuid::Uuid;

use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::{
        header::AUTHORIZATION,
        request::Parts,
        StatusCode,
    },
};

use crate::shared::state::{AppState, LoginState};

pub struct AuthenticatedUser {
    pub user_id: Uuid,
    pub jti: Uuid,
    pub exp: usize,
}

#[async_trait]
impl FromRequestParts<AppState> for AuthenticatedUser {
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let login_state = Arc::<LoginState>::from_ref(state);

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

        let claims = login_state
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

        let jti = Uuid::parse_str(&claims.jti)
            .map_err(|_| {
                (
                    StatusCode::UNAUTHORIZED,
                    "Invalid token ID",
                )
            })?;

        Ok(Self {
            user_id,
            jti,
            exp: claims.exp,
        })
    }
}