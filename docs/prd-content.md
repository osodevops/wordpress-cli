# wpx — A Rust-Native WordPress CLI for Agents and Humans

## Product Requirements Document (PRD)

**Version:** 1.0.0-draft  
**Author:** [Your Name]  
**Date:** March 2026  
**Status:** Draft  
**Target Audience:** Coding agents (Claude Code, Cursor, Codex, etc.)

---

## 1. Executive Summary

**wpx** is a Rust-native command-line interface for managing WordPress sites remotely via the WordPress REST API (v2), the Abilities API (v1), and the WooCommerce REST API (v3). Unlike the existing PHP-based WP-CLI — which requires local PHP execution and a bootstrapped WordPress installation — wpx operates as a standalone, zero-dependency binary that communicates over HTTPS. It is designed as an agent-first CLI: every command produces structured, machine-readable output by default, supports schema introspection, and exposes itself via MCP (Model Context Protocol) for seamless AI agent integration.

### Why This Exists

The existing WP-CLI (github.com/wp-cli/wp-cli) is the de facto standard for WordPress command-line management. However, it has fundamental architectural limitations that make it poorly suited for the emerging agent-driven automation era:

1. **PHP Dependency** — Requires a local PHP runtime (7.2.24+) and a bootstrapped WordPress installation. Every command spawns a full PHP/WordPress process, adding 150–350ms overhead per invocation on barebones installs, and significantly more with plugins loaded.

2. **Local-Only Execution** — Cannot manage remote WordPress sites without SSH access. No native REST API integration.

3. **Human-Only Output** — Output is designed for human eyes: ASCII tables, colorized text, interactive prompts. Machine-readable output (--format=json) is inconsistently available and not the default. The `wp db query` command, for example, has no JSON output at all.

4. **No Agent Affordances** — No schema introspection, no semantic exit codes beyond 0/1, no TTY detection for automatic output switching, no NDJSON streaming, no MCP server capability.

5. **No Native Concurrency** — PHP's execution model limits parallel operations. Bulk operations on large sites (media regeneration, search-replace across thousands of posts) are single-threaded.

6. **Missing Modern WordPress APIs** — No integration with the Abilities API (WordPress 6.9+), no native MCP adapter support, no WooCommerce REST API coverage.

wpx solves all of these by being a fast, portable Rust binary that treats every WordPress site as a remote API endpoint.

---

## 2. Goals

