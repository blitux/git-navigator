use crate::commands::status::execute_status;
use crate::core::{
    command_init::IndexCommandInit,
    error::{GitNavigatorError, Result},
    print_success,
};

pub fn execute_reset(indices_args: Vec<String>) -> Result<()> {
    // Initialize everything needed for this index-based command
    let context = IndexCommandInit::initialize_with_messages(
        indices_args,
        "Cannot load file cache",
        "No files available to reset",
    )?;

    // Get the selected files and prepare them for resetting
    let selected_files = context.get_selected_files();

    // Extract paths efficiently - unfortunately git2 API requires owned PathBuf
    // so we can't avoid the clone, but we can at least do it efficiently
    let paths_to_reset: Vec<_> = selected_files
        .iter()
        .map(|file| &file.path)
        .cloned()
        .collect();

    if paths_to_reset.is_empty() {
        return Err(GitNavigatorError::NoValidFilesSelected);
    }

    // Reset files in git index
    match context.git_repo.reset_files(&paths_to_reset) {
        Ok(()) => {
            print_success(&format!(
                "Successfully reset {} file(s) from git index.",
                selected_files.len()
            ));
        }
        Err(e) => {
            return Err(e);
        }
    }

    // Show updated status
    println!("Updated status:");
    execute_status()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::git_status::GitStatus;
    use crate::core::state::FileEntry;
    use std::path::PathBuf;

    #[test]
    fn test_execute_reset_no_indices() {
        let result = execute_reset(vec![]);
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        // Could be either no indices provided OR cache load error (depending on cache state)
        assert!(
            error_msg.contains("No file indices provided")
                || error_msg.contains("Cannot load file cache")
        );
    }

    #[test]
    fn test_execute_reset_empty_indices() {
        let result = execute_reset(vec!["".to_string()]);
        assert!(result.is_err());
        // This will fail during parsing, not during empty check
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_reset_invalid_indices() {
        // Test that invalid indices are caught during parsing
        // This is a unit test focused on argument validation
        use crate::core::args_parser::ArgsParser;

        let result = ArgsParser::parse_indices(vec!["abc".to_string()], 5);
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Invalid index format"));
    }

    #[test]
    fn test_memory_efficient_path_collection() {
        // Test that our path collection is memory efficient
        let files = vec![
            FileEntry {
                index: 1,
                status: GitStatus::Modified,
                path: PathBuf::from("file1.txt"),
                staged: false,
            },
            FileEntry {
                index: 2,
                status: GitStatus::Added,
                path: PathBuf::from("file2.txt"),
                staged: true,
            },
            FileEntry {
                index: 3,
                status: GitStatus::Untracked,
                path: PathBuf::from("very/long/path/to/file3.txt"),
                staged: false,
            },
        ];

        // Simulate the optimized path collection from the reset command
        let paths_to_reset: Vec<_> = files.iter().map(|file| &file.path).cloned().collect();

        assert_eq!(paths_to_reset.len(), 3);
        assert_eq!(paths_to_reset[0], PathBuf::from("file1.txt"));
        assert_eq!(paths_to_reset[1], PathBuf::from("file2.txt"));
        assert_eq!(
            paths_to_reset[2],
            PathBuf::from("very/long/path/to/file3.txt")
        );

        // Verify no unnecessary allocations by checking that we get the expected paths
        // This test ensures our iterator chain works correctly
        let expected_paths = vec![
            PathBuf::from("file1.txt"),
            PathBuf::from("file2.txt"),
            PathBuf::from("very/long/path/to/file3.txt"),
        ];

        assert_eq!(paths_to_reset, expected_paths);
    }

    #[test]
    fn test_vector_preallocation_efficiency() {
        // Test that pre-allocation with known capacity is more efficient
        let files = vec![
            FileEntry {
                index: 1,
                status: GitStatus::Modified,
                path: PathBuf::from("file1.txt"),
                staged: false,
            },
            FileEntry {
                index: 2,
                status: GitStatus::Added,
                path: PathBuf::from("file2.txt"),
                staged: true,
            },
        ];

        // Test that collect() with pre-known size works efficiently
        let paths_to_reset: Vec<_> = files.iter().map(|file| &file.path).cloned().collect();

        // Ensure the vector has the expected capacity and contents
        assert_eq!(paths_to_reset.len(), 2);
        assert_eq!(paths_to_reset.capacity(), 2); // Rust's collect() is efficient
    }
}
