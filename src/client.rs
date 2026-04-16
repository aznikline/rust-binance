use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use hmac::{Hmac, Mac};
use reqwest::Method;
use serde::de::DeserializeOwned;
use sha2::Sha256;

use crate::error::{BinanceApiError, Error};
use crate::types::{
    AccountBalance, AccountInformation, AccountTrade, AggregateTrade, AggregateTradeRequest,
    AllOrdersRequest, AveragePrice, BookTicker, CancelOrderListRequest, CancelOrderRequest,
    CancelReplaceMode, CancelReplaceResponse, CommissionRates, CreateOcoOrderRequest,
    CreateOrderRequest, ExchangeInfo, Kline, KlinesRequest, MyTradesRequest, OrderAmendment,
    OrderAmendmentsRequest, OrderBook, OrderCountUsage, OrderListQueryRequest, OrderListSummary,
    OrderQueryRequest, OrderResponse, PreventedMatch, PreventedMatchesRequest, PriceTicker,
    ServerTimeResponse, SymbolFilters, Ticker24hr, ToParams, Trade,
};

type HmacSha256 = Hmac<Sha256>;
type TimestampProvider = Arc<dyn Fn() -> u64 + Send + Sync>;

const API_KEY_HEADER: &str = "X-MBX-APIKEY";
const DEFAULT_RECV_WINDOW: u64 = 5_000;
const DEFAULT_BASE_URL: &str = "https://api.binance.com";

#[derive(Clone)]
pub struct BinanceClient {
    http: reqwest::Client,
    api_key: Option<String>,
    api_secret: Option<String>,
    rest_base_url: String,
    recv_window: u64,
    timestamp_provider: TimestampProvider,
}

#[derive(Default)]
pub struct BinanceClientBuilder {
    api_key: Option<String>,
    api_secret: Option<String>,
    rest_base_url: Option<String>,
    recv_window: Option<u64>,
    timestamp_provider: Option<TimestampProvider>,
}

impl BinanceClient {
    pub fn builder() -> BinanceClientBuilder {
        BinanceClientBuilder::default()
    }

    pub async fn ping(&self) -> Result<(), Error> {
        self.send_public::<serde_json::Value>(Method::GET, "/api/v3/ping", Vec::new())
            .await
            .map(|_| ())
    }

    pub async fn server_time(&self) -> Result<ServerTimeResponse, Error> {
        self.send_public(Method::GET, "/api/v3/time", Vec::new())
            .await
    }

    pub async fn exchange_info(&self, symbol: Option<&str>) -> Result<ExchangeInfo, Error> {
        let params = optional_params([("symbol", symbol.map(ToOwned::to_owned))]);
        self.send_public(Method::GET, "/api/v3/exchangeInfo", params)
            .await
    }

    pub async fn ticker_price(&self, symbol: Option<&str>) -> Result<Vec<PriceTicker>, Error> {
        let params = optional_params([("symbol", symbol.map(ToOwned::to_owned))]);
        self.send_public(Method::GET, "/api/v3/ticker/price", params)
            .await
    }

    pub async fn book_ticker(&self, symbol: &str) -> Result<BookTicker, Error> {
        let params = optional_params([("symbol", Some(symbol.to_owned()))]);
        self.send_public(Method::GET, "/api/v3/ticker/bookTicker", params)
            .await
    }

    pub async fn book_tickers(&self) -> Result<Vec<BookTicker>, Error> {
        self.send_public(Method::GET, "/api/v3/ticker/bookTicker", Vec::new())
            .await
    }

    pub async fn recent_trades(
        &self,
        symbol: &str,
        limit: Option<u16>,
    ) -> Result<Vec<Trade>, Error> {
        let params = optional_params([
            ("symbol", Some(symbol.to_owned())),
            ("limit", limit.map(|value| value.to_string())),
        ]);
        self.send_public(Method::GET, "/api/v3/trades", params)
            .await
    }

    pub async fn historical_trades(
        &self,
        symbol: &str,
        limit: Option<u16>,
        from_id: Option<u64>,
    ) -> Result<Vec<Trade>, Error> {
        let params = optional_params([
            ("symbol", Some(symbol.to_owned())),
            ("limit", limit.map(|value| value.to_string())),
            ("fromId", from_id.map(|value| value.to_string())),
        ]);
        self.send_keyed_public(Method::GET, "/api/v3/historicalTrades", params)
            .await
    }

