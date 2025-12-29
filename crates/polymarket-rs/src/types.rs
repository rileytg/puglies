// AIDEV-NOTE: Polymarket types - mirrors frontend types.ts, keep in sync

use serde::{Deserialize, Serialize};

/// Market token (outcome)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    pub token_id: String,
    pub outcome: String,
    pub price: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub winner: Option<bool>,
}

impl Token {
    /// Parse tokens from API response strings
    pub fn from_api_strings(
        outcomes: &str,
        prices: &str,
        token_ids: &str,
    ) -> Vec<Token> {
        let outcomes: Vec<String> = serde_json::from_str(outcomes).unwrap_or_default();
        let prices: Vec<String> = serde_json::from_str(prices).unwrap_or_default();
        let token_ids: Vec<String> = serde_json::from_str(token_ids).unwrap_or_default();

        outcomes
            .into_iter()
            .zip(prices.into_iter())
            .zip(token_ids.into_iter())
            .map(|((outcome, price), token_id)| Token {
                token_id,
                outcome,
                price: price.parse().unwrap_or(0.0),
                winner: None,
            })
            .collect()
    }
}

/// Market rewards configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketRewards {
    pub min_size: f64,
    pub max_spread: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_start_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_end_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_game_multiplier: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rewards_daily_rate: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rewards_min_size: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rewards_max_spread: Option<f64>,
}

/// Raw market from Gamma API (with JSON string fields)
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawMarket {
    pub id: String,
    pub condition_id: String,
    #[serde(default, alias = "questionID")]
    pub question_id: String,
    pub question: String,
    #[serde(default)]
    pub description: String,
    #[serde(default, alias = "slug")]
    pub market_slug: String,
    #[serde(default)]
    pub end_date_iso: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub game_start_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    #[serde(default)]
    pub active: bool,
    #[serde(default)]
    pub closed: bool,
    #[serde(default)]
    pub archived: bool,
    #[serde(default)]
    pub accepting_orders: bool,
    #[serde(default, alias = "volumeNum")]
    pub volume_num: f64,
    #[serde(default, alias = "liquidityNum")]
    pub liquidity_num: f64,
    #[serde(default)]
    pub spread: f64,
    // AIDEV-NOTE: minimum_order_size is usually 1.0 for most markets
    #[serde(default = "default_min_order_size")]
    pub minimum_order_size: f64,
    #[serde(default = "default_min_tick_size")]
    pub minimum_tick_size: f64,
    // Raw string fields from API
    #[serde(default)]
    pub outcomes: String,
    #[serde(default)]
    pub outcome_prices: String,
    #[serde(default)]
    pub clob_token_ids: String,
}

/// Polymarket market (processed)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Market {
    pub id: String,
    pub condition_id: String,
    pub question_id: String,
    pub question: String,
    pub description: String,
    pub market_slug: String,
    pub end_date_iso: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub game_start_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    pub tokens: Vec<Token>,
    pub active: bool,
    pub closed: bool,
    pub archived: bool,
    pub accepting_orders: bool,
    pub volume_num: f64,
    pub liquidity_num: f64,
    pub spread: f64,
    pub minimum_order_size: f64,
    pub minimum_tick_size: f64,
}

// Default values for optional API fields
fn default_min_order_size() -> f64 { 1.0 }
fn default_min_tick_size() -> f64 { 0.01 }

impl From<RawMarket> for Market {
    fn from(raw: RawMarket) -> Self {
        let tokens = Token::from_api_strings(
            &raw.outcomes,
            &raw.outcome_prices,
            &raw.clob_token_ids,
        );

        Self {
            id: raw.id,
            condition_id: raw.condition_id,
            question_id: raw.question_id,
            question: raw.question,
            description: raw.description,
            market_slug: raw.market_slug,
            end_date_iso: raw.end_date_iso,
            game_start_time: raw.game_start_time,
            icon: raw.icon,
            image: raw.image,
            tokens,
            active: raw.active,
            closed: raw.closed,
            archived: raw.archived,
            accepting_orders: raw.accepting_orders,
            volume_num: raw.volume_num,
            liquidity_num: raw.liquidity_num,
            spread: raw.spread,
            minimum_order_size: raw.minimum_order_size,
            minimum_tick_size: raw.minimum_tick_size,
        }
    }
}

