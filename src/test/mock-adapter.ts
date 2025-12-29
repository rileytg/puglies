// AIDEV-NOTE: Mock backend adapter for testing - all methods return configurable test data
import { vi } from "vitest";
import type { BackendAdapter } from "@/lib/backend/types";
import type { EventSubscriber, EventCallback, UnsubscribeFn } from "@/lib/backend/events";
import type {
  Market,
  Event,
  ConnectionStatus,
  AuthStatus,
  Balance,
  Position,
  Order,
  PlaceOrderResult,
  CancelResult,
  PriceHistoryResult,
} from "@/lib/types";

// ============================================================================
// Test Fixtures - Default mock data
// ============================================================================

export const mockMarket: Market = {
  id: "12345",
  condition_id: "0xabc123",
  question_id: "0xdef456",
  question: "Test Market Question?",
  description: "Test market description",
  market_slug: "test-market",
  end_date_iso: "2025-12-31T00:00:00Z",
  tokens: [
    { token_id: "token-yes", outcome: "Yes", price: 0.65 },
    { token_id: "token-no", outcome: "No", price: 0.35 },
  ],
  tags: ["test", "mock"],
  active: true,
  closed: false,
  archived: false,
  accepting_orders: true,
  minimum_order_size: 1,
  minimum_tick_size: 0.01,
  volume: "1000000",
  volume_num: 1000000,
  liquidity: "50000",
  liquidity_num: 50000,
  spread: 0.02,
};

export const mockEvent: Event = {
  id: "event-1",
  ticker: "TEST",
  slug: "test-event",
  title: "Test Event",
  description: "Test event description",
  active: true,
  closed: false,
  archived: false,
  new: false,
  featured: false,
  restricted: false,
  markets: [mockMarket],
  total_volume: 1000000,
  total_liquidity: 50000,
};

export const mockAuthStatus: AuthStatus = {
  isAuthenticated: true,
  address: "0x1234567890abcdef",
  polymarketAddress: "0xpolymarket123",
};

export const mockBalance: Balance = {
  balance: "1000000000", // 1000 USDC (6 decimals)
  allowances: {},
};

export const mockPosition: Position = {
  asset: "token-yes",
  conditionId: "0xabc123",
  size: 100,
  avgPrice: 0.55,
  initialValue: 55,
  currentValue: 65,
  cashPnl: 10,
  percentPnl: 18.18,
  curPrice: 0.65,
  title: "Test Market Question?",
  outcome: "Yes",
  proxyWallet: "0xproxy123",
};

export const mockOrder: Order = {
  id: "order-1",
  market: "0xabc123",
  asset: "token-yes",
  side: "Buy",
  originalSize: "100",
  sizeMatched: "0",
  price: "0.60",
  status: "open",
  orderType: "GTC",
  createdAt: "2025-01-01T00:00:00Z",
};

export const mockConnectionStatus: ConnectionStatus = {
  clob: "connected",
  rtds: "connected",
};

export const mockPlaceOrderResult: PlaceOrderResult = {
  success: true,
  orderId: "order-new-1",
  orderHashes: ["hash-1"],
  status: "placed",
};

export const mockCancelResult: CancelResult = {
  canceled: ["order-1"],
  notCanceled: {},
};

export const mockPriceHistory: PriceHistoryResult = {
  history: [
    { t: 1704067200, p: 0.50 },
    { t: 1704153600, p: 0.55 },
    { t: 1704240000, p: 0.60 },
    { t: 1704326400, p: 0.65 },
  ],
  cachedCount: 4,
  fetchedCount: 0,
};

// ============================================================================
// Mock Backend Adapter
// ============================================================================

export interface MockBackendConfig {
  markets?: Market[];
  market?: Market | null;
  events?: Event[];
  authStatus?: AuthStatus;
  balance?: Balance;
  positions?: Position[];
  orders?: Order[];
  connectionStatus?: ConnectionStatus;
  placeOrderResult?: PlaceOrderResult;
  cancelResult?: CancelResult;
  priceHistory?: PriceHistoryResult;
  // Error simulation
  shouldThrow?: boolean;
  errorMessage?: string;
}

