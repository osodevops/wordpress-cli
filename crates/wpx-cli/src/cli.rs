use clap::{Args, Parser, Subcommand};
use wpx_output::OutputFormat;

/// wpx — A Rust-native WordPress CLI for agents and humans.
///
/// Manage WordPress sites remotely via the REST API.
/// Produces structured, machine-readable output by default when piped.
#[derive(Debug, Parser)]
#[command(
    name = "wpx",
    version,
    about = "Rust-native WordPress CLI for agents and humans",
    long_about = "wpx manages WordPress sites remotely via the REST API.\n\n\
        Every command produces structured, machine-readable output by default \
        when piped, and human-readable tables when run interactively."
)]
pub struct Cli {
    #[command(flatten)]
    pub global: GlobalFlags,

    #[command(subcommand)]
    pub command: Commands,
}

/// Global flags available on every command.
#[derive(Debug, Args)]
pub struct GlobalFlags {
    /// Target site profile name from config.
    #[arg(long, env = "WPX_SITE", default_value = "default", global = true)]
    pub site: String,

    /// Direct URL override (skips profile lookup).
    #[arg(long, env = "WPX_URL", global = true)]
    pub url: Option<String>,

    /// Output format: json, table, csv, yaml, ndjson, auto.
    #[arg(long, env = "WPX_OUTPUT", default_value = "auto", value_enum, global = true)]
    pub output: OutputFormat,

    /// Comma-separated field mask to reduce output.
    #[arg(long, value_delimiter = ',', global = true)]
    pub fields: Option<Vec<String>>,

    /// Disable colored output.
    #[arg(long, env = "NO_COLOR", global = true)]
    pub no_color: bool,

    /// Disable all interactive prompts.
    #[arg(long, env = "WPX_NO_PROMPT", global = true)]
    pub no_prompt: bool,

    /// Suppress non-essential output.
    #[arg(long, global = true)]
    pub quiet: bool,

    /// Enable debug logging to stderr.
    #[arg(long, global = true)]
    pub verbose: bool,

    /// Request timeout in seconds.
    #[arg(long, env = "WPX_TIMEOUT", default_value = "30", global = true)]
    pub timeout: u64,

    /// Retry count for failed requests.
    #[arg(long, env = "WPX_RETRIES", default_value = "3", global = true)]
    pub retries: u32,

    /// Show what would be done without executing.
    #[arg(long, global = true)]
    pub dry_run: bool,

    /// Skip confirmation prompts for destructive actions.
    #[arg(long, global = true)]
    pub confirm: bool,

    /// Fetch all pages and stream results (forces NDJSON output).
    #[arg(long, global = true)]
    pub all_pages: bool,
}

