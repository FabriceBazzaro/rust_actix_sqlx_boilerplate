use serde::{Deserialize, Serialize};
use chrono::{Duration, Utc};
use jsonwebtoken::{EncodingKey, Header};
use actix_web::cookie::Cookie;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JwtToken {
    pub iat: usize,  // Auto validated - Optional. Issued at (as UTC timestamp)
    pub exp: usize,  // Required (validate_exp defaults to true in validation). Expiration time (as UTC timestamp)

    pub id: uuid::Uuid,
    pub user_id: uuid::Uuid,
}

impl JwtToken {
    fn encode(&self, secret: &[u8]) -> String {
        jsonwebtoken::encode(
            &Header::default(),
            &self,
            &EncodingKey::from_secret(secret),
        )
        .unwrap()
    }

    pub fn generate_access_token(user_id: uuid::Uuid) -> Self {
        let now = Utc::now();
        JwtToken {
            exp: (now + Duration::minutes(60)).timestamp() as usize,
            iat: now.timestamp() as usize,

            id: uuid::Uuid::new_v4(),
            user_id
        }
    }

    pub fn generate_refresh_token(user_id: uuid::Uuid) -> Self {
        let now = Utc::now();
        JwtToken {
            exp: (now + Duration::days(7)).timestamp() as usize,
            iat: now.timestamp() as usize,

            id: uuid::Uuid::new_v4(),
            user_id
        }
    }

    pub fn generate_cookie(&self, secret: &[u8], name: String) -> Cookie {
        Cookie::build(name, self.encode(secret))
                .path("/")
                .secure(true)
                .http_only(true)
                .finish()
    }

    pub fn rebuild_cookie_from_value(name: String, value: String) -> Cookie<'static> {
        Cookie::build(name, value)
                .path("/")
                .secure(true)
                .http_only(true)
                .finish()
    }

}