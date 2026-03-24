# CLAUDE.md — Agent & LLM Context for wpx

## Project Overview

wpx is a Rust-native WordPress CLI designed for both AI agents and humans. It manages WordPress sites remotely via the REST API, producing structured machine-readable output (JSON) when piped and human-readable tables when run interactively. The tool is organized as a Cargo workspace of seven crates, supports multi-site fleet management, and exposes an MCP (Model Context Protocol) server for direct AI agent integration. Authentication supports WordPress application passwords and OAuth 2.1 with PKCE.

## Build & Test

```bash
# Build the entire workspace
cargo build

# Build release binary
cargo build --release

# Run all tests across the workspace
cargo test

# Run tests for a specific crate
cargo test -p wpx-core
cargo test -p wpx-cli
cargo test -p wpx-mcp

# Run the CLI
cargo run -- post list --site default
cargo run -- --help

# Run with verbose logging (debug to stderr)
cargo run -- --verbose post list

# Generate shell completions
cargo run -- completions bash > wpx.bash
```

## Architecture & Code Organization

```
wordpress-cli/
├── Cargo.toml                          # Workspace root: 7 member crates, shared deps
├── CLAUDE.md                           # This file — agent/LLM context
│
├── crates/
│   ├── wpx-core/                       # Domain types, Resource trait, error types
│   │   └── src/
│   │       ├── lib.rs                  # Re-exports: Resource, WpxError, ExitCode
│   │       ├── error.rs               # WpxError enum, ExitCode enum (0-10), JSON error output
│   │       ├── resource.rs            # Resource trait definition (the core abstraction)
│   │       └── resources/
│   │           ├── mod.rs             # Module declarations for all resource types
│   │           ├── post.rs            # Post struct, PostCreateParams, impl Resource
│   │           ├── page.rs            # Page struct, impl Resource
│   │           ├── media.rs           # Media struct, impl Resource
│   │           ├── user.rs            # User struct, impl Resource
│   │           ├── comment.rs         # Comment struct, impl Resource
│   │           ├── category.rs        # Category struct, impl Resource
│   │           ├── tag.rs             # Tag struct, impl Resource
│   │           ├── taxonomy.rs        # Taxonomy struct, impl Resource
│   │           ├── plugin.rs          # Plugin struct, impl Resource
│   │           ├── theme.rs           # Theme struct, impl Resource
│   │           ├── post_type.rs       # PostType struct, impl Resource
│   │           ├── post_status.rs     # PostStatus struct, impl Resource
│   │           ├── block.rs           # Block (reusable block) struct, impl Resource
│   │           ├── block_type.rs      # BlockType struct, impl Resource
│   │           ├── block_pattern.rs   # BlockPattern struct, impl Resource
│   │           ├── block_pattern_category.rs  # BlockPatternCategory struct, impl Resource
│   │           ├── widget.rs          # Widget struct, impl Resource
│   │           ├── widget_type.rs     # WidgetType struct, impl Resource
│   │           ├── sidebar.rs         # Sidebar struct, impl Resource
│   │           ├── menu.rs            # Menu struct, impl Resource
│   │           ├── menu_item.rs       # MenuItem struct, impl Resource
│   │           ├── menu_location.rs   # MenuLocation struct, impl Resource
│   │           └── search_result.rs   # SearchResult struct, impl Resource
│   │
│   ├── wpx-api/                        # HTTP client layer wrapping reqwest
│   │   └── src/
│   │       ├── lib.rs                 # Re-exports: WpClient, ApiResponse
│   │       ├── client.rs             # WpClient: GET/POST/PUT/DELETE, retry logic, backoff,
│   │       │                          #   multipart upload, discover(), bridge_call()
│   │       ├── response.rs           # ApiResponse<T> with total/total_pages from WP headers
│   │       └── error.rs              # Maps reqwest/HTTP errors to WpxError variants
│   │
│   ├── wpx-auth/                       # Authentication providers
│   │   └── src/
│   │       ├── lib.rs                 # Re-exports: AuthProvider, NoAuth, ApplicationPasswordAuth, OAuthAuth
│   │       ├── provider.rs           # AuthProvider trait, NoAuth impl
│   │       ├── basic.rs              # ApplicationPasswordAuth (HTTP Basic with app passwords)
│   │       └── oauth.rs              # OAuthAuth (Bearer token), PkceChallenge, run_oauth_flow(),
│   │                                  #   refresh_token(), local callback server
│   │
│   ├── wpx-config/                     # Configuration and credential management
│   │   └── src/
│   │       ├── lib.rs                 # Re-exports: WpxConfig, CredentialStore, SiteProfile
│   │       ├── config.rs             # WpxConfig: TOML loading, site profiles, precedence:
│   │       │                          #   project (.wpx.toml) > user (~/.config/wpx/config.toml) > defaults
│   │       ├── profile.rs            # SiteProfile struct: url, auth method, username
│   │       └── credentials.rs        # CredentialStore: ~/.config/wpx/credentials.toml,
│   │                                  #   SiteCredentials with app-password and OAuth fields
│   │
│   ├── wpx-output/                     # Output formatting and rendering
│   │   └── src/
│   │       ├── lib.rs                 # Re-exports: OutputFormat, RenderPayload, render, render_with_config
│   │       ├── format.rs             # OutputFormat enum: Auto, Json, Table, Csv, Yaml, Ndjson
│   │       │                          #   Auto resolves to Table (TTY) or Json (piped)
│   │       ├── render.rs             # RenderPayload struct, render functions for each format,
│   │       │                          #   OutputConfig with field mask support
│   │       └── fields.rs             # apply_field_mask(): filters JSON output to requested fields
│   │
│   ├── wpx-mcp/                        # MCP (Model Context Protocol) server
│   │   └── src/
│   │       ├── lib.rs                 # Re-exports: serve_stdio
│   │       ├── server.rs             # JSON-RPC 2.0 over stdio: initialize, tools/list, tools/call,
│   │       │                          #   resources/list, resources/read; builds WpClient per request
│   │       └── tools.rs              # ToolDef struct, generate_tools() -> 35+ tool definitions,
│   │                                  #   tool_name_to_command_path() for MCP-to-CLI mapping
│   │
│   └── wpx-cli/                        # CLI binary, command routing, CRUD helpers
│       └── src/
│           ├── main.rs               # Entry point: parse CLI, run command, render output, handle errors
│           ├── cli.rs                # Clap derive structs: Cli, GlobalFlags, Commands enum (26 subcommands),
│           │                          #   McpCommands, AuthCommands
│           ├── context.rs            # build_client(): resolves site profile + credentials -> WpClient
│           ├── crud.rs               # Generic CRUD helpers: list, get, create, update, delete,
│           │                          #   list_all_pages (streaming NDJSON), list_object_keyed,
│           │                          #   get_by_slug, to_query_params, object_values_to_array
│           ├── dispatch.rs           # Unified dispatcher: dispatch(command_path, args, client, dry_run)
│           │                          #   used by CLI, MCP server, and fleet exec
│           └── commands/
│               ├── mod.rs            # Module declarations
│               ├── post.rs           # PostCommands: list, get, create, update, delete, search
│               ├── page.rs           # PageCommands: list, get, create, update, delete
│               ├── media.rs          # MediaCommands: list, get, upload, delete
│               ├── user.rs           # UserCommands: list, get, me
│               ├── comment.rs        # CommentCommands: list, get, create, update, delete
│               ├── category.rs       # CategoryCommands: list, get, create, update, delete
│               ├── tag.rs            # TagCommands: list, get, create, update, delete
│               ├── taxonomy.rs       # TaxonomyCommands: list, get
│               ├── plugin.rs         # PluginCommands: list, install, activate, deactivate, delete
│               ├── theme.rs          # ThemeCommands: list, activate
│               ├── post_type.rs      # PostTypeCommands: list, get
│               ├── post_status.rs    # PostStatusCommands: list, get
│               ├── block.rs          # BlockCommands: list, get, create, update, delete
│               ├── block_type.rs     # BlockTypeCommands: list, get
│               ├── block_pattern.rs  # BlockPatternCommands: list
│               ├── block_pattern_category.rs  # BlockPatternCategoryCommands: list
│               ├── widget.rs         # WidgetCommands: list, get, create, update, delete
│               ├── widget_type.rs    # WidgetTypeCommands: list, get
│               ├── sidebar.rs        # SidebarCommands: list, get
│               ├── menu.rs           # MenuCommands: list, get, create, update, delete
│               ├── menu_item.rs      # MenuItemCommands: list, get, create, update, delete
│               ├── menu_location.rs  # MenuLocationCommands: list, get
│               ├── search.rs         # Global search: SearchArgs, handle()
│               ├── settings.rs       # SettingsCommands: list, get, set (aliased as "option")
│               ├── fleet.rs          # FleetCommands: exec (concurrent multi-site), status
│               ├── discover.rs       # Site capability discovery (REST API, wpx-bridge, WooCommerce)
│               ├── schema.rs         # JSON Schema introspection for command inputs/outputs
│               └── auth.rs           # AuthCommands: set, test, list, logout, oauth
```

