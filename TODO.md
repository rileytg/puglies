# PLGUI - Task Checklist

## Phase 1: Foundation ✓

### Project Setup
- [x] Initialize Tauri v2 project with React template
- [x] Configure TypeScript strict mode
- [x] Set up Tailwind CSS
- [x] Install shadcn/ui and configure components
- [x] Set up Zustand
- [x] Configure Vite for development
- [x] Set up Rust workspace structure
- [x] Add Cargo dependencies (reqwest, serde, tokio, thiserror)
- [x] Configure Tauri permissions and capabilities

### Gamma API Client (Rust)
- [x] Create `api/gamma.rs` module
- [x] Implement `get_markets()` - fetch market list
- [x] Implement `get_market(condition_id)` - single market
- [x] Implement `get_events()` - event list
- [x] Add response types with serde deserialization
- [x] Add error handling with thiserror
- [ ] Write unit tests for API parsing

### Tauri Commands
- [x] Create `commands/markets.rs`
- [x] Implement `get_markets` command
- [x] Implement `get_market` command
- [x] Implement `search_markets` command
- [x] Register commands in `main.rs`

### Frontend - Layout
- [x] Create `Sidebar` component with navigation
- [x] Create `Header` component with status indicators
- [x] Create `StatusBar` component
- [x] Set up React Router for navigation
- [x] Create base layout wrapper

### Frontend - Markets
- [x] Create `markets` Zustand store
- [x] Create `MarketList` component
- [x] Create `MarketCard` component
- [x] Create `MarketSearch` component with filtering
- [x] Create `MarketDetail` page
- [x] Wire up Tauri invoke calls
- [x] Add loading and error states

### Phase 1 Verification
- [x] Markets load and display correctly
- [x] Search/filter works
- [x] Navigation between views works
- [x] App builds without errors

---

## Phase 2: Real-Time Data ✓

### WebSocket Manager (Rust)
- [x] Create `websocket/manager.rs`
- [x] Implement connection state machine
- [x] Add exponential backoff reconnection
- [x] Create message parsing and routing

### RTDS WebSocket
- [x] Create `websocket/rtds.rs`
- [x] Implement connection to `wss://ws-live-data.polymarket.com`
- [x] Parse market activity messages
- [x] Parse trade messages
- [x] Emit `price_update` Tauri events
- [x] Emit `trade_update` Tauri events

### CLOB WebSocket
- [x] Create `websocket/clob.rs`
- [x] Implement connection to CLOB WS
- [x] Subscribe to order book channels
- [x] Parse order book snapshot messages
- [x] Parse order book delta messages
- [x] Emit `orderbook_update` Tauri events

### Frontend - WebSocket Integration
- [x] Create `useTauriEvents` hook
- [x] Create `websocket` Zustand store for connection status
- [x] Create `orderbook` Zustand store
- [ ] Update `MarketCard` with live prices
- [x] Add connection status indicator to header

### Frontend - Order Book
- [x] Create `OrderBook` component
- [x] Implement depth visualization (horizontal bars)
- [x] Add bid/ask spread display
- [ ] Add price level highlighting

### Frontend - Charts
- [x] Install Lightweight Charts
- [x] Create `PriceChart` component
- [ ] Implement candlestick/line chart toggle
- [ ] Add time range selector (1h, 24h, 7d, 30d)
- [ ] Wire up historical data fetching
- [x] Add real-time price updates to chart

### Phase 2 Verification
- [ ] Prices update in real-time
- [ ] Order book updates smoothly
- [ ] Charts display and update
- [ ] Reconnection works after disconnect

---

## Phase 3: Authentication ✓

### EIP-712 Signing (Rust)
- [x] Add alloy-signer dependencies
- [x] Create `auth/eip712.rs`
- [x] Implement typed data structure for Polymarket
- [x] Implement signing function
- [ ] Test signature verification

### HMAC Authentication (Rust)
- [x] Create `auth/hmac.rs`
- [x] Implement L2 header generation
- [x] Add timestamp and signature headers

### API Key Derivation
- [x] Create `commands/auth.rs`
- [x] Implement nonce generation
- [x] Implement API key derivation flow
- [x] POST to `/auth/api-key` endpoint
- [x] Store credentials in keyring

### Secure Storage
- [x] Add keyring-rs dependency
- [x] Implement credential storage
- [x] Implement credential retrieval
- [x] Implement credential deletion (logout)

### CLOB REST Client
- [x] Create `api/clob.rs`
- [x] Implement authenticated request helper
- [x] Implement `get_positions()`
- [x] Implement `get_orders()`
- [x] Implement `get_balance()`

