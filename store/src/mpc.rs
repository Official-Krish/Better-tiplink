use chrono::Utc;
use crate::Store;

impl Store {
    pub async fn store_keypair_mpc_1(&self, public_key: &str, private_key: &str, user_id: &str) -> Result<(), sqlx::Error> {
        // Store the key pair in the MPC server databases
        let created_at = Utc::now();
        
        // Store in MPC Server 1
        sqlx::query!(
            "INSERT INTO keyshares (user_id, public_key, secret_key, created_at) VALUES ($1, $2, $3, $4)",
            user_id,
            public_key,
            private_key,
            created_at
        )
        .execute(&self.mpc_server_1)
        .await?;
        
        Ok(())
        
    }

    pub async fn store_keypair_mpc_2(&self, public_key: &str, private_key: &str, user_id: &str) -> Result<(), sqlx::Error> {
        // Store the key pair in the MPC server databases
        let created_at = Utc::now();

        // Store in MPC Server 2
        sqlx::query!(
            "INSERT INTO keyshares (user_id, public_key, secret_key, created_at) VALUES ($1, $2, $3, $4)",
            user_id,
            public_key,
            private_key,
            created_at
        )
        .execute(&self.mpc_server_2)
        .await?;
        
        Ok(())
        
    }
}