// AIDEV-NOTE: Tests for backend provider - isTauri detection and getBackend/getEventSubscriber
import { describe, it, expect, beforeEach, vi } from "vitest";
import {
  isTauri,
  getBackend,
  getEventSubscriber,
  _resetForTesting,
  _setBackendForTesting,
  _setEventSubscriberForTesting,
} from "../index";
import { createMockBackend, createMockEventSubscriber } from "@/test/mock-adapter";

describe("isTauri", () => {
  beforeEach(() => {
    // Clean up window state
    delete (window as Record<string, unknown>).__TAURI_INTERNALS__;
  });

  it("returns false when __TAURI_INTERNALS__ is not present", () => {
    expect(isTauri()).toBe(false);
  });

  it("returns true when __TAURI_INTERNALS__ is present", () => {
    (window as Record<string, unknown>).__TAURI_INTERNALS__ = {};
    expect(isTauri()).toBe(true);
  });

  it("returns false in non-browser environment", () => {
    // In jsdom, window is always defined, but we can test the logic
    // by checking the condition works correctly
    expect(typeof window !== "undefined").toBe(true);
    expect("__TAURI_INTERNALS__" in window).toBe(false);
    expect(isTauri()).toBe(false);
  });
});

describe("getBackend", () => {
  beforeEach(() => {
    _resetForTesting();
    delete (window as Record<string, unknown>).__TAURI_INTERNALS__;
  });

  it("returns WebBackendAdapter when not in Tauri", async () => {
    const backend = await getBackend();
    // WebBackendAdapter methods throw "not implemented" errors
    await expect(backend.getMarkets()).rejects.toThrow("not implemented");
  });

  it("returns same instance on subsequent calls (singleton)", async () => {
    const backend1 = await getBackend();
    const backend2 = await getBackend();
    expect(backend1).toBe(backend2);
  });

  it("returns different instance after reset", async () => {
    const backend1 = await getBackend();
    _resetForTesting();
    const backend2 = await getBackend();
    // Both are WebBackendAdapter but different instances
    expect(backend1).not.toBe(backend2);
  });
});

describe("getEventSubscriber", () => {
  beforeEach(() => {
    _resetForTesting();
    delete (window as Record<string, unknown>).__TAURI_INTERNALS__;
  });

  it("returns WebEventSubscriber when not in Tauri", async () => {
    const subscriber = await getEventSubscriber();
    // WebEventSubscriber has subscribe method
    expect(typeof subscriber.subscribe).toBe("function");
    expect(typeof subscriber.subscribeMany).toBe("function");
  });

  it("returns same instance on subsequent calls (singleton)", async () => {
    const sub1 = await getEventSubscriber();
    const sub2 = await getEventSubscriber();
    expect(sub1).toBe(sub2);
  });
});

describe("_setBackendForTesting", () => {
  beforeEach(() => {
    _resetForTesting();
  });

  it("allows injecting a mock backend", async () => {
    const mockBackend = createMockBackend({
      markets: [
        {
          id: "test-123",
          condition_id: "0xtest",
          question_id: "0xq",
          question: "Test?",
          description: "desc",
          market_slug: "test",
          end_date_iso: "2025-12-31",
          tokens: [],
          tags: [],
          active: true,
          closed: false,
          archived: false,
          accepting_orders: true,
          minimum_order_size: 1,
          minimum_tick_size: 0.01,
          volume: "100",
          volume_num: 100,
          liquidity: "50",
          liquidity_num: 50,
          spread: 0.01,
        },
      ],
    });

    _setBackendForTesting(mockBackend);
    const backend = await getBackend();

    expect(backend).toBe(mockBackend);

    const markets = await backend.getMarkets();
    expect(markets).toHaveLength(1);
    expect(markets[0].id).toBe("test-123");
  });

  it("mock backend methods are called correctly", async () => {
    const mockBackend = createMockBackend();
    _setBackendForTesting(mockBackend);

    const backend = await getBackend();
    await backend.getMarkets("query", 10, 5);

    expect(mockBackend.getMarkets).toHaveBeenCalledWith("query", 10, 5);
  });
});

