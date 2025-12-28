# PLGUI Implementation Plan

## Overview
Cross-platform Polymarket trading desktop app using Tauri v2 (Rust + React).

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    React Frontend                        │
│  Zustand stores ← useTauriEvents() ← Tauri Events       │
│  Components → invoke() → Tauri Commands                  │
└─────────────────────────┬───────────────────────────────┘
                          │ IPC
┌─────────────────────────┴───────────────────────────────┐
│                    Rust Backend                          │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────┐  │
│  │ Commands    │  │ WebSocket   │  │ Auth            │  │
│  │ (API layer) │  │ Manager     │  │ (EIP-712/HMAC)  │  │
│  └──────┬──────┘  └──────┬──────┘  └────────┬────────┘  │
│         │                │                   │           │
│  ┌──────┴────────────────┴───────────────────┴────────┐ │
│  │              HTTP/WS Clients                        │ │
│  │  reqwest (REST) + tokio-tungstenite (WebSocket)    │ │
│  └────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────┐
│                 Polymarket APIs                          │
│  Gamma API (markets) │ CLOB REST (orders) │ WebSockets  │
└─────────────────────────────────────────────────────────┘
```

## Phases

### Phase 1: Foundation
Set up Tauri project, implement Gamma API client, build static market browser.
- No auth required
- Read-only market data
- Basic UI shell

### Phase 2: Real-Time Data
Add WebSocket connections for live prices and order book.
- RTDS for market activity
- CLOB WS for order book depth
- Price charts with historical data

### Phase 3: Authentication
Implement wallet auth and portfolio viewing.
- EIP-712 signing for API key derivation
- Secure credential storage (OS keyring)
- Portfolio and positions display

### Phase 4: Trading
Full order management.
- Place orders (GTC, FOK)
- Cancel orders
- Real-time order status

### Phase 5: Polish
Production-ready features.
- Watchlists
- Notifications
- Settings/preferences
- Performance optimization

## Key Decisions

1. **Tauri v2** over Electron: Smaller bundle, lower memory, Rust safety
2. **Zustand** over Redux: Simpler API, better Tauri event integration
3. **ethers-rs** for signing: Mature, well-documented EIP-712 support
4. **Lightweight Charts**: TradingView quality without complexity
5. **shadcn/ui**: Unstyled Radix primitives, full control over design

## External Dependencies

### Polymarket APIs
- **Gamma API**: `https://gamma-api.polymarket.com` - Market metadata
- **CLOB REST**: `https://clob.polymarket.com` - Orders, positions
- **CLOB WS**: `wss://ws-subscriptions-clob.polymarket.com` - Order book
- **RTDS WS**: `wss://ws-live-data.polymarket.com` - Market activity

### Docs
- Polymarket: https://docs.polymarket.com/
- Tauri v2: https://v2.tauri.app/
- ethers-rs: https://docs.rs/ethers/

## File Structure

```
plgui/
├── src-tauri/           # Rust backend
│   ├── src/
│   │   ├── main.rs
│   │   ├── commands/    # Tauri command handlers
│   │   ├── websocket/   # WS connection management
│   │   ├── auth/        # EIP-712, HMAC
│   │   └── api/         # REST clients
│   └── Cargo.toml
├── src/                 # React frontend
│   ├── components/      # UI components
│   ├── stores/          # Zustand state
│   ├── hooks/           # Custom hooks
│   └── lib/             # Utilities
├── PLAN.md
├── TODO.md
└── tauri-gui.md         # Detailed spec
```
