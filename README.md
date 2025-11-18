# Nova MCP (Rust SDK Edition)

A minimal Model Context Protocol (MCP) server built with the official [rust-sdk](https://github.com/modelcontextprotocol/rust-sdk).
It exposes two simple, unauthenticated tools and targets the `2024-11-05` MCP protocol version so it works with OpenAI Responses.

## Tools
- **echo**: Echo back the provided `message` string.
- **increment**: Increment an in-memory counter and return the new value.

## Running
```bash
# Run over stdio (ideal for OpenAI Responses MCP tool)
cargo run --bin nova-mcp-stdio
```

Set the `RUST_LOG` environment variable (e.g., `RUST_LOG=debug`) to see additional server logs.

## How it Works
The server is implemented with the `rmcp` crate's tool macros. It advertises the `2024-11-05` protocol version
for compatibility with the current OpenAI MCP implementation and does not require any API keys.
