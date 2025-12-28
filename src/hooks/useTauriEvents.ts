// AIDEV-NOTE: Hook for subscribing to Tauri events from Rust backend
import { useEffect, useRef } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

type EventCallback<T> = (payload: T) => void;

/**
 * Subscribe to a Tauri event from the Rust backend
 * Automatically handles cleanup on unmount
 */
export function useTauriEvent<T>(
  eventName: string,
  callback: EventCallback<T>
): void {
  const callbackRef = useRef(callback);
  callbackRef.current = callback;

  useEffect(() => {
    let unlisten: UnlistenFn | undefined;

    const setupListener = async () => {
      unlisten = await listen<T>(eventName, (event) => {
        callbackRef.current(event.payload);
      });
    };

    setupListener();

    return () => {
      if (unlisten) {
        unlisten();
      }
    };
  }, [eventName]);
}

/**
 * Subscribe to multiple Tauri events at once
 */
export function useTauriEvents(
  events: Array<{ name: string; callback: (payload: unknown) => void }>
): void {
  const eventsRef = useRef(events);
  eventsRef.current = events;

  useEffect(() => {
    const unlisteners: UnlistenFn[] = [];

    const setupListeners = async () => {
      for (const { name, callback } of eventsRef.current) {
        const unlisten = await listen(name, (event) => {
          callback(event.payload);
        });
        unlisteners.push(unlisten);
      }
    };

    setupListeners();

    return () => {
      unlisteners.forEach((unlisten) => unlisten());
    };
  }, []);
}
