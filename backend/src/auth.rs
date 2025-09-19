use std::env;

use dotenvy::dotenv;
use jsonwebtoken::{encode, Header, EncodingKey};
use chrono::Utc;
use jsonwebtoken::{decode, DecodingKey, Validation};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Payload {
    id: String,
    exp: usize,
}

pub fn create_jwt(user_id: String) -> Result<String, jsonwebtoken::errors::Error> {
    dotenv().ok();

    let expiration = Utc::now()
        .checked_add_signed(chrono::Duration::minutes(60))
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = Payload {
        id: user_id,
        exp: expiration,
    };
    let jwt_secret = env::var("JWT_SECRET")
        .unwrap_or_else(|_| panic!("JWT_SECRET must be set"));

    encode(&Header::default(), &claims, &EncodingKey::from_secret(jwt_secret.as_ref()))
}

pub fn verify_jwt(token: &str) -> Result<Payload, jsonwebtoken::errors::Error> {
    let jwt_secret = env::var("JWT_SECRET")
        .unwrap_or_else(|_| panic!("JWT_SECRET must be set"));

    decode::<Payload>(&token, &DecodingKey::from_secret(jwt_secret.as_ref()), &Validation::default())
        .map(|data| data.claims)
}