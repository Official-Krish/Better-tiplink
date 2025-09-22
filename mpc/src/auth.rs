use std::env;
use dotenvy::dotenv;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use chrono::Utc;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Payload {
    pub id: String,
    pub exp: usize,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct InternalPayload {
    pub id: String,
    pub exp: usize,
}

pub fn verify_jwt(token: &str) -> Result<Payload, jsonwebtoken::errors::Error> {
    let jwt_secret = env::var("JWT_SECRET")
        .unwrap_or_else(|_| panic!("JWT_SECRET must be set"));

    decode::<Payload>(&token, &DecodingKey::from_secret(jwt_secret.as_ref()), &Validation::default())
        .map(|data| data.claims)
}

pub fn create_jwt_for_communication(id: String) -> Result<String, jsonwebtoken::errors::Error> {
    dotenv().ok();

    let expiration = Utc::now()
        .checked_add_signed(chrono::Duration::minutes(60))
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = InternalPayload {
        id: id,
        exp: expiration,
    };
    let jwt_secret = env::var("JWT_SECRET")
        .unwrap_or_else(|_| panic!("JWT_SECRET must be set"));

    encode(&Header::default(), &claims, &EncodingKey::from_secret(jwt_secret.as_ref()))
}