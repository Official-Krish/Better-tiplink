use actix_web::{web::{self, Data}, App, HttpResponse, HttpServer, Result};
use solana_sdk::{signature::Keypair};
use store::Store;
use std::sync::{Arc, Mutex};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let s = match Store::new().await {
        Ok(store) => store,
        Err(e) => {
            eprintln!("Failed to initialize the store: {}", e);
            std::process::exit(1);
        }
    };
    let arced_s = Arc::new(Mutex::new(s.mpc_server_1));
    HttpServer::new(move || {
        App::new()
            .service(generate)
            .app_data(Data::new(arced_s.clone()))
    })
    .bind("127.0.0.1:9000")?
    .run()
    .await
}

#[actix_web::post("/generate")]
pub async fn generate(store: web::Data<Arc<Mutex<Store>>>) -> Result<HttpResponse> {
    let keypair = Keypair::new();
    let locked_store = match store.lock() {
        Ok(locked) => locked,
        Err(_) => return Ok(HttpResponse::InternalServerError().body("Failed to lock store")),
    };

    Ok(HttpResponse::Ok().json(format!("Generated keypair: {:?}", keypair)))
}