export function createMockBackend(config: MockBackendConfig = {}): BackendAdapter {
  const throwIfConfigured = () => {
    if (config.shouldThrow) {
      throw new Error(config.errorMessage || "Mock error");
    }
  };

  return {
    // Markets
    getMarkets: vi.fn(async () => {
      throwIfConfigured();
      return config.markets ?? [mockMarket];
    }),
    getMarket: vi.fn(async () => {
      throwIfConfigured();
      if (config.market === null) {
        throw new Error("Market not found");
      }
      return config.market ?? mockMarket;
    }),
    getEvents: vi.fn(async () => {
      throwIfConfigured();
      return config.events ?? [mockEvent];
    }),
    searchMarkets: vi.fn(async () => {
      throwIfConfigured();
      return config.markets ?? [mockMarket];
    }),
    getPriceHistory: vi.fn(async () => {
      throwIfConfigured();
      return config.priceHistory ?? mockPriceHistory;
    }),

    // WebSocket
    connectRtds: vi.fn(async () => {
      throwIfConfigured();
    }),
    disconnectRtds: vi.fn(async () => {
      throwIfConfigured();
    }),
    connectClob: vi.fn(async () => {
      throwIfConfigured();
    }),
    disconnectClob: vi.fn(async () => {
      throwIfConfigured();
    }),
    getConnectionStatus: vi.fn(async () => {
      throwIfConfigured();
      return config.connectionStatus ?? mockConnectionStatus;
    }),

    // Auth
    getAuthStatus: vi.fn(async () => {
      throwIfConfigured();
      return config.authStatus ?? mockAuthStatus;
    }),
    login: vi.fn(async () => {
      throwIfConfigured();
      return config.authStatus ?? mockAuthStatus;
    }),
    logout: vi.fn(async () => {
      throwIfConfigured();
      return { isAuthenticated: false };
    }),
    setPolymarketAddress: vi.fn(async () => {
      throwIfConfigured();
    }),
    getBalance: vi.fn(async () => {
      throwIfConfigured();
      return config.balance ?? mockBalance;
    }),
    getPositions: vi.fn(async () => {
      throwIfConfigured();
      return config.positions ?? [mockPosition];
    }),
    getOrders: vi.fn(async () => {
      throwIfConfigured();
      return config.orders ?? [mockOrder];
    }),

    // Trading
    placeOrder: vi.fn(async () => {
      throwIfConfigured();
      return config.placeOrderResult ?? mockPlaceOrderResult;
    }),
    cancelOrder: vi.fn(async () => {
      throwIfConfigured();
      return config.cancelResult ?? mockCancelResult;
    }),
    cancelAllOrders: vi.fn(async () => {
      throwIfConfigured();
      return config.cancelResult ?? mockCancelResult;
    }),
    cancelMarketOrders: vi.fn(async () => {
      throwIfConfigured();
      return config.cancelResult ?? mockCancelResult;
    }),
  };
}

// ============================================================================
// Mock Event Subscriber
// ============================================================================

export interface MockEventSubscriberConfig {
  shouldThrow?: boolean;
  errorMessage?: string;
}

export function createMockEventSubscriber(
  config: MockEventSubscriberConfig = {}
): EventSubscriber & {
  emit: <T>(eventName: string, payload: T) => void;
  getSubscriptions: () => Map<string, Set<EventCallback<unknown>>>;
} {
  const subscriptions = new Map<string, Set<EventCallback<unknown>>>();

  const throwIfConfigured = () => {
    if (config.shouldThrow) {
      throw new Error(config.errorMessage || "Mock event error");
    }
  };

  return {
    subscribe: vi.fn(async <T>(
      eventName: string,
      callback: EventCallback<T>
    ): Promise<UnsubscribeFn> => {
      throwIfConfigured();

      if (!subscriptions.has(eventName)) {
        subscriptions.set(eventName, new Set());
      }
      subscriptions.get(eventName)!.add(callback as EventCallback<unknown>);

      return () => {
        subscriptions.get(eventName)?.delete(callback as EventCallback<unknown>);
      };
    }),

    subscribeMany: vi.fn(async (
      events: Array<{ name: string; callback: EventCallback<unknown> }>
    ): Promise<UnsubscribeFn> => {
      throwIfConfigured();

      for (const { name, callback } of events) {
        if (!subscriptions.has(name)) {
          subscriptions.set(name, new Set());
        }
        subscriptions.get(name)!.add(callback);
      }

      return () => {
        for (const { name, callback } of events) {
          subscriptions.get(name)?.delete(callback);
        }
      };
    }),

    // Test helper: emit an event to all subscribers
    emit: <T>(eventName: string, payload: T) => {
      const callbacks = subscriptions.get(eventName);
      if (callbacks) {
        for (const cb of callbacks) {
          cb(payload);
        }
      }
    },

    // Test helper: get all subscriptions for inspection
    getSubscriptions: () => subscriptions,
  };
}
