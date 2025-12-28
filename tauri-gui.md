# Polymarket GUI (`plgui`) - Tauri Implementation Plan

## Overview
A cross-platform desktop GUI for Polymarket trading using Tauri (Rust backend + React frontend), with real-time WebSocket data and full trading capabilities.

## Tech Stack

### Backend (Rust/Tauri)
- **Framework**: Tauri v2
- **WebSockets**: tokio-tungstenite
- **Crypto**: ethers-rs (EIP-712 signing, ECDSA)
- **HTTP Client**: reqwest
- **Serialization**: serde + serde_json
- **Async Runtime**: tokio
- **Secure Storage**: keyring-rs

### Frontend (React + TypeScript)
- **Framework**: React 18 + TypeScript
- **State Management**: Zustand (lightweight, works well with Tauri events)
- **Styling**: Tailwind CSS
- **Charts**: Lightweight Charts (TradingView) or Recharts
- **UI Components**: shadcn/ui (Radix primitives)
- **Build Tool**: Vite

## Project Structure
```
plgui/
├── src-tauri/
│   ├── src/
│   │   ├── main.rs                 # Tauri entry point
│   │   ├── lib.rs                  # Library exports
│   │   ├── commands/               # Tauri command handlers
│   │   │   ├── mod.rs
│   │   │   ├── auth.rs             # Login, API key derivation
│   │   │   ├── markets.rs          # Market queries
│   │   │   ├── orders.rs           # Order placement/cancellation
│   │   │   └── portfolio.rs        # Positions, balances
│   │   ├── websocket/              # WebSocket connections
│   │   │   ├── mod.rs
│   │   │   ├── manager.rs          # Connection manager
│   │   │   ├── clob.rs             # CLOB WS handler
│   │   │   └── rtds.rs             # RTDS WS handler
│   │   ├── auth/                   # Authentication
│   │   │   ├── mod.rs
│   │   │   ├── eip712.rs           # L1 signing
│   │   │   └── hmac.rs             # L2 HMAC signing
│   │   ├── api/                    # REST API clients
│   │   │   ├── mod.rs
│   │   │   ├── clob.rs             # CLOB REST client
│   │   │   └── gamma.rs            # Gamma API client
│   │   ├── state.rs                # Application state
│   │   └── error.rs                # Error types
│   ├── Cargo.toml
│   └── tauri.conf.json
├── src/                            # React frontend
│   ├── main.tsx                    # React entry
│   ├── App.tsx                     # Root component + routing
│   ├── stores/                     # Zustand stores
│   │   ├── auth.ts
│   │   ├── markets.ts
│   │   ├── orderbook.ts
│   │   ├── portfolio.ts
│   │   └── websocket.ts
│   ├── components/                 # UI components
│   │   ├── layout/
│   │   │   ├── Sidebar.tsx
│   │   │   ├── Header.tsx
│   │   │   └── StatusBar.tsx
│   │   ├── markets/
│   │   │   ├── MarketList.tsx
│   │   │   ├── MarketCard.tsx
│   │   │   ├── MarketDetail.tsx
│   │   │   └── MarketSearch.tsx
│   │   ├── trading/
│   │   │   ├── OrderBook.tsx
│   │   │   ├── OrderForm.tsx
│   │   │   ├── PriceChart.tsx
│   │   │   └── TradeHistory.tsx
│   │   ├── portfolio/
│   │   │   ├── PositionsList.tsx
│   │   │   ├── OrdersList.tsx
│   │   │   └── BalanceSummary.tsx
│   │   └── common/
│   │       ├── Modal.tsx
│   │       ├── Button.tsx
│   │       ├── Input.tsx
│   │       └── Toast.tsx
│   ├── hooks/                      # Custom React hooks
│   │   ├── useTauriEvents.ts       # Listen to Rust events
│   │   ├── useWebSocket.ts
│   │   └── useMarket.ts
│   ├── lib/                        # Utilities
│   │   ├── tauri.ts                # Tauri invoke wrappers
│   │   ├── format.ts               # Number/date formatting
│   │   └── types.ts                # Shared types
│   └── styles/
│       └── globals.css             # Tailwind imports
├── package.json
├── vite.config.ts
├── tailwind.config.js
└── tsconfig.json
```

## WebSocket Endpoints
1. **CLOB WS** (`wss://ws-subscriptions-clob.polymarket.com/ws/market`) - Order book, user orders
2. **RTDS** (`wss://ws-live-data.polymarket.com`) - Market activity, trades

