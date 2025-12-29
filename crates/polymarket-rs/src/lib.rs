// AIDEV-NOTE: polymarket-rs - Polymarket API client library (zero Tauri deps)
//
// This crate provides:
// - REST API clients (Gamma, CLOB)
// - WebSocket clients (CLOB order book, RTDS live data)
// - Authentication (EIP-712 signing, HMAC)
// - Common types (Market, Order, Position, etc.)

pub mod api;
pub mod auth;
pub mod error;
pub mod types;
pub mod ws;

// Re-export main types for convenience
pub use api::{ClobClient, GammaClient};
pub use auth::{ApiCredentials, AuthHeaders, AuthStatus, HmacAuth, L1Headers, OrderSigner, PolymarketSigner};
pub use error::{ApiError, ApiResult};
pub use types::{
    Balance, ClobTrade, ConnectionState, ConnectionStatus, Event, Market,
    Order, OrderBookLevel, OrderBookSnapshot, Position, PricePoint,
    PriceUpdate, RawMarket, Token,
};
pub use ws::{ClobWebSocket, EventEmitter, NoOpEmitter, ReconnectConfig, RtdsClient, WebSocketManager};
