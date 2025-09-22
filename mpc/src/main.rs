use actix_web::{web::{self, post}, App, Error, HttpResponse, HttpServer};
use serde::{Deserialize, Serialize};

pub mod error;
pub mod serialization;
pub mod tss;
pub mod auth;
pub mod middleware;

use crate::{serialization::PartialSignature, tss::{key_agg, sign_and_broadcast, step_one, step_two}};
use solana_sdk::{instruction::Instruction, message::Message, native_token, signature::Keypair, system_instruction, transaction::Transaction};
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use crate::serialization::{AggMessage1, SecretAggStepOne};
use base64::engine::Engine;

#[derive(Serialize, Deserialize)]
pub struct GeneratePubKeyInput {
    pub user_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct AggAndStep1Input {
    pub keypair_base64: String,
}

#[derive(Serialize, Deserialize)]
pub struct AggAndStep1Output {
    agg_message1: AggMessage1,
    secret_agg_step_one: SecretAggStepOne,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct AggAndStep2Input {
    pub keypair_base64: String,
    pub amount: f64,
    pub to: String,
    pub keys: Vec<String>,
    pub first_messages: Vec<AggMessage1>,
    pub secret_state: SecretAggStepOne,
}

#[derive(Serialize, Deserialize)]
pub struct AggAndStep2Output {
    pub partial_signature: PartialSignature,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SignatureAggregationInput {
    pub amount: f64,
    pub to: Pubkey,
    pub keys: Vec<Pubkey>,
    pub signatures: Vec<PartialSignature>,
}

#[derive(Serialize, Deserialize)]
struct BroadcastResponse {
    signature: Transaction,
}

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    HttpServer::new(|| {
        App::new()
        .route("/generate", post().to(generate).wrap(middleware::AuthMiddleware))
        .route("/agg-send-step1", post().to(agg_send_step1).wrap(middleware::AuthMiddleware))
        .route("/agg-send-step2", post().to(agg_send_step2).wrap(middleware::AuthMiddleware))
        .route(
            "/aggregate-signatures-broadcast",
            post().to(aggregate_signatures_broadcast).wrap(middleware::AuthMiddleware),
        )
    })
    
        .bind("127.0.0.1:8080")?
        .run()
        .await
}

async fn generate(data: web::Json<GeneratePubKeyInput>) -> Result<HttpResponse, Error> {
    let token = match auth::create_jwt_for_communication(data.user_id.clone()) {
        Ok(t) => t,
        Err(e) => {
            let error_message = format!("Error creating JWT: {:?}", e);
            return Ok(HttpResponse::InternalServerError().body(error_message));
        }
    };
    let mut pub_keys = vec![];
    let client = reqwest::Client::new();
    let data_to_send = GeneratePubKeyInput {
        user_id: data.user_id.clone(),
    };

    let target_url = vec![
        "http://localhost:9000/generatePubKey",
        "http://localhost:9001/generatePubKey",
    ];

    for url in target_url {
        match client.post(url)
            .json(&data_to_send)
            .bearer_auth(&token)
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    let response_body = match response.text().await {
                        Ok(body) => body,
                        Err(_) => {
                            let error_message = format!("Failed to read response body from {}", url);
                            return Ok(HttpResponse::InternalServerError().body(error_message));
                        }
                    };
                    pub_keys.push(response_body);
                } else {
                    let error_message = format!("Failed to send data to {}: {:?}", url, response.status());
                    return Ok(HttpResponse::InternalServerError().body(error_message));
                }
            }
            Err(e) => {
                let error_message = format!("Error sending request to {}: {:?}", url, e);
                return Ok(HttpResponse::InternalServerError().body(error_message));
            }
        }
    }

    let pub_keys: Vec<Pubkey> = pub_keys.into_iter().map(|key| Pubkey::from_str(&key).unwrap()).collect();
    let final_pub_key = match key_agg(pub_keys, None) {
        Ok(key) => key,
        Err(e) => {
            return Ok(HttpResponse::InternalServerError().body(format!("Error aggregating keys: {:?}", e)));
        }
    };

    Ok(HttpResponse::Ok().json(&final_pub_key))
}

