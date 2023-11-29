use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use sqlx::{Postgres, Pool, FromRow, Error};
use uuid::Uuid;

#[derive(Debug, Deserialize, FromRow, Serialize, Clone)]
pub struct Token {
    pub user_id: Uuid,
    pub token_id: Uuid,

    pub expiration: DateTime<Utc>,
    pub is_valid: bool
}

impl Token {
    pub async fn declare_new(user_id: Uuid, token_id: Uuid, expiration: DateTime<Utc>, db: &Pool<Postgres>) -> Result<Token, Error> {
        sqlx::query_as!(
            Token,
            "INSERT INTO tokens (user_id, token_id, expiration) VALUES ($1, $2, $3) RETURNING *",
            user_id,
            token_id,
            expiration,
        )
            .fetch_one(db)
            .await
    }

    pub async fn is_valid(user_id: Uuid, token_id: Uuid, db: &Pool<Postgres>) -> Result<bool, Error> {
        sqlx::query!("SELECT is_valid FROM tokens WHERE user_id = $1 AND token_id = $2",
            user_id,
            token_id)
            .fetch_one(db)
            .await
            .map(|row| row.is_valid)
    }


    pub async fn remove_expired(user_id: Uuid, db: &Pool<Postgres>) -> Result<(), Error> {
        sqlx::query!("DELETE FROM tokens WHERE user_id = $1 AND expiration < now()",
            user_id)
            .execute(db)
            .await
            .map(|_| ())
    }

    pub async fn invalidate(user_id: Uuid, token_id: Uuid, db: &Pool<Postgres>) -> Result<(), Error> {
        sqlx::query!("UPDATE tokens SET is_valid = false WHERE user_id = $1 AND token_id = $2",
            user_id,
            token_id)
            .execute(db)
            .await
            .map(|_| ())
    }

}