use crate::errors::{AppError, AppResult};
use serde::de::DeserializeOwned;
use std::time::Duration;
use tokio::sync::Mutex;

pub const BASE_URL: &str = "https://api.the-odds-api.com/v4";

#[derive(Debug, Clone, Default)]
pub struct CreditInfo {
    pub remaining: Option<u32>,
    pub used: Option<u32>,
}

pub struct OddsApiClient {
    http: reqwest::Client,
    base_url: String,
    api_key: String,
    credits: Mutex<CreditInfo>,
}

impl OddsApiClient {
    pub fn new(api_key: impl Into<String>) -> AppResult<Self> {
        Self::with_base_url(api_key, BASE_URL)
    }

    pub fn with_base_url(
        api_key: impl Into<String>,
        base_url: impl Into<String>,
    ) -> AppResult<Self> {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .build()?;
        Ok(Self {
            http,
            base_url: base_url.into(),
            api_key: api_key.into(),
            credits: Mutex::new(CreditInfo::default()),
        })
    }

    pub async fn last_credits(&self) -> CreditInfo {
        self.credits.lock().await.clone()
    }

    pub async fn get<T: DeserializeOwned>(
        &self,
        path: &str,
        params: &[(&str, String)],
    ) -> AppResult<T> {
        let url = format!("{}{}", self.base_url, path);
        let mut query: Vec<(&str, String)> = params.to_vec();
        query.push(("apiKey", self.api_key.clone()));

        let response = self.http.get(&url).query(&query).send().await?;
        let status = response.status();

        let remaining = parse_header_u32(&response, "x-requests-remaining");
        let used = parse_header_u32(&response, "x-requests-used");
        {
            let mut guard = self.credits.lock().await;
            *guard = CreditInfo { remaining, used };
        }

        if !status.is_success() {
            let msg = response.text().await.unwrap_or_default();
            return Err(AppError::Api {
                status: status.as_u16(),
                msg,
            });
        }

        let parsed = response.json::<T>().await?;
        Ok(parsed)
    }
}

fn parse_header_u32(response: &reqwest::Response, name: &str) -> Option<u32> {
    response
        .headers()
        .get(name)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<u32>().ok())
}
