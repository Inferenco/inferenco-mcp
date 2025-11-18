# Inferenco MCP Server

A tiny-but-complete Model Context Protocol (MCP) server powered by the official
[rmcp](https://github.com/modelcontextprotocol/rust-sdk) crate. Inferenco MCP
focuses on being the simplest possible reference implementation: it exposes a
handful of fun demo tools (echo, reverse text, dice roll, UTC clock, and a
stateful counter), runs happily over stdio or HTTP, and ships with ready-to-run
Docker and shell scripts.

---

## Feature Highlights

- :sparkles: **Five demo tools out of the box** – echo, reverse text, dice roll,
  UTC clock, and a stateful counter
- :electric_plug: **Rust async (Tokio) runtime** with rmcp’s derive macros
- :gear: **Multiple transports** – stdio by default, HTTP ready through env vars
- :card_file_box: **Deterministic configuration** via environment variables or a
  TOML file (`config.example.toml`)
- :package: **Dockerfile + docker-compose.yaml** for rapid deployment
- :test_tube: **Example client** (`examples/test_client.rs`) showing direct API
  usage without spinning up the full server

---

## Repository Layout

```
src/
├── main.rs                 # inferenco-mcp-stdio binary entrypoint
└── server/                 # Tool implementations + rmcp wiring
    ├── dto.rs              # Tool argument structs
    ├── implementation.rs   # ToolService implementation
    └── mod.rs
examples/
└── test_client.rs          # Demonstrates calling tools directly
scripts/                    # Helper scripts (build/test)
docker/                     # Container build + compose files
config.example.toml         # Optional config file template
.env.example                # Environment variable template
documentation.md            # Extended system documentation
```

---

## Quick Start

```bash
git clone https://github.com/your-org/inferenco-mcp
cd inferenco-mcp
cargo run --bin inferenco-mcp-stdio
```

You should see log lines telling you that the server is running and which tools
are available. By default the binary serves over stdio, which is the transport
required by OpenAI’s MCP tool interface.

### Example Client

```bash
cargo run --example test_client
```

The example lists every tool and invokes some of them (`echo`, `increment`, etc.)
using the public `ToolService` API.

---

## Configuration

Inferenco MCP reads configuration from environment variables (e.g. a `.env`
file, Docker env block, or shell exports). The most common options are:

| Variable | Default | Purpose |
| --- | --- | --- |
| `INFERENCO_MCP_TRANSPORT` | `stdio` | Transport to start (`stdio` or `http`) |
| `INFERENCO_MCP_PORT` | `8080` | HTTP port (when transport = `http`) |
| `INFERENCO_MCP_LOG_LEVEL` | `info` | Log level passed to `tracing-subscriber` |
| `INFERENCO_MCP_AUTH_ENABLED` | `false` | Whether HTTP requests require an API key |
| `INFERENCO_MCP_API_KEYS` | _empty_ | Comma-separated API keys when auth is enabled |
| `INFERENCO_MCP_AUTH_HEADER` | `x-api-key` | HTTP header that carries the API key |

You can copy `.env.example` to `.env` and tweak the values locally. Docker
resources under `docker/` already export the environment variables documented
above.

### TOML Configuration

A sample `config.example.toml` is provided for teams that prefer file-based
configuration. The format mirrors the environment variables; the binary takes
the env vars first and uses the TOML file as a fallback.

---

## Integrating with the Responses API

When running in HTTP mode, Inferenco MCP exposes a JSON-RPC 2.0 endpoint that can be integrated with external services or used as a standalone API.

### Setup

1. **Enable HTTP transport:**

```bash
export INFERENCO_MCP_TRANSPORT=http
export INFERENCO_MCP_PORT=8080
```

Or add to your `.env` file:
```bash
INFERENCO_MCP_TRANSPORT=http
INFERENCO_MCP_PORT=8080
```

2. **Start the server:**

```bash
cargo run --bin inferenco-mcp-stdio
```

The server will listen on `http://localhost:8080` (or your configured port).

### API Endpoint

The MCP server exposes a single JSON-RPC endpoint:

- **URL:** `http://localhost:8080/rpc`
- **Method:** `POST`
- **Content-Type:** `application/json`

### Authentication (Optional)

If you've enabled authentication:

```bash
export INFERENCO_MCP_AUTH_ENABLED=true
export INFERENCO_MCP_API_KEYS=your-api-key-1,your-api-key-2
export INFERENCO_MCP_AUTH_HEADER=x-api-key
```

Include the API key in your requests:

```bash
curl -X POST http://localhost:8080/rpc \
  -H "Content-Type: application/json" \
  -H "x-api-key: your-api-key-1" \
  -d '{"jsonrpc":"2.0","id":1,"method":"tools/list"}'
```

### Making Requests

#### List Available Tools

```bash
curl -X POST http://localhost:8080/rpc \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "tools/list"
  }'
```

#### Call a Tool

```bash
curl -X POST http://localhost:8080/rpc \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 2,
    "method": "tools/call",
    "params": {
      "name": "echo",
      "arguments": {
        "message": "Hello, MCP!"
      }
    }
  }'
```

#### Example: Increment Counter

```bash
curl -X POST http://localhost:8080/rpc \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 3,
    "method": "tools/call",
    "params": {
      "name": "increment",
      "arguments": {}
    }
  }'
```

### Response Format

All responses follow the JSON-RPC 2.0 specification:

**Success Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "Hello, MCP!"
      }
    ]
  }
}
```

**Error Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32602,
    "message": "Invalid params"
  }
}
```

