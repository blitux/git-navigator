use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

mod common;
use common::{assertions, repository::*};
use git_navigator::core::git::GitRepo;

#[cfg(test)]
mod branches_command_tests {
    use super::*;

    #[test]
    fn test_gb_lists_branches() -> anyhow::Result<()> {
        let repo = setup_test_repo_with_initial_commit()?;

        // Create additional branches
        let git_repo = GitRepo::open(&repo.path)?;
        git_repo.create_branch("feature-branch")?;
        git_repo.create_branch("hotfix-branch")?;
        git_repo.checkout_branch("main")?;

        let mut cmd = Command::cargo_bin("git-navigator")?;
        let output = cmd
            .arg("branches")
            .current_dir(&repo.path)
            .assert()
            .success();

        // Check that branches are listed with proper format
        output
            .stdout(predicate::str::contains("[*] main")) // Current branch
            .stdout(predicate::str::contains("[1] feature-branch"))
            .stdout(predicate::str::contains("[2] hotfix-branch"));

        Ok(())
    }

    #[test]
    fn test_gb_checkout_branch_by_index() -> anyhow::Result<()> {
        let repo = setup_test_repo_with_initial_commit()?;

        // Create additional branch
        let git_repo = GitRepo::open(&repo.path)?;
        git_repo.create_branch("feature-branch")?;
        git_repo.checkout_branch("main")?;

        // Run gb first to cache branches
        let mut cmd = Command::cargo_bin("git-navigator")?;
        cmd.arg("branches")
            .current_dir(&repo.path)
            .assert()
            .success();

        // Now checkout branch by index
        let mut cmd = Command::cargo_bin("git-navigator")?;
        cmd.arg("branches")
            .arg("1") // feature-branch should be index 1
            .current_dir(&repo.path)
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "Switched to branch 'feature-branch'",
            ));

        Ok(())
    }

    #[test]
    fn test_gb_checkout_current_branch_fails() -> anyhow::Result<()> {
        let repo = setup_test_repo_with_initial_commit()?;

        // Create additional branch
        let git_repo = GitRepo::open(&repo.path)?;
        git_repo.create_branch("feature-branch")?;

        // Run gb first to cache branches
        let mut cmd = Command::cargo_bin("git-navigator")?;
        cmd.arg("branches")
            .current_dir(&repo.path)
            .assert()
            .success();

        // Try to checkout current branch (which is not numbered)
        let mut cmd = Command::cargo_bin("git-navigator")?;
        cmd.arg("branches")
            .arg("0") // Invalid index
            .current_dir(&repo.path)
            .assert()
            .failure()
            .stdout(predicate::str::contains(
                "Cannot switch to current branch. Run 'gs' first to see available files.",
            ));

        Ok(())
    }

    #[test]
    fn test_gb_invalid_index_fails() -> anyhow::Result<()> {
        let repo = setup_test_repo_with_initial_commit()?;

        // Create additional branch
        let git_repo = GitRepo::open(&repo.path)?;
        git_repo.create_branch("feature-branch")?;

        // Run gb first to cache branches
        let mut cmd = Command::cargo_bin("git-navigator")?;
        cmd.arg("branches")
            .current_dir(&repo.path)
            .assert()
            .success();

        // Try to checkout invalid index
        let mut cmd = Command::cargo_bin("git-navigator")?;
        cmd.arg("branches")
            .arg("5") // Invalid index
            .current_dir(&repo.path)
            .assert()
            .failure()
            .stdout(predicate::str::contains("Branch index 5 not found"));

        Ok(())
    }

    #[test]
    fn test_gb_not_in_git_repo() -> anyhow::Result<()> {
        // Use completely independent temp directory to avoid git discovery
        use tempfile::TempDir;
        let temp_dir = TempDir::new()?;
        let non_repo_path = temp_dir.path().join("not-a-repo");
        std::fs::create_dir(&non_repo_path)?;

        let mut cmd = Command::cargo_bin("git-navigator")?;
        cmd.arg("branches")
            .current_dir(non_repo_path)
            .assert()
            .failure()
            .stdout(assertions::not_in_git_repo());

        Ok(())
    }

    #[test]
    fn test_gb_no_cached_branches() -> anyhow::Result<()> {
        let repo = setup_test_repo_with_initial_commit()?;

        // Try to checkout without running gb first (no cache)
        let mut cmd = Command::cargo_bin("git-navigator")?;
        cmd.arg("branches")
            .arg("1")
            .current_dir(&repo.path)
            .assert()
            .failure()
            .stdout(predicate::str::contains("Cannot load branch cache"));

        Ok(())
    }
}

#[cfg(test)]
mod branch_utilities {
    use super::*;

    #[test]
    fn test_create_branch() -> anyhow::Result<()> {
        let repo = setup_test_repo_with_initial_commit()?;

        // Create a new branch
        let git_repo = GitRepo::open(&repo.path)?;
        git_repo.create_branch("test-branch")?;

        // Verify branch exists
        let output = std::process::Command::new("git")
            .args(["branch"])
            .current_dir(&repo.path)
            .output()
            .map_err(|e| anyhow::anyhow!("Failed to execute git branch: {}", e))?;

        let branches = String::from_utf8_lossy(&output.stdout);
        assert!(branches.contains("test-branch"));

        Ok(())
    }
}
