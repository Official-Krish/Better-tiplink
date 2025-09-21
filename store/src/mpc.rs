use chrono::Utc;
use crate::Store;
use sqlx::Row;

#[derive(Debug)]
pub enum MpcServerError {
    UserExists,
    InvalidInput(String),
    DatabaseError(String),
}

impl std::fmt::Display for MpcServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MpcServerError::UserExists => write!(f, "User already exists"),
            MpcServerError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            MpcServerError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
        }
    }
}

#[derive(Debug, Clone)]
pub struct StoredKeypair {
    pub public_key: String,
}

#[derive(Debug, Clone)]
pub struct GetKeyPairOutput {
    pub pub_key: String,
    pub secret_key: String,
}

impl Store {
    pub async fn store_keypair_mpc_1(&self, public_key: &str, private_key: &str, user_id: &str) -> Result<StoredKeypair, MpcServerError> {
        // Store the key pair in the MPC server databases
        let created_at = Utc::now();
        
        let existing_user = sqlx::query(
            "SELECT id FROM keyshares WHERE user_id = $1"
        )
        .bind(user_id)
        .fetch_optional(&self.mpc_server_1)
        .await
        .map_err(|e| MpcServerError::DatabaseError(e.to_string()))?;

        if existing_user.is_some() {
            return Err(MpcServerError::UserExists);
        }

        // Store in MPC Server 1
        sqlx::query(
            "INSERT INTO keyshares (user_id, public_key, secret_key, created_at) VALUES ($1, $2, $3, $4)",
        )
        .bind(user_id)
        .bind(public_key)
        .bind(private_key)
        .bind(created_at)
        .execute(&self.mpc_server_1)
        .await
        .map_err(|e| MpcServerError::DatabaseError(e.to_string()))?;

        Ok(StoredKeypair {
            public_key: public_key.to_string(),
        })
        
    }

    pub async fn store_keypair_mpc_2(&self, public_key: &str, private_key: &str, user_id: &str) -> Result<StoredKeypair, MpcServerError> {
        // Store the key pair in the MPC server databases
        let created_at = Utc::now();

        // Store in MPC Server 2
        let existing_user = sqlx::query(
            "SELECT id FROM keyshares WHERE user_id = $1"
        )
        .bind(user_id)
        .fetch_optional(&self.mpc_server_2)
        .await
        .map_err(|e| MpcServerError::DatabaseError(e.to_string()))?;

        if existing_user.is_some() {
            return Err(MpcServerError::UserExists);
        }

        // Store in MPC Server 1
        sqlx::query(
            "INSERT INTO keyshares (user_id, public_key, secret_key, created_at) VALUES ($1, $2, $3, $4)",
        )
        .bind(user_id)
        .bind(public_key)
        .bind(private_key)
        .bind(created_at)
        .execute(&self.mpc_server_2)
        .await
        .map_err(|e| MpcServerError::DatabaseError(e.to_string()))?;

        Ok(StoredKeypair {
            public_key: public_key.to_string(),
        })
        
    }

    pub async fn get_keypair_mpc_1(&self, user_id: &str) -> Result<GetKeyPairOutput, MpcServerError> {
        let rows = sqlx::query(
            "SELECT public_key, secret_key FROM keyshares WHERE user_id = $1"
        )
        .bind(user_id)
        .fetch_all(&self.mpc_server_1)
        .await
        .map_err(|e| MpcServerError::DatabaseError(e.to_string()))?;

        if rows.is_empty() {
            return Err(MpcServerError::InvalidInput("User ID not found".to_string()));
        }
        let row = &rows[0];
        let public_key: String = row.try_get("public_key").map_err(|e| MpcServerError::DatabaseError(e.to_string()))?;
        let secret_key: String = row.try_get("secret_key").map_err(|e| MpcServerError::DatabaseError(e.to_string()))?;   
        Ok(GetKeyPairOutput {
            pub_key: public_key,
            secret_key,
        })

    }

    pub async fn get_keypair_mpc_2(&self, user_id: &str) -> Result<GetKeyPairOutput, MpcServerError> {
        let rows = sqlx::query(
            "SELECT public_key, secret_key FROM keyshares WHERE user_id = $1"
        )
        .bind(user_id)
        .fetch_all(&self.mpc_server_2)
        .await
        .map_err(|e| MpcServerError::DatabaseError(e.to_string()))?;

        if rows.is_empty() {
            return Err(MpcServerError::InvalidInput("User ID not found".to_string()));
        }
        let row = &rows[0];
        let public_key: String = row.try_get("public_key").map_err(|e| MpcServerError::DatabaseError(e.to_string()))?;
        let secret_key: String = row.try_get("secret_key").map_err(|e| MpcServerError::DatabaseError(e.to_string()))?;   
        Ok(GetKeyPairOutput {
            pub_key: public_key,
            secret_key,
        })

    }
}
