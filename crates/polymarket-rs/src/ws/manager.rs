// AIDEV-NOTE: WebSocket manager - state machine with exponential backoff reconnection

use std::sync::Arc;
use std::time::Duration;
use parking_lot::RwLock;

use crate::types::{ConnectionState, ConnectionStatus};
use super::EventEmitter;

/// Configuration for reconnection behavior
#[derive(Debug, Clone)]
pub struct ReconnectConfig {
    /// Initial delay before first reconnect attempt
    pub initial_delay: Duration,
    /// Maximum delay between reconnect attempts
    pub max_delay: Duration,
    /// Multiplier for exponential backoff
    pub multiplier: f64,
    /// Maximum number of reconnect attempts (None = infinite)
    pub max_attempts: Option<u32>,
}

impl Default for ReconnectConfig {
    fn default() -> Self {
        Self {
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(30),
            multiplier: 2.0,
            max_attempts: None, // Keep trying forever
        }
    }
}

/// Shared state for a WebSocket connection
pub struct WebSocketState {
    pub state: ConnectionState,
    pub reconnect_attempts: u32,
    pub last_message_time: Option<std::time::Instant>,
}

impl Default for WebSocketState {
    fn default() -> Self {
        Self {
            state: ConnectionState::Disconnected,
            reconnect_attempts: 0,
            last_message_time: None,
        }
    }
}

/// Central manager for all WebSocket connections
/// Generic over E: EventEmitter to allow Tauri or other event systems
pub struct WebSocketManager<E: EventEmitter> {
    emitter: Arc<E>,
    rtds_state: Arc<RwLock<WebSocketState>>,
    clob_state: Arc<RwLock<WebSocketState>>,
}

impl<E: EventEmitter> WebSocketManager<E> {
    pub fn new(emitter: Arc<E>) -> Self {
        Self {
            emitter,
            rtds_state: Arc::new(RwLock::new(WebSocketState::default())),
            clob_state: Arc::new(RwLock::new(WebSocketState::default())),
        }
    }

    /// Get the event emitter
    pub fn emitter(&self) -> &Arc<E> {
        &self.emitter
    }

    /// Get the current RTDS connection state
    pub fn rtds_state(&self) -> ConnectionState {
        self.rtds_state.read().state
    }

    /// Get the current CLOB connection state
    pub fn clob_state(&self) -> ConnectionState {
        self.clob_state.read().state
    }

    /// Update RTDS connection state and emit event
    pub fn set_rtds_state(&self, state: ConnectionState) {
        {
            let mut ws_state = self.rtds_state.write();
            ws_state.state = state;
            if state == ConnectionState::Connected {
                ws_state.reconnect_attempts = 0;
            }
        }
        self.emit_connection_status();
    }

    /// Update CLOB connection state and emit event
    pub fn set_clob_state(&self, state: ConnectionState) {
        {
            let mut ws_state = self.clob_state.write();
            ws_state.state = state;
            if state == ConnectionState::Connected {
                ws_state.reconnect_attempts = 0;
            }
        }
        self.emit_connection_status();
    }

    /// Increment reconnect attempts for RTDS and return current count
    pub fn increment_rtds_reconnect(&self) -> u32 {
        let mut state = self.rtds_state.write();
        state.reconnect_attempts += 1;
        state.reconnect_attempts
    }

    /// Increment reconnect attempts for CLOB and return current count
    pub fn increment_clob_reconnect(&self) -> u32 {
        let mut state = self.clob_state.write();
        state.reconnect_attempts += 1;
        state.reconnect_attempts
    }

    /// Calculate delay for next reconnection attempt using exponential backoff
    pub fn calculate_reconnect_delay(attempts: u32, config: &ReconnectConfig) -> Duration {
        let delay_secs = config.initial_delay.as_secs_f64()
            * config.multiplier.powi(attempts.saturating_sub(1) as i32);
        let capped_delay = delay_secs.min(config.max_delay.as_secs_f64());
        Duration::from_secs_f64(capped_delay)
    }

    /// Emit current connection status
    fn emit_connection_status(&self) {
        let status = ConnectionStatus {
            rtds: self.rtds_state(),
            clob: self.clob_state(),
        };
        self.emitter.emit_connection_status(&status);
    }

    /// Record that a message was received (for connection health tracking)
    pub fn record_rtds_message(&self) {
        let mut state = self.rtds_state.write();
        state.last_message_time = Some(std::time::Instant::now());
    }

    pub fn record_clob_message(&self) {
        let mut state = self.clob_state.write();
        state.last_message_time = Some(std::time::Instant::now());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ws::events::NoOpEmitter;

    #[test]
    fn test_reconnect_delay_calculation() {
        let config = ReconnectConfig::default();

        // First attempt: 1 second
        let delay1 = WebSocketManager::<NoOpEmitter>::calculate_reconnect_delay(1, &config);
        assert_eq!(delay1, Duration::from_secs(1));

        // Second attempt: 2 seconds
        let delay2 = WebSocketManager::<NoOpEmitter>::calculate_reconnect_delay(2, &config);
        assert_eq!(delay2, Duration::from_secs(2));

        // Third attempt: 4 seconds
        let delay3 = WebSocketManager::<NoOpEmitter>::calculate_reconnect_delay(3, &config);
        assert_eq!(delay3, Duration::from_secs(4));

        // Should cap at max_delay (30 seconds)
        let delay_many = WebSocketManager::<NoOpEmitter>::calculate_reconnect_delay(10, &config);
        assert_eq!(delay_many, Duration::from_secs(30));
    }

    #[test]
    fn test_websocket_manager_state() {
        let emitter = Arc::new(NoOpEmitter);
        let manager = WebSocketManager::new(emitter);

        assert_eq!(manager.rtds_state(), ConnectionState::Disconnected);
        assert_eq!(manager.clob_state(), ConnectionState::Disconnected);

        manager.set_rtds_state(ConnectionState::Connected);
        assert_eq!(manager.rtds_state(), ConnectionState::Connected);

        manager.set_clob_state(ConnectionState::Connecting);
        assert_eq!(manager.clob_state(), ConnectionState::Connecting);
    }

    #[test]
    fn test_reconnect_counter() {
        let emitter = Arc::new(NoOpEmitter);
        let manager = WebSocketManager::new(emitter);

        assert_eq!(manager.increment_rtds_reconnect(), 1);
        assert_eq!(manager.increment_rtds_reconnect(), 2);

        // Reset when connected
        manager.set_rtds_state(ConnectionState::Connected);
        assert_eq!(manager.increment_rtds_reconnect(), 1);
    }
}
