import { invoke } from "@tauri-apps/api/core";
import type {
  Market,
  Event,
  ConnectionStatus,
  AuthStatus,
  Balance,
  Position,
  Order,
  OrderParams,
  PlaceOrderResult,
  CancelResult,
  PriceHistoryParams,
  PriceHistoryResult,
} from "./types";

// AIDEV-NOTE: Tauri command wrappers - keep in sync with src-tauri/src/commands/

// Market commands
export async function getMarkets(
  query?: string,
  limit?: number,
  offset?: number
): Promise<Market[]> {
  return invoke("get_markets", { query, limit, offset });
}

// AIDEV-NOTE: marketId is Gamma's internal ID, not condition_id
export async function getMarket(marketId: string): Promise<Market> {
  return invoke("get_market", { marketId });
}

export async function getEvents(limit?: number): Promise<Event[]> {
  return invoke("get_events", { limit });
}

export async function searchMarkets(query: string): Promise<Market[]> {
  return invoke("search_markets", { query });
}

// AIDEV-NOTE: Fetches price history with caching - checks DB first, then API
export async function getPriceHistory(
  params: PriceHistoryParams
): Promise<PriceHistoryResult> {
  return invoke("get_price_history", { params });
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

// Trading commands
export async function placeOrder(
  params: OrderParams,
  privateKey: string
): Promise<PlaceOrderResult> {
  return invoke("place_order", { params, privateKey });
}

export async function cancelOrder(orderId: string): Promise<CancelResult> {
  return invoke("cancel_order", { orderId });
}

export async function cancelAllOrders(): Promise<CancelResult> {
  return invoke("cancel_all_orders");
}

export async function cancelMarketOrders(marketId: string): Promise<CancelResult> {
  return invoke("cancel_market_orders", { marketId });
}
