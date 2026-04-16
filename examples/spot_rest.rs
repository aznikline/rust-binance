use python_binance_rs::{
    BinanceClient, CreateOrderRequest, KlineInterval, KlinesRequest, OrderSide, TimeInForce,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = BinanceClient::builder()
        .api_key(std::env::var("BINANCE_API_KEY")?)
        .api_secret(std::env::var("BINANCE_API_SECRET")?)
        .build()?;

    let server_time = client.server_time().await?;
    println!("server time: {}", server_time.server_time);

    let candles = client
        .klines(&KlinesRequest::new("BTCUSDT", KlineInterval::OneMinute).limit(5))
        .await?;
    println!("fetched {} klines", candles.len());

    let request = CreateOrderRequest::limit(
        "BTCUSDT",
        OrderSide::Buy,
        "0.00100000",
        "50000.00",
        TimeInForce::Gtc,
    );

    println!(
        "signed order payload preview: {}",
        client.debug_signed_query(&request)?
    );

    Ok(())
}