## Key Patterns

### Resource Trait

The core abstraction. Every WordPress entity implements this trait, which unlocks generic CRUD operations:

```rust
pub trait Resource:
    serde::Serialize + serde::de::DeserializeOwned + Send + Sync + std::fmt::Debug + 'static
{
    /// Display name (e.g., "post", "page").
    const NAME: &'static str;
    /// Plural form (e.g., "posts", "pages").
    const NAME_PLURAL: &'static str;
    /// REST API path (e.g., "wp/v2/posts").
    const API_PATH: &'static str;
    /// Default columns for table output.
    const DEFAULT_TABLE_FIELDS: &'static [&'static str];
}
```

Example implementation (`crates/wpx-core/src/resources/post.rs`):

```rust
impl Resource for Post {
    const NAME: &'static str = "post";
    const NAME_PLURAL: &'static str = "posts";
    const API_PATH: &'static str = "wp/v2/posts";
    const DEFAULT_TABLE_FIELDS: &'static [&'static str] =
        &["id", "title", "status", "date", "author"];
}
```

### Adding a New Resource (Step-by-Step)

1. **Define the resource struct** in `crates/wpx-core/src/resources/<name>.rs`:
   - Add a struct with `#[derive(Debug, Clone, Serialize, Deserialize)]`
   - Implement the `Resource` trait with `NAME`, `NAME_PLURAL`, `API_PATH`, `DEFAULT_TABLE_FIELDS`
   - Add `CreateParams` / `UpdateParams` structs if the resource supports writes
   - Register the module in `crates/wpx-core/src/resources/mod.rs`

