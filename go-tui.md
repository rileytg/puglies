# Polymarket TUI (`pltui`) - Implementation Plan

## Overview
A Go-based Terminal User Interface for Polymarket trading using Bubble Tea, with real-time WebSocket data and full trading capabilities.

## Tech Stack
- **TUI Framework**: Bubble Tea (Elm architecture)
- **Styling**: Lipgloss
- **WebSockets**: gorilla/websocket + go-polymarket-real-time-data-client
- **Crypto**: go-ethereum (EIP-712 signing)
- **Charts**: ntcharts (terminal sparklines)

## Project Structure
```
pltui/
├── cmd/pltui/main.go           # Entry point
├── internal/
│   ├── app/                    # Root Bubble Tea model + router
│   ├── auth/                   # L1/L2 authentication (EIP-712, HMAC)
│   ├── clob/                   # CLOB REST client (orders, positions)
│   ├── websocket/              # WS manager (CLOB + RTDS connections)
│   ├── state/                  # Thread-safe state store
│   ├── views/                  # Dashboard, Market, Portfolio views
│   │   ├── dashboard/
│   │   ├── market_browser/
│   │   ├── market_detail/
│   │   ├── portfolio/
│   │   └── components/         # Reusable: table, chart, modal
│   ├── gamma/                  # Gamma API (market metadata)
│   └── config/                 # Config + keyring
└── pkg/types/                  # Shared types
```

## WebSocket Endpoints
1. **CLOB WS** (`wss://ws-subscriptions-clob.polymarket.com/ws/market`) - Order book, user orders
2. **RTDS** (`wss://ws-live-data.polymarket.com`) - Market activity, trades

## Authentication Flow
1. Load private key (env var `POLYMARKET_PRIVATE_KEY`)
2. Check for existing API credentials
3. If none: derive via L1 EIP-712 signing → POST `/auth/api-key`
4. Use L2 HMAC signing for all trading requests

---

## Implementation Phases

### Phase 1: Project Setup + Read-Only Market Browser
**Goal**: Browse markets, view basic details (no auth required)

1. Initialize Go module with dependencies
2. Create project structure
3. Implement Gamma API client for market metadata
4. Build root app model with view routing
5. Create market browser (list + search)
6. Create market detail view (static info)
7. Add keyboard navigation (`j/k`, Enter, Esc, `q`)
8. Style with lipgloss

### Phase 2: Real-Time WebSocket Data
**Goal**: Live prices, order book depth, charts

1. Integrate RTDS WebSocket client
2. Build WebSocket manager with reconnection
3. Create thread-safe state store
4. Wire WS messages → Bubble Tea messages
5. Add live price updates to market browser
6. Build order book depth visualization
7. Add terminal price charts (ntcharts)
8. Connection status indicator

### Phase 3: Authentication + Portfolio
**Goal**: View user positions and orders

1. Implement L1 EIP-712 signing
2. Implement L2 HMAC authentication
3. API key derivation flow
4. Secure credential storage (keyring)
5. Implement position/order fetching (REST)
6. Build portfolio view with P&L
7. Connect to CLOB user WebSocket channel

### Phase 4: Trading
**Goal**: Place and cancel orders

1. Order creation with signing
2. Order placement API integration
3. Order entry form component
4. Order type selection (GTC, FOK)
5. Confirmation modal
6. Cancel order(s) functionality
7. Real-time order status updates
8. Error handling + feedback

### Phase 5: Polish
**Goal**: Production-ready UX

1. Watchlist functionality
2. Fuzzy market search
3. Help overlay
4. Settings/preferences
5. Order history
6. Performance optimization
7. Logging + debugging tools

---

## Key Dependencies
```go
github.com/charmbracelet/bubbletea
github.com/charmbracelet/bubbles
github.com/charmbracelet/lipgloss
github.com/gorilla/websocket
github.com/Matthew17-21/go-polymarket-real-time-data-client
github.com/ethereum/go-ethereum
github.com/shopspring/decimal
github.com/NimbleMarkets/ntcharts
github.com/spf13/viper
github.com/zalando/go-keyring
```

## Data Flow: WebSocket → State → UI
```
┌─────────────────────────────────────────────────────────────────┐
│                        WebSocket Layer                          │
├──────────────────────────┬──────────────────────────────────────┤
│   CLOB WebSocket         │        RTDS WebSocket                │
│   (Order Book, User)     │        (Market Activity)             │
└──────────┬───────────────┴────────────────┬─────────────────────┘
           │                                │
           ▼                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                    WebSocket Manager                            │
│  - Converts raw messages to typed tea.Msg                       │
│  - Sends via program.Send()                                     │
└──────────────────────────┬──────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Bubble Tea Runtime                           │
│  - Receives messages in Update()                                │
│  - Calls appropriate handler                                    │
└──────────────────────────┬──────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Root App Model Update()                      │
│  - Routes messages to child models                              │
│  - Updates central state store                                  │
│  - Returns updated model + commands                             │
└──────────────────────────┬──────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────┐
│                    View() Rendering                             │
│  - Reads from state store                                       │
│  - Renders UI using lipgloss                                    │
│  - Composes child view outputs                                  │
└─────────────────────────────────────────────────────────────────┘
```

## UI Layouts

### Dashboard
```
┌────────────────────────────────────────────────────────────────┐
│ PLTUI - Polymarket Terminal                    [WS: ●] [Auth: ●]│
├────────────────────────────────────────────────────────────────┤
│ Portfolio Summary          │  Watchlist / Hot Markets           │
│ ─────────────────          │  ─────────────────────             │
│ Total Value:   $1,234.56   │  [1] Trump Win        0.52 ▲+0.03  │
│ Open P&L:      +$45.23     │  [2] BTC > 100k      0.78 ▼-0.01  │
│ Day P&L:       +$12.10     │  [3] Fed Rate Cut    0.34 ─       │
├────────────────────────────┴───────────────────────────────────┤
│ Active Orders (3)                                               │
├────────────────────────────────────────────────────────────────┤
│ [m] Markets  [p] Portfolio  [t] Trade  [?] Help  [q] Quit       │
└────────────────────────────────────────────────────────────────┘
```

### Market Detail
```
┌────────────────────────────────────────────────────────────────┐
│ Will Donald Trump win 2024 election?            Volume: $50.2M  │
├────────────────────────────────────────────────────────────────┤
│ Price Chart (24h)          │  Order Book                        │
│ ─────────────────          │  ──────────                        │
│      0.55 ┤    ╭───╮       │  BIDS          │  ASKS             │
│      0.52 ┤───╯    ╰──     │  0.51  [████]  │  0.53 [██]        │
│      0.49 ┤                │  0.50  [██████]│  0.54 [████]      │
├────────────────────────────┴───────────────────────────────────┤
│ Place Order                                                     │
│ Side: [YES ▼]  Price: [0.52    ]  Size: [100     ]  [$52.00]   │
│ [Enter] Place Order   [Tab] Switch Side   [Esc] Cancel          │
└────────────────────────────────────────────────────────────────┘
```

## Critical Implementation Notes
- **Thread safety**: State store uses `sync.RWMutex` (WS goroutines write, UI reads)
- **Reconnection**: Exponential backoff for WebSocket disconnects
- **Order book throttling**: High-frequency updates need UI throttling
- **Security**: Never log private keys; use keyring for credential storage

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

## Start Command
```bash
go mod init github.com/rileytg/pltui
```
