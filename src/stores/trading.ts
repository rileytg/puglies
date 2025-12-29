// AIDEV-NOTE: Trading state store - manages order placement and cancellation

import { create } from "zustand";
import type { OrderParams, PlaceOrderResult } from "@/lib/types";
import * as tauri from "@/lib/tauri";

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
      const result = await tauri.placeOrder(params, privateKey);
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
      const result = await tauri.cancelOrder(orderId);
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
      await tauri.cancelAllOrders();
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
      await tauri.cancelMarketOrders(marketId);
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
