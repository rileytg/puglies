// AIDEV-NOTE: WebSocket module - manages RTDS (market activity) and CLOB (order book) connections

mod manager;
mod rtds;
mod clob;

pub use manager::{WebSocketManager, ConnectionState};
pub use rtds::RtdsClient;
pub use clob::ClobWebSocket;