### Tauri Auth Commands
- [x] Implement `login` command
- [x] Implement `logout` command
- [x] Implement `get_auth_status` command
- [x] Implement `get_positions` command
- [x] Implement `get_orders` command
- [x] Implement `get_balance` command

### Frontend - Auth
- [x] Create `auth` Zustand store
- [x] Create `LoginModal` component
- [x] Add secure private key input (password field)
- [x] Add login/logout to user menu
- [ ] Implement protected route wrapper

### Frontend - Portfolio
- [x] Create `auth` store with portfolio data
- [x] Create Portfolio page with balance display
- [x] Create positions table
- [x] Create orders table
- [x] Add P&L display

### CLOB User WebSocket
- [ ] Subscribe to user channel after auth
- [ ] Handle order status updates
- [ ] Handle position updates
- [ ] Emit user-specific events

### Phase 3 Verification
- [ ] Login flow works end-to-end
- [x] Credentials persist across restarts (keyring)
- [x] Portfolio displays correctly
- [ ] User WebSocket updates work
- [x] Logout clears session

---

## Phase 4: Trading

### Order Signing (Rust)
- [ ] Implement order struct with all fields
- [ ] Implement order hashing (EIP-712)
- [ ] Implement order signing
- [ ] Generate deterministic order IDs

### Order API (Rust)
- [ ] Implement `POST /order` (place order)
- [ ] Implement `DELETE /order/{id}` (cancel)
- [ ] Implement `DELETE /orders` (bulk cancel)
- [ ] Handle order response types

### Tauri Trading Commands
- [ ] Implement `place_order` command
- [ ] Implement `cancel_order` command
- [ ] Implement `cancel_all_orders` command

### Frontend - Order Form
- [ ] Create `OrderForm` component
- [ ] Add side toggle (Yes/No)
- [ ] Add price input with validation
- [ ] Add size input with max calculation
- [ ] Add order type selector (GTC, FOK)
- [ ] Display total cost and potential return
- [ ] Add form validation

### Frontend - Order Confirmation
- [ ] Create `OrderConfirmation` modal
- [ ] Display order summary
- [ ] Add confirm/cancel buttons
- [ ] Handle submission state

### Frontend - Order Management
- [ ] Add cancel button to orders list
- [ ] Add "Cancel All" functionality
- [ ] Implement optimistic updates
- [ ] Add success/error toasts

### Keyboard Shortcuts
- [ ] Ctrl+Enter to submit order
- [ ] Escape to close modals
- [ ] Tab to switch form fields

### Phase 4 Verification
- [ ] Orders place successfully
- [ ] Order status updates in real-time
- [ ] Cancellation works
- [ ] Error handling displays correctly

---

## Phase 5: Polish

### Watchlist
- [ ] Create `watchlist` Zustand store
- [ ] Add/remove markets to watchlist
- [ ] Display watchlist in sidebar
- [ ] Persist to local storage

### Search Improvements
- [ ] Add fuse.js for fuzzy search
- [ ] Add category filters
- [ ] Add volume/activity filters
- [ ] Add sort options

### Trade History
- [ ] Implement order history fetching
- [ ] Create `TradeHistory` component
- [ ] Add CSV export

### Notifications
- [ ] Add toast notification system
- [ ] Implement order fill notifications
- [ ] Add system tray notifications (Tauri)
- [ ] Add price alerts (optional)

### Settings
- [ ] Create Settings page
- [ ] Add theme toggle (light/dark/system)
- [ ] Add default order size setting
- [ ] Add confirmation preferences
- [ ] Persist settings to local storage

### Performance
- [ ] Add virtual scrolling to long lists
- [ ] Implement WebSocket message batching
- [ ] Add React.memo to heavy components
- [ ] Profile and optimize renders

### Developer Tools
- [ ] Add logging configuration
- [ ] Create debug panel (optional)
- [ ] Add WebSocket inspector (optional)

### Final Polish
- [ ] Review all error messages
- [ ] Add loading skeletons
- [ ] Test all keyboard navigation
- [ ] Cross-platform testing (macOS, Windows, Linux)
- [ ] Update app icons and metadata

### Phase 5 Verification
- [ ] All features work smoothly
- [ ] No memory leaks
- [ ] App starts quickly
- [ ] Builds for all platforms

---

## Release Checklist
- [ ] Version bump
- [ ] Update changelog
- [ ] Build release binaries
- [ ] Code signing (macOS, Windows)
- [ ] Create GitHub release
- [ ] Update documentation
