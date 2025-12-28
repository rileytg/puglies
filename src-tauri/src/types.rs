use serde::{Deserialize, Serialize};

// AIDEV-NOTE: Types mirror frontend types.ts - keep in sync

/// Market token (outcome)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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
#[serde(rename_all = "camelCase")]
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

/// Raw market from API (with JSON string fields)
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawMarket {
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
#[serde(rename_all = "camelCase")]
pub struct Market {
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
}

impl From<RawMarket> for Market {
    fn from(raw: RawMarket) -> Self {
        let tokens = Token::from_api_strings(
            &raw.outcomes,
            &raw.outcome_prices,
            &raw.clob_token_ids,
        );

        Self {
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
        }
    }
}

/// Polymarket event (collection of markets)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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
