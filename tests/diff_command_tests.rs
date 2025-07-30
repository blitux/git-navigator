use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

mod common;
use common::{assertions, fixtures::*, repository::*};

#[cfg(test)]
mod diff_command_tests {
    use super::*;

    #[test]
    fn test_gd_no_indices() -> anyhow::Result<()> {
        let repo = setup_test_repo()?;

        let mut cmd = Command::cargo_bin("git-navigator")?;
        cmd.arg("diff")
            .current_dir(&repo.path)
            .assert()
            .failure()
            .stdout(
                predicate::str::contains("No file indices provided").or(assertions::cache_error()),
            );

        Ok(())
    }

    #[test]
    fn test_gd_invalid_indices() -> anyhow::Result<()> {
        let repo = setup_test_repo()?;

        let mut cmd = Command::cargo_bin("git-navigator")?;
        cmd.arg("diff")
            .arg("invalid")
            .current_dir(&repo.path)
            .assert()
            .failure()
            .stdout(predicate::str::contains("Error"));

        Ok(())
    }

    #[test]
    fn test_gd_not_in_git_repo() -> anyhow::Result<()> {
        // Use completely independent temp directory to avoid git discovery
        use tempfile::TempDir;
        let temp_dir = TempDir::new()?;
        let non_repo_path = temp_dir.path().join("not-a-repo");
        std::fs::create_dir(&non_repo_path)?;

        let mut cmd = Command::cargo_bin("git-navigator")?;
        cmd.arg("diff")
            .arg("1")
            .current_dir(non_repo_path)
            .assert()
            .failure()
            .stdout(assertions::not_in_git_repo());

        Ok(())
    }

    #[test]
    fn test_gd_no_cached_files() -> anyhow::Result<()> {
        let repo = setup_test_repo()?;

        // Create a file but don't run gs first
        create_file(&repo.path, "test.txt", "test content")?;

        let mut cmd = Command::cargo_bin("git-navigator")?;
        cmd.arg("diff")
            .arg("1")
            .current_dir(&repo.path)
            .assert()
            .failure()
            .stderr(assertions::cache_error());

        Ok(())
    }

    #[test]
    fn test_gd_with_modified_file() -> anyhow::Result<()> {
        let repo = setup_test_repo()?;

        // Create and commit initial file
        create_file(&repo.path, "file1.txt", "initial content\nline 2\n")?;
        git_add(&repo.path, "file1.txt")?;
        git_commit(&repo.path, "Initial commit")?;

        // Modify the file
        create_file(
            &repo.path,
            "file1.txt",
            "modified content\nline 2\nnew line\n",
        )?;

        // Run gs first to cache files
        run_status_to_cache(&repo.path)?;

        // Now run gd
        let mut cmd = Command::cargo_bin("git-navigator")?;
        cmd.arg("diff")
            .arg("1")
            .current_dir(&repo.path)
            .assert()
            .success()
            .stdout(predicate::str::contains("Showing diff for"))
            .stdout(predicate::str::contains("[1] file1.txt"))
            .stdout(predicate::str::contains("diff --git"))
            .stdout(predicate::str::contains("+")) // Should have additions
            .stdout(predicate::str::contains("-")); // Should have deletions

        Ok(())
    }

    #[test]
    fn test_gd_with_untracked_file() -> anyhow::Result<()> {
        let repo = setup_test_repo()?;

        // Create untracked file
        create_file(&repo.path, "newfile.txt", "new content\nline 2\n")?;

        // Run gs first to cache files
        run_status_to_cache(&repo.path)?;

        // Now run gd
        let mut cmd = Command::cargo_bin("git-navigator")?;
        cmd.arg("diff")
            .arg("1")
            .current_dir(&repo.path)
            .assert()
            .success()
            .stdout(predicate::str::contains("Showing diff for"))
            .stdout(predicate::str::contains("[1] newfile.txt"))
            .stdout(predicate::str::contains("untracked"));

        Ok(())
    }

    #[test]
    fn test_gd_with_multiple_files() -> anyhow::Result<()> {
        let repo = create_multi_file_repo()?;

        // Run gs first to cache files
        run_status_to_cache(&repo.path)?;

        // Now run gd with multiple indices
        let mut cmd = Command::cargo_bin("git-navigator")?;
        cmd.arg("diff")
            .arg("1,3")
            .current_dir(&repo.path)
            .assert()
            .success()
            .stdout(predicate::str::contains("Showing diff for 2 file(s)"))
            .stdout(predicate::str::contains("[1]"))
            .stdout(predicate::str::contains("[3]"))
            .stdout(predicate::str::contains("═══")); // Should have file separators

        Ok(())
    }

    #[test]
    fn test_gd_with_range() -> anyhow::Result<()> {
        let repo = setup_test_repo()?;

        // Create and commit initial files
        create_test_files(&repo.path, &["file1.txt", "file2.txt", "file3.txt"])?;
        git_add(&repo.path, ".")?;
        git_commit(&repo.path, "Initial commit")?;

        // Modify all files
        modify_test_files(&repo.path, &["file1.txt", "file2.txt", "file3.txt"])?;

        // Run gs first to cache files
        run_status_to_cache(&repo.path)?;

        // Now run gd with range
        let mut cmd = Command::cargo_bin("git-navigator")?;
        cmd.arg("diff")
            .arg("1-3")
            .current_dir(&repo.path)
            .assert()
            .success()
            .stdout(predicate::str::contains("Showing diff for 3 file(s)"))
            .stdout(predicate::str::contains("[1]"))
            .stdout(predicate::str::contains("[2]"))
            .stdout(predicate::str::contains("[3]"))
            .stdout(predicate::str::contains("═══")); // Should have file separators

        Ok(())
    }

    #[test]
    fn test_gd_index_out_of_bounds() -> anyhow::Result<()> {
        let repo = setup_test_repo()?;

        // Create one file
        create_file(&repo.path, "file1.txt", "content\n")?;

        // Run gs first to cache files
        run_status_to_cache(&repo.path)?;

        // Try to diff index 5 (out of bounds)
        let mut cmd = Command::cargo_bin("git-navigator")?;
        cmd.arg("diff")
            .arg("5")
            .current_dir(&repo.path)
            .assert()
            .failure()
            .stdout(predicate::str::contains("Error"));

        Ok(())
    }
}
