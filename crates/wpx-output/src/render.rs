use crate::fields::apply_field_mask;
use crate::format::OutputFormat;
use serde_json::Value;
use std::io::Write;
use wpx_core::WpxError;

/// A payload to be rendered to stdout.
pub struct RenderPayload {
    /// The data to render (serialized to serde_json::Value).
    pub data: Value,
    /// Optional summary line, shown only in table mode.
    pub summary: Option<String>,
}

/// Output configuration derived from global flags.
pub struct OutputConfig {
    pub format: OutputFormat,
    pub fields: Option<Vec<String>>,
}

/// Render a payload to stdout in the specified format.
pub fn render(payload: &RenderPayload, format: OutputFormat) -> Result<(), WpxError> {
    render_with_config(
        payload,
        &OutputConfig {
            format,
            fields: None,
        },
    )
}

/// Render a payload with full output configuration (format + field mask).
pub fn render_with_config(payload: &RenderPayload, config: &OutputConfig) -> Result<(), WpxError> {
    let format = config.format.resolve();

    // Apply field mask if specified
    let data = if let Some(fields) = &config.fields {
        apply_field_mask(payload.data.clone(), fields)
    } else {
        payload.data.clone()
    };

    let stdout = std::io::stdout();
    let mut out = stdout.lock();

    match format {
        OutputFormat::Json | OutputFormat::Auto => {
            render_json(&data, &mut out)?;
        }
        OutputFormat::Table => {
            render_table(&data, &mut out)?;
            if let Some(summary) = &payload.summary {
                writeln!(out).map_err(io_err)?;
                writeln!(out, "  {summary}").map_err(io_err)?;
            }
        }
        OutputFormat::Csv => {
            render_csv(&data, &mut out)?;
        }
        OutputFormat::Yaml => {
            render_yaml(&data, &mut out)?;
        }
        OutputFormat::Ndjson => {
            render_ndjson(&data, &mut out)?;
        }
    }

    Ok(())
}

fn render_json(data: &Value, out: &mut impl Write) -> Result<(), WpxError> {
    serde_json::to_writer_pretty(&mut *out, data).map_err(|e| WpxError::Other(e.to_string()))?;
    writeln!(out).map_err(io_err)?;
    Ok(())
}

fn render_table(data: &Value, out: &mut impl Write) -> Result<(), WpxError> {
    match data {
        Value::Array(arr) if !arr.is_empty() => {
            // Extract column headers from the first object
            let headers: Vec<String> = if let Some(Value::Object(first)) = arr.first() {
                first.keys().cloned().collect()
            } else {
                // Non-object array: render as single-column table
                writeln!(out, "  VALUE").map_err(io_err)?;
                for item in arr {
                    writeln!(out, "  {}", format_cell(item)).map_err(io_err)?;
                }
                return Ok(());
            };

            // Calculate column widths
            let mut widths: Vec<usize> = headers.iter().map(|h| h.len()).collect();
            let rows: Vec<Vec<String>> = arr
                .iter()
                .filter_map(|v| v.as_object())
                .map(|obj| {
                    headers
                        .iter()
                        .enumerate()
                        .map(|(i, h)| {
                            let cell = format_cell(obj.get(h).unwrap_or(&Value::Null));
                            widths[i] = widths[i].max(cell.len());
                            cell
                        })
                        .collect()
                })
                .collect();

            // Print header
            write!(out, " ").map_err(io_err)?;
            for (i, header) in headers.iter().enumerate() {
                write!(
                    out,
                    " {:width$}",
                    header.to_uppercase(),
                    width = widths[i] + 1
                )
                .map_err(io_err)?;
            }
            writeln!(out).map_err(io_err)?;

            // Print rows
            for row in &rows {
                write!(out, " ").map_err(io_err)?;
                for (i, cell) in row.iter().enumerate() {
                    write!(out, " {:width$}", cell, width = widths[i] + 1).map_err(io_err)?;
                }
                writeln!(out).map_err(io_err)?;
            }
        }
        Value::Object(_) => {
            // Single object: render as key-value pairs
            if let Some(obj) = data.as_object() {
                let key_width = obj.keys().map(|k| k.len()).max().unwrap_or(0);
                for (key, value) in obj {
                    writeln!(
                        out,
                        "  {:width$}  {}",
                        key,
                        format_cell(value),
                        width = key_width
                    )
                    .map_err(io_err)?;
                }
            }
        }
        _ => {
            // Scalar or empty: just print as JSON
            render_json(data, out)?;
        }
    }
    Ok(())
}

