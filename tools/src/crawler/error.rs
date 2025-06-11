use thiserror::Error; // For declarative error types

/// Custom error type for the `file_eyes` library.
#[derive(Debug, Error)]
pub enum CrawlerError {
    /// Indicates that the provided root path does not exist.
    #[error("Root path '{0}' does not exist.")]
    RootPathDoesNotExist(std::path::PathBuf),
    /// Indicates that the provided root path is not a directory.
    #[error("Root path '{0}' is not a directory.")]
    RootPathIsNotDirectory(std::path::PathBuf),
    /// Indicates that a requested path is not a file or does not exist.
    #[error("Path '{0}' is not a file or does not exist.")]
    PathNotAFile(std::path::PathBuf),
    /// Indicates that a requested path is not a directory or does not exist.
    #[error("Path '{0}' is not a directory or does not exist.")]
    PathNotADirectory(std::path::PathBuf),
    /// Indicates an attempt to access a file or directory outside the defined root path.
    #[error("Attempted to access path '{0}' outside of root path.")]
    AccessOutsideRoot(std::path::PathBuf),
    /// An I/O error occurred during a file system operation.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    /// An error occurred during path canonicalization.
    #[error("Path canonicalization error: {0}")]
    Canonicalization(std::io::Error),
    /// Any other unforeseen error.
    #[error("An unexpected error occurred: {0}")]
    Other(String),
}
