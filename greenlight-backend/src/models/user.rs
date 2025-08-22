use sqlx::PgPool;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::error::Result;

#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct User {
    pub id: Uuid,
    pub public_key: String,
    pub password_hash: String,
    pub encrypted_seed: Option<String>,
    pub encrypted_device_creds: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub public_key: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub public_key: String,
    pub password: String,
}

pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_user(
        &self,
        public_key: &str,
        password_hash: &str,
        encrypted_seed: &str,
    ) -> Result<User> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        
        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (id, public_key, password_hash, encrypted_seed, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $5)
            RETURNING id, public_key, password_hash, encrypted_seed, encrypted_device_creds, created_at, updated_at
            "#
        )
        .bind(id)
        .bind(public_key)
        .bind(password_hash)
        .bind(encrypted_seed)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn find_by_public_key(&self, public_key: &str) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            "SELECT id, public_key, password_hash, encrypted_seed, encrypted_device_creds, created_at, updated_at FROM users WHERE public_key = $1"
        )
        .bind(public_key)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            "SELECT id, public_key, password_hash, encrypted_seed, encrypted_device_creds, created_at, updated_at FROM users WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn update_device_credentials(&self, user_id: Uuid, encrypted_creds: &str) -> Result<()> {
        sqlx::query(
            "UPDATE users SET encrypted_device_creds = $1, updated_at = $2 WHERE id = $3"
        )
        .bind(encrypted_creds)
        .bind(Utc::now())
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn public_key_exists(&self, public_key: &str) -> Result<bool> {
        let result = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM users WHERE public_key = $1)"
        )
        .bind(public_key)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }
}
