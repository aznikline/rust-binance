use httpmock::Method::{GET, POST};
use httpmock::MockServer;
use python_binance_rs::{
    AggregateTrade, AllOrdersRequest, BinanceClient, BookTicker, CancelOrderListRequest,
    CancelOrderRequest, CancelReplaceMode, CreateOcoOrderRequest, CreateOrderRequest, Error,
    KlineInterval, MyTradesRequest, OrderAmendmentsRequest, OrderListQueryRequest, OrderSide,
    OrderType, PreventedMatchesRequest, TimeInForce,
};
use serde_json::json;

#[tokio::test]
async fn signed_requests_require_credentials() {
    let client = BinanceClient::builder()
        .rest_base_url("https://api.binance.com")
        .build()
        .expect("client should build without credentials for public requests");

    let error = client
        .account()
        .await
        .expect_err("account endpoint should reject missing credentials");

    assert!(matches!(error, Error::MissingCredentials));
}

#[tokio::test]
async fn serializes_public_kline_query_without_empty_fields() {
    let client = BinanceClient::builder()
        .rest_base_url("https://api.binance.com")
        .build()
        .expect("client should build");

    let query = client.debug_public_query([
        ("symbol", Some("BTCUSDT".to_string())),
        ("interval", Some(KlineInterval::OneMinute.to_string())),
        ("limit", Some("500".to_string())),
        ("startTime", None),
    ]);

    assert_eq!(query, "symbol=BTCUSDT&interval=1m&limit=500");
}

#[tokio::test]
async fn signs_order_requests_like_binance_requires() {
    let client = BinanceClient::builder()
        .api_key("key")
        .api_secret("secret")
        .fixed_timestamp(1_717_171_717_171)
        .recv_window(5_000)
        .rest_base_url("https://api.binance.com")
        .build()
        .expect("client should build");

    let query = client
        .debug_signed_query(&CreateOrderRequest::limit(
            "BTCUSDT",
            OrderSide::Buy,
            "0.01000000",
            "65000.00",
            TimeInForce::Gtc,
        ))
        .expect("signing should succeed");

    assert!(query.contains("symbol=BTCUSDT"));
    assert!(query.contains("side=BUY"));
    assert!(query.contains("type=LIMIT"));
    assert!(query.contains("timeInForce=GTC"));
    assert!(query.contains("quantity=0.01000000"));
    assert!(query.contains("price=65000.00"));
    assert!(query.contains("recvWindow=5000"));
    assert!(query.contains("timestamp=1717171717171"));
    assert!(
        query.ends_with(
            "signature=81127ef2b71bbe612d6c6030b45191a21247c02b929182e6ffb7466303560c52"
        )
    );
}

#[test]
fn cancel_order_request_allows_order_id_lookup() {
    let request = CancelOrderRequest::by_order_id("BTCUSDT", 42);

    assert_eq!(request.symbol, "BTCUSDT");
    assert_eq!(request.order_id, Some(42));
    assert_eq!(request.orig_client_order_id, None);
}

#[test]
fn market_order_omits_limit_only_fields() {
    let request = CreateOrderRequest::market("BTCUSDT", OrderSide::Sell, "0.15000000");

    assert_eq!(request.symbol, "BTCUSDT");
    assert_eq!(request.side, OrderSide::Sell);
    assert_eq!(request.order_type, OrderType::Market);
    assert_eq!(request.quantity.as_deref(), Some("0.15000000"));
    assert_eq!(request.time_in_force, None);
    assert_eq!(request.price, None);
}

#[tokio::test]
async fn fetches_server_time_from_rest_api() {
    let server = MockServer::start_async().await;
    let mock = server
        .mock_async(|when, then| {
            when.method(GET).path("/api/v3/time");
            then.status(200)
                .json_body(json!({ "serverTime": 1717171717171_u64 }));
        })
        .await;

    let client = BinanceClient::builder()
        .rest_base_url(server.base_url())
        .build()
        .expect("client should build");

    let response = client.server_time().await.expect("request should succeed");

    mock.assert_async().await;
    assert_eq!(response.server_time, 1_717_171_717_171);
}

#[tokio::test]
async fn posts_signed_orders_with_api_key_header() {
    let server = MockServer::start_async().await;
    let mock = server
        .mock_async(|when, then| {
            when.method(POST)
                .path("/api/v3/order")
                .header("x-mbx-apikey", "key")
                .header("content-type", "application/x-www-form-urlencoded")
                .body_contains("symbol=BTCUSDT")
                .body_contains("side=BUY")
                .body_contains("type=LIMIT")
                .body_contains("timeInForce=GTC")
                .body_contains(
                    "signature=81127ef2b71bbe612d6c6030b45191a21247c02b929182e6ffb7466303560c52",
                );
            then.status(200).json_body(json!({
                "symbol": "BTCUSDT",
                "orderId": 42_u64,
                "orderListId": -1,
                "clientOrderId": "abc-123",
                "transactTime": 1717171717171_u64,
                "price": "65000.00",
                "origQty": "0.01000000",
                "executedQty": "0.00000000",
                "cummulativeQuoteQty": "0.00000000",
                "status": "NEW",
                "timeInForce": "GTC",
                "type": "LIMIT",
                "side": "BUY"
            }));
        })
        .await;

    let client = BinanceClient::builder()
        .api_key("key")
        .api_secret("secret")
        .fixed_timestamp(1_717_171_717_171)
        .recv_window(5_000)
        .rest_base_url(server.base_url())
        .build()
        .expect("client should build");

    let response = client
        .create_order(&CreateOrderRequest::limit(
            "BTCUSDT",
            OrderSide::Buy,
            "0.01000000",
            "65000.00",
            TimeInForce::Gtc,
        ))
        .await
        .expect("request should succeed");

    mock.assert_async().await;
    assert_eq!(response.symbol, "BTCUSDT");
    assert_eq!(response.order_id, 42);
    assert_eq!(response.status.as_deref(), Some("NEW"));
}