/// Top-level command groups.
#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Manage posts (CRUD, search, bulk operations).
    Post {
        #[command(subcommand)]
        command: crate::commands::post::PostCommands,
    },

    /// Manage pages (CRUD).
    Page {
        #[command(subcommand)]
        command: crate::commands::page::PageCommands,
    },

    /// Manage media attachments.
    Media {
        #[command(subcommand)]
        command: crate::commands::media::MediaCommands,
    },

    /// Manage users.
    User {
        #[command(subcommand)]
        command: crate::commands::user::UserCommands,
    },

    /// Manage comments.
    Comment {
        #[command(subcommand)]
        command: crate::commands::comment::CommentCommands,
    },

    /// Manage categories.
    Category {
        #[command(subcommand)]
        command: crate::commands::category::CategoryCommands,
    },

    /// Manage tags.
    Tag {
        #[command(subcommand)]
        command: crate::commands::tag::TagCommands,
    },

    /// List and inspect taxonomies.
    Taxonomy {
        #[command(subcommand)]
        command: crate::commands::taxonomy::TaxonomyCommands,
    },

    /// Manage plugins.
    Plugin {
        #[command(subcommand)]
        command: crate::commands::plugin::PluginCommands,
    },

    /// Manage themes.
    Theme {
        #[command(subcommand)]
        command: crate::commands::theme::ThemeCommands,
    },

    /// List and inspect post types.
    #[command(name = "post-type")]
    PostType {
        #[command(subcommand)]
        command: crate::commands::post_type::PostTypeCommands,
    },

    /// List and inspect post statuses.
    #[command(name = "post-status")]
    PostStatus {
        #[command(subcommand)]
        command: crate::commands::post_status::PostStatusCommands,
    },

    /// List and inspect block types.
    #[command(name = "block-type")]
    BlockType {
        #[command(subcommand)]
        command: crate::commands::block_type::BlockTypeCommands,
    },

    /// List block patterns.
    #[command(name = "block-pattern")]
    BlockPattern {
        #[command(subcommand)]
        command: crate::commands::block_pattern::BlockPatternCommands,
    },

    /// List block pattern categories.
    #[command(name = "block-pattern-category")]
    BlockPatternCategory {
        #[command(subcommand)]
        command: crate::commands::block_pattern_category::BlockPatternCategoryCommands,
    },

    /// List and inspect widget types.
    #[command(name = "widget-type")]
    WidgetType {
        #[command(subcommand)]
        command: crate::commands::widget_type::WidgetTypeCommands,
    },

    /// List and inspect sidebars.
    Sidebar {
        #[command(subcommand)]
        command: crate::commands::sidebar::SidebarCommands,
    },

    /// List and inspect menu locations.
    #[command(name = "menu-location")]
    MenuLocation {
        #[command(subcommand)]
        command: crate::commands::menu_location::MenuLocationCommands,
    },

    /// Manage reusable blocks (CRUD, search, render).
    Block {
        #[command(subcommand)]
        command: crate::commands::block::BlockCommands,
    },

    /// Manage widgets (CRUD).
    Widget {
        #[command(subcommand)]
        command: crate::commands::widget::WidgetCommands,
    },

    /// Manage navigation menus (CRUD).
    Menu {
        #[command(subcommand)]
        command: crate::commands::menu::MenuCommands,
    },

    /// Manage navigation menu items (CRUD).
    #[command(name = "menu-item")]
    MenuItem {
        #[command(subcommand)]
        command: crate::commands::menu_item::MenuItemCommands,
    },

    /// Global search across posts, pages, terms, etc.
    Search {
        /// Search query.
        query: String,
        #[command(flatten)]
        args: crate::commands::search::SearchArgs,
    },

    /// Manage site settings / options.
    Settings {
        #[command(subcommand)]
        command: crate::commands::settings::SettingsCommands,
    },

    /// Manage site settings / options (alias for settings).
    Option {
        #[command(subcommand)]
        command: crate::commands::settings::SettingsCommands,
    },

    /// Run commands across multiple WordPress sites.
    Fleet {
        #[command(subcommand)]
        command: crate::commands::fleet::FleetCommands,
    },

    /// Start the MCP (Model Context Protocol) server for AI agent integration.
    Mcp {
        #[command(subcommand)]
        command: McpCommands,
    },

    /// Discover a WordPress site's capabilities (REST API, wpx-bridge, WooCommerce).
    Discover {
        /// URL of the WordPress site to discover.
        url: String,
    },

    /// Show JSON Schema for a command's input/output.
    Schema {
        /// Command path (e.g., "post list"). Omit to list all schemas.
        command_path: Vec<String>,
    },

    /// Manage authentication and site credentials.
    Auth {
        #[command(subcommand)]
        command: AuthCommands,
    },

    /// Show wpx version, configured sites, and capabilities.
    Info,

    /// Show version information.
    Version,

    /// Generate shell completions.
    Completions {
        /// Shell to generate completions for.
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },
}

/// MCP server subcommands.
#[derive(Debug, Subcommand)]
pub enum McpCommands {
    /// Start the MCP server.
    Serve {
        /// Transport type: "stdio" or "sse".
        #[arg(long, default_value = "stdio")]
        transport: String,
        /// Port for SSE transport.
        #[arg(long, default_value = "3000")]
        port: u16,
    },
}

/// Auth subcommands.
#[derive(Debug, Subcommand)]
pub enum AuthCommands {
    /// Set credentials for a site.
    Set {
        /// Username for authentication.
        #[arg(long)]
        username: String,
        /// Application password.
        #[arg(long)]
        password: String,
    },
    /// Test authentication against a site.
    Test,
    /// List configured sites.
    List,
    /// Remove credentials for a site.
    Logout,
    /// Authenticate via OAuth 2.1 (authorization code + PKCE).
    Oauth {
        /// OAuth client ID.
        #[arg(long)]
        client_id: String,
        /// Authorization endpoint URL.
        #[arg(long)]
        authorize_url: String,
        /// Token endpoint URL.
        #[arg(long)]
        token_url: String,
    },
}
