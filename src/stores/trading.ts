// AIDEV-NOTE: Trading state store - manages order placement and cancellation

import { create } from "zustand";
import type { OrderParams, PlaceOrderResult } from "@/lib/types";
import { getBackend } from "@/lib/backend";

interface TradingState {
  // Order form state
  isPlacingOrder: boolean;
  orderError: string | null;
  lastOrderResult: PlaceOrderResult | null;

  // Cancellation state
  isCancelling: boolean;
  cancelError: string | null;

  // Actions
  placeOrder: (params: OrderParams, privateKey: string) => Promise<boolean>;
  cancelOrder: (orderId: string) => Promise<boolean>;
  cancelAllOrders: () => Promise<boolean>;
  cancelMarketOrders: (marketId: string) => Promise<boolean>;
  clearErrors: () => void;
  clearOrderResult: () => void;
}

export const useTradingStore = create<TradingState>((set) => ({
  // Initial state
  isPlacingOrder: false,
  orderError: null,
  lastOrderResult: null,
  isCancelling: false,
  cancelError: null,

  // Place a new order
  placeOrder: async (params: OrderParams, privateKey: string) => {
    set({ isPlacingOrder: true, orderError: null, lastOrderResult: null });

    try {
      const backend = await getBackend();
      const result = await backend.placeOrder(params, privateKey);
      set({ isPlacingOrder: false, lastOrderResult: result });

      if (!result.success) {
        set({ orderError: result.errorMsg || "Order failed" });
        return false;
      }

      return true;
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      set({ isPlacingOrder: false, orderError: errorMessage });
      return false;
    }
  },

  // Cancel a specific order
  cancelOrder: async (orderId: string) => {
    set({ isCancelling: true, cancelError: null });

    try {
      const backend = await getBackend();
      const result = await backend.cancelOrder(orderId);
      set({ isCancelling: false });

      // Check if the order was successfully canceled
      if (result.canceled.includes(orderId)) {
        return true;
      }

      // Check if it failed
      const failReason = result.notCanceled[orderId];
      if (failReason) {
        set({ cancelError: failReason });
        return false;
      }

      return true;
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      set({ isCancelling: false, cancelError: errorMessage });
      return false;
    }
  },

  // Cancel all orders
  cancelAllOrders: async () => {
    set({ isCancelling: true, cancelError: null });

    try {
      const backend = await getBackend();
      await backend.cancelAllOrders();
      set({ isCancelling: false });
      return true;
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      set({ isCancelling: false, cancelError: errorMessage });
      return false;
    }
  },

  // Cancel all orders for a market
  cancelMarketOrders: async (marketId: string) => {
    set({ isCancelling: true, cancelError: null });

    try {
      const backend = await getBackend();
      await backend.cancelMarketOrders(marketId);
      set({ isCancelling: false });
      return true;
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      set({ isCancelling: false, cancelError: errorMessage });
      return false;
    }
  },

  // Clear errors
  clearErrors: () => set({ orderError: null, cancelError: null }),

  // Clear last order result
  clearOrderResult: () => set({ lastOrderResult: null }),
}));