#[tokio::test]
async fn fetches_single_book_ticker() {
    let server = MockServer::start_async().await;
    let mock = server
        .mock_async(|when, then| {
            when.method(GET)
                .path("/api/v3/ticker/bookTicker")
                .query_param("symbol", "BTCUSDT");
            then.status(200).json_body(json!({
                "symbol": "BTCUSDT",
                "bidPrice": "65000.10",
                "bidQty": "1.23400000",
                "askPrice": "65000.20",
                "askQty": "0.50000000"
            }));
        })
        .await;

    let client = BinanceClient::builder()
        .rest_base_url(server.base_url())
        .build()
        .expect("client should build");

    let response = client
        .book_ticker("BTCUSDT")
        .await
        .expect("request should succeed");

    mock.assert_async().await;
    assert_eq!(
        response,
        BookTicker {
            symbol: "BTCUSDT".to_string(),
            bid_price: "65000.10".to_string(),
            bid_qty: "1.23400000".to_string(),
            ask_price: "65000.20".to_string(),
            ask_qty: "0.50000000".to_string(),
        }
    );
}

#[tokio::test]
async fn fetches_recent_trades() {
    let server = MockServer::start_async().await;
    let mock = server
        .mock_async(|when, then| {
            when.method(GET)
                .path("/api/v3/trades")
                .query_param("symbol", "BTCUSDT")
                .query_param("limit", "2");
            then.status(200).json_body(json!([
                {
                    "id": 1_u64,
                    "price": "65000.10",
                    "qty": "0.01000000",
                    "quoteQty": "650.00100000",
                    "time": 1717171717000_u64,
                    "isBuyerMaker": true,
                    "isBestMatch": true
                },
                {
                    "id": 2_u64,
                    "price": "65000.20",
                    "qty": "0.02000000",
                    "quoteQty": "1300.00400000",
                    "time": 1717171718000_u64,
                    "isBuyerMaker": false,
                    "isBestMatch": true
                }
            ]));
        })
        .await;

    let client = BinanceClient::builder()
        .rest_base_url(server.base_url())
        .build()
        .expect("client should build");

    let response = client
        .recent_trades("BTCUSDT", Some(2))
        .await
        .expect("request should succeed");

    mock.assert_async().await;
    assert_eq!(response.len(), 2);
    assert_eq!(response[0].id, 1);
    assert_eq!(response[1].is_buyer_maker, false);
}

#[tokio::test]
async fn historical_trades_require_api_key_header() {
    let server = MockServer::start_async().await;
    let mock = server
        .mock_async(|when, then| {
            when.method(GET)
                .path("/api/v3/historicalTrades")
                .header("x-mbx-apikey", "key")
                .query_param("symbol", "BTCUSDT")
                .query_param("limit", "1")
                .query_param("fromId", "100");
            then.status(200).json_body(json!([
                {
                    "id": 100_u64,
                    "price": "64999.90",
                    "qty": "0.10000000",
                    "quoteQty": "6499.99000000",
                    "time": 1717171716000_u64,
                    "isBuyerMaker": true,
                    "isBestMatch": true
                }
            ]));
        })
        .await;

    let client = BinanceClient::builder()
        .api_key("key")
        .rest_base_url(server.base_url())
        .build()
        .expect("client should build");

    let response = client
        .historical_trades("BTCUSDT", Some(1), Some(100))
        .await
        .expect("request should succeed");

    mock.assert_async().await;
    assert_eq!(response[0].id, 100);
}

#[tokio::test]
async fn fetches_aggregate_trades() {
    let server = MockServer::start_async().await;
    let mock = server
        .mock_async(|when, then| {
            when.method(GET)
                .path("/api/v3/aggTrades")
                .query_param("symbol", "BTCUSDT")
                .query_param("limit", "1")
                .query_param("fromId", "5");
            then.status(200).json_body(json!([
                {
                    "a": 7_u64,
                    "p": "65000.00",
                    "q": "0.25000000",
                    "f": 5_u64,
                    "l": 6_u64,
                    "T": 1717171719000_u64,
                    "m": true,
                    "M": true
                }
            ]));
        })
        .await;

    let client = BinanceClient::builder()
        .rest_base_url(server.base_url())
        .build()
        .expect("client should build");

    let response = client
        .aggregate_trades("BTCUSDT", Some(1), Some(5), None, None)
        .await
        .expect("request should succeed");

    mock.assert_async().await;
    assert_eq!(
        response,
        vec![AggregateTrade {
            aggregate_trade_id: 7,
            price: "65000.00".to_string(),
            quantity: "0.25000000".to_string(),
            first_trade_id: 5,
            last_trade_id: 6,
            timestamp: 1_717_171_719_000,
            is_buyer_maker: true,
            is_best_match: true,
        }]
    );
}

