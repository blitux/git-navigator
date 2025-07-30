//! Git repository management and setup utilities
//!
//! Provides functions for creating and managing test repositories with various states
//! and configurations for comprehensive testing scenarios.

#![allow(dead_code)]

use git_navigator::core::error::{GitNavigatorError, Result};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Test repository setup result containing both the temporary directory
/// and the repository path. The TempDir must be kept alive for the duration
/// of the test to prevent cleanup.
pub struct TestRepo {
    pub temp_dir: TempDir,
    pub path: PathBuf,
}

impl TestRepo {
    /// Get the repository path as a reference
    pub fn path(&self) -> &Path {
        &self.path
    }
}

/// Sets up a fresh git repository for testing
///
/// Creates a temporary directory, initializes it as a git repository,
/// and sets up basic git configuration to avoid user prompts.
///
/// # Returns
///
/// A `TestRepo` containing both the temporary directory (which must be kept alive)
/// and the repository path.
///
/// # Example
///
/// ```rust
/// use git_navigator_tests::common::setup_test_repo;
///
/// #[test]
/// fn my_test() -> anyhow::Result<()> {
///     let repo = setup_test_repo()?;
///     // Use repo.path() for git operations
///     Ok(())
/// }
/// ```
pub fn setup_test_repo() -> Result<TestRepo> {
    let temp_dir = TempDir::new().map_err(|e| GitNavigatorError::Io(e))?;
    let repo_path = temp_dir.path().to_path_buf();

    // Initialize git repo
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(&repo_path)
        .output()
        .map_err(|e| GitNavigatorError::Io(e))?;

    // Set git config to avoid prompts during tests
    std::process::Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(&repo_path)
        .output()
        .map_err(|e| GitNavigatorError::Io(e))?;

    std::process::Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(&repo_path)
        .output()
        .map_err(|e| GitNavigatorError::Io(e))?;

    Ok(TestRepo {
        temp_dir,
        path: repo_path,
    })
}

/// Sets up a git repository with an initial commit
///
/// Creates a repository using `setup_test_repo()` and adds an initial commit
/// with a basic file to establish a git history.
///
/// # Returns
///
/// A `TestRepo` with an initial commit containing "initial.txt"
pub fn setup_test_repo_with_initial_commit() -> Result<TestRepo> {
    let repo = setup_test_repo()?;

    // Create initial file and commit
    create_file(&repo.path, "initial.txt", "initial content\n")?;
    git_add(&repo.path, "initial.txt")?;
    git_commit(&repo.path, "Initial commit")?;

    Ok(repo)
}

/// Creates a file with specified content in the repository
///
/// # Arguments
///
/// * `repo_path` - Path to the repository
/// * `filename` - Name of the file to create
/// * `content` - Content to write to the file
pub fn create_file(repo_path: &Path, filename: &str, content: &str) -> Result<()> {
    fs::write(repo_path.join(filename), content).map_err(|e| GitNavigatorError::Io(e))?;
    Ok(())
}

/// Adds a file to the git index
///
/// # Arguments
///
/// * `repo_path` - Path to the repository  
/// * `filename` - Name of the file to add (or "." for all files)
pub fn git_add(repo_path: &Path, filename: &str) -> Result<()> {
    std::process::Command::new("git")
        .args(["add", filename])
        .current_dir(repo_path)
        .output()
        .map_err(|e| GitNavigatorError::Io(e))?;
    Ok(())
}

/// Creates a git commit with the specified message
///
/// # Arguments
///
/// * `repo_path` - Path to the repository
/// * `message` - Commit message
pub fn git_commit(repo_path: &Path, message: &str) -> Result<()> {
    std::process::Command::new("git")
        .args(["commit", "-m", message])
        .current_dir(repo_path)
        .output()
        .map_err(|e| GitNavigatorError::Io(e))?;
    Ok(())
}

/// Removes a file from the filesystem (not from git)
///
/// # Arguments
///
/// * `repo_path` - Path to the repository
/// * `filename` - Name of the file to remove
pub fn remove_file(repo_path: &Path, filename: &str) -> Result<()> {
    fs::remove_file(repo_path.join(filename)).map_err(|e| GitNavigatorError::Io(e))?;
    Ok(())
}

/// Creates multiple test files with sequential content
///
/// # Arguments
///
/// * `repo_path` - Path to the repository
/// * `filenames` - Slice of filenames to create
///
/// # Example
///
/// ```rust
/// create_test_files(&repo.path, &["file1.txt", "file2.txt", "file3.txt"])?;
/// ```
pub fn create_test_files(repo_path: &Path, filenames: &[&str]) -> Result<()> {
    for (i, filename) in filenames.iter().enumerate() {
        let content = format!("content{}\nline 2\n", i + 1);
        create_file(repo_path, filename, &content)?;
    }
    Ok(())
}

/// Modifies multiple test files with new content
///
/// # Arguments
///
/// * `repo_path` - Path to the repository
/// * `filenames` - Slice of filenames to modify
pub fn modify_test_files(repo_path: &Path, filenames: &[&str]) -> Result<()> {
    for (i, filename) in filenames.iter().enumerate() {
        let content = format!("modified{}\nline 2\nnew line\n", i + 1);
        create_file(repo_path, filename, &content)?;
    }
    Ok(())
}

/// Runs the git-navigator status command to populate cache
///
/// # Arguments
///
/// * `repo_path` - Path to the repository
pub fn run_status_to_cache(repo_path: &Path) -> Result<()> {
    use assert_cmd::prelude::*;
    use std::process::Command;

    let mut cmd = Command::cargo_bin("git-navigator")
        .map_err(|e| GitNavigatorError::custom_cache_error("Command not found", e))?;
    cmd.arg("status").current_dir(repo_path).assert().success();
    Ok(())
}

/// Creates a GitRepo from a TestRepo for use with git2-based operations
pub fn create_git_repo(test_repo: &TestRepo) -> Result<git_navigator::core::git::GitRepo> {
    git_navigator::core::git::GitRepo::open(&test_repo.path)
}

/// Sets up a test repo and returns both TestRepo and GitRepo
pub fn setup_test_git_repo() -> Result<(TestRepo, git_navigator::core::git::GitRepo)> {
    let test_repo = setup_test_repo()?;
    let git_repo = create_git_repo(&test_repo)?;
    Ok((test_repo, git_repo))
}
