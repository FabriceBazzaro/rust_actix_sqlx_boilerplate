use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use sqlx::{Postgres, Pool, FromRow, Error};

use crate::shared::tools::generate_string_number;

#[derive(Debug, Deserialize, FromRow, Serialize, Clone)]
pub struct Code {
    pub id: uuid::Uuid,
    pub code: String,
    pub tries: i16,
    pub emitted_at: DateTime<Utc>,
}

impl Code {
    pub async fn create_code(id: uuid::Uuid, db: &Pool<Postgres>) -> Result<Code, Error> {
        sqlx::query_as!(
            Code,
            "INSERT INTO codes (id, code) VALUES ($1, $2) ON CONFLICT (id) DO UPDATE SET code = EXCLUDED.code, tries = 0, emitted_at = DEFAULT RETURNING *",
            id,
            &generate_string_number(6),
        )
            .fetch_one(db)
            .await
    }

    pub async fn get_code_from_id(id: uuid::Uuid, db: &Pool<Postgres>) -> Option<Code> {
        sqlx::query_as!(Code, "SELECT * FROM codes WHERE id = $1", id)
            .fetch_optional(db)
            .await
            .unwrap()
    }

    pub async fn add_try(id: uuid::Uuid, db: &Pool<Postgres>) -> i16 {
        sqlx::query!("UPDATE codes SET tries = tries + 1 WHERE id = $1 RETURNING tries", id)
            .fetch_one(db)
            .await
            .map(|res| res.tries)
            .unwrap()
    }
}