#[tokio::test]
async fn fetches_average_price() {
    let server = MockServer::start_async().await;
    let mock = server
        .mock_async(|when, then| {
            when.method(GET)
                .path("/api/v3/avgPrice")
                .query_param("symbol", "BTCUSDT");
            then.status(200).json_body(json!({
                "mins": 5_u64,
                "price": "65123.45",
                "closeTime": 1717171720000_u64
            }));
        })
        .await;

    let client = BinanceClient::builder()
        .rest_base_url(server.base_url())
        .build()
        .expect("client should build");

    let response = client
        .average_price("BTCUSDT")
        .await
        .expect("request should succeed");

    mock.assert_async().await;
    assert_eq!(response.price, "65123.45");
    assert_eq!(response.close_time, Some(1_717_171_720_000));
}

#[tokio::test]
async fn fetches_ui_klines() {
    let server = MockServer::start_async().await;
    let mock = server
        .mock_async(|when, then| {
            when.method(GET)
                .path("/api/v3/uiKlines")
                .query_param("symbol", "BTCUSDT")
                .query_param("interval", "1m")
                .query_param("limit", "1");
            then.status(200).json_body(json!([[
                1717171710000_u64,
                "65000.00",
                "65010.00",
                "64990.00",
                "65005.00",
                "10.00000000",
                1717171769999_u64,
                "650050.00",
                100_u64,
                "6.00000000",
                "390030.00",
                "0"
            ]]));
        })
        .await;

    let client = BinanceClient::builder()
        .rest_base_url(server.base_url())
        .build()
        .expect("client should build");

    let response = client
        .ui_klines(
            &python_binance_rs::KlinesRequest::new("BTCUSDT", KlineInterval::OneMinute).limit(1),
        )
        .await
        .expect("request should succeed");

    mock.assert_async().await;
    assert_eq!(response.len(), 1);
}

#[tokio::test]
async fn finds_symbol_info_by_symbol() {
    let server = MockServer::start_async().await;
    let mock = server
        .mock_async(|when, then| {
            when.method(GET).path("/api/v3/exchangeInfo");
            then.status(200).json_body(json!({
                "timezone": "UTC",
                "serverTime": 1717171717000_u64,
                "symbols": [
                    { "symbol": "ETHUSDT", "status": "TRADING" },
                    { "symbol": "BTCUSDT", "status": "TRADING", "baseAsset": "BTC", "quoteAsset": "USDT" }
                ]
            }));
        })
        .await;

    let client = BinanceClient::builder()
        .rest_base_url(server.base_url())
        .build()
        .expect("client should build");

    let response = client
        .symbol_info("BTCUSDT")
        .await
        .expect("request should succeed")
        .expect("symbol should exist");

    mock.assert_async().await;
    assert_eq!(response["baseAsset"], "BTC");
    assert_eq!(response["quoteAsset"], "USDT");
}

#[tokio::test]
async fn fetches_single_24hr_ticker() {
    let server = MockServer::start_async().await;
    let mock = server
        .mock_async(|when, then| {
            when.method(GET)
                .path("/api/v3/ticker/24hr")
                .query_param("symbol", "BTCUSDT");
            then.status(200).json_body(json!({
                "symbol": "BTCUSDT",
                "priceChange": "100.00",
                "priceChangePercent": "0.15",
                "weightedAvgPrice": "64950.00",
                "prevClosePrice": "64900.00",
                "lastPrice": "65000.00",
                "lastQty": "0.10000000",
                "bidPrice": "64999.90",
                "bidQty": "1.00000000",
                "askPrice": "65000.10",
                "askQty": "0.80000000",
                "openPrice": "64900.00",
                "highPrice": "65200.00",
                "lowPrice": "64500.00",
                "volume": "1234.50000000",
                "quoteVolume": "80123456.78",
                "openTime": 1717085317000_u64,
                "closeTime": 1717171717000_u64,
                "firstId": 1_u64,
                "lastId": 999_u64,
                "count": 999_u64
            }));
        })
        .await;

    let client = BinanceClient::builder()
        .rest_base_url(server.base_url())
        .build()
        .expect("client should build");

    let response = client
        .ticker_24hr("BTCUSDT")
        .await
        .expect("request should succeed");

    mock.assert_async().await;
    assert_eq!(response.symbol, "BTCUSDT");
    assert_eq!(response.last_price, "65000.00");
    assert_eq!(response.count, 999);
}