    pub async fn aggregate_trades(
        &self,
        symbol: &str,
        limit: Option<u16>,
        from_id: Option<u64>,
        start_time: Option<u64>,
        end_time: Option<u64>,
    ) -> Result<Vec<AggregateTrade>, Error> {
        self.aggregate_trades_with_request(
            &AggregateTradeRequest::new(symbol)
                .limit_opt(limit)
                .from_id_opt(from_id)
                .start_time_opt(start_time)
                .end_time_opt(end_time),
        )
        .await
    }

    pub async fn aggregate_trades_with_request(
        &self,
        request: &AggregateTradeRequest,
    ) -> Result<Vec<AggregateTrade>, Error> {
        self.send_public(Method::GET, "/api/v3/aggTrades", request.to_params())
            .await
    }

    pub async fn average_price(&self, symbol: &str) -> Result<AveragePrice, Error> {
        let params = optional_params([("symbol", Some(symbol.to_owned()))]);
        self.send_public(Method::GET, "/api/v3/avgPrice", params)
            .await
    }

    pub async fn ticker_24hr(&self, symbol: &str) -> Result<Ticker24hr, Error> {
        let params = optional_params([("symbol", Some(symbol.to_owned()))]);
        self.send_public(Method::GET, "/api/v3/ticker/24hr", params)
            .await
    }

    pub async fn ticker_24hr_all(&self) -> Result<Vec<Ticker24hr>, Error> {
        self.send_public(Method::GET, "/api/v3/ticker/24hr", Vec::new())
            .await
    }

    pub async fn order_book(&self, symbol: &str, limit: Option<u16>) -> Result<OrderBook, Error> {
        let params = optional_params([
            ("symbol", Some(symbol.to_owned())),
            ("limit", limit.map(|value| value.to_string())),
        ]);
        self.send_public(Method::GET, "/api/v3/depth", params).await
    }

    pub async fn klines(&self, request: &KlinesRequest) -> Result<Vec<Kline>, Error> {
        self.send_public(Method::GET, "/api/v3/klines", request.to_params())
            .await
    }

    pub async fn ui_klines(&self, request: &KlinesRequest) -> Result<Vec<Kline>, Error> {
        self.send_public(Method::GET, "/api/v3/uiKlines", request.to_params())
            .await
    }

    pub async fn symbol_info(&self, symbol: &str) -> Result<Option<serde_json::Value>, Error> {
        let exchange_info = self.exchange_info(None).await?;
        Ok(exchange_info
            .get("symbols")
            .and_then(|symbols| symbols.as_array())
            .and_then(|symbols| {
                symbols.iter().find(|entry| {
                    entry.get("symbol").and_then(|value| value.as_str()) == Some(symbol)
                })
            })
            .cloned())
    }

    pub async fn account(&self) -> Result<AccountInformation, Error> {
        self.send_signed(Method::GET, "/api/v3/account", Vec::new())
            .await
    }

    pub async fn asset_balance(&self, asset: &str) -> Result<Option<AccountBalance>, Error> {
        let account = self.account().await?;
        Ok(account
            .balances
            .into_iter()
            .find(|balance| balance.asset == asset))
    }

    pub async fn all_orders(
        &self,
        request: &AllOrdersRequest,
    ) -> Result<Vec<OrderResponse>, Error> {
        self.send_signed(Method::GET, "/api/v3/allOrders", request.to_params())
            .await
    }

    pub async fn my_trades(&self, request: &MyTradesRequest) -> Result<Vec<AccountTrade>, Error> {
        self.send_signed(Method::GET, "/api/v3/myTrades", request.to_params())
            .await
    }

    pub async fn current_order_count(&self) -> Result<Vec<OrderCountUsage>, Error> {
        self.send_signed(Method::GET, "/api/v3/rateLimit/order", Vec::new())
            .await
    }

    pub async fn open_order_lists(&self) -> Result<Vec<OrderListSummary>, Error> {
        self.send_signed(Method::GET, "/api/v3/openOrderList", Vec::new())
            .await
    }

    pub async fn order_list(
        &self,
        request: &OrderListQueryRequest,
    ) -> Result<OrderListSummary, Error> {
        self.send_signed(Method::GET, "/api/v3/orderList", request.to_params())
            .await
    }

    pub async fn all_order_lists(
        &self,
        from_id: Option<u64>,
        start_time: Option<u64>,
        end_time: Option<u64>,
        limit: Option<u16>,
    ) -> Result<Vec<OrderListSummary>, Error> {
        let params = optional_params([
            ("fromId", from_id.map(|value| value.to_string())),
            ("startTime", start_time.map(|value| value.to_string())),
            ("endTime", end_time.map(|value| value.to_string())),
            ("limit", limit.map(|value| value.to_string())),
        ]);
        self.send_signed(Method::GET, "/api/v3/allOrderList", params)
            .await
    }

