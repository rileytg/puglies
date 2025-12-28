# PLGUI Agent Instructions

## Project Overview

PLGUI is a cross-platform Polymarket trading desktop application built with:
- **Frontend**: React + TypeScript + Tailwind CSS + shadcn/ui + Zustand
- **Backend**: Tauri v2 (Rust)
- **Build**: Vite + pnpm

## Architecture

```
Frontend (React)                    Backend (Rust/Tauri)
┌────────────────┐                  ┌─────────────────┐
│ Zustand stores │◄── events ──────│ WebSocket mgr   │
│ Components     │── invoke() ────►│ Commands        │
└────────────────┘                  │ API clients     │
                                    └─────────────────┘
```

## Key Files

### Rust Backend (`src-tauri/`)
- `src/main.rs` - Tauri app entry point
- `src/lib.rs` - Command registration
- `src/commands/` - Tauri command handlers (invoke targets)
- `src/api/` - REST API clients (Gamma, CLOB)
- `src/websocket/` - WebSocket connection managers (Phase 2+)
- `src/auth/` - EIP-712 signing, HMAC auth (Phase 3+)
- `src/types.rs` - Shared type definitions
- `src/error.rs` - Error handling with thiserror

### React Frontend (`src/`)
- `src/App.tsx` - Router and layout setup
- `src/stores/` - Zustand state management
- `src/components/` - UI components (layout/, markets/, ui/)
- `src/pages/` - Route pages
- `src/lib/tauri.ts` - Tauri invoke wrappers
- `src/lib/types.ts` - TypeScript types (mirrors Rust types)

## Development Commands

```bash
# Install dependencies
pnpm install

# Development mode (frontend + Tauri)
pnpm tauri dev

# Build for production
pnpm tauri build

# Frontend only
pnpm dev
pnpm build
```

## Coding Standards

### Rust
- Use `thiserror` for error types
- All public functions need documentation
- Prefer `Result<T, AppError>` return types
- Use `#[tauri::command]` for IPC handlers
- Emit events via `app.emit("event_name", payload)`

### TypeScript/React
- Strict TypeScript (no `any`)
- Zustand for state (not Context or Redux)
- Use `invoke<T>()` wrapper from `lib/tauri.ts`
- Components in PascalCase, hooks with `use` prefix
- shadcn/ui components in `components/ui/`

### Tauri IPC Pattern

```rust
// Rust command
#[tauri::command]
async fn get_markets(limit: Option<u32>) -> Result<Vec<Market>, AppError> {
    // ...
}
```

```typescript
// TypeScript invoke
const markets = await invoke<Market[]>('get_markets', { limit: 50 });
```

## Current Phase: 3 (Authentication)

Phase 2 (Real-Time Data) completed:
- WebSocket manager with exponential backoff reconnection
- RTDS WebSocket for live market prices
- CLOB WebSocket for order book depth
- Frontend event hooks, stores, OrderBook and PriceChart components

Phase 3 Focus areas:
1. EIP-712 signing for Polymarket API authentication
2. HMAC authentication for CLOB REST API
3. API key derivation flow
4. Secure credential storage using OS keyring
5. Portfolio and positions display

## External APIs

| API | URL | Auth | Purpose |
|-----|-----|------|---------|
| Gamma | `https://gamma-api.polymarket.com` | None | Market metadata |
| CLOB REST | `https://clob.polymarket.com` | HMAC | Orders, positions |
| CLOB WS | `wss://ws-subscriptions-clob.polymarket.com` | Token | Order book |
| RTDS WS | `wss://ws-live-data.polymarket.com` | None | Market activity |

## Important Notes

- Never store private keys in plain text - use OS keyring (Phase 3)
- WebSocket connections must handle reconnection gracefully
- All prices are in USDC (6 decimals)
- Market outcomes are binary (Yes/No tokens)
- Use `condition_id` as the unique market identifier