## Authentication Flow
1. User provides private key (secure input, never persisted in plaintext)
2. Check keyring for existing API credentials
3. If none: derive via L1 EIP-712 signing → POST `/auth/api-key`
4. Store credentials in OS keyring (keyring-rs)
5. Use L2 HMAC signing for all trading requests

---

## Implementation Phases

### Phase 1: Project Setup + Static Market Browser
**Goal**: Scaffold Tauri app, browse markets (no auth required)

#### Rust Backend
1. Initialize Tauri v2 project with React template
2. Set up Cargo workspace structure
3. Implement Gamma API client (`/markets`, `/events`)
4. Create Tauri commands:
   - `get_markets(query, limit, offset)` → Market list
   - `get_market(condition_id)` → Market details
   - `get_events()` → Event list
5. Add error handling with thiserror

#### React Frontend
1. Set up Vite + React + TypeScript
2. Install and configure Tailwind CSS + shadcn/ui
3. Create base layout (Sidebar, Header, Main content area)
4. Build market list with search/filter
5. Build market detail view (static info only)
6. Create Zustand store for markets
7. Wire up Tauri invoke calls

#### Deliverables
- Browsable market list with search
- Market detail page with outcomes, description
- Responsive layout with sidebar navigation

---

### Phase 2: Real-Time WebSocket Data
**Goal**: Live prices, order book, price charts

#### Rust Backend
1. Implement WebSocket manager with tokio-tungstenite
2. Connect to RTDS for market activity
3. Connect to CLOB WS for order book depth
4. Implement reconnection with exponential backoff
5. Create Tauri event emitters:
   - `price_update` → Frontend
   - `orderbook_update` → Frontend
   - `trade_update` → Frontend
   - `connection_status` → Frontend
6. Add message throttling for high-frequency updates

#### React Frontend
1. Create WebSocket status indicator in header
2. Implement `useTauriEvents` hook for event subscription
3. Build real-time price display on market cards
4. Build order book visualization (depth chart + table)
5. Integrate TradingView Lightweight Charts for price history
6. Create Zustand stores for orderbook and price data
7. Add loading states and error handling

#### Deliverables
- Live price updates on all market cards
- Real-time order book depth visualization
- Interactive price charts (1h, 24h, 7d, 30d)
- Connection status indicator with auto-reconnect

---

### Phase 3: Authentication + Portfolio
**Goal**: Login with private key, view positions and orders

#### Rust Backend
1. Implement EIP-712 typed data signing (ethers-rs)
2. Implement HMAC-SHA256 for L2 auth headers
3. Create API key derivation flow:
   - Generate nonce
   - Sign with EIP-712
   - POST to `/auth/api-key`
4. Integrate keyring-rs for secure credential storage
5. Implement CLOB REST endpoints:
   - `GET /positions` → User positions
   - `GET /orders` → Active orders
   - `GET /balance` → Available balance
6. Subscribe to user channel on CLOB WebSocket
7. Create Tauri commands:
   - `login(private_key)` → Derive/load credentials
   - `logout()` → Clear session
   - `get_positions()` → Position list
   - `get_orders()` → Order list
   - `get_balance()` → Balance info

#### React Frontend
1. Create login modal with secure private key input
2. Add auth state to Zustand store
3. Build portfolio dashboard:
   - Total value display
   - Unrealized P&L
   - Position list with market links
4. Build active orders list with cancel button
5. Add protected route handling
6. Create user menu with logout option

#### Deliverables
- Secure login flow with private key
- Portfolio overview with positions and P&L
- Active orders list
- Persistent session (credentials stored securely)

---

### Phase 4: Trading
**Goal**: Place and cancel orders

#### Rust Backend
1. Implement order creation with EIP-712 signing:
   - Build order struct
   - Hash and sign
   - Generate order ID
2. Implement order placement (`POST /order`)
3. Implement order cancellation (`DELETE /order/{id}`)
4. Implement bulk cancel (`DELETE /orders`)
5. Create Tauri commands:
   - `place_order(market_id, side, price, size, order_type)` → Order result
   - `cancel_order(order_id)` → Cancel result
   - `cancel_all_orders(market_id?)` → Bulk cancel result
6. Real-time order status via WebSocket

#### React Frontend
1. Build order entry form:
   - Side toggle (Yes/No)
   - Price input with validation
   - Size input with max calculation
   - Total cost display
