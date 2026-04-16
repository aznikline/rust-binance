use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};

use crate::{Error, KlineInterval};

const DEFAULT_WS_BASE_URL: &str = "wss://stream.binance.com:9443";

pub type WsConnection = WebSocketStream<MaybeTlsStream<TcpStream>>;

#[derive(Debug, Clone)]
pub struct BinanceWebsocketClient {
    base_url: String,
}

#[derive(Debug, Clone)]
pub struct BinanceWebsocketClientBuilder {
    base_url: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct CombinedStream<T> {
    pub stream: String,
    pub data: T,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct TradeStreamEvent {
    #[serde(rename = "e")]
    pub event_type: String,
    #[serde(rename = "E")]
    pub event_time: u64,
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "t")]
    pub trade_id: u64,
    #[serde(rename = "p")]
    pub price: String,
    #[serde(rename = "q")]
    pub quantity: String,
    #[serde(rename = "T")]
    pub trade_time: u64,
    #[serde(rename = "m")]
    pub is_buyer_maker: bool,
    #[serde(rename = "M")]
    pub is_best_match: bool,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct BookTickerStreamEvent {
    #[serde(rename = "u")]
    pub update_id: u64,
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "b")]
    pub best_bid_price: String,
    #[serde(rename = "B")]
    pub best_bid_quantity: String,
    #[serde(rename = "a")]
    pub best_ask_price: String,
    #[serde(rename = "A")]
    pub best_ask_quantity: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct KlineStreamEvent {
    #[serde(rename = "e")]
    pub event_type: String,
    #[serde(rename = "E")]
    pub event_time: u64,
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "k")]
    pub kline: KlinePayload,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct KlinePayload {
    #[serde(rename = "t")]
    pub start_time: u64,
    #[serde(rename = "T")]
    pub close_time: u64,
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "i")]
    pub interval: String,
    #[serde(rename = "f")]
    pub first_trade_id: u64,
    #[serde(rename = "L")]
    pub last_trade_id: u64,
    #[serde(rename = "o")]
    pub open_price: String,
    #[serde(rename = "c")]
    pub close_price: String,
    #[serde(rename = "h")]
    pub high_price: String,
    #[serde(rename = "l")]
    pub low_price: String,
    #[serde(rename = "v")]
    pub base_asset_volume: String,
    #[serde(rename = "n")]
    pub trade_count: u64,
    #[serde(rename = "x")]
    pub is_closed: bool,
    #[serde(rename = "q")]
    pub quote_asset_volume: String,
    #[serde(rename = "V")]
    pub taker_buy_base_asset_volume: String,
    #[serde(rename = "Q")]
    pub taker_buy_quote_asset_volume: String,
    #[serde(rename = "B")]
    pub ignore: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(tag = "e")]