### Integration Examples

#### Python

```python
import requests

url = "http://localhost:8080/rpc"
headers = {
    "Content-Type": "application/json",
    # "x-api-key": "your-api-key"  # If auth is enabled
}

# List tools
response = requests.post(url, json={
    "jsonrpc": "2.0",
    "id": 1,
    "method": "tools/list"
}, headers=headers)
print(response.json())

# Call a tool
response = requests.post(url, json={
    "jsonrpc": "2.0",
    "id": 2,
    "method": "tools/call",
    "params": {
        "name": "echo",
        "arguments": {"message": "Hello from Python!"}
    }
}, headers=headers)
print(response.json())
```

#### JavaScript/Node.js

```javascript
const fetch = require('node-fetch');

const url = 'http://localhost:8080/rpc';
const headers = {
  'Content-Type': 'application/json',
  // 'x-api-key': 'your-api-key'  // If auth is enabled
};

// List tools
fetch(url, {
  method: 'POST',
  headers,
  body: JSON.stringify({
    jsonrpc: '2.0',
    id: 1,
    method: 'tools/list'
  })
})
  .then(res => res.json())
  .then(data => console.log(data));

// Call a tool
fetch(url, {
  method: 'POST',
  headers,
  body: JSON.stringify({
    jsonrpc: '2.0',
    id: 2,
    method: 'tools/call',
    params: {
      name: 'echo',
      arguments: { message: 'Hello from JavaScript!' }
    }
  })
})
  .then(res => res.json())
  .then(data => console.log(data));
```

### Docker Integration

When running via Docker Compose, the server is automatically configured for HTTP:

```bash
docker compose -f docker/docker-compose.yml up --build
```

The server will be available at `http://localhost:8080/rpc` (or your configured port).

---

## Running via Docker

```bash
docker compose -f docker/docker-compose.yml up --build
```

The compose file builds the release binary, mounts a persistent sled database
volume, and exposes port `8080` (adjustable via environment variables). For
single-container deployments you can also build directly:

```bash
docker build -t inferenco-mcp -f docker/Dockerfile .
docker run --rm -it -p 8080:8080 inferenco-mcp
```

---

## Development

Run the standard checks:

```bash
cargo fmt
cargo clippy --all-targets --all-features
cargo test
cargo check --examples
```

Two helper scripts exist:

- `scripts/build.sh` – build the release binary and print helpful info
- `scripts/test.sh` – run a lightweight tools/list + tools/call smoke test

---

## Extending the Server

Adding a new tool is straightforward:

1. Define the request struct in `src/server/dto.rs`
2. Add an async method to `ToolService` and decorate it with `#[tool(...)]`
3. Update `ToolService::available_tools()` output as needed (rmcp handles this)
4. Rebuild the server; the new tool is advertised automatically

See `src/server/implementation.rs` for working examples.

---

## Troubleshooting

- **No logs?** Ensure `RUST_LOG` or `INFERENCO_MCP_LOG_LEVEL` is set to a visible
  level (e.g., `debug`).
- **HTTP requests rejected?** Confirm `INFERENCO_MCP_TRANSPORT=http`, the port is
  exposed, and—if auth is enabled—you include the configured header and API key.
- **Example client errors?** The example calls the tools directly; make sure you
  rebuilt the crate after editing `ToolService`.

For anything else, file an issue on the repository—this server is purposely
small so issues are usually easy to diagnose.
