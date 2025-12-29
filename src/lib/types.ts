// Market types from Gamma API
// AIDEV-NOTE: id is internal Gamma API ID (numeric), condition_id is on-chain ID (hex)
export interface Market {
  id: string;
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
  conditionId: string;
  size: number;
  avgPrice: number;
  initialValue: number;
  currentValue: number;
  cashPnl: number;
  percentPnl: number;
  curPrice: number;
  title: string;
  outcome: string;
  proxyWallet: string;
}

export interface Order {
  id: string;
  market: string;
  asset: string;
  side: string;
  originalSize: string;
  sizeMatched: string;
  price: string;
  status: string;
  orderType: string;
  createdAt: string;
}

// Auth types
export interface AuthStatus {
  isAuthenticated: boolean;
  address?: string;
  polymarketAddress?: string;
}

export interface Balance {
  balance: string;
  allowances: Record<string, string>;
}

// Trading types
export type OrderSide = "Buy" | "Sell";
export type OrderTimeInForce = "Gtc" | "Fok" | "Gtd";

// Order parameters from user input
export interface OrderParams {
  tokenId: string;
  side: OrderSide;
  price: number;      // 0.0-1.0
  size: number;       // Number of shares
  orderType: OrderTimeInForce;
  expirationSecs?: number;
}

// Order placement result
export interface PlaceOrderResult {
  success: boolean;
  errorMsg?: string;
  orderId?: string;
  orderHashes?: string[];
  status?: string;
}

// Cancel result
export interface CancelResult {
  canceled: string[];
  notCanceled: Record<string, string>;
}

// WebSocket message types (from Rust backend)
export interface PriceUpdate {
  msg_type?: string;
  market: string;
  asset_id?: string;  // Token ID for matching specific outcomes
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

// Price history types
// AIDEV-NOTE: Matches PricePoint from Rust clob.rs
export interface PricePoint {
  t: number;  // Unix timestamp (seconds)
  p: number;  // Price (0.0 - 1.0)
}

export interface PriceHistoryParams {
  tokenId: string;
  interval?: "1h" | "6h" | "1d" | "1w" | "max";
  fidelity?: number;  // Resolution in minutes
}

export interface PriceHistoryResult {
  history: PricePoint[];
  cachedCount: number;
  fetchedCount: number;
}