2. **Add CLI commands** in `crates/wpx-cli/src/commands/<name>.rs`:
   - Define a `<Name>Commands` enum with subcommands (List, Get, Create, etc.)
   - Define args structs with `#[derive(Debug, Args, Serialize)]` (Serialize enables query param conversion)
   - Write a `handle()` function that delegates to the generic CRUD helpers
   - Register the module in `crates/wpx-cli/src/commands/mod.rs`

3. **Add the top-level command** in `crates/wpx-cli/src/cli.rs`:
   - Add a variant to the `Commands` enum with `#[command(subcommand)]`

4. **Wire up in main.rs** (`crates/wpx-cli/src/main.rs`):
   - Add a match arm in `run()` that calls `commands::<name>::handle()`

5. **Add dispatch entries** in `crates/wpx-cli/src/dispatch.rs`:
   - Add match arms for each subcommand (e.g., `["<name>", "list"]`, `["<name>", "get"]`)
   - Use the generic CRUD functions: `crud::list::<Name>()`, `crud::get::<Name>()`, etc.

6. **Add MCP tool definitions** in `crates/wpx-mcp/src/tools.rs`:
   - Add entries to the `entries` vec in `generate_tools()` with JSON Schema
   - Tool name convention: `"wpx_<resource>_<action>"` (e.g., `"wpx_post_list"`)

