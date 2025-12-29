// AIDEV-NOTE: WebSocket state management - tracks connection status and provides connect/disconnect actions
import { create } from "zustand";
import type { ConnectionStatus, ConnectionStateValue } from "@/lib/types";
import { getBackend } from "@/lib/backend";

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
      const backend = await getBackend();
      await backend.connectRtds(markets);
    } finally {
      set({ isConnecting: false });
    }
  },

  disconnectFromRtds: async () => {
    const backend = await getBackend();
    await backend.disconnectRtds();
  },

  connectToClob: async (tokenIds) => {
    set({ isConnecting: true });
    try {
      const backend = await getBackend();
      await backend.connectClob(tokenIds);
    } finally {
      set({ isConnecting: false });
    }
  },

  disconnectFromClob: async () => {
    const backend = await getBackend();
    await backend.disconnectClob();
  },

  refreshStatus: async () => {
    const backend = await getBackend();
    const status = await backend.getConnectionStatus();
    set({ status });
  },
}));
