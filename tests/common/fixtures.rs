//! Test data generation utilities and predefined scenarios
//!
//! Provides functions for creating repositories with specific file states
//! and configurations to test various git scenarios consistently.

#![allow(dead_code)]

use super::repository::*;
use git_navigator::core::error::Result;

/// Scenario: Repository with multiple files for range testing
/// Creates a repository with 5 files for testing index ranges
pub fn create_multi_file_repo() -> Result<TestRepo> {
    let repo = setup_test_repo()?;

    // Create and commit initial files
    create_test_files(&repo.path, &["file1.txt", "file2.txt", "file3.txt"])?;
    git_add(&repo.path, ".")?;
    git_commit(&repo.path, "Initial commit")?;

    // Modify all files and add new ones
    modify_test_files(&repo.path, &["file1.txt", "file2.txt", "file3.txt"])?;
    create_test_files(&repo.path, &["file4.txt", "file5.txt"])?;

    Ok(repo)
}
