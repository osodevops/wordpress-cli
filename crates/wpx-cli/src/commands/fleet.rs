use clap::Subcommand;
use serde_json::json;
use std::time::Instant;
use wpx_api::WpClient;
use wpx_auth::{ApplicationPasswordAuth, AuthProvider, NoAuth, OAuthAuth};
use wpx_config::{CredentialStore, WpxConfig};
use wpx_core::WpxError;
use wpx_output::RenderPayload;

#[derive(Debug, Subcommand)]
pub enum FleetCommands {
    /// Execute a command across multiple sites.
    Exec {
        /// Command to execute (e.g., "plugin list").
        command: String,
        /// Comma-separated site names or glob pattern.
        #[arg(long)]
        sites: Option<String>,
        /// Maximum concurrent executions.
        #[arg(long, default_value = "5")]
        concurrency: usize,
    },
    /// Show status of all configured sites.
    Status,
}

pub async fn handle(
    command: &FleetCommands,
    dry_run: bool,
    timeout: u64,
) -> Result<RenderPayload, WpxError> {
    match command {
        FleetCommands::Exec {
            command: cmd,
            sites,
            concurrency,
        } => handle_exec(cmd, sites.as_deref(), *concurrency, dry_run, timeout).await,
        FleetCommands::Status => handle_status(timeout).await,
    }
}

async fn handle_exec(
    command_str: &str,
    sites_filter: Option<&str>,
    concurrency: usize,
    dry_run: bool,
    timeout: u64,
) -> Result<RenderPayload, WpxError> {
    let config = WpxConfig::load();
    let _store = CredentialStore::load();

    // Resolve which sites to target
    let site_names: Vec<String> = match sites_filter {
        Some(filter) => {
            let patterns: Vec<&str> = filter.split(',').map(|s| s.trim()).collect();
            config
                .sites
                .keys()
                .filter(|name| {
                    patterns.iter().any(|pattern| {
                        if pattern.contains('*') {
                            glob_match(pattern, name)
                        } else {
                            *name == pattern
                        }
                    })
                })
                .cloned()
                .collect()
        }
        None => config.sites.keys().cloned().collect(),
    };

    if site_names.is_empty() {
        return Err(WpxError::Config {
            message: "No matching sites found".into(),
        });
    }

    // Parse the command string into path segments
    let parts: Vec<&str> = command_str.split_whitespace().collect();
    if parts.is_empty() {
        return Err(WpxError::Validation {
            field: "command".into(),
            message: "Empty command string".into(),
        });
    }

    let total_start = Instant::now();
    let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(concurrency));
    let mut handles = Vec::new();

    for site_name in &site_names {
        let sem = semaphore.clone();
        let site_name = site_name.clone();
        let parts: Vec<String> = parts.iter().map(|s| s.to_string()).collect();
        let config = WpxConfig::load();
        let store = CredentialStore::load();
        let dry_run_flag = dry_run;
        let timeout_val = timeout;

        let handle = tokio::spawn(async move {
            let _permit = sem.acquire().await.unwrap();
            let start = Instant::now();

            let result = execute_on_site(
                &site_name,
                &parts,
                &config,
                &store,
                dry_run_flag,
                timeout_val,
            )
            .await;
            let duration = start.elapsed();

            let profile = config.get_site(&site_name);
            let url = profile.map(|p| p.url.as_str()).unwrap_or("unknown");

            match result {
                Ok(payload) => json!({
                    "site": site_name,
                    "url": url,
                    "status": "success",
                    "data": payload.data,
                    "duration_ms": duration.as_millis() as u64,
                }),
                Err(e) => json!({
                    "site": site_name,
                    "url": url,
                    "status": "error",
                    "error": e.to_string(),
                    "duration_ms": duration.as_millis() as u64,
                }),
            }
        });

        handles.push(handle);
    }

    // Collect results
    let mut results = Vec::new();
    for handle in handles {
        match handle.await {
            Ok(result) => results.push(result),
            Err(e) => results.push(json!({
                "status": "error",
                "error": format!("Task panicked: {e}"),
            })),
        }
    }

    let total_duration = total_start.elapsed();
    let succeeded = results
        .iter()
        .filter(|r| r.get("status").and_then(|s| s.as_str()) == Some("success"))
        .count();
    let failed = results.len() - succeeded;

    Ok(RenderPayload {
        data: json!({
            "results": results,
            "summary": {
                "total": results.len(),
                "succeeded": succeeded,
                "failed": failed,
                "total_duration_ms": total_duration.as_millis() as u64,
            }
        }),
        summary: Some(format!(
            "{} sites: {} succeeded, {} failed ({:.1}s)",
            results.len(),
            succeeded,
            failed,
            total_duration.as_secs_f64()
        )),
    })
}