#[tokio::test]
async fn asset_balance_returns_matching_asset() {
    let server = MockServer::start_async().await;
    let mock = server
        .mock_async(|when, then| {
            when.method(GET)
                .path("/api/v3/account")
                .header("x-mbx-apikey", "key")
                .query_param("recvWindow", "5000")
                .query_param("timestamp", "1717171717171");
            then.status(200).json_body(json!({
                "makerCommission": 15,
                "takerCommission": 15,
                "buyerCommission": 0,
                "sellerCommission": 0,
                "canTrade": true,
                "canWithdraw": true,
                "canDeposit": true,
                "updateTime": 1717171717000_u64,
                "accountType": "SPOT",
                "balances": [
                    { "asset": "BTC", "free": "0.50000000", "locked": "0.10000000" },
                    { "asset": "USDT", "free": "1000.00000000", "locked": "0.00000000" }
                ]
            }));
        })
        .await;

    let client = BinanceClient::builder()
        .api_key("key")
        .api_secret("secret")
        .fixed_timestamp(1_717_171_717_171)
        .recv_window(5_000)
        .rest_base_url(server.base_url())
        .build()
        .expect("client should build");

    let balance = client
        .asset_balance("USDT")
        .await
        .expect("request should succeed")
        .expect("asset should exist");

    mock.assert_async().await;
    assert_eq!(balance.asset, "USDT");
    assert_eq!(balance.free, "1000.00000000");
}

#[tokio::test]
async fn fetches_all_orders() {
    let server = MockServer::start_async().await;
    let mock = server
        .mock_async(|when, then| {
            when.method(GET)
                .path("/api/v3/allOrders")
                .header("x-mbx-apikey", "key")
                .query_param("symbol", "BTCUSDT")
                .query_param("limit", "2")
                .query_param("recvWindow", "5000")
                .query_param("timestamp", "1717171717171");
            then.status(200).json_body(json!([
                {
                    "symbol": "BTCUSDT",
                    "orderId": 1_u64,
                    "orderListId": -1,
                    "clientOrderId": "a",
                    "price": "65000.00",
                    "origQty": "0.01000000",
                    "executedQty": "0.00000000",
                    "cummulativeQuoteQty": "0.00000000",
                    "status": "NEW",
                    "timeInForce": "GTC",
                    "type": "LIMIT",
                    "side": "BUY"
                },
                {
                    "symbol": "BTCUSDT",
                    "orderId": 2_u64,
                    "orderListId": -1,
                    "clientOrderId": "b",
                    "price": "65100.00",
                    "origQty": "0.02000000",
                    "executedQty": "0.01000000",
                    "cummulativeQuoteQty": "651.00000000",
                    "status": "PARTIALLY_FILLED",
                    "timeInForce": "GTC",
                    "type": "LIMIT",
                    "side": "BUY"
                }
            ]));
        })
        .await;

    let client = BinanceClient::builder()
        .api_key("key")
        .api_secret("secret")
        .fixed_timestamp(1_717_171_717_171)
        .recv_window(5_000)
        .rest_base_url(server.base_url())
        .build()
        .expect("client should build");

    let response = client
        .all_orders(&AllOrdersRequest::new("BTCUSDT").limit(2))
        .await
        .expect("request should succeed");

    mock.assert_async().await;
    assert_eq!(response.len(), 2);
    assert_eq!(response[1].order_id, 2);
    assert_eq!(response[1].status.as_deref(), Some("PARTIALLY_FILLED"));
}

#[tokio::test]
async fn fetches_account_trades() {
    let server = MockServer::start_async().await;
    let mock = server
        .mock_async(|when, then| {
            when.method(GET)
                .path("/api/v3/myTrades")
                .header("x-mbx-apikey", "key")
                .query_param("symbol", "BTCUSDT")
                .query_param("orderId", "42")
                .query_param("fromId", "100")
                .query_param("limit", "1")
                .query_param("recvWindow", "5000")
                .query_param("timestamp", "1717171717171");
            then.status(200).json_body(json!([
                {
                    "symbol": "BTCUSDT",
                    "id": 28457_u64,
                    "orderId": 42_u64,
                    "orderListId": -1,
                    "price": "65000.00",
                    "qty": "0.01000000",
                    "quoteQty": "650.00000000",
                    "commission": "0.65000000",
                    "commissionAsset": "USDT",
                    "time": 1717171717000_u64,
                    "isBuyer": true,
                    "isMaker": false,
                    "isBestMatch": true
                }
            ]));
        })
        .await;

    let client = BinanceClient::builder()
        .api_key("key")
        .api_secret("secret")
        .fixed_timestamp(1_717_171_717_171)
        .recv_window(5_000)
        .rest_base_url(server.base_url())
        .build()
        .expect("client should build");

    let response = client
        .my_trades(
            &MyTradesRequest::new("BTCUSDT")
                .order_id(42)
                .from_id(100)
                .limit(1),
        )
        .await
        .expect("request should succeed");

    mock.assert_async().await;
    assert_eq!(response.len(), 1);
    assert_eq!(response[0].order_id, 42);
    assert!(response[0].is_buyer);
}

