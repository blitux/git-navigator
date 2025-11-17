//! Remove command wrapper that resolves file indices to paths.
//!
//! This module provides a wrapper around the system `rm` command that allows
//! using file indices from the git-navigator cache instead of typing full paths.
//!
//! # Features
//! - **Index resolution**: Converts numeric indices to file paths
//! - **Mixed arguments**: Supports mixing indices with literal paths and flags
//! - **Pass-through**: Non-numeric arguments are passed directly to `rm`
//! - **Cache integration**: Uses the same cache as other git-navigator commands
//!
//! # Examples
//! ```bash
//! # Remove file by index
//! git-navigator rm 1
//!
//! # Remove multiple files using ranges
//! git-navigator rm 1-3
//!
//! # Use with rm flags
//! git-navigator rm -f 5
//!
//! # Remove recursively
//! git-navigator rm -rf 2
//! ```

#[cfg(not(test))]
use crate::commands::status::load_files_cache;
use crate::core::{
    error::{GitNavigatorError, Result},
    git::GitRepo,
    index_parser::IndexParser,
    print_error, print_info,
    state::FileEntry,
};
use std::env;
use std::path::PathBuf;
use std::process::Command;

/// Determines if a string is a numeric index or index pattern
fn is_index_pattern(arg: &str) -> bool {
    // Check if it's a single number
    if arg.parse::<usize>().is_ok() {
        return true;
    }

    // Check if it's a range (e.g., "1-3")
    if arg.contains('-') && !arg.starts_with('-') {
        // Ensure it's not a flag like "-f" or "-rf"
        let parts: Vec<&str> = arg.split('-').collect();
        if parts.len() == 2 {
            return parts[0].parse::<usize>().is_ok() && parts[1].parse::<usize>().is_ok();
        }
    }

    // Check if it's comma-separated (e.g., "1,2,3")
    if arg.contains(',') {
        return arg.split(',').all(|part| {
            part.trim().parse::<usize>().is_ok()
                || (part.contains('-')
                    && !part.starts_with('-')
                    && part
                        .split('-')
                        .all(|p| p.trim().parse::<usize>().is_ok()))
        });
    }

    false
}

/// Resolves index patterns to file paths using the cache
fn resolve_indices(index_str: &str, files: &[FileEntry]) -> Result<Vec<PathBuf>> {
    let indices = IndexParser::parse(index_str)
        .map_err(|e| GitNavigatorError::invalid_index_format(e.to_string()))?;

    // Validate indices are within bounds
    IndexParser::validate(&indices, files.len())?;

    // Convert indices to file paths (indices are 1-based)
    let paths = indices
        .iter()
        .map(|&idx| files[idx - 1].path.clone())
        .collect();

    Ok(paths)
}

/// Processes arguments and separates indices from literal arguments
fn process_arguments(args: Vec<String>, files: &[FileEntry]) -> Result<Vec<String>> {
    let mut resolved_args = Vec::new();
    let mut index_patterns = Vec::new();
    let mut flags = Vec::new();

    // First pass: separate index patterns, flags, and literal arguments
    for arg in args {
        if arg.starts_with('-') {
            // This is a flag (e.g., -f, -r, -rf)
            flags.push(arg);
        } else if is_index_pattern(&arg) {
            index_patterns.push(arg);
        } else {
            // Literal path argument
            resolved_args.push(arg);
        }
    }

    // Second pass: resolve all index patterns
    if !index_patterns.is_empty() {
        let indices_str = index_patterns.join(" ");
        let paths = resolve_indices(&indices_str, files)?;

        // Convert paths to strings
        let path_strings: Vec<String> = paths
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect();

        resolved_args.extend(path_strings);
    }

    // Combine: flags first, then resolved paths
    flags.extend(resolved_args);
    Ok(flags)
}