2. Add order type selector (GTC, FOK, GTD)
3. Build confirmation modal with order summary
4. Add order feedback (success toast, error display)
5. Implement optimistic updates for order list
6. Add keyboard shortcuts for quick trading

#### Deliverables
- Full order entry form with validation
- Order confirmation modal
- Cancel individual/all orders
- Real-time order status updates
- Keyboard shortcuts (Ctrl+Enter to submit, etc.)

---

### Phase 5: Polish + Advanced Features
**Goal**: Production-ready UX

#### Features
1. **Watchlist**
   - Add/remove markets to watchlist
   - Persist to local storage
   - Quick access in sidebar

2. **Advanced Search**
   - Fuzzy search with fuse.js
   - Filter by category, volume, status
   - Sort options (volume, price change, etc.)

3. **Trade History**
   - Full order history (filled, cancelled)
   - Export to CSV

4. **Notifications**
   - Order fill notifications
   - Price alerts
   - System tray notifications (Tauri)

5. **Settings**
   - Theme (light/dark/system)
   - Default order size
   - Confirmation preferences
   - API endpoint override (for testnet)

6. **Performance**
   - Virtual scrolling for large lists
   - WebSocket message batching
   - Memoization and optimization

7. **Developer Tools**
   - Logging panel
   - WebSocket inspector
   - State debugger

#### Deliverables
- Polished, production-ready application
- Persistent user preferences
- Full feature parity with web app
- System tray integration

---

## Key Dependencies

### Rust (Cargo.toml)
```toml
[dependencies]
tauri = { version = "2", features = ["tray-icon", "protocol-asset"] }
tokio = { version = "1", features = ["full"] }
tokio-tungstenite = { version = "0.21", features = ["native-tls"] }
reqwest = { version = "0.12", features = ["json", "rustls-tls"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
ethers = { version = "2", features = ["eip712"] }
keyring = "2"
thiserror = "1"
tracing = "0.1"
tracing-subscriber = "0.3"
```

### Frontend (package.json)
```json
{
  "dependencies": {
    "@tauri-apps/api": "^2",
    "react": "^18",
    "react-dom": "^18",
    "react-router-dom": "^6",
    "zustand": "^4",
    "lightweight-charts": "^4",
    "fuse.js": "^7",
    "@radix-ui/react-dialog": "^1",
    "@radix-ui/react-dropdown-menu": "^1"
  },
  "devDependencies": {
    "@tauri-apps/cli": "^2",
    "typescript": "^5",
    "vite": "^5",
    "tailwindcss": "^3",
    "@types/react": "^18"
  }
}
```

---

## Data Flow: Rust → React

