# wpx -- WordPress CLI for AI Agents & Humans

[![CI](https://github.com/osodevops/wordpress-cli/actions/workflows/ci.yml/badge.svg)](https://github.com/osodevops/wordpress-cli/actions/workflows/ci.yml)
[![License: MIT/Apache-2.0](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-2021-orange.svg)](https://www.rust-lang.org/)

Rust-native CLI for managing WordPress sites remotely via the REST API. Inspired by [WP-CLI](https://github.com/wp-cli/wp-cli), but built from the ground up for AI agents -- because agents shouldn't need PHP installed to manage a WordPress site.

WP-CLI requires a local PHP runtime, a bootstrapped WordPress installation, and SSH access to run. That's fine for humans on a server, but it's a non-starter for AI coding agents like Claude, Cursor, or Codex that operate remotely over HTTPS. **wpx** solves this by communicating purely via the WordPress REST API as a single static binary with zero runtime dependencies.

Designed agent-first: structured JSON output, schema introspection, and semantic exit codes let AI agents operate WordPress programmatically. Designed human-friendly: auto-detecting table output, colored terminals, and shell completions make it pleasant for interactive use.

## Features

- **Full WordPress REST API coverage** -- 34 command groups spanning posts, pages, media, users, plugins, themes, menus, widgets, blocks, and more
- **Structured output** -- JSON, Table, CSV, YAML, and NDJSON formats with automatic TTY detection
- **Schema introspection** -- `wpx schema <command>` emits JSON Schema for any command's input/output
- **Semantic exit codes** -- 11 distinct codes (0-10) so agents can programmatically decide retry/abort/fix strategies
- **Multi-site fleet management** -- run commands across dozens of sites concurrently with `wpx fleet exec`
- **Application Passwords & OAuth 2.1** -- secure authentication without exposing wp-admin credentials
- **Field masking** -- `--fields id,title,status` to reduce payload size for agent consumption
- **Automatic pagination** -- `--all-pages` streams every result as NDJSON
- **Site discovery** -- `wpx discover <url>` probes a site's REST API capabilities
- **Dry-run mode** -- `--dry-run` previews destructive operations before executing

## Quick Start

### Install

**Homebrew** (macOS/Linux):

```bash
brew tap osodevops/wpx
brew install osodevops/tap/wordpress-cli
```

**Cargo** (from source):

```bash
cargo install --git https://github.com/osodevops/wordpress-cli wpx-cli
```

**Binary download** -- grab the latest release for your platform from the [Releases](https://github.com/osodevops/wordpress-cli/releases) page.

### Configure a Site

```bash
wpx auth set --site production --username admin --password "xxxx xxxx xxxx xxxx"
```

Credentials are stored in `~/.config/wpx/credentials.toml` with restricted file permissions.

### Test Authentication

```bash
wpx auth test --site production
```

### Your First Commands

```bash
# List recent posts (human-readable table in terminal)
wpx post list --site production

# Same command, but pipe to jq (auto-switches to JSON)
wpx post list --site production | jq '.[0].title'

# List plugins as JSON
wpx plugin list --site production --output json

# Create a draft post
wpx post create --site production --title "Hello from wpx" --status draft

# Global search
wpx search "migration guide" --site production
```

### Custom Post Types, Content Files, Media and Raw Routes

```bash
# Any post type exposed in REST: --type resolves wp/v2/types/<slug> to its rest_base
wpx post list --type blog --per-page 5
wpx post get 6166 --type blog --context edit --fields id,slug,content.raw,acf,meta

# Push block markup from a file (or "-" for stdin); unknown JSON keys pass through verbatim
wpx post create --type blog --title "Hello" --status draft --content-file ./post.html
echo '{"title":"Hi","status":"draft","blog_category":[9],"acf":{"hero":{"title":"x"}}}' \
  | wpx post create --type blog --json

# Pages: template + featured image + ACF/meta via --json
wpx page update 4338 --template template-kafka-services.php --featured-media 6168 --content-file page.html

# Upload media
wpx media upload ./hero.png --title "Hero" --alt-text "Kafka cluster diagram"

# Anything else: raw REST route (relative to /wp-json/)
wpx api GET wp/v2/types/blog --fields rest_base
wpx api POST rankmath/v1/updateMeta --data '{"objectType":"post","objectID":42,"meta":{"rank_math_title":"x"}}'
```

Every write honours `--dry-run`.

## Command Reference

### Content

| Command | Description | Subcommands |
|---------|-------------|-------------|
| `post` | Manage posts of any type (`--type blog`, `--type page`, ...) | `list`, `get`, `create`, `update`, `delete`, `search` |
| `page` | Manage pages | `list`, `get`, `create`, `update`, `delete` |
| `media` | Manage media attachments | `list`, `get`, `upload`, `update`, `delete` |
| `comment` | Manage comments | `list`, `get`, `create`, `update`, `delete` |
| `block` | Manage reusable blocks | `list`, `get`, `create`, `update`, `delete`, `search`, `render` |
| `search` | Global search across content | *(direct command -- takes a query argument)* |

### Taxonomy

| Command | Description | Subcommands |
|---------|-------------|-------------|
| `category` | Manage categories | `list`, `get`, `create`, `update`, `delete` |
| `tag` | Manage tags | `list`, `get`, `create`, `update`, `delete` |
| `taxonomy` | List and inspect taxonomies | `list`, `get` |

### Users

| Command | Description | Subcommands |
|---------|-------------|-------------|
| `user` | Manage users | `list`, `get`, `me`, `create`, `update`, `delete` |

### Site Management

| Command | Description | Subcommands |
|---------|-------------|-------------|
| `plugin` | Manage plugins | `list`, `get`, `install`, `activate`, `deactivate`, `update`, `delete`, `status` |
| `theme` | Manage themes | `list`, `get`, `activate`, `delete`, `status` |
| `settings` | Manage site settings | `list`, `get`, `set` |
| `option` | Alias for `settings` | `list`, `get`, `set` |

### Block Editor

| Command | Description | Subcommands |
|---------|-------------|-------------|
| `block-type` | List and inspect block types | `list`, `get` |
| `block-pattern` | List block patterns | `list` |
| `block-pattern-category` | List block pattern categories | `list` |

### Navigation & Layout

| Command | Description | Subcommands |
|---------|-------------|-------------|
| `menu` | Manage navigation menus | `list`, `get`, `create`, `update`, `delete` |
| `menu-item` | Manage menu items | `list`, `get`, `create`, `update`, `delete` |
| `menu-location` | List and inspect menu locations | `list`, `get` |
| `widget` | Manage widgets | `list`, `get`, `create`, `update`, `delete` |
| `widget-type` | List and inspect widget types | `list`, `get` |
| `sidebar` | List and inspect sidebars | `list`, `get` |

### Introspection

| Command | Description | Subcommands |
|---------|-------------|-------------|
| `post-type` | List and inspect post types | `list`, `get` |
| `post-status` | List and inspect post statuses | `list`, `get` |
| `discover` | Probe a site's REST API capabilities | *(direct command -- takes a URL argument)* |
| `schema` | Show JSON Schema for a command | *(direct command -- takes a command path)* |
| `api` | Call any REST route under `/wp-json/` | *(direct command -- `api <METHOD> <path>`)* |

### Utilities

| Command | Description | Subcommands |
|---------|-------------|-------------|
| `fleet` | Run commands across multiple sites | `exec`, `status` |
| `auth` | Manage authentication | `set`, `test`, `list`, `logout`, `oauth` |
| `info` | Show version, sites, capabilities | *(direct command)* |
| `version` | Show version information | *(direct command)* |
| `completions` | Generate shell completions | *(direct command -- takes a shell argument)* |

## Global Flags

| Flag | Env Var | Default | Description |
|------|---------|---------|-------------|
| `--site <name>` | `WPX_SITE` | `default` | Target site profile name |
| `--url <url>` | `WPX_URL` | -- | Direct URL override (skips profile lookup) |
| `--output <fmt>` | `WPX_OUTPUT` | `auto` | Output format: `json`, `table`, `csv`, `yaml`, `ndjson`, `auto` |
| `--fields <f1,f2>` | -- | -- | Comma-separated field mask to reduce output; dotted paths select nested keys (`content.raw`, `acf.hero.title`) |
| `--no-color` | `NO_COLOR` | -- | Disable colored output |
| `--no-prompt` | `WPX_NO_PROMPT` | -- | Disable all interactive prompts |
| `--quiet` | -- | -- | Suppress non-essential output |
| `--verbose` | -- | -- | Enable debug logging to stderr |
| `--timeout <secs>` | `WPX_TIMEOUT` | `30` | Request timeout in seconds |
| `--retries <n>` | `WPX_RETRIES` | `3` | Retry count for failed requests |
| `--dry-run` | -- | -- | Show what would be done without executing |
| `--confirm` | -- | -- | Skip confirmation prompts for destructive actions |
| `--all-pages` | -- | -- | Fetch all pages and stream results (forces NDJSON) |

## Configuration

### Config Files

- **User-level:** `~/.config/wpx/config.toml`
- **Project-level:** `./.wpx.toml` (checked in to your repo for team defaults)
- **Credentials:** `~/.config/wpx/credentials.toml` (never commit this)

Example `config.toml`:

```toml
[default]
output = "json"
timeout = 60
retries = 5

[sites.production]
url = "https://example.com"
auth = "application-password"
username = "admin"

[sites.staging]
url = "https://staging.example.com"
auth = "application-password"
username = "editor"
```

### Environment Variables

| Variable | Description |
|----------|-------------|
| `WPX_SITE` | Default site profile name |
| `WPX_URL` | Direct WordPress URL (bypasses profile lookup) |
| `WPX_USERNAME` | Username for application-password auth (overrides `credentials.toml`) |
| `WPX_PASSWORD` | Application password (alias: `WPX_APP_PASSWORD`); with `WPX_URL` no site profile is needed -- ideal for CI |
| `WPX_OUTPUT` | Default output format |
| `WPX_TIMEOUT` | Request timeout in seconds |
| `WPX_RETRIES` | Retry count for failed requests |
| `WPX_NO_PROMPT` | Disable interactive prompts |
| `NO_COLOR` | Disable colored output (standard convention) |

### Precedence

```
CLI flags  >  env vars  >  project config (.wpx.toml)  >  user config (~/.config/wpx/config.toml)  >  defaults
```

## Authentication

### Application Passwords (Recommended)

WordPress 5.6+ supports Application Passwords natively. No plugins required.

1. Log in to wp-admin and navigate to **Users > Profile**
2. Scroll to **Application Passwords**
3. Enter a name (e.g., "wpx CLI") and click **Add New Application Password**
4. Copy the generated password (spaces are fine -- wpx strips them)
5. Store it with wpx:

```bash
wpx auth set --site production --username admin --password "XXXX XXXX XXXX XXXX"
```

`auth set` stores the password; the site URL comes from a `[sites.production]` profile in
`~/.config/wpx/config.toml` or a project `.wpx.toml`. For CI, skip the files entirely:

```bash
export WPX_URL=https://example.com WPX_USERNAME=ci-bot WPX_PASSWORD="xxxx xxxx xxxx xxxx"
wpx auth test
```

### OAuth 2.1

For environments that require OAuth (headless WordPress, enterprise SSO):

```bash
wpx auth oauth \
  --site production \
  --client-id "your-client-id" \
  --authorize-url "https://example.com/oauth/authorize" \
  --token-url "https://example.com/oauth/token"
```

This opens a browser for the authorization code flow with PKCE, then stores the resulting token.

## Output Formats

wpx supports six output formats. By default (`--output auto`), it renders tables when stdout is a TTY and JSON when piped.

| Format | Flag | Description |
|--------|------|-------------|
| Auto | `--output auto` | Table if TTY, JSON if piped |
| JSON | `--output json` | Pretty-printed JSON |
| Table | `--output table` | ASCII table for human reading |
| CSV | `--output csv` | Comma-separated values |
| YAML | `--output yaml` | YAML format |
| NDJSON | `--output ndjson` | Newline-delimited JSON (one object per line) |

```bash
# Interactive terminal: auto-renders a table
wpx post list --site production

# Piped to another program: auto-switches to JSON
wpx post list --site production | jq '.[] | .title'

# Force a specific format
wpx plugin list --site production --output csv > plugins.csv
```

## Fleet Management

Run commands across multiple WordPress sites concurrently:

```bash
# Check plugin status across all configured sites
wpx fleet exec "plugin list" --sites "production,staging,dev"

# Update a plugin everywhere (with concurrency limit)
wpx fleet exec "plugin update --slug akismet" --concurrency 3

# Check fleet health
wpx fleet status
```

## Exit Codes

Every wpx command returns a semantic exit code that agents can use for programmatic error handling:

| Code | Name | Description |
|------|------|-------------|
| `0` | Success | Command completed successfully |
| `1` | General Error | Unclassified error |
| `2` | Invalid Args | Bad CLI arguments or configuration |
| `3` | Auth Failure | Authentication failed (bad credentials, expired token) |
| `4` | Not Found | Requested resource does not exist |
| `5` | Permission Denied | Authenticated but not authorized |
| `6` | Rate Limited | Too many requests (check `retry_after_secs` in error JSON) |
| `7` | Network Error | DNS, TLS, or connection failure |
| `8` | Server Error | WordPress returned HTTP 5xx |
| `9` | Conflict | Resource conflict (e.g., duplicate slug) |
| `10` | Validation Error | Invalid field value (check `field` in error JSON) |

Errors are emitted as structured JSON on stderr:

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

## Schema Introspection

Agents can discover the input/output shape of any command at runtime:

```bash
# Show schema for a specific command
wpx schema post list

# List all available schemas
wpx schema
```

## Architecture

```
wpx binary (6 crates)
├── wpx-cli       CLI parsing (clap), command dispatch, shell completions
├── wpx-core      Resource trait, domain types, semantic exit codes
├── wpx-api       HTTP client (reqwest + rustls), retry, pagination
├── wpx-auth      Application Passwords, OAuth 2.1 (PKCE)
├── wpx-config    TOML config, site profiles, credential store
└── wpx-output    JSON, Table, CSV, YAML, NDJSON rendering
```

All crates share a workspace version (`0.1.0`) and are published together.

## Building from Source

```bash
# Clone
git clone https://github.com/osodevops/wordpress-cli.git
cd wordpress-cli

# Build
cargo build --release

# Run tests
cargo test --workspace

# Lint
cargo clippy --workspace -- -D warnings

# The binary is at target/release/wpx
./target/release/wpx --version
```

### Generate Shell Completions

```bash
# Bash
wpx completions bash > ~/.local/share/bash-completion/completions/wpx

# Zsh
wpx completions zsh > ~/.zfunc/_wpx

# Fish
wpx completions fish > ~/.config/fish/completions/wpx.fish
```

## License

Licensed under either of:

- [MIT License](LICENSE-MIT)
- [Apache License, Version 2.0](LICENSE-APACHE)

at your option.
