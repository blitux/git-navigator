use crate::core::{
    command_init::IndexCommandInit,
    error::{GitNavigatorError, Result},
    git::GitRepo,
    git_status::GitStatus,
    print_error_with_structured_usage,
    state::FileEntry,
};
use colored::*;

pub fn execute_diff(indices_args: Vec<String>) -> Result<()> {
    // Initialize everything needed for this index-based command
    let context = match IndexCommandInit::initialize_with_messages(
        indices_args,
        "Cannot load file cache",
        "No files found in cache",
    ) {
        Ok(context) => context,
        Err(GitNavigatorError::NoIndicesProvided) => {
            print_error_with_structured_usage(
                "No file indices provided",
                &["gd <index>..."],
                &[("-h, --help", "Show this help message")],
            );
            return Err(GitNavigatorError::NoIndicesProvided);
        }
        Err(e) => return Err(e),
    };

    // Get the files to diff
    let files_to_diff = context.get_selected_files();

    println!("Showing diff for {} file(s):", files_to_diff.len());
    for file in &files_to_diff {
        println!("  [{}] {}", file.index, file.path.display());
    }
    println!();

    // Show diff for each file
    for (i, file) in files_to_diff.iter().enumerate() {
        if files_to_diff.len() > 1 {
            if i > 0 {
                println!(); // Extra spacing between files
            }
            print!("{}", "═══ ".bright_blue().bold());
            print!("{}", file.path.to_string_lossy().bright_blue().bold());
            println!("{}", " ═══".bright_blue().bold());
        }
        show_file_diff(&context.git_repo, file)?;
    }

    Ok(())
}

