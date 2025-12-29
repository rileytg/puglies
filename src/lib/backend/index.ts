// AIDEV-NOTE: Backend provider - auto-detects Tauri vs Web environment

import type { BackendAdapter } from "./types";
import type { EventSubscriber } from "./events";

// Singleton instances (lazy-loaded)
let backendAdapter: BackendAdapter | null = null;
let eventSubscriber: EventSubscriber | null = null;

/**
 * Detect if running inside Tauri
 */
export function isTauri(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

/**
 * Get the backend adapter (singleton).
 * Auto-selects Tauri or Web based on environment.
 */
export async function getBackend(): Promise<BackendAdapter> {
  if (backendAdapter) return backendAdapter;

  if (isTauri()) {
    const { TauriBackendAdapter } = await import("./tauri-adapter");
    backendAdapter = new TauriBackendAdapter();
  } else {
    const { WebBackendAdapter } = await import("./web-adapter");
    backendAdapter = new WebBackendAdapter();
  }

  return backendAdapter;
}

/**
 * Get the event subscriber (singleton).
 * Auto-selects Tauri or Web based on environment.
 */
export async function getEventSubscriber(): Promise<EventSubscriber> {
  if (eventSubscriber) return eventSubscriber;

  if (isTauri()) {
    const { TauriEventSubscriber } = await import("./tauri-adapter");
    eventSubscriber = new TauriEventSubscriber();
  } else {
    const { WebEventSubscriber } = await import("./web-adapter");
    eventSubscriber = new WebEventSubscriber();
  }

  return eventSubscriber;
}

/**
 * Reset singletons (for testing only).
 * @internal
 */
export function _resetForTesting(): void {
  backendAdapter = null;
  eventSubscriber = null;
}

/**
 * Inject a custom backend adapter (for testing).
 * @internal
 */
export function _setBackendForTesting(adapter: BackendAdapter): void {
  backendAdapter = adapter;
}

/**
 * Inject a custom event subscriber (for testing).
 * @internal
 */
export function _setEventSubscriberForTesting(subscriber: EventSubscriber): void {
  eventSubscriber = subscriber;
}

// Re-export types and constants
export type { BackendAdapter } from "./types";
export type { EventSubscriber, EventCallback, UnsubscribeFn, EventName } from "./events";
export { EVENTS } from "./events";
