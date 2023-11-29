use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use sqlx::{Postgres, Pool, FromRow, Row, Error};

#[derive(Debug, Deserialize, FromRow, Serialize, Clone)]
pub struct User {
    pub id: uuid::Uuid,
    pub email: String,
    pub language_id: String,
    pub role: String,
    pub verified: bool,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl User {
    pub async fn is_user_exist(email: String, db: &Pool<Postgres>) -> bool {
        sqlx::query("SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)")
            .bind(email)
            .fetch_one(db)
            .await
            .unwrap()
            .get(0)
    }

    pub async fn get_user_from_email(email: String, db: &Pool<Postgres>) -> Option<User> {
        sqlx::query_as!(User, "SELECT * FROM users WHERE email = $1", email)
            .fetch_optional(db)
            .await
            .unwrap()
    }

    pub async fn create_user(email: String, language: String, db: &Pool<Postgres>) -> Result<User, Error> {
        sqlx::query_as!(
            User,
            "INSERT INTO users (email, language_id, role, verified) VALUES ($1, $2, $3, $4) RETURNING *",
            email.to_lowercase(),
            language.to_lowercase(),
            "user".to_string(),
            false
        )
            .fetch_one(db)
            .await
    }

    pub async fn set_email_verified(id: uuid::Uuid, db: &Pool<Postgres>) -> Result<User, Error> {
        sqlx::query_as!(
            User,
            "UPDATE users SET verified = true WHERE id = $1 RETURNING *",
            id
        )
            .fetch_one(db)
            .await
    }
}