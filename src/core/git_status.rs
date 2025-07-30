//! Type-safe git file status enumeration.
//!
//! This module defines [`GitStatus`] which replaces string-based status codes throughout
//! the codebase with a proper enumeration. This provides better type safety, performance,
//! and maintainability compared to string matching.
//!
//! # Public API
//! - [`GitStatus`]: Main enumeration for all git file status types
//!
//! # Key Features  
//! - **Type safety**: Compile-time checking instead of runtime string comparisons
//! - **git2 integration**: Direct conversion from git2::Status flags
//! - **Display formatting**: Consistent string representation for UI output
//! - **Sorting logic**: Built-in priority ordering for status display
//! - **Backward compatibility**: String conversion for legacy code

use serde::{Deserialize, Serialize};
use std::fmt;

/// Git file status enum to replace string-based status codes
///
/// This provides type safety, better performance, and cleaner code
/// compared to string matching throughout the codebase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GitStatus {
    /// Modified file (M)
    Modified,
    /// Added/new file in index (A)
    Added,
    /// Deleted file (D)
    Deleted,
    /// Renamed file (R)
    Renamed,
    /// Copied file (C)
    Copied,
    /// Type changed (T)
    TypeChanged,
    /// Untracked file (??)
    Untracked,
    /// Unmerged/conflicted file (UU)
    Unmerged,
}

impl GitStatus {
    /// Convert from git2::Status flags to GitStatus enum
    /// Returns the status and whether it's staged
    pub fn from_git2_staged(flags: git2::Status) -> Option<(GitStatus, bool)> {
        // Check staged changes first
        if flags.contains(git2::Status::INDEX_NEW) {
            return Some((GitStatus::Added, true));
        }
        if flags.contains(git2::Status::INDEX_MODIFIED) {
            return Some((GitStatus::Modified, true));
        }
        if flags.contains(git2::Status::INDEX_DELETED) {
            return Some((GitStatus::Deleted, true));
        }
        if flags.contains(git2::Status::INDEX_RENAMED) {
            return Some((GitStatus::Renamed, true));
        }
        if flags.contains(git2::Status::INDEX_TYPECHANGE) {
            return Some((GitStatus::TypeChanged, true));
        }

        None
    }

    /// Convert from git2::Status flags to GitStatus enum for unstaged changes
    /// Returns the status and whether it's staged (always false for unstaged)
    pub fn from_git2_unstaged(flags: git2::Status) -> Option<(GitStatus, bool)> {
        // Conflicted files (highest priority for unstaged)
        if flags.contains(git2::Status::CONFLICTED) {
            return Some((GitStatus::Unmerged, false));
        }

        // Check unstaged changes
        if flags.contains(git2::Status::WT_NEW) {
            return Some((GitStatus::Untracked, false));
        }
        if flags.contains(git2::Status::WT_MODIFIED) {
            return Some((GitStatus::Modified, false));
        }
        if flags.contains(git2::Status::WT_DELETED) {
            return Some((GitStatus::Deleted, false));
        }
        if flags.contains(git2::Status::WT_RENAMED) {
            return Some((GitStatus::Renamed, false));
        }
        if flags.contains(git2::Status::WT_TYPECHANGE) {
            return Some((GitStatus::TypeChanged, false));
        }

        None
    }

    /// Get the string representation for display (legacy compatibility)
    pub fn as_str(&self) -> &'static str {
        match self {
            GitStatus::Modified => "M",
            GitStatus::Added => "A",
            GitStatus::Deleted => "D",
            GitStatus::Renamed => "R",
            GitStatus::Copied => "C",
            GitStatus::TypeChanged => "T",
            GitStatus::Untracked => "??",
            GitStatus::Unmerged => "UU",
        }
    }

    /// Get sort priority for status ordering
    /// Used to maintain consistent file ordering in status display
    pub fn sort_priority(&self, staged: bool) -> u8 {
        match (self, staged) {
            // Group 1: Unmerged states (highest priority - conflicts need attention)
            (GitStatus::Unmerged, _) => 0,
            // Group 2: Staged changes
            (GitStatus::Added, true) => 1,
            (GitStatus::Modified, true) => 2,
            (GitStatus::Deleted, true) => 3,
            (GitStatus::Renamed, true) => 4,
            (GitStatus::Copied, true) => 5,
            (GitStatus::TypeChanged, true) => 6,
            // Group 3: Unstaged changes
            (GitStatus::Modified, false) => 7,
            (GitStatus::Deleted, false) => 8,
            (GitStatus::Renamed, false) => 9,
            (GitStatus::Copied, false) => 10,
            (GitStatus::TypeChanged, false) => 11,
            // Group 4: Untracked
            (GitStatus::Untracked, _) => 12,
            // Default
            _ => 13,
        }
    }

    /// Get human-readable description for status
    pub fn description(&self) -> &'static str {
        match self {
            GitStatus::Modified => "modified",
            GitStatus::Added => "new",
            GitStatus::Deleted => "deleted",
            GitStatus::Renamed => "renamed",
            GitStatus::Copied => "copied",
            GitStatus::TypeChanged => "type changed",
            GitStatus::Untracked => "untracked",
            GitStatus::Unmerged => "both modified",
        }
    }

    /// Check if this status represents a staged change
    pub fn is_staged_by_default(&self) -> bool {
        matches!(self, GitStatus::Added)
    }

    /// Check if this status can be staged
    pub fn can_be_staged(&self) -> bool {
        !matches!(self, GitStatus::Untracked | GitStatus::Unmerged)
    }
}

