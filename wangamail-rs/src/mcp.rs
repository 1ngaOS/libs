//! MCP (Model Context Protocol) server module for AI tools.
//!
//! When the `mcp` feature is enabled, expose a **send_email** tool so AI assistants
//! (Cursor, Claude, etc.) can send email via Microsoft Graph in projects that run
//! this server.
//!
//! ## Environment variables (for the MCP server process)
//!
//! - `AZURE_TENANT_ID` – Azure AD tenant ID
//! - `AZURE_CLIENT_ID` – App (client) ID
//! - `AZURE_CLIENT_SECRET` – Client secret
//!
//! The app must have **Mail.Send** application permission and admin consent.
//!
//! ## Running the MCP server (stdio)
//!
//! Build with `cargo build --features mcp`, then run the binary. It reads JSON-RPC
//! from stdin and writes to stdout, so it can be used as a subprocess MCP server
//! (e.g. in Cursor or Claude Desktop).
//!
//! ## Client configuration example (Cursor)
//!
//! In `.cursor/mcp.json` or Cursor MCP settings, add a server that runs the
//! wangamail-rs MCP binary with env vars set (e.g. via a wrapper script).

use crate::{BodyType, GraphMailClient, Message, MessageBody, Recipient, SendMailRequest};
use rmcp::model::*;
use rmcp::{ServerHandler, ServiceExt};
use schemars::JsonSchema;
use serde::Deserialize;
use std::sync::Arc;

/// Parameters for the **send_email** MCP tool.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct SendEmailParams {
    /// User id or userPrincipalName to send as (e.g. `user@tenant.onmicrosoft.com`).
    #[schemars(description = "User id or userPrincipalName to send as (e.g. user@tenant.onmicrosoft.com)")]
    pub from_user: String,

    /// Recipient email addresses (to).
    #[schemars(description = "Recipient email addresses (to)")]
    pub to: Vec<String>,

    /// Subject of the email.
    #[schemars(description = "Subject of the email")]
    pub subject: String,

    /// Body content (plain text or HTML depending on body_type).
    #[schemars(description = "Body content (plain text or HTML depending on body_type)")]
    pub body: String,

    /// Body format: "text" or "html". Default "text".
    #[schemars(description = "Body format: \"text\" or \"html\". Default \"text\"")]
    #[serde(default)]
    pub body_type: String,

    /// CC recipient email addresses. Optional.
    #[schemars(description = "CC recipient email addresses. Optional")]
    #[serde(default)]
    pub cc: Vec<String>,

    /// BCC recipient email addresses. Optional.
    #[schemars(description = "BCC recipient email addresses. Optional")]
    #[serde(default)]
    pub bcc: Vec<String>,
}

/// MCP server that exposes a **send_email** tool using wangamail-rs.
#[derive(Clone)]
pub struct WangaMailMcpServer {
    client: Arc<GraphMailClient>,
}

impl WangaMailMcpServer {
    /// Create an MCP server that uses the given [GraphMailClient].
    pub fn new(client: GraphMailClient) -> Self {
        Self {
            client: Arc::new(client),
        }
    }

    /// Build the MCP server from environment variables:
    /// `AZURE_TENANT_ID`, `AZURE_CLIENT_ID`, `AZURE_CLIENT_SECRET`.
    pub fn from_env() -> crate::Result<Self> {
        let tenant_id = std::env::var("AZURE_TENANT_ID")
            .map_err(|_| crate::Error::Config("AZURE_TENANT_ID not set".into()))?;
        let client_id = std::env::var("AZURE_CLIENT_ID")
            .map_err(|_| crate::Error::Config("AZURE_CLIENT_ID not set".into()))?;
        let client_secret = std::env::var("AZURE_CLIENT_SECRET")
            .map_err(|_| crate::Error::Config("AZURE_CLIENT_SECRET not set".into()))?;
        let client = GraphMailClient::builder()
            .tenant_id(tenant_id)
            .client_id(client_id)
            .client_secret(client_secret)
            .build()?;
        Ok(Self::new(client))
    }

    /// Run the MCP server over stdio (stdin/stdout). Use this as the main entry
    /// when the process is launched as an MCP subprocess.
    pub async fn run_stdio(self) -> rmcp::Result<()> {
        let transport = (tokio::io::stdin(), tokio::io::stdout());
        let service = self.serve(transport).await?;
        service.waiting().await?;
        Ok(())
    }

    fn send_email_tool() -> Tool {
        Tool::new(
            "send_email",
            "Send an email via Microsoft Graph on behalf of a Microsoft tenant user. Requires Azure app registration with Mail.Send application permission.",
            Arc::new(serde_json::Map::new()),
        )
        .with_input_schema::<SendEmailParams>()
    }
}

impl ServerHandler for WangaMailMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            ..Default::default()
        }
    }

    fn get_tool(&self, name: &str) -> Option<Tool> {
        if name == "send_email" {
            Some(Self::send_email_tool())
        } else {
            None
        }
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        Ok(ListToolsResult {
            tools: vec![Self::send_email_tool()],
            next_cursor: None,
            meta: None,
        })
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        if request.name != "send_email" {
            return Err(ErrorData::invalid_params(
                "unknown_tool",
                Some(serde_json::json!({ "name": request.name })),
            ));
        }

        let params: SendEmailParams = serde_json::from_value(request.arguments.unwrap_or(serde_json::Value::Null))
            .map_err(|e| ErrorData::invalid_params("invalid_arguments", Some(serde_json::json!({ "error": e.to_string() }))))?;

        let body_type = if params.body_type.eq_ignore_ascii_case("html") {
            BodyType::HTML
        } else {
            BodyType::Text
        };

        let to_recipients: Vec<Recipient> = params.to.iter().map(|a| Recipient::new(a.as_str())).collect();
        let cc_recipients: Vec<Recipient> = params.cc.iter().map(|a| Recipient::new(a.as_str())).collect();
        let bcc_recipients: Vec<Recipient> = params.bcc.iter().map(|a| Recipient::new(a.as_str())).collect();

        let message = Message {
            subject: params.subject,
            body: MessageBody {
                content_type: body_type,
                content: params.body,
            },
            to_recipients,
            cc_recipients,
            bcc_recipients,
            ..Default::default()
        };

        let req = SendMailRequest::new(message);

        match self.client.send_mail(&params.from_user, req).await {
            Ok(()) => Ok(CallToolResult::success(vec![
                Content {
                    raw: RawContent::Text(RawTextContent {
                        text: "Email sent successfully.".to_string(),
                        meta: None,
                    }),
                    annotations: None,
                },
            ])),
            Err(e) => Ok(CallToolResult::error(vec![
                Content {
                    raw: RawContent::Text(RawTextContent {
                        text: format!("Send failed: {}", e),
                        meta: None,
                    }),
                    annotations: None,
                },
            ])),
        }
    }
}
