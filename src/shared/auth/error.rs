use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("invalid token")]
    InvalidToken,

    #[error("token creation failed")]
    TokenCreation,
}