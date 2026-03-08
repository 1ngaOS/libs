use crate::auth::TokenProvider;
use crate::error::{Error, Result};
use crate::graph::SendMailRequest;
use reqwest::Client;
use std::sync::Arc;

/// Default Microsoft Graph scope for client credentials.
const DEFAULT_SCOPE: &str = "https://graph.microsoft.com/.default";

/// Base URL for global Microsoft Graph.
const DEFAULT_GRAPH_BASE: &str = "https://graph.microsoft.com/v1.0";

/// Builder for [`GraphMailClient`].
///
/// Required: [`tenant_id`](GraphMailClientBuilder::tenant_id), [`client_id`](GraphMailClientBuilder::client_id),
/// [`client_secret`](GraphMailClientBuilder::client_secret). Optional overrides for sovereign clouds:
/// [`token_url`](GraphMailClientBuilder::token_url), [`graph_base`](GraphMailClientBuilder::graph_base), [`scope`](GraphMailClientBuilder::scope).
#[derive(Debug, Clone, Default)]
pub struct GraphMailClientBuilder {
    tenant_id: Option<String>,
    client_id: Option<String>,
    client_secret: Option<String>,
    token_url: Option<String>,
    graph_base: Option<String>,
    scope: Option<String>,
}

impl GraphMailClientBuilder {
    /// Create a new builder with no configuration set.
    pub fn new() -> Self {
        Self::default()
    }

    /// Azure AD tenant ID (GUID or domain).
    pub fn tenant_id(mut self, id: impl Into<String>) -> Self {
        self.tenant_id = Some(id.into());
        self
    }

    /// Application (client) ID from app registration.
    pub fn client_id(mut self, id: impl Into<String>) -> Self {
        self.client_id = Some(id.into());
        self
    }

    /// Client secret from app registration.
    pub fn client_secret(mut self, secret: impl Into<String>) -> Self {
        self.client_secret = Some(secret.into());
        self
    }

    /// Override token endpoint (e.g. for sovereign clouds). Default: `https://login.microsoftonline.com/{tenant}/oauth2/v2.0/token`
    pub fn token_url(mut self, url: impl Into<String>) -> Self {
        self.token_url = Some(url.into());
        self
    }

    /// Override Graph API base URL (e.g. for sovereign clouds). Default: `https://graph.microsoft.com/v1.0`
    pub fn graph_base(mut self, base: impl Into<String>) -> Self {
        self.graph_base = Some(base.into());
        self
    }

    /// Override scope. Default: `https://graph.microsoft.com/.default`
    pub fn scope(mut self, scope: impl Into<String>) -> Self {
        self.scope = Some(scope.into());
        self
    }

    /// Build the [`GraphMailClient`]. Fails if `tenant_id`, `client_id`, or `client_secret` are missing.
    pub fn build(self) -> Result<GraphMailClient> {
        let tenant_id = self
            .tenant_id
            .ok_or_else(|| Error::Config("tenant_id is required".into()))?;
        let client_id = self
            .client_id
            .ok_or_else(|| Error::Config("client_id is required".into()))?;
        let client_secret = self
            .client_secret
            .ok_or_else(|| Error::Config("client_secret is required".into()))?;

        let token_url = self.token_url.unwrap_or_else(|| {
            format!(
                "https://login.microsoftonline.com/{}/oauth2/v2.0/token",
                tenant_id
            )
        });
        let graph_base = self
            .graph_base
            .unwrap_or_else(|| DEFAULT_GRAPH_BASE.to_string())
            .trim_end_matches('/')
            .to_string();
        let scope = self.scope.unwrap_or_else(|| DEFAULT_SCOPE.to_string());

        let http_client = Client::builder()
            .build()
            .map_err(|e| Error::Config(format!("HTTP client: {}", e)))?;

        let token_provider = TokenProvider::new(
            tenant_id,
            client_id,
            client_secret,
            token_url,
            scope,
            http_client.clone(),
        );

        Ok(GraphMailClient {
            http_client,
            token_provider: Arc::new(token_provider),
            graph_base,
        })
    }
}

/// Client for sending email via Microsoft Graph (app-only, client credentials).
///
/// Obtain an access token using the OAuth2 client credentials flow and call the Graph
/// `POST /users/{id}/sendMail` API. Use [`GraphMailClient::builder`] to construct.
#[derive(Clone)]
pub struct GraphMailClient {
    http_client: Client,
    token_provider: Arc<TokenProvider>,
    graph_base: String,
}

impl GraphMailClient {
    /// Return a new builder for configuring and creating a [`GraphMailClient`].
    pub fn builder() -> GraphMailClientBuilder {
        GraphMailClientBuilder::new()
    }

    /// Send an email as the given user (user id or userPrincipalName).
    ///
    /// The app must have **Mail.Send** application permission and admin consent.
    /// The user must have a mailbox in Exchange Online.
    pub async fn send_mail(&self, from_user: &str, request: SendMailRequest) -> Result<()> {
        let token = self.token_provider.get_token().await?;
        let url = format!(
            "{}/users/{}/sendMail",
            self.graph_base,
            urlencoding::encode(from_user)
        );

        let res = self
            .http_client
            .post(&url)
            .bearer_auth(&token)
            .json(&request)
            .send()
            .await?;

        let status = res.status();
        if status.as_u16() == 202 {
            return Ok(());
        }

        let body = res.text().await.unwrap_or_default();
        Err(Error::Graph(format!(
            "sendMail failed: {} {}",
            status, body
        )))
    }
}
