use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApplicationError {
    #[error("Failed to parse post structure from {path}: {reason}")]
    ParsingError { path: PathBuf, reason: String },

    #[error("Failed to read file at {path}: {source}")]
    ReadingError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to create post from {path}: {reason}")]
    PostCreationError { path: PathBuf, reason: String },

    #[error("Failed to parse YAML: {0}")]
    YamlError(#[from] serde_yaml::Error),

    #[error("Failed to read file: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Failed to clone repository: {0}")]
    GitError(#[from] git2::Error),
}