fn render_csv(data: &Value, out: &mut impl Write) -> Result<(), WpxError> {
    match data {
        Value::Array(arr) if !arr.is_empty() => {
            // Extract headers from first object
            let headers: Vec<String> = if let Some(Value::Object(first)) = arr.first() {
                first.keys().cloned().collect()
            } else {
                return render_json(data, out);
            };

            // Write header row
            writeln!(out, "{}", headers.join(",")).map_err(io_err)?;

            // Write data rows
            for item in arr {
                if let Some(obj) = item.as_object() {
                    let row: Vec<String> = headers
                        .iter()
                        .map(|h| csv_escape(&format_cell(obj.get(h).unwrap_or(&Value::Null))))
                        .collect();
                    writeln!(out, "{}", row.join(",")).map_err(io_err)?;
                }
            }
        }
        _ => render_json(data, out)?,
    }
    Ok(())
}

fn render_yaml(data: &Value, out: &mut impl Write) -> Result<(), WpxError> {
    let yaml = serde_yaml::to_string(data).map_err(|e| WpxError::Other(e.to_string()))?;
    write!(out, "{yaml}").map_err(io_err)?;
    Ok(())
}

fn render_ndjson(data: &Value, out: &mut impl Write) -> Result<(), WpxError> {
    match data {
        Value::Array(arr) => {
            for item in arr {
                serde_json::to_writer(&mut *out, item)
                    .map_err(|e| WpxError::Other(e.to_string()))?;
                writeln!(out).map_err(io_err)?;
            }
        }
        _ => {
            serde_json::to_writer(&mut *out, data).map_err(|e| WpxError::Other(e.to_string()))?;
            writeln!(out).map_err(io_err)?;
        }
    }
    Ok(())
}

/// Format a JSON value as a table cell string.
fn format_cell(value: &Value) -> String {
    match value {
        Value::Null => "—".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.clone(),
        Value::Object(obj) => {
            // For rendered content objects like {rendered: "...", raw: "..."}
            if let Some(rendered) = obj.get("rendered") {
                return format_cell(rendered);
            }
            serde_json::to_string(value).unwrap_or_default()
        }
        Value::Array(_) => serde_json::to_string(value).unwrap_or_default(),
    }
}

/// Escape a value for CSV output.
fn csv_escape(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

fn io_err(e: std::io::Error) -> WpxError {
    WpxError::Other(e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn render_json_output() {
        let data = json!({"id": 1, "title": "Hello"});
        let mut buf = Vec::new();
        render_json(&data, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("\"id\": 1"));
        assert!(output.contains("\"title\": \"Hello\""));
    }

    #[test]
    fn render_csv_output() {
        let data = json!([
            {"id": 1, "title": "Hello"},
            {"id": 2, "title": "World"},
        ]);
        let mut buf = Vec::new();
        render_csv(&data, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        let lines: Vec<&str> = output.trim().lines().collect();
        assert_eq!(lines[0], "id,title");
        assert_eq!(lines[1], "1,Hello");
        assert_eq!(lines[2], "2,World");
    }

    #[test]
    fn render_ndjson_output() {
        let data = json!([
            {"id": 1, "title": "Hello"},
            {"id": 2, "title": "World"},
        ]);
        let mut buf = Vec::new();
        render_ndjson(&data, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        let lines: Vec<&str> = output.trim().lines().collect();
        assert_eq!(lines.len(), 2);
        // Each line should be valid JSON
        for line in &lines {
            serde_json::from_str::<Value>(line).unwrap();
        }
    }

    #[test]
    fn render_table_output() {
        let data = json!([
            {"id": 1, "name": "Alice"},
            {"id": 2, "name": "Bob"},
        ]);
        let mut buf = Vec::new();
        render_table(&data, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("ID"));
        assert!(output.contains("NAME"));
        assert!(output.contains("Alice"));
        assert!(output.contains("Bob"));
    }

    #[test]
    fn render_single_object_as_table() {
        let data = json!({"id": 1, "name": "Alice", "status": "active"});
        let mut buf = Vec::new();
        render_table(&data, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("id"));
        assert!(output.contains("Alice"));
    }

    #[test]
    fn format_cell_rendered_content() {
        let value = json!({"rendered": "<p>Hello</p>", "raw": "Hello"});
        assert_eq!(format_cell(&value), "<p>Hello</p>");
    }

    #[test]
    fn csv_escape_with_commas() {
        assert_eq!(csv_escape("hello,world"), "\"hello,world\"");
        assert_eq!(csv_escape("simple"), "simple");
    }

    #[test]
    fn render_with_field_mask() {
        let payload = RenderPayload {
            data: json!([
                {"id": 1, "title": "Hello", "status": "publish"},
                {"id": 2, "title": "World", "status": "draft"},
            ]),
            summary: None,
        };
        let config = OutputConfig {
            format: OutputFormat::Json,
            fields: Some(vec!["id".into(), "title".into()]),
        };
        let mut buf = Vec::new();

        // We can't easily capture stdout, but let's test the field mask directly
        let filtered = apply_field_mask(payload.data.clone(), &["id".into(), "title".into()]);
        render_json(&filtered, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("\"id\""));
        assert!(output.contains("\"title\""));
        assert!(!output.contains("\"status\""));
    }
}
