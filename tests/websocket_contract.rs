use python_binance_rs::{
    BinanceWebsocketClient, BookTickerStreamEvent, CombinedStream, KlineInterval, KlineStreamEvent,
    TradeStreamEvent, UserDataEvent,
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
