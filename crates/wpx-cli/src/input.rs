//! Helpers for reading request bodies and content from files / stdin.

use wpx_core::WpxError;

/// Read a JSON payload from stdin and deserialize it.
pub fn json_from_stdin<T: serde::de::DeserializeOwned>() -> Result<T, WpxError> {
    let stdin = std::io::read_to_string(std::io::stdin())
        .map_err(|e| WpxError::Other(format!("Failed to read stdin: {e}")))?;
    serde_json::from_str(&stdin).map_err(|e| WpxError::Validation {
        field: "json".into(),
        message: format!("Invalid JSON input: {e}"),
    })
}

/// Resolve post content from `--content` or `--content-file`.
///
/// `--content-file -` reads stdin. Returns `Ok(None)` when neither was given.
/// `reading_stdin_json` guards against both `--json` and `--content-file -`
/// competing for stdin.
pub fn resolve_content(
    content: Option<&str>,
    content_file: Option<&str>,
    reading_stdin_json: bool,
) -> Result<Option<String>, WpxError> {
    match (content, content_file) {
        (Some(_), Some(_)) => Err(WpxError::Validation {
            field: "content".into(),
            message: "--content and --content-file are mutually exclusive".into(),
        }),
        (Some(c), None) => Ok(Some(c.to_string())),
        (None, Some("-")) => {
            if reading_stdin_json {
                return Err(WpxError::Validation {
                    field: "content_file".into(),
                    message: "--content-file - cannot be combined with --json (both read stdin)"
                        .into(),
                });
            }
            let text = std::io::read_to_string(std::io::stdin())
                .map_err(|e| WpxError::Other(format!("Failed to read stdin: {e}")))?;
            Ok(Some(text))
        }
        (None, Some(path)) => {
            let text = std::fs::read_to_string(path).map_err(|e| WpxError::Validation {
                field: "content_file".into(),
                message: format!("Cannot read '{path}': {e}"),
            })?;
            Ok(Some(text))
        }
        (None, None) => Ok(None),
    }
}

/// Parse repeated `key=value` arguments into query pairs.
pub fn parse_key_values(items: &[String]) -> Result<Vec<(String, String)>, WpxError> {
    items
        .iter()
        .map(|item| {
            let (k, v) = item.split_once('=').ok_or_else(|| WpxError::Validation {
                field: "query".into(),
                message: format!("Expected key=value, got '{item}'"),
            })?;
            if k.trim().is_empty() {
                return Err(WpxError::Validation {
                    field: "query".into(),
                    message: format!("Empty key in '{item}'"),
                });
            }
            Ok((k.trim().to_string(), v.to_string()))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_inline_wins_when_only_content_given() {
        assert_eq!(
            resolve_content(Some("<p>x</p>"), None, false).unwrap(),
            Some("<p>x</p>".into())
        );
    }

    #[test]
    fn content_and_file_conflict() {
        let err = resolve_content(Some("a"), Some("b.html"), false).unwrap_err();
        assert!(matches!(err, WpxError::Validation { .. }));
    }

    #[test]
    fn content_file_reads_from_disk() {
        let dir = std::env::temp_dir().join(format!("wpx-input-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("body.html");
        std::fs::write(
            &file,
            "<!-- wp:paragraph --><p>hi</p><!-- /wp:paragraph -->",
        )
        .unwrap();
        let got = resolve_content(None, Some(file.to_str().unwrap()), false).unwrap();
        assert!(got.unwrap().starts_with("<!-- wp:paragraph -->"));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn content_file_missing_is_validation_error() {
        let err = resolve_content(None, Some("/definitely/missing.html"), false).unwrap_err();
        assert!(matches!(err, WpxError::Validation { .. }));
    }

    #[test]
    fn stdin_conflict_with_json() {
        let err = resolve_content(None, Some("-"), true).unwrap_err();
        assert!(matches!(err, WpxError::Validation { .. }));
    }

    #[test]
    fn none_when_nothing_given() {
        assert_eq!(resolve_content(None, None, false).unwrap(), None);
    }

    #[test]
    fn key_values_parse() {
        let got = parse_key_values(&["context=edit".into(), "per_page=5".into()]).unwrap();
        assert_eq!(
            got,
            vec![
                ("context".to_string(), "edit".to_string()),
                ("per_page".to_string(), "5".to_string())
            ]
        );
        assert!(parse_key_values(&["novalue".into()]).is_err());
        assert!(parse_key_values(&["=x".into()]).is_err());
    }
}
