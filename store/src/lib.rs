pub mod user;

use sqlx::PgPool;

pub struct Store {
    pub pool: PgPool,
}

impl Store {
    pub async fn new() -> Result<Self, sqlx::Error> {
            let database_url = dotenvy::var("DATABASE_URL").map_err(|e| sqlx::Error::Configuration(Box::new(e)))?;
            let pool = PgPool::connect(&database_url).await?;
            Ok(Self { pool })
        }
}
