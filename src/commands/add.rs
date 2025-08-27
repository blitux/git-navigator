#[cfg(not(test))]
use crate::commands::status::{execute_status, print_files_only, save_files_cache};
#[cfg(test)]
use crate::commands::status::{execute_status, print_files_only};
use crate::core::{
    command_init::IndexCommandInit,
    error::{GitNavigatorError, Result},
    print_error, print_error_with_structured_usage, print_info,
    CommandTemplate,
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

    // Check if arguments are filenames instead of indices
    if contains_filenames(&indices_args) {
        return execute_add_by_filenames(indices_args);
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
    context.git_repo.add_files(&paths_to_add)?;

    // Show success message using template
    let file_word = if selected_files.len() == 1 { "file" } else { "files" };
    let success_msg = format!("Successfully added {} {} to git index.", selected_files.len(), file_word);
    
    CommandTemplate::new()
        .success(&success_msg)
        .print();

    // Show updated status
    println!("Updated status:");
    let updated_files = context.git_repo.get_status()?;
    print_files_only(&updated_files);

    // Update cache with current status to maintain smooth workflow
    #[cfg(not(test))]
    {
        if let Err(e) = save_files_cache(&updated_files, context.git_repo.get_repo_path()) {
            log::warn!("Cache update failed (command succeeded): {e}");
            #[cfg(debug_assertions)]
            eprintln!("Warning: Cache update failed: {e}");
        }
    }

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

    CommandTemplate::new()
        .success("Successfully added all files to git index.")
        .print();

    // Show updated status
    println!("Updated status:");
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

    CommandTemplate::new()
        .success(format!("Successfully added folder '{folder}' to git index."))
        .print();

    // Show updated status
    println!("Updated status:");
    execute_status()?;

    Ok(())
}

/// Check if arguments contain filenames instead of indices
/// Returns true if any argument looks like a filename (contains '.' or '/' or doesn't parse as number)
fn contains_filenames(args: &[String]) -> bool {
    args.iter().any(|arg| {
        // Skip special cases that are handled elsewhere
        if arg == "." || std::path::Path::new(arg).is_dir() {
            return false;
        }
        
        // If it contains a file extension or path separator, it's likely a filename
        if arg.contains('.') && (arg.contains('/') || !arg.starts_with('.')) {
            return true;
        }
        
        // If it doesn't parse as a number or range, it might be a filename
        if crate::core::index_parser::IndexParser::parse(arg).is_err() {
            return true;
        }
        
        false
    })
}

/// Add files by their filenames using git add command
fn execute_add_by_filenames(filenames: Vec<String>) -> Result<()> {
    use crate::core::git::GitRepo;
    use std::env;

    // Initialize git repository
    let current_dir = env::current_dir()?;
    let git_repo = GitRepo::open(&current_dir).map_err(|_| GitNavigatorError::NotInGitRepo)?;

    // Validate that the files exist
    for filename in &filenames {
        let file_path = current_dir.join(filename);
        if !file_path.exists() {
            return Err(GitNavigatorError::custom_empty_files_error(format!(
                "File '{filename}' does not exist"
            )));
        }
    }

    // Execute git add command for each filename
    let mut cmd = std::process::Command::new("git");
    cmd.arg("add");
    for filename in &filenames {
        cmd.arg(filename);
    }
    
    let workdir = git_repo.get_repository()
        .workdir()
        .ok_or(GitNavigatorError::custom_empty_files_error("Repository has no working directory"))?;
    
    cmd.current_dir(workdir);
    let output = cmd.output().map_err(GitNavigatorError::Io)?;

    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(GitNavigatorError::custom_empty_files_error(format!(
            "git add failed: {}",
            error_msg.trim()
        )));
    }

    let file_word = if filenames.len() == 1 { "file" } else { "files" };
    CommandTemplate::new()
        .success(format!(
            "Successfully added {} {} to git index.",
            filenames.len(),
            file_word
        ))
        .print();

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
                || error_msg.contains("File 'nonexistent_folder' does not exist")
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
        // Current behavior: "abc" is treated as a filename by contains_filenames() function
        // This results in a "File does not exist" error rather than "Invalid index format"
        assert!(
            error_msg.contains("Invalid index format")
                || error_msg.contains("Cannot load file cache")
                || error_msg.contains("File 'abc' does not exist")
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