pub enum UserDataEvent {
    #[serde(rename = "executionReport")]
    ExecutionReport(ExecutionReportEvent),
    #[serde(rename = "outboundAccountPosition")]
    OutboundAccountPosition(OutboundAccountPositionEvent),
    #[serde(rename = "balanceUpdate")]
    BalanceUpdate(BalanceUpdateEvent),
    #[serde(rename = "listStatus")]
    ListStatus(ListStatusEvent),
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ExecutionReportEvent {
    #[serde(rename = "E")]
    pub event_time: u64,
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "c")]
    pub client_order_id: String,
    #[serde(rename = "S")]
    pub side: String,
    #[serde(rename = "o")]
    pub order_type: String,
    #[serde(rename = "f")]
    pub time_in_force: String,
    #[serde(rename = "q")]
    pub order_quantity: String,
    #[serde(rename = "p")]
    pub order_price: String,
    #[serde(rename = "x")]
    pub current_execution_type: String,
    #[serde(rename = "X")]
    pub current_order_status: String,
    #[serde(rename = "i")]
    pub order_id: u64,
    #[serde(rename = "l")]
    pub last_executed_quantity: String,
    #[serde(rename = "z")]
    pub cumulative_filled_quantity: String,
    #[serde(rename = "L")]
    pub last_executed_price: String,
    #[serde(rename = "n")]
    pub commission_amount: String,
    #[serde(rename = "N")]
    pub commission_asset: Option<String>,
    #[serde(rename = "T")]
    pub transaction_time: u64,
    #[serde(rename = "t")]
    pub trade_id: i64,
    #[serde(rename = "m")]
    pub is_maker_side: bool,
    #[serde(rename = "w")]
    pub is_on_book: bool,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct OutboundAccountPositionEvent {
    #[serde(rename = "E")]
    pub event_time: u64,
    #[serde(rename = "u")]
    pub last_account_update_time: u64,
    #[serde(rename = "B")]
    pub balances: Vec<UserBalance>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct UserBalance {
    #[serde(rename = "a")]
    pub asset: String,
    #[serde(rename = "f")]
    pub free: String,
    #[serde(rename = "l")]
    pub locked: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct BalanceUpdateEvent {
    #[serde(rename = "E")]
    pub event_time: u64,
    #[serde(rename = "a")]
    pub asset: String,
    #[serde(rename = "d")]
    pub balance_delta: String,
    #[serde(rename = "T")]
    pub clear_time: u64,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ListStatusEvent {
    #[serde(rename = "E")]
    pub event_time: u64,
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "g")]
    pub order_list_id: u64,
    #[serde(rename = "c")]
    pub contingency_type: String,
    #[serde(rename = "l")]
    pub list_status_type: String,
    #[serde(rename = "L")]
    pub list_order_status: String,
    #[serde(rename = "C")]
    pub list_client_order_id: String,
    #[serde(rename = "O")]
    pub orders: Vec<ListStatusOrder>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ListStatusOrder {
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "i")]
    pub order_id: u64,
    #[serde(rename = "c")]
    pub client_order_id: String,
}

impl BinanceWebsocketClient {
    pub fn builder() -> BinanceWebsocketClientBuilder {
        BinanceWebsocketClientBuilder {
            base_url: DEFAULT_WS_BASE_URL.to_string(),
        }
    }

    pub fn trade_stream(symbol: &str) -> String {
        format!("{}@trade", normalize_symbol(symbol))
    }

    pub fn book_ticker_stream(symbol: &str) -> String {
        format!("{}@bookTicker", normalize_symbol(symbol))
    }

    pub fn kline_stream(symbol: &str, interval: KlineInterval) -> String {
        format!("{}@kline_{}", normalize_symbol(symbol), interval)
    }

    pub fn raw_stream_url(&self, stream_name: &str) -> String {
        format!("{}/ws/{}", self.base_url, stream_name)
    }

    pub fn combined_stream_url<I, S>(&self, streams: I) -> String
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let joined = streams
            .into_iter()
            .map(|stream| stream.as_ref().to_string())
            .collect::<Vec<_>>()
            .join("/");
        format!("{}/stream?streams={joined}", self.base_url)
    }

    pub async fn connect_raw(&self, stream_name: &str) -> Result<WsConnection, Error> {
        let (stream, _) = connect_async(self.raw_stream_url(stream_name)).await?;
        Ok(stream)
    }

    pub async fn connect_combined<I, S>(&self, streams: I) -> Result<WsConnection, Error>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let (stream, _) = connect_async(self.combined_stream_url(streams)).await?;
        Ok(stream)
    }
}

impl BinanceWebsocketClientBuilder {
    pub fn base_url(mut self, value: impl Into<String>) -> Self {
        self.base_url = value.into().trim_end_matches('/').to_string();
        self
    }

    pub fn build(self) -> BinanceWebsocketClient {
        BinanceWebsocketClient {
            base_url: self.base_url,
        }
    }
}

fn normalize_symbol(symbol: &str) -> String {
    symbol.to_ascii_lowercase()
}
