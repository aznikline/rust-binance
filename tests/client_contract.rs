use httpmock::Method::{GET, POST};
use httpmock::MockServer;
use python_binance_rs::{
    BinanceClient, CancelOrderRequest, CreateOrderRequest, Error, KlineInterval, OrderSide,
    OrderType, TimeInForce,
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
