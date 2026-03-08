//! Run the WangaMail MCP server over stdio.
//!
//! Requires the `mcp` feature and env vars: AZURE_TENANT_ID, AZURE_CLIENT_ID, AZURE_CLIENT_SECRET.
//!
//! Run: cargo run --example mcp_server --features mcp

#[cfg(feature = "mcp")]
fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let server = wangamail_rs::mcp::WangaMailMcpServer::from_env()?;
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(server.run_stdio())?;
    Ok(())
}

#[cfg(not(feature = "mcp"))]
fn main() {
    eprintln!("Build with --features mcp to run the MCP server.");
    std::process::exit(1);
}
