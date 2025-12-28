// AIDEV-NOTE: WebSocket state management - tracks connection status and provides connect/disconnect actions
import { create } from "zustand";
import type { ConnectionStatus, ConnectionStateValue } from "@/lib/types";
import {
  connectRtds,
  disconnectRtds,
  connectClob,
  disconnectClob,
  getConnectionStatus,
} from "@/lib/tauri";

interface WebSocketState {
  status: ConnectionStatus;
  lastUpdate: Date | null;
  isConnecting: boolean;

  // Actions
  setStatus: (status: ConnectionStatus) => void;
  setClobStatus: (status: ConnectionStateValue) => void;
  setRtdsStatus: (status: ConnectionStateValue) => void;
  setLastUpdate: (date: Date) => void;

  // Commands
  connectToRtds: (markets: string[]) => Promise<void>;
  disconnectFromRtds: () => Promise<void>;
  connectToClob: (tokenIds: string[]) => Promise<void>;
  disconnectFromClob: () => Promise<void>;
  refreshStatus: () => Promise<void>;
}

export const useWebSocketStore = create<WebSocketState>((set) => ({
  status: {
    clob: "disconnected",
    rtds: "disconnected",
  },
  lastUpdate: null,
  isConnecting: false,

  setStatus: (status) => set({ status }),

  setClobStatus: (clob) =>
    set((state) => ({ status: { ...state.status, clob } })),

  setRtdsStatus: (rtds) =>
    set((state) => ({ status: { ...state.status, rtds } })),

  setLastUpdate: (lastUpdate) => set({ lastUpdate }),

  connectToRtds: async (markets) => {
    set({ isConnecting: true });
    try {
      await connectRtds(markets);
    } finally {
      set({ isConnecting: false });
    }
  },

  disconnectFromRtds: async () => {
    await disconnectRtds();
  },

  connectToClob: async (tokenIds) => {
    set({ isConnecting: true });
    try {
      await connectClob(tokenIds);
    } finally {
      set({ isConnecting: false });
    }
  },

  disconnectFromClob: async () => {
    await disconnectClob();
  },

  refreshStatus: async () => {
    const status = await getConnectionStatus();
    set({ status });
  },
}));
