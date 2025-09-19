pub mod user;
pub mod mpc;

use std::time::Duration;

use sqlx::{postgres::PgPoolOptions, PgPool};

pub struct Store {
    pub backend: PgPool,
    pub mpc_server_1: PgPool,
    pub mpc_server_2: PgPool,
}

impl Store {
    pub async fn new() -> Result<Self, sqlx::Error> {
        let backend_url = dotenvy::var("BACKEND_DATABASE_URL")
            .map_err(|e| sqlx::Error::Io(std::io::Error::new(std::io::ErrorKind::NotFound, e)))?;
        
        let mpc1_url = dotenvy::var("MPC_SERVER_1_DATABASE_URL")
            .map_err(|e| sqlx::Error::Io(std::io::Error::new(std::io::ErrorKind::NotFound, e)))?;
        
        let mpc2_url = dotenvy::var("MPC_SERVER_2_DATABASE_URL")
            .map_err(|e| sqlx::Error::Io(std::io::Error::new(std::io::ErrorKind::NotFound, e)))?;

        let backend = PgPoolOptions::new()
            .max_connections(3) 
            .min_connections(1)  
            .acquire_timeout(Duration::from_secs(30))
            .connect(&backend_url)
            .await
            .map_err(|e| {
                eprintln!("Failed to connect to backend database: {}", e);
                e
            })?;

        let mpc_server_1 = PgPoolOptions::new()
            .max_connections(3)
            .min_connections(1)
            .acquire_timeout(Duration::from_secs(30))
            .connect(&mpc1_url)
            .await
            .map_err(|e| {
                eprintln!("Failed to connect to MPC server 1 database: {}", e);
                e
            })?;

        let mpc_server_2 = PgPoolOptions::new()
            .max_connections(3)
            .min_connections(1)
            .acquire_timeout(Duration::from_secs(30))
            .connect(&mpc2_url)
            .await
            .map_err(|e| {
                eprintln!("Failed to connect to MPC server 2 database: {}", e);
                e
            })?;

        Ok(Self { backend, mpc_server_1, mpc_server_2 })
    }
}