# Inferenco MCP – Extended Documentation

Inferenco MCP is intentionally small so you can understand how an MCP server
fits together. This document expands on the README and is split into three
layers: architecture, operations/configuration, and extensibility.

---

## 1. Architecture Overview

### 1.1 Components

| Component | Description |
| --- | --- |
| `ToolService` | Implements the tools and exposes an `rmcp::ServerHandler`. Lives in `src/server`. |
| `inferenco-mcp-stdio` | Binary entrypoint in `src/main.rs`. Boots tracing, selects transport, and runs the handler. |
| `rmcp` crate | Provides derive macros (`#[tool]`, `#[tool_router]`, `#[tool_handler]`) plus JSON-RPC glue. |
| Example client | `examples/test_client.rs` calls the tools directly, no JSON-RPC required. |
| Docker + scripts | Production-ish wrappers for building/running the server with consistent env vars. |

### 1.2 Tool Flow

1. A client sends `tools/list`. rmcp forwards the request to `ToolService`,
   which returns `tool_router.list_all()` – a vector describing every tool.
2. A client calls `tools/call` with `{ name, arguments }`.
3. rmcp matches the tool name, deserializes arguments into the requested struct,
   and executes the async Rust method (e.g., `echo` or `increment`).
4. Results become `CallToolResult` payloads containing text content and
   optional structured data.

### 1.3 Tool Implementations

- `echo` expects `EchoArgs { message: String }` and returns that message.
- `increment` mutates an in-memory counter stored in `Arc<Mutex<u32>>`.

Both tools demonstrate the two handler patterns you will typically need:
argument extraction via `Parameters<T>` and stateful access via shared structs.

---

## 2. Configuration & Operations

### 2.1 Environment Variables

The binary reads configuration from the process environment. The table below
mirrors `.env.example` and the Docker compose file.

| Variable | Type | Default | Description |
| --- | --- | --- | --- |
| `INFERENCO_MCP_TRANSPORT` | enum | `stdio` | Transport to start (`stdio` or `http`). |
| `INFERENCO_MCP_PORT` | u16 | `8080` | HTTP port (only used when transport = `http`). |
| `INFERENCO_MCP_LOG_LEVEL` | string | `info` | log level consumed by `tracing-subscriber`. |
| `INFERENCO_MCP_AUTH_ENABLED` | bool | `false` | Enables simple API-key auth for HTTP transport. |
| `INFERENCO_MCP_API_KEYS` | string | _empty_ | Comma-separated list of valid API keys. |
| `INFERENCO_MCP_AUTH_HEADER` | string | `x-api-key` | HTTP header to read when auth is on. |

> Tip: add `RUST_LOG=debug` when debugging the transport itself. The server
> already prints the protocol version and tool list on startup.

### 2.2 `.env` Workflow

1. Copy `.env.example` to `.env`.
2. Update the entries for your local environment (e.g., `INFERENCO_MCP_API_KEYS=devkey123`).
3. When using `cargo run`, the workspace automatically loads the file through
   standard dotenv tooling (if installed). Otherwise export the variables in
   your shell or let Docker compose do it.

### 2.3 TOML Configuration

`config.example.toml` acts as a reference when you prefer file-backed
configuration (e.g., for container images that do not rely on `.env`). The
sections map as follows:

| Section | Fields | Purpose |
| --- | --- | --- |
| `[server]` | `transport`, `port`, `log_level` | Controls the runtime transport and logging defaults. |
| `[auth]` | `enabled`, `allowed_keys`, `header_name` | Mirrors the environment-based auth settings. |
| `[apis]` | Optional API keys | Placeholder for future third-party integrations. |
| `[cache]` | `ttl_seconds`, `max_entries` | Reserved knobs if you add caching layers. |

The current binary does not read the TOML on its own; the template exists to
help teams that integrate the crate into larger applications.

### 2.4 Deployment Options

- **Local dev:** `cargo run --bin inferenco-mcp-stdio`. Recommended when testing
  stdio responses with OpenAI’s MCP clients.
- **Docker:** `docker compose -f docker/docker-compose.yml up --build`. Uses the
  same environment variables and exposes port `8080`.
- **CI/CD:** `scripts/build.sh` and `scripts/test.sh` provide deterministic
  entrypoints for pipelines.

### 2.5 Observability

- Logging: configured via `tracing-subscriber`. Respect `RUST_LOG` and
  `INFERENCO_MCP_LOG_LEVEL`.
- Health: the docker-compose file defines a basic HTTP POST healthcheck against
  `/rpc` so container orchestrators know when the server is ready.

---

## 3. Extending Inferenco MCP

### 3.1 Adding Tools

1. **Create DTOs:** add new structs in `src/server/dto.rs`. Derive
   `serde::Deserialize` and `schemars::JsonSchema`.
2. **Add handler:** implement an async method inside the `#[tool_router]`
   impl block (see `src/server/implementation.rs`). Annotate it with
   `#[tool(description = "...")]`.
3. **Return values:** use `CallToolResult::success(vec![Content::text(...)])` or
   `CallToolResult::structured(json!(...))`.
4. **State management:** store shared state on `ToolService` (e.g., `Arc<Mutex<_>>`)
   or wire in dependencies during `ToolService::new()`.

rmcp auto-updates the tool schema advertised to clients based on the handler
signature and `Parameters<T>` type.

### 3.2 Switching Transports

The binary currently ships with stdio enabled; HTTP requires you to introduce an
HTTP transport and wire it into `ServiceExt::serve`. If you embed this crate
into a larger application, you can reuse `ToolService` and supply your own
transport layer.

### 3.3 Integrating with Other Systems

Because the tools are plain async functions, nothing prevents you from calling
databases, REST APIs, or third-party SDKs. Keep these guidelines in mind:

- Avoid long blocking tasks; stay async.
- Perform validation in the DTOs or handler logic.
- Consider returning structured content for machine-readable results.

---

## 4. Troubleshooting & FAQ

| Symptom | Likely Cause | Fix |
| --- | --- | --- |
| `cargo run` prints nothing | Log level is filtering everything | Set `RUST_LOG=info` or `INFERENCO_MCP_LOG_LEVEL=info`. |
| HTTP requests hang | HTTP transport not enabled | Export `INFERENCO_MCP_TRANSPORT=http` and restart. |
| Unauthorized errors over HTTP | Auth enabled but no header | Supply `INFERENCO_MCP_AUTH_HEADER` with one of the comma-separated keys. |
| Example client panics | Tool signatures changed | Rebuild and ensure `available_tools()` returns the new tool metadata. |

If you encounter something not covered here, open an issue or inspect the
trace-level logs – rmcp surfaces most protocol errors clearly once you enable
`RUST_LOG=debug`.
