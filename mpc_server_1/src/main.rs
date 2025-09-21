use actix_web::{web::{self, Data}, App, HttpResponse, HttpServer, Result};
use solana_sdk::{signature::Keypair, signer::{Signer}};
use store::{Store};
use base64::engine::Engine;
use std::sync::{Arc, Mutex};
use serde::{Serialize, Deserialize};

mod convert;

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
            .service(get_keypair)
            .app_data(Data::new(arced_s.clone()))
    })
    .bind("127.0.0.1:9000")?
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

#[derive(Serialize, Deserialize)]
pub struct GetKeyPairInput {
    pub user_id: String,
}


#[derive(Serialize)]
pub struct GetKeyPairOutput {
    pub keypair_base64: String,
}


#[actix_web::post("/generatePubKey")]
pub async fn generate(store: web::Data<Arc<Mutex<Store>>>, data: web::Json<GeneratePubKeyInput>) -> Result<HttpResponse> {
    let user_id = data.user_id.clone();
    let keypair = Keypair::new();
    let locked_store = match store.lock() {
        Ok(locked) => locked,
        Err(_) => return Ok(HttpResponse::InternalServerError().body("Failed to lock store")),
    };

    let keypair = match locked_store.store_keypair_mpc_1(
        &keypair.pubkey().to_string(),
        &base64::engine::general_purpose::STANDARD.encode(keypair.to_bytes()),
        &user_id,
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

#[actix_web::post("/getKeyPair")]
pub async fn get_keypair(store: web::Data<Arc<Mutex<Store>>>, data: web::Json<GetKeyPairInput>) -> Result<HttpResponse> {
    let locked_store = match store.lock() {
        Ok(locked) => locked,
        Err(_) => return Ok(HttpResponse::InternalServerError().body("Failed to lock store")),
    };

    let response = match locked_store.get_keypair_mpc_1(&data.user_id).await {
        Ok(kps) => kps,
        Err(e) => {
            eprintln!("Failed to retrieve keypairs: {}", e);
            return Ok(HttpResponse::InternalServerError().body("Failed to retrieve keypairs"));
        }
    };
    let keypair = match convert::keypair_from_base64_strings(&response.secret_key, &response.pub_key) {
        Ok(kp) => kp.0,
        Err(e) => {
            eprintln!("Failed to convert keypair: {}", e);
            return Ok(HttpResponse::InternalServerError().body("Failed to convert keypair"));
        }
    };

    let keypair_bytes = keypair.to_bytes();
    let keypair_base64 = base64::engine::general_purpose::STANDARD.encode(keypair_bytes);

    Ok(HttpResponse::Ok().json(GetKeyPairOutput { keypair_base64 }))
}