7. **Add MCP dispatch** in `crates/wpx-mcp/src/server.rs`:
   - Add match arms in `dispatch_tool()` for the new commands

### CRUD Helpers

Located in `crates/wpx-cli/src/crud.rs`. All are generic over `R: Resource`:

| Helper | Signature | Notes |
|--------|-----------|-------|
| `list<R>` | `(client, params) -> RenderPayload` | Converts params to query string via `to_query_params()` |
| `list_all_pages<R>` | `(client, params) -> RenderPayload` | Streams all pages as NDJSON to stdout (100/page) |
| `list_object_keyed<R>` | `(client, api_path) -> RenderPayload` | For endpoints returning `{slug: {...}}` instead of arrays |
| `get<R>` | `(client, id) -> RenderPayload` | GET `{API_PATH}/{id}` |
| `get_by_slug<R>` | `(client, api_path, slug) -> RenderPayload` | GET `{api_path}/{slug}` |
| `create<R>` | `(client, body, dry_run) -> RenderPayload` | POST to `API_PATH`; dry_run returns what would be created |
| `update<R>` | `(client, id, body, dry_run) -> RenderPayload` | POST to `{API_PATH}/{id}` |
| `delete<R>` | `(client, id, force, dry_run) -> RenderPayload` | DELETE; force=true permanently deletes, false trashes |

The `to_query_params()` helper serializes any `Serialize` struct to `Vec<(String, String)>`, skipping `None` values. This is why list args structs derive both `Args` (for clap) and `Serialize` (for query params).

### API Response Format

The WordPress REST API returns pagination info in headers. `ApiResponse<T>` captures this:

```rust
pub struct ApiResponse<T> {
    pub data: T,
    pub total: Option<u64>,       // From X-WP-Total header
    pub total_pages: Option<u64>, // From X-WP-TotalPages header
}
```

### Unified Dispatcher

`crates/wpx-cli/src/dispatch.rs` provides a single entry point used by three contexts:

1. **CLI** (`main.rs`) -- Clap-parsed commands delegate here for some routes
2. **MCP server** (`wpx-mcp/src/server.rs`) -- Tool calls are converted to command paths
3. **Fleet exec** (`commands/fleet.rs`) -- Multi-site commands dispatch through here

Signature: `dispatch(command_path: &[&str], args: &Value, client: &WpClient, dry_run: bool) -> Result<RenderPayload, WpxError>`

Command paths are string slices like `["post", "list"]`, `["plugin", "activate"]`, `["search"]`.

## Configuration

### Config File Precedence

1. **Project-level**: `./.wpx.toml` (highest priority)
2. **User-level**: `~/.config/wpx/config.toml`
3. **Defaults**: Built-in defaults (lowest priority)

### Config File Format (`~/.config/wpx/config.toml`)

```toml
[default]
output = "json"       # auto | json | table | csv | yaml | ndjson
color = "auto"        # auto | always | never
timeout = 60          # Request timeout in seconds (default: 30)
retries = 5           # Retry count for failed requests (default: 3)

[sites.production]
url = "https://example.com"
auth = "application-password"   # Default auth method
username = "admin"

[sites.staging]
url = "https://staging.example.com"
```

### Credentials File (`~/.config/wpx/credentials.toml`)

Stored with `0600` permissions on Unix. Separate from config for security.