describe("_setEventSubscriberForTesting", () => {
  beforeEach(() => {
    _resetForTesting();
  });

  it("allows injecting a mock event subscriber", async () => {
    const mockSubscriber = createMockEventSubscriber();
    _setEventSubscriberForTesting(mockSubscriber);

    const subscriber = await getEventSubscriber();
    expect(subscriber).toBe(mockSubscriber);
  });

  it("mock subscriber can emit events to callbacks", async () => {
    const mockSubscriber = createMockEventSubscriber();
    _setEventSubscriberForTesting(mockSubscriber);

    const subscriber = await getEventSubscriber();
    const callback = vi.fn();

    await subscriber.subscribe("test_event", callback);
    mockSubscriber.emit("test_event", { data: "hello" });

    expect(callback).toHaveBeenCalledWith({ data: "hello" });
  });

  it("unsubscribe removes callback", async () => {
    const mockSubscriber = createMockEventSubscriber();
    _setEventSubscriberForTesting(mockSubscriber);

    const subscriber = await getEventSubscriber();
    const callback = vi.fn();

    const unsubscribe = await subscriber.subscribe("test_event", callback);
    unsubscribe();

    mockSubscriber.emit("test_event", { data: "should not receive" });
    expect(callback).not.toHaveBeenCalled();
  });
});

describe("BackendAdapter interface contract", () => {
  // These tests verify the mock adapter implements the full interface
  let backend: ReturnType<typeof createMockBackend>;

  beforeEach(() => {
    backend = createMockBackend();
  });

  describe("Markets API", () => {
    it("getMarkets returns array of markets", async () => {
      const markets = await backend.getMarkets();
      expect(Array.isArray(markets)).toBe(true);
      expect(markets[0]).toHaveProperty("id");
      expect(markets[0]).toHaveProperty("question");
      expect(markets[0]).toHaveProperty("tokens");
    });

    it("getMarket returns single market", async () => {
      const market = await backend.getMarket("123");
      expect(market).toHaveProperty("id");
      expect(market).toHaveProperty("condition_id");
    });

    it("searchMarkets filters markets", async () => {
      const markets = await backend.searchMarkets("test");
      expect(Array.isArray(markets)).toBe(true);
      expect(backend.searchMarkets).toHaveBeenCalledWith("test");
    });

    it("getPriceHistory returns history data", async () => {
      const result = await backend.getPriceHistory({ tokenId: "token-1" });
      expect(result).toHaveProperty("history");
      expect(result).toHaveProperty("cachedCount");
      expect(result).toHaveProperty("fetchedCount");
      expect(Array.isArray(result.history)).toBe(true);
    });
  });

  describe("WebSocket API", () => {
    it("connectRtds connects to RTDS", async () => {
      await backend.connectRtds(["market-1", "market-2"]);
      expect(backend.connectRtds).toHaveBeenCalledWith(["market-1", "market-2"]);
    });

    it("connectClob connects to CLOB", async () => {
      await backend.connectClob(["token-1"]);
      expect(backend.connectClob).toHaveBeenCalledWith(["token-1"]);
    });

    it("getConnectionStatus returns status object", async () => {
      const status = await backend.getConnectionStatus();
      expect(status).toHaveProperty("clob");
      expect(status).toHaveProperty("rtds");
    });
  });

  describe("Auth API", () => {
    it("getAuthStatus returns auth state", async () => {
      const status = await backend.getAuthStatus();
      expect(status).toHaveProperty("isAuthenticated");
    });

    it("login returns updated auth state", async () => {
      const status = await backend.login("private-key");
      expect(status.isAuthenticated).toBe(true);
    });

    it("logout returns unauthenticated state", async () => {
      const status = await backend.logout();
      expect(status.isAuthenticated).toBe(false);
    });

    it("getBalance returns balance object", async () => {
      const balance = await backend.getBalance();
      expect(balance).toHaveProperty("balance");
      expect(balance).toHaveProperty("allowances");
    });

    it("getPositions returns array of positions", async () => {
      const positions = await backend.getPositions("0xaddr");
      expect(Array.isArray(positions)).toBe(true);
      expect(positions[0]).toHaveProperty("size");
      expect(positions[0]).toHaveProperty("cashPnl");
    });

    it("getOrders returns array of orders", async () => {
      const orders = await backend.getOrders();
      expect(Array.isArray(orders)).toBe(true);
      expect(orders[0]).toHaveProperty("id");
      expect(orders[0]).toHaveProperty("status");
    });
  });

  describe("Trading API", () => {
    it("placeOrder returns result with success status", async () => {
      const result = await backend.placeOrder(
        {
          tokenId: "token-1",
          side: "Buy",
          price: 0.65,
          size: 100,
          orderType: "Gtc",
        },
        "private-key"
      );
      expect(result).toHaveProperty("success");
      expect(result.success).toBe(true);
    });

    it("cancelOrder returns cancel result", async () => {
      const result = await backend.cancelOrder("order-1");
      expect(result).toHaveProperty("canceled");
      expect(result).toHaveProperty("notCanceled");
    });

    it("cancelAllOrders returns cancel result", async () => {
      const result = await backend.cancelAllOrders();
      expect(result).toHaveProperty("canceled");
    });
  });

  describe("Error handling", () => {
    it("throws when configured to throw", async () => {
      const errorBackend = createMockBackend({
        shouldThrow: true,
        errorMessage: "Test error",
      });

      await expect(errorBackend.getMarkets()).rejects.toThrow("Test error");
      await expect(errorBackend.getAuthStatus()).rejects.toThrow("Test error");
      await expect(errorBackend.placeOrder(
        { tokenId: "t", side: "Buy", price: 0.5, size: 1, orderType: "Gtc" },
        "key"
      )).rejects.toThrow("Test error");
    });

    it("getMarket throws when market is null", async () => {
      const backend = createMockBackend({ market: null });
      await expect(backend.getMarket("nonexistent")).rejects.toThrow("Market not found");
    });
  });
});

