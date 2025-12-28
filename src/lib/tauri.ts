import { invoke } from "@tauri-apps/api/core";
import type { Market, Event, ConnectionStatus } from "./types";

// AIDEV-NOTE: Tauri command wrappers - keep in sync with src-tauri/src/commands/

// Market commands
export async function getMarkets(
  query?: string,
  limit?: number,
  offset?: number
): Promise<Market[]> {
  return invoke("get_markets", { query, limit, offset });
}

export async function getMarket(conditionId: string): Promise<Market> {
  return invoke("get_market", { conditionId });
}

export async function getEvents(limit?: number): Promise<Event[]> {
  return invoke("get_events", { limit });
}

export async function searchMarkets(query: string): Promise<Market[]> {
  return invoke("search_markets", { query });
}

// WebSocket commands
export async function connectRtds(markets: string[]): Promise<void> {
  return invoke("connect_rtds", { markets });
}

export async function disconnectRtds(): Promise<void> {
  return invoke("disconnect_rtds");
}

export async function connectClob(tokenIds: string[]): Promise<void> {
  return invoke("connect_clob", { tokenIds });
}

export async function disconnectClob(): Promise<void> {
  return invoke("disconnect_clob");
}

export async function getConnectionStatus(): Promise<ConnectionStatus> {
  return invoke("get_connection_status");
}
