// AIDEV-NOTE: Hook for subscribing to backend events (Tauri or Web)
import { useEffect, useRef } from "react";
import { getEventSubscriber, type EventCallback, type UnsubscribeFn } from "@/lib/backend";

/**
 * Subscribe to a backend event.
 * Works with Tauri (listen) or Web (SSE) automatically.
 * Handles cleanup on unmount.
 */
export function useBackendEvent<T>(
  eventName: string,
  callback: EventCallback<T>
): void {
  const callbackRef = useRef(callback);
  callbackRef.current = callback;

  useEffect(() => {
    let unsubscribe: UnsubscribeFn | undefined;

    const setup = async () => {
      const subscriber = await getEventSubscriber();
      unsubscribe = await subscriber.subscribe<T>(eventName, (payload) => {
        callbackRef.current(payload);
      });
    };

    setup();

    return () => {
      unsubscribe?.();
    };
  }, [eventName]);
}

/**
 * Subscribe to multiple backend events at once.
 * Works with Tauri (listen) or Web (SSE) automatically.
 */
export function useBackendEvents(
  events: Array<{ name: string; callback: (payload: unknown) => void }>
): void {
  const eventsRef = useRef(events);
  eventsRef.current = events;

  useEffect(() => {
    let unsubscribe: UnsubscribeFn | undefined;

    const setup = async () => {
      const subscriber = await getEventSubscriber();
      unsubscribe = await subscriber.subscribeMany(
        eventsRef.current.map(({ name, callback }) => ({
          name,
          callback,
        }))
      );
    };

    setup();

    return () => {
      unsubscribe?.();
    };
  }, []);
}
