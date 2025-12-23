//! Error types for the ref-resolver plugin.

use thiserror::Error;

/// Errors that can occur during plugin execution.
#[derive(Error, Debug)]
pub enum Error {
    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// Processing error during chapter transformation
    #[error("Processing error in {chapter}: {message}")]
    Processing { chapter: String, message: String },

    /// Reference resolution error
    #[error("Failed to resolve reference '{reference}': {reason}")]
    Resolution { reference: String, reason: String },

    /// Invalid reference syntax
    #[error("Invalid reference syntax '{reference}': {reason}")]
    InvalidSyntax { reference: String, reason: String },

    /// File not found when resolving glob pattern
    #[error("No file found matching pattern '{pattern}' for reference '{reference}'")]
    FileNotFound { pattern: String, reference: String },

    /// Multiple files found when expecting one
    #[error("Multiple files found for reference '{reference}': {files:?}")]
    AmbiguousReference {
        reference: String,
        files: Vec<String>,
    },

    /// Unknown reference prefix
    #[error("Unknown reference prefix '{prefix}' in '{reference}'")]
    UnknownPrefix { prefix: String, reference: String },

    /// MDBook error wrapper
    #[error("MDBook error: {0}")]
    MdBook(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Glob pattern error
    #[error("Invalid glob pattern: {0}")]
    GlobPattern(#[from] glob::PatternError),
}

impl From<mdbook::errors::Error> for Error {
    fn from(err: mdbook::errors::Error) -> Self {
        Error::MdBook(err.to_string())
    }
}
