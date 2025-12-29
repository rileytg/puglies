// AIDEV-NOTE: Tauri-specific error type that wraps polymarket_rs::ApiError
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

    #[error("Auth error: {0}")]
    Auth(String),

    #[error("Database error: {0}")]
    Database(String),

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

// Convert from polymarket-rs ApiError to AppError
impl From<polymarket_rs::ApiError> for AppError {
    fn from(e: polymarket_rs::ApiError) -> Self {
        use polymarket_rs::ApiError;
        match e {
            ApiError::Http(e) => AppError::Http(e),
            ApiError::Json(e) => AppError::Json(e),
            ApiError::MarketNotFound(id) => AppError::MarketNotFound(id),
            ApiError::Auth(msg) => AppError::Auth(msg),
            ApiError::Signing(msg) => AppError::Auth(msg),
            ApiError::WebSocket(msg) => AppError::Api(msg),
            ApiError::Api(msg) => AppError::Api(msg),
        }
    }
}