#[tokio::test]
async fn fetches_current_order_count() {
    let server = MockServer::start_async().await;
    let mock = server
        .mock_async(|when, then| {
            when.method(GET)
                .path("/api/v3/rateLimit/order")
                .header("x-mbx-apikey", "key")
                .query_param("recvWindow", "5000")
                .query_param("timestamp", "1717171717171");
            then.status(200).json_body(json!([
                {
                    "rateLimitType": "ORDERS",
                    "interval": "SECOND",
                    "intervalNum": 10,
                    "limit": 50,
                    "count": 0
                },
                {
                    "rateLimitType": "ORDERS",
                    "interval": "DAY",
                    "intervalNum": 1,
                    "limit": 160000,
                    "count": 4
                }
            ]));
        })
        .await;

    let client = BinanceClient::builder()
        .api_key("key")
        .api_secret("secret")
        .fixed_timestamp(1_717_171_717_171)
        .recv_window(5_000)
        .rest_base_url(server.base_url())
        .build()
        .expect("client should build");

    let response = client
        .current_order_count()
        .await
        .expect("request should succeed");

    mock.assert_async().await;
    assert_eq!(response.len(), 2);
    assert_eq!(response[1].count, 4);
}

#[tokio::test]
async fn fetches_open_order_lists() {
    let server = MockServer::start_async().await;
    let mock = server
        .mock_async(|when, then| {
            when.method(GET)
                .path("/api/v3/openOrderList")
                .header("x-mbx-apikey", "key")
                .query_param("recvWindow", "5000")
                .query_param("timestamp", "1717171717171");
            then.status(200).json_body(json!([
                {
                    "orderListId": 31_u64,
                    "contingencyType": "OCO",
                    "listStatusType": "EXEC_STARTED",
                    "listOrderStatus": "EXECUTING",
                    "listClientOrderId": "wuB13fmulKj3YjdqWEcsnp",
                    "transactionTime": 1565246080644_u64,
                    "symbol": "LTCBTC",
                    "orders": [
                        { "symbol": "LTCBTC", "orderId": 4_u64, "clientOrderId": "r3EH2N76dHfLoSZWIUw1bT" },
                        { "symbol": "LTCBTC", "orderId": 5_u64, "clientOrderId": "Cv1SnyPD3qhqpbjpYEHbd2" }
                    ]
                }
            ]));
        })
        .await;

    let client = BinanceClient::builder()
        .api_key("key")
        .api_secret("secret")
        .fixed_timestamp(1_717_171_717_171)
        .recv_window(5_000)
        .rest_base_url(server.base_url())
        .build()
        .expect("client should build");

    let response = client
        .open_order_lists()
        .await
        .expect("request should succeed");

    mock.assert_async().await;
    assert_eq!(response.len(), 1);
    assert_eq!(response[0].order_list_id, 31);
    assert_eq!(response[0].orders.len(), 2);
}

#[tokio::test]
async fn fetches_specific_order_list() {
    let server = MockServer::start_async().await;
    let mock = server
        .mock_async(|when, then| {
            when.method(GET)
                .path("/api/v3/orderList")
                .header("x-mbx-apikey", "key")
                .query_param("orderListId", "31")
                .query_param("recvWindow", "5000")
                .query_param("timestamp", "1717171717171");
            then.status(200).json_body(json!({
                "orderListId": 31_u64,
                "contingencyType": "OCO",
                "listStatusType": "EXEC_STARTED",
                "listOrderStatus": "EXECUTING",
                "listClientOrderId": "wuB13fmulKj3YjdqWEcsnp",
                "transactionTime": 1565246080644_u64,
                "symbol": "LTCBTC",
                "orders": [
                    { "symbol": "LTCBTC", "orderId": 4_u64, "clientOrderId": "r3EH2N76dHfLoSZWIUw1bT" },
                    { "symbol": "LTCBTC", "orderId": 5_u64, "clientOrderId": "Cv1SnyPD3qhqpbjpYEHbd2" }
                ]
            }));
        })
        .await;

    let client = BinanceClient::builder()
        .api_key("key")
        .api_secret("secret")
        .fixed_timestamp(1_717_171_717_171)
        .recv_window(5_000)
        .rest_base_url(server.base_url())
        .build()
        .expect("client should build");

    let response = client
        .order_list(&OrderListQueryRequest::by_order_list_id(31))
        .await
        .expect("request should succeed");

    mock.assert_async().await;
    assert_eq!(response.order_list_id, 31);
    assert_eq!(response.symbol, "LTCBTC");
}

