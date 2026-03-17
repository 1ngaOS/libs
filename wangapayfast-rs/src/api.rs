use std::collections::BTreeMap;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use time::{format_description::well_known::Iso8601, OffsetDateTime};
use url::form_urlencoded;

use crate::error::{Error, Result};

/// PayFast API environment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApiEnvironment {
    /// Production API.
    Live,
    /// Sandbox mode (adds `testing=true` query parameter).
    Sandbox,
}

impl ApiEnvironment {
    fn base_url(self) -> &'static str {
        // Matches the official PayFast PHP SDK default.
        "https://api.payfast.co.za"
    }
}

/// Configuration for calling PayFast server-to-server APIs.
#[derive(Debug, Clone)]
pub struct PayFastApiConfig {
    /// Merchant ID as configured in PayFast.
    pub merchant_id: String,
    /// Salt passphrase ("passphrase") used to sign API requests.
    pub passphrase: Option<String>,
    /// API version header (default: `v1`).
    pub version: String,
    /// Live vs sandbox behaviour.
    pub environment: ApiEnvironment,
    /// Optional timeout (default: 30s).
    pub timeout: Duration,
}

impl PayFastApiConfig {
    /// Create a new API config.
    pub fn new(merchant_id: impl Into<String>) -> Self {
        Self {
            merchant_id: merchant_id.into(),
            passphrase: None,
            version: "v1".to_string(),
            environment: ApiEnvironment::Sandbox,
            timeout: Duration::from_secs(30),
        }
    }

    /// Set passphrase used for request signatures.
    pub fn with_passphrase(mut self, passphrase: impl Into<String>) -> Self {
        self.passphrase = Some(passphrase.into());
        self
    }

    /// Set API version header value (default: `v1`).
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }

    /// Set live/sandbox environment.
    pub fn with_environment(mut self, environment: ApiEnvironment) -> Self {
        self.environment = environment;
        self
    }

    /// Set request timeout.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
}

/// A small async API client for PayFast.
#[derive(Clone)]
pub struct PayFastApiClient {
    cfg: PayFastApiConfig,
    http: reqwest::Client,
}

impl PayFastApiClient {
    /// Create a new API client.
    pub fn new(cfg: PayFastApiConfig) -> Result<Self> {
        let http = reqwest::Client::builder().timeout(cfg.timeout).build()?;

        Ok(Self { cfg, http })
    }

    /// Fetch a subscription.
    pub async fn subscription_fetch(&self, token: &str) -> Result<serde_json::Value> {
        self.request_json(
            reqwest::Method::GET,
            &format!("/subscriptions/{token}/fetch"),
            BTreeMap::new(),
            None,
        )
        .await
    }

    /// Pause a subscription.
    pub async fn subscription_pause(
        &self,
        token: &str,
        cycles: Option<u32>,
    ) -> Result<serde_json::Value> {
        let body = cycles.map(|c| serde_json::json!({ "cycles": c }));
        self.request_json(
            reqwest::Method::PUT,
            &format!("/subscriptions/{token}/pause"),
            BTreeMap::new(),
            body,
        )
        .await
    }

    /// Unpause a subscription.
    pub async fn subscription_unpause(&self, token: &str) -> Result<serde_json::Value> {
        self.request_json(
            reqwest::Method::PUT,
            &format!("/subscriptions/{token}/unpause"),
            BTreeMap::new(),
            None,
        )
        .await
    }

    /// Cancel a subscription.
    pub async fn subscription_cancel(&self, token: &str) -> Result<serde_json::Value> {
        self.request_json(
            reqwest::Method::PUT,
            &format!("/subscriptions/{token}/cancel"),
            BTreeMap::new(),
            None,
        )
        .await
    }

    /// Update a subscription.
    pub async fn subscription_update(
        &self,
        token: &str,
        req: SubscriptionUpdate,
    ) -> Result<serde_json::Value> {
        self.request_json(
            reqwest::Method::PATCH,
            &format!("/subscriptions/{token}/update"),
            BTreeMap::new(),
            Some(serde_json::to_value(req).map_err(Error::Json)?),
        )
        .await
    }

    /// Charge a tokenization (ad-hoc) agreement.
    pub async fn subscription_adhoc(
        &self,
        token: &str,
        req: AdhocCharge,
    ) -> Result<serde_json::Value> {
        self.request_json(
            reqwest::Method::POST,
            &format!("/subscriptions/{token}/adhoc"),
            BTreeMap::new(),
            Some(serde_json::to_value(req).map_err(Error::Json)?),
        )
        .await
    }

