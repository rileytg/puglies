# Polymarket API Documentation

Comprehensive documentation for the Polymarket APIs used in this application.

## Table of Contents

1. [Overview](#overview)
2. [Market Identifiers](#market-identifiers)
3. [Gamma API](#gamma-api)
4. [CLOB REST API](#clob-rest-api)
5. [Data API](#data-api)
6. [WebSocket APIs](#websocket-apis)
7. [Authentication](#authentication)
8. [Order Signing (EIP-712)](#order-signing-eip-712)
9. [Common Gotchas](#common-gotchas)

---

## Overview

Polymarket uses multiple APIs for different purposes:

| API | Base URL | Auth | Purpose |
|-----|----------|------|---------|
| Gamma | `https://gamma-api.polymarket.com` | None | Market metadata, search |
| CLOB REST | `https://clob.polymarket.com` | HMAC | Orders, balances, API keys |
| Data API | `https://data-api.polymarket.com` | None | Positions, historical data |
| CLOB WebSocket | `wss://ws-subscriptions-clob.polymarket.com` | Token | Order book streaming |
| RTDS WebSocket | `wss://ws-live-data.polymarket.com` | None | Live market data |

---

## Market Identifiers

**CRITICAL**: Markets have multiple identifiers. Using the wrong one causes errors.

| Field | Format | Example | Use Case |
|-------|--------|---------|----------|
| `id` | Numeric string | `"516710"` | Gamma API path lookups |
| `condition_id` | Hex (66 chars) | `"0x7c6c69d91b21cbbea08a13d0ad51c0e96a956045aaadc77bce507c6b0475b66e"` | CLOB orders, WebSocket subscriptions |
| `question_id` | Hex (66 chars) | `"0xabc123..."` | On-chain question identifier |
| `market_slug` | URL-safe string | `"will-btc-reach-100k"` | Polymarket website URLs |

### Common Mistakes

```
# WRONG - Gamma API returns 422 "id is invalid"
GET /markets/0x7c6c69d91b21cbbea08a13d0ad51c0e96a956045aaadc77bce507c6b0475b66e

# CORRECT - Use internal numeric ID
GET /markets/516710
```

---

## Gamma API

Base URL: `https://gamma-api.polymarket.com`

### List Markets

```
GET /markets
```

Query parameters:
- `active=true` - Only active markets
- `closed=false` - Exclude closed markets
- `archived=false` - Exclude archived markets
- `limit=50` - Number of results (default 50)
- `offset=0` - Pagination offset
- `order=volumeNum` - Sort field (camelCase)
- `ascending=false` - Sort direction
- `text_query=<search>` - Full-text search (URL encoded)
- `slug_contains=<text>` - Filter by slug substring

**Response**: Array of market objects (NOT wrapped in an object)

```json
[
  {
    "id": "516710",
    "conditionId": "0x7c6c69d91b21cbbea08a13d0ad51c0e96a956045aaadc77bce507c6b0475b66e",
    "questionId": "0xabc...",
    "question": "US recession in 2025?",
    "description": "...",
    "marketSlug": "us-recession-2025",
    "endDateIso": "2025-12-31T23:59:59Z",
    "outcomes": "[\"Yes\",\"No\"]",
    "outcomePrices": "[\"0.35\",\"0.65\"]",
    "clobTokenIds": "[\"12345...\",\"67890...\"]",
    "volumeNum": 1500000.50,
    "liquidityNum": 250000.00,
    "spread": 0.02,
    "active": true,
    "closed": false,
    "archived": false,
    "acceptingOrders": true
  }
]
```

**Note**: `outcomes`, `outcomePrices`, and `clobTokenIds` are JSON-encoded strings that need parsing.

### Get Single Market

```
GET /markets/{id}
```

**Important**: Use the internal numeric `id`, NOT `conditionId`.

```bash
# Correct
curl https://gamma-api.polymarket.com/markets/516710

# Wrong - returns 422 error
curl https://gamma-api.polymarket.com/markets/0x7c6c69d9...
```

### List Events

```
GET /events?active=true&closed=false&limit=20&order=volume&ascending=false
```

Events are collections of related markets (e.g., "2024 US Election" contains multiple markets).

---

## CLOB REST API

Base URL: `https://clob.polymarket.com`

All endpoints require L2 HMAC authentication (see [Authentication](#authentication)).

### Get Balance

```
GET /balance
```

Response:
```json
{
  "balance": "192159379",
  "allowances": {
    "0x4bFb41d5B3570DeFd03C39a9A4D8dE6Bd8B8982E": "115792089237316195..."
  }
}
```

**Note**: Balance is in USDC wei (6 decimals). Divide by 1e6 to get USDC.

### Get Orders

```
GET /data/orders
```

**Note**: The endpoint is `/data/orders`, NOT `/orders` (which returns 405).

Response:
```json
[
  {
    "id": "abc123",
    "market": "0x7c6c69d9...",
    "asset_id": "token_id_here",
    "side": "BUY",
    "original_size": "100.0",
    "size_matched": "50.0",
    "price": "0.35",
    "status": "LIVE",
    "order_type": "GTC",
    "created_at": "2024-01-15T10:30:00Z"
  }
]
```

### Place Order

```
POST /order
Content-Type: application/json

{
  "order": {
    "salt": 12345,
    "maker": "0xYourAddress",
    "signer": "0xSignerAddress",
    "taker": "0x0000000000000000000000000000000000000000",
    "tokenId": "clob_token_id",
    "makerAmount": "1000000",
    "takerAmount": "350000",
    "expiration": 0,
    "nonce": 0,
    "feeRateBps": 0,
    "side": 0,
    "signatureType": 0,
    "signature": "0x..."
  },
  "owner": "0xYourAddress",
  "orderType": "GTC"
}
```

### Cancel Order

```
DELETE /order?orderID={order_id}
```

### Cancel All Orders

```
DELETE /cancel-all
```

### Cancel Market Orders

```
DELETE /cancel-market-orders?market={condition_id}
```

### Derive API Key

```
GET /auth/api-key
Headers:
  POLY_ADDRESS: 0xYourAddress
  POLY_SIGNATURE: 0x...  (EIP-712 signature)
  POLY_TIMESTAMP: 1234567890
  POLY_NONCE: 0
```

Response:
```json
{
  "apiKey": "abc123...",
  "secret": "base64_encoded_secret",
  "passphrase": "random_passphrase"
}
```

---

## Data API

Base URL: `https://data-api.polymarket.com`

### Get Positions

```
GET /positions?user={polymarket_address}
```

**Note**: Use the Polymarket proxy wallet address, not the signing wallet.

Response:
```json
[
  {
    "asset": "token_id",
    "conditionId": "0x7c6c69d9...",
    "size": 100.5,
    "avgPrice": 0.35,
    "initialValue": 35.175,
    "currentValue": 40.20,
    "cashPnl": 5.025,
    "percentPnl": 14.29,
    "curPrice": 0.40,
    "title": "Will X happen?",
    "outcome": "Yes",
    "proxyWallet": "0x..."
  }
]
```

---

## WebSocket APIs

### CLOB WebSocket (Order Book)

URL: `wss://ws-subscriptions-clob.polymarket.com/ws/market`

Requires authentication token from `/ws-auth-token` endpoint.

Subscribe to assets:
```json
{
  "type": "Market",
  "assets_ids": ["token_id_1", "token_id_2"]
}
```

Messages:
- `book` - Full order book snapshot
- `price_change` - Price updates
- `trade` - Trade executions

### RTDS WebSocket (Live Data)

URL: `wss://ws-live-data.polymarket.com` (NO `/ws` suffix - returns 403!)

No authentication required.

Subscribe to market price changes:
```json
{
  "action": "subscribe",
  "subscriptions": [
    {
      "topic": "clob_market",
      "type": "price_change",
      "filters": "[\"token_id_1\",\"token_id_2\"]"
    }
  ]
}
```

**Note**: The `filters` field is a JSON-encoded string array of **token IDs** (not condition_id).

#### Response Message Format (Abbreviated)

RTDS uses abbreviated field names to minimize bandwidth:

```json
{
  "connection_id": "WVHt1fsirPECGfA=",
  "payload": {
    "m": "0x836b850fc838195374862551a36f1c8691d96ff01e58b0a071f0fc1a0e357fb1",
    "pc": [
      {
        "a": "50862799703982327636174441241062907649998751737045006653560124656563256528691",
        "p": "0.987",
        "s": "100",
        "b": "0.986",
        "k": "0.988",
        "h": "0xabc..."
      }
    ]
  }
}
```

Field abbreviations:
| Abbrev | Full Name | Description |
|--------|-----------|-------------|
| `m` | market | Condition ID (hex) |
| `pc` | price_changes | Array of price updates |
| `a` | asset_id | Token ID |
| `p` | price | Last price |
| `s` | size | Trade size |
| `b` | best_bid | Best bid price |
| `k` | best_ask | Best ask price |
| `h` | hash | Order book hash |

Available topics:
- `clob_market` with types: `price_change`, `agg_orderbook`, `last_trade_price`, `tick_size_change`
- `crypto_prices` - Binance price feeds
- `crypto_prices_chainlink` - Chainlink oracle prices

---

## Authentication

### L1 Headers (API Key Derivation)

Used once to derive API credentials from a signed message.

```
POLY_ADDRESS: 0xYourWalletAddress
POLY_SIGNATURE: 0x... (EIP-712 signature of ClobAuthDomain)
POLY_TIMESTAMP: 1234567890 (Unix seconds)
POLY_NONCE: 0
```

EIP-712 Domain:
```json
{
  "name": "ClobAuthDomain",
  "version": "1",
  "chainId": 137
}
```

Type:
```json
{
  "ClobAuth": [
    {"name": "address", "type": "address"},
    {"name": "timestamp", "type": "string"},
    {"name": "nonce", "type": "uint256"},
    {"name": "message", "type": "string"}
  ]
}
```

Message: `"This message attests that I control the given wallet"`

### L2 Headers (HMAC for API Calls)

Used for authenticated requests after obtaining API credentials.

```
POLY_ADDRESS: 0xYourWalletAddress
POLY_SIGNATURE: base64_hmac_sha256(secret, message)
POLY_TIMESTAMP: 1234567890
POLY_API_KEY: abc123...
POLY_PASSPHRASE: your_passphrase
```

HMAC Message Format:
```
{timestamp}{method}{path}{body}
```

Example:
```
1234567890GET/orders
1234567890POST/order{"order":...}
```

---

## Order Signing (EIP-712)

Orders are signed using a DIFFERENT EIP-712 domain than authentication.

### CTF Exchange Domain

```json
{
  "name": "Polymarket CTF Exchange",
  "version": "1",
  "chainId": 137,
  "verifyingContract": "0x4bFb41d5B3570DeFd03C39a9A4D8dE6Bd8B8982E"
}
```

### Order Type

```json
{
  "Order": [
    {"name": "salt", "type": "uint256"},
    {"name": "maker", "type": "address"},
    {"name": "signer", "type": "address"},
    {"name": "taker", "type": "address"},
    {"name": "tokenId", "type": "uint256"},
    {"name": "makerAmount", "type": "uint256"},
    {"name": "takerAmount", "type": "uint256"},
    {"name": "expiration", "type": "uint256"},
    {"name": "nonce", "type": "uint256"},
    {"name": "feeRateBps", "type": "uint256"},
    {"name": "side", "type": "uint8"},
    {"name": "signatureType", "type": "uint8"}
  ]
}
```

### Amount Calculations

For **BUY** orders (paying USDC for shares):
```
makerAmount = price * size * 1e6  (USDC in wei)
takerAmount = size * 1e6          (shares)
side = 0
```

For **SELL** orders (selling shares for USDC):
```
makerAmount = size * 1e6          (shares)
takerAmount = price * size * 1e6  (USDC in wei)
side = 1
```

---

## Common Gotchas

### 1. API Returns Arrays, Not Objects

Gamma API endpoints return raw arrays:
```json
[{...}, {...}]  // Correct
{"markets": [{...}]}  // Wrong - not how it works
```

### 2. JSON-Encoded String Fields

Some Gamma fields contain JSON strings that need parsing:
```json
{
  "outcomes": "[\"Yes\",\"No\"]",
  "outcomePrices": "[\"0.35\",\"0.65\"]",
  "clobTokenIds": "[\"123\",\"456\"]"
}
```

### 3. camelCase vs snake_case

- Gamma API uses camelCase (`conditionId`, `volumeNum`)
- CLOB API uses snake_case (`condition_id`, `order_type`)
- Tauri auto-converts between camelCase (JS) and snake_case (Rust)

### 4. Two Different EIP-712 Domains

```
ClobAuth domain:     name = "ClobAuthDomain"           (for API key derivation)
CTF Exchange domain: name = "Polymarket CTF Exchange"  (for order signing)
```

### 5. Market Lookup by ID Only

```bash
# Works - returns single market
GET /markets/516710

# Fails with 422 - condition_id is not accepted in path
GET /markets/0x7c6c69d91b21cbbea08a13d0ad51c0e96a956045aaadc77bce507c6b0475b66e

# Query param doesn't filter correctly
GET /markets?condition_id=0x7c6c69d9...  # Returns wrong results
```

### 6. Polymarket Address vs Signing Wallet

Users have two addresses:
- **Signing wallet**: Their Ethereum wallet used to sign transactions
- **Polymarket address**: A proxy wallet specific to Polymarket

Positions are queried using the Polymarket address, not the signing wallet.

### 7. Balance in Wei

Balance is returned in USDC wei (6 decimals):
```
"balance": "192159379"  // = 192.159379 USDC
```

### 8. Token IDs for Trading

Each market outcome has a `token_id` used for:
- CLOB WebSocket subscriptions (order book)
- Placing orders (specify which outcome to buy/sell)

The `condition_id` is used for:
- RTDS WebSocket subscriptions (live prices)
- Identifying the market itself
