use std::path::PathBuf;

/// Print content variants: raw text or a file path.
#[derive(Debug, Clone)]
pub enum DruckerContent {
    /// Print a text string (written to a temp file internally).
    Text(String),
    /// Print an existing file at this path.
    File(PathBuf),
}