    /// Transaction history over a date range.
    pub async fn transaction_history_range(
        &self,
        q: TransactionHistoryRange,
    ) -> Result<serde_json::Value> {
        let mut query = BTreeMap::new();
        query.insert("from".to_string(), q.from);
        if let Some(to) = q.to {
            query.insert("to".to_string(), to);
        }
        if let Some(offset) = q.offset {
            query.insert("offset".to_string(), offset.to_string());
        }
        if let Some(limit) = q.limit {
            query.insert("limit".to_string(), limit.to_string());
        }
        self.request_json(reqwest::Method::GET, "/transactions/history", query, None)
            .await
    }

    /// Daily transaction history.
    pub async fn transaction_history_daily(
        &self,
        q: TransactionHistoryByDate,
    ) -> Result<serde_json::Value> {
        self.request_json(
            reqwest::Method::GET,
            "/transactions/history/daily",
            q.into_query(),
            None,
        )
        .await
    }

    /// Weekly transaction history.
    pub async fn transaction_history_weekly(
        &self,
        q: TransactionHistoryByDate,
    ) -> Result<serde_json::Value> {
        self.request_json(
            reqwest::Method::GET,
            "/transactions/history/weekly",
            q.into_query(),
            None,
        )
        .await
    }

    /// Monthly transaction history (`date` is `YYYY-MM`).
    pub async fn transaction_history_monthly(
        &self,
        q: TransactionHistoryByMonth,
    ) -> Result<serde_json::Value> {
        self.request_json(
            reqwest::Method::GET,
            "/transactions/history/monthly",
            q.into_query(),
            None,
        )
        .await
    }

    /// Query a credit card transaction.
    pub async fn credit_card_transaction_fetch(&self, token: &str) -> Result<serde_json::Value> {
        self.request_json(
            reqwest::Method::GET,
            &format!("/process/query/{token}"),
            BTreeMap::new(),
            None,
        )
        .await
    }

    /// Fetch a refund.
    pub async fn refund_fetch(&self, id: &str) -> Result<serde_json::Value> {
        self.request_json(
            reqwest::Method::GET,
            &format!("/refunds/{id}"),
            BTreeMap::new(),
            None,
        )
        .await
    }

    /// Create a refund.
    pub async fn refund_create(&self, id: &str, req: RefundCreate) -> Result<serde_json::Value> {
        self.request_json(
            reqwest::Method::POST,
            &format!("/refunds/{id}"),
            BTreeMap::new(),
            Some(serde_json::to_value(req).map_err(Error::Json)?),
        )
        .await
    }

    async fn request_json(
        &self,
        method: reqwest::Method,
        path: &str,
        mut query: BTreeMap<String, String>,
        body: Option<serde_json::Value>,
    ) -> Result<serde_json::Value> {
        if self.cfg.environment == ApiEnvironment::Sandbox {
            query.insert("testing".to_string(), "true".to_string());
        }

        let timestamp = OffsetDateTime::now_utc()
            .format(&Iso8601::DEFAULT)
            .map_err(|e| Error::Other(format!("timestamp format: {e}")))?;

        let mut headers_for_sig = BTreeMap::new();
        headers_for_sig.insert("merchant-id".to_string(), self.cfg.merchant_id.clone());
        headers_for_sig.insert("version".to_string(), self.cfg.version.clone());
        headers_for_sig.insert("timestamp".to_string(), timestamp.clone());

        let mut sig_data: BTreeMap<String, String> = BTreeMap::new();
        sig_data.extend(headers_for_sig.clone());
        sig_data.extend(query.clone());
        if let Some(b) = &body {
            if let Some(obj) = b.as_object() {
                for (k, v) in obj {
                    if let Some(s) = v.as_str() {
                        sig_data.insert(k.clone(), s.to_string());
                    } else {
                        sig_data.insert(k.clone(), v.to_string());
                    }
                }
            }
        }

        let signature = generate_api_signature(&sig_data, self.cfg.passphrase.as_deref());

        let url = format!("{}{}", self.cfg.environment.base_url(), path);
        let mut req = self.http.request(method, url);
        req = req.header("merchant-id", &self.cfg.merchant_id);
        req = req.header("version", &self.cfg.version);
        req = req.header("timestamp", &timestamp);
        req = req.header("signature", signature);

        if !query.is_empty() {
            req = req.query(&query);
        }
        if let Some(b) = body {
            req = req.json(&b);
        }

        let resp = req.send().await?;
        let status = resp.status();
        let text = resp.text().await?;
        if !status.is_success() {
            return Err(Error::ApiHttp {
                status: status.as_u16(),
                body: text,
            });
        }

        Ok(serde_json::from_str(&text)?)
    }
}