impl fmt::Display for GitStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Convert from legacy string status to GitStatus enum
/// Used during migration period for backward compatibility
impl From<&str> for GitStatus {
    fn from(status: &str) -> Self {
        match status {
            "M" => GitStatus::Modified,
            "A" => GitStatus::Added,
            "D" => GitStatus::Deleted,
            "R" => GitStatus::Renamed,
            "C" => GitStatus::Copied,
            "T" => GitStatus::TypeChanged,
            "??" => GitStatus::Untracked,
            "UU" => GitStatus::Unmerged,
            _ => GitStatus::Modified, // Default fallback
        }
    }
}

impl From<String> for GitStatus {
    fn from(status: String) -> Self {
        GitStatus::from(status.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_status_as_str() {
        assert_eq!(GitStatus::Modified.as_str(), "M");
        assert_eq!(GitStatus::Added.as_str(), "A");
        assert_eq!(GitStatus::Deleted.as_str(), "D");
        assert_eq!(GitStatus::Renamed.as_str(), "R");
        assert_eq!(GitStatus::Copied.as_str(), "C");
        assert_eq!(GitStatus::TypeChanged.as_str(), "T");
        assert_eq!(GitStatus::Untracked.as_str(), "??");
        assert_eq!(GitStatus::Unmerged.as_str(), "UU");
    }

    #[test]
    fn test_git_status_from_str() {
        assert_eq!(GitStatus::from("M"), GitStatus::Modified);
        assert_eq!(GitStatus::from("A"), GitStatus::Added);
        assert_eq!(GitStatus::from("D"), GitStatus::Deleted);
        assert_eq!(GitStatus::from("R"), GitStatus::Renamed);
        assert_eq!(GitStatus::from("C"), GitStatus::Copied);
        assert_eq!(GitStatus::from("T"), GitStatus::TypeChanged);
        assert_eq!(GitStatus::from("??"), GitStatus::Untracked);
        assert_eq!(GitStatus::from("UU"), GitStatus::Unmerged);
        assert_eq!(GitStatus::from("unknown"), GitStatus::Modified); // Fallback
    }

    #[test]
    fn test_git_status_display() {
        assert_eq!(format!("{}", GitStatus::Modified), "M");
        assert_eq!(format!("{}", GitStatus::Untracked), "??");
        assert_eq!(format!("{}", GitStatus::Unmerged), "UU");
    }

    #[test]
    fn test_sort_priority() {
        // Unmerged has highest priority
        assert_eq!(GitStatus::Unmerged.sort_priority(false), 0);

        // Staged changes come before unstaged
        assert!(GitStatus::Added.sort_priority(true) < GitStatus::Modified.sort_priority(false));

        // Untracked has lowest priority
        assert!(
            GitStatus::Untracked.sort_priority(false) > GitStatus::Modified.sort_priority(false)
        );
    }

    #[test]
    fn test_description() {
        assert_eq!(GitStatus::Modified.description(), "modified");
        assert_eq!(GitStatus::Added.description(), "new");
        assert_eq!(GitStatus::Untracked.description(), "untracked");
        assert_eq!(GitStatus::Unmerged.description(), "both modified");
    }

    #[test]
    fn test_staging_properties() {
        assert!(GitStatus::Added.is_staged_by_default());
        assert!(!GitStatus::Modified.is_staged_by_default());

        assert!(GitStatus::Modified.can_be_staged());
        assert!(!GitStatus::Untracked.can_be_staged());
        assert!(!GitStatus::Unmerged.can_be_staged());
    }

    #[test]
    fn test_from_git2_flags() {
        // Test staged conversions
        let staged_new = git2::Status::INDEX_NEW;
        assert_eq!(
            GitStatus::from_git2_staged(staged_new),
            Some((GitStatus::Added, true))
        );

        let staged_modified = git2::Status::INDEX_MODIFIED;
        assert_eq!(
            GitStatus::from_git2_staged(staged_modified),
            Some((GitStatus::Modified, true))
        );

        // Test unstaged conversions
        let unstaged_new = git2::Status::WT_NEW;
        assert_eq!(
            GitStatus::from_git2_unstaged(unstaged_new),
            Some((GitStatus::Untracked, false))
        );

        let conflicted = git2::Status::CONFLICTED;
        assert_eq!(
            GitStatus::from_git2_unstaged(conflicted),
            Some((GitStatus::Unmerged, false))
        );
    }
}