    pub async fn cancel_order_list(
        &self,
        request: &CancelOrderListRequest,
    ) -> Result<OrderListSummary, Error> {
        self.send_signed(Method::DELETE, "/api/v3/orderList", request.to_params())
            .await
    }

    pub async fn cancel_open_orders(&self, symbol: &str) -> Result<serde_json::Value, Error> {
        let params = optional_params([("symbol", Some(symbol.to_owned()))]);
        self.send_signed(Method::DELETE, "/api/v3/openOrders", params)
            .await
    }

    pub async fn commission_rates(&self, symbol: &str) -> Result<CommissionRates, Error> {
        let params = optional_params([("symbol", Some(symbol.to_owned()))]);
        self.send_signed(Method::GET, "/api/v3/account/commission", params)
            .await
    }

    pub async fn prevented_matches(
        &self,
        request: &PreventedMatchesRequest,
    ) -> Result<Vec<PreventedMatch>, Error> {
        self.send_signed(
            Method::GET,
            "/api/v3/myPreventedMatches",
            request.to_params(),
        )
        .await
    }

    pub async fn order_amendments(
        &self,
        request: &OrderAmendmentsRequest,
    ) -> Result<Vec<OrderAmendment>, Error> {
        self.send_signed(Method::GET, "/api/v3/order/amendments", request.to_params())
            .await
    }

    pub async fn symbol_filters(&self, symbol: &str) -> Result<SymbolFilters, Error> {
        let params = optional_params([("symbol", Some(symbol.to_owned()))]);
        self.send_signed(Method::GET, "/api/v3/myFilters", params)
            .await
    }

    pub async fn create_oco_order(
        &self,
        request: &CreateOcoOrderRequest,
    ) -> Result<OrderListSummary, Error> {
        self.send_signed(Method::POST, "/api/v3/orderList/oco", request.to_params())
            .await
    }

    pub async fn cancel_replace_order(
        &self,
        cancel_order_id: u64,
        mode: CancelReplaceMode,
        new_order: &CreateOrderRequest,
    ) -> Result<CancelReplaceResponse, Error> {
        let mut params = new_order.to_params();
        params.push(("cancelReplaceMode".to_string(), mode.as_str().to_string()));
        params.push(("cancelOrderId".to_string(), cancel_order_id.to_string()));
        self.send_signed(Method::POST, "/api/v3/order/cancelReplace", params)
            .await
    }

    pub async fn open_orders(&self, symbol: Option<&str>) -> Result<Vec<OrderResponse>, Error> {
        let params = optional_params([("symbol", symbol.map(ToOwned::to_owned))]);
        self.send_signed(Method::GET, "/api/v3/openOrders", params)
            .await
    }

    pub async fn create_order(&self, request: &CreateOrderRequest) -> Result<OrderResponse, Error> {
        self.send_signed(Method::POST, "/api/v3/order", request.to_params())
            .await
    }

    pub async fn cancel_order(&self, request: &CancelOrderRequest) -> Result<OrderResponse, Error> {
        self.send_signed(Method::DELETE, "/api/v3/order", request.to_params())
            .await
    }

    pub async fn get_order(&self, request: &OrderQueryRequest) -> Result<OrderResponse, Error> {
        self.send_signed(Method::GET, "/api/v3/order", request.to_params())
            .await
    }

    pub fn debug_public_query<I, K>(&self, params: I) -> String
    where
        I: IntoIterator<Item = (K, Option<String>)>,
        K: Into<String>,
    {
        let pairs = params
            .into_iter()
            .filter_map(|(key, value)| value.map(|value| (key.into(), value)))
            .collect::<Vec<_>>();
        encode_params(&pairs)
    }

    pub fn debug_signed_query<T: ToParams>(&self, request: &T) -> Result<String, Error> {
        self.signed_query(request.to_params())
    }

    async fn send_public<T: DeserializeOwned>(
        &self,
        method: Method,
        path: &str,
        params: Vec<(String, String)>,
    ) -> Result<T, Error> {
        let query = encode_params(&params);
        let url = self.url(path, (!query.is_empty()).then_some(query.as_str()));
        let request = self.http.request(method, url);
        self.execute(request).await
    }

    async fn send_keyed_public<T: DeserializeOwned>(
        &self,
        method: Method,
        path: &str,
        params: Vec<(String, String)>,
    ) -> Result<T, Error> {
        let api_key = self.api_key.as_deref().ok_or(Error::MissingCredentials)?;
        let query = encode_params(&params);
        let url = self.url(path, (!query.is_empty()).then_some(query.as_str()));
        let request = self
            .http
            .request(method, url)
            .header(API_KEY_HEADER, api_key);
        self.execute(request).await
    }

