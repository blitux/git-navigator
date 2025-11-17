use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

mod common;
use common::repository::*;
use std::fs;

#[cfg(test)]
mod reset_command_tests {
    use super::*;

    #[test]
    fn test_reset_by_path() -> anyhow::Result<()> {
        let repo = setup_test_repo()?;

        // Create and stage a file
        fs::write(repo.path.join("test.txt"), "content")?;
        git_add(&repo.path, "test.txt")?;

        // Verify file is staged
        let output = std::process::Command::new("git")
            .args(["status", "--short"])
            .current_dir(&repo.path)
            .output()?;
        let status = String::from_utf8_lossy(&output.stdout);
        assert!(status.contains("test.txt"));

        // Reset using path
        let mut cmd = Command::cargo_bin("git-navigator")?;
        cmd.arg("reset")
            .arg("test.txt")
            .current_dir(&repo.path)
            .assert()
            .success()
            .stdout(predicate::str::contains("Successfully reset 1 file"));

        Ok(())
    }

    #[test]
    fn test_reset_dot_all_files() -> anyhow::Result<()> {
        let repo = setup_test_repo()?;

        // Create and stage multiple files
        fs::write(repo.path.join("file1.txt"), "content1")?;
        fs::write(repo.path.join("file2.txt"), "content2")?;
        git_add(&repo.path, "file1.txt")?;
        git_add(&repo.path, "file2.txt")?;

        // Reset all with "."
        let mut cmd = Command::cargo_bin("git-navigator")?;
        cmd.arg("reset")
            .arg(".")
            .current_dir(&repo.path)
            .assert()
            .success()
            .stdout(predicate::str::contains("Successfully reset all staged files"));

        Ok(())
    }

    #[test]
    fn test_reset_folder() -> anyhow::Result<()> {
        let repo = setup_test_repo()?;

        // Create folder with files and stage them
        fs::create_dir_all(repo.path.join("src"))?;
        fs::write(repo.path.join("src/file1.rs"), "content1")?;
        fs::write(repo.path.join("src/file2.rs"), "content2")?;
        git_add(&repo.path, "src/file1.rs")?;
        git_add(&repo.path, "src/file2.rs")?;

        // Reset folder
        let mut cmd = Command::cargo_bin("git-navigator")?;
        cmd.arg("reset")
            .arg("src")
            .current_dir(&repo.path)
            .assert()
            .success()
            .stdout(predicate::str::contains("Successfully reset folder 'src'"));

        Ok(())
    }

    #[test]
    fn test_reset_multiple_paths() -> anyhow::Result<()> {
        let repo = setup_test_repo()?;

        // Create and stage multiple files
        fs::write(repo.path.join("file1.txt"), "content1")?;
        fs::write(repo.path.join("file2.txt"), "content2")?;
        fs::write(repo.path.join("file3.txt"), "content3")?;
        git_add(&repo.path, "file1.txt")?;
        git_add(&repo.path, "file2.txt")?;
        git_add(&repo.path, "file3.txt")?;

        // Reset specific files by path
        let mut cmd = Command::cargo_bin("git-navigator")?;
        cmd.arg("reset")
            .arg("file1.txt")
            .arg("file3.txt")
            .current_dir(&repo.path)
            .assert()
            .success()
            .stdout(predicate::str::contains("Successfully reset 2 files"));

        Ok(())
    }

    #[test]
    fn test_reset_index_still_works() -> anyhow::Result<()> {
        let repo = setup_test_repo()?;

        // Create and stage a file
        fs::write(repo.path.join("test.txt"), "content")?;
        git_add(&repo.path, "test.txt")?;

        // Run status to cache files
        let mut cmd = Command::cargo_bin("git-navigator")?;
        cmd.arg("status").current_dir(&repo.path).assert().success();

        // Reset using index (should still work)
        let mut cmd = Command::cargo_bin("git-navigator")?;
        cmd.arg("reset")
            .arg("1")
            .current_dir(&repo.path)
            .assert()
            .success()
            .stdout(predicate::str::contains("Successfully reset 1 file"));

        Ok(())
    }

    #[test]
    fn test_reset_no_args_error() -> anyhow::Result<()> {
        let repo = setup_test_repo()?;

        // Reset with no arguments should fail
        let mut cmd = Command::cargo_bin("git-navigator")?;
        cmd.arg("reset")
            .current_dir(&repo.path)
            .assert()
            .failure()
            .stdout(predicate::str::contains("No file indices provided"));

        Ok(())
    }

    #[test]
    fn test_reset_path_with_subdirectory() -> anyhow::Result<()> {
        let repo = setup_test_repo()?;

        // Create nested directory structure
        fs::create_dir_all(repo.path.join("src/commands"))?;
        fs::write(repo.path.join("src/commands/reset.rs"), "content")?;
        git_add(&repo.path, "src/commands/reset.rs")?;

        // Reset using full path
        let mut cmd = Command::cargo_bin("git-navigator")?;
        cmd.arg("reset")
            .arg("src/commands/reset.rs")
            .current_dir(&repo.path)
            .assert()
            .success()
            .stdout(predicate::str::contains("Successfully reset 1 file"));

        Ok(())
    }

    #[test]
    fn test_reset_nonexistent_path() -> anyhow::Result<()> {
        let repo = setup_test_repo_with_initial_commit()?;

        // Try to reset a file that doesn't exist
        // Git reset doesn't fail on non-existent files (it's a no-op)
        // So we need to check that git doesn't complain
        let mut cmd = Command::cargo_bin("git-navigator")?;
        cmd.arg("reset")
            .arg("nonexistent.txt")
            .current_dir(&repo.path)
            .assert()
            .success(); // Git allows resetting non-existent files

        Ok(())
    }

    #[test]
    fn test_reset_deleted_file_by_path() -> anyhow::Result<()> {
        let repo = setup_test_repo()?;

        // Create, commit, then delete and stage deletion
        fs::write(repo.path.join("file.txt"), "content")?;
        git_add(&repo.path, "file.txt")?;
        git_commit(&repo.path, "Add file")?;

        fs::remove_file(repo.path.join("file.txt"))?;
        git_add(&repo.path, "file.txt")?; // Stage deletion

        // Reset the staged deletion using path
        let mut cmd = Command::cargo_bin("git-navigator")?;
        cmd.arg("reset")
            .arg("file.txt")
            .current_dir(&repo.path)
            .assert()
            .success()
            .stdout(predicate::str::contains("Successfully reset 1 file"));

        Ok(())
    }
}