```
┌─────────────────────────────────────────────────────────────────────┐
│                        Rust Backend (Tauri)                          │
├────────────────────────────┬────────────────────────────────────────┤
│   CLOB WebSocket           │        RTDS WebSocket                   │
│   (tokio-tungstenite)      │        (tokio-tungstenite)              │
└──────────┬─────────────────┴──────────────────┬─────────────────────┘
           │                                    │
           ▼                                    ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    WebSocket Manager (Rust)                          │
│  - Parses JSON messages                                              │
│  - Throttles high-frequency updates                                  │
│  - Emits Tauri events via app.emit()                                 │
└──────────────────────────┬──────────────────────────────────────────┘
                           │
                           ▼ Tauri Events
┌─────────────────────────────────────────────────────────────────────┐
│                    React Frontend                                    │
│  - useTauriEvents() hook subscribes to events                        │
│  - Updates Zustand stores                                            │
│  - React re-renders affected components                              │
└──────────────────────────┬──────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    UI Components                                     │
│  - Read from Zustand stores                                          │
│  - Call Tauri commands for actions                                   │
│  - Render with Tailwind + Radix                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## UI Mockups

### Main Dashboard
```
┌─────────────────────────────────────────────────────────────────────────┐
│  🔷 PLGUI                              🔴 WS  👤 0x1234...5678  [─][□][×]│
├──────────┬──────────────────────────────────────────────────────────────┤
│          │  Dashboard                                                    │
│ 📊 Dash  │  ─────────────────────────────────────────────────────────── │
│          │                                                               │
│ 🔍 Markets│  ┌─────────────────────┐  ┌─────────────────────────────┐   │
│          │  │ Portfolio            │  │ Watchlist                    │   │
│ 💼 Port  │  │ ──────────           │  │ ─────────                    │   │
│          │  │ Total: $4,521.34     │  │ Trump Win       0.52  ▲+3.2% │   │
│ ⚙️ Settings│ │ P&L:   +$234.12     │  │ BTC > 100k      0.78  ▼-1.1% │   │
│          │  │ Open:  3 positions   │  │ Fed Cut Dec     0.34  ─ 0.0% │   │
│          │  └─────────────────────┘  └─────────────────────────────┘   │
│          │                                                               │
│          │  ┌───────────────────────────────────────────────────────┐   │
│          │  │ Active Orders                                          │   │
│          │  │ ────────────────────────────────────────────────────── │   │
│          │  │ Trump Win  │  BUY YES  │  0.51  │  100  │  $51.00  [✕] │   │
│          │  │ BTC > 100k │  SELL NO  │  0.25  │  50   │  $12.50  [✕] │   │
│          │  └───────────────────────────────────────────────────────┘   │
├──────────┴──────────────────────────────────────────────────────────────┤
│  WebSocket: Connected  │  Last update: 2s ago  │  v0.1.0                 │
└─────────────────────────────────────────────────────────────────────────┘
```

### Market Detail + Trading
```
┌─────────────────────────────────────────────────────────────────────────┐
│  🔷 PLGUI                              🟢 WS  👤 0x1234...5678  [─][□][×]│
├──────────┬──────────────────────────────────────────────────────────────┤
│          │  ← Back                                                       │
│ 📊 Dash  │                                                               │
│          │  Will Donald Trump win the 2024 Presidential Election?        │
│ 🔍 Markets│  Volume: $52.3M  │  Liquidity: $1.2M  │  Ends: Nov 5, 2024   │
│          │  ─────────────────────────────────────────────────────────── │
│ 💼 Port  │                                                               │
│          │  ┌─────────────────────────────┐ ┌─────────────────────────┐ │
│ ⚙️ Settings│ │ Price Chart                 │ │ Order Book              │ │
│          │  │    0.55 ┤      ╭──╮         │ │ ────────────────────    │ │
│          │  │    0.52 ┤  ╭───╯  ╰──       │ │ Bids         Asks       │ │
│          │  │    0.49 ┤──╯                │ │ 0.51 ████ │ ██ 0.53     │ │
│          │  │    0.46 ┤                   │ │ 0.50 ██████│ ████ 0.54  │ │
│          │  │         └──────────────     │ │ 0.49 ████  │ ██████ 0.55│ │
│          │  └─────────────────────────────┘ └─────────────────────────┘ │
│          │                                                               │
│          │  ┌───────────────────────────────────────────────────────┐   │
│          │  │ Place Order                                            │   │
│          │  │ ┌─────────┐ ┌─────────┐  ┌────────────┐ ┌────────────┐│   │
│          │  │ │ ✓ YES   │ │   NO    │  │ Price: 0.52│ │ Shares: 100││   │
│          │  │ └─────────┘ └─────────┘  └────────────┘ └────────────┘│   │
│          │  │                                                        │   │
│          │  │ Total Cost: $52.00        Potential Return: $48.00     │   │
│          │  │                                    [  Place Order  ]   │   │
│          │  └───────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Security Considerations
- **Private keys**: Never stored in plaintext; held only in memory during session
- **Keyring storage**: API credentials stored via OS keyring (Keychain/Windows Credential Manager/Secret Service)
- **IPC security**: Tauri's allowlist restricts which commands frontend can invoke
- **No remote code**: All frontend assets bundled locally
- **Audit dependencies**: Regular `cargo audit` and `npm audit`

## API Reference

### Polymarket Docs
- Main: https://docs.polymarket.com/
- CLOB WebSocket: https://docs.polymarket.com/developers/CLOB/websocket/wss-overview
- Gamma API: https://docs.polymarket.com/developers/Gamma-API/overview

### Key REST Endpoints (clob.polymarket.com)
- `GET /markets` - List markets
- `GET /book?token_id=xxx` - Order book
- `POST /order` - Place order
- `DELETE /order/{id}` - Cancel order
- `GET /positions` - User positions
- `GET /orders` - Active orders

---

## Getting Started

```bash
# Prerequisites
# - Rust 1.75+
# - Node.js 20+
# - pnpm (recommended) or npm

# Initialize project
pnpm create tauri-app plgui --template react-ts

# Install dependencies
cd plgui
pnpm install

# Run in development
pnpm tauri dev

# Build for production
pnpm tauri build
```