    async fn send_signed<T: DeserializeOwned>(
        &self,
        method: Method,
        path: &str,
        mut params: Vec<(String, String)>,
    ) -> Result<T, Error> {
        let api_key = self.api_key.as_deref().ok_or(Error::MissingCredentials)?;
        let query = self.signed_query_with_pairs(&mut params)?;

        let request = match method {
            Method::POST => self
                .http
                .request(method, self.url(path, None))
                .header(API_KEY_HEADER, api_key)
                .header("Content-Type", "application/x-www-form-urlencoded")
                .body(query),
            _ => self
                .http
                .request(method, self.url(path, Some(query.as_str())))
                .header(API_KEY_HEADER, api_key),
        };

        self.execute(request).await
    }

    async fn execute<T: DeserializeOwned>(
        &self,
        request: reqwest::RequestBuilder,
    ) -> Result<T, Error> {
        let response = request.send().await?;
        let status = response.status();
        let body = response.text().await?;

        if !status.is_success() {
            let api_error = serde_json::from_str::<BinanceApiError>(&body).unwrap_or_else(|_| {
                BinanceApiError {
                    code: status.as_u16() as i64,
                    message: body.clone(),
                }
            });
            return Err(Error::Api(api_error));
        }

        Ok(serde_json::from_str(&body)?)
    }

    fn signed_query(&self, params: Vec<(String, String)>) -> Result<String, Error> {
        let mut params = params;
        self.signed_query_with_pairs(&mut params)
    }

    fn signed_query_with_pairs(&self, params: &mut Vec<(String, String)>) -> Result<String, Error> {
        let api_secret = self
            .api_secret
            .as_deref()
            .ok_or(Error::MissingCredentials)?;
        params.push(("recvWindow".to_string(), self.recv_window.to_string()));
        params.push((
            "timestamp".to_string(),
            (self.timestamp_provider)().to_string(),
        ));

        let query = encode_params(params);
        let signature = sign(api_secret, &query)?;

        Ok(format!("{query}&signature={signature}"))
    }

    fn url(&self, path: &str, query: Option<&str>) -> String {
        match query {
            Some(query) if !query.is_empty() => format!("{}{}?{}", self.rest_base_url, path, query),
            _ => format!("{}{}", self.rest_base_url, path),
        }
    }
}

impl BinanceClientBuilder {
    pub fn api_key(mut self, value: impl Into<String>) -> Self {
        self.api_key = Some(value.into());
        self
    }

    pub fn api_secret(mut self, value: impl Into<String>) -> Self {
        self.api_secret = Some(value.into());
        self
    }

    pub fn rest_base_url(mut self, value: impl Into<String>) -> Self {
        self.rest_base_url = Some(value.into());
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

    pub fn build(self) -> Result<BinanceClient, Error> {
        let rest_base_url = self
            .rest_base_url
            .unwrap_or_else(|| DEFAULT_BASE_URL.to_string())
            .trim_end_matches('/')
            .to_string();

        Ok(BinanceClient {
            http: reqwest::Client::builder()
                .user_agent(concat!(
                    env!("CARGO_PKG_NAME"),
                    "/",
                    env!("CARGO_PKG_VERSION")
                ))
                .build()?,
            api_key: self.api_key,
            api_secret: self.api_secret,
            rest_base_url,
            recv_window: self.recv_window.unwrap_or(DEFAULT_RECV_WINDOW),
            timestamp_provider: self
                .timestamp_provider
                .unwrap_or_else(default_timestamp_provider),
        })
    }
}

fn default_timestamp_provider() -> TimestampProvider {
    Arc::new(|| {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_millis() as u64
    })
}

fn optional_params<const N: usize>(params: [(&str, Option<String>); N]) -> Vec<(String, String)> {
    params
        .into_iter()
        .filter_map(|(key, value)| value.map(|value| (key.to_string(), value)))
        .collect()
}

fn encode_params(params: &[(String, String)]) -> String {
    serde_urlencoded::to_string(params).expect("query params should serialize")
}

fn sign(secret: &str, payload: &str) -> Result<String, Error> {
    let mut signer =
        HmacSha256::new_from_slice(secret.as_bytes()).map_err(|_| Error::InvalidSecretKey)?;
    signer.update(payload.as_bytes());
    Ok(hex::encode(signer.finalize().into_bytes()))
}
