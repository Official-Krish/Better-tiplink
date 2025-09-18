use std::sync::{Arc, Mutex};

use actix_web::{web::Data, App, HttpServer};

mod routes;
use routes::*;
use store::Store;
mod auth;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let s = match Store::new().await {
        Ok(store) => store,
        Err(e) => {
            eprintln!("Failed to initialize the store: {}", e);
            std::process::exit(1);
        }
    };
    let arced_s = Arc::new(Mutex::new(s.backend));
    HttpServer::new(move || {
        App::new()
            .service(sign_up)  
            .service(sign_in)
            .service(get_user)
            .service(quote)
            .service(swap)
            .service(sol_balance)
            .service(token_balance)
            .app_data(Data::new(arced_s.clone()))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}