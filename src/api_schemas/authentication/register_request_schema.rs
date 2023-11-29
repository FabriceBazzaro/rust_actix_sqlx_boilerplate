use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct RegisterRequestSchema {
    pub email: String,
    pub language: String,
}
