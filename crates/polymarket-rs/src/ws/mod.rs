// AIDEV-NOTE: WebSocket module - manages RTDS (market activity) and CLOB (order book) connections

mod events;
mod manager;
mod rtds;
mod clob;

#[cfg(test)]
mod tests;

pub use events::{EventEmitter, NoOpEmitter, RtdsTrade};
pub use manager::{WebSocketManager, WebSocketState, ReconnectConfig};
pub use rtds::RtdsClient;
pub use clob::ClobWebSocket;
