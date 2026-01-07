use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum DruckerContent {
    /// Print a text string (written to a temp file internally).
    Text(String),
    /// Print an existing file at this path.
    File(PathBuf),
}
