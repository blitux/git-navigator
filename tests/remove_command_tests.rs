use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

mod common;
use common::repository::*;
use std::fs;

#[cfg(test)]
mod remove_command_tests {
    use super::*;

    #[test]
    fn test_remove_single_file_by_index() -> anyhow::Result<()> {
        let repo = setup_test_repo()?;

        // Create a file
        fs::write(repo.path.join("test.txt"), "content")?;

        // Run gs to populate cache
        let mut gs_cmd = Command::cargo_bin("git-navigator")?;
        gs_cmd
            .arg("status")
            .current_dir(&repo.path)
            .assert()
            .success();

        // Verify file exists
        assert!(repo.path.join("test.txt").exists());

        // Remove file by index
        let mut cmd = Command::cargo_bin("git-navigator")?;
        cmd.arg("remove")
            .arg("1")
            .current_dir(&repo.path)
            .assert()
            .success();

        // Verify file was removed
        assert!(!repo.path.join("test.txt").exists());

        Ok(())
    }

    #[test]
    fn test_remove_multiple_files_by_range() -> anyhow::Result<()> {
        let repo = setup_test_repo()?;

        // Create multiple files
        fs::write(repo.path.join("file1.txt"), "content1")?;
        fs::write(repo.path.join("file2.txt"), "content2")?;
        fs::write(repo.path.join("file3.txt"), "content3")?;

        // Run gs to populate cache
        let mut gs_cmd = Command::cargo_bin("git-navigator")?;
        gs_cmd
            .arg("status")
            .current_dir(&repo.path)
            .assert()
            .success();

        // Verify files exist
        assert!(repo.path.join("file1.txt").exists());
        assert!(repo.path.join("file2.txt").exists());
        assert!(repo.path.join("file3.txt").exists());

        // Remove files by range
        let mut cmd = Command::cargo_bin("git-navigator")?;
        cmd.arg("remove")
            .arg("1-3")
            .current_dir(&repo.path)
            .assert()
            .success();

        // Verify files were removed
        assert!(!repo.path.join("file1.txt").exists());
        assert!(!repo.path.join("file2.txt").exists());
        assert!(!repo.path.join("file3.txt").exists());

        Ok(())
    }

    #[test]
    fn test_remove_with_force_flag() -> anyhow::Result<()> {
        let repo = setup_test_repo()?;

        // Create a file
        fs::write(repo.path.join("test.txt"), "content")?;

        // Run gs to populate cache
        let mut gs_cmd = Command::cargo_bin("git-navigator")?;
        gs_cmd
            .arg("status")
            .current_dir(&repo.path)
            .assert()
            .success();

        // Remove file with -f flag (pass through using --)
        let mut cmd = Command::cargo_bin("git-navigator")?;
        cmd.arg("remove")
            .arg("--")
            .arg("-f")
            .arg("1")
            .current_dir(&repo.path)
            .assert()
            .success();

        // Verify file was removed
        assert!(!repo.path.join("test.txt").exists());

        Ok(())
    }

    #[test]
    fn test_remove_directory_recursively() -> anyhow::Result<()> {
        let repo = setup_test_repo()?;

        // Create directory with file
        fs::create_dir_all(repo.path.join("test_dir"))?;
        fs::write(repo.path.join("test_dir/file.txt"), "content")?;

        // Run gs to populate cache (the directory will show as untracked)
        let mut gs_cmd = Command::cargo_bin("git-navigator")?;
        gs_cmd
            .arg("status")
            .current_dir(&repo.path)
            .assert()
            .success();

        // Verify directory exists
        assert!(repo.path.join("test_dir").exists());

        // Remove directory recursively by index (pass through using --)
        let mut cmd = Command::cargo_bin("git-navigator")?;
        cmd.arg("remove")
            .arg("--")
            .arg("-rf")
            .arg("1")
            .current_dir(&repo.path)
            .assert()
            .success();

        // Verify directory was removed
        assert!(!repo.path.join("test_dir").exists());

        Ok(())
    }

    #[test]
    fn test_remove_without_cache_fails() -> anyhow::Result<()> {
        let repo = setup_test_repo()?;

        // Create a file but DON'T run gs
        fs::write(repo.path.join("test.txt"), "content")?;

        // Try to remove by index without cache
        let mut cmd = Command::cargo_bin("git-navigator")?;
        cmd.arg("remove")
            .arg("1")
            .current_dir(&repo.path)
            .assert()
            .failure()
            .stdout(predicate::str::contains("Cannot load file cache"));

        // Verify file still exists
        assert!(repo.path.join("test.txt").exists());

        Ok(())
    }

    #[test]
    fn test_remove_no_arguments_shows_usage() -> anyhow::Result<()> {
        let repo = setup_test_repo()?;

        // Try remove without arguments
        let mut cmd = Command::cargo_bin("git-navigator")?;
        cmd.arg("remove")
            .current_dir(&repo.path)
            .assert()
            .failure()
            .stdout(predicate::str::contains("No arguments provided"));

        Ok(())
    }

    #[test]
    fn test_remove_literal_paths_without_indices() -> anyhow::Result<()> {
        let repo = setup_test_repo()?;

        // Create file
        fs::write(repo.path.join("test.txt"), "content")?;

        // Verify file exists
        assert!(repo.path.join("test.txt").exists());

        // Remove using only literal path (no indices)
        let mut cmd = Command::cargo_bin("git-navigator")?;
        cmd.arg("remove")
            .arg("test.txt")
            .current_dir(&repo.path)
            .assert()
            .success();

        // Verify file was removed
        assert!(!repo.path.join("test.txt").exists());

        Ok(())
    }

    #[test]
    fn test_remove_invalid_index_fails() -> anyhow::Result<()> {
        let repo = setup_test_repo()?;

        // Create a single file
        fs::write(repo.path.join("test.txt"), "content")?;

        // Run gs to populate cache
        let mut gs_cmd = Command::cargo_bin("git-navigator")?;
        gs_cmd
            .arg("status")
            .current_dir(&repo.path)
            .assert()
            .success();

        // Try to remove with out-of-bounds index
        let mut cmd = Command::cargo_bin("git-navigator")?;
        cmd.arg("remove")
            .arg("99")
            .current_dir(&repo.path)
            .assert()
            .failure()
            .stdout(predicate::str::contains("out of range"));

        // Verify file still exists
        assert!(repo.path.join("test.txt").exists());

        Ok(())
    }

    #[test]
    fn test_remove_mixed_indices_and_paths() -> anyhow::Result<()> {
        let repo = setup_test_repo()?;

        // Create files
        fs::write(repo.path.join("indexed.txt"), "indexed content")?;
        fs::write(repo.path.join("literal.txt"), "literal content")?;

        // Run gs to populate cache
        let mut gs_cmd = Command::cargo_bin("git-navigator")?;
        gs_cmd
            .arg("status")
            .current_dir(&repo.path)
            .assert()
            .success();

        // Verify files exist
        assert!(repo.path.join("indexed.txt").exists());
        assert!(repo.path.join("literal.txt").exists());

        // Remove by mixing index and literal path
        let mut cmd = Command::cargo_bin("git-navigator")?;
        cmd.arg("remove")
            .arg("1")
            .arg("literal.txt")
            .current_dir(&repo.path)
            .assert()
            .success();

        // Verify both files were removed
        assert!(!repo.path.join("indexed.txt").exists());
        assert!(!repo.path.join("literal.txt").exists());

        Ok(())
    }
}
