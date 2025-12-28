import { create } from "zustand";
import type { ConnectionStatus } from "@/lib/types";

interface WebSocketState {
  status: ConnectionStatus;
  lastUpdate: Date | null;

  // Actions
  setClobStatus: (status: ConnectionStatus["clob"]) => void;
  setRtdsStatus: (status: ConnectionStatus["rtds"]) => void;
  setLastUpdate: (date: Date) => void;
}

export const useWebSocketStore = create<WebSocketState>((set) => ({
  status: {
    clob: "disconnected",
    rtds: "disconnected",
  },
  lastUpdate: null,

  setClobStatus: (clob) =>
    set((state) => ({ status: { ...state.status, clob } })),

  setRtdsStatus: (rtds) =>
    set((state) => ({ status: { ...state.status, rtds } })),

  setLastUpdate: (lastUpdate) => set({ lastUpdate }),
}));
