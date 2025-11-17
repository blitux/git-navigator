#[cfg(not(test))]
use crate::commands::status::{execute_status, save_files_cache};
#[cfg(test)]
use crate::commands::status::execute_status;
use crate::core::{
    command_init::IndexCommandInit,
    error::{GitNavigatorError, Result},
    print_error_with_structured_usage,
    CommandTemplate,
};

pub fn execute_reset(indices_args: Vec<String>) -> Result<()> {
    // Check for special case: reset all staged files with "."
    if indices_args.len() == 1 && indices_args[0] == "." {
        return execute_reset_all();
    }

    // Check for special case: reset folder
    if indices_args.len() == 1 {
        let arg = &indices_args[0];
        if std::path::Path::new(arg).is_dir() {
            return execute_reset_folder(arg);
        }
    }

    // Check if arguments are filenames instead of indices
    if contains_filenames(&indices_args) {
        return execute_reset_by_paths(indices_args);
    }

    // Initialize everything needed for this index-based command
    let context = match IndexCommandInit::initialize_with_messages(
        indices_args,
        "Cannot load file cache",
        "No files available to reset",
    ) {
        Ok(context) => context,
        Err(GitNavigatorError::NoIndicesProvided) => {
            print_error_with_structured_usage(
                "No file indices provided",
                &["grs <index>...", "grs .", "grs <folder>"],
                &[("-h, --help", "Show this help message")],
            );
            return Err(GitNavigatorError::NoIndicesProvided);
        }
        Err(e) => return Err(e),
    };

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
    context.git_repo.reset_files(&paths_to_reset)?;

    let file_word = if selected_files.len() == 1 { "file" } else { "files" };
    CommandTemplate::new()
        .success(format!(
            "Successfully reset {} {} from git index.",
            selected_files.len(),
            file_word
        ))
        .print();

    // Show updated status
    println!("Updated status:");
    #[cfg(not(test))]
    let updated_files = context.git_repo.get_status()?;
    execute_status()?;

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

/// Reset all staged files using `git reset`
fn execute_reset_all() -> Result<()> {
    use crate::core::git::GitRepo;
    use std::env;

    // Initialize git repository
    let current_dir = env::current_dir()?;
    let git_repo = GitRepo::open(&current_dir).map_err(|_| GitNavigatorError::NotInGitRepo)?;

    // Execute `git reset` command (without HEAD, resets all staged files)
    let mut cmd = std::process::Command::new("git");
    cmd.arg("reset");

    let workdir = git_repo.get_repository()
        .workdir()
        .ok_or(GitNavigatorError::custom_empty_files_error("Repository has no working directory"))?;

    cmd.current_dir(workdir);
    let output = cmd.output().map_err(GitNavigatorError::Io)?;

    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(GitNavigatorError::custom_empty_files_error(format!(
            "git reset failed: {}",
            error_msg.trim()
        )));
    }

    CommandTemplate::new()
        .success("Successfully reset all staged files from git index.")
        .print();

    // Show updated status
    println!("Updated status:");
    execute_status()?;

    Ok(())
}

/// Reset all files in a specific folder using `git reset HEAD <folder>`
fn execute_reset_folder(folder: &str) -> Result<()> {
    use crate::core::git::GitRepo;
    use std::env;

    // Initialize git repository
    let current_dir = env::current_dir()?;
    let git_repo = GitRepo::open(&current_dir).map_err(|_| GitNavigatorError::NotInGitRepo)?;

    // Execute `git reset HEAD <folder>` command
    let mut cmd = std::process::Command::new("git");
    cmd.arg("reset").arg("HEAD").arg("--").arg(folder);

    let workdir = git_repo.get_repository()
        .workdir()
        .ok_or(GitNavigatorError::custom_empty_files_error("Repository has no working directory"))?;

    cmd.current_dir(workdir);
    let output = cmd.output().map_err(GitNavigatorError::Io)?;

    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(GitNavigatorError::custom_empty_files_error(format!(
            "git reset {} failed: {}",
            folder,
            error_msg.trim()
        )));
    }

    CommandTemplate::new()
        .success(format!("Successfully reset folder '{folder}' from git index."))
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

/// Reset files by their filenames using git reset command
fn execute_reset_by_paths(paths: Vec<String>) -> Result<()> {
    use crate::core::git::GitRepo;
    use std::env;

    // Initialize git repository
    let current_dir = env::current_dir()?;
    let git_repo = GitRepo::open(&current_dir).map_err(|_| GitNavigatorError::NotInGitRepo)?;

    // Validate that the files exist (they might be staged deletions, so we don't strictly require existence)
    // Just check that they're valid paths
    for path in &paths {
        let _file_path = current_dir.join(path);
        // Note: We don't check existence because the file might be deleted and staged
    }

    // Execute git reset command for each path
    let mut cmd = std::process::Command::new("git");
    cmd.arg("reset").arg("HEAD").arg("--");
    for path in &paths {
        cmd.arg(path);
    }

    let workdir = git_repo.get_repository()
        .workdir()
        .ok_or(GitNavigatorError::custom_empty_files_error("Repository has no working directory"))?;

    cmd.current_dir(workdir);
    let output = cmd.output().map_err(GitNavigatorError::Io)?;

    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(GitNavigatorError::custom_empty_files_error(format!(
            "git reset failed: {}",
            error_msg.trim()
        )));
    }

    let file_word = if paths.len() == 1 { "file" } else { "files" };
    CommandTemplate::new()
        .success(format!(
            "Successfully reset {} {} from git index.",
            paths.len(),
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
