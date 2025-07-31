use assert_cmd::prelude::*;
use git_navigator::core::git_status::GitStatus;
use predicates::prelude::*;
use std::process::Command;

mod common;
use common::{assertions, fixtures::*, repository::*};
use git_navigator::core::state::FileEntry;

#[cfg(test)]
mod status_command_tests {
    use super::*;

    #[test]
    fn test_gs_shows_numbered_modified_files() -> anyhow::Result<()> {
        let repo = setup_test_repo_with_initial_commit()?;

        // Modify the committed file
        create_file(&repo.path, "initial.txt", "modified content")?;

        let mut cmd = Command::cargo_bin("git-navigator")?;

        cmd.arg("status")
            .current_dir(&repo.path)
            .assert()
            .success()
            .stdout(assertions::has_branch_info())
            .stdout(assertions::has_parent_info())
            .stdout(assertions::has_status("modified"))
            .stdout(assertions::has_file_index(1))
            .stdout(predicate::str::contains("initial.txt"));

        Ok(())
    }

    #[test]
    fn test_gs_shows_numbered_untracked_files() -> anyhow::Result<()> {
        let repo = setup_test_repo()?;

        // Create untracked file
        create_file(&repo.path, "newfile.txt", "new content")?;

        let mut cmd = Command::cargo_bin("git-navigator")?;

        cmd.arg("status")
            .current_dir(&repo.path)
            .assert()
            .success()
            .stdout(assertions::has_branch_info())
            .stdout(assertions::has_parent_info())
            .stdout(assertions::has_status("untracked"))
            .stdout(assertions::has_file_index(1))
            .stdout(predicate::str::contains("newfile.txt"));

        Ok(())
    }

    #[test]
    fn test_gs_shows_multiple_files_with_indices() -> anyhow::Result<()> {
        let repo = create_multi_file_repo()?;

        let mut cmd = Command::cargo_bin("git-navigator")?;
        let output = cmd.arg("status").current_dir(&repo.path).assert().success();

        // Check that all files are shown with proper indices
        output
            .stdout(assertions::has_branch_info())
            .stdout(assertions::has_parent_info())
            .stdout(assertions::has_file_index(1))
            .stdout(assertions::has_file_index(2))
            .stdout(assertions::has_file_index(3));

        Ok(())
    }

    #[test]
    fn test_gs_shows_deleted_files() -> anyhow::Result<()> {
        let repo = setup_test_repo()?;

        // Create and commit file
        create_file(&repo.path, "to_delete.txt", "will be deleted")?;
        git_add(&repo.path, "to_delete.txt")?;
        git_commit(&repo.path, "Add file to delete")?;

        // Delete the file
        remove_file(&repo.path, "to_delete.txt")?;

        let mut cmd = Command::cargo_bin("git-navigator")?;

        cmd.arg("status")
            .current_dir(&repo.path)
            .assert()
            .success()
            .stdout(assertions::has_branch_info())
            .stdout(assertions::has_parent_info())
            .stdout(assertions::has_status("deleted"))
            .stdout(assertions::has_file_index(1))
            .stdout(predicate::str::contains("to_delete.txt"));

        Ok(())
    }

    #[test]
    fn test_gs_empty_repository() -> anyhow::Result<()> {
        let repo = setup_test_repo()?;

        let mut cmd = Command::cargo_bin("git-navigator")?;
        cmd.arg("status")
            .current_dir(&repo.path)
            .assert()
            .success()
            .stdout(assertions::has_branch_info())
            .stdout(assertions::has_parent_info());

        Ok(())
    }

    #[test]
    fn test_gs_not_in_git_repo() -> anyhow::Result<()> {
        // Use completely independent temp directory to avoid git discovery
        use tempfile::TempDir;
        let temp_dir = TempDir::new()?;
        let non_repo_path = temp_dir.path().join("not-a-repo");
        std::fs::create_dir(&non_repo_path)?;

        let mut cmd = Command::cargo_bin("git-navigator")?;
        cmd.arg("status")
            .current_dir(non_repo_path)
            .assert()
            .failure()
            .stdout(assertions::not_in_git_repo());

        Ok(())
    }
}

#[cfg(test)]
mod file_entry_tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_file_entry_creation() {
        let entry = FileEntry {
            index: 1,
            status: GitStatus::Modified,
            path: PathBuf::from("src/main.rs"),
            staged: false,
        };

        assert_eq!(entry.index, 1);
        assert_eq!(entry.status, GitStatus::Modified);
        assert_eq!(entry.path, PathBuf::from("src/main.rs"));
        assert!(!entry.staged);
    }

    #[test]
    fn test_file_entry_serialization() -> anyhow::Result<()> {
        let entry = FileEntry {
            index: 2,
            status: GitStatus::Untracked,
            path: PathBuf::from("newfile.txt"),
            staged: false,
        };

        let json = serde_json::to_string(&entry)?;
        let deserialized: FileEntry = serde_json::from_str(&json)?;

        assert_eq!(entry, deserialized);
        Ok(())
    }
}
