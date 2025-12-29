// AIDEV-NOTE: Backend adapter interface - abstracts Tauri vs Web runtime

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
} from "../types";

/**
 * Backend adapter interface for all API commands.
 * Implement this for Tauri (invoke) and Web (HTTP/fetch).
 */
export interface BackendAdapter {
  // Markets
  getMarkets(query?: string, limit?: number, offset?: number): Promise<Market[]>;
  getMarket(marketId: string): Promise<Market>;
  getEvents(limit?: number): Promise<Event[]>;
  searchMarkets(query: string): Promise<Market[]>;
  getPriceHistory(params: PriceHistoryParams): Promise<PriceHistoryResult>;

  // WebSocket
  connectRtds(markets: string[]): Promise<void>;
  disconnectRtds(): Promise<void>;
  connectClob(tokenIds: string[]): Promise<void>;
  disconnectClob(): Promise<void>;
  getConnectionStatus(): Promise<ConnectionStatus>;

  // Auth
  getAuthStatus(): Promise<AuthStatus>;
  login(privateKey: string): Promise<AuthStatus>;
  logout(): Promise<AuthStatus>;
  setPolymarketAddress(address: string): Promise<void>;
  getBalance(): Promise<Balance>;
  getPositions(address: string): Promise<Position[]>;
  getOrders(): Promise<Order[]>;

  // Trading
  placeOrder(params: OrderParams, privateKey: string): Promise<PlaceOrderResult>;
  cancelOrder(orderId: string): Promise<CancelResult>;
  cancelAllOrders(): Promise<CancelResult>;
  cancelMarketOrders(marketId: string): Promise<CancelResult>;
}