/// Polymarket event (collection of markets)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: String,
    #[serde(default)]
    pub ticker: String,
    #[serde(default)]
    pub slug: String,
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(default)]
    pub active: bool,
    #[serde(default)]
    pub closed: bool,
    #[serde(default)]
    pub archived: bool,
    #[serde(default)]
    pub new: bool,
    #[serde(default)]
    pub featured: bool,
    #[serde(default)]
    pub restricted: bool,
    #[serde(default)]
    pub markets: Vec<Market>,
    #[serde(default)]
    pub total_volume: f64,
    #[serde(default)]
    pub total_liquidity: f64,
}

// ============================================================================
// WebSocket Event Types
// ============================================================================

/// Connection state for WebSocket clients
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Reconnecting,
    Failed,
}

impl Default for ConnectionState {
    fn default() -> Self {
        Self::Disconnected
    }
}

/// Connection status for both WebSocket clients
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionStatus {
    pub clob: ConnectionState,
    pub rtds: ConnectionState,
}

/// Price update from WebSocket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceUpdate {
    pub market: String,
    /// Token/asset ID - always present in CLOB, sometimes in RTDS
    pub asset_id: String,
    pub price: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<i64>,
}

/// Order book level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBookLevel {
    pub price: String,
    pub size: String,
}

/// Order book snapshot from CLOB WebSocket
/// AIDEV-NOTE: timestamp comes as String from API, last_trade_price is optional
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBookSnapshot {
    #[serde(rename = "event_type")]
    pub event_type: Option<String>,
    pub asset_id: String,
    pub market: Option<String>,
    pub hash: Option<String>,
    #[serde(default)]
    pub timestamp: Option<i64>,
    pub bids: Vec<OrderBookLevel>,
    pub asks: Vec<OrderBookLevel>,
    #[serde(default)]
    pub last_trade_price: Option<String>,
}

/// Trade event from CLOB
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClobTrade {
    #[serde(rename = "event_type")]
    pub event_type: Option<String>,
    pub asset_id: String,
    pub market: Option<String>,
    pub price: String,
    pub size: String,
    pub side: String,
    pub timestamp: Option<i64>,
    pub trade_id: Option<String>,
}

// ============================================================================
// CLOB API Types
// ============================================================================

/// Balance response from CLOB API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Balance {
    pub balance: String,
    #[serde(default)]
    pub allowances: std::collections::HashMap<String, String>,
}

/// Position from Data API
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Position {
    pub asset: String,
    pub condition_id: String,
    pub size: f64,
    pub avg_price: f64,
    pub initial_value: f64,
    pub current_value: f64,
    pub cash_pnl: f64,
    pub percent_pnl: f64,
    pub cur_price: f64,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub outcome: String,
    #[serde(default)]
    pub proxy_wallet: String,
}

/// Order from CLOB API
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Order {
    pub id: String,
    pub market: String,
    #[serde(default, alias = "asset_id")]
    pub asset: String,
    pub side: String,
    pub original_size: String,
    pub size_matched: String,
    pub price: String,
    pub status: String,
    #[serde(default)]
    pub order_type: String,
    pub created_at: String,
}

/// Price history point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricePoint {
    pub t: i64,  // Unix timestamp (seconds)
    pub p: f64,  // Price (0.0 - 1.0)
}

/// Price history response from Data API
#[derive(Debug, Clone, Deserialize)]
pub struct PriceHistoryResponse {
    pub history: Vec<PricePoint>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_from_api_strings() {
        let outcomes = r#"["Yes","No"]"#;
        let prices = r#"["0.65","0.35"]"#;
        let token_ids = r#"["token1","token2"]"#;

        let tokens = Token::from_api_strings(outcomes, prices, token_ids);

        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].outcome, "Yes");
        assert_eq!(tokens[0].price, 0.65);
        assert_eq!(tokens[1].outcome, "No");
        assert_eq!(tokens[1].price, 0.35);
    }

    #[test]
    fn test_market_deserialization() {
        let json = r#"{
            "id": "123",
            "conditionId": "0xabc",
            "question": "Test market?",
            "outcomes": "[\"Yes\",\"No\"]",
            "outcomePrices": "[\"0.5\",\"0.5\"]",
            "clobTokenIds": "[\"t1\",\"t2\"]"
        }"#;

        let raw: RawMarket = serde_json::from_str(json).unwrap();
        let market: Market = raw.into();

        assert_eq!(market.id, "123");
        assert_eq!(market.condition_id, "0xabc");
        assert_eq!(market.tokens.len(), 2);
    }

    #[test]
    fn test_connection_state_serialization() {
        let state = ConnectionState::Connected;
        let json = serde_json::to_string(&state).unwrap();
        assert_eq!(json, r#""connected""#);
    }
}
