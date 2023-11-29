use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ConfirmCodeRequestSchema {
    pub email: String,
    pub code: String,
}