fn show_file_diff(git_repo: &GitRepo, file: &FileEntry) -> Result<()> {
    let workdir = git_repo.get_repository().workdir().ok_or_else(|| {
        crate::core::error::GitNavigatorError::custom_empty_files_error("No workdir found")
    })?;

    let mut cmd = std::process::Command::new("git");
    cmd.current_dir(workdir);

    match file.status {
        GitStatus::Untracked => {
            println!(
                "File is untracked: {}. No diff to show.",
                file.path.display()
            );
            return Ok(());
        }
        GitStatus::Deleted => {
            cmd.arg("diff")
                .arg("--color")
                .arg("HEAD")
                .arg("--")
                .arg(&file.path);
        }
        _ => {
            if file.staged {
                cmd.arg("diff")
                    .arg("--cached")
                    .arg("--color")
                    .arg("HEAD")
                    .arg("--")
                    .arg(&file.path);
            } else {
                cmd.arg("diff").arg("--color").arg("--").arg(&file.path);
            }
        }
    }

    let output = cmd
        .output()
        .map_err(|e| crate::core::error::GitNavigatorError::Io(e))?;

    if output.status.success() {
        let diff_output = String::from_utf8_lossy(&output.stdout);
        if !diff_output.trim().is_empty() {
            println!("{}", diff_output);
        } else {
            println!("No changes to show for {}", file.path.display());
        }
    } else {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(
            crate::core::error::GitNavigatorError::custom_empty_files_error(&format!(
                "git diff failed: {}",
                error_msg.trim()
            )),
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::error::GitNavigatorError;
    use std::{env, fs};
    use tempfile::TempDir;

    fn setup_test_repo() -> Result<(TempDir, GitRepo)> {
        let temp_dir = TempDir::new().map_err(|e| GitNavigatorError::Io(e))?;
        let repo_path = temp_dir.path();

        std::process::Command::new("git")
            .args(["init"])
            .current_dir(&repo_path)
            .output()
            .map_err(|e| GitNavigatorError::Io(e))?;

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

        let git_repo = GitRepo::open(&repo_path)?;
        Ok((temp_dir, git_repo))
    }

    #[test]
    fn test_execute_diff_no_indices() {
        let result = execute_diff(vec![]);
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("No file indices provided")
                || error_msg.contains("Cannot load file cache")
        );
    }

    #[test]
    fn test_execute_diff_empty_indices() {
        let result = execute_diff(vec!["".to_string()]);
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_diff_invalid_indices() {
        let result = execute_diff(vec!["abc".to_string()]);
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_diff_not_in_git_repo() -> Result<()> {
        let temp_dir = TempDir::new().map_err(|e| GitNavigatorError::Io(e))?;
        let non_repo_path = temp_dir.path();

        let original_dir = env::current_dir()?;
        env::set_current_dir(non_repo_path)?;

        let result = execute_diff(vec!["1".to_string()]);

        env::set_current_dir(original_dir)?;

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Not in a git repository"));
        Ok(())
    }

    #[test]
    fn test_show_file_diff_untracked() -> Result<()> {
        let (_temp_dir, git_repo) = setup_test_repo()?;
        let repo_path = git_repo.get_repository().workdir().unwrap();

        let test_file = repo_path.join("test.txt");
        fs::write(&test_file, "test content\nline 2\n")?;

        let file_entry = FileEntry {
            index: 1,
            status: GitStatus::Untracked,
            path: "test.txt".into(),
            staged: false,
        };

        let result = show_file_diff(&git_repo, &file_entry);
        assert!(result.is_ok());

        Ok(())
    }

    #[test]
    fn test_git_diff_command_integration() -> Result<()> {
        let (_temp_dir, git_repo) = setup_test_repo()?;
        let workdir = git_repo.get_repository().workdir().unwrap();

        let test_file = workdir.join("test.txt");
        std::fs::write(&test_file, "original content\n").map_err(|e| GitNavigatorError::Io(e))?;

        std::process::Command::new("git")
            .args(["add", "test.txt"])
            .current_dir(workdir)
            .output()
            .map_err(|e| GitNavigatorError::Io(e))?;

        std::process::Command::new("git")
            .args(["commit", "-m", "Initial commit"])
            .current_dir(workdir)
            .output()
            .map_err(|e| GitNavigatorError::Io(e))?;

        std::fs::write(&test_file, "modified content\n").map_err(|e| GitNavigatorError::Io(e))?;

        let output = std::process::Command::new("git")
            .args(["diff", "--", "test.txt"])
            .current_dir(workdir)
            .output()
            .map_err(|e| GitNavigatorError::Io(e))?;

        let diff_output = String::from_utf8_lossy(&output.stdout);
        assert!(!diff_output.trim().is_empty());
        assert!(diff_output.contains("-original content"));
        assert!(diff_output.contains("+modified content"));

        Ok(())
    }

    #[test]
    fn test_diff_error_handling_logic() {
        use crate::core::git_status::GitStatus;
        use crate::core::state::FileEntry;
        use std::path::PathBuf;

        let file_entry = FileEntry {
            index: 1,
            status: GitStatus::Modified,
            path: PathBuf::from("nonexistent.txt"),
            staged: false,
        };

        assert_eq!(file_entry.path, PathBuf::from("nonexistent.txt"));
        assert_eq!(file_entry.status, GitStatus::Modified);
        assert!(!file_entry.staged);

        let path_str = file_entry.path.to_string_lossy();
        assert!(path_str.contains("nonexistent.txt"));
    }

    #[test]
    fn test_file_status_logic() {
        use crate::core::git_status::GitStatus;

        let statuses = vec![
            GitStatus::Modified,
            GitStatus::Added,
            GitStatus::Deleted,
            GitStatus::Renamed,
            GitStatus::Copied,
            GitStatus::Untracked,
            GitStatus::Unmerged,
        ];

        for status in statuses {
            let description = status.description();
            assert!(!description.is_empty());

            match status {
                GitStatus::Modified => assert_eq!(description, "modified"),
                GitStatus::Added => assert_eq!(description, "new"),
                GitStatus::Deleted => assert_eq!(description, "deleted"),
                GitStatus::Renamed => assert_eq!(description, "renamed"),
                GitStatus::Copied => assert_eq!(description, "copied"),
                GitStatus::Untracked => assert_eq!(description, "untracked"),
                GitStatus::Unmerged => assert_eq!(description, "both modified"),
                _ => {}
            }
        }
    }
}
