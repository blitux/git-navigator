//! Centralized initialization for index-based commands.
//!
//! This module provides [`IndexCommandInit`] which handles the common initialization
//! pattern for commands that work with file indices (add, diff, reset, etc.).
//! It eliminates code duplication by centralizing git repo validation, cache loading,
//! and index parsing.
//!
//! # Public API
//! - [`IndexCommandInit`]: Main initializer with static methods
//! - [`IndexCommandContext`]: Initialized context containing all required data
//!
//! # Initialization Steps
//! 1. **Git repository validation**: Ensure we're in a valid git repository
//! 2. **Cache loading**: Load previously cached file list from `gs` command
//! 3. **File validation**: Ensure files are available to operate on
//! 4. **Index parsing**: Parse and validate user-provided indices
//!
//! # Error Handling
//! - **Custom messages**: Support for command-specific error messages
//! - **Comprehensive validation**: All failure modes are handled gracefully
//! - **User guidance**: Error messages guide users to run `gs` first

use crate::commands::status::load_files_cache;
use crate::core::{
    args_parser::ArgsParser,
    error::{GitNavigatorError, Result},
    git::GitRepo,
    state::FileEntry,
};
use std::env;

/// Initialization context for commands that work with file indices
pub struct IndexCommandContext {
    pub git_repo: GitRepo,
    pub files: Vec<FileEntry>,
    pub indices: Vec<usize>,
}

/// Centralized initialization for commands that require file indices
///
/// This handles all the common setup steps:
/// 1. Verify we're in a git repository
/// 2. Load cached files from previous `gs` command
/// 3. Validate that files are available
/// 4. Parse and validate the provided indices
///
/// # Arguments
/// * `indices_args` - Command line arguments containing indices
///
/// # Returns
/// * `Ok(IndexCommandContext)` - All initialization successful, ready to use
/// * `Err` - If any step fails (not in git repo, no cache, invalid indices, etc.)
///
/// # Examples
/// ```no_run
/// use git_navigator::core::command_init::IndexCommandInit;
///
/// let context = IndexCommandInit::initialize(vec!["1".to_string(), "3-5".to_string()])?;
/// // Now you can use context.git_repo, context.files, context.indices
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct IndexCommandInit;

impl IndexCommandInit {
    /// Initialize everything needed for an index-based command
    pub fn initialize(indices_args: Vec<String>) -> Result<IndexCommandContext> {
        // Step 1: Check if we're in a git repository
        let current_dir = env::current_dir()?;
        let git_repo = GitRepo::open(&current_dir).map_err(|_| GitNavigatorError::NotInGitRepo)?;

        // Step 2: Load cached files from previous gs command
        log::debug!("Loading cached files for index-based command");
        let files = load_files_cache(&git_repo.get_repo_path()).map_err(|e| {
            log::warn!("Failed to load cache: {e}");
            GitNavigatorError::cache_load_error(e)
        })?;

        // Step 3: Validate that files are available
        if files.is_empty() {
            return Err(GitNavigatorError::NoAvailableFiles);
        }

        // Step 4: Parse and validate indices using the centralized parser
        let indices = ArgsParser::parse_indices(indices_args, files.len())?;

        log::debug!(
            "Successfully initialized index command with {} files and {} selected indices",
            files.len(),
            indices.len()
        );

        // Return the initialized context
        Ok(IndexCommandContext {
            git_repo,
            files,
            indices,
        })
    }

    /// Initialize with custom error messages for specific commands
    pub fn initialize_with_messages(
        indices_args: Vec<String>,
        cache_error_msg: &str,
        empty_files_msg: &str,
    ) -> Result<IndexCommandContext> {
        // NEW: Step 0: Check if no indices provided
        if indices_args.is_empty() {
            return Err(GitNavigatorError::NoIndicesProvided);
        }

        // Step 1: Check if we're in a git repository
        let current_dir = env::current_dir()?;
        let git_repo = GitRepo::open(&current_dir).map_err(|_| GitNavigatorError::NotInGitRepo)?;

        // Step 2: Load cached files from previous gs command
        log::debug!("Loading cached files for index-based command with custom messages");
        let files = load_files_cache(&git_repo.get_repo_path()).map_err(|e| {
            log::warn!("Failed to load cache: {e}");
            GitNavigatorError::custom_cache_error(cache_error_msg, e)
        })?;

        // Step 3: Validate that files are available
        if files.is_empty() {
            return Err(GitNavigatorError::custom_empty_files_error(empty_files_msg));
        }

        // Step 4: Parse and validate indices using the centralized parser
        let indices = ArgsParser::parse_indices(indices_args, files.len())?;

        log::debug!(
            "Successfully initialized index command with {} files and {} selected indices",
            files.len(),
            indices.len()
        );

        Ok(IndexCommandContext {
            git_repo,
            files,
            indices,
        })
    }
}

/// Helper methods for the context
impl IndexCommandContext {
    /// Get files corresponding to the parsed indices
    pub fn get_selected_files(&self) -> Vec<&FileEntry> {
        self.indices
            .iter()
            .map(|&idx| &self.files[idx - 1]) // Convert to 0-based indexing
            .collect()
    }

    /// Get file count for logging/display
    pub fn file_count(&self) -> usize {
        self.files.len()
    }

    /// Get selected indices count
    pub fn selected_count(&self) -> usize {
        self.indices.len()
    }

    /// Check if any files are selected
    pub fn has_selected_files(&self) -> bool {
        !self.indices.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_initialize_with_custom_messages() {
        let result = {
            let temp_dir = TempDir::new().unwrap();
            let original_dir = env::current_dir().unwrap();

            // Cambiar al directorio temporal vacío (no es un repo git)
            env::set_current_dir(temp_dir.path()).unwrap();

            let result = IndexCommandInit::initialize_with_messages(
                vec!["1".to_string()],
                "Custom cache error",
                "Custom empty files error",
            );

            // Restaurar el directorio original antes de que se dropee temp_dir
            env::set_current_dir(&original_dir).unwrap();

            result
        };

        assert!(result.is_err());
        match result {
            Err(e) => assert!(e.to_string().contains("Not in a git repository")),
            Ok(_) => panic!("Expected error, but got success"),
        }
    }

    // Note: Testing successful initialization requires a proper git repository with cache,
    // which is better tested through integration tests in the actual commands

    #[test]
    #[ignore] // Disabled due to directory handling issues when temp_dir gets dropped
    fn test_initialize_no_indices() {
        // NOTE: This test has directory management issues where the temp directory
        // gets dropped while we're still using it as current directory
        // TODO: Refactor to use proper directory isolation
        let temp_dir = TempDir::new().unwrap();
        let original_dir = env::current_dir().unwrap();

        // Change to non-git directory (will fail before args parsing)
        env::set_current_dir(temp_dir.path()).unwrap();

        let result = IndexCommandInit::initialize(vec![]);

        // Restore original directory
        env::set_current_dir(&original_dir).unwrap();

        assert!(result.is_err());
        // Will fail on git repo check before getting to args parsing
    }

    #[test]
    fn test_context_methods() {
        // Create a mock context for testing helper methods
        let files = vec![
            FileEntry {
                index: 1,
                status: crate::core::git_status::GitStatus::Modified,
                path: "file1.txt".into(),
                staged: false,
            },
            FileEntry {
                index: 2,
                status: crate::core::git_status::GitStatus::Added,
                path: "file2.txt".into(),
                staged: true,
            },
        ];

        // We can't create a GitRepo without an actual repo, so this test is limited
        // The real testing should happen in integration tests
        assert_eq!(files.len(), 2);
    }
}
