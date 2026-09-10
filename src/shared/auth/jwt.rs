use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use time::{Duration, OffsetDateTime};

use super::claims::Claims;
use super::error::AuthError;

pub struct JwtService {
    encoding_key: EncodingKey, // clave para firmr el token
    decoding_key: DecodingKey, // clave para verificar el token
    expiration: Duration, // duracion del token
}

impl JwtService {
    // crea un nuevo servicio JWT con la clave y la duracion
    pub fn new(secret: &str, expiration_seconds: i64) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
            expiration: Duration::seconds(expiration_seconds),
        }
    }

    // crea un token para el usuario
    pub fn create_token(&self, user_id: &str) -> Result<String, AuthError> {
        let now = OffsetDateTime::now_utc().unix_timestamp();

        let claims = Claims {
            sub: user_id.to_string(),
            iat: now as usize,
            exp: (now + self.expiration.whole_seconds()) as usize,
        };

        encode(
            &Header::default(),
            &claims,
            &self.encoding_key,
        )
        .map_err(|_| AuthError::TokenCreation)
    }

    // valida el token y devuelve los datos de claims
    pub fn validate_token(&self, token: &str) -> Result<Claims, AuthError> {
        decode::<Claims>(
            token,
            &self.decoding_key,
            &Validation::default(),
        )
        .map(|data| data.claims)
        .map_err(|_| AuthError::InvalidToken)
    }
}