/// Subscription update payload.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SubscriptionUpdate {
    /// Number of cycles/payments. `0` may mean indefinite depending on PayFast rules.
    pub cycles: Option<u32>,
    /// Frequency code as per PayFast (e.g. 1 daily, 3 monthly, etc.).
    pub frequency: Option<u32>,
    /// Date (YYYY-MM-DD).
    pub run_date: Option<String>,
    /// Amount in cents (ZAR) as per PayFast API docs / SDKs.
    pub amount: Option<u32>,
}

/// Ad-hoc charge payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdhocCharge {
    /// Amount in cents.
    pub amount: u32,
    /// Item / order name.
    pub item_name: String,
    /// Optional card CVV for the charge.
    pub cc_cvv: Option<u16>,
}

/// Query payload for transaction history over a date range.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionHistoryRange {
    /// Date (YYYY-MM-DD).
    pub from: String,
    /// Date (YYYY-MM-DD).
    pub to: Option<String>,
    /// Pagination offset.
    pub offset: Option<u32>,
    /// Pagination limit.
    pub limit: Option<u32>,
}

/// Query payload for daily/weekly transaction history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionHistoryByDate {
    /// Date (YYYY-MM-DD).
    pub date: String,
    /// Pagination offset.
    pub offset: Option<u32>,
    /// Pagination limit.
    pub limit: Option<u32>,
}

impl TransactionHistoryByDate {
    fn into_query(self) -> BTreeMap<String, String> {
        let mut q = BTreeMap::new();
        q.insert("date".to_string(), self.date);
        if let Some(o) = self.offset {
            q.insert("offset".to_string(), o.to_string());
        }
        if let Some(l) = self.limit {
            q.insert("limit".to_string(), l.to_string());
        }
        q
    }
}

/// Query payload for monthly transaction history (`date` is `YYYY-MM`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionHistoryByMonth {
    /// Month (YYYY-MM).
    pub date: String,
    /// Pagination offset.
    pub offset: Option<u32>,
    /// Pagination limit.
    pub limit: Option<u32>,
}

impl TransactionHistoryByMonth {
    fn into_query(self) -> BTreeMap<String, String> {
        let mut q = BTreeMap::new();
        q.insert("date".to_string(), self.date);
        if let Some(o) = self.offset {
            q.insert("offset".to_string(), o.to_string());
        }
        if let Some(l) = self.limit {
            q.insert("limit".to_string(), l.to_string());
        }
        q
    }
}

/// Refund creation payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefundCreate {
    /// Optional amount in cents to refund.
    pub amount: Option<u32>,
    /// Optional refund reason.
    pub reason: Option<String>,
    /// Optional account type (e.g. `"savings"`), if required by PayFast.
    pub acc_type: Option<String>,
    /// Defaults to 1 in the official SDKs.
    pub notify_buyer: Option<u8>,
}

/// Generate an API signature according to PayFast's API signing scheme.
///
/// This mirrors the official PayFast PHP SDK behaviour:
/// - add `passphrase` if provided
/// - sort keys alphabetically
/// - URL encode values (form encoding)
/// - join `k=v&...` excluding `signature`
/// - MD5 the resulting string
pub fn generate_api_signature(data: &BTreeMap<String, String>, passphrase: Option<&str>) -> String {
    let mut m: BTreeMap<String, String> = data.clone();
    if let Some(p) = passphrase {
        m.insert("passphrase".to_string(), p.to_string());
    }

    let mut serializer = form_urlencoded::Serializer::new(String::new());
    for (k, v) in m.iter() {
        if k == "signature" {
            continue;
        }
        serializer.append_pair(k, v);
    }

    let canonical = serializer.finish();
    let digest = md5::compute(canonical.as_bytes());
    format!("{:x}", digest)
}
