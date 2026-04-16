# python-binance-rs

`python-binance-rs` is a `python-binance`-inspired async Rust client for Binance Spot REST APIs.

This repository focuses on the most valuable Spot REST surface first instead of attempting full parity with the Python package in one pass.

## Quick start

Run tests first:

```bash
cargo test
```

Run the example against Binance Spot:

```bash
BINANCE_API_KEY=... BINANCE_API_SECRET=... cargo run --example spot_rest
```

Add it as a dependency:

```toml
[dependencies]
python-binance-rs = { path = "." }
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

## Repository map

```text
src/
  client.rs   core REST client
  ws.rs       websocket helpers and typed events
  types.rs    request/response models
  error.rs    library error types
examples/
  spot_rest.rs
tests/
  client_contract.rs
  websocket_contract.rs
```

## Current status

Today this crate is strongest on:

- Binance Spot REST support
- signed request handling
- async `reqwest` transport with `rustls`
- Spot websocket helpers, typed event parsing, and WS API request/response models

It is not yet aiming for full `python-binance` parity.

## Current scope

- Public REST endpoints
  - `ping`
  - `server_time`
  - `exchange_info`
  - `symbol_info`
  - `ticker_price`
  - `book_ticker`
  - `book_tickers`
  - `recent_trades`
  - `historical_trades`
  - `aggregate_trades`
  - `average_price`
  - `ticker_24hr`
  - `ticker_24hr_all`
  - `order_book`
  - `klines`
  - `ui_klines`
- Signed REST endpoints
  - `account`
  - `asset_balance`
  - `all_orders`
  - `my_trades`
  - `current_order_count`
  - `open_order_lists`
  - `order_list`
  - `all_order_lists`
  - `cancel_order_list`
  - `cancel_open_orders`
  - `commission_rates`
  - `prevented_matches`
  - `order_amendments`
  - `symbol_filters`
  - `open_orders`
  - `create_oco_order`
  - `cancel_replace_order`
  - `create_order`
  - `cancel_order`
  - `get_order`
- Binance-compatible request signing
- Async `reqwest` transport with `rustls`
- Spot websocket support
  - raw and combined market stream URL builders
  - websocket connection helpers
  - typed parsing for trade, bookTicker, kline, and core user-data events
  - websocket API base client and request/response envelopes
  - helpers for `userDataStream.subscribe` and `userDataStream.subscribe.listenToken`
  - signed WS API request builder for flows like `session.logon`

## Example code

```rust
use python_binance_rs::{BinanceClient, CreateOrderRequest, OrderSide, TimeInForce};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = BinanceClient::builder()
        .api_key(std::env::var("BINANCE_API_KEY")?)
        .api_secret(std::env::var("BINANCE_API_SECRET")?)
        .build()?;

    let time = client.server_time().await?;
    println!("server time: {}", time.server_time);

    let order = client
        .create_order(&CreateOrderRequest::limit(
            "BTCUSDT",
            OrderSide::Buy,
            "0.00100000",
            "50000.00",
            TimeInForce::Gtc,
        ))
        .await?;

    println!("new order id: {}", order.order_id);
    Ok(())
}
```

## Notes

- The client targets the Binance Spot REST API hosted at `https://api.binance.com` by default.
- Signed endpoints require both API key and API secret.
- `exchange_info` is returned as `serde_json::Value` for now to keep the initial implementation compact.
- The current library does not yet cover websocket streams, futures/margin APIs, or the entire `python-binance` surface.
