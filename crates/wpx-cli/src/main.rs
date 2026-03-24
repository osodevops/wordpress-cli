mod cli;
mod commands;
mod context;
mod crud;
pub mod dispatch;

use clap::{CommandFactory, Parser};
use cli::{Cli, Commands};
use wpx_core::WpxError;
use wpx_output::{render_with_config, OutputConfig, RenderPayload};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    init_tracing(cli.global.verbose, cli.global.quiet);

    let result = run(&cli).await;

    match result {
        Ok(payload) => {
            if !payload.data.is_null() {
                let config = OutputConfig {
                    format: cli.global.output,
                    fields: cli.global.fields.clone(),
                };
                if let Err(e) = render_with_config(&payload, &config) {
                    print_error(&e);
                    std::process::exit(e.exit_code().as_u8() as i32);
                }
            }
        }
        Err(e) => {
            print_error(&e);
            std::process::exit(e.exit_code().as_u8() as i32);
        }
    }
}

async fn run(cli: &Cli) -> Result<RenderPayload, WpxError> {
    match &cli.command {
        // Resource commands (require API client)
        Commands::Post { command } => {
            let client = context::build_client(&cli.global)?;
            commands::post::handle(command, &client, cli.global.dry_run, cli.global.all_pages).await
        }
        Commands::Page { command } => {
            let client = context::build_client(&cli.global)?;
            commands::page::handle(command, &client, cli.global.dry_run).await
        }
        Commands::Media { command } => {
            let client = context::build_client(&cli.global)?;
            commands::media::handle(command, &client, cli.global.dry_run).await
        }
        Commands::User { command } => {
            let client = context::build_client(&cli.global)?;
            commands::user::handle(command, &client, cli.global.dry_run).await
        }
        Commands::Comment { command } => {
            let client = context::build_client(&cli.global)?;
            commands::comment::handle(command, &client, cli.global.dry_run).await
        }
        Commands::Category { command } => {
            let client = context::build_client(&cli.global)?;
            commands::category::handle(command, &client, cli.global.dry_run).await
        }
        Commands::Tag { command } => {
            let client = context::build_client(&cli.global)?;
            commands::tag::handle(command, &client, cli.global.dry_run).await
        }
        Commands::Taxonomy { command } => {
            let client = context::build_client(&cli.global)?;
            commands::taxonomy::handle(command, &client).await
        }
        Commands::Plugin { command } => {
            let client = context::build_client(&cli.global)?;
            commands::plugin::handle(command, &client, cli.global.dry_run).await
        }
        Commands::Theme { command } => {
            let client = context::build_client(&cli.global)?;
            commands::theme::handle(command, &client, cli.global.dry_run).await
        }
        Commands::PostType { command } => {
            let client = context::build_client(&cli.global)?;
            commands::post_type::handle(command, &client).await
        }
        Commands::PostStatus { command } => {
            let client = context::build_client(&cli.global)?;
            commands::post_status::handle(command, &client).await
        }
        Commands::BlockType { command } => {
            let client = context::build_client(&cli.global)?;
            commands::block_type::handle(command, &client).await
        }
        Commands::BlockPattern { command } => {
            let client = context::build_client(&cli.global)?;
            commands::block_pattern::handle(command, &client).await
        }
        Commands::BlockPatternCategory { command } => {
            let client = context::build_client(&cli.global)?;
            commands::block_pattern_category::handle(command, &client).await
        }
        Commands::WidgetType { command } => {
            let client = context::build_client(&cli.global)?;
            commands::widget_type::handle(command, &client).await
        }
        Commands::Sidebar { command } => {
            let client = context::build_client(&cli.global)?;
            commands::sidebar::handle(command, &client).await
        }
        Commands::MenuLocation { command } => {
            let client = context::build_client(&cli.global)?;
            commands::menu_location::handle(command, &client).await
        }
        Commands::Block { command } => {
            let client = context::build_client(&cli.global)?;
            commands::block::handle(command, &client, cli.global.dry_run).await
        }
        Commands::Widget { command } => {
            let client = context::build_client(&cli.global)?;
            commands::widget::handle(command, &client, cli.global.dry_run).await
        }
        Commands::Menu { command } => {
            let client = context::build_client(&cli.global)?;
            commands::menu::handle(command, &client, cli.global.dry_run).await
        }
        Commands::MenuItem { command } => {
            let client = context::build_client(&cli.global)?;
            commands::menu_item::handle(command, &client, cli.global.dry_run).await
        }

        Commands::Search { query, args } => {
            let client = context::build_client(&cli.global)?;
            commands::search::handle(query, args, &client).await
        }
        Commands::Settings { command } | Commands::Option { command } => {
            let client = context::build_client(&cli.global)?;
            commands::settings::handle(command, &client, cli.global.dry_run).await
        }

        // Fleet management
        Commands::Fleet { command } => {
            commands::fleet::handle(command, cli.global.dry_run, cli.global.timeout).await
        }

        // MCP server
        Commands::Mcp { command } => {
            match command {
                cli::McpCommands::Serve { transport, port: _ } => {
                    if transport != "stdio" {
                        return Err(WpxError::Other(
                            "Only 'stdio' transport is currently supported. SSE/HTTP transport coming soon.".into()
                        ));
                    }
                    wpx_mcp::serve_stdio(&cli.global.site).await?;
                    Ok(RenderPayload {
                        data: serde_json::Value::Null,
                        summary: None,
                    })
                }
            }
        }

        // Discover site capabilities (no auth needed)
        Commands::Discover { url } => {
            let base_url = url::Url::parse(url).map_err(|e| WpxError::Config {
                message: format!("Invalid URL: {e}"),
            })?;
            let client =
                wpx_api::WpClient::new(base_url, Box::new(wpx_auth::NoAuth), cli.global.timeout, 0)?;
            commands::discover::handle(&client).await
        }

        // Schema introspection (no API client needed)
        Commands::Schema { command_path } => commands::schema::handle_schema(command_path),

        // Auth commands
        Commands::Auth { command } => {
            commands::auth::handle(command, &cli.global.site, cli.global.url.as_deref()).await
        }

        // Meta commands
        Commands::Info => {
            let config = wpx_config::WpxConfig::load();
            let sites: Vec<String> = config.sites.keys().cloned().collect();
            Ok(RenderPayload {
                data: serde_json::json!({
                    "version": env!("CARGO_PKG_VERSION"),
                    "name": "wpx",
                    "description": "Rust-native WordPress CLI for agents and humans",
                    "configured_sites": sites,
                }),
                summary: None,
            })
        }
        Commands::Version => Ok(RenderPayload {
            data: serde_json::json!({ "version": env!("CARGO_PKG_VERSION") }),
            summary: None,
        }),
        Commands::Completions { shell } => {
            let mut cmd = Cli::command();
            clap_complete::generate(*shell, &mut cmd, "wpx", &mut std::io::stdout());
            Ok(RenderPayload {
                data: serde_json::Value::Null,
                summary: None,
            })
        }
    }
}

fn init_tracing(verbose: bool, quiet: bool) {
    use tracing_subscriber::EnvFilter;
    let filter = if verbose {
        EnvFilter::new("debug")
    } else if quiet {
        EnvFilter::new("error")
    } else {
        EnvFilter::new("warn")
    };
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .with_target(false)
        .init();
}

fn print_error(e: &WpxError) {
    let error_json = e.to_error_json();
    if let Ok(formatted) = serde_json::to_string_pretty(&error_json) {
        eprintln!("{formatted}");
    } else {
        eprintln!("Error: {e}");
    }
}
