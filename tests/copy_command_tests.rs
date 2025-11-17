use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

mod common;
use common::repository::*;
use std::fs;

#[cfg(test)]
mod copy_command_tests {
    use super::*;

    #[test]
    fn test_copy_single_file_by_index() -> anyhow::Result<()> {
        let repo = setup_test_repo()?;

        // Create and modify a file
        fs::write(repo.path.join("test.txt"), "content")?;

        // Run gs to populate cache
        let mut gs_cmd = Command::cargo_bin("git-navigator")?;
        gs_cmd
            .arg("status")
            .current_dir(&repo.path)
            .assert()
            .success();

        // Create destination directory
        let dest_dir = repo.path.join("backup");
        fs::create_dir(&dest_dir)?;

        // Copy file by index
        let mut cmd = Command::cargo_bin("git-navigator")?;
        cmd.arg("copy")
            .arg("1")
            .arg(dest_dir.to_str().unwrap())
            .current_dir(&repo.path)
            .assert()
            .success();

        // Verify file was copied
        assert!(dest_dir.join("test.txt").exists());
        let content = fs::read_to_string(dest_dir.join("test.txt"))?;
        assert_eq!(content, "content");

        Ok(())
    }

    #[test]
    fn test_copy_multiple_files_by_range() -> anyhow::Result<()> {
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

        // Create destination directory
        let dest_dir = repo.path.join("backup");
        fs::create_dir(&dest_dir)?;

        // Copy files by range
        let mut cmd = Command::cargo_bin("git-navigator")?;
        cmd.arg("copy")
            .arg("1-3")
            .arg(dest_dir.to_str().unwrap())
            .current_dir(&repo.path)
            .assert()
            .success();

        // Verify files were copied
        assert!(dest_dir.join("file1.txt").exists());
        assert!(dest_dir.join("file2.txt").exists());
        assert!(dest_dir.join("file3.txt").exists());

        Ok(())
    }

    #[test]
    fn test_copy_mixed_indices_and_paths() -> anyhow::Result<()> {
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

        // Create destination directory
        let dest_dir = repo.path.join("backup");
        fs::create_dir(&dest_dir)?;

        // Copy by mixing index and literal path
        let mut cmd = Command::cargo_bin("git-navigator")?;
        cmd.arg("copy")
            .arg("1")
            .arg("literal.txt")
            .arg(dest_dir.to_str().unwrap())
            .current_dir(&repo.path)
            .assert()
            .success();

        // Verify both files were copied
        assert!(dest_dir.join("indexed.txt").exists());
        assert!(dest_dir.join("literal.txt").exists());

        Ok(())
    }

    #[test]
    fn test_copy_without_cache_fails() -> anyhow::Result<()> {
        let repo = setup_test_repo()?;

        // Create a file but DON'T run gs
        fs::write(repo.path.join("test.txt"), "content")?;

        // Create destination directory
        let dest_dir = repo.path.join("backup");
        fs::create_dir(&dest_dir)?;

        // Try to copy by index without cache
        let mut cmd = Command::cargo_bin("git-navigator")?;
        cmd.arg("copy")
            .arg("1")
            .arg(dest_dir.to_str().unwrap())
            .current_dir(&repo.path)
            .assert()
            .failure()
            .stdout(predicate::str::contains("Cannot load file cache"));

        Ok(())
    }

    #[test]
    fn test_copy_no_arguments_shows_usage() -> anyhow::Result<()> {
        let repo = setup_test_repo()?;

        // Try copy without arguments
        let mut cmd = Command::cargo_bin("git-navigator")?;
        cmd.arg("copy")
            .current_dir(&repo.path)
            .assert()
            .failure()
            .stdout(predicate::str::contains("No arguments provided"));

        Ok(())
    }

    #[test]
    fn test_copy_literal_paths_without_indices() -> anyhow::Result<()> {
        let repo = setup_test_repo()?;

        // Create files
        fs::write(repo.path.join("source.txt"), "source content")?;

        // Create destination directory
        let dest_dir = repo.path.join("backup");
        fs::create_dir(&dest_dir)?;

        // Copy using only literal paths (no indices)
        let mut cmd = Command::cargo_bin("git-navigator")?;
        cmd.arg("copy")
            .arg("source.txt")
            .arg(dest_dir.to_str().unwrap())
            .current_dir(&repo.path)
            .assert()
            .success();

        // Verify file was copied
        assert!(dest_dir.join("source.txt").exists());
        let content = fs::read_to_string(dest_dir.join("source.txt"))?;
        assert_eq!(content, "source content");

        Ok(())
    }

    #[test]
    fn test_copy_invalid_index_fails() -> anyhow::Result<()> {
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

        // Create destination directory
        let dest_dir = repo.path.join("backup");
        fs::create_dir(&dest_dir)?;

        // Try to copy with out-of-bounds index
        let mut cmd = Command::cargo_bin("git-navigator")?;
        cmd.arg("copy")
            .arg("99")
            .arg(dest_dir.to_str().unwrap())
            .current_dir(&repo.path)
            .assert()
            .failure()
            .stdout(predicate::str::contains("out of range"));

        Ok(())
    }
}
