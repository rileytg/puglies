// AIDEV-NOTE: Tauri implementation - wraps invoke() and listen()

import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { BackendAdapter } from "./types";
import type { EventSubscriber, EventCallback, UnsubscribeFn } from "./events";
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
 * Tauri backend adapter - uses invoke() for commands
 */
export class TauriBackendAdapter implements BackendAdapter {
  // Markets
  async getMarkets(query?: string, limit?: number, offset?: number): Promise<Market[]> {
    return invoke("get_markets", { query, limit, offset });
  }

  async getMarket(marketId: string): Promise<Market> {
    return invoke("get_market", { marketId });
  }

  async getEvents(limit?: number): Promise<Event[]> {
    return invoke("get_events", { limit });
  }

  async searchMarkets(query: string): Promise<Market[]> {
    return invoke("search_markets", { query });
  }

  async getPriceHistory(params: PriceHistoryParams): Promise<PriceHistoryResult> {
    return invoke("get_price_history", { params });
  }

  // WebSocket
  async connectRtds(markets: string[]): Promise<void> {
    return invoke("connect_rtds", { markets });
  }

  async disconnectRtds(): Promise<void> {
    return invoke("disconnect_rtds");
  }

  async connectClob(tokenIds: string[]): Promise<void> {
    return invoke("connect_clob", { tokenIds });
  }

  async disconnectClob(): Promise<void> {
    return invoke("disconnect_clob");
  }

  async getConnectionStatus(): Promise<ConnectionStatus> {
    return invoke("get_connection_status");
  }

  // Auth
  async getAuthStatus(): Promise<AuthStatus> {
    return invoke("get_auth_status");
  }

  async login(privateKey: string): Promise<AuthStatus> {
    return invoke("login", { privateKey });
  }

  async logout(): Promise<AuthStatus> {
    return invoke("logout");
  }

  async setPolymarketAddress(address: string): Promise<void> {
    return invoke("set_polymarket_address", { address });
  }

  async getBalance(): Promise<Balance> {
    return invoke("get_balance");
  }

  async getPositions(address: string): Promise<Position[]> {
    return invoke("get_positions", { address });
  }

  async getOrders(): Promise<Order[]> {
    return invoke("get_orders");
  }

  // Trading
  async placeOrder(params: OrderParams, privateKey: string): Promise<PlaceOrderResult> {
    return invoke("place_order", { params, privateKey });
  }

  async cancelOrder(orderId: string): Promise<CancelResult> {
    return invoke("cancel_order", { orderId });
  }

  async cancelAllOrders(): Promise<CancelResult> {
    return invoke("cancel_all_orders");
  }

  async cancelMarketOrders(marketId: string): Promise<CancelResult> {
    return invoke("cancel_market_orders", { marketId });
  }
}

/**
 * Tauri event subscriber - uses listen() for real-time events
 */
export class TauriEventSubscriber implements EventSubscriber {
  async subscribe<T>(eventName: string, callback: EventCallback<T>): Promise<UnsubscribeFn> {
    const unlisten = await listen<T>(eventName, (event) => callback(event.payload));
    return unlisten;
  }

  async subscribeMany(
    events: Array<{ name: string; callback: EventCallback<unknown> }>
  ): Promise<UnsubscribeFn> {
    const unlisteners: UnlistenFn[] = [];
    for (const { name, callback } of events) {
      const unlisten = await listen(name, (event) => callback(event.payload));
      unlisteners.push(unlisten);
    }
    return () => unlisteners.forEach((u) => u());
  }
}
