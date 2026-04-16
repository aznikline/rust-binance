use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use futures_util::{SinkExt, StreamExt};
use hmac::{Hmac, Mac};
use serde::de::DeserializeOwned;
use sha2::Sha256;
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};

use crate::{Error, KlineInterval};

const DEFAULT_WS_BASE_URL: &str = "wss://stream.binance.com:9443";
const DEFAULT_WS_API_BASE_URL: &str = "wss://ws-api.binance.com:443/ws-api/v3";
const DEFAULT_RECV_WINDOW: u64 = 5_000;

pub type WsConnection = WebSocketStream<MaybeTlsStream<TcpStream>>;
type HmacSha256 = Hmac<Sha256>;
type TimestampProvider = Arc<dyn Fn() -> u64 + Send + Sync>;

#[derive(Debug, Clone)]
pub struct BinanceWebsocketClient {
    base_url: String,
}

#[derive(Clone)]
pub struct BinanceWebsocketApiClient {
    base_url: String,
    api_key: Option<String>,
    api_secret: Option<String>,
    recv_window: u64,
    timestamp_provider: TimestampProvider,
}

#[derive(Debug, Clone)]
pub struct BinanceWebsocketClientBuilder {
    base_url: String,
}

#[derive(Clone)]
pub struct BinanceWebsocketApiClientBuilder {
    base_url: String,
    api_key: Option<String>,
    api_secret: Option<String>,
    recv_window: Option<u64>,
    timestamp_provider: Option<TimestampProvider>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct CombinedStream<T> {
    pub stream: String,
    pub data: T,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct BinanceWebsocketApiRequest<T = serde_json::Value> {
    pub id: String,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<T>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct BinanceWebsocketApiResponse<T> {
    pub id: String,
    pub status: u16,
    #[serde(default)]
    pub result: T,
    #[serde(rename = "rateLimits")]
    pub rate_limits: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct UserDataStreamEvent {
    #[serde(rename = "subscriptionId")]
    pub subscription_id: u64,
    pub event: UserDataEvent,
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

    pub fn api_builder() -> BinanceWebsocketApiClientBuilder {
        BinanceWebsocketApiClientBuilder {
            base_url: DEFAULT_WS_API_BASE_URL.to_string(),
            api_key: None,
            api_secret: None,
            recv_window: None,
            timestamp_provider: None,
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

impl BinanceWebsocketApiClient {
    pub fn builder() -> BinanceWebsocketApiClientBuilder {
        BinanceWebsocketApiClientBuilder {
            base_url: DEFAULT_WS_API_BASE_URL.to_string(),
            api_key: None,
            api_secret: None,
            recv_window: None,
            timestamp_provider: None,
        }
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub async fn connect(&self) -> Result<WsConnection, Error> {
        let (stream, _) = connect_async(self.base_url()).await?;
        Ok(stream)
    }

    pub fn signed_request(
        &self,
        id: impl Into<String>,
        method: impl Into<String>,
        params: serde_json::Value,
    ) -> Result<BinanceWebsocketApiRequest<serde_json::Value>, Error> {
        let api_key = self.api_key.as_deref().ok_or(Error::MissingCredentials)?;
        let api_secret = self.api_secret.as_deref().ok_or(Error::MissingCredentials)?;
        let mut params = ensure_object(params);
        params.entry("apiKey".to_string()).or_insert(api_key.into());
        params
            .entry("recvWindow".to_string())
            .or_insert(self.recv_window.into());
        params
            .entry("timestamp".to_string())
            .or_insert((self.timestamp_provider)().into());

        let signature_payload = canonicalize_params(&params);
        let signature = sign(api_secret, &signature_payload)?;
        params.insert("signature".to_string(), signature.into());

        Ok(BinanceWebsocketApiRequest {
            id: id.into(),
            method: method.into(),
            params: Some(serde_json::Value::Object(params)),
        })
    }

    pub fn session_logon_request(
        &self,
        id: impl Into<String>,
    ) -> Result<BinanceWebsocketApiRequest<serde_json::Value>, Error> {
        self.signed_request(id, "session.logon", serde_json::json!({}))
    }

    pub async fn send_request<T: serde::Serialize>(
        &self,
        connection: &mut WsConnection,
        request: &T,
    ) -> Result<(), Error> {
        let payload = serde_json::to_string(request)?;
        connection.send(Message::Text(payload.into())).await?;
        Ok(())
    }

    pub async fn read_response<T: DeserializeOwned>(
        &self,
        connection: &mut WsConnection,
    ) -> Result<T, Error> {
        while let Some(message) = connection.next().await {
            let message = message?;
            match message {
                Message::Text(text) => return Ok(serde_json::from_str(&text)?),
                Message::Binary(bytes) => return Ok(serde_json::from_slice(&bytes)?),
                Message::Ping(_) | Message::Pong(_) => continue,
                Message::Close(_) => {
                    return Err(tokio_tungstenite::tungstenite::Error::ConnectionClosed.into())
                }
                Message::Frame(_) => continue,
            }
        }
        Err(tokio_tungstenite::tungstenite::Error::ConnectionClosed.into())
    }

    pub async fn request<T, R>(
        &self,
        connection: &mut WsConnection,
        request: &T,
    ) -> Result<R, Error>
    where
        T: serde::Serialize,
        R: DeserializeOwned,
    {
        self.send_request(connection, request).await?;
        self.read_response(connection).await
    }
}

impl BinanceWebsocketApiClientBuilder {
    pub fn base_url(mut self, value: impl Into<String>) -> Self {
        self.base_url = value.into().trim_end_matches('/').to_string();
        self
    }

    pub fn api_key(mut self, value: impl Into<String>) -> Self {
        self.api_key = Some(value.into());
        self
    }

    pub fn api_secret(mut self, value: impl Into<String>) -> Self {
        self.api_secret = Some(value.into());
        self
    }

    pub fn recv_window(mut self, value: u64) -> Self {
        self.recv_window = Some(value);
        self
    }

    pub fn fixed_timestamp(mut self, value: u64) -> Self {
        self.timestamp_provider = Some(Arc::new(move || value));
        self
    }

    pub fn build(self) -> BinanceWebsocketApiClient {
        BinanceWebsocketApiClient {
            base_url: self.base_url,
            api_key: self.api_key,
            api_secret: self.api_secret,
            recv_window: self.recv_window.unwrap_or(DEFAULT_RECV_WINDOW),
            timestamp_provider: self.timestamp_provider.unwrap_or_else(default_timestamp_provider),
        }
    }
}

impl BinanceWebsocketApiRequest<serde_json::Value> {
    pub fn new(id: impl Into<String>, method: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            method: method.into(),
            params: None,
        }
    }

    pub fn with_params(
        id: impl Into<String>,
        method: impl Into<String>,
        params: serde_json::Value,
    ) -> Self {
        Self {
            id: id.into(),
            method: method.into(),
            params: Some(params),
        }
    }

    pub fn user_data_subscribe(id: impl Into<String>) -> Self {
        Self::new(id, "userDataStream.subscribe")
    }

    pub fn user_data_subscribe_listen_token(
        id: impl Into<String>,
        listen_token: impl Into<String>,
    ) -> Self {
        Self::with_params(
            id,
            "userDataStream.subscribe.listenToken",
            serde_json::json!({ "listenToken": listen_token.into() }),
        )
    }
}

fn normalize_symbol(symbol: &str) -> String {
    symbol.to_ascii_lowercase()
}

fn default_timestamp_provider() -> TimestampProvider {
    Arc::new(|| {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_millis() as u64
    })
}

fn ensure_object(value: serde_json::Value) -> serde_json::Map<String, serde_json::Value> {
    match value {
        serde_json::Value::Object(map) => map,
        _ => serde_json::Map::new(),
    }
}

fn canonicalize_params(params: &serde_json::Map<String, serde_json::Value>) -> String {
    let mut pairs = params
        .iter()
        .map(|(key, value)| (key.clone(), json_value_to_param_string(value)))
        .collect::<Vec<_>>();
    pairs.sort_by(|left, right| left.0.cmp(&right.0));
    serde_urlencoded::to_string(pairs).expect("ws api params should serialize")
}

fn json_value_to_param_string(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Null => String::new(),
        serde_json::Value::Bool(v) => v.to_string(),
        serde_json::Value::Number(v) => v.to_string(),
        serde_json::Value::String(v) => v.clone(),
        other => other.to_string(),
    }
}

fn sign(secret: &str, payload: &str) -> Result<String, Error> {
    let mut signer =
        HmacSha256::new_from_slice(secret.as_bytes()).map_err(|_| Error::InvalidSecretKey)?;
    signer.update(payload.as_bytes());
    Ok(hex::encode(signer.finalize().into_bytes()))
}
