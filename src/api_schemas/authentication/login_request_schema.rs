use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct LoginRequestSchema {
    pub email: String,
}
