# Inferenco MCP Server

A tiny-but-complete Model Context Protocol (MCP) server powered by the official
[rmcp](https://github.com/modelcontextprotocol/rust-sdk) crate. Inferenco MCP
focuses on being the simplest possible reference implementation: it exposes two
tools (`echo` and `increment`), runs happily over stdio or HTTP, and ships with
ready-to-run Docker and shell scripts.

---

## Feature Highlights

- :sparkles: **Two tools out of the box** – an echo and a stateful counter
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

The example lists every tool and invokes both `echo` and `increment` using the
public `ToolService` API.

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
