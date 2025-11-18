use axum::body::Bytes;
use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::{sse::Event, IntoResponse, Json, Sse},
    routing::{get, post},
    Router,
};
use dotenvy::dotenv;
use inferenco_mcp::server::ToolService;
use rmcp::{transport::stdio, ServiceExt};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, convert::Infallible, env, sync::Arc, time::Duration};
use tokio_stream::{Stream, StreamExt as _};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    #[serde(default)]
    id: Option<serde_json::Value>,
    method: String,
    #[serde(default)]
    params: Option<serde_json::Value>,
}

#[derive(Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<serde_json::Value>,
}

async fn handle_rpc(
    State(service): State<Arc<ToolService>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<JsonRpcResponse>, StatusCode> {
    let body = String::from_utf8(body.to_vec()).map_err(|_| StatusCode::BAD_REQUEST)?;
    // Check authentication if enabled
    if env::var("INFERENCO_MCP_AUTH_ENABLED").unwrap_or_else(|_| "false".to_string()) == "true" {
        let auth_header =
            env::var("INFERENCO_MCP_AUTH_HEADER").unwrap_or_else(|_| "x-api-key".to_string());
        let api_keys = env::var("INFERENCO_MCP_API_KEYS")
            .unwrap_or_default()
            .split(',')
            .map(|s| s.trim().to_string())
            .collect::<Vec<_>>();

        if let Some(header_value) = headers.get(&auth_header) {
            let provided_key = header_value.to_str().unwrap_or("");
            if !api_keys.contains(&provided_key.to_string()) {
                return Err(StatusCode::UNAUTHORIZED);
            }
        } else {
            return Err(StatusCode::UNAUTHORIZED);
        }
    }

    let request: JsonRpcRequest =
        serde_json::from_str(&body).map_err(|_| StatusCode::BAD_REQUEST)?;

    if request.jsonrpc != "2.0" {
        return Ok(Json(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id.unwrap_or(serde_json::Value::Null),
            result: None,
            error: Some(serde_json::json!({
                "code": -32600,
                "message": "Invalid Request"
            })),
        }));
    }

    // Handle notifications (requests without id) - just acknowledge, don't respond
    if request.id.is_none() {
        // For notifications, we still process them but return empty response or 204
        // Actually, JSON-RPC 2.0 says notifications should not receive a response
        // But HTTP requires a response, so we'll return a minimal one
        match request.method.as_str() {
            "notifications/initialized" => {
                // Client is notifying us that initialization is complete
                // Return empty response for notifications
                return Ok(Json(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: serde_json::Value::Null,
                    result: Some(serde_json::json!({})),
                    error: None,
                }));
            }
            _ => {
                // Unknown notification, just acknowledge
                return Ok(Json(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: serde_json::Value::Null,
                    result: Some(serde_json::json!({})),
                    error: None,
                }));
            }
        }
    }

    let response = match request.method.as_str() {
        "initialize" => {
            let server_info = service.get_server_info();
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id.unwrap_or(serde_json::Value::Null),
                result: Some(serde_json::json!({
                    "protocolVersion": server_info.protocol_version.to_string(),
                    "capabilities": {
                        "tools": {}
                    },
                    "serverInfo": {
                        "name": server_info.server_info.name,
                        "version": server_info.server_info.version
                    }
                })),
                error: None,
            }
        }
        "tools/list" => {
            let tools = service.available_tools();
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id.unwrap_or(serde_json::Value::Null),
                result: Some(serde_json::json!({
                    "tools": tools
                })),
                error: None,
            }
        }
        "tools/call" => {
            if let Some(params) = request.params {
                if let (Some(name), args) = (
                    params.get("name").and_then(|v| v.as_str()),
                    params
                        .get("arguments")
                        .cloned()
                        .unwrap_or(serde_json::json!({})),
                ) {
                    match service.call_tool(name, args).await {
                        Ok(result) => {
                            // Convert CallToolResult to MCP response format
                            let content: Vec<serde_json::Value> = result
                                .content
                                .into_iter()
                                .map(|c| match c.raw {
                                    rmcp::model::RawContent::Text(text) => {
                                        serde_json::json!({"type": "text", "text": text.text})
                                    }
                                    rmcp::model::RawContent::Resource(_)
                                    | rmcp::model::RawContent::Image(_)
                                    | rmcp::model::RawContent::Audio(_)
                                    | rmcp::model::RawContent::ResourceLink(_) => {
                                        // Other content types not fully implemented yet
                                        serde_json::json!({
                                            "type": "text",
                                            "text": "Content type not supported"
                                        })
                                    }
                                })
                                .collect();

                            JsonRpcResponse {
                                jsonrpc: "2.0".to_string(),
                                id: request.id.unwrap_or(serde_json::Value::Null),
                                result: Some(serde_json::json!({
                                    "content": content
                                })),
                                error: None,
                            }
                        }
                        Err(e) => JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            id: request.id.unwrap_or(serde_json::Value::Null),
                            result: None,
                            error: Some(serde_json::json!({
                                "code": -32603,
                                "message": e.to_string()
                            })),
                        },
                    }
                } else {
                    JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id: request.id.unwrap_or(serde_json::Value::Null),
                        result: None,
                        error: Some(serde_json::json!({
                            "code": -32602,
                            "message": "Invalid params"
                        })),
                    }
                }
            } else {
                JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id.unwrap_or(serde_json::Value::Null),
                    result: None,
                    error: Some(serde_json::json!({
                        "code": -32602,
                        "message": "Invalid params"
                    })),
                }
            }
        }
        _ => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id.unwrap_or(serde_json::Value::Null),
            result: None,
            error: Some(serde_json::json!({
                "code": -32601,
                "message": "Method not found"
            })),
        },
    };

    Ok(Json(response))
}

