use crate::error::{Error, Result};
use serde::Deserialize;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Token response from Microsoft identity platform.
#[derive(Debug, Deserialize)]
pub(super) struct TokenResponse {
    pub access_token: String,
    #[allow(dead_code)]
    pub token_type: String,
    pub expires_in: u64,
}

/// Simple in-memory token cache: stores token and expiry.
/// We refresh when less than 60 seconds remain.
struct TokenCache {
    token: String,
    expires_at: Instant,
}

const REFRESH_BUFFER_SECS: u64 = 60;

pub(super) struct TokenProvider {
    #[allow(dead_code)]
    tenant_id: String,
    client_id: String,
    client_secret: String,
    token_url: String,
    scope: String,
    http_client: reqwest::Client,
    cached: Mutex<Option<TokenCache>>,
}

impl TokenProvider {
    pub fn new(
        tenant_id: String,
        client_id: String,
        client_secret: String,
        token_url: String,
        scope: String,
        http_client: reqwest::Client,
    ) -> Self {
        Self {
            tenant_id,
            client_id,
            client_secret,
            token_url,
            scope,
            http_client,
            cached: Mutex::new(None),
        }
    }

    pub async fn get_token(&self) -> Result<String> {
        {
            let guard = self
                .cached
                .lock()
                .map_err(|_| Error::Config("token cache mutex poisoned".into()))?;
            if let Some(cache) = guard.as_ref() {
                if cache
                    .expires_at
                    .saturating_duration_since(Instant::now())
                    .as_secs()
                    > REFRESH_BUFFER_SECS
                {
                    return Ok(cache.token.clone());
                }
            }
        }

        let form = [
            ("grant_type", "client_credentials"),
            ("client_id", self.client_id.as_str()),
            ("client_secret", self.client_secret.as_str()),
            ("scope", self.scope.as_str()),
        ];
        let res = self
            .http_client
            .post(&self.token_url)
            .form(&form)
            .send()
            .await?;

        let status = res.status();
        let body = res.text().await?;

        if !status.is_success() {
            let err_msg: Option<AuthErrorResponse> = serde_json::from_str(&body).ok();
            return Err(Error::Auth(
                err_msg
                    .map(|e| format!("{}: {}", e.error, e.error_description.unwrap_or_default()))
                    .unwrap_or_else(|| format!("HTTP {}: {}", status, body)),
            ));
        }

        let token_res: TokenResponse = serde_json::from_str(&body)
            .map_err(|e| Error::Auth(format!("Invalid token response: {e}")))?;

        let expires_at = Instant::now()
            + Duration::from_secs(token_res.expires_in.saturating_sub(REFRESH_BUFFER_SECS));
        let token = token_res.access_token;
        {
            let mut guard = self
                .cached
                .lock()
                .map_err(|_| Error::Config("token cache mutex poisoned".into()))?;
            *guard = Some(TokenCache {
                token: token.clone(),
                expires_at,
            });
        }
        Ok(token)
    }
}

#[derive(Debug, Deserialize)]
struct AuthErrorResponse {
    error: String,
    #[serde(rename = "error_description")]
    error_description: Option<String>,
}