/// Executes the remove command with index resolution
///
/// # Arguments
/// * `args` - Command line arguments which may include indices, paths, and flags
///
/// # Examples
/// ```no_run
/// use git_navigator::commands::remove::execute_remove;
///
/// // Remove file by index
/// execute_remove(vec!["1".to_string()])?;
///
/// // Remove with flags
/// execute_remove(vec!["-f".to_string(), "5".to_string()])?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn execute_remove(args: Vec<String>) -> Result<()> {
    // Check if any arguments were provided
    if args.is_empty() {
        print_error("No arguments provided");
        print_info("Usage: git-navigator rm [options] <indices|paths>...");
        print_info("Examples:");
        print_info("  git-navigator rm 1           # Remove file 1");
        print_info("  git-navigator rm 1-3         # Remove files 1-3");
        print_info("  git-navigator rm -f 5        # Force remove file 5");
        print_info("  git-navigator rm -rf 2       # Recursively remove directory 2");
        return Err(GitNavigatorError::NoIndicesProvided);
    }

    // Check if we're in a git repository
    let current_dir = env::current_dir()?;
    let git_repo = GitRepo::open(&current_dir).map_err(|_| GitNavigatorError::NotInGitRepo)?;

    // Load file cache - only if we have index patterns
    let has_indices = args.iter().any(|arg| is_index_pattern(arg));

    let resolved_args = if has_indices {
        // Load cache to resolve indices
        #[cfg(not(test))]
        let files = load_files_cache(&git_repo.get_repo_path()).map_err(|e| {
            log::warn!("Failed to load cache: {e}");
            GitNavigatorError::cache_load_error(e)
        })?;

        #[cfg(test)]
        let files: Vec<FileEntry> = Vec::new();

        if files.is_empty() {
            print_error("No files in cache. Run 'git-navigator status' first.");
            return Err(GitNavigatorError::NoFilesAvailable);
        }

        // Process arguments and resolve indices
        process_arguments(args, &files)?
    } else {
        // No indices, pass arguments through as-is
        args
    };

    // Execute system rm command
    let output = Command::new("rm")
        .args(&resolved_args)
        .output()
        .map_err(|e| GitNavigatorError::UpdateFailed(format!("Failed to execute rm: {e}")))?;

    // Check if command succeeded
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        print_error(&format!("rm command failed: {stderr}"));
        return Err(GitNavigatorError::UpdateFailed("Remove operation failed".to_string()));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_index_pattern_single_number() {
        assert!(is_index_pattern("1"));
        assert!(is_index_pattern("42"));
        assert!(is_index_pattern("999"));
    }

    #[test]
    fn test_is_index_pattern_range() {
        assert!(is_index_pattern("1-3"));
        assert!(is_index_pattern("5-10"));
    }

    #[test]
    fn test_is_index_pattern_comma_separated() {
        assert!(is_index_pattern("1,2,3"));
        assert!(is_index_pattern("1,5,10"));
    }

    #[test]
    fn test_is_index_pattern_mixed() {
        assert!(is_index_pattern("1-3,5,7-9"));
    }

    #[test]
    fn test_is_index_pattern_not_index() {
        assert!(!is_index_pattern("/tmp/file"));
        assert!(!is_index_pattern("README.md"));
        assert!(!is_index_pattern("-r"));
        assert!(!is_index_pattern("-f"));
        assert!(!is_index_pattern("-rf"));
        assert!(!is_index_pattern("--recursive"));
        assert!(!is_index_pattern("./src"));
    }

    #[test]
    fn test_resolve_indices_single() -> Result<()> {
        let files = vec![
            FileEntry {
                index: 1,
                status: crate::core::git_status::GitStatus::Modified,
                path: PathBuf::from("src/main.rs"),
                staged: false,
            },
            FileEntry {
                index: 2,
                status: crate::core::git_status::GitStatus::Modified,
                path: PathBuf::from("src/lib.rs"),
                staged: false,
            },
        ];

        let result = resolve_indices("1", &files)?;
        assert_eq!(result, vec![PathBuf::from("src/main.rs")]);
        Ok(())
    }

    #[test]
    fn test_resolve_indices_range() -> Result<()> {
        let files = vec![
            FileEntry {
                index: 1,
                status: crate::core::git_status::GitStatus::Modified,
                path: PathBuf::from("file1.txt"),
                staged: false,
            },
            FileEntry {
                index: 2,
                status: crate::core::git_status::GitStatus::Modified,
                path: PathBuf::from("file2.txt"),
                staged: false,
            },
            FileEntry {
                index: 3,
                status: crate::core::git_status::GitStatus::Modified,
                path: PathBuf::from("file3.txt"),
                staged: false,
            },
        ];

        let result = resolve_indices("1-3", &files)?;
        assert_eq!(
            result,
            vec![
                PathBuf::from("file1.txt"),
                PathBuf::from("file2.txt"),
                PathBuf::from("file3.txt")
            ]
        );
        Ok(())
    }

    #[test]
    fn test_resolve_indices_out_of_bounds() {
        let files = vec![FileEntry {
            index: 1,
            status: crate::core::git_status::GitStatus::Modified,
            path: PathBuf::from("file1.txt"),
            staged: false,
        }];

        let result = resolve_indices("5", &files);
        assert!(result.is_err());
    }

    #[test]
    fn test_process_arguments_only_indices() -> Result<()> {
        let files = vec![
            FileEntry {
                index: 1,
                status: crate::core::git_status::GitStatus::Modified,
                path: PathBuf::from("src/main.rs"),
                staged: false,
            },
            FileEntry {
                index: 2,
                status: crate::core::git_status::GitStatus::Modified,
                path: PathBuf::from("src/lib.rs"),
                staged: false,
            },
        ];

        let args = vec!["1".to_string()];
        let result = process_arguments(args, &files)?;
        assert_eq!(result, vec!["src/main.rs"]);
        Ok(())
    }

    #[test]
    fn test_process_arguments_with_flags() -> Result<()> {
        let files = vec![FileEntry {
            index: 1,
            status: crate::core::git_status::GitStatus::Modified,
            path: PathBuf::from("src/dir"),
            staged: false,
        }];

        let args = vec!["-rf".to_string(), "1".to_string()];
        let result = process_arguments(args, &files)?;
        assert_eq!(result, vec!["-rf", "src/dir"]);
        Ok(())
    }

    #[test]
    fn test_process_arguments_multiple_flags() -> Result<()> {
        let files = vec![FileEntry {
            index: 1,
            status: crate::core::git_status::GitStatus::Modified,
            path: PathBuf::from("test.txt"),
            staged: false,
        }];

        let args = vec!["-f".to_string(), "-v".to_string(), "1".to_string()];
        let result = process_arguments(args, &files)?;
        assert_eq!(result, vec!["-f", "-v", "test.txt"]);
        Ok(())
    }

    #[test]
    fn test_process_arguments_no_indices() -> Result<()> {
        let files = vec![FileEntry {
            index: 1,
            status: crate::core::git_status::GitStatus::Modified,
            path: PathBuf::from("dummy.txt"),
            staged: false,
        }];

        let args = vec!["file1.txt".to_string()];
        let result = process_arguments(args, &files)?;
        assert_eq!(result, vec!["file1.txt"]);
        Ok(())
    }

    #[test]
    fn test_process_arguments_mixed_with_literal_paths() -> Result<()> {
        let files = vec![
            FileEntry {
                index: 1,
                status: crate::core::git_status::GitStatus::Modified,
                path: PathBuf::from("src/main.rs"),
                staged: false,
            },
            FileEntry {
                index: 2,
                status: crate::core::git_status::GitStatus::Modified,
                path: PathBuf::from("src/lib.rs"),
                staged: false,
            },
        ];

        let args = vec!["1".to_string(), "other.txt".to_string()];
        let result = process_arguments(args, &files)?;
        assert_eq!(result, vec!["other.txt", "src/main.rs"]);
        Ok(())
    }
}
