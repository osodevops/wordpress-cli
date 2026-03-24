use clap::ValueEnum;

/// Output format for command results.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    /// Automatically detect: table if TTY, JSON if piped.
    Auto,
    /// Pretty-printed JSON.
    Json,
    /// ASCII table (human-readable).
    Table,
    /// Comma-separated values.
    Csv,
    /// YAML format.
    Yaml,
    /// Newline-delimited JSON (one JSON object per line).
    Ndjson,
}

impl Default for OutputFormat {
    fn default() -> Self {
        Self::Auto
    }
}

impl OutputFormat {
    /// Resolve `Auto` to a concrete format based on whether stdout is a TTY.
    pub fn resolve(self) -> Self {
        if self == Self::Auto {
            if std::io::IsTerminal::is_terminal(&std::io::stdout()) {
                Self::Table
            } else {
                Self::Json
            }
        } else {
            self
        }
    }
}
