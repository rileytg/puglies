// AIDEV-NOTE: Authenticated CLOB REST API client for positions, orders, and balances

use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, instrument};

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
    #[serde(default, alias = "asset_id")]
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

/// AIDEV-NOTE: Orders response is wrapped: {"data": [], "next_cursor": ..., "limit": ..., "count": ...}
#[derive(Debug, Clone, Deserialize)]
pub struct OrdersResponse {
    pub data: Vec<Order>,
    #[serde(default)]
    pub next_cursor: Option<String>,
    #[serde(default)]
    pub limit: Option<u32>,
    #[serde(default)]
    pub count: Option<u32>,
}

/// API key derivation response
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiKeyResponse {
    pub api_key: String,
    pub secret: String,
    pub passphrase: String,
}

/// AIDEV-NOTE: Price history response from /prices-history
/// Returns array of {t: timestamp, p: price} points
#[derive(Debug, Clone, Deserialize)]
pub struct PriceHistoryResponse {
    pub history: Vec<PricePoint>,
}

/// Single price point in history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricePoint {
    /// Unix timestamp (seconds)
    pub t: i64,
    /// Price (0.0 - 1.0)
    pub p: f64,
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
    /// AIDEV-NOTE: Endpoint is /data/orders, NOT /orders (405 error)
    #[instrument(skip(self))]
    pub async fn get_orders(&self) -> Result<Vec<Order>, AppError> {
        let hmac = self.hmac_auth.as_ref()
            .ok_or_else(|| AppError::Internal("Not authenticated".to_string()))?;

        let path = "/data/orders";
        let url = format!("{}{}", self.base_url, path);
        let headers = hmac.generate_headers("GET", path, None)?;

        debug!("Fetching orders from: {}", url);

        let response = headers.apply_to_request(self.client.get(&url))
            .send()
            .await?;

        let status = response.status();
        let text = response.text().await.unwrap_or_default();

        debug!("Orders response status: {}, body length: {}", status, text.len());

        if !status.is_success() {
            return Err(AppError::Api(format!("Orders request failed ({}): {}", status, text)));
        }

        // AIDEV-NOTE: Log first 500 chars of response for debugging parse errors
        let preview = if text.len() > 500 { &text[..500] } else { &text };
        debug!("Orders response preview: {}", preview);

        // AIDEV-NOTE: Response is wrapped in {"data": [...], ...}
        let response: OrdersResponse = serde_json::from_str(&text).map_err(|e| {
            error!("Failed to parse orders: {}. Response: {}", e, preview);
            AppError::Internal(format!("Failed to parse orders: {}", e))
        })?;
        debug!("Fetched {} orders", response.data.len());
        Ok(response.data)
    }

    // ========== Order Placement & Cancellation ==========

    /// Place a new order
    /// AIDEV-NOTE: Requires EIP-712 signed order + L2 HMAC headers
    #[instrument(skip(self, signed_order))]
    pub async fn place_order(
        &self,
        signed_order: super::order::SignedOrder,
        owner: &str,
        order_type: super::order::OrderType,
    ) -> Result<super::order::PlaceOrderResponse, AppError> {
        let hmac = self.hmac_auth.as_ref()
            .ok_or_else(|| AppError::Internal("Not authenticated".to_string()))?;

        let path = "/order";
        let url = format!("{}{}", self.base_url, path);

        let request = super::order::PlaceOrderRequest {
            order: signed_order,
            owner: owner.to_string(),
            order_type,
        };

        let body_json = serde_json::to_string(&request)
            .map_err(|e| AppError::Internal(format!("Failed to serialize order: {}", e)))?;

        debug!("Placing order at: {}", url);
        debug!("Order body: {}", body_json);

        let headers = hmac.generate_headers("POST", path, Some(&body_json))?;

        let response = headers.apply_to_request(
            self.client.post(&url)
                .header("Content-Type", "application/json")
                .body(body_json)
        ).send().await?;

        let status = response.status();
        let text = response.text().await.unwrap_or_default();

        debug!("Place order response ({}): {}", status, text);

        if !status.is_success() {
            return Err(AppError::Api(format!("Order placement failed ({}): {}", status, text)));
        }

        let result: super::order::PlaceOrderResponse = serde_json::from_str(&text)
            .map_err(|e| AppError::Internal(format!("Failed to parse order response: {}", e)))?;

        Ok(result)
    }

    /// Cancel a specific order by ID
    #[instrument(skip(self))]
    pub async fn cancel_order(&self, order_id: &str) -> Result<super::order::CancelResponse, AppError> {
        let hmac = self.hmac_auth.as_ref()
            .ok_or_else(|| AppError::Internal("Not authenticated".to_string()))?;

        // AIDEV-NOTE: Path for HMAC is just /order, query params are separate
        let path = "/order";
        let url = format!("{}{}?orderID={}", self.base_url, path, order_id);
        let headers = hmac.generate_headers("DELETE", path, None)?;

        debug!("Cancelling order: {}", order_id);

        let response = headers.apply_to_request(self.client.delete(&url))
            .send()
            .await?;

        let status = response.status();
        let text = response.text().await.unwrap_or_default();

        debug!("Cancel order response ({}): {}", status, text);

        if !status.is_success() {
            return Err(AppError::Api(format!("Cancel failed ({}): {}", status, text)));
        }

        let result: super::order::CancelResponse = serde_json::from_str(&text)
            .map_err(|e| AppError::Internal(format!("Failed to parse cancel response: {}", e)))?;

        Ok(result)
    }

    /// Cancel all open orders
    #[instrument(skip(self))]
    pub async fn cancel_all_orders(&self) -> Result<super::order::CancelResponse, AppError> {
        let hmac = self.hmac_auth.as_ref()
            .ok_or_else(|| AppError::Internal("Not authenticated".to_string()))?;

        let path = "/cancel-all";
        let url = format!("{}{}", self.base_url, path);
        let headers = hmac.generate_headers("DELETE", path, None)?;

        debug!("Cancelling all orders");

        let response = headers.apply_to_request(self.client.delete(&url))
            .send()
            .await?;

        let status = response.status();
        let text = response.text().await.unwrap_or_default();

        debug!("Cancel all response ({}): {}", status, text);

        if !status.is_success() {
            return Err(AppError::Api(format!("Cancel all failed ({}): {}", status, text)));
        }

        let result: super::order::CancelResponse = serde_json::from_str(&text)
            .map_err(|e| AppError::Internal(format!("Failed to parse cancel response: {}", e)))?;

        Ok(result)
    }

    // ========== Price History ==========

    /// Fetch price history for a token
    /// AIDEV-NOTE: No auth required - public endpoint
    /// Parameters:
    /// - token_id: CLOB token ID (long numeric string)
    /// - interval: "1h", "6h", "1d", "1w", "max" (optional, defaults to "max")
    /// - fidelity: resolution in minutes (optional, e.g., 60 for hourly)
    /// - start_ts/end_ts: Unix timestamps for custom range (optional)
    #[instrument(skip(self))]
    pub async fn get_price_history(
        &self,
        token_id: &str,
        interval: Option<&str>,
        fidelity: Option<u32>,
        start_ts: Option<i64>,
        end_ts: Option<i64>,
    ) -> Result<Vec<PricePoint>, AppError> {
        let mut url = format!("{}/prices-history?market={}", self.base_url, token_id);

        // Add optional parameters
        if let Some(iv) = interval {
            url.push_str(&format!("&interval={}", iv));
        }
        if let Some(f) = fidelity {
            url.push_str(&format!("&fidelity={}", f));
        }
        if let Some(start) = start_ts {
            url.push_str(&format!("&startTs={}", start));
        }
        if let Some(end) = end_ts {
            url.push_str(&format!("&endTs={}", end));
        }

        debug!("Fetching price history from: {}", url);

        let response = self.client.get(&url).send().await?;

        let status = response.status();
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(AppError::Api(format!(
                "Price history request failed ({}): {}",
                status, text
            )));
        }

        let text = response.text().await?;
        let parsed: PriceHistoryResponse = serde_json::from_str(&text).map_err(|e| {
            debug!("Failed to parse price history: {}. Response: {}", e, &text[..text.len().min(500)]);
            AppError::Internal(format!("Failed to parse price history: {}", e))
        })?;

        debug!("Fetched {} price history points for {}", parsed.history.len(), token_id);
        Ok(parsed.history)
    }

    /// Cancel all orders for a specific market
    #[instrument(skip(self))]
    pub async fn cancel_market_orders(&self, market_id: &str) -> Result<super::order::CancelResponse, AppError> {
        let hmac = self.hmac_auth.as_ref()
            .ok_or_else(|| AppError::Internal("Not authenticated".to_string()))?;

        // AIDEV-NOTE: Path for HMAC is just /cancel-market-orders
        let path = "/cancel-market-orders";
        let url = format!("{}{}?market={}", self.base_url, path, market_id);
        let headers = hmac.generate_headers("DELETE", path, None)?;

        debug!("Cancelling orders for market: {}", market_id);

        let response = headers.apply_to_request(self.client.delete(&url))
            .send()
            .await?;

        let status = response.status();
        let text = response.text().await.unwrap_or_default();

        debug!("Cancel market orders response ({}): {}", status, text);

        if !status.is_success() {
            return Err(AppError::Api(format!("Cancel market orders failed ({}): {}", status, text)));
        }

        let result: super::order::CancelResponse = serde_json::from_str(&text)
            .map_err(|e| AppError::Internal(format!("Failed to parse cancel response: {}", e)))?;

        Ok(result)
    }
}

impl Default for ClobClient {
    fn default() -> Self {
        Self::new()
    }
}