async fn agg_send_step1(data: web::Json<AggAndStep1Input>) -> Result<HttpResponse, Error> {
    let keypair_bytes = match base64::engine::general_purpose::STANDARD.decode(&data.keypair_base64) {
        Ok(bytes) => bytes,
        Err(_) => return Ok(HttpResponse::BadRequest().body("Invalid base64 for keypair bytes")),
    };
    let keypair = match Keypair::from_bytes(keypair_bytes.as_slice()) {
        Ok(kp) => kp,
        Err(_) => return Ok(HttpResponse::BadRequest().body("Invalid keypair bytes")),
    };
    let response = step_one(keypair);
    Ok(HttpResponse::Ok().json(AggAndStep1Output {
        agg_message1: response.0,
        secret_agg_step_one: response.1,
    }))
}

async fn agg_send_step2(data: web::Json<AggAndStep2Input>) -> Result<HttpResponse, Error> {
    let keypair_bytes = match base64::engine::general_purpose::STANDARD.decode(&data.keypair_base64) {
        Ok(bytes) => bytes,
        Err(_) => return Ok(HttpResponse::BadRequest().body("Invalid base64 for keypair bytes")),
    };
    let keypair = match Keypair::from_bytes(keypair_bytes.as_slice()) {
        Ok(kp) => kp,
        Err(_) => return Ok(HttpResponse::BadRequest().body("Invalid keypair bytes")),
    };
    let to = match Pubkey::from_str(&data.to) {
        Ok(pk) => pk,
        Err(_) => return Ok(HttpResponse::BadRequest().body("Invalid recipient public key")),
    };
    let keys: Vec<Pubkey> = match data.keys.iter().map(|k| Pubkey::from_str(k)).collect() {
        Ok(ks) => ks,
        Err(_) => return Ok(HttpResponse::BadRequest().body("Invalid public key in keys array")),
    };
    let recent_block_hash = solana_sdk::hash::Hash::new_unique();
    let first_messages = data.first_messages.clone();
    let secret_state = data.secret_state.clone();
    let response = step_two(keypair, data.amount, to, None, recent_block_hash, keys, first_messages, secret_state);
    match response {
        Ok(sig) => Ok(HttpResponse::Ok().json(AggAndStep2Output { partial_signature: sig })),
        Err(e) => Ok(HttpResponse::InternalServerError().body(format!("Error in step two: {:?}", e))),
    }
}

async fn aggregate_signatures_broadcast(data: web::Json<SignatureAggregationInput>) -> Result<HttpResponse, Error> {
    let recent_block_hash = solana_sdk::hash::Hash::new_unique();
    let keys = data.keys.clone();
    let signatures = data.signatures.clone();
    let response = sign_and_broadcast(data.amount, data.to, None, recent_block_hash, keys, signatures);

    match response {
        Ok(sig) => {
            let broadcast_response = BroadcastResponse {
                signature: sig,
            };
            Ok(HttpResponse::Ok().json(broadcast_response))
        }
        Err(e) => Ok(HttpResponse::InternalServerError().body(format!("Error aggregating signatures and broadcasting: {:?}", e))),
    }
}


pub fn create_unsigned_transaction(amount: f64, to: &Pubkey, memo: Option<String>, payer: &Pubkey) -> Transaction {
    let amount = native_token::sol_to_lamports(amount);
    let transfer_ins = system_instruction::transfer(payer, to, amount);
    let msg = match memo {
        None => Message::new(&[transfer_ins], Some(payer)),
        Some(memo) => {
            let memo_ins = Instruction { program_id: spl_memo::id(), accounts: Vec::new(), data: memo.into_bytes() };
            Message::new(&[transfer_ins, memo_ins], Some(payer))
        }
    };
    Transaction::new_unsigned(msg)
}