```toml
[sites.production]
auth_type = "application-password"
username = "admin"
password = "xxxx xxxx xxxx xxxx"

[sites.staging]
auth_type = "oauth2"
access_token = "eyJ..."
refresh_token = "eyJ..."
token_expiry = "2026-04-01T00:00:00Z"
client_id = "wp-client-id"
authorize_url = "https://staging.example.com/oauth/authorize"
token_url = "https://staging.example.com/oauth/token"
```

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `WPX_SITE` | Target site profile name | `default` |
| `WPX_URL` | Direct URL override (skips profile lookup) | — |
| `WPX_OUTPUT` | Output format: json, table, csv, yaml, ndjson, auto | `auto` |
| `WPX_TIMEOUT` | Request timeout in seconds | `30` |
| `WPX_RETRIES` | Retry count for failed requests | `3` |
| `WPX_NO_PROMPT` | Disable interactive prompts | — |
| `NO_COLOR` | Disable colored output | — |

### Global CLI Flags

All flags are available on every command via `--flag`:

`--site`, `--url`, `--output`, `--fields` (comma-separated field mask), `--no-color`, `--no-prompt`, `--quiet`, `--verbose`, `--timeout`, `--retries`, `--dry-run`, `--confirm`, `--all-pages`

## Error Handling

### WpxError Variants

All errors in the application funnel through a single `WpxError` enum defined in `crates/wpx-core/src/error.rs`:

| Variant | Description |
|---------|-------------|
| `Api { code, message, status, suggestion }` | WordPress REST API error response |
| `Auth { message }` | Authentication failure |
| `Config { message }` | Configuration error |
| `Network(String)` | Network/connection error |
| `Io(std::io::Error)` | File system I/O error |
| `Validation { field, message }` | Input validation error |
| `NotFound { resource, id }` | Resource not found |
| `PermissionDenied { message }` | Insufficient permissions |
| `RateLimited { retry_after_secs }` | Rate limit exceeded |
| `Server { status, message }` | 5xx server error |
| `Conflict { message }` | 409 conflict |
| `Other(String)` | Catch-all |

### Exit Codes

Semantic exit codes for agent-friendly error handling. Agents can programmatically decide retry/abort/fix strategies based on these codes:

| Code | Name | Description |
|------|------|-------------|
| 0 | `Success` | Success |
| 1 | `GeneralError` | General error |
| 2 | `InvalidArgs` | Invalid arguments |
| 3 | `AuthFailure` | Authentication failure |
| 4 | `NotFound` | Resource not found |
| 5 | `PermissionDenied` | Permission denied |
| 6 | `RateLimited` | Rate limited |
| 7 | `NetworkError` | Network error |
| 8 | `ServerError` | Server error (5xx) |
| 9 | `Conflict` | Conflict (409) |
| 10 | `ValidationError` | Validation error |

### Structured Error Output

Errors are written to stderr as JSON:

```json
{
  "error": true,
  "message": "Resource not found: post 99",
  "exit_code": 4,
  "resource": "post",
  "id": "99",
  "suggestion": "Use 'wpx post list' to find valid IDs"
}
```

### Retry Logic

The `WpClient` retries on transient failures (429 Too Many Requests, 5xx Server Error, timeouts, connection errors) with exponential backoff (1s, 2s, 4s, 8s...). Respects the `Retry-After` header when present. Default: 3 retries.

## MCP Server

### Starting the Server

```bash
wpx mcp serve                         # stdio transport (default)
wpx mcp serve --transport stdio        # explicit stdio
wpx --site production mcp serve        # target a specific site
```

### Protocol

JSON-RPC 2.0 over stdin/stdout (one JSON object per line). Supported methods:

- `initialize` -- Returns server capabilities and version
- `tools/list` -- Returns 35+ tool definitions with JSON Schema
- `tools/call` -- Dispatches to wpx command handlers
- `resources/list` -- Returns available MCP resources (`wpx://sites`, `wpx://info`)
- `resources/read` -- Reads an MCP resource
- `ping` -- Health check

### Tool Naming Convention

CLI command paths are converted to MCP tool names:

