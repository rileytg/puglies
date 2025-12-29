// AIDEV-NOTE: Event subscription abstraction - Tauri listen() vs SSE/WebSocket

export type EventCallback<T> = (payload: T) => void;
export type UnsubscribeFn = () => void;

/**
 * Event subscriber interface for real-time updates.
 * Abstracts Tauri events vs SSE/WebSocket for web.
 */
export interface EventSubscriber {
  /**
   * Subscribe to a named event.
   * Returns an unsubscribe function.
   */
  subscribe<T>(eventName: string, callback: EventCallback<T>): Promise<UnsubscribeFn>;

  /**
   * Subscribe to multiple events at once.
   */
  subscribeMany(
    events: Array<{ name: string; callback: EventCallback<unknown> }>
  ): Promise<UnsubscribeFn>;
}

// Event names as constants for type safety
export const EVENTS = {
  CONNECTION_STATUS: "connection_status",
  ORDERBOOK_SNAPSHOT: "orderbook_snapshot",
  ORDERBOOK_DELTA: "orderbook_delta",
  PRICE_UPDATE: "price_update",
  TRADE_UPDATE: "trade_update",
  CLOB_TRADE: "clob_trade",
} as const;

export type EventName = (typeof EVENTS)[keyof typeof EVENTS];
