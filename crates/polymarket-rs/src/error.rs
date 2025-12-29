// AIDEV-NOTE: Core API errors - NO Tauri dependencies
// Tauri app wraps these in its own serializable AppError

use thiserror::Error;

/// Errors from Polymarket API operations
#[derive(Error, Debug)]
pub enum ApiError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON parsing failed: {0}")]
    Json(#[from] serde_json::Error),

    #[error("WebSocket error: {0}")]
    WebSocket(String),

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("Signing error: {0}")]
    Signing(String),

    #[error("Market not found: {0}")]
    MarketNotFound(String),

    #[error("API error: {0}")]
    Api(String),
}

/// Result type alias for API operations
pub type ApiResult<T> = Result<T, ApiError>;
