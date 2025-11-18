use crate::server::EchoArgs;
use rmcp::{
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{
        CallToolResult, Content, Implementation, ProtocolVersion, ServerCapabilities, ServerInfo,
        Tool,
    },
    tool, tool_handler, tool_router, ErrorData as McpError,
};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct ToolService {
    counter: Arc<Mutex<u32>>,
    tool_router: ToolRouter<Self>,
}

impl ToolService {
    pub fn new() -> Self {
        Self {
            counter: Arc::new(Mutex::new(0)),
            tool_router: Self::tool_router(),
        }
    }

    /// Return the list of tools this service exposes.
    pub fn available_tools(&self) -> Vec<Tool> {
        self.tool_router.list_all()
    }
}

impl Default for ToolService {
    fn default() -> Self {
        Self::new()
    }
}

#[tool_router(vis = "pub")]
impl ToolService {
    #[tool(description = "Echo back the provided message.")]
    pub async fn echo(
        &self,
        Parameters(args): Parameters<EchoArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(CallToolResult::success(vec![Content::text(args.message)]))
    }

    #[tool(description = "Increment an in-memory counter and return the new value.")]
    pub async fn increment(&self) -> Result<CallToolResult, McpError> {
        let mut counter = self.counter.lock().await;
        *counter += 1;
        Ok(CallToolResult::success(vec![Content::text(
            counter.to_string(),
        )]))
    }
}

#[tool_handler]
impl rmcp::ServerHandler for ToolService {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation::from_build_env(),
            instructions: Some(
                "A minimal MCP tool server built with the official Rust SDK. ".to_string()
                    + "Provides echo and counter tools without any API key requirements.",
            ),
        }
    }
}
