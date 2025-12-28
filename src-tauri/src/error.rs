use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON parsing failed: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Market not found: {0}")]
    MarketNotFound(String),

    #[error("API error: {0}")]
    Api(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

// AIDEV-NOTE: Tauri requires errors to be serializable
impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
