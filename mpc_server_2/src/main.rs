use actix_web::{web::{self, Data}, App, HttpResponse, HttpServer, Result};
use solana_sdk::{signature::Keypair, signer::Signer};
use store::{Store};
use base64::engine::Engine;
use std::sync::{Arc, Mutex};
use serde::{Serialize, Deserialize};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let s = match Store::new().await {
        Ok(store) => store,
        Err(e) => {
            eprintln!("Failed to initialize the store: {}", e);
            std::process::exit(1);
        }
    };
    let arced_s = Arc::new(Mutex::new(s.mpc_server_2));
    HttpServer::new(move || {
        App::new()
            .service(generate)
            .app_data(Data::new(arced_s.clone()))
    })
    .bind("127.0.0.1:9001")?
    .run()
    .await
}

#[derive(Serialize, Deserialize)]
pub struct GenerateOutput {
    pub pubkey: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct GeneratePubKeyInput {
    pub user_id: String,
}

#[actix_web::post("/generatePubKey")]
pub async fn generate(store: web::Data<Arc<Mutex<Store>>>, data: web::Json<GeneratePubKeyInput>) -> Result<HttpResponse> {
    let user_id = data.user_id.clone();
    let keypair = Keypair::new();
    let locked_store = match store.lock() {
        Ok(locked) => locked,
        Err(_) => return Ok(HttpResponse::InternalServerError().body("Failed to lock store")),
    };

    let keypair = match locked_store.store_keypair_mpc_2(
        &keypair.pubkey().to_string(),
        &base64::engine::general_purpose::STANDARD.encode(keypair.to_bytes()),
        &user_id
    ).await {
        Ok(kp) => kp,
        Err(e) => {
            eprintln!("Failed to insert keypair: {}", e);
            return Ok(HttpResponse::InternalServerError().body("Failed to insert keypair"));
        }
    };

    Ok(HttpResponse::Ok().json(GenerateOutput {
        pubkey: keypair.public_key,
    }))
}