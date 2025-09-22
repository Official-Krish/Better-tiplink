use std::{sync::{Arc, Mutex}, vec};

use actix_web::{web, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use store::Store;
use solana_sdk::{pubkey::Pubkey, transaction::Transaction};

#[derive(Deserialize)]
pub struct QuoteRequest {
    input_mint: String,
    output_mint: String,
    amount: u64,
    slippage: Option<u64>,
}

#[derive(Serialize, Deserialize)]
pub struct QuoteResponse {
    #[serde(rename = "inputMint")]
    pub input_mint: String,
    #[serde(rename = "inAmount")]
    pub in_amount: String,
    #[serde(rename = "outputMint")]
    pub output_mint: String,
    #[serde(rename = "outAmount")]
    pub out_amount: String,
    #[serde(rename = "otherAmountThreshold")]
    pub other_amount_threshold: String,
    #[serde(rename = "swapMode")]
    pub swap_mode: String,
    #[serde(rename = "slippageBps")]
    pub slippage_bps: u64,
    #[serde(rename = "platformFee")]
    pub platform_fee: Option<serde_json::Value>,
    #[serde(rename = "priceImpactPct")]
    pub price_impact_pct: String,
    #[serde(rename = "routePlan")]
    pub route_plan: Vec<RoutePlan>,
    #[serde(rename = "contextSlot")]
    pub context_slot: u64,
    #[serde(rename = "timeTaken")]
    pub time_taken: f64,
}

#[derive(Serialize, Deserialize)]
pub struct RoutePlan {
    #[serde(rename = "swapInfo")]
    pub swap_info: SwapInfo,
    pub percent: u64,
}

#[derive(Serialize, Deserialize)]
pub struct SwapInfo {
    #[serde(rename = "ammKey")]
    pub amm_key: String,
    pub label: String,
    #[serde(rename = "inputMint")]
    pub input_mint: String,
    #[serde(rename = "outputMint")]
    pub output_mint: String,
    #[serde(rename = "inAmount")]
    pub in_amount: String,
    #[serde(rename = "outAmount")]
    pub out_amount: String,
    #[serde(rename = "feeAmount")]
    pub fee_amount: String,
    #[serde(rename = "feeMint")]
    pub fee_mint: String,
}


#[derive(Deserialize)]
pub struct SwapRequest {
    to: String,
    amount: f64,
    user_id: String,
}

#[derive(Serialize)]
pub struct SwapResponse {
    pub signature: Transaction,
}

#[derive(Serialize)]
pub struct BalanceResponse {
    pub lamports: u64,
}

#[derive(Serialize)]
pub struct TokenBalanceResponse {
    pub amount: u64,
}

#[derive(Serialize, Deserialize)]
pub struct GetKeyPairInput {
    pub user_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct AggAndStep1Output {
    pub agg_message1: String,
    pub secret_agg_step_one: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct AggAndStep2Input {
    pub keypair_base64: String,
    pub amount: f64,
    pub to: String,
    pub keys: Vec<String>,
    pub first_messages: Vec<String>,
    pub secret_state: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SignatureAggregationInput {
    pub amount: f64,
    pub to: Pubkey,
    pub keys: Vec<Pubkey>,
    pub signatures: Vec<String>,
}

#[actix_web::post("/quote")]
pub async fn quote(req: web::Json<QuoteRequest>) -> Result<HttpResponse> {
    let input_mint = req.input_mint.clone();
    let output_mint = req.output_mint.clone();
    let amount = req.amount;
    let slippage = req.slippage.unwrap_or(50);
    
    let client = reqwest::Client::new();

    let target_url = format!("https://quote-api.jup.ag/v6/quote?inputMint={}&outputMint={}&amount={}&slippageBps={}",
        input_mint, output_mint, amount, slippage);

    match client.get(target_url)
        .send()
        .await
    {
        Ok(response) => {
            println!("Received response with status: {}", response.status());
            if response.status().is_success() {
                let body = response.text().await.unwrap_or_default();
                match serde_json::from_str::<QuoteResponse>(&body) {
                    Ok(parsed_response) => {
                        return Ok(HttpResponse::Ok().json(parsed_response));
                    }
                    Err(e) => {
                        let error_message = format!("Failed to parse response: {:?}", e);
                        return Ok(HttpResponse::InternalServerError().body(error_message));
                    }
                }
            } else {
                return Ok(HttpResponse::InternalServerError().body("Failed to fetch quote from external API"));
            }
        }
        Err(e) => {
            let error_message = format!("Error generating public key: {:?}", e);
            return Ok(HttpResponse::InternalServerError().body(error_message));
        }
    }
}

#[actix_web::post("/swap")]
pub async fn swap(req: web::Json<SwapRequest>) -> Result<HttpResponse> {
    let token = match crate::auth::create_jwt_for_communication(req.user_id.clone()) {
        Ok(t) => t,
        Err(e) => {
            let error_message = format!("Error creating JWT: {:?}", e);
            return Ok(HttpResponse::InternalServerError().body(error_message));
        }
    };
    let mut keypairs = vec![];

    let client = reqwest::Client::new();
    let target_url = vec![
        "http://localhost:9000/getKeyPair",
        "http://localhost:9001/getKeyPair",
    ];

    for url in target_url {
        match client.post(url)
            .json(&GetKeyPairInput {
                user_id: req.user_id.clone(),
            })
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
                    keypairs.push(response_body);
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

    let mut step1_response = vec![vec![]; keypairs.len()];

    for (i, keypair) in keypairs.iter().enumerate() {
        match client.post("http://localhost:8080/agg-send-step1")
            .json(&keypair)
            .bearer_auth(&token)
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    let response_body = match response.json::<AggAndStep1Output>().await {
                        Ok(body) => body,
                        Err(_) => {
                            let error_message = format!("Failed to parse JSON from agg-send-step1");
                            return Ok(HttpResponse::InternalServerError().body(error_message));
                        }
                    };
                    step1_response[i] = vec![
                        response_body.agg_message1,
                        response_body.secret_agg_step_one,
                    ];
                } else {
                    let error_message = format!("Failed to send data to agg-send-step1: {:?}", response.status());
                    return Ok(HttpResponse::InternalServerError().body(error_message));
                }
            }
            Err(e) => {
                let error_message = format!("Error sending request to agg-send-step1: {:?}", e);
                return Ok(HttpResponse::InternalServerError().body(error_message));
            }
        };
    }

    let mut step2_response = vec![];
    for i in 0..keypairs.len() {
        match client.post("http://localhost:8080/agg-send-step2")
            .json(&AggAndStep2Input{
                keypair_base64: keypairs[i].clone(),
                amount: req.amount,
                to: req.to.clone(),
                keys: keypairs.clone(),
                first_messages: step1_response.iter().map(|r| r[0].clone()).collect(),
                secret_state: step1_response[i][1].clone(),
            })
            .bearer_auth(token.clone())
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    let response_body = match response.text().await {
                        Ok(body) => body,
                        Err(_) => {
                            let error_message = format!("Failed to read response body from agg-send-step2");
                            return Ok(HttpResponse::InternalServerError().body(error_message));
                        }
                    };
                    step2_response.push(response_body);
                } else {
                    let error_message = format!("Failed to send data to agg-send-step2: {:?}", response.status());
                    return Ok(HttpResponse::InternalServerError().body(error_message));
                }
            }
            Err(e) => {
                let error_message = format!("Error sending request to agg-send-step2: {:?}", e);
                return Ok(HttpResponse::InternalServerError().body(error_message));
            }
        };
    }

    match client.post("http://localhost:8080/aggregate-signatures-broadcast")
        .json(&SignatureAggregationInput{
            amount: req.amount as f64,
            to: req.to.parse().unwrap(),
            keys: keypairs.iter().map(|k| k.parse().unwrap()).collect(),
            signatures: step2_response.iter().map(|s| s.parse().unwrap()).collect(),
        })
        .bearer_auth(&token)
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                let response_body = match response.json::<Transaction>().await {
                    Ok(body) => body,
                    Err(_) => {
                        let error_message = format!("Failed to read response body from aggregate-signatures-broadcast");
                        return Ok(HttpResponse::InternalServerError().body(error_message));
                    }
                };
                return Ok(HttpResponse::Ok().json(SwapResponse {
                    signature: response_body,
                }));
            } else {
                let error_message = format!("Failed to send data to aggregate-signatures-broadcast: {:?}", response.status());
                return Ok(HttpResponse::InternalServerError().body(error_message));
            }
        }
        Err(e) => {
            let error_message = format!("Error sending request to aggregate-signatures-broadcast: {:?}", e);
            return Ok(HttpResponse::InternalServerError().body(error_message));
        }
    };
}

