// AIDEV-NOTE: Provider component that listens to backend events and updates stores
import { useEffect } from "react";
import { getEventSubscriber, EVENTS } from "@/lib/backend";
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
    let unsubscribe: (() => void) | undefined;

    const setupListeners = async () => {
      const subscriber = await getEventSubscriber();

      unsubscribe = await subscriber.subscribeMany([
        // Connection status updates
        {
          name: EVENTS.CONNECTION_STATUS,
          callback: (payload) => {
            setStatus(payload as ConnectionStatus);
          },
        },
        // Order book snapshots
        {
          name: EVENTS.ORDERBOOK_SNAPSHOT,
          callback: (payload) => {
            const snapshot = payload as OrderBookSnapshot;
            setSnapshot(snapshot.asset_id, snapshot);
            setLastUpdate(new Date());
          },
        },
        // Order book deltas
        {
          name: EVENTS.ORDERBOOK_DELTA,
          callback: (payload) => {
            applyDelta(payload as OrderBookDelta);
            setLastUpdate(new Date());
          },
        },
        // Price updates (RTDS)
        {
          name: EVENTS.PRICE_UPDATE,
          callback: () => {
            setLastUpdate(new Date());
            // Price updates can be handled by individual components via useBackendEvent
          },
        },
        // Trade updates (RTDS)
        {
          name: EVENTS.TRADE_UPDATE,
          callback: () => {
            setLastUpdate(new Date());
            // Trade updates can be handled by individual components
          },
        },
        // CLOB trades
        {
          name: EVENTS.CLOB_TRADE,
          callback: () => {
            setLastUpdate(new Date());
            // CLOB trades can be handled by individual components
          },
        },
      ]);
    };

    setupListeners();

    return () => {
      unsubscribe?.();
    };
  }, [setStatus, setLastUpdate, setSnapshot, applyDelta]);

  return <>{children}</>;
}
