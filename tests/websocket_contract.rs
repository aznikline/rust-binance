use python_binance_rs::{
    BinanceWebsocketApiClient, BinanceWebsocketApiRequest, BinanceWebsocketApiResponse,
    BinanceWebsocketClient, BookTickerStreamEvent, CombinedStream, KlineInterval, KlineStreamEvent,
    TradeStreamEvent, UserDataEvent, UserDataStreamEvent,
};

#[test]
fn builds_raw_and_combined_stream_urls() {
    let ws = BinanceWebsocketClient::builder()
        .base_url("wss://stream.binance.com:9443")
        .build();

    assert_eq!(
        ws.raw_stream_url(&BinanceWebsocketClient::trade_stream("BTCUSDT")),
        "wss://stream.binance.com:9443/ws/btcusdt@trade"
    );
    assert_eq!(
        ws.combined_stream_url([
            BinanceWebsocketClient::trade_stream("BTCUSDT"),
            BinanceWebsocketClient::book_ticker_stream("ETHUSDT"),
        ]),
        "wss://stream.binance.com:9443/stream?streams=btcusdt@trade/ethusdt@bookTicker"
    );
}

#[test]
fn builds_kline_stream_name() {
    assert_eq!(
        BinanceWebsocketClient::kline_stream("BTCUSDT", KlineInterval::OneMinute),
        "btcusdt@kline_1m"
    );
}

#[test]
fn parses_trade_stream_event() {
    let event: TradeStreamEvent = serde_json::from_str(
        r#"{
            "e":"trade",
            "E":123456789,
            "s":"BNBBTC",
            "t":12345,
            "p":"0.001",
            "q":"100",
            "T":123456785,
            "m":true,
            "M":true
        }"#,
    )
    .expect("trade event should parse");

    assert_eq!(event.symbol, "BNBBTC");
    assert_eq!(event.trade_id, 12345);
    assert!(event.is_buyer_maker);
}

#[test]
fn parses_combined_book_ticker_event() {
    let event: CombinedStream<BookTickerStreamEvent> = serde_json::from_str(
        r#"{
            "stream":"btcusdt@bookTicker",
            "data":{
                "u":400900217,
                "s":"BTCUSDT",
                "b":"0.0024",
                "B":"10",
                "a":"0.0026",
                "A":"100"
            }
        }"#,
    )
    .expect("combined event should parse");

    assert_eq!(event.stream, "btcusdt@bookTicker");
    assert_eq!(event.data.symbol, "BTCUSDT");
    assert_eq!(event.data.best_bid_price, "0.0024");
}

#[test]
fn parses_kline_stream_event() {
    let event: KlineStreamEvent = serde_json::from_str(
        r#"{
            "e":"kline",
            "E":1672515782136,
            "s":"BNBBTC",
            "k":{
                "t":1672515780000,
                "T":1672515839999,
                "s":"BNBBTC",
                "i":"1m",
                "f":100,
                "L":200,
                "o":"0.0010",
                "c":"0.0020",
                "h":"0.0025",
                "l":"0.0015",
                "v":"1000",
                "n":100,
                "x":false,
                "q":"1.0000",
                "V":"500",
                "Q":"0.500",
                "B":"123456"
            }
        }"#,
    )
    .expect("kline event should parse");

    assert_eq!(event.symbol, "BNBBTC");
    assert_eq!(event.kline.interval, "1m");
    assert!(!event.kline.is_closed);
}

#[test]
fn parses_user_data_execution_report() {
    let event: UserDataEvent = serde_json::from_str(
        r#"{
            "e":"executionReport",
            "E":1499405658658,
            "s":"ETHBTC",
            "c":"mUvoqJxFIILMdfAW5iGSOW",
            "S":"BUY",
            "o":"LIMIT",
            "f":"GTC",
            "q":"1.00000000",
            "p":"0.10264410",
            "x":"NEW",
            "X":"NEW",
            "i":4293153,
            "l":"0.00000000",
            "z":"0.00000000",
            "L":"0.00000000",
            "n":"0",
            "N":null,
            "T":1499405658657,
            "t":-1,
            "m":false,
            "w":true
        }"#,
    )
    .expect("execution report should parse");

    match event {
        UserDataEvent::ExecutionReport(report) => {
            assert_eq!(report.symbol, "ETHBTC");
            assert_eq!(report.order_id, 4_293_153);
            assert_eq!(report.current_execution_type, "NEW");
        }
        other => panic!("unexpected event: {other:?}"),
    }
}

