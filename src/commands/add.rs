use crate::commands::status::{execute_status, print_files_only};
use crate::core::{
    command_init::IndexCommandInit,
    error::{GitNavigatorError, Result},
    print_error, print_error_with_structured_usage, print_info, print_success,
};

pub fn execute_add(indices_args: Vec<String>) -> Result<()> {
    // Check for special case: add all files with "."
    if indices_args.len() == 1 && indices_args[0] == "." {
        return execute_add_all();
    }

    // Check for special case: add folder
    if indices_args.len() == 1 {
        let arg = &indices_args[0];
        if std::path::Path::new(arg).is_dir() {
            return execute_add_folder(arg);
        }
    }

    // Initialize everything needed for this index-based command
    let context = match IndexCommandInit::initialize_with_messages(
        indices_args,
        "Cannot load file cache",
        "No files available to add",
    ) {
        Ok(context) => context,
        Err(GitNavigatorError::NoIndicesProvided) => {
            print_error_with_structured_usage(
                "No file indices provided",
                &["ga <index>...", "ga .", "ga <folder>"],
                &[("-h, --help", "Show this help message")],
            );
            return Err(GitNavigatorError::NoIndicesProvided);
        }
        Err(e) => return Err(e),
    };

    // Check if there are any changes available to add
    let current_status = context.git_repo.get_status()?;
    if current_status.is_empty() {
        print_error("There are no changes to be added");
        print_info("Current status:");
        execute_status()?;
        return Ok(()); // Exit cleanly after showing formatted error
    }

    // Get the selected files and prepare them for adding
    let selected_files = context.get_selected_files();

    // Extract paths efficiently - unfortunately git2 API requires owned PathBuf
    // so we can't avoid the clone, but we can at least do it efficiently
    let paths_to_add: Vec<_> = selected_files
        .iter()
        .map(|file| &file.path)
        .cloned()
        .collect();

    if paths_to_add.is_empty() {
        return Err(GitNavigatorError::NoValidFilesSelected);
    }

    // Add files to git index
    match context.git_repo.add_files(&paths_to_add) {
        Ok(()) => {
            print_success(&format!(
                "Successfully added {} file(s) to git index.",
                selected_files.len()
            ));
        }
        Err(e) => {
            return Err(e);
        }
    }

    // Show updated status
    print_info("Updated status:");
    let updated_files = context.git_repo.get_status()?;
    print_files_only(&updated_files);

    Ok(())
}

/// Add all files to the git index using `git add .`
fn execute_add_all() -> Result<()> {
    use crate::core::git::GitRepo;
    use std::env;

    // Initialize git repository
    let current_dir = env::current_dir()?;
    let git_repo = GitRepo::open(&current_dir).map_err(|_| GitNavigatorError::NotInGitRepo)?;

    // Execute `git add .` command
    let mut cmd = std::process::Command::new("git");
    cmd.arg("add").arg(".");
    
    let workdir = git_repo.get_repository()
        .workdir()
        .ok_or(GitNavigatorError::custom_empty_files_error("Repository has no working directory"))?;
    
    cmd.current_dir(workdir);
    let output = cmd.output().map_err(GitNavigatorError::Io)?;

    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(GitNavigatorError::custom_empty_files_error(format!(
            "git add . failed: {}",
            error_msg.trim()
        )));
    }

    print_success("Successfully added all files to git index.");

    // Show updated status
    print_info("Updated status:");
    execute_status()?;

    Ok(())
}

