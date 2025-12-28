import { invoke } from "@tauri-apps/api/core";
import type { Market, Event } from "./types";

// AIDEV-NOTE: Tauri command wrappers - keep in sync with src-tauri/src/commands/

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
