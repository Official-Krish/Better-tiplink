use crate::Store;
use chrono::{Utc};

#[derive(Debug, Clone)]
pub struct User {
    pub id: String,
    pub email: String,
    pub created_at: String,
    pub public_key: String,
}

#[derive(Debug)]
pub struct CreateUserRequest {
    pub user_id: String,
    pub email: String,
    pub password: String,
    pub pub_key: String
}

#[derive(Debug)]
pub enum UserError {
    UserExists,
    InvalidInput(String),
    DatabaseError(String),
}

impl std::fmt::Display for UserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserError::UserExists => write!(f, "User already exists"),
            UserError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            UserError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
        }
    }
}

impl std::error::Error for UserError {}

impl Store {
    pub async fn create_user(&self, request: CreateUserRequest) -> Result<User, UserError> {
        // Validate email format
        if !request.email.contains('@') {
            return Err(UserError::InvalidInput("Invalid email format".to_string()));
        }

        // Validate password length
        if request.password.len() < 6 {
            return Err(UserError::InvalidInput("Password must be at least 6 characters".to_string()));
        }

        // Check if user already exists
        let existing_user = sqlx::query!(
            "SELECT id FROM users WHERE email = $1",
            request.email
        )
        .fetch_optional(&self.backend)
        .await
        .map_err(|e| UserError::DatabaseError(e.to_string()))?;

        if existing_user.is_some() {
            return Err(UserError::UserExists);
        }

        // Hash the password
        let password_hash = bcrypt::hash(&request.password, bcrypt::DEFAULT_COST)
            .map_err(|e| UserError::DatabaseError(format!("Password hashing failed: {}", e)))?;

        let user_id = request.user_id.clone();
        let created_at = Utc::now();
        let pub_key = request.pub_key.clone();

        // Insert user into database
        sqlx::query!(
            "INSERT INTO users (id, email, password, created_at, updated_at, public_key) VALUES ($1, $2, $3, $4, $5, $6)",
            user_id,
            request.email,
            password_hash,
            created_at,
            created_at,
            pub_key
        )
        .execute(&self.backend)
        .await
        .map_err(|e| UserError::DatabaseError(e.to_string()))?;

        // Return the created user
        let user = User {
            id: user_id,
            email: request.email,
            created_at: created_at.to_rfc3339(),
            public_key: pub_key.to_string(),
        };

        Ok(user)
    }

    pub async fn sign_in(&self, email: String, password: String) -> Result<User, UserError> {
        // Fetch user by email
        let record = sqlx::query!(
            "SELECT id, email, password, created_at, public_key FROM users WHERE email = $1",
            email
        )
        .fetch_optional(&self.backend)
        .await
        .map_err(|e| UserError::DatabaseError(e.to_string()))?;

        let record = match record {
            Some(rec) => rec,
            None => return Err(UserError::InvalidInput("Invalid email or password".to_string())),
        };

        // Verify password
        let is_valid = bcrypt::verify(&password, &record.password)
            .map_err(|e| UserError::DatabaseError(format!("Password verification failed: {}", e)))?;

        if !is_valid {
            return Err(UserError::InvalidInput("Invalid email or password".to_string()));
        }

        // Return the user
        let user = User {
            id: record.id,
            email: record.email,
            created_at: record.created_at.to_rfc3339(),
            public_key: record.public_key,
        };

        Ok(user)
    }

    pub async fn get_user_by_id(&self, user_id: String) -> Result<User, UserError> {
        let record = sqlx::query!(
            "SELECT id, email, created_at, public_key FROM users WHERE id = $1",
            user_id
        )
        .fetch_optional(&self.backend)
        .await
        .map_err(|e| UserError::DatabaseError(e.to_string()))?;

        let record = match record {
            Some(rec) => rec,
            None => return Err(UserError::InvalidInput("User not found".to_string())),
        };

        let user = User {
            id: record.id,
            email: record.email,
            created_at: record.created_at.to_rfc3339(),
            public_key: record.public_key, 
        };

        Ok(user)
    }
}
