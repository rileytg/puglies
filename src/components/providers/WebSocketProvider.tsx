// AIDEV-NOTE: Provider component that listens to Tauri events and updates stores
import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { useWebSocketStore } from "@/stores/websocket";
import { useOrderBookStore } from "@/stores/orderbook";
import type {
  ConnectionStatus,
  OrderBookSnapshot,
  OrderBookDelta,
} from "@/lib/types";

interface WebSocketProviderProps {
  children: React.ReactNode;
}

export function WebSocketProvider({ children }: WebSocketProviderProps) {
  const setStatus = useWebSocketStore((state) => state.setStatus);
  const setLastUpdate = useWebSocketStore((state) => state.setLastUpdate);
  const setSnapshot = useOrderBookStore((state) => state.setSnapshot);
  const applyDelta = useOrderBookStore((state) => state.applyDelta);

  useEffect(() => {
    const unlisteners: (() => void)[] = [];

    const setupListeners = async () => {
      // Connection status updates
      const unlistenStatus = await listen<ConnectionStatus>(
        "connection_status",
        (event) => {
          setStatus(event.payload);
        }
      );
      unlisteners.push(unlistenStatus);

      // Order book snapshots
      const unlistenSnapshot = await listen<OrderBookSnapshot>(
        "orderbook_snapshot",
        (event) => {
          setSnapshot(event.payload.asset_id, event.payload);
          setLastUpdate(new Date());
        }
      );
      unlisteners.push(unlistenSnapshot);

      // Order book deltas
      const unlistenDelta = await listen<OrderBookDelta>(
        "orderbook_delta",
        (event) => {
          applyDelta(event.payload);
          setLastUpdate(new Date());
        }
      );
      unlisteners.push(unlistenDelta);

      // Price updates (RTDS)
      const unlistenPrice = await listen("price_update", () => {
        setLastUpdate(new Date());
        // Price updates can be handled by individual components via useTauriEvent
      });
      unlisteners.push(unlistenPrice);

      // Trade updates (RTDS)
      const unlistenTrade = await listen("trade_update", () => {
        setLastUpdate(new Date());
        // Trade updates can be handled by individual components
      });
      unlisteners.push(unlistenTrade);

      // CLOB trades
      const unlistenClobTrade = await listen("clob_trade", () => {
        setLastUpdate(new Date());
        // CLOB trades can be handled by individual components
      });
      unlisteners.push(unlistenClobTrade);
    };

    setupListeners();

    return () => {
      unlisteners.forEach((unlisten) => unlisten());
    };
  }, [setStatus, setLastUpdate, setSnapshot, applyDelta]);

  return <>{children}</>;
}
