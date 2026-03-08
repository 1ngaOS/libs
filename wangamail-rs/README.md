# wangamail-rs

Send email on behalf of a Microsoft tenant using [Microsoft Graph API](https://learn.microsoft.com/en-us/graph/overview) and app registration credentials (OAuth2 client credentials flow). Suitable for daemon apps and backend services. Part of the WangaMail family: **wangamail-rs** (Rust), **wangamail-js** (JavaScript), **wangamail-py** (Python), **wangamail-net** (.NET).

## Requirements

- **Rust** 2021 edition
- An **async runtime** (e.g. `tokio` with `full` or `rt-multi-thread` + `macros`) in your binary when calling the async API

## Azure setup

1. In [Azure Portal](https://portal.azure.com), go to **Microsoft Entra ID** → **App registrations** → **New registration**.
2. Create a **client secret** under **Certificates & secrets**.
3. Under **API permissions**, add **Microsoft Graph** → **Application permissions** → **Mail.Send**, then **Grant admin consent**.
4. To send as a specific user, use that user’s **Object (ID)** or **User principal name** (e.g. `user@yourtenant.onmicrosoft.com`) as `from_user`. The app will send mail from that user’s mailbox (Exchange Online).

## Usage

Add to `Cargo.toml`:

```toml
[dependencies]
wangamail-rs = "0.1"
tokio = { version = "1", features = ["full"] }
```

Example:

```rust
use wangamail_rs::{GraphMailClient, Message, MessageBody, BodyType, Recipient, SendMailRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = GraphMailClient::builder()
        .tenant_id(std::env::var("AZURE_TENANT_ID")?)
        .client_id(std::env::var("AZURE_CLIENT_ID")?)
        .client_secret(std::env::var("AZURE_CLIENT_SECRET")?)
        .build()?;

    let request = SendMailRequest::new(Message {
        subject: "Hello from Graph".to_string(),
        body: MessageBody {
            content_type: BodyType::Text,
            content: "This email was sent via Microsoft Graph.".to_string(),
        },
        to_recipients: vec![Recipient::new("recipient@example.com")],
        ..Default::default()
    });

    // Send as this user (must have a mailbox; app needs Mail.Send application permission)
    client
        .send_mail("user@yourtenant.onmicrosoft.com", request)
        .await?;

    Ok(())
}
```

## Sovereign clouds

Override the token and Graph endpoints when using national clouds:

```rust
let client = GraphMailClient::builder()
    .tenant_id(tenant_id)
    .client_id(client_id)
    .client_secret(client_secret)
    .token_url("https://login.microsoftonline.us/your-tenant-id/oauth2/v2.0/token")
    .graph_base("https://graph.microsoft.us/v1.0")
    .build()?;
```

## MCP (Model Context Protocol)

With the **mcp** feature, the crate exposes an MCP server so AI assistants (Cursor, Claude Desktop, etc.) can send email via the **send_email** tool.

### Enable the feature

```toml
[dependencies]
wangamail-rs = { version = "0.1", features = ["mcp"] }
```

### Run the MCP server (stdio)

Set `AZURE_TENANT_ID`, `AZURE_CLIENT_ID`, and `AZURE_CLIENT_SECRET`, then:

```bash
cargo run --example mcp_server --features mcp
```

The process speaks JSON-RPC over stdin/stdout. Configure your AI client to run this command as an MCP server (e.g. in Cursor MCP settings or Claude Desktop `claude_desktop_config.json`).

### Tool: send_email

- **from_user** – User id or userPrincipalName to send as (e.g. `user@tenant.onmicrosoft.com`).
- **to** – List of recipient email addresses.
- **subject** – Subject line.
- **body** – Plain text or HTML body.
- **body_type** – `"text"` or `"html"` (default `"text"`).
- **cc**, **bcc** – Optional lists of addresses.

### Use from code

```rust
use wangamail_rs::mcp::WangaMailMcpServer;

// From env (AZURE_TENANT_ID, AZURE_CLIENT_ID, AZURE_CLIENT_SECRET)
let server = WangaMailMcpServer::from_env()?;

// Or with an existing client
let server = WangaMailMcpServer::new(client);

// Run over stdio (for subprocess use)
server.run_stdio().await?;
```

## CI/CD

- **Pull requests:** Format (`cargo fmt --check`), lint (clippy), check, test.
- **Push to `main`:** Bump patch version, build, publish to [crates.io](https://crates.io/crates/wangamail-rs), create a GitHub release.

To enable **publish to crates.io**, add a repository secret:

- **`CARGO_REGISTRY_TOKEN`** – crates.io API token with publish scope ([crates.io/settings/tokens](https://crates.io/settings/tokens)).

## License

MIT OR Apache-2.0
