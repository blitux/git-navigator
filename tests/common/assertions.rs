//! Common assertion helpers for test output validation
//!
//! Provides predicates and assertion utilities for validating git-navigator
//! command output, error messages, and expected behaviors.

#![allow(dead_code)]

use predicates::prelude::*;

/// Creates a predicate that checks for git repository error messages
pub fn not_in_git_repo() -> impl Predicate<str> {
    predicates::str::contains("Not in a git repository")
        .or(predicates::str::contains("NotInGitRepo"))
}

/// Creates a predicate that checks for cache-related error messages
pub fn cache_error() -> impl Predicate<str> {
    predicates::str::contains("No cached files found")
        .or(predicates::str::contains("Cache file does not exist"))
        .or(predicates::str::contains("Cannot load file cache"))
}

/// Creates a predicate that checks for common output elements
pub fn has_branch_info() -> impl Predicate<str> {
    predicates::str::contains("Branch:")
}

/// Creates a predicate that checks for parent commit info
pub fn has_parent_info() -> impl Predicate<str> {
    predicates::str::contains("Parent:")
}

/// Creates a predicate that checks for numbered file indices
pub fn has_file_index(index: u32) -> impl Predicate<str> {
    predicates::str::contains(format!("[{}]", index))
}

/// Creates a predicate that checks for git status descriptions
pub fn has_status(status: &str) -> impl Predicate<str> {
    predicates::str::contains(format!("({})", status))
}
