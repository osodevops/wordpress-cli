use serde_json::{json, Value};

/// A registered MCP tool definition.
pub struct ToolDef {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

/// Generate MCP tool definitions from the wpx schema registry.
///
/// Converts command paths like "post list" to tool names like "wpx_post_list".
pub fn generate_tools() -> Vec<ToolDef> {
    // Define tools matching the schema registry
    let entries = vec![
        (
            "post list",
            "List WordPress posts with optional filters",
            json!({"type":"object","properties":{"status":{"type":"string"},"search":{"type":"string"},"author":{"type":"integer"},"per_page":{"type":"integer","default":10},"page":{"type":"integer"},"orderby":{"type":"string"},"order":{"type":"string"}}}),
        ),
        (
            "post get",
            "Get a single post by ID",
            json!({"type":"object","properties":{"id":{"type":"integer"}},"required":["id"]}),
        ),
        (
            "post create",
            "Create a new post",
            json!({"type":"object","properties":{"title":{"type":"string"},"content":{"type":"string"},"status":{"type":"string"},"author":{"type":"integer"}}}),
        ),
        (
            "post update",
            "Update an existing post",
            json!({"type":"object","properties":{"id":{"type":"integer"},"title":{"type":"string"},"content":{"type":"string"},"status":{"type":"string"}},"required":["id"]}),
        ),
        (
            "post delete",
            "Delete or trash a post",
            json!({"type":"object","properties":{"id":{"type":"integer"},"force":{"type":"boolean"}},"required":["id"]}),
        ),
        (
            "page list",
            "List pages",
            json!({"type":"object","properties":{"status":{"type":"string"},"per_page":{"type":"integer"}}}),
        ),
        (
            "page get",
            "Get a page by ID",
            json!({"type":"object","properties":{"id":{"type":"integer"}},"required":["id"]}),
        ),
        (
            "page create",
            "Create a page",
            json!({"type":"object","properties":{"title":{"type":"string"},"content":{"type":"string"},"status":{"type":"string"}}}),
        ),
        (
            "media list",
            "List media attachments",
            json!({"type":"object","properties":{"media_type":{"type":"string"},"per_page":{"type":"integer"}}}),
        ),
        (
            "media get",
            "Get a media item",
            json!({"type":"object","properties":{"id":{"type":"integer"}},"required":["id"]}),
        ),
        (
            "user list",
            "List users",
            json!({"type":"object","properties":{"roles":{"type":"string"},"per_page":{"type":"integer"}}}),
        ),
        (
            "user get",
            "Get a user by ID",
            json!({"type":"object","properties":{"id":{"type":"integer"}},"required":["id"]}),
        ),
        (
            "user me",
            "Get the authenticated user",
            json!({"type":"object","properties":{}}),
        ),
        (
            "comment list",
            "List comments",
            json!({"type":"object","properties":{"post":{"type":"integer"},"status":{"type":"string"}}}),
        ),
        (
            "category list",
            "List categories",
            json!({"type":"object","properties":{"search":{"type":"string"}}}),
        ),
        (
            "tag list",
            "List tags",
            json!({"type":"object","properties":{"search":{"type":"string"}}}),
        ),
        (
            "taxonomy list",
            "List taxonomies",
            json!({"type":"object","properties":{}}),
        ),
        (
            "plugin list",
            "List installed plugins",
            json!({"type":"object","properties":{"status":{"type":"string"}}}),
        ),
        (
            "plugin install",
            "Install a plugin",
            json!({"type":"object","properties":{"slug":{"type":"string"},"activate":{"type":"boolean"}},"required":["slug"]}),
        ),
        (
            "plugin activate",
            "Activate a plugin",
            json!({"type":"object","properties":{"slug":{"type":"string"}},"required":["slug"]}),
        ),
        (
            "plugin deactivate",
            "Deactivate a plugin",
            json!({"type":"object","properties":{"slug":{"type":"string"}},"required":["slug"]}),
        ),
        (
            "theme list",
            "List installed themes",
            json!({"type":"object","properties":{"status":{"type":"string"}}}),
        ),
        (
            "theme activate",
            "Activate a theme",
            json!({"type":"object","properties":{"slug":{"type":"string"}},"required":["slug"]}),
        ),
        (
            "search",
            "Global search across content",
            json!({"type":"object","properties":{"query":{"type":"string"},"type":{"type":"string"},"subtype":{"type":"string"}},"required":["query"]}),
        ),
        (
            "settings list",
            "List site settings",
            json!({"type":"object","properties":{}}),
        ),
        (
            "settings get",
            "Get a setting",
            json!({"type":"object","properties":{"key":{"type":"string"}},"required":["key"]}),
        ),
        (
            "settings set",
            "Update a setting",
            json!({"type":"object","properties":{"key":{"type":"string"},"value":{}},"required":["key","value"]}),
        ),
        (
            "post-type list",
            "List registered post types",
            json!({"type":"object","properties":{}}),
        ),
        (
            "post-status list",
            "List registered post statuses",
            json!({"type":"object","properties":{}}),
        ),
        (
            "block list",
            "List reusable blocks",
            json!({"type":"object","properties":{"per_page":{"type":"integer"}}}),
        ),
        (
            "block get",
            "Get a reusable block",
            json!({"type":"object","properties":{"id":{"type":"integer"}},"required":["id"]}),
        ),
        (
            "menu list",
            "List navigation menus",
            json!({"type":"object","properties":{}}),
        ),
        (
            "menu-item list",
            "List menu items",
            json!({"type":"object","properties":{"menus":{"type":"integer"}}}),
        ),
        (
            "widget list",
            "List widgets",
            json!({"type":"object","properties":{"sidebar":{"type":"string"}}}),
        ),
        (
            "sidebar list",
            "List sidebars",
            json!({"type":"object","properties":{}}),
        ),
        (
            "discover",
            "Discover site capabilities",
            json!({"type":"object","properties":{}}),
        ),
    ];

    entries
        .into_iter()
        .map(|(cmd, desc, schema)| ToolDef {
            name: format!("wpx_{}", cmd.replace(' ', "_").replace('-', "_")),
            description: desc.to_string(),
            input_schema: schema,
        })
        .collect()
}

/// Convert an MCP tool name back to a command path.
/// "wpx_post_list" -> ["post", "list"]
pub fn tool_name_to_command_path(name: &str) -> Vec<String> {
    let stripped = name.strip_prefix("wpx_").unwrap_or(name);
    // Handle special cases with hyphens
    let normalized = stripped
        .replace("post_type", "post-type")
        .replace("post_status", "post-status")
        .replace("menu_item", "menu-item")
        .replace("search_replace", "search-replace")
        .replace("cron_event", "cron event")
        .replace("cron_schedule", "cron schedule");

    normalized
        .split('_')
        .map(|s| {
            // Re-split on spaces for nested commands
            s.to_string()
        })
        .flat_map(|s| {
            if s.contains(' ') {
                s.split(' ').map(|p| p.to_string()).collect::<Vec<_>>()
            } else {
                vec![s]
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_generation() {
        let tools = generate_tools();
        assert!(tools.len() > 30);
        assert!(tools.iter().any(|t| t.name == "wpx_post_list"));
        assert!(tools.iter().any(|t| t.name == "wpx_plugin_install"));
        assert!(tools.iter().any(|t| t.name == "wpx_discover"));
        // Bridge commands should NOT be present
        assert!(!tools.iter().any(|t| t.name == "wpx_db_query"));
        assert!(!tools.iter().any(|t| t.name == "wpx_cache_flush"));
    }

    #[test]
    fn tool_name_conversion() {
        assert_eq!(
            tool_name_to_command_path("wpx_post_list"),
            vec!["post", "list"]
        );
        assert_eq!(
            tool_name_to_command_path("wpx_plugin_install"),
            vec!["plugin", "install"]
        );
        assert_eq!(tool_name_to_command_path("wpx_discover"), vec!["discover"]);
    }
}
