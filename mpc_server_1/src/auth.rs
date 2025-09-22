use std::env;
use jsonwebtoken::{decode, DecodingKey, Validation};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Payload {
    pub id: String,
    pub exp: usize,
}

pub fn verify_jwt(token: &str) -> Result<Payload, jsonwebtoken::errors::Error> {
    let jwt_secret = env::var("JWT_SECRET")
        .unwrap_or_else(|_| panic!("JWT_SECRET must be set"));

    decode::<Payload>(&token, &DecodingKey::from_secret(jwt_secret.as_ref()), &Validation::default())
        .map(|data| data.claims)
}