| # | Goal | Success Metric |
|---|------|----------------|
| G1 | Full feature parity with WP-CLI | 100% coverage of all WP-CLI command groups via REST API equivalents |
| G2 | Agent-first design | Every command supports --json output, schema introspection, semantic exit codes, and MCP exposure |
| G3 | Zero-dependency binary | Single static binary, no PHP/WordPress/runtime dependencies required |
| G4 | Remote-first architecture | Manage any WordPress site over HTTPS without SSH access |
| G5 | Sub-100ms cold start | CLI startup in <100ms (vs WP-CLI's 150-350ms+ per command) |
| G6 | Concurrent operations | Parallel bulk operations (media regeneration, batch updates, multi-site management) |
| G7 | MCP server mode | Built-in MCP server exposing all CLI capabilities to AI agents |

---

## 3. Non-Goals

| # | Non-Goal | Reason |
|---|----------|--------|
| NG1 | Replace WP-CLI's PHP code generation (scaffold) | Code generation (scaffolding plugins/themes/post-types) requires PHP templates and a local WordPress filesystem. Out of scope for v1 — consider as a plugin/extension later. |
| NG2 | Local PHP execution mode | wpx is remote-first. It does not bootstrap WordPress locally. Users needing local execution should continue using WP-CLI. |
| NG3 | WordPress installation/setup | `wp core download`, `wp core install`, `wp config create` require filesystem access. Out of scope. |
| NG4 | PHP eval/shell commands | `wp eval`, `wp eval-file`, `wp shell` execute arbitrary PHP, which requires a local PHP process. Not applicable to a REST API client. |
| NG5 | GUI or TUI dashboard | This is a CLI tool. No curses/TUI interfaces in v1. |
| NG6 | Backward compatibility with WP-CLI flag syntax | wpx uses its own idiomatic command structure. A compatibility shim may come later. |

---

## 4. Target Users

### 4.1 Primary: AI Agents
- Claude Code, Cursor, OpenAI Codex, GitHub Copilot CLI
- Automated CI/CD pipelines managing WordPress deployments
- Custom automation agents built with LangChain, CrewAI, or similar frameworks
- MCP-enabled AI clients (Claude Desktop, VS Code MCP extensions)

### 4.2 Secondary: DevOps Engineers & WordPress Developers
- Managing fleets of WordPress sites remotely
- Running bulk operations across multiple sites
- Integrating WordPress management into existing CLI/scripting workflows
- Managing headless WordPress installations

---

## 5. Architecture

### 5.1 High-Level Architecture

```
┌──────────────────────────────────────────────────────────┐
│                        wpx binary                        │
│                                                          │
│  ┌─────────┐  ┌──────────┐  ┌─────────┐  ┌───────────┐ │
│  │   CLI   │  │   MCP    │  │  Config  │  │  Output   │ │
│  │ Parser  │  │  Server  │  │  Mgr     │  │  Renderer │ │
│  │ (clap)  │  │ (stdio/  │  │          │  │ (json/    │ │
│  │         │  │  sse)    │  │          │  │  table/   │ │
│  └────┬────┘  └────┬─────┘  └────┬─────┘  │  yaml/   │ │
│       │            │             │         │  csv)    │ │
│       └────────┬───┘─────────────┘         └────┬─────┘ │
│                │                                │       │
│  ┌─────────────▼────────────────────────────────▼─────┐ │
│  │              Command Router / Executor              │ │
│  └─────────────────────────┬──────────────────────────┘ │
│                            │                            │
│  ┌─────────────────────────▼──────────────────────────┐ │
│  │                  API Client Layer                   │ │
│  │  ┌──────────┐  ┌───────────┐  ┌─────────────────┐ │ │
│  │  │  WP REST  │  │ Abilities │  │   WooCommerce   │ │ │
│  │  │  API v2   │  │  API v1   │  │    REST v3      │ │ │
│  │  └──────────┘  └───────────┘  └─────────────────┘ │ │
│  └─────────────────────────┬──────────────────────────┘ │
│                            │                            │
│  ┌─────────────────────────▼──────────────────────────┐ │
│  │              HTTP Client (reqwest)                  │ │
│  │  Connection pooling · Retry logic · Rate limiting  │ │
│  │  TLS · Compression · Timeout management            │ │
│  └────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────┘
                            │
                     HTTPS (TLS 1.2+)
                            │
                  ┌─────────▼─────────┐
                  │  WordPress Site(s) │
                  │  REST API / Auth   │
                  └───────────────────┘
```

### 5.2 Core Crates / Modules

| Crate | Purpose |
|-------|---------|
| `wpx-cli` | Binary entrypoint, clap argument parsing, TTY detection, output rendering |
| `wpx-core` | Command definitions, business logic, validation |
| `wpx-api` | HTTP client, WordPress REST API v2 bindings, error mapping |
| `wpx-abilities` | WordPress Abilities API v1 client |
| `wpx-woo` | WooCommerce REST API v3 client |
| `wpx-auth` | Authentication providers (Application Passwords, OAuth 2.1, JWT) |
| `wpx-config` | Configuration file management, site profiles, credential storage |
| `wpx-mcp` | MCP server implementation (stdio and SSE transports) |
| `wpx-output` | Output formatters (JSON, NDJSON, Table, CSV, YAML), field masks |

### 5.3 Key Dependencies (Rust Ecosystem)

| Dependency | Purpose |
|------------|---------|
| `clap` (derive) | CLI argument parsing with derive macros |
| `reqwest` | Async HTTP client with connection pooling |
| `tokio` | Async runtime for concurrent operations |
| `serde` / `serde_json` | Serialization/deserialization of API payloads |
| `tabled` | Human-readable table output |
| `indicatif` | Progress bars for long-running operations |
| `keyring` | OS-native credential storage |
| `toml` | Configuration file parsing |
| `jsonschema` | Schema validation for MCP and API contracts |
| `tower` | Middleware for retry, rate-limiting, timeout |
| `tracing` | Structured logging and diagnostics |

---

## 6. Authentication

### 6.1 Supported Methods

| Method | Priority | Use Case |
|--------|----------|----------|
| Application Passwords | P0 | Default. Built into WordPress 5.6+. Basic Auth over HTTPS. |
| OAuth 2.1 | P0 | WordPress.com hosted sites, enterprise SSO |
| JWT (via plugin) | P1 | Sites using JWT Authentication plugins |
| Custom Headers | P2 | Sites using custom auth middleware |

### 6.2 Credential Storage

```
~/.config/wpx/config.toml        # Site profiles & preferences
~/.config/wpx/credentials.toml   # Encrypted credentials (or OS keyring)
```

**Site Profile Example (config.toml):**

```toml
[default]
output = "json"          # Default output format
color = "auto"           # auto | always | never
timeout = 30             # Request timeout in seconds
retries = 3              # Auto-retry count

[sites.production]
url = "https://example.com"
auth = "application-password"
username = "admin"
# Password stored in OS keyring or credentials.toml

[sites.staging]
url = "https://staging.example.com"
auth = "oauth2"
client_id = "..."
# Tokens managed automatically

[sites.woocommerce-store]
url = "https://shop.example.com"
auth = "woo-api-key"
consumer_key = "ck_..."
# Consumer secret stored in OS keyring
```

### 6.3 Auth Commands

```bash
# Interactive setup — generates application password via browser flow
wpx auth login --site production

# Manual credential setup
wpx auth set --site production --username admin --password "xxxx xxxx xxxx xxxx"

# Test authentication
wpx auth test --site production

# List configured sites
wpx auth list

# Remove credentials
wpx auth logout --site production

# Rotate application password
wpx auth rotate --site production
```

---

## 7. Command Reference — Full Feature Parity Matrix

### 7.1 Command Structure

wpx uses a **noun-verb** (resource-action) structure for agent discoverability:

```
wpx <resource> <action> [arguments] [flags]
```

Examples:
```bash
wpx post list --site production --status publish --per-page 50
wpx plugin install woocommerce --site production --activate
wpx user create --site production --username newuser --email user@example.com --role editor
```

### 7.2 Global Flags (Available on Every Command)

| Flag | Env Var | Default | Description |
|------|---------|---------|-------------|
| `--site <name>` | `WPX_SITE` | `default` | Target site profile name |
| `--url <url>` | `WPX_URL` | — | Direct URL override (skips profile) |
| `--output <fmt>` | `WPX_OUTPUT` | `auto` | Output format: `json`, `table`, `csv`, `yaml`, `ndjson`, `auto` |
| `--fields <list>` | — | all | Comma-separated field mask (reduces output) |
| `--no-color` | `NO_COLOR` | — | Disable colored output |
| `--no-prompt` | `WPX_NO_PROMPT` | — | Disable all interactive prompts |
| `--quiet` | — | — | Suppress non-essential output |
| `--verbose` | — | — | Enable debug logging to stderr |
| `--timeout <secs>` | `WPX_TIMEOUT` | 30 | Request timeout |
| `--retries <n>` | `WPX_RETRIES` | 3 | Retry count for failed requests |
| `--dry-run` | — | — | Show what would be done without executing |
| `--confirm` | — | — | Skip confirmation prompts (for destructive actions) |

**TTY Detection:**
- When stdout is a TTY → `auto` resolves to `table` (human-readable)
- When stdout is piped → `auto` resolves to `json` (machine-readable)

### 7.3 Complete Command Groups

Below is the full command matrix mapping WP-CLI commands to wpx equivalents, grouped by resource.

---

#### 7.3.1 Posts

**WP-CLI equivalent:** `wp post`  
**REST API:** `/wp/v2/posts`, `/wp/v2/pages`

| Command | Description | WP-CLI Equivalent |
|---------|-------------|-------------------|
| `wpx post list` | List posts with filters (status, type, author, date range, search) | `wp post list` |
| `wpx post get <id>` | Get a single post by ID | `wp post get <id>` |
| `wpx post create` | Create a new post (accepts --json stdin for full payload) | `wp post create` |
| `wpx post update <id>` | Update an existing post | `wp post update <id>` |
| `wpx post delete <id>` | Delete/trash a post | `wp post delete <id>` |
| `wpx post meta list <id>` | List post meta | `wp post meta list <id>` |
| `wpx post meta get <id> <key>` | Get specific meta value | `wp post meta get` |
| `wpx post meta set <id> <key> <value>` | Set meta value | `wp post meta update` |
| `wpx post meta delete <id> <key>` | Delete meta | `wp post meta delete` |
| `wpx post revision list <id>` | List revisions for a post | `wp post list --post_type=revision` |
| `wpx post revision get <post_id> <rev_id>` | Get specific revision | — |
| `wpx post revision delete <post_id> <rev_id>` | Delete specific revision | — |
| `wpx post revision restore <post_id> <rev_id>` | Restore a revision | — |
| `wpx post search <query>` | Full-text search across posts | `wp post list --s=<query>` |
| `wpx post bulk-update` | Batch update multiple posts (accepts NDJSON stdin) | — (new) |
| `wpx post bulk-delete` | Batch delete posts by filter | — (new) |
| `wpx post export` | Export posts to WXR/JSON format | `wp export` |

**Flags specific to `wpx post list`:**
```
--status <status>       draft | publish | pending | private | future | trash
--type <post_type>      post | page | any custom post type
--author <id>           Filter by author ID
--category <id|slug>    Filter by category
--tag <id|slug>         Filter by tag
--search <query>        Search term
--after <date>          Posts after date (ISO 8601)
--before <date>         Posts before date (ISO 8601)
--per-page <n>          Results per page (default 10, max 100)
--page <n>              Page number
--order <asc|desc>      Sort direction
--orderby <field>       date | title | id | modified | slug | relevance
```

---

#### 7.3.2 Pages

**WP-CLI equivalent:** `wp post` (with --post_type=page)  
**REST API:** `/wp/v2/pages`

| Command | Description |
|---------|-------------|
| `wpx page list` | List pages |
| `wpx page get <id>` | Get a page |
| `wpx page create` | Create a page |
| `wpx page update <id>` | Update a page |
| `wpx page delete <id>` | Delete a page |
| `wpx page revision list <id>` | List page revisions |

---

#### 7.3.3 Media

**WP-CLI equivalent:** `wp media`  
**REST API:** `/wp/v2/media`

| Command | Description | WP-CLI Equivalent |
|---------|-------------|-------------------|
| `wpx media list` | List media attachments | `wp media list` |
| `wpx media get <id>` | Get media item details | — |
| `wpx media upload <file>` | Upload a file (multipart form) | `wp media import` |
| `wpx media upload-url <url>` | Sideload media from URL | `wp media import <url>` |
| `wpx media update <id>` | Update media metadata (title, alt, caption) | — |
| `wpx media delete <id>` | Delete media item | — |
| `wpx media download <id> <path>` | Download media file locally | — (new) |
| `wpx media bulk-upload <dir>` | Upload all files from a directory concurrently | — (new) |
| `wpx media regenerate` | Trigger thumbnail regeneration (via Abilities API or plugin) | `wp media regenerate` |
| `wpx media sizes` | List registered image sizes | `wp media image-size` |

---

#### 7.3.4 Plugins

**WP-CLI equivalent:** `wp plugin`  
**REST API:** `/wp/v2/plugins`

| Command | Description | WP-CLI Equivalent |
|---------|-------------|-------------------|
| `wpx plugin list` | List all plugins with status | `wp plugin list` |
| `wpx plugin get <slug>` | Get plugin details | `wp plugin get` |
| `wpx plugin install <slug>` | Install a plugin from WordPress.org | `wp plugin install` |
| `wpx plugin activate <slug>` | Activate a plugin | `wp plugin activate` |
| `wpx plugin deactivate <slug>` | Deactivate a plugin | `wp plugin deactivate` |
| `wpx plugin delete <slug>` | Delete a plugin | `wp plugin delete` |
| `wpx plugin update <slug>` | Update a plugin to latest version | `wp plugin update` |
| `wpx plugin update --all` | Update all plugins | `wp plugin update --all` |
| `wpx plugin search <query>` | Search WordPress.org plugin directory | `wp plugin search` |
| `wpx plugin verify <slug>` | Verify plugin checksum | `wp plugin verify-checksums` |
| `wpx plugin auto-update enable <slug>` | Enable auto-updates for a plugin | `wp plugin auto-updates enable` |
| `wpx plugin auto-update disable <slug>` | Disable auto-updates | `wp plugin auto-updates disable` |
| `wpx plugin status` | Show summary of all plugin statuses | `wp plugin status` |

---

#### 7.3.5 Themes

**WP-CLI equivalent:** `wp theme`  
**REST API:** `/wp/v2/themes`

| Command | Description | WP-CLI Equivalent |
|---------|-------------|-------------------|
| `wpx theme list` | List installed themes | `wp theme list` |
| `wpx theme get <slug>` | Get theme details | `wp theme get` |
| `wpx theme install <slug>` | Install a theme | `wp theme install` |
| `wpx theme activate <slug>` | Activate a theme | `wp theme activate` |
| `wpx theme delete <slug>` | Delete a theme | `wp theme delete` |
| `wpx theme update <slug>` | Update a theme | `wp theme update` |
| `wpx theme update --all` | Update all themes | `wp theme update --all` |
| `wpx theme search <query>` | Search WordPress.org theme directory | `wp theme search` |
| `wpx theme status` | Show summary of theme statuses | `wp theme status` |

---

#### 7.3.6 Users

**WP-CLI equivalent:** `wp user`  
**REST API:** `/wp/v2/users`

| Command | Description | WP-CLI Equivalent |
|---------|-------------|-------------------|
| `wpx user list` | List users | `wp user list` |
| `wpx user get <id>` | Get user details | `wp user get` |
| `wpx user create` | Create a user | `wp user create` |
| `wpx user update <id>` | Update user | `wp user update` |
| `wpx user delete <id>` | Delete user (with --reassign) | `wp user delete` |
| `wpx user me` | Get current authenticated user | — (new) |
| `wpx user role set <id> <role>` | Set user role | `wp user set-role` |
| `wpx user role add <id> <role>` | Add role to user | `wp user add-role` |
| `wpx user role remove <id> <role>` | Remove role from user | `wp user remove-role` |
| `wpx user meta list <id>` | List user meta | `wp user meta list` |
| `wpx user meta get <id> <key>` | Get user meta | `wp user meta get` |
| `wpx user meta set <id> <key> <val>` | Set user meta | `wp user meta update` |
| `wpx user meta delete <id> <key>` | Delete user meta | `wp user meta delete` |
| `wpx user app-password list <id>` | List application passwords | — (new) |
| `wpx user app-password create <id>` | Create application password | — (new) |
| `wpx user app-password delete <id> <uuid>` | Revoke application password | — (new) |

---

#### 7.3.7 Comments

**WP-CLI equivalent:** `wp comment`  
**REST API:** `/wp/v2/comments`

| Command | Description |
|---------|-------------|
| `wpx comment list` | List comments with filters |
| `wpx comment get <id>` | Get a comment |
| `wpx comment create` | Create a comment |
| `wpx comment update <id>` | Update a comment |
| `wpx comment delete <id>` | Delete a comment |
| `wpx comment approve <id>` | Approve a pending comment |
| `wpx comment spam <id>` | Mark as spam |
| `wpx comment trash <id>` | Trash a comment |
| `wpx comment bulk-approve` | Approve all pending comments matching filter |
| `wpx comment bulk-spam` | Mark all matching comments as spam |
| `wpx comment bulk-delete` | Delete all matching comments |

---

#### 7.3.8 Taxonomies, Categories & Tags

**WP-CLI equivalent:** `wp taxonomy`, `wp term`  
**REST API:** `/wp/v2/taxonomies`, `/wp/v2/categories`, `/wp/v2/tags`

| Command | Description |
|---------|-------------|
| `wpx taxonomy list` | List registered taxonomies |
| `wpx taxonomy get <slug>` | Get taxonomy details |
| `wpx category list` | List categories |
| `wpx category get <id>` | Get category |
| `wpx category create` | Create category |
| `wpx category update <id>` | Update category |
| `wpx category delete <id>` | Delete category |
| `wpx tag list` | List tags |
| `wpx tag get <id>` | Get tag |
| `wpx tag create` | Create tag |
| `wpx tag update <id>` | Update tag |
| `wpx tag delete <id>` | Delete tag |
| `wpx term list <taxonomy>` | List terms for any taxonomy |
| `wpx term create <taxonomy>` | Create term in custom taxonomy |
| `wpx term update <taxonomy> <id>` | Update term |
| `wpx term delete <taxonomy> <id>` | Delete term |

---

#### 7.3.9 Menus & Navigation

**WP-CLI equivalent:** `wp menu`  
**REST API:** `/wp/v2/menus`, `/wp/v2/menu-items`, `/wp/v2/menu-locations`

| Command | Description |
|---------|-------------|
| `wpx menu list` | List menus |
| `wpx menu get <id>` | Get menu details |
| `wpx menu create` | Create a menu |
| `wpx menu update <id>` | Update a menu |
| `wpx menu delete <id>` | Delete a menu |
| `wpx menu-item list <menu_id>` | List items in a menu |
| `wpx menu-item create <menu_id>` | Add item to menu |
| `wpx menu-item update <item_id>` | Update menu item |
| `wpx menu-item delete <item_id>` | Remove menu item |
| `wpx menu-location list` | List registered menu locations |
| `wpx menu-location assign <location> <menu_id>` | Assign menu to location |

---

#### 7.3.10 Options / Settings

**WP-CLI equivalent:** `wp option`  
**REST API:** `/wp/v2/settings`

| Command | Description |
|---------|-------------|
| `wpx option list` | List all exposed site settings |
| `wpx option get <key>` | Get a specific setting |
| `wpx option set <key> <value>` | Update a setting |
| `wpx setting list` | Alias for option list |
| `wpx setting get <key>` | Alias for option get |
| `wpx setting set <key> <value>` | Alias for option set |

**Note:** The REST API only exposes settings registered via `register_setting()` with `show_in_rest = true`. The full `wp_options` table is not accessible remotely by default. This is a known limitation vs WP-CLI. Document which settings are available and how to extend via plugin.

---

#### 7.3.11 Blocks & Block Editor

**WP-CLI equivalent:** `wp block`  
**REST API:** `/wp/v2/blocks`, `/wp/v2/block-types`, `/wp/v2/block-renderer`, `/wp/v2/block-directory/search`, `/wp/v2/block-patterns`, `/wp/v2/block-pattern-categories`

| Command | Description |
|---------|-------------|
| `wpx block list` | List reusable blocks |
| `wpx block get <id>` | Get a reusable block |
| `wpx block create` | Create a reusable block |
| `wpx block update <id>` | Update a reusable block |
| `wpx block delete <id>` | Delete a reusable block |
| `wpx block-type list` | List registered block types |
| `wpx block-type get <name>` | Get block type details |
| `wpx block render <name>` | Render a block server-side |
| `wpx block search <query>` | Search block directory |
| `wpx block-pattern list` | List block patterns |
| `wpx block-pattern-category list` | List block pattern categories |

---

#### 7.3.12 Widgets & Sidebars

**WP-CLI equivalent:** `wp widget`, `wp sidebar`  
**REST API:** `/wp/v2/widgets`, `/wp/v2/sidebars`, `/wp/v2/widget-types`

| Command | Description |
|---------|-------------|
| `wpx widget list` | List active widgets |
| `wpx widget get <id>` | Get widget details |
| `wpx widget create` | Add a widget |
| `wpx widget update <id>` | Update a widget |
| `wpx widget delete <id>` | Remove a widget |
| `wpx widget-type list` | List available widget types |
| `wpx sidebar list` | List registered sidebars |
| `wpx sidebar get <id>` | Get sidebar with its widgets |

---

#### 7.3.13 Search

**REST API:** `/wp/v2/search`

| Command | Description |
|---------|-------------|
| `wpx search <query>` | Global search across posts, pages, terms, etc. |

**Flags:**
```
--type <type>       post | term | post-format
--subtype <type>    post | page | category | tag | custom
--per-page <n>      Results per page
```

---

#### 7.3.14 Post Types & Statuses

**WP-CLI equivalent:** `wp post-type`, `wp post-status`  
**REST API:** `/wp/v2/types`, `/wp/v2/statuses`

| Command | Description |
|---------|-------------|
| `wpx post-type list` | List registered post types |
| `wpx post-type get <slug>` | Get post type details |
| `wpx post-status list` | List registered post statuses |

---

#### 7.3.15 Rewrites & Permalinks

**WP-CLI equivalent:** `wp rewrite`  
**Note:** Limited REST API surface. Some operations may require Abilities API.

| Command | Description |
|---------|-------------|
| `wpx rewrite flush` | Flush rewrite rules (via Abilities API) |
| `wpx rewrite list` | List rewrite rules (if exposed) |

---

#### 7.3.16 Roles & Capabilities

**WP-CLI equivalent:** `wp role`, `wp cap`  
**Note:** Requires custom REST endpoints or Abilities API.

| Command | Description |
|---------|-------------|
| `wpx role list` | List user roles |
| `wpx role get <role>` | Get role details with capabilities |
| `wpx role create <role> <name>` | Create a new role |
| `wpx role delete <role>` | Delete a role |
| `wpx cap list <role>` | List capabilities for a role |
| `wpx cap add <role> <cap>` | Add capability to role |
| `wpx cap remove <role> <cap>` | Remove capability from role |

---

#### 7.3.17 Cron

**WP-CLI equivalent:** `wp cron`  
**Note:** Requires custom REST endpoints or Abilities API.

| Command | Description |
|---------|-------------|
| `wpx cron event list` | List scheduled cron events |
| `wpx cron event run <hook>` | Trigger a cron event |
| `wpx cron event delete <hook>` | Delete a cron event |
| `wpx cron schedule list` | List cron schedules |
| `wpx cron test` | Test if WP-Cron is functional |

---

#### 7.3.18 Cache & Transients

**WP-CLI equivalent:** `wp cache`, `wp transient`  
**Note:** Requires custom REST endpoints or Abilities API.

| Command | Description |
|---------|-------------|
| `wpx cache flush` | Flush the object cache |
| `wpx cache get <key> [group]` | Get a cache value |
| `wpx cache set <key> <value> [group]` | Set a cache value |
| `wpx cache delete <key> [group]` | Delete a cache entry |
| `wpx transient get <key>` | Get a transient |
| `wpx transient set <key> <value> [expiry]` | Set a transient |
| `wpx transient delete <key>` | Delete a transient |
| `wpx transient delete --all` | Delete all transients |
| `wpx transient delete --expired` | Delete only expired transients |

---

#### 7.3.19 Database (via Abilities API)

**WP-CLI equivalent:** `wp db`  
**Note:** Direct database operations are not available via REST API. These require the Abilities API with a custom ability registered on the WordPress side, or a companion plugin.

| Command | Description | Implementation |
|---------|-------------|----------------|
| `wpx db query <sql>` | Execute a read-only SQL query | Abilities API |
| `wpx db export` | Export database dump | Abilities API |
| `wpx db import <file>` | Import database dump | Abilities API |
| `wpx db search <string>` | Search database for a string | Abilities API |
| `wpx db size` | Show database size | Abilities API |
| `wpx db tables` | List database tables | Abilities API |
| `wpx db optimize` | Optimize database tables | Abilities API |
| `wpx db repair` | Repair database tables | Abilities API |

---

#### 7.3.20 Search-Replace (via Abilities API)

**WP-CLI equivalent:** `wp search-replace`

| Command | Description |
|---------|-------------|
| `wpx search-replace <old> <new>` | Search and replace in database (serialization-aware) |

**Flags:**
```
--tables <list>     Comma-separated table list
--dry-run           Show what would change without applying
--precise           Use PHP serialization-safe replacement
--report            Show detailed change report
```

---

#### 7.3.21 Maintenance Mode

**WP-CLI equivalent:** `wp maintenance-mode`

| Command | Description |
|---------|-------------|
| `wpx maintenance enable` | Enable maintenance mode |
| `wpx maintenance disable` | Disable maintenance mode |
| `wpx maintenance status` | Check maintenance mode status |

---

#### 7.3.22 Languages / i18n

**WP-CLI equivalent:** `wp language`

| Command | Description |
|---------|-------------|
| `wpx language list` | List available languages |
| `wpx language install <locale>` | Install a language pack |
| `wpx language activate <locale>` | Set site language |
| `wpx language update` | Update all language packs |
| `wpx language uninstall <locale>` | Remove a language pack |

---

#### 7.3.23 Embed / oEmbed

**WP-CLI equivalent:** `wp embed`

| Command | Description |
|---------|-------------|
| `wpx embed fetch <url>` | Fetch oEmbed data for a URL |
| `wpx embed provider list` | List registered oEmbed providers |
| `wpx embed cache clear` | Clear embed cache |

---

#### 7.3.24 Site Health

**WP-CLI equivalent:** `wp site-health` (proposed in WP-CLI ideas #168)

| Command | Description |
|---------|-------------|
| `wpx health status` | Get overall site health status |
| `wpx health tests` | Run site health tests |
| `wpx health info` | Get site environment info |

---

#### 7.3.25 Multisite / Network

**WP-CLI equivalent:** `wp site`, `wp network`, `wp super-admin`  
**Note:** Requires multisite-enabled WordPress installation.

| Command | Description |
|---------|-------------|
| `wpx site list` | List sites in multisite network |
| `wpx site get <id>` | Get site details |
| `wpx site create` | Create a new site in the network |
| `wpx site delete <id>` | Delete a site |
| `wpx site activate <id>` | Activate a site |
| `wpx site deactivate <id>` | Deactivate (archive) a site |
| `wpx site spam <id>` | Mark site as spam |
| `wpx network meta list` | List network meta |
| `wpx super-admin list` | List super admins |
| `wpx super-admin add <user>` | Add super admin |
| `wpx super-admin remove <user>` | Remove super admin |

---

#### 7.3.26 WordPress Abilities API

**New — no WP-CLI equivalent**  
**REST API:** `/wp-abilities/v1/`

| Command | Description |
|---------|-------------|
| `wpx ability list` | List all registered abilities |
| `wpx ability get <name>` | Get ability details (schema, permissions) |
| `wpx ability run <name>` | Execute an ability with JSON input |
| `wpx ability category list` | List ability categories |
| `wpx ability category get <slug>` | Get category details |
| `wpx ability schema <name>` | Show input/output JSON Schema for an ability |

---

#### 7.3.27 WooCommerce (Extension Module)

**REST API:** `/wc/v3/`  
**Auth:** WooCommerce consumer key/secret

| Command | Description |
|---------|-------------|
| `wpx woo product list` | List products |
| `wpx woo product get <id>` | Get product details |
| `wpx woo product create` | Create a product |
| `wpx woo product update <id>` | Update a product |
| `wpx woo product delete <id>` | Delete a product |
| `wpx woo product variation list <product_id>` | List product variations |
| `wpx woo order list` | List orders |
| `wpx woo order get <id>` | Get order details |
| `wpx woo order create` | Create an order |
| `wpx woo order update <id>` | Update an order |
| `wpx woo order note list <order_id>` | List order notes |
| `wpx woo order note create <order_id>` | Add an order note |
| `wpx woo customer list` | List customers |
| `wpx woo customer get <id>` | Get customer details |
| `wpx woo customer create` | Create a customer |
| `wpx woo coupon list` | List coupons |
| `wpx woo coupon create` | Create a coupon |
| `wpx woo report sales` | Get sales report |
| `wpx woo report top-sellers` | Get top sellers |
| `wpx woo shipping zone list` | List shipping zones |
| `wpx woo tax rate list` | List tax rates |
| `wpx woo webhook list` | List webhooks |
| `wpx woo webhook create` | Create a webhook |
| `wpx woo setting list` | List WooCommerce settings |
| `wpx woo setting get <group> <id>` | Get specific setting |
| `wpx woo setting update <group> <id>` | Update a setting |
| `wpx woo system-status` | Get WooCommerce system status |

---

#### 7.3.28 Meta Commands

| Command | Description | WP-CLI Equivalent |
|---------|-------------|-------------------|
| `wpx info` | Show wpx version, configured sites, capabilities | `wp --info` |
| `wpx version` | Show version | `wp cli version` |
| `wpx self-update` | Update wpx to latest version | `wp cli update` |
| `wpx completions <shell>` | Generate shell completions (bash, zsh, fish, powershell) | `wp cli completions` |
| `wpx schema <command>` | Show JSON Schema for a command's input/output | — (new, agent-critical) |
| `wpx discover <url>` | Discover API capabilities of a WordPress site | — (new) |
| `wpx alias list` | List site aliases | `wp cli alias list` |
| `wpx alias set <name> <url>` | Set a site alias | `wp cli alias add` |
| `wpx help <command>` | Get help for a command | `wp help` |
| `wpx doctor` | Run diagnostic checks on configured sites | — (new) |

---

## 8. Agent-First Design Principles

### 8.1 Schema Introspection

Every command is self-documenting and machine-queryable:

```bash
# Show what a command accepts and returns
wpx schema post list
```

Output:
```json
{
  "command": "post list",
  "description": "List posts with optional filters",
  "input": {
    "type": "object",
    "properties": {
      "status": {
        "type": "string",
        "enum": ["publish", "draft", "pending", "private", "future", "trash"],
        "default": "publish"
      },
      "type": {
        "type": "string",
        "default": "post"
      },
      "per_page": {
        "type": "integer",
        "minimum": 1,
        "maximum": 100,
        "default": 10
      },
      "page": { "type": "integer", "minimum": 1 },
      "search": { "type": "string" },
      "author": { "type": "integer" },
      "orderby": {
        "type": "string",
        "enum": ["date", "id", "title", "slug", "modified", "relevance"]
      },
      "order": {
        "type": "string",
        "enum": ["asc", "desc"],
        "default": "desc"
      }
    }
  },
  "output": {
    "type": "array",
    "items": {
      "type": "object",
      "properties": {
        "id": { "type": "integer" },
        "title": { "type": "string" },
        "status": { "type": "string" },
        "date": { "type": "string", "format": "date-time" },
        "author": { "type": "integer" },
        "link": { "type": "string", "format": "uri" },
        "excerpt": { "type": "string" },
        "content": { "type": "string" }
      }
    }
  },
  "exit_codes": {
    "0": "Success",
    "1": "General error",
    "2": "Invalid arguments",
    "3": "Authentication failure",
    "4": "Resource not found",
    "5": "Permission denied",
    "6": "Rate limited",
    "7": "Network error",
    "8": "Server error (5xx)",
    "9": "Conflict (resource already exists)",
    "10": "Validation error"
  }
}
```

### 8.2 Semantic Exit Codes

| Code | Meaning | Agent Action |
|------|---------|--------------|
| 0 | Success | Continue |
| 1 | General error | Log and retry or abort |
| 2 | Invalid arguments | Fix arguments and retry |
| 3 | Authentication failure | Re-authenticate |
| 4 | Resource not found (404) | Handle gracefully |
| 5 | Permission denied (403) | Escalate or skip |
| 6 | Rate limited (429) | Wait and retry with backoff |
| 7 | Network error | Retry with backoff |
| 8 | Server error (5xx) | Retry with backoff |
| 9 | Conflict (409) | Resolve conflict |
| 10 | Validation error (422) | Fix input |

### 8.3 Structured Error Output

Errors always emit structured JSON to stderr:

```json
{
  "error": true,
  "code": "rest_post_invalid_id",
  "message": "Post not found with ID 99999",
  "status": 404,
  "exit_code": 4,
  "details": {
    "requested_id": 99999,
    "suggestion": "Use 'wpx post list' to find valid post IDs"
  }
}
```

### 8.4 NDJSON Streaming

For large result sets, use NDJSON (Newline Delimited JSON) to stream results without buffering:

```bash
wpx post list --status publish --per-page 100 --all-pages --output ndjson
```

Each line is a complete JSON object:
```
{"id":1,"title":"Hello World","status":"publish","date":"2026-01-01T00:00:00"}
{"id":2,"title":"Second Post","status":"publish","date":"2026-01-02T00:00:00"}
...
```

### 8.5 JSON Stdin for Complex Payloads

Agents can pass full payloads via stdin:

```bash
echo '{"title":"New Post","content":"<p>Hello</p>","status":"draft"}' | wpx post create --json
```

Or from a file:
```bash
wpx post create --json < post-payload.json
```

### 8.6 Field Masks

Reduce output to only needed fields (saves agent context window tokens):

```bash
wpx post list --fields id,title,status
```

```json
[
  {"id": 1, "title": "Hello World", "status": "publish"},
  {"id": 2, "title": "Sample Page", "status": "publish"}
]
```

### 8.7 Dry Run Mode

Every mutating command supports `--dry-run`:

```bash
wpx post delete 42 --dry-run
```

```json
{
  "dry_run": true,
  "action": "delete",
  "resource": "post",
  "id": 42,
  "would_delete": {
    "title": "Old Post",
    "status": "publish",
    "word_count": 1500
  }
}
```

### 8.8 Idempotent Operations

Commands are designed to be safely retried:

```bash
# Installing an already-installed plugin returns success, not error
wpx plugin install woocommerce --activate
# Exit code: 0
# {"status": "already_active", "version": "9.1.0"}
```

---

## 9. MCP Server Mode

wpx includes a built-in MCP (Model Context Protocol) server that exposes all CLI capabilities to AI agents.

### 9.1 Starting the MCP Server

```bash
# stdio transport (for Claude Desktop, Cursor, VS Code)
wpx mcp serve --transport stdio

# SSE transport (for web-based agents)
wpx mcp serve --transport sse --port 3000

# With specific site context
wpx mcp serve --site production --transport stdio
```

### 9.2 MCP Client Configuration

**Claude Desktop / claude_desktop_config.json:**
```json
{
  "mcpServers": {
    "wpx": {
      "command": "wpx",
      "args": ["mcp", "serve", "--transport", "stdio", "--site", "production"]
    }
  }
}
```

**VS Code settings.json:**
```json
{
  "mcp.servers": {
    "wpx": {
      "command": "wpx",
      "args": ["mcp", "serve", "--transport", "stdio", "--site", "production"]
    }
  }
}
```

### 9.3 Exposed MCP Tools

Every wpx command is automatically exposed as an MCP tool. Example tool definitions:

```json
{
  "name": "wpx_post_list",
  "description": "List WordPress posts with optional filters",
  "inputSchema": {
    "type": "object",
    "properties": {
      "site": { "type": "string", "description": "Site profile name" },
      "status": { "type": "string", "enum": ["publish", "draft", "pending", "private", "future", "trash"] },
      "search": { "type": "string" },
      "per_page": { "type": "integer", "minimum": 1, "maximum": 100 },
      "author": { "type": "integer" },
      "fields": { "type": "string", "description": "Comma-separated field mask" }
    }
  }
}
```

### 9.4 MCP Resources

Read-only data is also exposed as MCP resources:

| Resource URI | Description |
|-------------|-------------|
| `wpx://sites` | List of configured site profiles |
| `wpx://sites/{name}/info` | Site information and capabilities |
| `wpx://sites/{name}/plugins` | Current plugin list and status |
| `wpx://sites/{name}/themes` | Current theme list and status |
| `wpx://sites/{name}/health` | Site health status |

### 9.5 Safety Controls

- All mutating operations require explicit agent confirmation (MCP `tool` primitive)
- Read-only operations are exposed as MCP `resource` primitives (safe, no side effects)
- Per-site permission controls in config:

```toml
[sites.production.mcp]
allowed_commands = ["post.*", "plugin.list", "theme.list", "health.*"]
denied_commands = ["user.delete", "db.*", "plugin.delete"]
max_mutations_per_minute = 10
```

---

## 10. Multi-Site Fleet Management

A key differentiator: managing multiple WordPress sites from one CLI.

### 10.1 Fleet Commands

```bash
# Run a command across all configured sites
wpx fleet exec "plugin list" --output table

# Run across specific sites
wpx fleet exec "plugin update --all" --sites production,staging,dev

# Run across sites matching a pattern
wpx fleet exec "health status" --sites "client-*"

# Fleet status dashboard
wpx fleet status
```

### 10.2 Fleet Output

```json
{
  "results": [
    {
      "site": "production",
      "url": "https://example.com",
      "status": "success",
      "data": { ... },
      "duration_ms": 245
    },
    {
      "site": "staging",
      "url": "https://staging.example.com",
      "status": "success",
      "data": { ... },
      "duration_ms": 198
    }
  ],
  "summary": {
    "total": 2,
    "succeeded": 2,
    "failed": 0
  }
}
```

### 10.3 Concurrency Control

```bash
# Run across 20 sites, 5 at a time
wpx fleet exec "cache flush" --sites "client-*" --concurrency 5
```

---

## 11. Companion WordPress Plugin (wpx-bridge)

Some WP-CLI commands require server-side operations that the standard REST API does not expose (database operations, search-replace, cache management, cron, rewrite flushing). wpx includes an optional companion WordPress plugin called **wpx-bridge** that registers WordPress Abilities for these operations.

### 11.1 Plugin Architecture

- Lightweight (<50KB). No admin UI. REST/Abilities only.
- Registers abilities under the `wpx/` namespace.
- Respects WordPress capabilities and permissions.
- Auto-discovered by `wpx discover <url>`.

### 11.2 Registered Abilities

| Ability Name | Description | Required Capability |
|-------------|-------------|---------------------|
| `wpx/db-query` | Execute read-only SQL queries | `manage_options` |
| `wpx/db-export` | Export database | `manage_options` |
| `wpx/db-import` | Import database | `manage_options` |
| `wpx/db-search` | Search database tables | `manage_options` |
| `wpx/db-optimize` | Optimize tables | `manage_options` |
| `wpx/db-repair` | Repair tables | `manage_options` |
| `wpx/db-size` | Get database size | `manage_options` |
| `wpx/db-tables` | List tables | `manage_options` |
| `wpx/search-replace` | Serialization-safe search-replace | `manage_options` |
| `wpx/cache-flush` | Flush object cache | `manage_options` |
| `wpx/cache-get` | Get cache entry | `manage_options` |
| `wpx/cache-set` | Set cache entry | `manage_options` |
| `wpx/cache-delete` | Delete cache entry | `manage_options` |
| `wpx/transient-get` | Get transient | `manage_options` |
| `wpx/transient-set` | Set transient | `manage_options` |
| `wpx/transient-delete` | Delete transient(s) | `manage_options` |
| `wpx/cron-list` | List cron events | `manage_options` |
| `wpx/cron-run` | Run cron event | `manage_options` |
| `wpx/cron-delete` | Delete cron event | `manage_options` |
| `wpx/cron-schedules` | List cron schedules | `manage_options` |
| `wpx/rewrite-flush` | Flush rewrite rules | `manage_options` |
| `wpx/rewrite-list` | List rewrite rules | `manage_options` |
| `wpx/maintenance-enable` | Enable maintenance mode | `manage_options` |
| `wpx/maintenance-disable` | Disable maintenance mode | `manage_options` |
| `wpx/maintenance-status` | Check maintenance status | `manage_options` |
| `wpx/health-status` | Site health status | `manage_options` |
| `wpx/health-tests` | Run health tests | `manage_options` |
| `wpx/health-info` | Get environment info | `manage_options` |
| `wpx/media-regenerate` | Regenerate thumbnails | `manage_options` |
| `wpx/role-list` | List user roles | `manage_options` |
| `wpx/role-create` | Create role | `manage_options` |
| `wpx/role-delete` | Delete role | `manage_options` |
| `wpx/cap-add` | Add capability to role | `manage_options` |
| `wpx/cap-remove` | Remove capability from role | `manage_options` |

### 11.3 Graceful Degradation

When the wpx-bridge plugin is not installed, wpx will:
1. Detect missing abilities via `wpx discover`
2. Clearly indicate which commands are unavailable and why
3. Suggest installing the companion plugin
4. Still function fully for all native REST API commands

---

## 12. Output Examples

### 12.1 Table Output (Human, TTY)

```
$ wpx plugin list --site production

  NAME                    STATUS    VERSION   UPDATE
  akismet                 active    5.3.1     5.3.2
  woocommerce             active    9.1.0     —
  jetpack                 inactive  13.1      13.2
  wordpress-seo           active    22.1      —

  4 plugins total · 3 active · 1 inactive · 1 update available
```

### 12.2 JSON Output (Agent, Piped)

```json
[
  {
    "name": "akismet",
    "status": "active",
    "version": "5.3.1",
    "update_available": "5.3.2",
    "author": "Automattic",
    "requires_wp": "6.0",
    "requires_php": "7.4"
  },
  {
    "name": "woocommerce",
    "status": "active",
    "version": "9.1.0",
    "update_available": null,
    "author": "Automattic",
    "requires_wp": "6.4",
    "requires_php": "7.4"
  }
]
```

### 12.3 CSV Output

```csv
name,status,version,update_available
akismet,active,5.3.1,5.3.2
woocommerce,active,9.1.0,
jetpack,inactive,13.1,13.2
wordpress-seo,active,22.1,
```

### 12.4 NDJSON Output (Streaming)

```
{"name":"akismet","status":"active","version":"5.3.1","update_available":"5.3.2"}
{"name":"woocommerce","status":"active","version":"9.1.0","update_available":null}
{"name":"jetpack","status":"inactive","version":"13.1","update_available":"13.2"}
{"name":"wordpress-seo","status":"active","version":"22.1","update_available":null}
```

---

## 13. Error Handling Strategy

### 13.1 Retry Logic

```
Attempt 1: request
  ↓ failure (429 Rate Limited)
Wait: 1s (from Retry-After header, or exponential backoff)
Attempt 2: request
  ↓ failure (503 Service Unavailable)
Wait: 2s
Attempt 3: request
  ↓ success → return result
```

Configurable via `--retries` and `WPX_RETRIES`.

### 13.2 Error Response Structure

All errors emit to stderr as JSON:

```json
{
  "error": true,
  "code": "rest_forbidden",
  "message": "You are not authorized to perform this action",
  "status": 403,
  "exit_code": 5,
  "request": {
    "method": "DELETE",
    "endpoint": "/wp/v2/posts/42"
  },
  "suggestion": "Check that the authenticated user has 'delete_posts' capability"
}
```

### 13.3 Network Diagnostics

```bash
wpx doctor --site production
```

```json
{
  "site": "production",
  "url": "https://example.com",
  "checks": {
    "dns_resolution": { "status": "ok", "ip": "93.184.216.34", "ms": 12 },
    "tls_handshake": { "status": "ok", "version": "TLSv1.3", "ms": 45 },
    "rest_api_reachable": { "status": "ok", "wp_version": "6.9", "ms": 230 },
    "authentication": { "status": "ok", "user": "admin", "role": "administrator" },
    "abilities_api": { "status": "ok", "abilities_count": 34 },
    "woocommerce_api": { "status": "ok", "wc_version": "9.1.0" },
    "wpx_bridge": { "status": "installed", "version": "1.0.0", "abilities": 28 }
  },
  "latency_ms": 230,
  "overall": "healthy"
}
```

---

## 14. Performance Requirements

| Metric | Target | Comparison (WP-CLI) |
|--------|--------|---------------------|
| Cold start (no network) | <50ms | 150-350ms+ |
| Simple API request (e.g., post list) | <500ms total | 1-3s (with WP bootstrap) |
| Concurrent bulk operations | 10-50 parallel requests | Single-threaded |
| Binary size | <15MB (static) | ~30MB (PHAR + PHP) |
| Memory usage | <50MB typical | 128-512MB (PHP + WordPress) |
| Streaming (NDJSON) | Start output in <200ms | N/A (buffers all) |

---

## 15. Security Considerations

### 15.1 Credential Storage
- Prefer OS keyring (macOS Keychain, Windows Credential Manager, Linux Secret Service) via the `keyring` crate
- Fallback: encrypted credentials file with file-permission restrictions (0600)
- Never log credentials in debug output
- Application passwords are scoped and revocable per-session

### 15.2 Network Security
- HTTPS required by default. HTTP only with explicit `--insecure` flag
- Certificate validation enabled by default
- Support for custom CA certificates (corporate proxies)
- No cookies or session state stored

### 15.3 MCP Safety
- All destructive MCP tools require agent confirmation
- Configurable command allowlists/blocklists per site
- Rate limiting on mutation operations
- Audit log of all MCP-triggered actions

---

## 16. Configuration Precedence

From highest to lowest priority:

```
1. Explicit CLI flags          (--site production --output json)
2. Environment variables       (WPX_SITE=production WPX_OUTPUT=json)
3. Project config              (./.wpx.toml in current directory)
4. User config                 (~/.config/wpx/config.toml)
5. Built-in defaults
```

---

## 17. Phased Delivery Plan

### Phase 1 — Foundation (MVP)
- Core CLI framework (clap, output rendering, config management)
- Authentication (Application Passwords)
- Posts, Pages, Media, Users CRUD
- Plugins and Themes management
- Comments, Categories, Tags
- JSON/Table/CSV output + TTY detection
- Schema introspection (`wpx schema`)
- Semantic exit codes
- Shell completions

### Phase 2 — Extended API Coverage
- Blocks, Block Types, Patterns
- Widgets, Sidebars
- Menus, Menu Items, Menu Locations
- Search, Post Types, Post Statuses
- Settings/Options
- Multisite/Network commands
- Roles & Capabilities
- OAuth 2.1 authentication
- NDJSON streaming

### Phase 3 — Agent & Bridge
- MCP server mode (stdio + SSE)
- wpx-bridge companion plugin
- Database commands (via Abilities API)
- Search-Replace (via Abilities API)
- Cache/Transient management
- Cron management
- Maintenance mode
- Rewrite rules
- Site Health
- Media regeneration
- Fleet management (`wpx fleet`)
- Dry-run mode for all mutations

### Phase 4 — WooCommerce & Polish
- Full WooCommerce REST API coverage
- Products, Orders, Customers, Coupons, Reports
- Shipping, Tax, Webhooks, Settings
- Self-update mechanism
- Telemetry (opt-in)
- Plugin ecosystem (custom command extensions)
- Performance benchmarks and optimization pass

---

## 18. Testing Strategy

### 18.1 Unit Tests
- Every API client function tested with mock HTTP responses
- Schema validation tests for all command inputs/outputs
- Configuration parsing and precedence tests

### 18.2 Integration Tests
- Docker-based WordPress test environment (similar to wordpress-rs approach)
- Full CRUD lifecycle tests for every resource type
- Authentication flow tests (Application Passwords, OAuth)
- WooCommerce integration tests
- Multisite integration tests

### 18.3 Agent Simulation Tests
- Automated test suite that drives wpx via MCP protocol
- Verifies schema introspection contracts
- Tests exit code accuracy
- Tests NDJSON streaming correctness
- Tests idempotency guarantees
- Tests error recovery flows

### 18.4 CLI Output Contract Tests
- JSON Schema validation of all command outputs (CI-enforced)
- Breaking change detection on every PR
- Regression tests using `assert_cmd` and `predicates` crates

---

## 19. Distribution

| Platform | Method |
|----------|--------|
| macOS (ARM/x86) | Homebrew tap, direct binary download |
| Linux (ARM/x86) | APT/RPM repos, direct binary, Docker image |
| Windows (x86) | Scoop, WinGet, direct binary |
| Cargo | `cargo install wpx` |
| Docker | `docker run wpx/wpx post list --site production` |
| GitHub Releases | Pre-built binaries for all platforms |
| Nix | Nix package |

---

## 20. Open Questions

| # | Question | Owner | Blocking? |
|---|----------|-------|-----------|
| OQ1 | Should wpx support WPGraphQL as an alternative API backend? | Engineering | No (P2 consideration) |
| OQ2 | How to handle WordPress.com hosted sites vs self-hosted? OAuth flows differ. | Engineering | No (Phase 2) |
| OQ3 | Should the wpx-bridge plugin be submitted to WordPress.org plugin directory? | Product | No |
| OQ4 | What is the minimum WordPress version to support? 6.0? 6.5? | Product | Yes (Phase 1) |
| OQ5 | Should wpx support reading wp-cli.yml for migration purposes? | Engineering | No |
| OQ6 | How to handle very large media uploads (>100MB) — chunked upload support? | Engineering | No (Phase 2) |
| OQ7 | License: MIT, Apache 2.0, or MPL 2.0 (matching wordpress-rs)? | Legal | Yes (before public release) |
| OQ8 | Should fleet management support dynamic site discovery (e.g., from a fleet registry API)? | Product | No (Phase 4) |

---

## Appendix A: WordPress REST API Endpoint Coverage

### Core REST API (wp/v2)

| Endpoint | wpx Command | Phase |
|----------|-------------|-------|
| `/wp/v2/posts` | `wpx post *` | 1 |
| `/wp/v2/posts/<id>/revisions` | `wpx post revision *` | 1 |
| `/wp/v2/pages` | `wpx page *` | 1 |
| `/wp/v2/pages/<id>/revisions` | `wpx page revision *` | 1 |
| `/wp/v2/media` | `wpx media *` | 1 |
| `/wp/v2/comments` | `wpx comment *` | 1 |
| `/wp/v2/categories` | `wpx category *` | 1 |
| `/wp/v2/tags` | `wpx tag *` | 1 |
| `/wp/v2/taxonomies` | `wpx taxonomy *` | 1 |
| `/wp/v2/users` | `wpx user *` | 1 |
| `/wp/v2/plugins` | `wpx plugin *` | 1 |
| `/wp/v2/themes` | `wpx theme *` | 1 |
| `/wp/v2/types` | `wpx post-type *` | 2 |
| `/wp/v2/statuses` | `wpx post-status *` | 2 |
| `/wp/v2/settings` | `wpx option *` | 2 |
| `/wp/v2/search` | `wpx search` | 2 |
| `/wp/v2/blocks` | `wpx block *` | 2 |
| `/wp/v2/block-types` | `wpx block-type *` | 2 |
| `/wp/v2/block-renderer` | `wpx block render` | 2 |
| `/wp/v2/block-directory/search` | `wpx block search` | 2 |
| `/wp/v2/block-patterns` | `wpx block-pattern *` | 2 |
| `/wp/v2/block-pattern-categories` | `wpx block-pattern-category *` | 2 |
| `/wp/v2/widgets` | `wpx widget *` | 2 |
| `/wp/v2/sidebars` | `wpx sidebar *` | 2 |
| `/wp/v2/widget-types` | `wpx widget-type *` | 2 |
| `/wp/v2/menus` | `wpx menu *` | 2 |
| `/wp/v2/menu-items` | `wpx menu-item *` | 2 |
| `/wp/v2/menu-locations` | `wpx menu-location *` | 2 |
| `/wp/v2/sites` | `wpx site *` (multisite) | 2 |

### Abilities API (wp-abilities/v1)

| Endpoint | wpx Command | Phase |
|----------|-------------|-------|
| `/wp-abilities/v1/abilities` | `wpx ability list` | 3 |
| `/wp-abilities/v1/abilities/<name>` | `wpx ability get` | 3 |
| `/wp-abilities/v1/abilities/<name>/run` | `wpx ability run` | 3 |
| `/wp-abilities/v1/categories` | `wpx ability category list` | 3 |

### WooCommerce REST API (wc/v3)

| Endpoint | wpx Command | Phase |
|----------|-------------|-------|
| `/wc/v3/products` | `wpx woo product *` | 4 |
| `/wc/v3/products/<id>/variations` | `wpx woo product variation *` | 4 |
| `/wc/v3/orders` | `wpx woo order *` | 4 |
| `/wc/v3/orders/<id>/notes` | `wpx woo order note *` | 4 |
| `/wc/v3/customers` | `wpx woo customer *` | 4 |
| `/wc/v3/coupons` | `wpx woo coupon *` | 4 |
| `/wc/v3/reports/sales` | `wpx woo report *` | 4 |
| `/wc/v3/shipping/zones` | `wpx woo shipping *` | 4 |
| `/wc/v3/taxes` | `wpx woo tax *` | 4 |
| `/wc/v3/webhooks` | `wpx woo webhook *` | 4 |
| `/wc/v3/settings` | `wpx woo setting *` | 4 |
| `/wc/v3/system_status` | `wpx woo system-status` | 4 |

---

## Appendix B: WP-CLI Feature Parity Checklist

| WP-CLI Command | wpx Equivalent | Status | Notes |
|---------------|---------------|--------|-------|
| `wp post` | `wpx post` | Planned (Phase 1) | Full CRUD + revisions + meta + bulk |
| `wp page` | `wpx page` | Planned (Phase 1) | Separate resource (not --post_type) |
| `wp media` | `wpx media` | Planned (Phase 1) | Upload via multipart + sideload |
| `wp comment` | `wpx comment` | Planned (Phase 1) | + bulk operations |
| `wp user` | `wpx user` | Planned (Phase 1) | + app-password management |
| `wp plugin` | `wpx plugin` | Planned (Phase 1) | + auto-update management |
| `wp theme` | `wpx theme` | Planned (Phase 1) | Full lifecycle |
| `wp option` | `wpx option` | Planned (Phase 2) | Limited to REST-exposed settings |
| `wp term` | `wpx term` | Planned (Phase 1) | Custom taxonomy support |
| `wp taxonomy` | `wpx taxonomy` | Planned (Phase 1) | Read-only |
| `wp menu` | `wpx menu` | Planned (Phase 2) | Full menu + items |
| `wp widget` | `wpx widget` | Planned (Phase 2) | + widget types |
| `wp sidebar` | `wpx sidebar` | Planned (Phase 2) | Read-only |
| `wp block` | `wpx block` | Planned (Phase 2) | Reusable blocks + types + patterns |
| `wp search-replace` | `wpx search-replace` | Planned (Phase 3) | Via wpx-bridge |
| `wp db` | `wpx db` | Planned (Phase 3) | Via wpx-bridge |
| `wp cache` | `wpx cache` | Planned (Phase 3) | Via wpx-bridge |
| `wp transient` | `wpx transient` | Planned (Phase 3) | Via wpx-bridge |
| `wp cron` | `wpx cron` | Planned (Phase 3) | Via wpx-bridge |
| `wp rewrite` | `wpx rewrite` | Planned (Phase 3) | Via wpx-bridge |
| `wp role` | `wpx role` | Planned (Phase 2) | Via custom endpoint or Abilities |
| `wp cap` | `wpx cap` | Planned (Phase 2) | Via custom endpoint or Abilities |
| `wp site` (multisite) | `wpx site` | Planned (Phase 2) | Network management |
| `wp super-admin` | `wpx super-admin` | Planned (Phase 2) | Multisite |
| `wp network` | `wpx network` (via site/fleet) | Planned (Phase 2) | — |
| `wp language` | `wpx language` | Planned (Phase 2) | — |
| `wp maintenance-mode` | `wpx maintenance` | Planned (Phase 3) | Via wpx-bridge |
| `wp embed` | `wpx embed` | Planned (Phase 2) | — |
| `wp core` | Out of scope | N/A | Requires filesystem access |
| `wp config` | Out of scope | N/A | Requires filesystem access |
| `wp scaffold` | Out of scope | N/A | Requires PHP/filesystem |
| `wp eval` / `wp eval-file` | Out of scope | N/A | Requires PHP runtime |
| `wp shell` | Out of scope | N/A | Requires PHP runtime |
| `wp server` | Out of scope | N/A | Requires local PHP |
| `wp db` (direct SQL) | Partially via bridge | Phase 3 | Read-only by default for safety |
| `wp export` | `wpx post export` | Planned (Phase 1) | JSON/WXR export via REST |
| `wp import` | Future | TBD | Complex — requires server-side processing |
| `wp profile` | `wpx doctor` | Planned (Phase 3) | Network-level diagnostics instead |
| `wp i18n` | Out of scope | N/A | Build tool — requires filesystem |
| `wp dist-archive` | Out of scope | N/A | Build tool — requires filesystem |
| `wp find` | Out of scope | N/A | Filesystem scan |
| `wp package` | N/A | N/A | wpx uses native extension system |

---

## Appendix C: Research Sources

- WP-CLI GitHub Repository: https://github.com/wp-cli/wp-cli
- WP-CLI Command Reference: https://developer.wordpress.org/cli/commands/
- WP-CLI Ideas / Feature Requests: https://github.com/wp-cli/ideas/issues
- WordPress REST API Reference: https://developer.wordpress.org/rest-api/reference/
- WordPress Abilities API (6.9): https://make.wordpress.org/core/2025/11/10/abilities-api-in-wordpress-6-9/
- WordPress MCP Adapter: https://developer.wordpress.org/news/2026/02/from-abilities-to-ai-agents-introducing-the-wordpress-mcp-adapter/
- Automattic wordpress-rs (Rust WordPress API): https://github.com/Automattic/wordpress-rs
- WordPress Application Passwords: https://make.wordpress.org/core/2020/11/05/application-passwords-integration-guide/
- WooCommerce REST API: https://developer.woocommerce.com/docs/apis/rest-api/
- InfoQ: Keep the Terminal Relevant — Patterns for AI Agent Driven CLIs: https://www.infoq.com/articles/ai-agent-cli/
- DEV.to: Rewrite Your CLI for Agents (Or Get Replaced): https://dev.to/meimakes/rewrite-your-cli-for-agents-or-get-replaced-2a2h
- RudderStack: AI Agents Need Two Interfaces — CLI and MCP: https://www.rudderstack.com/blog/ai-agents-cli-mcp-design-pattern/
- WordPress.com MCP Integration: https://developer.wordpress.com/docs/mcp/
- WP-CLI Persistent Instance Feature Request: https://github.com/wp-cli/ideas/issues/179
- WP-CLI Rust Speed-Up Feature Request: https://github.com/wp-cli/ideas/issues/178
- Stack Overflow: Speed up slow WP-CLI: https://stackoverflow.com/questions/59092832/speed-up-slow-wp-cli
