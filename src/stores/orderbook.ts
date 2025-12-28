// AIDEV-NOTE: Order book state management - stores order book snapshots and applies deltas
import { create } from "zustand";
import type { OrderBookLevel, OrderBookSnapshot, OrderBookDelta } from "@/lib/types";

interface OrderBookData {
  bids: OrderBookLevel[];
  asks: OrderBookLevel[];
  lastUpdate: number | null;
}

interface OrderBookState {
  // Map of asset_id -> order book data
  orderBooks: Map<string, OrderBookData>;

  // Actions
  setSnapshot: (assetId: string, snapshot: OrderBookSnapshot) => void;
  applyDelta: (delta: OrderBookDelta) => void;
  getOrderBook: (assetId: string) => OrderBookData | undefined;
  clearOrderBook: (assetId: string) => void;
  clearAll: () => void;
}

export const useOrderBookStore = create<OrderBookState>((set, get) => ({
  orderBooks: new Map(),

  setSnapshot: (assetId, snapshot) => {
    set((state) => {
      const newBooks = new Map(state.orderBooks);
      newBooks.set(assetId, {
        bids: snapshot.bids,
        asks: snapshot.asks,
        lastUpdate: snapshot.timestamp || Date.now(),
      });
      return { orderBooks: newBooks };
    });
  },

  applyDelta: (delta) => {
    set((state) => {
      const current = state.orderBooks.get(delta.asset_id);
      if (!current) return state;

      const newBooks = new Map(state.orderBooks);
      const levels = delta.side === "BUY" ? [...current.bids] : [...current.asks];

      // Find existing level at this price
      const existingIdx = levels.findIndex((l) => l.price === delta.price);
      const newSize = parseFloat(delta.size);

      if (newSize === 0) {
        // Remove level if size is 0
        if (existingIdx !== -1) {
          levels.splice(existingIdx, 1);
        }
      } else if (existingIdx !== -1) {
        // Update existing level
        levels[existingIdx] = { price: delta.price, size: delta.size };
      } else {
        // Insert new level in sorted order
        const newLevel = { price: delta.price, size: delta.size };
        const insertIdx = levels.findIndex((l) => {
          const existingPrice = parseFloat(l.price);
          const newPrice = parseFloat(delta.price);
          // Bids: descending order, Asks: ascending order
          return delta.side === "BUY" ? newPrice > existingPrice : newPrice < existingPrice;
        });
        if (insertIdx === -1) {
          levels.push(newLevel);
        } else {
          levels.splice(insertIdx, 0, newLevel);
        }
      }

      const updatedBook = {
        ...current,
        lastUpdate: delta.timestamp || Date.now(),
        ...(delta.side === "BUY" ? { bids: levels } : { asks: levels }),
      };

      newBooks.set(delta.asset_id, updatedBook);
      return { orderBooks: newBooks };
    });
  },

  getOrderBook: (assetId) => {
    return get().orderBooks.get(assetId);
  },

  clearOrderBook: (assetId) => {
    set((state) => {
      const newBooks = new Map(state.orderBooks);
      newBooks.delete(assetId);
      return { orderBooks: newBooks };
    });
  },

  clearAll: () => {
    set({ orderBooks: new Map() });
  },
}));
