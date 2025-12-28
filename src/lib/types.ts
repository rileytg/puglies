// Market types from Gamma API
export interface Market {
  condition_id: string;
  question_id: string;
  question: string;
  description: string;
  market_slug: string;
  end_date_iso: string;
  game_start_time?: string;
  seconds_delay?: number;
  fpmm?: string;
  maker_base_fee?: number;
  taker_base_fee?: number;
  notifications_enabled?: boolean;
  neg_risk?: boolean;
  neg_risk_market_id?: string;
  neg_risk_request_id?: string;
  icon?: string;
  image?: string;
  rewards?: MarketRewards;
  tokens: Token[];
  tags?: string[];
  active: boolean;
  closed: boolean;
  archived: boolean;
  accepting_orders: boolean;
  accepting_order_timestamp?: string;
  minimum_order_size: number;
  minimum_tick_size: number;
  volume: string;
  volume_num: number;
  liquidity: string;
  liquidity_num: number;
  spread: number;
}

export interface Token {
  token_id: string;
  outcome: string;
  price: number;
  winner?: boolean;
}

export interface MarketRewards {
  min_size: number;
  max_spread: number;
  event_start_date?: string;
  event_end_date?: string;
  in_game_multiplier?: number;
  rewards_daily_rate?: number;
  rewards_min_size?: number;
  rewards_max_spread?: number;
}

export interface Event {
  id: string;
  ticker: string;
  slug: string;
  title: string;
  description: string;
  start_date?: string;
  end_date?: string;
  image?: string;
  icon?: string;
  active: boolean;
  closed: boolean;
  archived: boolean;
  new: boolean;
  featured: boolean;
  restricted: boolean;
  markets: Market[];
  total_volume: number;
  total_liquidity: number;
}

// CLOB types
export interface OrderBook {
  market: string;
  asset_id: string;
  hash: string;
  bids: OrderBookLevel[];
  asks: OrderBookLevel[];
  timestamp: string;
}

export interface OrderBookLevel {
  price: string;
  size: string;
}

export interface Position {
  asset: string;
  condition_id: string;
  size: string;
  avg_price: string;
  realized_pnl: string;
  cur_price: string;
}

export interface Order {
  id: string;
  market: string;
  asset_id: string;
  side: "BUY" | "SELL";
  type: "GTC" | "FOK" | "GTD";
  original_size: string;
  size_matched: string;
  price: string;
  status: "live" | "matched" | "cancelled";
  created_at: string;
  expiration?: string;
}

// WebSocket message types (from Rust backend)
export interface PriceUpdate {
  msg_type?: string;
  market: string;
  price: number;
  timestamp?: number;
}

export interface TradeUpdate {
  msg_type?: string;
  market: string;
  price: number;
  size: number;
  side: string;
  timestamp?: number;
}

export interface OrderBookSnapshot {
  event_type?: string;
  asset_id: string;
  market?: string;
  hash?: string;
  timestamp?: number;
  bids: OrderBookLevel[];
  asks: OrderBookLevel[];
}

export interface OrderBookDelta {
  event_type?: string;
  asset_id: string;
  market?: string;
  side: string;
  price: string;
  size: string;
  timestamp?: number;
}

export interface ClobTrade {
  event_type?: string;
  asset_id: string;
  market?: string;
  price: string;
  size: string;
  side: string;
  timestamp?: number;
  trade_id?: string;
}

// App state types
export type ConnectionStateValue = "disconnected" | "connecting" | "connected" | "reconnecting" | "failed";

export interface ConnectionStatus {
  clob: ConnectionStateValue;
  rtds: ConnectionStateValue;
}
