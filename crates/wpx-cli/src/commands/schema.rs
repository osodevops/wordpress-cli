use serde_json::json;
use wpx_core::WpxError;
use wpx_output::RenderPayload;

/// Static schema registry. Each entry maps a command path to its JSON Schema.
struct SchemaEntry {
    command: &'static str,
    description: &'static str,
    input: serde_json::Value,
    output: serde_json::Value,
}

fn exit_codes() -> serde_json::Value {
    json!({
        "0": "Success",
        "1": "General error",
        "2": "Invalid arguments",
        "3": "Authentication failure",
        "4": "Resource not found",
        "5": "Permission denied",
        "6": "Rate limited",
        "7": "Network error",
        "8": "Server error (5xx)",
        "9": "Conflict",
        "10": "Validation error"
    })
}

fn schemas() -> Vec<SchemaEntry> {
    vec![
        SchemaEntry {
            command: "post list",
            description: "List posts with optional filters (any post type via `type`)",
            input: json!({
                "type": "object",
                "properties": {
                    "type": { "type": "string", "description": "Post type slug; resolved via wp/v2/types/{slug} to its REST base (default: post)" },
                    "rest_base": { "type": "string", "description": "REST collection path to use instead of resolving `type` (e.g. blog, wc/v3/products)" },
                    "status": { "type": "string", "enum": ["publish", "draft", "pending", "private", "future", "trash"] },
                    "slug": { "type": "string" },
                    "context": { "type": "string", "enum": ["view", "edit", "embed"] },
                    "search": { "type": "string" },
                    "author": { "type": "integer" },
                    "per_page": { "type": "integer", "minimum": 1, "maximum": 100, "default": 10 },
                    "page": { "type": "integer", "minimum": 1 },
                    "orderby": { "type": "string", "enum": ["date", "id", "title", "slug", "modified", "relevance"] },
                    "order": { "type": "string", "enum": ["asc", "desc"], "default": "desc" }
                }
            }),
            output: json!({ "type": "array", "items": { "type": "object", "properties": {
                "id": { "type": "integer" }, "title": { "type": "object" }, "status": { "type": "string" },
                "date": { "type": "string" }, "author": { "type": "integer" }, "link": { "type": "string" }
            }}}),
        },
        SchemaEntry {
            command: "post get",
            description: "Get a single post by ID (any post type via `type`; `context: edit` returns content.raw)",
            input: json!({ "type": "object", "properties": {
                "id": { "type": "integer" },
                "type": { "type": "string" },
                "rest_base": { "type": "string" },
                "context": { "type": "string", "enum": ["view", "edit", "embed"] }
            }, "required": ["id"] }),
            output: json!({ "type": "object", "properties": {
                "id": { "type": "integer" }, "title": { "type": "object" }, "status": { "type": "string" },
                "content": { "type": "object" }, "date": { "type": "string" }
            }}),
        },
        SchemaEntry {
            command: "post create",
            description: "Create a new post (any post type via `type`; unknown keys pass through to WordPress)",
            input: json!({ "type": "object", "properties": {
                "type": { "type": "string", "description": "Post type slug (default: post)" },
                "rest_base": { "type": "string" },
                "title": { "type": "string" },
                "content": { "type": "string", "description": "HTML or block markup" },
                "content_file": { "type": "string", "description": "CLI only: read content from a file, '-' for stdin" },
                "status": { "type": "string", "enum": ["publish", "draft", "pending", "private", "future"] },
                "author": { "type": "integer" }, "excerpt": { "type": "string" }, "slug": { "type": "string" },
                "template": { "type": "string", "description": "Template file name, e.g. template-landing.php" },
                "featured_media": { "type": "integer", "description": "Attachment ID" },
                "meta": { "type": "object", "description": "Post meta (keys registered with show_in_rest)" },
                "acf": { "type": "object", "description": "ACF fields (field groups exposed with show_in_rest)" }
            }, "additionalProperties": { "description": "Custom taxonomies (e.g. blog_category: [9]) and other REST fields are passed through verbatim" } }),
            output: json!({ "type": "object" }),
        },
        SchemaEntry {
            command: "post update",
            description: "Update an existing post (any post type via `type`; same body as post create)",
            input: json!({ "type": "object", "properties": {
                "id": { "type": "integer" }, "type": { "type": "string" }, "rest_base": { "type": "string" },
                "title": { "type": "string" }, "content": { "type": "string" }, "content_file": { "type": "string" },
                "status": { "type": "string" }, "template": { "type": "string" }, "featured_media": { "type": "integer" },
                "meta": { "type": "object" }, "acf": { "type": "object" }
            }, "required": ["id"], "additionalProperties": true }),
            output: json!({ "type": "object" }),
        },
        SchemaEntry {
            command: "post delete",
            description: "Delete or trash a post (any post type via `type`)",
            input: json!({ "type": "object", "properties": {
                "id": { "type": "integer" }, "type": { "type": "string" }, "rest_base": { "type": "string" },
                "force": { "type": "boolean", "default": false }
            }, "required": ["id"] }),
            output: json!({ "type": "object" }),
        },
        SchemaEntry {
            command: "page list",
            description: "List pages",
            input: json!({"type":"object","properties":{"status":{"type":"string"},"slug":{"type":"string"},"per_page":{"type":"integer"},"context":{"type":"string","enum":["view","edit","embed"]}}}),
            output: json!({"type":"array"}),
        },
        SchemaEntry {
            command: "page get",
            description: "Get a page by ID (`context: edit` returns content.raw)",
            input: json!({"type":"object","properties":{"id":{"type":"integer"},"context":{"type":"string","enum":["view","edit","embed"]}},"required":["id"]}),
            output: json!({"type":"object"}),
        },
        SchemaEntry {
            command: "page create",
            description: "Create a page (unknown keys such as acf/meta pass through to WordPress)",
            input: json!({"type":"object","properties":{
                "title":{"type":"string"},"content":{"type":"string"},
                "content_file":{"type":"string","description":"CLI only: read content from a file, '-' for stdin"},
                "status":{"type":"string"},"slug":{"type":"string"},"parent":{"type":"integer"},"menu_order":{"type":"integer"},
                "template":{"type":"string"},"featured_media":{"type":"integer"},
                "meta":{"type":"object"},"acf":{"type":"object"}
            },"additionalProperties":true}),
            output: json!({"type":"object"}),
        },
        SchemaEntry {
            command: "page update",
            description: "Update a page (same body as page create)",
            input: json!({"type":"object","properties":{
                "id":{"type":"integer"},"title":{"type":"string"},"content":{"type":"string"},"content_file":{"type":"string"},
                "status":{"type":"string"},"template":{"type":"string"},"featured_media":{"type":"integer"},
                "meta":{"type":"object"},"acf":{"type":"object"}
            },"required":["id"],"additionalProperties":true}),
            output: json!({"type":"object"}),
        },
        SchemaEntry {
            command: "media list",
            description: "List media attachments",
            input: json!({"type":"object","properties":{"media_type":{"type":"string"},"per_page":{"type":"integer"}}}),
            output: json!({"type":"array"}),
        },
        SchemaEntry {
            command: "media get",
            description: "Get a media item by ID",
            input: json!({"type":"object","properties":{"id":{"type":"integer"}},"required":["id"]}),
            output: json!({"type":"object"}),
        },
        SchemaEntry {
            command: "media upload",
            description: "Upload a local file to the media library (multipart POST wp/v2/media)",
            input: json!({"type":"object","properties":{
                "file":{"type":"string","description":"Local file path"},
                "title":{"type":"string"},"alt_text":{"type":"string"},"caption":{"type":"string"},
                "description":{"type":"string"},"post":{"type":"integer","description":"Attach to post ID"},
                "mime_type":{"type":"string","description":"Override the MIME type guessed from the extension"}
            },"required":["file"]}),
            output: json!({"type":"object","properties":{"id":{"type":"integer"},"source_url":{"type":"string"},"mime_type":{"type":"string"}}}),
        },
        SchemaEntry {
            command: "api",
            description: "Call any REST route under /wp-json/ (escape hatch for routes without a typed command)",
            input: json!({"type":"object","properties":{
                "method":{"type":"string","enum":["GET","POST","PUT","PATCH","DELETE"],"default":"GET"},
                "path":{"type":"string","description":"Route relative to /wp-json/, e.g. wp/v2/types/blog or rankmath/v1/updateMeta"},
                "query":{"type":"object","description":"Query parameters (CLI: repeatable --query key=value)"},
                "body":{"description":"JSON request body (CLI: --data '<json>' or --json for stdin)"}
            },"required":["path"]}),
            output: json!({"description":"Raw JSON response from WordPress"}),
        },
        SchemaEntry {
            command: "user list",
            description: "List users",
            input: json!({"type":"object","properties":{"roles":{"type":"string"},"per_page":{"type":"integer"}}}),
            output: json!({"type":"array"}),
        },
        SchemaEntry {
            command: "user get",
            description: "Get a user by ID",
            input: json!({"type":"object","properties":{"id":{"type":"integer"}},"required":["id"]}),
            output: json!({"type":"object"}),
        },
        SchemaEntry {
            command: "user me",
            description: "Get the authenticated user",
            input: json!({"type":"object"}),
            output: json!({"type":"object"}),
        },
        SchemaEntry {
            command: "comment list",
            description: "List comments",
            input: json!({"type":"object","properties":{"post":{"type":"integer"},"status":{"type":"string"}}}),
            output: json!({"type":"array"}),
        },
        SchemaEntry {
            command: "category list",
            description: "List categories",
            input: json!({"type":"object","properties":{"search":{"type":"string"},"per_page":{"type":"integer"}}}),
            output: json!({"type":"array"}),
        },
        SchemaEntry {
            command: "tag list",
            description: "List tags",
            input: json!({"type":"object","properties":{"search":{"type":"string"},"per_page":{"type":"integer"}}}),
            output: json!({"type":"array"}),
        },
        SchemaEntry {
            command: "taxonomy list",
            description: "List registered taxonomies",
            input: json!({"type":"object"}),
            output: json!({"type":"array"}),
        },
        SchemaEntry {
            command: "plugin list",
            description: "List installed plugins",
            input: json!({"type":"object","properties":{"status":{"type":"string"}}}),
            output: json!({"type":"array"}),
        },
        SchemaEntry {
            command: "plugin install",
            description: "Install a plugin",
            input: json!({"type":"object","properties":{"slug":{"type":"string"},"activate":{"type":"boolean"}},"required":["slug"]}),
            output: json!({"type":"object"}),
        },
        SchemaEntry {
            command: "plugin activate",
            description: "Activate a plugin",
            input: json!({"type":"object","properties":{"slug":{"type":"string"}},"required":["slug"]}),
            output: json!({"type":"object"}),
        },
        SchemaEntry {
            command: "theme list",
            description: "List installed themes",
            input: json!({"type":"object","properties":{"status":{"type":"string"}}}),
            output: json!({"type":"array"}),
        },
        SchemaEntry {
            command: "theme activate",
            description: "Activate a theme",
            input: json!({"type":"object","properties":{"slug":{"type":"string"}},"required":["slug"]}),
            output: json!({"type":"object"}),
        },
        SchemaEntry {
            command: "auth test",
            description: "Test authentication",
            input: json!({"type":"object"}),
            output: json!({"type":"object"}),
        },
        SchemaEntry {
            command: "auth list",
            description: "List configured sites",
            input: json!({"type":"object"}),
            output: json!({"type":"array"}),
        },
    ]
}

pub fn handle_schema(command_path: &[String]) -> Result<RenderPayload, WpxError> {
    let all = schemas();

    if command_path.is_empty() {
        // List all schemas
        let listing: Vec<serde_json::Value> = all
            .iter()
            .map(|s| json!({ "command": s.command, "description": s.description }))
            .collect();
        return Ok(RenderPayload {
            data: json!(listing),
            summary: Some(format!("{} commands available", listing.len())),
        });
    }

    let path = command_path.join(" ");
    let entry = all.iter().find(|s| s.command == path);

    match entry {
        Some(schema) => Ok(RenderPayload {
            data: json!({
                "command": schema.command,
                "description": schema.description,
                "input": schema.input,
                "output": schema.output,
                "exit_codes": exit_codes(),
            }),
            summary: None,
        }),
        None => Err(WpxError::NotFound {
            resource: "schema".into(),
            id: path,
        }),
    }
}
