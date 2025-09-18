use actix_web::{App, HttpResponse, HttpServer};
use solana_sdk::{signature::Keypair, signer::Signer};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(move || {
        App::new()
            .service(generate)  
    })
    .bind("127.0.0.1:9000")?
    .run()
    .await
}

#[actix_web::post("/generate")]
pub async fn generate() -> Result<HttpResponse, actix_web::Error> {
    let keypair = Keypair::new();
    //TODO: Store the keypair in db
    Ok(HttpResponse::Ok().body(keypair.pubkey().to_string()))
}