async fn handle_health() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "service": "inferenco-mcp",
        "protocol_version": rmcp::model::ProtocolVersion::LATEST.to_string()
    }))
}

fn create_keepalive_stream() -> impl Stream<Item = Result<Event, Infallible>> + Send + 'static {
    tokio_stream::wrappers::IntervalStream::new(tokio::time::interval(Duration::from_secs(30)))
        .map(|_| Ok(Event::default().comment("keepalive")))
}

async fn handle_sse(
    State(service): State<Arc<ToolService>>,
    Query(params): Query<HashMap<String, String>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>> + Send + 'static> {
    let service_clone = service.clone();

    // Handle authentication if enabled
    let auth_enabled =
        env::var("INFERENCO_MCP_AUTH_ENABLED").unwrap_or_else(|_| "false".to_string()) == "true";

    // Check authentication first
    if auth_enabled {
        let api_keys: Vec<String> = env::var("INFERENCO_MCP_API_KEYS")
            .unwrap_or_default()
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();

        let is_authorized = if let Some(token) = params.get("token") {
            api_keys.contains(token)
        } else {
            false
        };

        if !is_authorized {
            // Return error event
            let error_event = Event::default()
                .json_data(serde_json::json!({
                    "jsonrpc": "2.0",
                    "error": {
                        "code": -32000,
                        "message": if params.get("token").is_some() { "Unauthorized" } else { "Authentication required" }
                    }
                }))
                .unwrap();
            let error_stream = tokio_stream::once(Ok(error_event));
            let stream = error_stream.chain(create_keepalive_stream());
            return Sse::new(stream).keep_alive(
                axum::response::sse::KeepAlive::new()
                    .interval(Duration::from_secs(15))
                    .text("keep-alive-text"),
            );
        }
    }

    // Send initial connection event
    let server_info = service_clone.get_server_info();
    let init_event = Event::default()
        .json_data(serde_json::json!({
            "jsonrpc": "2.0",
            "result": {
                "protocolVersion": server_info.protocol_version.to_string(),
                "capabilities": {
                    "tools": {}
                },
                "serverInfo": {
                    "name": server_info.server_info.name,
                    "version": server_info.server_info.version
                }
            }
        }))
        .unwrap();

    // Create a stream that sends the initial event and then keeps connection alive
    let init_stream = tokio_stream::once(Ok(init_event));
    let stream = init_stream.chain(create_keepalive_stream());

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep-alive-text"),
    )
}

async fn handle_sse_message(
    State(service): State<Arc<ToolService>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<JsonRpcResponse>, StatusCode> {
    // SSE messages can also be sent via POST to /sse endpoint
    // This allows bidirectional communication
    handle_rpc(State(service), headers, body).await
}

async fn start_http_server(service: ToolService) -> Result<(), Box<dyn std::error::Error>> {
    let port = env::var("INFERENCO_MCP_PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .unwrap_or(8080);

    let service = Arc::new(service);

    let app = Router::new()
        .route("/rpc", post(handle_rpc))
        .route("/sse", get(handle_sse).post(handle_sse_message))
        .route("/health", get(handle_health))
        .route("/", get(handle_health))
        .with_state(service);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;

    tracing::info!("Inferenco MCP server listening on http://0.0.0.0:{}", port);
    tracing::info!("  - JSON-RPC endpoint: http://0.0.0.0:{}/rpc", port);
    tracing::info!("  - SSE endpoint: http://0.0.0.0:{}/sse", port);
    tracing::info!("  - Health endpoint: http://0.0.0.0:{}/health", port);
    tracing::info!(
        "Inferenco MCP server is running with protocol version {}",
        rmcp::model::ProtocolVersion::LATEST
    );
    tracing::info!("Available tools: echo, reverse_text, increment, current_time, roll_dice");

    axum::serve(listener, app).await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let transport = env::var("INFERENCO_MCP_TRANSPORT").unwrap_or_else(|_| "stdio".to_string());
    let service = ToolService::new();

    match transport.as_str() {
        "http" => {
            start_http_server(service).await?;
        }
        "stdio" => {
            let server = service.serve(stdio()).await.inspect_err(|error| {
                tracing::error!(%error, "failed to start MCP server");
            })?;

            tracing::info!(
                "Inferenco MCP server is running with protocol version {}",
                rmcp::model::ProtocolVersion::LATEST
            );
            tracing::info!(
                "Available tools: echo, reverse_text, increment, current_time, roll_dice"
            );

            // This will never return for stdio transport
            server.waiting().await?;
        }
        _ => {
            // Default to stdio for unknown transport values
            tracing::warn!("Unknown transport '{}', defaulting to stdio", transport);
            let server = service.serve(stdio()).await.inspect_err(|error| {
                tracing::error!(%error, "failed to start MCP server");
            })?;

            tracing::info!(
                "Inferenco MCP server is running with protocol version {}",
                rmcp::model::ProtocolVersion::LATEST
            );
            tracing::info!(
                "Available tools: echo, reverse_text, increment, current_time, roll_dice"
            );

            // This will never return for stdio transport
            server.waiting().await?;
        }
    }

    Ok(())
}