#[tokio::test]
async fn fetches_all_order_lists() {
    let server = MockServer::start_async().await;
    let mock = server
        .mock_async(|when, then| {
            when.method(GET)
                .path("/api/v3/allOrderList")
                .header("x-mbx-apikey", "key")
                .query_param("fromId", "30")
                .query_param("limit", "2")
                .query_param("recvWindow", "5000")
                .query_param("timestamp", "1717171717171");
            then.status(200).json_body(json!([
                {
                    "orderListId": 31_u64,
                    "contingencyType": "OCO",
                    "listStatusType": "ALL_DONE",
                    "listOrderStatus": "ALL_DONE",
                    "listClientOrderId": "abc",
                    "transactionTime": 1565246080644_u64,
                    "symbol": "LTCBTC",
                    "orders": []
                },
                {
                    "orderListId": 32_u64,
                    "contingencyType": "OCO",
                    "listStatusType": "ALL_DONE",
                    "listOrderStatus": "ALL_DONE",
                    "listClientOrderId": "def",
                    "transactionTime": 1565246081644_u64,
                    "symbol": "ETHBTC",
                    "orders": []
                }
            ]));
        })
        .await;

    let client = BinanceClient::builder()
        .api_key("key")
        .api_secret("secret")
        .fixed_timestamp(1_717_171_717_171)
        .recv_window(5_000)
        .rest_base_url(server.base_url())
        .build()
        .expect("client should build");

    let response = client
        .all_order_lists(Some(30), None, None, Some(2))
        .await
        .expect("request should succeed");

    mock.assert_async().await;
    assert_eq!(response.len(), 2);
    assert_eq!(response[1].order_list_id, 32);
}

#[tokio::test]
async fn cancels_order_list() {
    let server = MockServer::start_async().await;
    let mock = server
        .mock_async(|when, then| {
            when.method(httpmock::Method::DELETE)
                .path("/api/v3/orderList")
                .header("x-mbx-apikey", "key")
                .query_param("symbol", "LTCBTC")
                .query_param("orderListId", "31")
                .query_param("recvWindow", "5000")
                .query_param("timestamp", "1717171717171");
            then.status(200).json_body(json!({
                "orderListId": 31_u64,
                "contingencyType": "OCO",
                "listStatusType": "ALL_DONE",
                "listOrderStatus": "ALL_DONE",
                "listClientOrderId": "wuB13fmulKj3YjdqWEcsnp",
                "transactionTime": 1565246080644_u64,
                "symbol": "LTCBTC",
                "orders": [
                    { "symbol": "LTCBTC", "orderId": 4_u64, "clientOrderId": "r3EH2N76dHfLoSZWIUw1bT" },
                    { "symbol": "LTCBTC", "orderId": 5_u64, "clientOrderId": "Cv1SnyPD3qhqpbjpYEHbd2" }
                ]
            }));
        })
        .await;

    let client = BinanceClient::builder()
        .api_key("key")
        .api_secret("secret")
        .fixed_timestamp(1_717_171_717_171)
        .recv_window(5_000)
        .rest_base_url(server.base_url())
        .build()
        .expect("client should build");

    let response = client
        .cancel_order_list(&CancelOrderListRequest::by_order_list_id("LTCBTC", 31))
        .await
        .expect("request should succeed");

    mock.assert_async().await;
    assert_eq!(response.order_list_id, 31);
    assert_eq!(response.list_status_type, "ALL_DONE");
}

#[tokio::test]
async fn cancels_all_open_orders_for_symbol() {
    let server = MockServer::start_async().await;
    let mock = server
        .mock_async(|when, then| {
            when.method(httpmock::Method::DELETE)
                .path("/api/v3/openOrders")
                .header("x-mbx-apikey", "key")
                .query_param("symbol", "BTCUSDT")
                .query_param("recvWindow", "5000")
                .query_param("timestamp", "1717171717171");
            then.status(200).json_body(json!([
                {
                    "symbol": "BTCUSDT",
                    "origClientOrderId": "E6APeyTJvkMvLMYMqu1KQ4",
                    "orderId": 11_u64,
                    "orderListId": -1,
                    "clientOrderId": "pXLV6Hz6mprAcVYpVMTGgx",
                    "price": "0.089853",
                    "origQty": "0.178622",
                    "executedQty": "0.000000",
                    "cummulativeQuoteQty": "0.000000",
                    "status": "CANCELED",
                    "timeInForce": "GTC",
                    "type": "LIMIT",
                    "side": "BUY"
                }
            ]));
        })
        .await;

    let client = BinanceClient::builder()
        .api_key("key")
        .api_secret("secret")
        .fixed_timestamp(1_717_171_717_171)
        .recv_window(5_000)
        .rest_base_url(server.base_url())
        .build()
        .expect("client should build");

    let response = client
        .cancel_open_orders("BTCUSDT")
        .await
        .expect("request should succeed");

    mock.assert_async().await;
    assert_eq!(response.as_array().expect("array response").len(), 1);
}

