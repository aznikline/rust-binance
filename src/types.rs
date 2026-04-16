use std::collections::HashMap;

use serde::{Deserialize, Serialize};

pub trait ToParams {
    fn to_params(&self) -> Vec<(String, String)>;
}

pub type ExchangeInfo = serde_json::Value;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerTimeResponse {
    pub server_time: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PriceTicker {
    pub symbol: String,
    pub price: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BookTicker {
    pub symbol: String,
    pub bid_price: String,
    pub bid_qty: String,
    pub ask_price: String,
    pub ask_qty: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Trade {
    pub id: u64,
    pub price: String,
    pub qty: String,
    pub quote_qty: String,
    pub time: u64,
    pub is_buyer_maker: bool,
    pub is_best_match: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct AggregateTrade {
    #[serde(rename = "a")]
    pub aggregate_trade_id: u64,
    #[serde(rename = "p")]
    pub price: String,
    #[serde(rename = "q")]
    pub quantity: String,
    #[serde(rename = "f")]
    pub first_trade_id: u64,
    #[serde(rename = "l")]
    pub last_trade_id: u64,
    #[serde(rename = "T")]
    pub timestamp: u64,
    #[serde(rename = "m")]
    pub is_buyer_maker: bool,
    #[serde(rename = "M")]
    pub is_best_match: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AveragePrice {
    pub mins: u64,
    pub price: String,
    pub close_time: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ticker24hr {
    pub symbol: String,
    pub price_change: String,
    pub price_change_percent: String,
    pub weighted_avg_price: String,
    pub prev_close_price: String,
    pub last_price: String,
    pub last_qty: String,
    pub bid_price: String,
    pub bid_qty: String,
    pub ask_price: String,
    pub ask_qty: String,
    pub open_price: String,
    pub high_price: String,
    pub low_price: String,
    pub volume: String,
    pub quote_volume: String,
    pub open_time: u64,
    pub close_time: u64,
    pub first_id: u64,
    pub last_id: u64,
    pub count: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderBook {
    pub last_update_id: u64,
    pub bids: Vec<[String; 2]>,
    pub asks: Vec<[String; 2]>,
}

pub type Kline = Vec<serde_json::Value>;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountInformation {
    pub maker_commission: u64,
    pub taker_commission: u64,
    pub buyer_commission: u64,
    pub seller_commission: u64,
    pub can_trade: bool,
    pub can_withdraw: bool,
    pub can_deposit: bool,
    pub update_time: u64,
    pub account_type: String,
    pub balances: Vec<AccountBalance>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AccountBalance {
    pub asset: String,
    pub free: String,
    pub locked: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderResponse {
    pub symbol: String,
    #[serde(default)]
    pub order_id: u64,
    #[serde(default)]
    pub order_list_id: i64,
    #[serde(default)]
    pub client_order_id: String,
    #[serde(default)]
    pub transact_time: Option<u64>,
    #[serde(default)]
    pub price: String,
    #[serde(default)]
    pub orig_qty: String,
    #[serde(default)]
    pub executed_qty: String,
    #[serde(default)]
    pub cummulative_quote_qty: String,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub time_in_force: Option<String>,
    #[serde(default, rename = "type")]
    pub order_type: Option<String>,
    #[serde(default)]
    pub side: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderSide {
    Buy,
    Sell,
}

impl OrderSide {
    fn as_str(self) -> &'static str {
        match self {
            Self::Buy => "BUY",
            Self::Sell => "SELL",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderType {
    Limit,
    Market,
}

impl OrderType {
    fn as_str(self) -> &'static str {
        match self {
            Self::Limit => "LIMIT",
            Self::Market => "MARKET",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeInForce {
    Gtc,
    Ioc,
    Fok,
}

impl TimeInForce {
    fn as_str(self) -> &'static str {
        match self {
            Self::Gtc => "GTC",
            Self::Ioc => "IOC",
            Self::Fok => "FOK",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KlineInterval {
    OneMinute,
    ThreeMinutes,
    FiveMinutes,
    FifteenMinutes,
    ThirtyMinutes,
    OneHour,
    TwoHours,
    FourHours,
    SixHours,
    EightHours,
    TwelveHours,
    OneDay,
    ThreeDays,
    OneWeek,
    OneMonth,
}

impl KlineInterval {
    fn as_str(self) -> &'static str {
        match self {
            Self::OneMinute => "1m",
            Self::ThreeMinutes => "3m",
            Self::FiveMinutes => "5m",
            Self::FifteenMinutes => "15m",
            Self::ThirtyMinutes => "30m",
            Self::OneHour => "1h",
            Self::TwoHours => "2h",
            Self::FourHours => "4h",
            Self::SixHours => "6h",
            Self::EightHours => "8h",
            Self::TwelveHours => "12h",
            Self::OneDay => "1d",
            Self::ThreeDays => "3d",
            Self::OneWeek => "1w",
            Self::OneMonth => "1M",
        }
    }
}

impl std::fmt::Display for KlineInterval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone)]
pub struct KlinesRequest {
    pub symbol: String,
    pub interval: KlineInterval,
    pub start_time: Option<u64>,
    pub end_time: Option<u64>,
    pub time_zone: Option<String>,
    pub limit: Option<u16>,
}

#[derive(Debug, Clone, Default)]
pub struct AggregateTradeRequest {
    pub symbol: String,
    pub from_id: Option<u64>,
    pub start_time: Option<u64>,
    pub end_time: Option<u64>,
    pub limit: Option<u16>,
}

impl AggregateTradeRequest {
    pub fn new(symbol: impl Into<String>) -> Self {
        Self {
            symbol: symbol.into(),
            from_id: None,
            start_time: None,
            end_time: None,
            limit: None,
        }
    }

    pub fn from_id(mut self, value: u64) -> Self {
        self.from_id = Some(value);
        self
    }

    pub fn from_id_opt(mut self, value: Option<u64>) -> Self {
        self.from_id = value;
        self
    }

    pub fn start_time(mut self, value: u64) -> Self {
        self.start_time = Some(value);
        self
    }

    pub fn start_time_opt(mut self, value: Option<u64>) -> Self {
        self.start_time = value;
        self
    }

    pub fn end_time(mut self, value: u64) -> Self {
        self.end_time = Some(value);
        self
    }

    pub fn end_time_opt(mut self, value: Option<u64>) -> Self {
        self.end_time = value;
        self
    }

    pub fn limit(mut self, value: u16) -> Self {
        self.limit = Some(value);
        self
    }

    pub fn limit_opt(mut self, value: Option<u16>) -> Self {
        self.limit = value;
        self
    }
}

impl ToParams for AggregateTradeRequest {
    fn to_params(&self) -> Vec<(String, String)> {
        let mut params = vec![("symbol".to_string(), self.symbol.clone())];
        if let Some(value) = self.from_id {
            params.push(("fromId".to_string(), value.to_string()));
        }
        if let Some(value) = self.start_time {
            params.push(("startTime".to_string(), value.to_string()));
        }
        if let Some(value) = self.end_time {
            params.push(("endTime".to_string(), value.to_string()));
        }
        if let Some(value) = self.limit {
            params.push(("limit".to_string(), value.to_string()));
        }
        params
    }
}

impl KlinesRequest {
    pub fn new(symbol: impl Into<String>, interval: KlineInterval) -> Self {
        Self {
            symbol: symbol.into(),
            interval,
            start_time: None,
            end_time: None,
            time_zone: None,
            limit: None,
        }
    }

    pub fn start_time(mut self, value: u64) -> Self {
        self.start_time = Some(value);
        self
    }

    pub fn end_time(mut self, value: u64) -> Self {
        self.end_time = Some(value);
        self
    }

    pub fn time_zone(mut self, value: impl Into<String>) -> Self {
        self.time_zone = Some(value.into());
        self
    }

    pub fn limit(mut self, value: u16) -> Self {
        self.limit = Some(value);
        self
    }
}

impl ToParams for KlinesRequest {
    fn to_params(&self) -> Vec<(String, String)> {
        let mut params = vec![
            ("symbol".to_string(), self.symbol.clone()),
            ("interval".to_string(), self.interval.to_string()),
        ];
        if let Some(value) = self.start_time {
            params.push(("startTime".to_string(), value.to_string()));
        }
        if let Some(value) = self.end_time {
            params.push(("endTime".to_string(), value.to_string()));
        }
        if let Some(value) = &self.time_zone {
            params.push(("timeZone".to_string(), value.clone()));
        }
        if let Some(value) = self.limit {
            params.push(("limit".to_string(), value.to_string()));
        }
        params
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateOrderRequest {
    pub symbol: String,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub time_in_force: Option<TimeInForce>,
    pub quantity: Option<String>,
    pub quote_order_qty: Option<String>,
    pub price: Option<String>,
    pub new_client_order_id: Option<String>,
    pub new_order_resp_type: Option<String>,
}

impl CreateOrderRequest {
    pub fn limit(
        symbol: impl Into<String>,
        side: OrderSide,
        quantity: impl Into<String>,
        price: impl Into<String>,
        time_in_force: TimeInForce,
    ) -> Self {
        Self {
            symbol: symbol.into(),
            side,
            order_type: OrderType::Limit,
            time_in_force: Some(time_in_force),
            quantity: Some(quantity.into()),
            quote_order_qty: None,
            price: Some(price.into()),
            new_client_order_id: None,
            new_order_resp_type: None,
        }
    }

    pub fn market(symbol: impl Into<String>, side: OrderSide, quantity: impl Into<String>) -> Self {
        Self {
            symbol: symbol.into(),
            side,
            order_type: OrderType::Market,
            time_in_force: None,
            quantity: Some(quantity.into()),
            quote_order_qty: None,
            price: None,
            new_client_order_id: None,
            new_order_resp_type: None,
        }
    }
}

impl ToParams for CreateOrderRequest {
    fn to_params(&self) -> Vec<(String, String)> {
        let mut params = vec![
            ("symbol".to_string(), self.symbol.clone()),
            ("side".to_string(), self.side.as_str().to_string()),
            ("type".to_string(), self.order_type.as_str().to_string()),
        ];
        if let Some(value) = self.time_in_force {
            params.push(("timeInForce".to_string(), value.as_str().to_string()));
        }
        if let Some(value) = &self.quantity {
            params.push(("quantity".to_string(), value.clone()));
        }
        if let Some(value) = &self.quote_order_qty {
            params.push(("quoteOrderQty".to_string(), value.clone()));
        }
        if let Some(value) = &self.price {
            params.push(("price".to_string(), value.clone()));
        }
        if let Some(value) = &self.new_client_order_id {
            params.push(("newClientOrderId".to_string(), value.clone()));
        }
        if let Some(value) = &self.new_order_resp_type {
            params.push(("newOrderRespType".to_string(), value.clone()));
        }
        params
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CancelOrderRequest {
    pub symbol: String,
    pub order_id: Option<u64>,
    pub orig_client_order_id: Option<String>,
    pub new_client_order_id: Option<String>,
}

impl CancelOrderRequest {
    pub fn by_order_id(symbol: impl Into<String>, order_id: u64) -> Self {
        Self {
            symbol: symbol.into(),
            order_id: Some(order_id),
            orig_client_order_id: None,
            new_client_order_id: None,
        }
    }

    pub fn by_client_order_id(
        symbol: impl Into<String>,
        orig_client_order_id: impl Into<String>,
    ) -> Self {
        Self {
            symbol: symbol.into(),
            order_id: None,
            orig_client_order_id: Some(orig_client_order_id.into()),
            new_client_order_id: None,
        }
    }
}

impl ToParams for CancelOrderRequest {
    fn to_params(&self) -> Vec<(String, String)> {
        let mut params = vec![("symbol".to_string(), self.symbol.clone())];
        if let Some(value) = self.order_id {
            params.push(("orderId".to_string(), value.to_string()));
        }
        if let Some(value) = &self.orig_client_order_id {
            params.push(("origClientOrderId".to_string(), value.clone()));
        }
        if let Some(value) = &self.new_client_order_id {
            params.push(("newClientOrderId".to_string(), value.clone()));
        }
        params
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderQueryRequest {
    pub symbol: String,
    pub order_id: Option<u64>,
    pub orig_client_order_id: Option<String>,
}

impl OrderQueryRequest {
    pub fn by_order_id(symbol: impl Into<String>, order_id: u64) -> Self {
        Self {
            symbol: symbol.into(),
            order_id: Some(order_id),
            orig_client_order_id: None,
        }
    }
}

impl ToParams for OrderQueryRequest {
    fn to_params(&self) -> Vec<(String, String)> {
        let mut params = vec![("symbol".to_string(), self.symbol.clone())];
        if let Some(value) = self.order_id {
            params.push(("orderId".to_string(), value.to_string()));
        }
        if let Some(value) = &self.orig_client_order_id {
            params.push(("origClientOrderId".to_string(), value.clone()));
        }
        params
    }
}
