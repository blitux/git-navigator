//! Domain-specific error types and error handling utilities.
//!
//! This module defines [`GitNavigatorError`] which provides comprehensive error handling
//! for all git-navigator operations. It uses `thiserror` for ergonomic error definitions
//! and includes specialized error constructors for common failure scenarios.
//!
//! # Public API
//! - [`GitNavigatorError`]: Main error enum covering all failure modes
//! - [`Result<T>`]: Type alias for `std::result::Result<T, GitNavigatorError>`
//!
//! # Error Categories
//! - **Git operations**: Repository not found, git2 library errors
//! - **File operations**: File not found, I/O errors, UTF-8 issues  
//! - **Index parsing**: Invalid format, out of bounds, validation errors
//! - **Cache operations**: Serialization, file system, missing cache errors

use std::path::PathBuf;
use thiserror::Error;

/// Domain-specific error types for git-navigator
#[derive(Error, Debug)]
pub enum GitNavigatorError {
    // Git repository errors
    #[error("Not in a git repository")]
    NotInGitRepo,

    #[error("Git repository error: {0}")]
    GitRepo(#[from] git2::Error),

    #[error("Invalid UTF-8 path in repository")]
    InvalidUtf8Path,

    // File operation errors
    #[error("File does not exist: {path}")]
    FileNotFound { path: PathBuf },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid UTF-8 in file content: {0}")]
    Utf8(#[from] std::str::Utf8Error),

    // Index parsing errors
    #[error("No file indices provided. Usage: <command> <indices>\nExample: ga 1 3-5,8")]
    NoIndicesProvided,

    #[error("No file indices provided")]
    NoIndicesProvidedForCommand { command: String },

    #[error("Invalid index format: {input}. Use format like: 1, 1-3, or 1,3,5")]
    InvalidIndexFormat { input: String },

    #[error("No valid indices provided. Use format like: 1, 1-3, or 1,3,5")]
    NoValidIndices,

    #[error("Invalid range format: '{range}'. Use format like '3-6'")]
    InvalidRangeFormat { range: String },

    #[error("Invalid number in range: '{number}'")]
    InvalidRangeNumber { number: String },

    #[error("Invalid range: start ({start}) must be <= end ({end})")]
    InvalidRangeOrder { start: usize, end: usize },

    #[error("Invalid number: '{number}'")]
    InvalidNumber { number: String },

    #[error("Index must be positive (got 0)")]
    ZeroIndex,

    #[error("Index {index} is out of range (1-{max} available)")]
    IndexOutOfRange { index: usize, max: usize },

    #[error("No files available to operate on")]
    NoFilesAvailable,

    // Cache errors
    #[error("Could not find cache directory")]
    CacheDirectoryNotFound,

    #[error("Failed to create cache directory '{path}': {source}")]
    CacheDirectoryCreationFailed {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("Failed to serialize cache data: {source}")]
    CacheSerializationFailed { source: serde_json::Error },

    #[error("Failed to write cache file '{path}': {source}")]
    CacheWriteFailed {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("Cannot load file cache: {source}. Run 'gs' first to generate file list.")]
    CacheLoadError {
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Cache file does not exist at '{path}'. Run 'gs' first to generate file list.")]
    CacheFileNotFound { path: PathBuf },

    #[error("Failed to read cache file '{path}': {source}")]
    CacheReadFailed {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("Failed to parse cache file '{path}': {source}")]
    CacheParseFailed {
        path: PathBuf,
        source: serde_json::Error,
    },

    #[error("No cached files found. Run 'gs' first to generate file list.")]
    NoCachedFiles,

    #[error("No files available. Run 'gs' first to see available files.")]
    NoAvailableFiles,

    #[error("{message}: {source}. Run 'gs' first to generate file list.")]
    CustomCacheError {
        message: String,
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("{message}. Run 'gs' first to see available files.")]
    CustomEmptyFilesError { message: String },

    // Git operation errors
    #[error("No valid files found for the specified indices.")]
    NoValidFilesSelected,

    #[error("There are no changes to be added")]
    NoChangesToAdd,

    #[error("Failed to add files to git index: {source}")]
    GitAddFailed { source: git2::Error },

    // JSON serialization errors
    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Convenience type alias for Results using GitNavigatorError
pub type Result<T> = std::result::Result<T, GitNavigatorError>;

impl GitNavigatorError {
    /// Create a custom cache error with a specific message
    pub fn custom_cache_error<E>(message: impl Into<String>, source: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self::CustomCacheError {
            message: message.into(),
            source: Box::new(source),
        }
    }

    /// Create a custom empty files error with a specific message
    pub fn custom_empty_files_error(message: impl Into<String>) -> Self {
        Self::CustomEmptyFilesError {
            message: message.into(),
        }
    }

    /// Create a file not found error
    pub fn file_not_found(path: impl Into<PathBuf>) -> Self {
        Self::FileNotFound { path: path.into() }
    }

    /// Create an index out of range error
    pub fn index_out_of_range(index: usize, max: usize) -> Self {
        Self::IndexOutOfRange { index, max }
    }

    /// Create a no indices provided error for a specific command
    pub fn no_indices_provided_for_command(command: impl Into<String>) -> Self {
        Self::NoIndicesProvidedForCommand {
            command: command.into(),
        }
    }

    /// Create an invalid index format error
    pub fn invalid_index_format(input: impl Into<String>) -> Self {
        Self::InvalidIndexFormat {
            input: input.into(),
        }
    }

    /// Create an invalid range format error
    pub fn invalid_range_format(range: impl Into<String>) -> Self {
        Self::InvalidRangeFormat {
            range: range.into(),
        }
    }

    /// Create an invalid range number error
    pub fn invalid_range_number(number: impl Into<String>) -> Self {
        Self::InvalidRangeNumber {
            number: number.into(),
        }
    }

    /// Create an invalid range order error
    pub fn invalid_range_order(start: usize, end: usize) -> Self {
        Self::InvalidRangeOrder { start, end }
    }

    /// Create an invalid number error
    pub fn invalid_number(number: impl Into<String>) -> Self {
        Self::InvalidNumber {
            number: number.into(),
        }
    }

    /// Create a git add failed error
    pub fn git_add_failed(source: git2::Error) -> Self {
        Self::GitAddFailed { source }
    }

    /// Create a cache load error
    pub fn cache_load_error<E>(source: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self::CacheLoadError {
            source: Box::new(source),
        }
    }

    /// Create a cache directory creation failed error
    pub fn cache_directory_creation_failed(
        path: impl Into<PathBuf>,
        source: std::io::Error,
    ) -> Self {
        Self::CacheDirectoryCreationFailed {
            path: path.into(),
            source,
        }
    }

    /// Create a cache serialization failed error
    pub fn cache_serialization_failed(source: serde_json::Error) -> Self {
        Self::CacheSerializationFailed { source }
    }

    /// Create a cache write failed error
    pub fn cache_write_failed(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::CacheWriteFailed {
            path: path.into(),
            source,
        }
    }

    /// Create a cache file not found error
    pub fn cache_file_not_found(path: impl Into<PathBuf>) -> Self {
        Self::CacheFileNotFound { path: path.into() }
    }

    /// Create a cache read failed error
    pub fn cache_read_failed(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::CacheReadFailed {
            path: path.into(),
            source,
        }
    }

    /// Create a cache parse failed error
    pub fn cache_parse_failed(path: impl Into<PathBuf>, source: serde_json::Error) -> Self {
        Self::CacheParseFailed {
            path: path.into(),
            source,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = GitNavigatorError::NotInGitRepo;
        assert_eq!(err.to_string(), "Not in a git repository");
    }

    #[test]
    fn test_file_not_found_error() {
        let err = GitNavigatorError::file_not_found("test.txt");
        assert_eq!(err.to_string(), "File does not exist: test.txt");
    }

    #[test]
    fn test_index_out_of_range_error() {
        let err = GitNavigatorError::index_out_of_range(5, 3);
        assert_eq!(err.to_string(), "Index 5 is out of range (1-3 available)");
    }

    #[test]
    fn test_invalid_index_format_error() {
        let err = GitNavigatorError::invalid_index_format("abc");
        assert_eq!(
            err.to_string(),
            "Invalid index format: abc. Use format like: 1, 1-3, or 1,3,5"
        );
    }

    #[test]
    fn test_custom_cache_error() {
        let inner_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err = GitNavigatorError::custom_cache_error("Custom message", inner_err);
        assert!(err.to_string().contains("Custom message"));
    }

    #[test]
    fn test_cache_directory_creation_failed() {
        let path = std::path::PathBuf::from("/test/path");
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "permission denied");
        let err = GitNavigatorError::cache_directory_creation_failed(&path, io_err);
        assert!(err.to_string().contains("/test/path"));
        assert!(err.to_string().contains("permission denied"));
    }

    #[test]
    fn test_cache_serialization_failed() {
        // Create a JSON error by deserializing invalid JSON and re-creating the error type
        let parse_err = serde_json::from_str::<serde_json::Value>("invalid").unwrap_err();
        let err = GitNavigatorError::cache_serialization_failed(parse_err);
        assert!(err.to_string().contains("Failed to serialize cache data"));
    }

    #[test]
    fn test_cache_write_failed() {
        let path = std::path::PathBuf::from("/test/file.json");
        let io_err = std::io::Error::new(std::io::ErrorKind::OutOfMemory, "no space left");
        let err = GitNavigatorError::cache_write_failed(&path, io_err);
        assert!(err.to_string().contains("/test/file.json"));
        assert!(err.to_string().contains("no space left"));
    }

    #[test]
    fn test_cache_file_not_found() {
        let path = std::path::PathBuf::from("/test/cache.json");
        let err = GitNavigatorError::cache_file_not_found(&path);
        assert!(err.to_string().contains("/test/cache.json"));
        assert!(err.to_string().contains("does not exist"));
    }

    #[test]
    fn test_cache_read_failed() {
        let path = std::path::PathBuf::from("/test/cache.json");
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
        let err = GitNavigatorError::cache_read_failed(&path, io_err);
        assert!(err.to_string().contains("/test/cache.json"));
        assert!(err.to_string().contains("access denied"));
    }

    #[test]
    fn test_cache_parse_failed() {
        let path = std::path::PathBuf::from("/test/cache.json");
        // Create a JSON parse error by trying to parse invalid JSON
        let json_err = serde_json::from_str::<serde_json::Value>("{ invalid json").unwrap_err();
        let err = GitNavigatorError::cache_parse_failed(&path, json_err);
        assert!(err.to_string().contains("/test/cache.json"));
        assert!(err.to_string().contains("Failed to parse"));
    }
}