#[tokio::test]
async fn fetches_commission_rates() {
    let server = MockServer::start_async().await;
    let mock = server
        .mock_async(|when, then| {
            when.method(GET)
                .path("/api/v3/account/commission")
                .header("x-mbx-apikey", "key")
                .query_param("symbol", "BTCUSDT")
                .query_param("recvWindow", "5000")
                .query_param("timestamp", "1717171717171");
            then.status(200).json_body(json!({
                "symbol": "BTCUSDT",
                "standardCommission": {
                    "maker": "0.00000010",
                    "taker": "0.00000020",
                    "buyer": "0.00000030",
                    "seller": "0.00000040"
                },
                "taxCommission": {
                    "maker": "0.00000110",
                    "taker": "0.00000120",
                    "buyer": "0.00000130",
                    "seller": "0.00000140"
                },
                "discount": {
                    "enabledForAccount": true,
                    "enabledForSymbol": true,
                    "discountAsset": "BNB",
                    "discount": "0.25000000"
                }
            }));
        })
        .await;

    let client = BinanceClient::builder()
        .api_key("key")
        .api_secret("secret")
        .fixed_timestamp(1_717_171_717_171)
        .recv_window(5_000)
        .rest_base_url(server.base_url())
        .build()
        .expect("client should build");

    let response = client
        .commission_rates("BTCUSDT")
        .await
        .expect("request should succeed");

    mock.assert_async().await;
    assert_eq!(response.symbol, "BTCUSDT");
    assert_eq!(response.discount.discount_asset, "BNB");
}

#[tokio::test]
async fn fetches_prevented_matches() {
    let server = MockServer::start_async().await;
    let mock = server
        .mock_async(|when, then| {
            when.method(GET)
                .path("/api/v3/myPreventedMatches")
                .header("x-mbx-apikey", "key")
                .query_param("symbol", "BTCUSDT")
                .query_param("orderId", "5")
                .query_param("fromPreventedMatchId", "1")
                .query_param("limit", "10")
                .query_param("recvWindow", "5000")
                .query_param("timestamp", "1717171717171");
            then.status(200).json_body(json!([
                {
                    "symbol": "BTCUSDT",
                    "preventedMatchId": 1_u64,
                    "takerOrderId": 5_u64,
                    "makerSymbol": "BTCUSDT",
                    "makerOrderId": 3_u64,
                    "tradeGroupId": 1_i64,
                    "selfTradePreventionMode": "EXPIRE_MAKER",
                    "price": "1.100000",
                    "makerPreventedQuantity": "1.300000",
                    "transactTime": 1669101687094_u64
                }
            ]));
        })
        .await;

    let client = BinanceClient::builder()
        .api_key("key")
        .api_secret("secret")
        .fixed_timestamp(1_717_171_717_171)
        .recv_window(5_000)
        .rest_base_url(server.base_url())
        .build()
        .expect("client should build");

    let response = client
        .prevented_matches(
            &PreventedMatchesRequest::new("BTCUSDT")
                .order_id(5)
                .from_prevented_match_id(1)
                .limit(10),
        )
        .await
        .expect("request should succeed");

    mock.assert_async().await;
    assert_eq!(response.len(), 1);
    assert_eq!(response[0].prevented_match_id, 1);
}

#[tokio::test]
async fn fetches_order_amendments() {
    let server = MockServer::start_async().await;
    let mock = server
        .mock_async(|when, then| {
            when.method(GET)
                .path("/api/v3/order/amendments")
                .header("x-mbx-apikey", "key")
                .query_param("symbol", "BTCUSDT")
                .query_param("orderId", "42")
                .query_param("fromExecutionId", "7")
                .query_param("limit", "2")
                .query_param("recvWindow", "5000")
                .query_param("timestamp", "1717171717171");
            then.status(200).json_body(json!([
                {
                    "symbol": "BTCUSDT",
                    "orderId": 42_u64,
                    "executionId": 8_u64,
                    "origClientOrderId": "old",
                    "newClientOrderId": "new",
                    "origQty": "1.00000000",
                    "newQty": "0.80000000",
                    "origPrice": "65000.00",
                    "newPrice": "64950.00",
                    "time": 1717171717000_u64
                }
            ]));
        })
        .await;

    let client = BinanceClient::builder()
        .api_key("key")
        .api_secret("secret")
        .fixed_timestamp(1_717_171_717_171)
        .recv_window(5_000)
        .rest_base_url(server.base_url())
        .build()
        .expect("client should build");

    let response = client
        .order_amendments(
            &OrderAmendmentsRequest::new("BTCUSDT", 42)
                .from_execution_id(7)
                .limit(2),
        )
        .await
        .expect("request should succeed");

    mock.assert_async().await;
    assert_eq!(response.len(), 1);
    assert_eq!(response[0].execution_id, 8);
    assert_eq!(response[0].new_client_order_id, "new");
}

#[tokio::test]
async fn fetches_symbol_filters() {
    let server = MockServer::start_async().await;
    let mock = server
        .mock_async(|when, then| {
            when.method(GET)
                .path("/api/v3/myFilters")
                .header("x-mbx-apikey", "key")
                .query_param("symbol", "BTCUSDT")
                .query_param("recvWindow", "5000")
                .query_param("timestamp", "1717171717171");
            then.status(200).json_body(json!({
                "symbol": "BTCUSDT",
                "filters": [
                    { "filterType": "PRICE_FILTER", "minPrice": "0.01000000", "tickSize": "0.01000000" },
                    { "filterType": "LOT_SIZE", "minQty": "0.00001000", "stepSize": "0.00001000" }
                ]
            }));
        })
        .await;

    let client = BinanceClient::builder()
        .api_key("key")
        .api_secret("secret")
        .fixed_timestamp(1_717_171_717_171)
        .recv_window(5_000)
        .rest_base_url(server.base_url())
        .build()
        .expect("client should build");

    let response = client
        .symbol_filters("BTCUSDT")
        .await
        .expect("request should succeed");

    mock.assert_async().await;
    assert_eq!(response.symbol, "BTCUSDT");
    assert_eq!(response.filters.len(), 2);
}

