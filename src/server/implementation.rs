use crate::server::{DiceArgs, EchoArgs, ReverseArgs};
use chrono::Utc;
use rand::Rng;
use rmcp::{
    handler::server::{router::tool::ToolRouter, wrapper::Parameters, ServerHandler},
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

    /// Get server info for initialization.
    pub fn get_server_info(&self) -> ServerInfo {
        self.get_info()
    }

    /// Call a tool by name with the provided arguments.
    pub async fn call_tool(
        &self,
        name: &str,
        arguments: serde_json::Value,
    ) -> Result<CallToolResult, McpError> {
        match name {
            "echo" => {
                let args: EchoArgs = serde_json::from_value(arguments)
                    .map_err(|_| McpError::invalid_params("Invalid echo arguments", None))?;
                self.echo(Parameters(args)).await
            }
            "reverse_text" => {
                let args: ReverseArgs = serde_json::from_value(arguments).map_err(|_| {
                    McpError::invalid_params("Invalid reverse_text arguments", None)
                })?;
                self.reverse_text(Parameters(args)).await
            }
            "increment" => self.increment().await,
            "current_time" => self.current_time().await,
            "roll_dice" => {
                let args: DiceArgs = serde_json::from_value(arguments)
                    .map_err(|_| McpError::invalid_params("Invalid roll_dice arguments", None))?;
                self.roll_dice(Parameters(args)).await
            }
            _ => Err(McpError::invalid_params("Tool not found", None)),
        }
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

    #[tool(description = "Reverse a piece of text.")]
    pub async fn reverse_text(
        &self,
        Parameters(args): Parameters<ReverseArgs>,
    ) -> Result<CallToolResult, McpError> {
        let reversed: String = args.text.chars().rev().collect();
        Ok(CallToolResult::success(vec![Content::text(reversed)]))
    }

    #[tool(description = "Increment an in-memory counter and return the new value.")]
    pub async fn increment(&self) -> Result<CallToolResult, McpError> {
        let mut counter = self.counter.lock().await;
        *counter += 1;
        Ok(CallToolResult::success(vec![Content::text(
            counter.to_string(),
        )]))
    }

    #[tool(description = "Return the current UTC time in RFC3339 format.")]
    pub async fn current_time(&self) -> Result<CallToolResult, McpError> {
        Ok(CallToolResult::success(vec![Content::text(
            Utc::now().to_rfc3339(),
        )]))
    }

    #[tool(description = "Roll a die with the provided number of sides (defaults to six-sided).")]
    pub async fn roll_dice(
        &self,
        Parameters(args): Parameters<DiceArgs>,
    ) -> Result<CallToolResult, McpError> {
        let sides = args.sides.max(2);
        let mut rng = rand::thread_rng();
        let value = rng.gen_range(1..=sides);

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Rolled {value} on a d{sides}"
        ))]))
    }
}

#[tool_handler]
impl rmcp::ServerHandler for ToolService {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::LATEST,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation::from_build_env(),
            instructions: Some(
                "A minimal MCP tool server built with the official Rust SDK. ".to_string()
                    + "Provides echo, text transformation, dice roll, clock, and counter tools "
                    + "without any API key requirements.",
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rmcp::{
        handler::server::wrapper::Parameters,
        model::{CallToolResult, RawContent},
    };

    fn text_output(result: CallToolResult) -> String {
        result
            .content
            .into_iter()
            .find_map(|content| match content.raw {
                RawContent::Text(text) => Some(text.text),
                _ => None,
            })
            .expect("tool result to contain text")
    }

    #[tokio::test]
    async fn reverse_text_returns_reversed_string() {
        let service = ToolService::new();
        let output = service
            .reverse_text(Parameters(ReverseArgs {
                text: "Inferenco".to_string(),
            }))
            .await
            .expect("tool to succeed");

        assert_eq!(text_output(output), "ocnerefnI");
    }

    #[tokio::test]
    async fn current_time_emits_rfc3339_timestamp() {
        let service = ToolService::new();
        let output = service
            .current_time()
            .await
            .expect("tool to produce a timestamp");

        let text = text_output(output);
        assert!(
            text.contains('T'),
            "timestamp missing RFC3339 separator: {text}"
        );
        let parsed =
            chrono::DateTime::parse_from_rfc3339(&text).expect("timestamp should parse as RFC3339");
        assert_eq!(
            parsed.offset().local_minus_utc(),
            0,
            "timestamp should be UTC: {text}"
        );
    }

    #[tokio::test]
    async fn roll_dice_respects_requested_sides() {
        let service = ToolService::new();
        let sides = 12;
        let output = service
            .roll_dice(Parameters(DiceArgs { sides }))
            .await
            .expect("tool to roll successfully");

        let text = text_output(output);
        let parts: Vec<_> = text.split_whitespace().collect();
        assert_eq!(parts.len(), 5, "unexpected dice output format: {text}");
        assert_eq!(parts[0], "Rolled");
        let value: u8 = parts[1].parse().expect("rolled value should be a number");
        let reported_sides: u8 = parts[4]
            .trim_start_matches('d')
            .parse()
            .expect("sides suffix should parse");

        assert_eq!(reported_sides, sides);
        assert!((1..=sides).contains(&value), "roll {value} outside bounds");
    }

    #[tokio::test]
    async fn roll_dice_enforces_minimum_of_two_sides() {
        let service = ToolService::new();
        let output = service
            .roll_dice(Parameters(DiceArgs { sides: 1 }))
            .await
            .expect("tool to roll successfully");

        let text = text_output(output);
        let reported_sides: u8 = text
            .split_whitespace()
            .last()
            .and_then(|suffix| suffix.trim_start_matches('d').parse().ok())
            .expect("output should contain die size");
        assert_eq!(reported_sides, 2);
    }
}
