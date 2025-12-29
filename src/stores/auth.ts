// AIDEV-NOTE: Auth state store - manages authentication status and portfolio data

import { create } from "zustand";
import type { AuthStatus, Balance, Position, Order } from "@/lib/types";
import * as tauri from "@/lib/tauri";

interface AuthState {
  // Auth status
  status: AuthStatus;
  isLoading: boolean;
  error: string | null;

  // Polymarket address (for positions - may differ from signing address)
  polymarketAddress: string | null;

  // Portfolio data
  balance: Balance | null;
  positions: Position[];
  orders: Order[];
  portfolioLoading: boolean;

  // Actions
  checkAuthStatus: () => Promise<void>;
  login: (privateKey: string) => Promise<boolean>;
  logout: () => Promise<void>;
  setPolymarketAddress: (address: string) => void;
  fetchPortfolio: () => Promise<void>;
  clearError: () => void;
}

export const useAuthStore = create<AuthState>((set, get) => ({
  // Initial state
  status: { isAuthenticated: false },
  isLoading: false,
  error: null,
  polymarketAddress: null,
  balance: null,
  positions: [],
  orders: [],
  portfolioLoading: false,

  // Check current auth status (on app load)
  checkAuthStatus: async () => {
    try {
      const status = await tauri.getAuthStatus();
      set({
        status,
        polymarketAddress: status.polymarketAddress || null,
      });

      // If authenticated, fetch portfolio data
      if (status.isAuthenticated) {
        get().fetchPortfolio();
      }
    } catch (err) {
      console.error("Failed to check auth status:", err);
    }
  },

  // Login with private key
  login: async (privateKey: string) => {
    set({ isLoading: true, error: null });

    try {
      const status = await tauri.login(privateKey);
      set({ status, isLoading: false });

      // Fetch portfolio after successful login
      if (status.isAuthenticated) {
        get().fetchPortfolio();
      }

      return true;
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      set({ isLoading: false, error: errorMessage });
      return false;
    }
  },

  // Logout
  logout: async () => {
    set({ isLoading: true, error: null });

    try {
      const status = await tauri.logout();
      set({
        status,
        isLoading: false,
        balance: null,
        positions: [],
        orders: [],
      });
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      set({ isLoading: false, error: errorMessage });
    }
  },

  // Set Polymarket address for fetching positions (persists to database)
  setPolymarketAddress: async (address: string) => {
    set({ polymarketAddress: address, portfolioLoading: true });

    // Persist to backend database
    try {
      await tauri.setPolymarketAddress(address);
    } catch (err) {
      console.error("Failed to persist polymarket address:", err);
    }

    // Fetch positions immediately (public API, no auth needed)
    try {
      console.log("Fetching positions for:", address);
      const positions = await tauri.getPositions(address);
      console.log("Got positions:", positions);
      set({ positions, portfolioLoading: false });
    } catch (err) {
      console.error("Failed to fetch positions:", err);
      set({ portfolioLoading: false });
    }
  },

  // Fetch portfolio data (balance, positions, orders)
  fetchPortfolio: async () => {
    const { status, polymarketAddress } = get();

    set({ portfolioLoading: true });

    try {
      // Fetch balance and orders if authenticated
      let balance: Balance | null = null;
      let orders: Order[] = [];

      if (status.isAuthenticated) {
        try {
          // AIDEV-NOTE: Fetch balance and orders separately - orders endpoint can hang
          const balancePromise = tauri.getBalance();
          const ordersPromise = tauri.getOrders();

          try {
            balance = await balancePromise;
          } catch (balanceErr) {
            console.error("Balance fetch failed:", balanceErr);
          }

          try {
            // AIDEV-NOTE: Orders endpoint can hang - add timeout to prevent blocking
            const ordersTimeout = new Promise<Order[]>((_, reject) =>
              setTimeout(() => reject(new Error("Orders fetch timed out")), 5000)
            );
            orders = await Promise.race([ordersPromise, ordersTimeout]);
          } catch (ordersErr) {
            console.warn("Orders fetch failed or timed out:", ordersErr);
          }
        } catch (err) {
          console.error("Failed to fetch balance/orders:", err);
        }
      }

      // Fetch positions if we have a Polymarket address (public API)
      let positions: Position[] = [];
      if (polymarketAddress) {
        try {
          positions = await tauri.getPositions(polymarketAddress);
        } catch (err) {
          console.error("Failed to fetch positions:", err);
        }
      }

      set({
        balance,
        positions,
        orders,
        portfolioLoading: false,
      });
    } catch (err) {
      console.error("Failed to fetch portfolio:", err);
      set({ portfolioLoading: false });
    }
  },

  // Clear error
  clearError: () => set({ error: null }),
}));
