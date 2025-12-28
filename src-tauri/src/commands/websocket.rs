// AIDEV-NOTE: Tauri commands for WebSocket connection management

use tauri::State;
use serde::Serialize;
use crate::WebSocketState;
use crate::websocket::{ConnectionState, RtdsClient, ClobWebSocket};

/// Response for connection status
#[derive(Debug, Serialize)]
pub struct ConnectionStatusResponse {
    pub rtds: ConnectionState,
    pub clob: ConnectionState,
}

/// Connect to RTDS WebSocket for market activity
#[tauri::command]
pub async fn connect_rtds(
    ws_state: State<'_, WebSocketState>,
    markets: Vec<String>,
) -> Result<(), String> {
    // Take out any existing client and disconnect it (outside await)
    let old_client = {
        let mut guard = ws_state.rtds.write();
        guard.take()
    };

    if let Some(mut client) = old_client {
        client.disconnect();
    }

    // Create and start new connection
    let mut client = RtdsClient::new(ws_state.manager.clone());
    client.connect(markets).await;

    // Store the new client
    {
        let mut guard = ws_state.rtds.write();
        *guard = Some(client);
    }

    Ok(())
}

/// Disconnect from RTDS WebSocket
#[tauri::command]
pub fn disconnect_rtds(ws_state: State<'_, WebSocketState>) -> Result<(), String> {
    let mut rtds_guard = ws_state.rtds.write();

    if let Some(mut client) = rtds_guard.take() {
        client.disconnect();
    }

    Ok(())
}

/// Connect to CLOB WebSocket for order book data
#[tauri::command]
pub async fn connect_clob(
    ws_state: State<'_, WebSocketState>,
    token_ids: Vec<String>,
) -> Result<(), String> {
    // Take out any existing client and disconnect it (outside await)
    let old_client = {
        let mut guard = ws_state.clob.write();
        guard.take()
    };

    if let Some(mut client) = old_client {
        client.disconnect();
    }

    // Create and start new connection
    let mut client = ClobWebSocket::new(ws_state.manager.clone());
    client.connect(token_ids).await;

    // Store the new client
    {
        let mut guard = ws_state.clob.write();
        *guard = Some(client);
    }

    Ok(())
}

/// Disconnect from CLOB WebSocket
#[tauri::command]
pub fn disconnect_clob(ws_state: State<'_, WebSocketState>) -> Result<(), String> {
    let mut clob_guard = ws_state.clob.write();

    if let Some(mut client) = clob_guard.take() {
        client.disconnect();
    }

    Ok(())
}

/// Get current connection status for both WebSockets
#[tauri::command]
pub fn get_connection_status(ws_state: State<'_, WebSocketState>) -> ConnectionStatusResponse {
    ConnectionStatusResponse {
        rtds: ws_state.manager.rtds_state(),
        clob: ws_state.manager.clob_state(),
    }
}