#[test]
fn builds_ws_api_url() {
    let ws_api = BinanceWebsocketApiClient::builder()
        .base_url("wss://ws-api.binance.com:443/ws-api/v3")
        .build();

    assert_eq!(
        ws_api.base_url(),
        "wss://ws-api.binance.com:443/ws-api/v3"
    );
}

#[test]
fn serializes_user_data_subscribe_request() {
    let request = BinanceWebsocketApiRequest::user_data_subscribe("req-1");
    let value = serde_json::to_value(&request).expect("request should serialize");

    assert_eq!(value["id"], "req-1");
    assert_eq!(value["method"], "userDataStream.subscribe");
    assert!(value.get("params").is_none());
}

#[test]
fn serializes_user_data_listen_token_request() {
    let request =
        BinanceWebsocketApiRequest::user_data_subscribe_listen_token("req-2", "listen-token");
    let value = serde_json::to_value(&request).expect("request should serialize");

    assert_eq!(value["id"], "req-2");
    assert_eq!(value["method"], "userDataStream.subscribe.listenToken");
    assert_eq!(value["params"]["listenToken"], "listen-token");
}

#[test]
fn parses_ws_api_response_envelope() {
    let response: BinanceWebsocketApiResponse<serde_json::Value> = serde_json::from_str(
        r#"{
            "id":"abc",
            "status":200,
            "result":{"subscriptionId":7},
            "rateLimits":[{"rateLimitType":"REQUEST_WEIGHT","interval":"MINUTE","intervalNum":1,"limit":6000,"count":2}]
        }"#,
    )
    .expect("response should parse");

    assert_eq!(response.id, "abc");
    assert_eq!(response.status, 200);
    assert_eq!(response.result["subscriptionId"], 7);
    assert_eq!(response.rate_limits.expect("rate limits").len(), 1);
}

#[test]
fn parses_subscription_wrapped_user_data_event() {
    let event: UserDataStreamEvent = serde_json::from_str(
        r#"{
            "subscriptionId":42,
            "event":{
                "e":"executionReport",
                "E":1499405658658,
                "s":"ETHBTC",
                "c":"mUvoqJxFIILMdfAW5iGSOW",
                "S":"BUY",
                "o":"LIMIT",
                "f":"GTC",
                "q":"1.00000000",
                "p":"0.10264410",
                "x":"NEW",
                "X":"NEW",
                "i":4293153,
                "l":"0.00000000",
                "z":"0.00000000",
                "L":"0.00000000",
                "n":"0",
                "N":null,
                "T":1499405658657,
                "t":-1,
                "m":false,
                "w":true
            }
        }"#,
    )
    .expect("wrapped event should parse");

    assert_eq!(event.subscription_id, 42);
    match event.event {
        UserDataEvent::ExecutionReport(report) => assert_eq!(report.symbol, "ETHBTC"),
        other => panic!("unexpected event: {other:?}"),
    }
}

#[test]
fn signs_ws_api_session_logon_request() {
    let ws_api = BinanceWebsocketApiClient::builder()
        .api_key("key")
        .api_secret("secret")
        .fixed_timestamp(1_717_171_717_171)
        .recv_window(5_000)
        .build();

    let request = ws_api
        .signed_request("req-3", "session.logon", serde_json::json!({}))
        .expect("signed request should build");
    let value = serde_json::to_value(&request).expect("request should serialize");

    assert_eq!(value["id"], "req-3");
    assert_eq!(value["method"], "session.logon");
    assert_eq!(value["params"]["apiKey"], "key");
    assert_eq!(value["params"]["recvWindow"], 5000);
    assert_eq!(value["params"]["timestamp"], 1717171717171_u64);
    assert_eq!(
        value["params"]["signature"],
        "66260eb1fe51f30b376af48de319274b952460f1dc9531c4cf35bcadca2d061f"
    );
}

#[test]
fn parses_user_data_subscribe_response() {
    let response: BinanceWebsocketApiResponse<serde_json::Value> = serde_json::from_str(
        r#"{
            "id":"req-4",
            "status":200,
            "result":{"subscriptionId":42}
        }"#,
    )
    .expect("response should parse");

    assert_eq!(response.id, "req-4");
    assert_eq!(response.result["subscriptionId"], 42);
}
