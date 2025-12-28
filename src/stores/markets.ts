import { create } from "zustand";
import type { Market, Event } from "@/lib/types";

interface MarketsState {
  markets: Market[];
  events: Event[];
  selectedMarket: Market | null;
  isLoading: boolean;
  error: string | null;
  searchQuery: string;

  // Actions
  setMarkets: (markets: Market[]) => void;
  setEvents: (events: Event[]) => void;
  setSelectedMarket: (market: Market | null) => void;
  setLoading: (loading: boolean) => void;
  setError: (error: string | null) => void;
  setSearchQuery: (query: string) => void;
  updateMarketPrice: (conditionId: string, tokenId: string, price: number) => void;
}

export const useMarketsStore = create<MarketsState>((set) => ({
  markets: [],
  events: [],
  selectedMarket: null,
  isLoading: false,
  error: null,
  searchQuery: "",

  setMarkets: (markets) => set({ markets }),
  setEvents: (events) => set({ events }),
  setSelectedMarket: (market) => set({ selectedMarket: market }),
  setLoading: (isLoading) => set({ isLoading }),
  setError: (error) => set({ error }),
  setSearchQuery: (searchQuery) => set({ searchQuery }),

  updateMarketPrice: (conditionId, tokenId, price) =>
    set((state) => ({
      markets: state.markets.map((market) => {
        if (market.condition_id !== conditionId) return market;
        return {
          ...market,
          tokens: market.tokens.map((token) =>
            token.token_id === tokenId ? { ...token, price } : token
          ),
        };
      }),
    })),
}));
