use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

mod common;
use common::repository::*;
use git_navigator::core::git::GitRepo;

#[cfg(test)]
mod checkout_command_tests {
    use super::*;

    #[test]
    fn test_gco_checkout_files_by_index() -> anyhow::Result<()> {
        let repo = setup_test_repo()?;

        // Create the file first
        std::fs::write(repo.path.join("file1.txt"), "original content")?;

        // Add some files to index first
        std::process::Command::new("git")
            .args(["add", "file1.txt"])
            .current_dir(&repo.path)
            .output()?;

        // Modify the file again to create both staged and unstaged changes
        std::fs::write(repo.path.join("file1.txt"), "modified again content")?;

        // Run gs to cache files
        let mut cmd = Command::cargo_bin("git-navigator")?;
        cmd.arg("status").current_dir(&repo.path).assert().success();

        // Checkout file by index (should restore to staged version)
        let mut cmd = Command::cargo_bin("git-navigator")?;
        cmd.arg("checkout")
            .arg("1")
            .current_dir(&repo.path)
            .assert()
            .success()
            .stdout(predicate::str::contains("Successfully checked out"))
            .stdout(predicate::str::contains("1 file(s)"));

        Ok(())
    }

    #[test]
    fn test_gco_checkout_branch_by_name() -> anyhow::Result<()> {
        let repo = setup_test_repo_with_initial_commit()?;

        // Create a test branch
        let git_repo = GitRepo::open(&repo.path)?;
        git_repo.create_branch("test-branch")?;
        git_repo.checkout_branch("main")?;

        // Checkout branch by name
        let mut cmd = Command::cargo_bin("git-navigator")?;
        cmd.arg("checkout")
            .arg("test-branch")
            .current_dir(&repo.path)
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "Successfully switched to branch 'test-branch'",
            ));

        Ok(())
    }

    #[test]
    fn test_gco_create_and_checkout_branch() -> anyhow::Result<()> {
        let repo = setup_test_repo_with_initial_commit()?;

        // Create and checkout new branch
        let mut cmd = Command::cargo_bin("git-navigator")?;
        cmd.arg("checkout")
            .arg("-b")
            .arg("new-feature")
            .current_dir(&repo.path)
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "Successfully created and switched to branch 'new-feature'",
            ));

        Ok(())
    }

    #[test]
    fn test_gco_no_arguments() -> anyhow::Result<()> {
        let repo = setup_test_repo_with_initial_commit()?;

        let mut cmd = Command::cargo_bin("git-navigator")?;
        cmd.arg("checkout")
            .current_dir(&repo.path)
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "No file indices or branch name provided",
            ))
            .stdout(predicate::str::contains("Usage:"));

        Ok(())
    }

    #[test]
    fn test_gco_incomplete_branch_creation() -> anyhow::Result<()> {
        let repo = setup_test_repo_with_initial_commit()?;

        let mut cmd = Command::cargo_bin("git-navigator")?;
        cmd.arg("checkout")
            .arg("-b")
            .current_dir(&repo.path)
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "Branch name required with -b flag",
            ))
            .stdout(predicate::str::contains("Usage:"));

        Ok(())
    }

    #[test]
    fn test_gco_checkout_invalid_branch() -> anyhow::Result<()> {
        let repo = setup_test_repo_with_initial_commit()?;

        let mut cmd = Command::cargo_bin("git-navigator")?;
        cmd.arg("checkout")
            .arg("nonexistent-branch")
            .current_dir(&repo.path)
            .assert()
            .failure()
            .stdout(predicate::str::contains(
                "Failed to checkout branch 'nonexistent-branch'",
            ));

        Ok(())
    }

    #[test]
    fn test_gco_mixed_indices() -> anyhow::Result<()> {
        let repo = setup_test_repo()?;

        // Create multiple files and add them to git
        std::fs::write(repo.path.join("file1.txt"), "content1")?;
        std::fs::write(repo.path.join("file2.txt"), "content2")?;
        std::fs::write(repo.path.join("file3.txt"), "content3")?;

        std::process::Command::new("git")
            .args(["add", "."])
            .current_dir(&repo.path)
            .output()?;

        // Modify files to create unstaged changes
        std::fs::write(repo.path.join("file1.txt"), "modified1")?;
        std::fs::write(repo.path.join("file2.txt"), "modified2")?;
        std::fs::write(repo.path.join("file3.txt"), "modified3")?;

        // Run gs to cache files
        let mut cmd = Command::cargo_bin("git-navigator")?;
        cmd.arg("status").current_dir(&repo.path).assert().success();

        // Checkout multiple files by index
        let mut cmd = Command::cargo_bin("git-navigator")?;
        cmd.arg("checkout")
            .arg("1,3")
            .current_dir(&repo.path)
            .assert()
            .success()
            .stdout(predicate::str::contains("Successfully checked out"))
            .stdout(predicate::str::contains("file(s)"));

        Ok(())
    }

    // Note: is_numeric_index is a private function, so we test it through the public API
    // by testing the behavior differences between numeric and branch arguments
}