/// Add all files in a specific folder using `git add <folder>`
fn execute_add_folder(folder: &str) -> Result<()> {
    use crate::core::git::GitRepo;
    use std::env;

    // Initialize git repository
    let current_dir = env::current_dir()?;
    let git_repo = GitRepo::open(&current_dir).map_err(|_| GitNavigatorError::NotInGitRepo)?;

    // Execute `git add <folder>` command
    let mut cmd = std::process::Command::new("git");
    cmd.arg("add").arg(folder);
    
    let workdir = git_repo.get_repository()
        .workdir()
        .ok_or(GitNavigatorError::custom_empty_files_error("Repository has no working directory"))?;
    
    cmd.current_dir(workdir);
    let output = cmd.output().map_err(GitNavigatorError::Io)?;

    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(GitNavigatorError::custom_empty_files_error(format!(
            "git add {} failed: {}",
            folder,
            error_msg.trim()
        )));
    }

    print_success(&format!("Successfully added folder '{}' to git index.", folder));

    // Show updated status
    print_info("Updated status:");
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
    fn test_execute_add_no_indices() {
        let result = execute_add(vec![]);
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        // Could be either no indices provided OR cache load error (depending on cache state)
        assert!(
            error_msg.contains("No file indices provided")
                || error_msg.contains("Cannot load file cache")
        );
    }

    #[test]
    fn test_execute_add_dot_argument() {
        // Test that "." is handled as a special case
        let result = execute_add(vec![".".to_string()]);
        // In this git repository, the command should succeed
        // because we're running in an actual git repo
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_add_dot_with_other_args() {
        // Test that "." with other arguments doesn't trigger the special case
        let result = execute_add(vec![".".to_string(), "1".to_string()]);
        assert!(result.is_err());
        // This should go through normal index parsing, not the special case
        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("Cannot load file cache")
                || error_msg.contains("Invalid index format")
                || error_msg.contains("Not in a git repository")
        );
    }

    #[test]
    fn test_execute_add_folder_argument() {
        // Test that a folder path is handled as a special case
        // Create a temporary directory for testing
        let temp_dir = std::env::temp_dir().join("test_folder_add");
        std::fs::create_dir_all(&temp_dir).unwrap();
        
        let result = execute_add(vec![temp_dir.to_string_lossy().to_string()]);
        // This will likely fail in test environment due to git repo setup,
        // but we're testing that it takes the special code path
        assert!(result.is_err());
        // The error should be from git operations, not from index parsing
        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("Not in a git repository") 
                || error_msg.contains("git add") 
                || error_msg.contains("Repository has no working directory")
        );

        // Clean up
        std::fs::remove_dir(&temp_dir).ok();
    }

    #[test]
    fn test_execute_add_nonexistent_folder() {
        // Test that non-existent folder doesn't trigger the special case
        let result = execute_add(vec!["nonexistent_folder".to_string()]);
        assert!(result.is_err());
        // This should go through normal index parsing since folder doesn't exist
        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("Cannot load file cache")
                || error_msg.contains("Invalid index format")
                || error_msg.contains("Not in a git repository")
        );
    }

    #[test]
    fn test_execute_add_folder_with_other_args() {
        // Test that folder with other arguments doesn't trigger the special case
        let temp_dir = std::env::temp_dir().join("test_folder_add_multi");
        std::fs::create_dir_all(&temp_dir).unwrap();
        
        let result = execute_add(vec![temp_dir.to_string_lossy().to_string(), "1".to_string()]);
        assert!(result.is_err());
        // This should go through normal index parsing, not the special case
        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("Cannot load file cache")
                || error_msg.contains("Invalid index format")
                || error_msg.contains("Not in a git repository")
        );

        // Clean up
        std::fs::remove_dir(&temp_dir).ok();
    }

    #[test]
    fn test_execute_add_empty_indices() {
        let result = execute_add(vec!["".to_string()]);
        assert!(result.is_err());
        // This will fail during parsing, not during empty check
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_add_invalid_indices() {
        let result = execute_add(vec!["abc".to_string()]);
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        // Could be either invalid index format OR cache load error (depending on cache state)
        assert!(
            error_msg.contains("Invalid index format")
                || error_msg.contains("Cannot load file cache")
        );
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

        // Simulate the optimized path collection from the add command
        let paths_to_add: Vec<_> = files.iter().map(|file| &file.path).cloned().collect();

        assert_eq!(paths_to_add.len(), 3);
        assert_eq!(paths_to_add[0], PathBuf::from("file1.txt"));
        assert_eq!(paths_to_add[1], PathBuf::from("file2.txt"));
        assert_eq!(
            paths_to_add[2],
            PathBuf::from("very/long/path/to/file3.txt")
        );

        // Verify no unnecessary allocations by checking that we get the expected paths
        // This test ensures our iterator chain works correctly
        let expected_paths = vec![
            PathBuf::from("file1.txt"),
            PathBuf::from("file2.txt"),
            PathBuf::from("very/long/path/to/file3.txt"),
        ];

        assert_eq!(paths_to_add, expected_paths);
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
        let paths_to_add: Vec<_> = files.iter().map(|file| &file.path).cloned().collect();

        // Ensure the vector has the expected capacity and contents
        assert_eq!(paths_to_add.len(), 2);
        assert_eq!(paths_to_add.capacity(), 2); // Rust's collect() is efficient
    }

    #[test]
    fn test_path_extraction_handles_deleted_files() {
        // Test that path extraction works correctly for deleted files
        let files = vec![
            FileEntry {
                index: 1,
                status: GitStatus::Modified,
                path: PathBuf::from("modified.txt"),
                staged: false,
            },
            FileEntry {
                index: 2,
                status: GitStatus::Deleted,
                path: PathBuf::from("deleted.txt"),
                staged: false,
            },
            FileEntry {
                index: 3,
                status: GitStatus::Added,
                path: PathBuf::from("added.txt"),
                staged: true,
            },
        ];

        // Extract paths like the add command does
        let paths_to_add: Vec<_> = files.iter().map(|file| &file.path).cloned().collect();

        assert_eq!(paths_to_add.len(), 3);
        assert_eq!(paths_to_add[0], PathBuf::from("modified.txt"));
        assert_eq!(paths_to_add[1], PathBuf::from("deleted.txt"));
        assert_eq!(paths_to_add[2], PathBuf::from("added.txt"));

        // Verify that deleted files are handled the same as other files
        // in the path extraction phase
        let deleted_file_path = &paths_to_add[1];
        assert_eq!(deleted_file_path.to_string_lossy(), "deleted.txt");
    }
}