```
CLI:  wpx post list          -> MCP tool: wpx_post_list
CLI:  wpx plugin install     -> MCP tool: wpx_plugin_install
CLI:  wpx post-type list     -> MCP tool: wpx_post_type_list
CLI:  wpx menu-item list     -> MCP tool: wpx_menu_item_list
CLI:  wpx search             -> MCP tool: wpx_search
CLI:  wpx discover           -> MCP tool: wpx_discover
```

Rule: `wpx_` prefix + command path with spaces and hyphens replaced by underscores.

Reverse mapping handles special cases (`post_type` -> `post-type`, `menu_item` -> `menu-item`) in `tool_name_to_command_path()`.

### MCP Tool Dispatch Flow

1. MCP client sends `tools/call` with `name: "wpx_post_list"` and `arguments: {...}`
2. `server.rs` extracts tool name and arguments
3. `tools::tool_name_to_command_path("wpx_post_list")` returns `["post", "list"]`
4. `dispatch_tool()` in `server.rs` matches the command path and calls the appropriate `WpClient` method
5. Result is returned as JSON-RPC response with `content: [{type: "text", text: "..."}]`

### MCP Resources

| URI | Description |
|-----|-------------|
| `wpx://sites` | List of configured site profiles (name, URL, auth method) |
| `wpx://info` | wpx version and configured site names |

## Dependencies (Key)

| Crate | Version | Purpose |
|-------|---------|---------|
| `clap` | 4.5 | CLI argument parsing (derive mode) |
| `reqwest` | 0.12 | HTTP client (rustls-tls, JSON, multipart) |
| `tokio` | 1 | Async runtime (full features) |
| `serde` / `serde_json` | 1.0 | Serialization/deserialization |
| `serde_yaml` | 0.9 | YAML output format |
| `toml` | 0.8 | Config file parsing |
| `csv` | 1.3 | CSV output format |
| `tabled` | 0.16 | Table rendering |
| `indicatif` | 0.17 | Progress indicators |
| `thiserror` | 2 | Error derive macros |
| `anyhow` | 1.0 | Error context |
| `tracing` | 0.1 | Structured logging |
| `keyring` | 3 | OS keyring integration (apple-native, linux-native) |
| `url` | 2 | URL parsing |
| `open` | 5 | Open browser for OAuth |
| `sha2` / `base64` / `rand` | — | PKCE challenge generation |
| `wiremock` | 0.6 | HTTP mocking (dev) |
| `assert_cmd` | 2.0 | CLI integration tests (dev) |
| `predicates` | 3.0 | Test assertions (dev) |

## Testing

### Test Organization

- **Unit tests**: Inline `#[cfg(test)] mod tests` in most source files
- **Integration tests**: Via `assert_cmd` for CLI binary testing
- **HTTP mocking**: Via `wiremock` for API client tests

### Running Tests

```bash
cargo test                             # All workspace tests
cargo test -p wpx-core                 # Core types and resource tests
cargo test -p wpx-cli                  # CLI and CRUD helper tests
cargo test -p wpx-mcp                  # MCP server and tool tests
cargo test -p wpx-api                  # HTTP client tests
cargo test -p wpx-auth                 # Auth provider tests
cargo test -p wpx-config               # Config parsing tests
cargo test -p wpx-output               # Output rendering tests
```

### Key Test Patterns

- Resource structs have deserialization tests with real WordPress JSON payloads
- CRUD helpers test query param generation and object-to-array conversion
- The dispatcher tests ID/string extraction from JSON args
- MCP tests verify tool count (>30), tool naming, and response structure
- Config tests verify TOML parsing and merge precedence
- Error tests verify exit code mapping and structured JSON output

### Useful Test Commands

```bash
# Run a specific test by name
cargo test -p wpx-core deserialize_post

# Run tests with output visible
cargo test -- --nocapture

# Check compilation without running tests
cargo check --workspace
```
