use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // sub: identificador del usuario
    pub jti: String,
    pub exp: usize, // exp: cuándo expira
    pub iat: usize, // iat: cuándo fue creado el token
}