#[actix_web::get("/sol-balance/{pubkey}")]
pub async fn sol_balance(path: web::Path<String>, store: web::Data<Arc<Mutex<Store>>>) -> Result<HttpResponse> {
    let pubkey = path.into_inner();
    let locked_store = match store.lock() {
        Ok(locked) => locked,
        Err(_) => return Ok(HttpResponse::InternalServerError().body("Failed to lock store")),
    };
    let balance = match locked_store.get_sol_balance(pubkey.to_string()).await {
        Ok(user) => user,
        Err(err) => return Ok(HttpResponse::BadRequest().body(err.to_string())),
    };

    let response = BalanceResponse {
        lamports: balance,
    };
    
    Ok(HttpResponse::Ok().json(response))
}

#[actix_web::get("/token-balance/{pubkey}/{mint}")]
pub async fn token_balance(path: web::Path<(String, String)>, store: web::Data<Arc<Mutex<Store>>>) -> Result<HttpResponse> {
    let (pubkey, mint) = path.into_inner();
    let locked_store = match store.lock() {
        Ok(locked) => locked,
        Err(_) => return Ok(HttpResponse::InternalServerError().body("Failed to lock store")),
    };
    let balance = match locked_store.get_token_balance(pubkey.to_string(), mint.to_string()).await {
        Ok(user) => user,
        Err(err) => return Ok(HttpResponse::BadRequest().body(err.to_string())),
    };

    let response = TokenBalanceResponse {
        amount: balance,
    };

    Ok(HttpResponse::Ok().json(response))
}   