describe("EventSubscriber interface contract", () => {
  let subscriber: ReturnType<typeof createMockEventSubscriber>;

  beforeEach(() => {
    subscriber = createMockEventSubscriber();
  });

  it("subscribe adds callback for event", async () => {
    const callback = vi.fn();
    await subscriber.subscribe("event_a", callback);

    const subs = subscriber.getSubscriptions();
    expect(subs.has("event_a")).toBe(true);
    expect(subs.get("event_a")?.size).toBe(1);
  });

  it("subscribeMany adds multiple callbacks", async () => {
    const cb1 = vi.fn();
    const cb2 = vi.fn();

    await subscriber.subscribeMany([
      { name: "event_a", callback: cb1 },
      { name: "event_b", callback: cb2 },
    ]);

    const subs = subscriber.getSubscriptions();
    expect(subs.has("event_a")).toBe(true);
    expect(subs.has("event_b")).toBe(true);
  });

  it("emit calls all subscribers for event", async () => {
    const cb1 = vi.fn();
    const cb2 = vi.fn();

    await subscriber.subscribe("event_x", cb1);
    await subscriber.subscribe("event_x", cb2);

    subscriber.emit("event_x", { value: 42 });

    expect(cb1).toHaveBeenCalledWith({ value: 42 });
    expect(cb2).toHaveBeenCalledWith({ value: 42 });
  });

  it("emit does not call subscribers for other events", async () => {
    const cb1 = vi.fn();
    const cb2 = vi.fn();

    await subscriber.subscribe("event_a", cb1);
    await subscriber.subscribe("event_b", cb2);

    subscriber.emit("event_a", { data: "a" });

    expect(cb1).toHaveBeenCalled();
    expect(cb2).not.toHaveBeenCalled();
  });

  it("subscribeMany unsubscribe removes all callbacks", async () => {
    const cb1 = vi.fn();
    const cb2 = vi.fn();

    const unsub = await subscriber.subscribeMany([
      { name: "event_a", callback: cb1 },
      { name: "event_b", callback: cb2 },
    ]);

    unsub();

    subscriber.emit("event_a", {});
    subscriber.emit("event_b", {});

    expect(cb1).not.toHaveBeenCalled();
    expect(cb2).not.toHaveBeenCalled();
  });

  it("throws when configured to throw", async () => {
    const errorSubscriber = createMockEventSubscriber({
      shouldThrow: true,
      errorMessage: "Event error",
    });

    await expect(errorSubscriber.subscribe("e", vi.fn())).rejects.toThrow("Event error");
  });
});
