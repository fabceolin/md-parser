//! Error types for md-parser

use thiserror::Error;

/// Errors that can occur during Markdown parsing
#[derive(Debug, Error)]
pub enum ParseError {
    /// Invalid markdown structure
    #[error("Invalid markdown structure: {0}")]
    InvalidStructure(String),

    /// Frontmatter parsing error (when frontmatter feature is enabled)
    #[cfg(feature = "frontmatter")]
    #[error("Frontmatter parse error: {0}")]
    FrontmatterError(String),

    /// IO error when reading files
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}