#[tokio::test]
async fn creates_oco_order_list() {
    let server = MockServer::start_async().await;
    let mock = server
        .mock_async(|when, then| {
            when.method(POST)
                .path("/api/v3/orderList/oco")
                .header("x-mbx-apikey", "key")
                .header("content-type", "application/x-www-form-urlencoded")
                .body_contains("symbol=BTCUSDT")
                .body_contains("side=SELL")
                .body_contains("quantity=0.10000000")
                .body_contains("aboveType=LIMIT_MAKER")
                .body_contains("belowType=STOP_LOSS_LIMIT")
                .body_contains("abovePrice=70000.00")
                .body_contains("belowPrice=64000.00")
                .body_contains("belowStopPrice=64500.00");
            then.status(200).json_body(json!({
                "orderListId": 33_u64,
                "contingencyType": "OCO",
                "listStatusType": "EXEC_STARTED",
                "listOrderStatus": "EXECUTING",
                "listClientOrderId": "oco-1",
                "transactionTime": 1717171717000_u64,
                "symbol": "BTCUSDT",
                "orders": [
                    { "symbol": "BTCUSDT", "orderId": 101_u64, "clientOrderId": "above-leg" },
                    { "symbol": "BTCUSDT", "orderId": 102_u64, "clientOrderId": "below-leg" }
                ]
            }));
        })
        .await;

    let client = BinanceClient::builder()
        .api_key("key")
        .api_secret("secret")
        .fixed_timestamp(1_717_171_717_171)
        .recv_window(5_000)
        .rest_base_url(server.base_url())
        .build()
        .expect("client should build");

    let request =
        CreateOcoOrderRequest::sell("BTCUSDT", "0.10000000", "70000.00", "64500.00", "64000.00");

    let response = client
        .create_oco_order(&request)
        .await
        .expect("request should succeed");

    mock.assert_async().await;
    assert_eq!(response.order_list_id, 33);
    assert_eq!(response.orders.len(), 2);
}

#[tokio::test]
async fn cancel_replace_order_uses_signed_body() {
    let server = MockServer::start_async().await;
    let mock = server
        .mock_async(|when, then| {
            when.method(POST)
                .path("/api/v3/order/cancelReplace")
                .header("x-mbx-apikey", "key")
                .header("content-type", "application/x-www-form-urlencoded")
                .body_contains("symbol=BTCUSDT")
                .body_contains("side=BUY")
                .body_contains("type=LIMIT")
                .body_contains("timeInForce=GTC")
                .body_contains("quantity=0.01000000")
                .body_contains("price=64000.00")
                .body_contains("cancelReplaceMode=STOP_ON_FAILURE")
                .body_contains("cancelOrderId=42");
            then.status(200).json_body(json!({
                "cancelResult": "SUCCESS",
                "newOrderResult": "SUCCESS",
                "cancelResponse": {
                    "symbol": "BTCUSDT",
                    "orderId": 42_u64,
                    "orderListId": -1,
                    "clientOrderId": "old-order",
                    "price": "65000.00",
                    "origQty": "0.01000000",
                    "executedQty": "0.00000000",
                    "cummulativeQuoteQty": "0.00000000",
                    "status": "CANCELED",
                    "timeInForce": "GTC",
                    "type": "LIMIT",
                    "side": "BUY"
                },
                "newOrderResponse": {
                    "symbol": "BTCUSDT",
                    "orderId": 43_u64,
                    "orderListId": -1,
                    "clientOrderId": "new-order",
                    "transactTime": 1717171717000_u64,
                    "price": "64000.00",
                    "origQty": "0.01000000",
                    "executedQty": "0.00000000",
                    "cummulativeQuoteQty": "0.00000000",
                    "status": "NEW",
                    "timeInForce": "GTC",
                    "type": "LIMIT",
                    "side": "BUY"
                }
            }));
        })
        .await;

    let client = BinanceClient::builder()
        .api_key("key")
        .api_secret("secret")
        .fixed_timestamp(1_717_171_717_171)
        .recv_window(5_000)
        .rest_base_url(server.base_url())
        .build()
        .expect("client should build");

    let new_order = CreateOrderRequest::limit(
        "BTCUSDT",
        OrderSide::Buy,
        "0.01000000",
        "64000.00",
        TimeInForce::Gtc,
    );

    let response = client
        .cancel_replace_order(42, CancelReplaceMode::StopOnFailure, &new_order)
        .await
        .expect("request should succeed");

    mock.assert_async().await;
    assert_eq!(response.cancel_result, "SUCCESS");
    assert_eq!(response.new_order_result, "SUCCESS");
    assert_eq!(
        response
            .new_order_response
            .expect("new order response should exist")
            .order_id,
        43
    );
}
