//! # wangamail-rs
//!
//! Send email on behalf of a Microsoft tenant using [Microsoft Graph API](https://learn.microsoft.com/en-us/graph/overview)
//! and app registration credentials (OAuth2 client credentials flow). Part of the WangaMail family
//! (`wangamail-rs`, `wangamail-js`, `wangamail-py`, `wangamail-net`).
//!
//! ---
//!
//! Developed with love by **1nga Solutions** · Logo: <https://www.1nga.com/logo.svg>
//!
//! ## Features
//!
//! - **Default:** Build a [`GraphMailClient`] and send mail with [`send_mail`](GraphMailClient::send_mail).
//! - **`mcp`:** Optional [Model Context Protocol](https://modelcontextprotocol.io/) server that exposes a
//!   **send_email** tool for AI assistants (Cursor, Claude Desktop, etc.). See the [`mcp`] module.
//!
//! ## Setup
//!
//! 1. Register an application in [Azure Portal](https://portal.azure.com) → Microsoft Entra ID → App registrations.
//! 2. Create a client secret for the app.
//! 3. Under **API permissions**, add application permission **Mail.Send** for Microsoft Graph and grant admin consent.
//! 4. To send as a specific user, that user must have a mailbox in Exchange Online; the app sends as the user identified by `from_user` (user id or userPrincipalName).
//!
//! ## Example
//!
//! ```no_run
//! use wangamail_rs::{GraphMailClient, Message, BodyType, Recipient, SendMailRequest};
//!
//! # async fn run() -> Result<(), Box<dyn std::error::Error>> {
//! let client = GraphMailClient::builder()
//!     .tenant_id("your-tenant-id")
//!     .client_id("your-client-id")
//!     .client_secret("your-client-secret")
//!     .build()?;
//!
//! let request = SendMailRequest::new(Message {
//!     subject: "Hello from Graph".to_string(),
//!     body: wangamail_rs::MessageBody {
//!         content_type: BodyType::Text,
//!         content: "This email was sent via Microsoft Graph.".to_string(),
//!     },
//!     to_recipients: vec![
//!         Recipient::new("recipient@example.com"),
//!     ],
//!     ..Default::default()
//! });
//!
//! client.send_mail("user@yourtenant.onmicrosoft.com", request).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Modules
//!
//! - **[`mcp`]** (optional, feature `mcp`): MCP server and **send_email** tool for AI tool use.

#![warn(missing_docs)]

mod auth;
mod client;
mod error;
mod graph;

#[cfg(feature = "mcp")]
pub mod mcp;

pub use client::{GraphMailClient, GraphMailClientBuilder};
pub use error::{Error, Result};
pub use graph::{
    BodyType, EmailAddress, FileAttachment, Message, MessageBody, Recipient, SendMailRequest,
};
