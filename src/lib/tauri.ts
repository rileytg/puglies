import { invoke } from "@tauri-apps/api/core";
import type { Market, Event, ConnectionStatus, AuthStatus, Balance, Position, Order } from "./types";

// AIDEV-NOTE: Tauri command wrappers - keep in sync with src-tauri/src/commands/

// Market commands
export async function getMarkets(
  query?: string,
  limit?: number,
  offset?: number
): Promise<Market[]> {
  return invoke("get_markets", { query, limit, offset });
}

export async function getMarket(conditionId: string): Promise<Market> {
  return invoke("get_market", { conditionId });
}

export async function getEvents(limit?: number): Promise<Event[]> {
  return invoke("get_events", { limit });
}

export async function searchMarkets(query: string): Promise<Market[]> {
  return invoke("search_markets", { query });
}

// WebSocket commands
export async function connectRtds(markets: string[]): Promise<void> {
  return invoke("connect_rtds", { markets });
}

export async function disconnectRtds(): Promise<void> {
  return invoke("disconnect_rtds");
}

export async function connectClob(tokenIds: string[]): Promise<void> {
  return invoke("connect_clob", { tokenIds });
}

export async function disconnectClob(): Promise<void> {
  return invoke("disconnect_clob");
}

export async function getConnectionStatus(): Promise<ConnectionStatus> {
  return invoke("get_connection_status");
}

// Auth commands
export async function getAuthStatus(): Promise<AuthStatus> {
  return invoke("get_auth_status");
}

export async function login(privateKey: string): Promise<AuthStatus> {
  return invoke("login", { privateKey });
}

export async function logout(): Promise<AuthStatus> {
  return invoke("logout");
}

export async function setPolymarketAddress(address: string): Promise<void> {
  return invoke("set_polymarket_address", { address });
}

export async function getBalance(): Promise<Balance> {
  return invoke("get_balance");
}

export async function getPositions(address: string): Promise<Position[]> {
  return invoke("get_positions", { address });
}

export async function getOrders(): Promise<Order[]> {
  return invoke("get_orders");
}
