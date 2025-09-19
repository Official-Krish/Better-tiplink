use std::sync::{Arc, Mutex};

use actix_web::{web::Data, App, HttpServer};

mod routes;
use routes::*;
use store::Store;
mod auth;
mod middleware;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let s = match Store::new().await {
        Ok(store) => store.backend,
        Err(e) => {
            eprintln!("Failed to initialize the store: {}", e);
            std::process::exit(1);
        }
    };
    let arced_s = Arc::new(Mutex::new(s));
    HttpServer::new(move || {
        App::new()
            .service(sign_up)  
            .service(sign_in)
            .service(get_user).wrap(middleware::AuthMiddleware)
            .service(quote).wrap(middleware::AuthMiddleware)
            .service(swap).wrap(middleware::AuthMiddleware)
            .service(sol_balance).wrap(middleware::AuthMiddleware)
            .service(token_balance).wrap(middleware::AuthMiddleware)
            .app_data(Data::new(arced_s.clone()))
    })
    .bind("127.0.0.1:3000")?
    .run()
    .await
}