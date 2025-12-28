# PLGUI - Task Checklist

## Phase 1: Foundation

### Project Setup
- [ ] Initialize Tauri v2 project with React template
- [ ] Configure TypeScript strict mode
- [ ] Set up Tailwind CSS
- [ ] Install shadcn/ui and configure components
- [ ] Set up Zustand
- [ ] Configure Vite for development
- [ ] Set up Rust workspace structure
- [ ] Add Cargo dependencies (reqwest, serde, tokio, thiserror)
- [ ] Configure Tauri permissions and capabilities

### Gamma API Client (Rust)
- [ ] Create `api/gamma.rs` module
- [ ] Implement `get_markets()` - fetch market list
- [ ] Implement `get_market(condition_id)` - single market
- [ ] Implement `get_events()` - event list
- [ ] Add response types with serde deserialization
- [ ] Add error handling with thiserror
- [ ] Write unit tests for API parsing

### Tauri Commands
- [ ] Create `commands/markets.rs`
- [ ] Implement `get_markets` command
- [ ] Implement `get_market` command
- [ ] Implement `search_markets` command
- [ ] Register commands in `main.rs`

### Frontend - Layout
- [ ] Create `Sidebar` component with navigation
- [ ] Create `Header` component with status indicators
- [ ] Create `StatusBar` component
- [ ] Set up React Router for navigation
- [ ] Create base layout wrapper

### Frontend - Markets
- [ ] Create `markets` Zustand store
- [ ] Create `MarketList` component
- [ ] Create `MarketCard` component
- [ ] Create `MarketSearch` component with filtering
- [ ] Create `MarketDetail` page
- [ ] Wire up Tauri invoke calls
- [ ] Add loading and error states

### Phase 1 Verification
- [ ] Markets load and display correctly
- [ ] Search/filter works
- [ ] Navigation between views works
- [ ] App builds without errors

---

## Phase 2: Real-Time Data

### WebSocket Manager (Rust)
- [ ] Create `websocket/manager.rs`
- [ ] Implement connection state machine
- [ ] Add exponential backoff reconnection
- [ ] Create message parsing and routing

### RTDS WebSocket
- [ ] Create `websocket/rtds.rs`
- [ ] Implement connection to `wss://ws-live-data.polymarket.com`
- [ ] Parse market activity messages
- [ ] Parse trade messages
- [ ] Emit `price_update` Tauri events
- [ ] Emit `trade_update` Tauri events

### CLOB WebSocket
- [ ] Create `websocket/clob.rs`
- [ ] Implement connection to CLOB WS
- [ ] Subscribe to order book channels
- [ ] Parse order book snapshot messages
- [ ] Parse order book delta messages
- [ ] Emit `orderbook_update` Tauri events

### Frontend - WebSocket Integration
- [ ] Create `useTauriEvents` hook
- [ ] Create `websocket` Zustand store for connection status
- [ ] Create `orderbook` Zustand store
- [ ] Update `MarketCard` with live prices
- [ ] Add connection status indicator to header

### Frontend - Order Book
- [ ] Create `OrderBook` component
- [ ] Implement depth visualization (horizontal bars)
- [ ] Add bid/ask spread display
- [ ] Add price level highlighting

### Frontend - Charts
- [ ] Install Lightweight Charts
- [ ] Create `PriceChart` component
- [ ] Implement candlestick/line chart toggle
- [ ] Add time range selector (1h, 24h, 7d, 30d)
- [ ] Wire up historical data fetching
- [ ] Add real-time price updates to chart

### Phase 2 Verification
- [ ] Prices update in real-time
- [ ] Order book updates smoothly
- [ ] Charts display and update
- [ ] Reconnection works after disconnect

---

## Phase 3: Authentication

### EIP-712 Signing (Rust)
- [ ] Add ethers-rs dependency
- [ ] Create `auth/eip712.rs`
- [ ] Implement typed data structure for Polymarket
- [ ] Implement signing function
- [ ] Test signature verification

### HMAC Authentication (Rust)
- [ ] Create `auth/hmac.rs`
- [ ] Implement L2 header generation
- [ ] Add timestamp and signature headers

### API Key Derivation
- [ ] Create `commands/auth.rs`
- [ ] Implement nonce generation
- [ ] Implement API key derivation flow
- [ ] POST to `/auth/api-key` endpoint
- [ ] Store credentials in keyring

### Secure Storage
- [ ] Add keyring-rs dependency
- [ ] Implement credential storage
- [ ] Implement credential retrieval
- [ ] Implement credential deletion (logout)

### CLOB REST Client
- [ ] Create `api/clob.rs`
- [ ] Implement authenticated request helper
- [ ] Implement `get_positions()`
- [ ] Implement `get_orders()`
- [ ] Implement `get_balance()`

### Tauri Auth Commands
- [ ] Implement `login` command
- [ ] Implement `logout` command
- [ ] Implement `get_auth_status` command
- [ ] Implement `get_positions` command
- [ ] Implement `get_orders` command
- [ ] Implement `get_balance` command

### Frontend - Auth
- [ ] Create `auth` Zustand store
- [ ] Create `LoginModal` component
- [ ] Add secure private key input (password field)
- [ ] Add login/logout to user menu
- [ ] Implement protected route wrapper

### Frontend - Portfolio
- [ ] Create `portfolio` Zustand store
- [ ] Create `BalanceSummary` component
- [ ] Create `PositionsList` component
- [ ] Create `OrdersList` component
- [ ] Create Portfolio dashboard page
- [ ] Add P&L calculations

### CLOB User WebSocket
- [ ] Subscribe to user channel after auth
- [ ] Handle order status updates
- [ ] Handle position updates
- [ ] Emit user-specific events

### Phase 3 Verification
- [ ] Login flow works end-to-end
- [ ] Credentials persist across restarts
- [ ] Portfolio displays correctly
- [ ] User WebSocket updates work
- [ ] Logout clears session

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