async fn handle_status(timeout: u64) -> Result<RenderPayload, WpxError> {
    let config = WpxConfig::load();

    let mut results = Vec::new();
    for (name, profile) in &config.sites {
        let base_url = match url::Url::parse(&profile.url) {
            Ok(u) => u,
            Err(_) => {
                results.push(json!({
                    "site": name,
                    "url": profile.url,
                    "status": "error",
                    "error": "Invalid URL",
                }));
                continue;
            }
        };

        let client = match WpClient::new(base_url, Box::new(NoAuth), timeout, 0) {
            Ok(c) => c,
            Err(e) => {
                results.push(json!({
                    "site": name,
                    "url": profile.url,
                    "status": "error",
                    "error": e.to_string(),
                }));
                continue;
            }
        };

        let discovery = client.discover().await;
        results.push(json!({
            "site": name,
            "url": profile.url,
            "status": "ok",
            "capabilities": discovery,
        }));
    }

    Ok(RenderPayload {
        data: json!(results),
        summary: Some(format!("{} sites checked", results.len())),
    })
}

async fn execute_on_site(
    site_name: &str,
    parts: &[String],
    config: &WpxConfig,
    store: &CredentialStore,
    dry_run: bool,
    timeout: u64,
) -> Result<RenderPayload, WpxError> {
    let profile = config.get_site(site_name).ok_or_else(|| WpxError::Config {
        message: format!("Site '{site_name}' not found in config"),
    })?;

    let base_url = url::Url::parse(&profile.url).map_err(|e| WpxError::Config {
        message: format!("Invalid URL '{}': {e}", profile.url),
    })?;

    let auth: Box<dyn AuthProvider> = if let Some(creds) = store.get(site_name) {
        match creds.auth_type.as_str() {
            "oauth2" => {
                if let Some(token) = &creds.access_token {
                    Box::new(OAuthAuth::new(token.clone()))
                } else {
                    Box::new(NoAuth)
                }
            }
            _ => Box::new(ApplicationPasswordAuth::new(
                creds.username.clone(),
                creds.password.clone(),
            )),
        }
    } else {
        Box::new(NoAuth)
    };

    let client = WpClient::new(base_url, auth, timeout, 3)?;

    let path_refs: Vec<&str> = parts.iter().map(|s| s.as_str()).collect();
    crate::dispatch::dispatch(&path_refs, &json!({}), &client, dry_run).await
}

/// Simple glob matching supporting only `*` wildcards.
fn glob_match(pattern: &str, text: &str) -> bool {
    let pattern_parts: Vec<&str> = pattern.split('*').collect();
    if pattern_parts.len() == 1 {
        return pattern == text;
    }

    let mut pos = 0;
    for (i, part) in pattern_parts.iter().enumerate() {
        if part.is_empty() {
            continue;
        }
        match text[pos..].find(part) {
            Some(found) => {
                if i == 0 && found != 0 {
                    return false; // pattern doesn't start with *
                }
                pos += found + part.len();
            }
            None => return false,
        }
    }

    // If pattern ends with *, any remaining text is ok
    pattern.ends_with('*') || pos == text.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glob_matching() {
        assert!(glob_match("client-*", "client-abc"));
        assert!(glob_match("client-*", "client-"));
        assert!(!glob_match("client-*", "prod-abc"));
        assert!(glob_match("*", "anything"));
        assert!(glob_match("exact", "exact"));
        assert!(!glob_match("exact", "other"));
        assert!(glob_match("*-prod", "client-prod"));
        assert!(!glob_match("*-prod", "client-staging"));
    }
}
