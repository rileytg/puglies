use serde::{Deserialize, Serialize};

// AIDEV-NOTE: Types mirror frontend types.ts - keep in sync

/// Market token (outcome)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    pub token_id: String,
    pub outcome: String,
    pub price: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub winner: Option<bool>,
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

/// Polymarket market
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Market {
    pub condition_id: String,
    #[serde(default)]
    pub question_id: String,
    pub question: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub market_slug: String,
    #[serde(default)]
    pub end_date_iso: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub game_start_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seconds_delay: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fpmm: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maker_base_fee: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub taker_base_fee: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notifications_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub neg_risk: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub neg_risk_market_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub neg_risk_request_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rewards: Option<MarketRewards>,
    #[serde(default)]
    pub tokens: Vec<Token>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub active: bool,
    #[serde(default)]
    pub closed: bool,
    #[serde(default)]
    pub archived: bool,
    #[serde(default)]
    pub accepting_orders: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accepting_order_timestamp: Option<String>,
    #[serde(default)]
    pub minimum_order_size: f64,
    #[serde(default)]
    pub minimum_tick_size: f64,
    #[serde(default)]
    pub volume: String,
    #[serde(default)]
    pub volume_num: f64,
    #[serde(default)]
    pub liquidity: String,
    #[serde(default)]
    pub liquidity_num: f64,
    #[serde(default)]
    pub spread: f64,
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
