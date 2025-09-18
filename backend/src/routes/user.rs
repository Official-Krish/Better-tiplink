use std::sync::{Arc, Mutex};

use actix_web::{web, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use store::{Store, user::CreateUserRequest};

use crate::auth::create_jwt;

#[derive(Deserialize)]
pub struct SignUpRequest {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct SignInRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct UserResponse {
    pub id: String,
    pub email: String,
    pub created_at: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub token: String,
}

#[actix_web::post("/signup")]
pub async fn sign_up(req: web::Json<SignUpRequest>, store: web::Data<Arc<Mutex<Store>>>) -> Result<HttpResponse> {
    let locked_store = match store.lock() {
        Ok(locked) => locked,
        Err(_) => return Ok(HttpResponse::InternalServerError().body("Failed to lock store")),
    };
    let create_user_request = CreateUserRequest {
        email: req.email.clone(),
        password: req.password.clone(),
    };
    let user = match locked_store.create_user(create_user_request).await {
        Ok(user) => user,
        Err(err) => return Ok(HttpResponse::BadRequest().body(err.to_string())),
    };
    let jwt = create_jwt(user.id.clone());
    match jwt {
        Ok(token) => {
            let response = AuthResponse { token };
            return Ok(HttpResponse::Ok().json(response));
        }
        Err(_) => {
            return Ok(HttpResponse::InternalServerError().body("Failed to create JWT"));
        }
    }
}

#[actix_web::post("/signin")]
pub async fn sign_in(req: web::Json<SignInRequest>, store: web::Data<Arc<Mutex<Store>>>) -> Result<HttpResponse> {
    let locked_store = match store.lock() {
        Ok(locked) => locked,
        Err(_) => return Ok(HttpResponse::InternalServerError().body("Failed to lock store")),
    };
    let user = match locked_store.sign_in(req.email.clone(), req.password.clone()).await {
        Ok(user) => user,
        Err(err) => return Ok(HttpResponse::BadRequest().body(err.to_string())),
    };
    let jwt = create_jwt(user.id.clone());
    match jwt {
        Ok(token) => {
            let response = AuthResponse { token };
            return Ok(HttpResponse::Ok().json(response));
        }
        Err(_) => {
            return Ok(HttpResponse::InternalServerError().body("Failed to create JWT"));
        }
    }
}

#[actix_web::get("/user/{id}")]
pub async fn get_user(path: web::Path<String>, store: web::Data<Arc<Mutex<Store>>>) -> Result<HttpResponse> {
    let user_id = path.into_inner();

    let locked_store = match store.lock() {
        Ok(locked) => locked,
        Err(_) => return Ok(HttpResponse::InternalServerError().body("Failed to lock store")),
    };
    let user = match locked_store.get_user_by_id(user_id.to_string()).await {
        Ok(user) => user,
        Err(err) => return Ok(HttpResponse::BadRequest().body(err.to_string())),
    };

    let user = UserResponse {
        id: user.id,
        email: user.email,
        created_at: user.created_at,
    };
    
    Ok(HttpResponse::Ok().json(user))
}
