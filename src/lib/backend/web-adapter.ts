// AIDEV-NOTE: Web implementation - HTTP fetch + SSE (stub for future use)

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
 * Web backend adapter - uses HTTP fetch for commands.
 * Currently a stub - implement when polymarket-server is ready.
 */
export class WebBackendAdapter implements BackendAdapter {
  // AIDEV-NOTE: baseUrl will be used when HTTP implementation is added
  // @ts-expect-error Property unused until HTTP implementation
  constructor(private baseUrl: string = "/api") {}

  private notImplemented(method: string): never {
    throw new Error(
      `WebBackendAdapter.${method}() not implemented. ` +
        `Run in Tauri or implement polymarket-server.`
    );
  }

  // Markets
  async getMarkets(_query?: string, _limit?: number, _offset?: number): Promise<Market[]> {
    this.notImplemented("getMarkets");
  }

  async getMarket(_marketId: string): Promise<Market> {
    this.notImplemented("getMarket");
  }

  async getEvents(_limit?: number): Promise<Event[]> {
    this.notImplemented("getEvents");
  }

  async searchMarkets(_query: string): Promise<Market[]> {
    this.notImplemented("searchMarkets");
  }

  async getPriceHistory(_params: PriceHistoryParams): Promise<PriceHistoryResult> {
    this.notImplemented("getPriceHistory");
  }

  // WebSocket
  async connectRtds(_markets: string[]): Promise<void> {
    this.notImplemented("connectRtds");
  }

  async disconnectRtds(): Promise<void> {
    this.notImplemented("disconnectRtds");
  }

  async connectClob(_tokenIds: string[]): Promise<void> {
    this.notImplemented("connectClob");
  }

  async disconnectClob(): Promise<void> {
    this.notImplemented("disconnectClob");
  }

  async getConnectionStatus(): Promise<ConnectionStatus> {
    this.notImplemented("getConnectionStatus");
  }

  // Auth
  async getAuthStatus(): Promise<AuthStatus> {
    this.notImplemented("getAuthStatus");
  }

  async login(_privateKey: string): Promise<AuthStatus> {
    this.notImplemented("login");
  }

  async logout(): Promise<AuthStatus> {
    this.notImplemented("logout");
  }

  async setPolymarketAddress(_address: string): Promise<void> {
    this.notImplemented("setPolymarketAddress");
  }

  async getBalance(): Promise<Balance> {
    this.notImplemented("getBalance");
  }

  async getPositions(_address: string): Promise<Position[]> {
    this.notImplemented("getPositions");
  }

  async getOrders(): Promise<Order[]> {
    this.notImplemented("getOrders");
  }

  // Trading
  async placeOrder(_params: OrderParams, _privateKey: string): Promise<PlaceOrderResult> {
    this.notImplemented("placeOrder");
  }

  async cancelOrder(_orderId: string): Promise<CancelResult> {
    this.notImplemented("cancelOrder");
  }

  async cancelAllOrders(): Promise<CancelResult> {
    this.notImplemented("cancelAllOrders");
  }

  async cancelMarketOrders(_marketId: string): Promise<CancelResult> {
    this.notImplemented("cancelMarketOrders");
  }
}

/**
 * Web event subscriber - uses SSE for real-time events.
 * Currently a stub - implement when polymarket-server is ready.
 */
export class WebEventSubscriber implements EventSubscriber {
  private eventSource: EventSource | null = null;
  private callbacks = new Map<string, Set<EventCallback<unknown>>>();

  constructor(private sseUrl: string = "/api/events") {}

  private ensureConnection(): void {
    if (this.eventSource) return;

    this.eventSource = new EventSource(this.sseUrl);
    this.eventSource.onmessage = (event) => {
      try {
        const { type, payload } = JSON.parse(event.data);
        const callbacks = this.callbacks.get(type);
        callbacks?.forEach((cb) => cb(payload));
      } catch (e) {
        console.error("Failed to parse SSE message:", e);
      }
    };
    this.eventSource.onerror = (e) => {
      console.error("SSE connection error:", e);
    };
  }

  async subscribe<T>(eventName: string, callback: EventCallback<T>): Promise<UnsubscribeFn> {
    this.ensureConnection();

    if (!this.callbacks.has(eventName)) {
      this.callbacks.set(eventName, new Set());
    }
    this.callbacks.get(eventName)!.add(callback as EventCallback<unknown>);

    return () => {
      this.callbacks.get(eventName)?.delete(callback as EventCallback<unknown>);
      // Close connection if no more subscribers
      if (this.getTotalSubscribers() === 0) {
        this.eventSource?.close();
        this.eventSource = null;
      }
    };
  }

  async subscribeMany(
    events: Array<{ name: string; callback: EventCallback<unknown> }>
  ): Promise<UnsubscribeFn> {
    const unsubscribers = await Promise.all(
      events.map(({ name, callback }) => this.subscribe(name, callback))
    );
    return () => unsubscribers.forEach((u) => u());
  }

  private getTotalSubscribers(): number {
    let total = 0;
    for (const set of this.callbacks.values()) {
      total += set.size;
    }
    return total;
  }
}
