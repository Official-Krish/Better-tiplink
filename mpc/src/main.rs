use std::vec;

use actix_web::{web::{self, post}, App, Error, HttpResponse, HttpServer};
use serde::{Deserialize, Serialize};

pub mod error;
pub mod serialization;
pub mod tss;

use crate::{error::Error as MpcError, tss::key_agg};
use solana_sdk::signature::{Keypair, Signature};
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

// pub fn create_unsigned_transaction(amount: f64, to: &Pubkey, memo: Option<String>, payer: &Pubkey) -> Transaction {
//     let amount = native_token::sol_to_lamports(amount);
//     let transfer_ins = system_instruction::transfer(payer, to, amount);
//     let msg = match memo {
//         None => Message::new(&[transfer_ins], Some(payer)),
//         Some(memo) => {
//             let memo_ins = Instruction { program_id: spl_memo::id(), accounts: Vec::new(), data: memo.into_bytes() };
//             Message::new(&[transfer_ins, memo_ins], Some(payer))
//         }
//     };
//     Transaction::new_unsigned(msg)
// }

#[derive(Serialize, Deserialize)]
pub struct GeneratePubKeyInput {
    pub user_id: String,
}

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    HttpServer::new(|| {
        App::new()
        .route("/generate", post().to(generate))
        .route("/send-single", post().to(send_single))
        .route("/aggregate-keys", post().to(aggregate_keys))
        .route("/agg-send-step1", post().to(agg_send_step1))
        .route("/agg-send-step2", post().to(agg_send_step2))
        .route(
            "/aggregate-signatures-broadcast",
            post().to(aggregate_signatures_broadcast),
        )
    })
    
        .bind("127.0.0.1:8080")?
        .run()
        .await
}

async fn generate(data: web::Json<GeneratePubKeyInput>) -> Result<HttpResponse, Error> {
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

    Ok(HttpResponse::Ok().json(final_pub_key.to_string()))
}

async fn send_single() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().body("Hello, world!"))
}

async fn aggregate_keys() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().body("Hello, world!"))
}

async fn agg_send_step1() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().body("Hello, world!"))
}

async fn agg_send_step2() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().body("Hello, world!"))
}

async fn aggregate_signatures_broadcast() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().body("Hello, world!"))
}