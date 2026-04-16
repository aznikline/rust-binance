use thiserror::Error;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct BinanceApiError {
    pub code: i64,
    #[serde(alias = "msg")]
    pub message: String,
}

impl std::fmt::Display for BinanceApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "binance api error {}: {}", self.code, self.message)
    }
}

impl std::error::Error for BinanceApiError {}

#[derive(Debug, Error)]
pub enum Error {
    #[error("binance credentials are required for signed endpoints")]
    MissingCredentials,
    #[error("invalid api secret key")]
    InvalidSecretKey,
    #[error(transparent)]
    Http(#[from] reqwest::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Api(BinanceApiError),
}
