use actix_web::{web, HttpResponse, Result};
use serde::{Deserialize, Serialize};

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
}

#[derive(Serialize)]
pub struct SwapResponse {
}

#[derive(Serialize)]
pub struct BalanceResponse {
}

#[derive(Serialize)]
pub struct TokenBalanceResponse {
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
    
    let response = SwapResponse {};
    
    Ok(HttpResponse::Ok().json(response))
}

#[actix_web::get("/sol-balance/{pubkey}")]
pub async fn sol_balance() -> Result<HttpResponse> {
    
    let response = BalanceResponse {
    };
    
    Ok(HttpResponse::Ok().json(response))
}

#[actix_web::get("/token-balance/{pubkey}/{mint}")]
pub async fn token_balance() -> Result<HttpResponse> {    
    
    let response = TokenBalanceResponse {
        
    };
    
    Ok(HttpResponse::Ok().json(response))
}
