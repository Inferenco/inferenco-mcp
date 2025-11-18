use inferenco_mcp::server::ToolService;
use rmcp::{transport::stdio, ServiceExt};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let service = ToolService::new()
        .serve(stdio())
        .await
        .inspect_err(|error| {
            tracing::error!(%error, "failed to start MCP server");
        })?;

    tracing::info!(
        "Inferenco MCP server is running with protocol version {}",
        rmcp::model::ProtocolVersion::LATEST
    );
    tracing::info!("Available tools: echo, increment");

    service.waiting().await?;
    Ok(())
}
