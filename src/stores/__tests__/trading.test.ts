// AIDEV-NOTE: Tests for trading store - order placement and cancellation with mocked backend
import { describe, it, expect, beforeEach } from "vitest";
import { useTradingStore } from "../trading";
import { _setBackendForTesting, _resetForTesting } from "@/lib/backend";
import {
  createMockBackend,
  mockPlaceOrderResult,
  mockCancelResult,
} from "@/test/mock-adapter";

describe("useTradingStore", () => {
  beforeEach(() => {
    _resetForTesting();
    useTradingStore.setState({
      isPlacingOrder: false,
      orderError: null,
      lastOrderResult: null,
      isCancelling: false,
      cancelError: null,
    });
  });

  describe("initial state", () => {
    it("starts with no active orders or errors", () => {
      const state = useTradingStore.getState();
      expect(state.isPlacingOrder).toBe(false);
      expect(state.orderError).toBeNull();
      expect(state.lastOrderResult).toBeNull();
      expect(state.isCancelling).toBe(false);
      expect(state.cancelError).toBeNull();
    });
  });

  describe("placeOrder", () => {
    const orderParams = {
      tokenId: "token-yes",
      side: "Buy" as const,
      price: 0.65,
      size: 100,
      orderType: "Gtc" as const,
    };

    it("places order successfully", async () => {
      const mockBackend = createMockBackend({
        placeOrderResult: mockPlaceOrderResult,
      });
      _setBackendForTesting(mockBackend);

      const result = await useTradingStore.getState().placeOrder(orderParams, "private-key");

      expect(result).toBe(true);
      expect(mockBackend.placeOrder).toHaveBeenCalledWith(orderParams, "private-key");

      const { isPlacingOrder, lastOrderResult, orderError } = useTradingStore.getState();
      expect(isPlacingOrder).toBe(false);
      expect(lastOrderResult).toEqual(mockPlaceOrderResult);
      expect(orderError).toBeNull();
    });

    it("returns false when order fails", async () => {
      const mockBackend = createMockBackend({
        placeOrderResult: {
          success: false,
          errorMsg: "Insufficient balance",
        },
      });
      _setBackendForTesting(mockBackend);

      const result = await useTradingStore.getState().placeOrder(orderParams, "key");

      expect(result).toBe(false);

      const { orderError, lastOrderResult } = useTradingStore.getState();
      expect(orderError).toBe("Insufficient balance");
      expect(lastOrderResult?.success).toBe(false);
    });

    it("handles network errors", async () => {
      const mockBackend = createMockBackend({
        shouldThrow: true,
        errorMessage: "Network timeout",
      });
      _setBackendForTesting(mockBackend);

      const result = await useTradingStore.getState().placeOrder(orderParams, "key");

      expect(result).toBe(false);

      const { orderError, isPlacingOrder } = useTradingStore.getState();
      expect(orderError).toBe("Network timeout");
      expect(isPlacingOrder).toBe(false);
    });

    it("stores order result with orderId", async () => {
      const mockBackend = createMockBackend({
        placeOrderResult: {
          success: true,
          orderId: "order-123",
          orderHashes: ["hash-abc"],
          status: "placed",
        },
      });
      _setBackendForTesting(mockBackend);

      await useTradingStore.getState().placeOrder(orderParams, "key");

      const { lastOrderResult } = useTradingStore.getState();
      expect(lastOrderResult?.orderId).toBe("order-123");
      expect(lastOrderResult?.orderHashes).toContain("hash-abc");
    });
  });

  describe("cancelOrder", () => {
    it("cancels order successfully", async () => {
      const mockBackend = createMockBackend({
        cancelResult: { canceled: ["order-1"], notCanceled: {} },
      });
      _setBackendForTesting(mockBackend);

      const result = await useTradingStore.getState().cancelOrder("order-1");

      expect(result).toBe(true);
      expect(mockBackend.cancelOrder).toHaveBeenCalledWith("order-1");

      const { isCancelling, cancelError } = useTradingStore.getState();
      expect(isCancelling).toBe(false);
      expect(cancelError).toBeNull();
    });

    it("returns false when cancel fails", async () => {
      const mockBackend = createMockBackend({
        cancelResult: {
          canceled: [],
          notCanceled: { "order-1": "Order already filled" },
        },
      });
      _setBackendForTesting(mockBackend);

      const result = await useTradingStore.getState().cancelOrder("order-1");

      expect(result).toBe(false);

      const { cancelError } = useTradingStore.getState();
      expect(cancelError).toBe("Order already filled");
    });

    it("handles network errors", async () => {
      const mockBackend = createMockBackend({
        shouldThrow: true,
        errorMessage: "Cancel failed",
      });
      _setBackendForTesting(mockBackend);

      const result = await useTradingStore.getState().cancelOrder("order-1");

      expect(result).toBe(false);

      const { cancelError, isCancelling } = useTradingStore.getState();
      expect(cancelError).toBe("Cancel failed");
      expect(isCancelling).toBe(false);
    });
  });

  describe("cancelAllOrders", () => {
    it("cancels all orders successfully", async () => {
      const mockBackend = createMockBackend({
        cancelResult: mockCancelResult,
      });
      _setBackendForTesting(mockBackend);

      const result = await useTradingStore.getState().cancelAllOrders();

      expect(result).toBe(true);
      expect(mockBackend.cancelAllOrders).toHaveBeenCalled();
    });

    it("handles errors", async () => {
      const mockBackend = createMockBackend({
        shouldThrow: true,
        errorMessage: "Batch cancel failed",
      });
      _setBackendForTesting(mockBackend);

      const result = await useTradingStore.getState().cancelAllOrders();

      expect(result).toBe(false);
      expect(useTradingStore.getState().cancelError).toBe("Batch cancel failed");
    });
  });

  describe("cancelMarketOrders", () => {
    it("cancels orders for specific market", async () => {
      const mockBackend = createMockBackend({
        cancelResult: mockCancelResult,
      });
      _setBackendForTesting(mockBackend);

      const result = await useTradingStore.getState().cancelMarketOrders("market-123");

      expect(result).toBe(true);
      expect(mockBackend.cancelMarketOrders).toHaveBeenCalledWith("market-123");
    });
  });

  describe("clearErrors", () => {
    it("clears both order and cancel errors", () => {
      useTradingStore.setState({
        orderError: "Order error",
        cancelError: "Cancel error",
      });

      useTradingStore.getState().clearErrors();

      const { orderError, cancelError } = useTradingStore.getState();
      expect(orderError).toBeNull();
      expect(cancelError).toBeNull();
    });
  });

  describe("clearOrderResult", () => {
    it("clears last order result", () => {
      useTradingStore.setState({
        lastOrderResult: mockPlaceOrderResult,
      });

      useTradingStore.getState().clearOrderResult();

      const { lastOrderResult } = useTradingStore.getState();
      expect(lastOrderResult).toBeNull();
    });
  });
});
