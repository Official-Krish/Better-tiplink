use anyhow::Result;
use tokio::sync::mpsc;
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::time::Duration;
use yellowstone_grpc_client::{Client as YellowstoneClient, SubscribeRequest, SubscribeResponse};
use solana_sdk::pubkey::Pubkey;
use solana_account_decoder::UiAccount;
use tracing::{info, warn};

#[derive(Debug)]
struct AccountUpdate {
    pubkey: String,
    owner: String,
    slot: u64,
    data: Vec<u8>, // raw account data
    lamports: Option<u64>,
    is_token_account: bool,
    token_amount: Option<u64>, 
}

struct ParsedTokenAccount {
    amount: u64,
    mint: String,
}

fn parse_spl_token_account(data: &Vec<u8>) -> Result<ParsedTokenAccount, anyhow::Error> {
    use spl_token::state::Account as SplAccount;
    let acc = SplAccount::unpack_from_slice(data)?;
    Ok(ParsedTokenAccount { amount: acc.amount, mint: acc.mint.to_string() })
}

// TODO: implement upsert logic with proper conflict handling
async fn upsert_balance(pool: &PgPool, pubkey: &str, owner_user_id: Option<i64>, token_mint: Option<&str>, amount: i128, slot: u64) -> Result<()> {
    sqlx::query!(
        r#"
        INSERT INTO balances (account_pubkey, amount, updated_at)
        VALUES ($1, $2, $3, $4, $5, now())
        ON CONFLICT (account_pubkey)
        DO UPDATE SET amount = EXCLUDED.amount, slot = EXCLUDED.slot, updated_at = now()
        WHERE balances.slot < EXCLUDED.slot
        "#,
        pubkey,
        owner_user_id,
        token_mint,
        amount,
        slot as i64
    )
    .execute(pool)
    .await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    // 1) DB pool
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL env var");
    let pool = PgPoolOptions::new()
        .max_connections(8)
        .connect(&db_url)
        .await?;

    // 2) create a processing channel to decouple network I/O from DB writes
    let (tx, mut rx) = mpsc::channel::<AccountUpdate>(10_000);

    // 3) Spawn DB writer workers
    let db_pool = pool.clone();
    tokio::spawn(async move {
        while let Some(update) = rx.recv().await {
            let owner_user_id = match sqlx::query_scalar!("SELECT user_id FROM user_accounts WHERE account_pubkey = $1", update.pubkey)
                .fetch_optional(&db_pool)
                .await
                .unwrap_or(None) {
                    Some(uid) => Some(uid as i64),
                    None => None,
                };

            if update.is_token_account {
                if let Some(amount) = update.token_amount {
                    if let Err(e) = upsert_balance(&db_pool, &update.pubkey, owner_user_id, None, amount as i128, update.slot).await {
                        warn!("DB upsert error: {:?}", e);
                    }
                }
            } else if let Some(lamports) = update.lamports {
                if let Err(e) = upsert_balance(&db_pool, &update.pubkey, owner_user_id, None, lamports as i128, update.slot).await {
                    warn!("DB upsert error: {:?}", e);
                }
            }
        }
    });

    let tls_config = ClientTlsConfig::new().with_native_roots();

        if let Ok(mut client) = GeyserGrpcClient::build_from_shared(
            "https://solana-yellowstone-grpc.publicnode.com:443",
        )
        .unwrap()
        .keep_alive_while_idle(true)
        .tls_config(tls_config)
        .unwrap()
        .connect()
        .await
        {
            let mut accounts: HashMap<String, SubscribeRequestFilterAccounts> = HashMap::new();

            let filter = SubscribeRequestFilterAccounts {
                owner: vec![],                                                             // TODO
                account: vec!["3ucNos4NbumPLZNWztqGHNFFgkHeRMBQAVemeeomsUxv".to_string()], // TODO
                ..Default::default()
            };

            accounts.insert("client".to_string(), filter);
            let (_tx, mut stream) = client
                .subscribe_with_request(Some(SubscribeRequest {
                    accounts,
                    ..Default::default()
                }))
                .await
                .expect("Error: unable to make grpc connection request");

            loop {
                let message = stream.next().await.unwrap();
                match msg {
                    Ok(SubscribeResponse::AccountUpdate(account_update_proto)) => {
                        // parse the proto fields
                        // Example fields: pubkey, owner, lamports, data (base64?), slot, is_token
                        let pubkey = account_update_proto.pubkey;
                        let slot = account_update_proto.slot;
                        let owner = account_update_proto.owner;
                        let raw_data = account_update_proto.data; // depending on proto type this might be Vec<u8>

                        // decide whether this is an SPL token account: quick check owner == token program id
                        let is_token_account = owner == "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";

                        let mut token_amount = None;
                        let mut lamports = account_update_proto.lamports; // maybe present

                        if is_token_account {
                            // decode SPL token Account state to extract amount (u64) & mint
                            // Use spl_token::state::Account::unpack or solana_account_decoder
                            if let Ok(parsed) = parse_spl_token_account(&raw_data) {
                                token_amount = Some(parsed.amount);
                                // optionally pass token mint string parsed.mint
                            }
                        }

                        let update = AccountUpdate {
                            pubkey,
                            owner,
                            slot,
                            data: raw_data,
                            lamports,
                            is_token_account,
                            token_amount,
                        };

                        // best-effort send (drop if channel full to avoid blocking network)
                        if let Err(_) = tx.try_send(update) {
                            warn!("db queue full; dropping account update");
                        }
                    }

                    Ok(SubscribeResponse::Ping(p)) => {
                        // reply to ping is handled by client library or you may need to send a ping back
                        client.reply_ping().await.ok();
                    }

                    Ok(SubscribeResponse::SlotNotification(slot_info)) => {
                        // optional: checkpoint last processed slot in DB for recovery
                        let last_slot = slot_info.slot;
                        let _ = sqlx::query!("INSERT INTO checkpoints (name, last_slot) VALUES ('yellowstone', $1) ON CONFLICT (name) DO UPDATE SET last_slot = EXCLUDED.last_slot", last_slot as i64)
                            .execute(&pool).await;
                    }

                    Ok(_) => { /* other messages: txs, blocks, etc */ }

                    Err(e) => {
                        warn!("stream error: {:?}; attempting reconnect", e);
                        // handle reconnect logic (sleep, backoff), or let client auto-reconnect
                        tokio::time::sleep(Duration::from_secs(1)).await;
                    }
                }
            }
        }
}
