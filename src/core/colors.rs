//! Unified color system for consistent git status visualization.
//!
//! This module provides a centralized color mapping system that ensures all git status
//! indicators use consistent colors throughout the application. It replaces scattered
//! color logic with a single source of truth.
//!
//! # Public API
//! - [`get_status_color_style`]: Get color function for a git status
//! - [`get_aligned_status`]: Get properly aligned colored status text
//! - [`get_colored_path`]: Apply status color to file paths
//! - [`get_legend_status`]: Format status for legend display
//! - [`format_file_status`]: Complete file line formatting (legacy)
//!
//! # Color Scheme
//! - **Modified**: Yellow for both staged and unstaged modifications
//! - **Added**: Green for new files in index
//! - **Deleted**: Red for removed files
//! - **Renamed/Copied**: Blue for file operations
//! - **Untracked**: Cyan for new untracked files
//! - **Unmerged**: Red bold for conflict resolution needed

use crate::core::git_status::GitStatus;
use colored::*;

/// Single function to apply color styling based on git status
/// Returns a closure that can be applied to any text to get the appropriate color
pub fn get_status_color_style(status: GitStatus) -> Box<dyn Fn(&str) -> ColoredString> {
    match status {
        GitStatus::Modified => Box::new(|text: &str| text.yellow()),
        GitStatus::Untracked => Box::new(|text: &str| text.cyan()),
        GitStatus::Deleted => Box::new(|text: &str| text.red()),
        GitStatus::Added => Box::new(|text: &str| text.green()),
        GitStatus::Renamed => Box::new(|text: &str| text.blue()),
        GitStatus::Copied => Box::new(|text: &str| text.blue()),
        GitStatus::TypeChanged => Box::new(|text: &str| text.magenta()),
        GitStatus::Unmerged => Box::new(|text: &str| text.red().bold()),
    }
}

/// Legacy function for string-based status (backward compatibility during migration)
pub fn get_status_color_style_legacy(status: &str) -> Box<dyn Fn(&str) -> ColoredString> {
    let git_status = GitStatus::from(status);
    get_status_color_style(git_status)
}

/// Get colored status symbol with proper alignment
pub fn get_aligned_status(status: GitStatus) -> ColoredString {
    let color_fn = get_status_color_style(status);
    let status_str = status.as_str();
    match status_str {
        s if s.len() == 2 => color_fn(status_str), // Double chars (UU, ??, etc.), no padding
        _ => color_fn(&format!("{status_str} ")),  // Single chars, add space for alignment
    }
}

/// Legacy function for string-based status (backward compatibility)
pub fn get_aligned_status_legacy(status: &str) -> ColoredString {
    let git_status = GitStatus::from(status);
    get_aligned_status(git_status)
}

/// Get colored file path using the status color
pub fn get_colored_path(status: GitStatus, path: &str) -> ColoredString {
    let color_fn = get_status_color_style(status);
    color_fn(path)
}

/// Legacy function for string-based status (backward compatibility)
pub fn get_colored_path_legacy(status: &str, path: &str) -> ColoredString {
    let git_status = GitStatus::from(status);
    get_colored_path(git_status, path)
}

/// Get colored status for legend display
pub fn get_legend_status(status: GitStatus) -> ColoredString {
    let color_fn = get_status_color_style(status);
    let status_str = status.as_str();
    match status_str {
        s if s.len() == 2 => color_fn(status_str), // Double chars (UU, ??, etc.)
        _ => color_fn(&format!("{status_str} ")),  // Single chars with space
    }
}

/// Legacy function for string-based status (backward compatibility)
pub fn get_legend_status_legacy(status: &str) -> ColoredString {
    let git_status = GitStatus::from(status);
    get_legend_status(git_status)
}

/// Legacy function for backwards compatibility (now uses unified color system)
pub fn format_file_status(index: usize, status: &str, path: &str) -> String {
    let index_colored = format!("[{index}]").cyan().bold();
    let status_colored = get_aligned_status_legacy(status);
    let path_colored = get_colored_path_legacy(status, path);

    // Format like git status: single space for ?? (two chars), two spaces for single char statuses
    let spacing = if status.len() == 2 { " " } else { "  " };
    format!("{index_colored} {status_colored}{spacing}{path_colored}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_modified_file() {
        let result = format_file_status(1, "M", "src/main.rs");
        // Test contains the expected components (actual colors won't show in test)
        assert!(result.contains("[1]"));
        assert!(result.contains("M"));
        assert!(result.contains("src/main.rs"));
    }

    #[test]
    fn test_format_untracked_file() {
        let result = format_file_status(2, "??", "newfile.txt");
        assert!(result.contains("[2]"));
        assert!(result.contains("??"));
        assert!(result.contains("newfile.txt"));
    }

    #[test]
    fn test_format_deleted_file() {
        let result = format_file_status(3, "D", "deleted.txt");
        assert!(result.contains("[3]"));
        assert!(result.contains("D"));
        assert!(result.contains("deleted.txt"));
    }

    #[test]
    fn test_get_aligned_status() {
        // Single character statuses should have padding
        let result_m = get_aligned_status(GitStatus::Modified);
        assert!(result_m.to_string().contains("M "));

        // Double character statuses should not have padding
        let result_untracked = get_aligned_status(GitStatus::Untracked);
        assert!(result_untracked.to_string().contains("??"));
        assert!(!result_untracked.to_string().contains("?? "));
    }

    #[test]
    fn test_unified_color_system() {
        // Test that all functions use the same underlying color for the same status
        let status = GitStatus::Modified;
        let path = "test.txt";

        // All should be consistently colored (yellow for modified files)
        let color_fn = get_status_color_style(status);
        let direct_colored = color_fn("M");
        let path_colored = get_colored_path(status, path);
        let aligned_status = get_aligned_status(status);
        let legend_status = get_legend_status(status);

        // All should contain the text and be colored
        assert!(direct_colored.to_string().contains("M"));
        assert!(path_colored.to_string().contains("test.txt"));
        assert!(aligned_status.to_string().contains("M "));
        assert!(legend_status.to_string().contains("M "));
    }

    #[test]
    fn test_status_color_style_consistency() {
        // Test that the color style function returns consistent results
        let statuses = [
            GitStatus::Modified,
            GitStatus::Untracked,
            GitStatus::Deleted,
            GitStatus::Added,
            GitStatus::Renamed,
            GitStatus::Copied,
            GitStatus::Unmerged,
        ];

        for status in &statuses {
            let color_fn = get_status_color_style(*status);
            let colored1 = color_fn("test");
            let colored2 = color_fn("test");
            // Both should produce the same colored output
            assert_eq!(colored1.to_string(), colored2.to_string());
        }
    }
}
