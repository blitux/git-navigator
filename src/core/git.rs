//! Git repository operations and file status management.
//!
//! This module provides a high-level interface to git operations through the [`GitRepo`] struct.
//! It wraps the `git2` library to provide git-navigator specific functionality for reading
//! repository status, adding files, and extracting repository metadata.
//!
//! # Public API
//! - [`GitRepo`]: Main interface for git repository operations
//!
//! # Key Features
//! - **Status reading**: Convert git2 status flags to typed [`GitStatus`] entries
//! - **File staging**: Add files to the git index with validation
//! - **File reset**: Reset files in the git index
//! - **Repository info**: Extract branch names, commit info, and repository paths
//! - **Type safety**: All operations return structured data instead of raw strings

use crate::core::{
    error::{GitNavigatorError, Result},
    git_status::GitStatus,
    state::FileEntry,
};
use git2::{Repository, StatusOptions};
use std::path::{Path, PathBuf};

pub struct GitRepo {
    repo: Repository,
}

impl GitRepo {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let repo = Repository::discover(path)?;
        Ok(GitRepo { repo })
    }

    /// Execute a git command in the repository's working directory
    fn execute_git_command(&self, mut cmd: std::process::Command) -> Result<()> {
        let workdir = self
            .repo
            .workdir()
            .ok_or(GitNavigatorError::custom_empty_files_error(
                "Repository has no working directory",
            ))?;

        cmd.current_dir(workdir);

        let output = cmd.output().map_err(|e| GitNavigatorError::Io(e))?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(GitNavigatorError::custom_empty_files_error(&format!(
                "git command failed: {}",
                error_msg.trim()
            )));
        }

        Ok(())
    }

    pub fn get_status(&self) -> Result<Vec<FileEntry>> {
        let mut opts = StatusOptions::new();
        opts.include_untracked(true);
        opts.include_ignored(false);

        let statuses = self.repo.statuses(Some(&mut opts))?;
        let mut files = Vec::new();

        for entry in statuses.iter() {
            let path = entry.path().ok_or(GitNavigatorError::InvalidUtf8Path)?;

            let status_flags = entry.status();
            let path_buf = PathBuf::from(path);

            // Handle staged changes
            if let Some((status, staged)) = GitStatus::from_git2_staged(status_flags) {
                files.push(FileEntry {
                    index: 0, // Will be recalculated in display order
                    status,
                    path: path_buf.clone(),
                    staged,
                });
            }

            // Handle unstaged changes (can be in addition to staged)
            if let Some((status, staged)) = GitStatus::from_git2_unstaged(status_flags) {
                files.push(FileEntry {
                    index: 0, // Will be recalculated in display order
                    status,
                    path: path_buf,
                    staged,
                });
            }
        }

        // Sort files by priority: unmerged, staged, unstaged, untracked
        files.sort_by(|a, b| {
            a.status
                .sort_priority(a.staged)
                .cmp(&b.status.sort_priority(b.staged))
                .then_with(|| a.path.cmp(&b.path))
        });

        // Recalculate indices in display order
        for (index, file) in files.iter_mut().enumerate() {
            file.index = index + 1; // 1-based indexing
        }

        Ok(files)
    }

    pub fn reset_files(&self, paths: &[PathBuf]) -> Result<()> {
        if paths.is_empty() {
            return Ok(());
        }

        let mut cmd = std::process::Command::new("git");
        cmd.arg("reset").arg("HEAD").arg("--");

        for path in paths {
            cmd.arg(path);
        }

        self.execute_git_command(cmd)
    }

    pub fn get_repo_path(&self) -> PathBuf {
        self.repo.path().to_path_buf()
    }

    pub fn get_repository(&self) -> &Repository {
        &self.repo
    }

    pub fn get_current_branch(&self) -> Result<String> {
        let head = self.repo.head()?;

        if let Some(branch_name) = head.shorthand() {
            if head.is_branch() {
                Ok(branch_name.to_string())
            } else {
                // Detached HEAD
                let oid = head.target().unwrap();
                Ok(format!("detached at {}", &oid.to_string()[..7]))
            }
        } else {
            Ok("-none-".to_string())
        }
    }

    pub fn get_parent_commit_info(&self) -> Result<(String, String)> {
        match self.repo.head() {
            Ok(head) => {
                if let Some(oid) = head.target() {
                    let commit = self.repo.find_commit(oid)?;
                    let short_hash = oid.to_string()[..7].to_string();
                    let message = commit
                        .message()
                        .unwrap_or("")
                        .lines()
                        .next()
                        .unwrap_or("")
                        .to_string();
                    Ok((short_hash, message))
                } else {
                    Ok(("".to_string(), "- no commits yet -".to_string()))
                }
            }
            Err(_) => Ok(("".to_string(), "- no commits yet -".to_string())),
        }
    }

    /// Get ahead/behind information for the current branch relative to its upstream
    /// Returns (ahead, behind) counts, or None if no upstream is set
    pub fn get_ahead_behind(&self) -> Result<Option<(usize, usize)>> {
        // Get the current branch reference
        let head = match self.repo.head() {
            Ok(head) => head,
            Err(_) => return Ok(None),
        };

        // Get local commit OID
        let local_oid = match head.target() {
            Some(oid) => oid,
            None => return Ok(None),
        };

        // Get branch name from HEAD
        let branch_name = match head.shorthand() {
            Some(name) => name,
            None => return Ok(None),
        };

        // Find the local branch object
        let local_branch = match self.repo.find_branch(branch_name, git2::BranchType::Local) {
            Ok(branch) => branch,
            Err(_) => return Ok(None),
        };

        // Get the upstream branch
        let upstream_branch = match local_branch.upstream() {
            Ok(upstream) => upstream,
            Err(_) => return Ok(None), // No upstream configured
        };

        // Get upstream commit OID
        let upstream_ref = upstream_branch.get();
        let upstream_oid = match upstream_ref.target() {
            Some(oid) => oid,
            None => return Ok(None),
        };

        // Calculate ahead/behind using git2's graph functionality
        match self.repo.graph_ahead_behind(local_oid, upstream_oid) {
            Ok((ahead, behind)) => Ok(Some((ahead, behind))),
            Err(_) => Ok(None),
        }
    }

    pub fn add_files(&self, paths: &[PathBuf]) -> Result<()> {
        if paths.is_empty() {
            return Ok(());
        }

        let mut cmd = std::process::Command::new("git");
        cmd.arg("add").arg("--");

        for path in paths {
            cmd.arg(path);
        }

        self.execute_git_command(cmd)
    }

    pub fn checkout_files(&self, paths: &[PathBuf]) -> Result<()> {
        if paths.is_empty() {
            return Ok(());
        }

        let mut cmd = std::process::Command::new("git");
        cmd.arg("checkout").arg("--");

        for path in paths {
            cmd.arg(path);
        }

        self.execute_git_command(cmd)
    }

    pub fn create_branch(&self, branch_name: &str) -> Result<()> {
        let mut cmd = std::process::Command::new("git");
        cmd.args(["checkout", "-b", branch_name]);
        self.execute_git_command(cmd)
    }

    pub fn checkout_branch(&self, branch_name: &str) -> Result<()> {
        let mut cmd = std::process::Command::new("git");
        cmd.args(["checkout", branch_name]);
        self.execute_git_command(cmd)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_test_repo() -> Result<(TempDir, crate::core::git::GitRepo)> {
        let temp_dir = TempDir::new().map_err(|e| GitNavigatorError::Io(e))?;
        let repo_path = temp_dir.path();

        // Initialize git repo
        std::process::Command::new("git")
            .args(["init"])
            .current_dir(&repo_path)
            .output()
            .map_err(|e| GitNavigatorError::Io(e))?;

        // Set git config
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
    fn test_open_git_repo() -> Result<()> {
        let (_temp_dir, _git_repo) = setup_test_repo()
            .map_err(|e| GitNavigatorError::custom_cache_error("Test repo setup failed", e))?;
        // If we get here without error, the repo was opened successfully
        Ok(())
    }

    #[test]
    fn test_get_status_empty_repo() -> Result<()> {
        let (_temp_dir, git_repo) = setup_test_repo()?;
        let files = git_repo.get_status()?;
        assert!(files.is_empty());
        Ok(())
    }

    #[test]
    fn test_get_status_with_untracked_file() -> Result<()> {
        let (_temp_dir, git_repo) = setup_test_repo()?;

        // Create an untracked file
        std::fs::write(
            git_repo
                .get_repository()
                .workdir()
                .unwrap()
                .join("test.txt"),
            "test content",
        )
        .map_err(|e| GitNavigatorError::Io(e))?;

        let files = git_repo.get_status()?;
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].status, GitStatus::Untracked);
        assert_eq!(files[0].path, PathBuf::from("test.txt"));
        assert!(!files[0].staged);

        Ok(())
    }

    #[test]
    fn test_open_non_git_directory() {
        // Use a non-existent path without creating actual directories
        // GitRepo::open will fail on any path that doesn't contain a .git directory
        let non_git_path = std::path::PathBuf::from("/tmp/definitely/not/a/git/repo");
        let result = GitRepo::open(&non_git_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_add_directory_with_files() -> Result<()> {
        let (_temp_dir, git_repo) = setup_test_repo()?;
        let workdir = git_repo.get_repository().workdir().unwrap();

        // Create a directory structure with files
        let test_dir = workdir.join("test_dir");
        std::fs::create_dir_all(&test_dir).map_err(|e| GitNavigatorError::Io(e))?;

        // Create files in the directory
        std::fs::write(test_dir.join("file1.txt"), "content1")
            .map_err(|e| GitNavigatorError::Io(e))?;
        std::fs::write(test_dir.join("file2.rs"), "content2")
            .map_err(|e| GitNavigatorError::Io(e))?;

        // Create a subdirectory with a file
        let sub_dir = test_dir.join("subdir");
        std::fs::create_dir_all(&sub_dir).map_err(|e| GitNavigatorError::Io(e))?;
        std::fs::write(sub_dir.join("nested.md"), "nested content")
            .map_err(|e| GitNavigatorError::Io(e))?;

        // Add the directory (should add all files recursively)
        let dir_path = workdir.join("test_dir");
        git_repo.add_files(&[dir_path])?;

        // Verify all files were added to the index
        let status = git_repo.get_status()?;
        let staged_files: Vec<_> = status
            .iter()
            .filter(|f| f.staged)
            .map(|f| f.path.as_path())
            .collect();

        assert_eq!(staged_files.len(), 3);
        assert!(staged_files.contains(&Path::new("test_dir/file1.txt")));
        assert!(staged_files.contains(&Path::new("test_dir/file2.rs")));
        assert!(staged_files.contains(&Path::new("test_dir/subdir/nested.md")));

        Ok(())
    }

    #[test]
    fn test_add_empty_directory() -> Result<()> {
        let (_temp_dir, git_repo) = setup_test_repo()?;
        let workdir = git_repo.get_repository().workdir().unwrap();

        // Create an empty directory
        let empty_dir = workdir.join("empty_dir");
        std::fs::create_dir_all(&empty_dir).map_err(|e| GitNavigatorError::Io(e))?;

        // Adding an empty directory should succeed but not stage anything
        let dir_path = workdir.join("empty_dir");
        git_repo.add_files(&[dir_path])?;

        // Verify no files were staged (empty directory has no files)
        let status = git_repo.get_status()?;
        let staged_files: Vec<_> = status.iter().filter(|f| f.staged).collect();

        assert!(staged_files.is_empty());

        Ok(())
    }

    #[test]
    fn test_add_mixed_files_and_directories() -> Result<()> {
        let (_temp_dir, git_repo) = setup_test_repo()?;
        let workdir = git_repo.get_repository().workdir().unwrap();

        // Create individual file
        std::fs::write(workdir.join("single.txt"), "single file content")
            .map_err(|e| GitNavigatorError::Io(e))?;

        // Create directory with files
        let test_dir = workdir.join("dir_with_files");
        std::fs::create_dir_all(&test_dir).map_err(|e| GitNavigatorError::Io(e))?;
        std::fs::write(test_dir.join("dir_file.rs"), "directory file content")
            .map_err(|e| GitNavigatorError::Io(e))?;

        // Add both file and directory in one operation
        let paths = vec![workdir.join("single.txt"), workdir.join("dir_with_files")];
        git_repo.add_files(&paths)?;

        // Verify both the individual file and directory files were staged
        let status = git_repo.get_status()?;
        let staged_files: Vec<_> = status
            .iter()
            .filter(|f| f.staged)
            .map(|f| f.path.as_path())
            .collect();

        assert_eq!(staged_files.len(), 2);
        assert!(staged_files.contains(&Path::new("single.txt")));
        assert!(staged_files.contains(&Path::new("dir_with_files/dir_file.rs")));

        Ok(())
    }

    #[test]
    fn test_add_deleted_file() -> Result<()> {
        let (_temp_dir, git_repo) = setup_test_repo()?;
        let workdir = git_repo.get_repository().workdir().unwrap();

        // Create and commit a file first
        let test_file = workdir.join("test_file.txt");
        std::fs::write(&test_file, "initial content").map_err(|e| GitNavigatorError::Io(e))?;

        // Add and commit the file
        git_repo.add_files(&[test_file.clone()])?;
        std::process::Command::new("git")
            .args(["commit", "-m", "Add test file"])
            .current_dir(workdir)
            .output()
            .map_err(|e| GitNavigatorError::Io(e))?;

        // Now delete the file from filesystem
        std::fs::remove_file(&test_file).map_err(|e| GitNavigatorError::Io(e))?;

        // Verify file shows as deleted in status
        let status_before_add = git_repo.get_status()?;
        let deleted_files: Vec<_> = status_before_add
            .iter()
            .filter(|f| f.status == GitStatus::Deleted && !f.staged)
            .collect();
        assert_eq!(deleted_files.len(), 1);
        assert_eq!(deleted_files[0].path, Path::new("test_file.txt"));

        // Add the deleted file (this should stage the deletion)
        git_repo.add_files(&[PathBuf::from("test_file.txt")])?;

        // Verify the deletion is now staged
        let status_after_add = git_repo.get_status()?;
        let staged_deletions: Vec<_> = status_after_add
            .iter()
            .filter(|f| f.status == GitStatus::Deleted && f.staged)
            .collect();

        assert_eq!(staged_deletions.len(), 1);
        assert_eq!(staged_deletions[0].path, Path::new("test_file.txt"));

        // Verify there are no unstaged deletions left
        let unstaged_deletions: Vec<_> = status_after_add
            .iter()
            .filter(|f| f.status == GitStatus::Deleted && !f.staged)
            .collect();
        assert_eq!(unstaged_deletions.len(), 0);

        Ok(())
    }

    #[test]
    fn test_add_files_git_command_integration() -> Result<()> {
        let (_temp_dir, git_repo) = setup_test_repo()?;
        let workdir = git_repo.get_repository().workdir().unwrap();

        // Create test files
        std::fs::write(workdir.join("file1.txt"), "content 1")
            .map_err(|e| GitNavigatorError::Io(e))?;
        std::fs::write(workdir.join("file2.rs"), "content 2")
            .map_err(|e| GitNavigatorError::Io(e))?;

        // Add multiple files at once using our new git command approach
        let paths = vec![PathBuf::from("file1.txt"), PathBuf::from("file2.rs")];
        git_repo.add_files(&paths)?;

        // Verify both files were staged
        let status = git_repo.get_status()?;
        let staged_files: Vec<_> = status
            .iter()
            .filter(|f| f.staged)
            .map(|f| f.path.as_path())
            .collect();

        assert_eq!(staged_files.len(), 2);
        assert!(staged_files.contains(&Path::new("file1.txt")));
        assert!(staged_files.contains(&Path::new("file2.rs")));

        Ok(())
    }

    #[test]
    fn test_add_files_empty_list() -> Result<()> {
        let (_temp_dir, git_repo) = setup_test_repo()?;

        // Adding empty list should succeed without error
        git_repo.add_files(&[])?;

        // Status should be empty (no files added)
        let status = git_repo.get_status()?;
        assert!(status.is_empty());

        Ok(())
    }

    #[test]
    fn test_reset_files_git_command() -> Result<()> {
        let (_temp_dir, git_repo) = setup_test_repo()?;
        let workdir = git_repo.get_repository().workdir().unwrap();

        // Create and commit a file first
        let test_file = workdir.join("test_reset.txt");
        std::fs::write(&test_file, "initial content").map_err(|e| GitNavigatorError::Io(e))?;

        git_repo.add_files(&[test_file.clone()])?;
        std::process::Command::new("git")
            .args(["commit", "-m", "Add test file"])
            .current_dir(workdir)
            .output()
            .map_err(|e| GitNavigatorError::Io(e))?;

        // Modify the file and stage the changes
        std::fs::write(&test_file, "modified content").map_err(|e| GitNavigatorError::Io(e))?;
        git_repo.add_files(&[test_file.clone()])?;

        // Verify file is staged
        let status_before_reset = git_repo.get_status()?;
        let staged_files: Vec<_> = status_before_reset.iter().filter(|f| f.staged).collect();
        assert_eq!(staged_files.len(), 1);

        // Reset the file using git command approach
        git_repo.reset_files(&[PathBuf::from("test_reset.txt")])?;

        // Verify file is no longer staged but still modified
        let status_after_reset = git_repo.get_status()?;
        let staged_files: Vec<_> = status_after_reset.iter().filter(|f| f.staged).collect();
        let unstaged_files: Vec<_> = status_after_reset.iter().filter(|f| !f.staged).collect();

        assert_eq!(staged_files.len(), 0);
        assert_eq!(unstaged_files.len(), 1);
        assert_eq!(unstaged_files[0].path, Path::new("test_reset.txt"));

        Ok(())
    }

    #[test]
    fn test_reset_multiple_files() -> Result<()> {
        let (_temp_dir, git_repo) = setup_test_repo()?;
        let workdir = git_repo.get_repository().workdir().unwrap();

        // Create and commit multiple files
        let file1 = workdir.join("file1.txt");
        let file2 = workdir.join("file2.txt");

        std::fs::write(&file1, "content 1").map_err(|e| GitNavigatorError::Io(e))?;
        std::fs::write(&file2, "content 2").map_err(|e| GitNavigatorError::Io(e))?;

        git_repo.add_files(&[file1.clone(), file2.clone()])?;
        std::process::Command::new("git")
            .args(["commit", "-m", "Add test files"])
            .current_dir(workdir)
            .output()
            .map_err(|e| GitNavigatorError::Io(e))?;

        // Modify both files and stage them
        std::fs::write(&file1, "modified content 1").map_err(|e| GitNavigatorError::Io(e))?;
        std::fs::write(&file2, "modified content 2").map_err(|e| GitNavigatorError::Io(e))?;

        git_repo.add_files(&[file1, file2])?;

        // Verify both files are staged
        let status_before_reset = git_repo.get_status()?;
        let staged_files: Vec<_> = status_before_reset.iter().filter(|f| f.staged).collect();
        assert_eq!(staged_files.len(), 2);

        // Reset both files at once
        let paths = vec![PathBuf::from("file1.txt"), PathBuf::from("file2.txt")];
        git_repo.reset_files(&paths)?;

        // Verify both files are no longer staged
        let status_after_reset = git_repo.get_status()?;
        let staged_files: Vec<_> = status_after_reset.iter().filter(|f| f.staged).collect();
        assert_eq!(staged_files.len(), 0);

        Ok(())
    }

    #[test]
    fn test_reset_files_empty_list() -> Result<()> {
        let (_temp_dir, git_repo) = setup_test_repo()?;

        // Resetting empty list should succeed without error
        git_repo.reset_files(&[])?;

        // Status should be empty (no files to reset)
        let status = git_repo.get_status()?;
        assert!(status.is_empty());

        Ok(())
    }
}
