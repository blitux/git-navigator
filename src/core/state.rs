//! State management and caching data structures.
//!
//! This module defines the core data structures used for caching file and branch
//! information between commands. The cache enables fast operations on previously
//! scanned repository state.
//!
//! # Public API
//! - [`FileEntry`]: Represents a single file with its git status and metadata
//! - [`BranchEntry`]: Represents a git branch with selection index
//! - [`StateCache`]: Complete repository state cache with timing information
//!
//! # Cache Strategy
//! - **JSON serialization**: Human-readable cache files for debugging
//! - **Timestamping**: Track when cache was last updated
//! - **Repository isolation**: Separate cache per repository path

use crate::core::git_status::GitStatus;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::SystemTime;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FileEntry {
    pub index: usize,
    pub status: GitStatus,
    pub path: PathBuf,
    pub staged: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BranchEntry {
    pub index: usize,
    pub name: String,
    pub is_current: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StateCache {
    pub files: Vec<FileEntry>,
    pub branches: Vec<BranchEntry>,
    pub last_updated: SystemTime,
    pub repo_path: PathBuf,
}

impl StateCache {
    pub fn new(repo_path: PathBuf) -> Self {
        Self {
            files: Vec::new(),
            branches: Vec::new(),
            last_updated: SystemTime::now(),
            repo_path,
        }
    }
}
