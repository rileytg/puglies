// AIDEV-NOTE: Authenticated CLOB REST API client for positions, orders, and balances

use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{debug, instrument};

use crate::auth::{ApiCredentials, HmacAuth, PolymarketSigner};
use crate::error::AppError;

const CLOB_API_BASE: &str = "https://clob.polymarket.com";
const DATA_API_BASE: &str = "https://data-api.polymarket.com";

/// Client for the Polymarket CLOB REST API (authenticated)
#[derive(Clone)]
pub struct ClobClient {
    client: Client,
    base_url: String,
    hmac_auth: Option<HmacAuth>,
}

/// User balance information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Balance {
    /// Available USDC balance (in wei, 6 decimals)
    #[serde(default)]
    pub balance: String,
    /// Allowances per contract address (we ignore this for now)
    #[serde(default)]
    pub allowances: std::collections::HashMap<String, String>,
}

/// User position in a market (from Data API)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Position {
    /// Asset/token ID
    #[serde(default)]
    pub asset: String,
    /// Condition ID of the market
    #[serde(default)]
    pub condition_id: String,
    /// Size of position
    #[serde(default)]
    pub size: f64,
    /// Average entry price
    #[serde(default)]
    pub avg_price: f64,
    /// Initial value
    #[serde(default)]
    pub initial_value: f64,
    /// Current value
    #[serde(default)]
    pub current_value: f64,
    /// Cash PnL
    #[serde(default)]
    pub cash_pnl: f64,
    /// Percent PnL
    #[serde(default)]
    pub percent_pnl: f64,
    /// Current price
    #[serde(default)]
    pub cur_price: f64,
    /// Market title
    #[serde(default)]
    pub title: String,
    /// The outcome name
    #[serde(default)]
    pub outcome: String,
    /// Proxy wallet address
    #[serde(default)]
    pub proxy_wallet: String,
}

/// User order
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Order {
    /// Order ID
    pub id: String,
    /// Market condition ID
    #[serde(default)]
    pub market: String,
    /// Asset/token ID
    pub asset: String,
    /// Side: BUY or SELL
    pub side: String,
    /// Original size
    pub original_size: String,
    /// Remaining size
    pub size_matched: String,
    /// Limit price
    pub price: String,
    /// Order status
    pub status: String,
    /// Order type (GTC, FOK, etc.)
    #[serde(default)]
    pub order_type: String,
    /// Created timestamp
    #[serde(default)]
    pub created_at: String,
}

/// API key derivation response
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiKeyResponse {
    pub api_key: String,
    pub secret: String,
    pub passphrase: String,
}

impl ClobClient {
    /// Create a new unauthenticated client
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            base_url: CLOB_API_BASE.to_string(),
            hmac_auth: None,
        }
    }

    /// Create an authenticated client with credentials
    pub fn with_credentials(credentials: &ApiCredentials) -> Self {
        Self {
            client: Client::new(),
            base_url: CLOB_API_BASE.to_string(),
            hmac_auth: Some(HmacAuth::new(credentials)),
        }
    }

    /// Set credentials for authentication
    pub fn set_credentials(&mut self, credentials: &ApiCredentials) {
        self.hmac_auth = Some(HmacAuth::new(credentials));
    }

    /// Derive API keys from wallet signature using L1 headers
    #[instrument(skip(self, signer))]
    pub async fn derive_api_key(&self, signer: &PolymarketSigner) -> Result<ApiCredentials, AppError> {
        // Generate L1 authentication headers
        let l1_headers = signer.create_l1_headers(0).await?;

        let url = format!("{}/auth/derive-api-key", self.base_url);
        debug!("Deriving API key at: {} with address {}", url, l1_headers.address);

        // Send GET request with L1 headers
        let response = l1_headers.apply_to_request(self.client.get(&url))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(AppError::Api(format!(
                "API key derivation failed ({}): {}",
                status, text
            )));
        }

        let api_response: ApiKeyResponse = response.json().await?;

        Ok(ApiCredentials {
            api_key: api_response.api_key,
            api_secret: api_response.secret,
            api_passphrase: api_response.passphrase,
            address: signer.address_string(),
        })
    }

    /// Get authenticated user's balance and allowance
    #[instrument(skip(self))]
    pub async fn get_balance(&self) -> Result<Balance, AppError> {
        let hmac = self.hmac_auth.as_ref()
            .ok_or_else(|| AppError::Internal("Not authenticated".to_string()))?;

        // AIDEV-NOTE: Correct endpoint is /balance-allowance, not /balance
        // AIDEV-NOTE: asset_type=COLLATERAL for USDC balance
        // AIDEV-NOTE: signature_type=2 for Polymarket proxy wallet balance (0=EOA, 1=?, 2=proxy)
        // AIDEV-NOTE: HMAC signature uses path only, not query params
        let path = "/balance-allowance";
        let url = format!("{}{}?asset_type=COLLATERAL&signature_type=2", self.base_url, path);
        let headers = hmac.generate_headers("GET", path, None)?;

        debug!("Fetching balance from: {}", url);

        let response = headers.apply_to_request(self.client.get(&url))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(AppError::Api(format!("Balance request failed ({}): {}", status, text)));
        }

        // Debug: Log raw response
        let text = response.text().await?;
        debug!("Balance raw response: {}", text);

        let balance: Balance = serde_json::from_str(&text)
            .map_err(|e| AppError::Internal(format!("Failed to parse balance: {}", e)))?;
        Ok(balance)
    }

    /// Get user's positions from Data API (uses address, not auth)
    #[instrument(skip(self))]
    pub async fn get_positions(&self, address: &str) -> Result<Vec<Position>, AppError> {
        let url = format!("{}/positions?user={}", DATA_API_BASE, address);

        debug!("Fetching positions from: {}", url);

        let response = self.client.get(&url)
            .send()
            .await?;

        let status = response.status();
        debug!("Positions response status: {}", status);

        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(AppError::Api(format!("Positions request failed ({}): {}", status, text)));
        }

        let text = response.text().await?;
        debug!("Positions response body length: {} chars", text.len());

        // Try to parse, with detailed error on failure
        let positions: Vec<Position> = serde_json::from_str(&text).map_err(|e| {
            debug!("Failed to parse positions: {}. First 500 chars: {}", e, &text[..text.len().min(500)]);
            AppError::Internal(format!("Failed to parse positions: {}", e))
        })?;

        debug!("Parsed {} positions", positions.len());
        Ok(positions)
    }

    /// Get authenticated user's open orders
    #[instrument(skip(self))]
    pub async fn get_orders(&self) -> Result<Vec<Order>, AppError> {
        let hmac = self.hmac_auth.as_ref()
            .ok_or_else(|| AppError::Internal("Not authenticated".to_string()))?;

        let path = "/orders";
        let url = format!("{}{}", self.base_url, path);
        let headers = hmac.generate_headers("GET", path, None)?;

        debug!("Fetching orders from: {}", url);

        let response = headers.apply_to_request(self.client.get(&url))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(AppError::Api(format!("Orders request failed ({}): {}", status, text)));
        }

        let orders: Vec<Order> = response.json().await?;
        debug!("Fetched {} orders", orders.len());
        Ok(orders)
    }
}

impl Default for ClobClient {
    fn default() -> Self {
        Self::new()
    }
}
