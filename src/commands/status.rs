use crate::core::{
    error::{GitNavigatorError, Result},
    git::GitRepo,
    git_status::GitStatus,
    state::StateCache,
    templates::{render_template, TemplateContext, TEMPLATES},
};
use std::env;
use std::fs;
use std::path::PathBuf;

pub fn execute_status() -> Result<()> {
    // Check if we're in a git repository
    let current_dir = env::current_dir()?;
    let git_repo = GitRepo::open(&current_dir).map_err(|_| GitNavigatorError::NotInGitRepo)?;

    // Get branch and commit information - keep as String for lifetime management
    let branch = git_repo
        .get_current_branch()
        .unwrap_or_else(|_| "-none-".to_string());
    let (hash, message) = git_repo
        .get_parent_commit_info()
        .unwrap_or_else(|_| ("".to_string(), "- no commits yet -".to_string()));

    // Get ahead/behind information and format it
    let ahead_behind_text = match git_repo.get_ahead_behind() {
        Ok(Some((ahead, behind))) => {
            use colored::*;
            if ahead > 0 && behind > 0 {
                format!(
                    " {}+{}/−{}{}",
                    "(".bright_black(),
                    ahead.to_string().white(),
                    behind.to_string().white(),
                    ")".bright_black()
                )
            } else if ahead > 0 {
                format!(
                    " {}+{}{}",
                    "(".bright_black(),
                    ahead.to_string().white(),
                    ")".bright_black()
                )
            } else if behind > 0 {
                format!(
                    " {}-{}{}",
                    "(".bright_black(),
                    behind.to_string().white(),
                    ")".bright_black()
                )
            } else {
                String::new()
            }
        }
        Ok(None) => String::new(),
        Err(_) => String::new(),
    };

    // Print header information with spacing
    println!(
        "{}",
        render_template(TEMPLATES.header_empty_line, &TemplateContext::default())
    );

    let branch_context = TemplateContext {
        branch_name: Some(&branch),
        ahead_behind: Some(&ahead_behind_text),
        ..Default::default()
    };
    println!(
        "{}",
        render_template(TEMPLATES.header_branch, &branch_context)
    );

    if hash.is_empty() {
        let parent_context = TemplateContext {
            commit_message: Some(&message),
            ..Default::default()
        };
        println!(
            "{}",
            render_template(TEMPLATES.header_parent_no_commits, &parent_context)
        );
    } else {
        let parent_context = TemplateContext {
            short_hash: Some(&hash),
            commit_message: Some(&message),
            ..Default::default()
        };
        println!(
            "{}",
            render_template(TEMPLATES.header_parent_with_commits, &parent_context)
        );
    }

    println!(
        "{}",
        render_template(TEMPLATES.header_empty_line, &TemplateContext::default())
    );

    // Get file status from git
    let files = git_repo.get_status()?;

    if files.is_empty() {
        // No files to show, similar to `git status` behavior
        return Ok(());
    }

    // Display files grouped by type like SCM Breeze
    print_grouped_status_sections(&files);

    // Save to cache for other commands (skip in test mode)
    #[cfg(not(test))]
    {
        if let Err(e) = save_files_cache(&files, git_repo.get_repo_path()) {
            // Log cache errors but don't fail the status command
            log::warn!("Cache save failed (status command will continue): {e}");
            // In debug mode, also print to stderr for development visibility
            #[cfg(debug_assertions)]
            eprintln!("Warning: Cache save failed: {e}");
        }
    }

    Ok(())
}

fn save_files_cache(files: &[crate::core::state::FileEntry], repo_path: PathBuf) -> Result<()> {
    use crate::core::error::GitNavigatorError;

    log::debug!("Attempting to save {} files to cache", files.len());

    // Get cache directory with detailed error context
    let cache_dir = get_cache_dir(&repo_path).map_err(|e| {
        log::warn!("Failed to determine cache directory: {e}");
        e
    })?;

    log::debug!("Using cache directory: {}", cache_dir.display());

    // Create cache directory with detailed error handling
    if let Err(e) = fs::create_dir_all(&cache_dir) {
        log::error!(
            "Failed to create cache directory '{}': {}",
            cache_dir.display(),
            e
        );
        return Err(GitNavigatorError::cache_directory_creation_failed(
            &cache_dir, e,
        ));
    }

    let cache_file = cache_dir.join("files.json");
    log::debug!("Cache file path: {}", cache_file.display());

    let cache = StateCache {
        files: files.to_vec(),
        branches: Vec::new(), // Not used for status command
        last_updated: std::time::SystemTime::now(),
        repo_path,
    };

    // Serialize cache data with error context
    let json = serde_json::to_string_pretty(&cache).map_err(|e| {
        log::error!("Failed to serialize cache data: {e}");
        GitNavigatorError::cache_serialization_failed(e)
    })?;

    // Write cache file with error context
    if let Err(e) = fs::write(&cache_file, json) {
        log::error!(
            "Failed to write cache file '{}': {}",
            cache_file.display(),
            e
        );
        return Err(GitNavigatorError::cache_write_failed(&cache_file, e));
    }

    log::debug!("Successfully cached {} files", files.len());
    Ok(())
}

fn get_cache_dir(repo_path: &PathBuf) -> Result<PathBuf> {
    // Respect XDG_CACHE_HOME environment variable first, fallback to dirs::cache_dir()
    let cache_home = std::env::var("XDG_CACHE_HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| dirs::cache_dir().unwrap_or_else(|| std::path::PathBuf::from("/tmp")));

    // Create a hash of the repo path for unique cache directory
    let repo_hash = format!("{:x}", md5::compute(repo_path.to_string_lossy().as_bytes()));

    log::debug!("get_cache_dir: repo_path = {repo_path:?}");
    log::debug!("get_cache_dir: cache_home = {cache_home:?}");
    log::debug!("get_cache_dir: repo_hash = {repo_hash:?}");

    Ok(cache_home.join("git-navigator").join(repo_hash))
}

pub fn load_files_cache(repo_path: &PathBuf) -> Result<Vec<crate::core::state::FileEntry>> {
    use crate::core::error::GitNavigatorError;

    log::debug!("Attempting to load cache for repo: {}", repo_path.display());

    let cache_dir = get_cache_dir(repo_path).map_err(|e| {
        log::warn!("Failed to determine cache directory: {e}");
        e
    })?;
    log::debug!("load_files_cache: cache_dir = {cache_dir:?}");

    let cache_file = cache_dir.join("files.json");
    log::debug!("Looking for cache file: {}", cache_file.display());
    log::debug!(
        "load_files_cache: cache_file = {:?}, exists = {}",
        cache_file,
        cache_file.exists()
    );

    if !cache_file.exists() {
        log::debug!("Cache file does not exist: {}", cache_file.display());
        return Err(GitNavigatorError::cache_file_not_found(&cache_file));
    }

    let content = fs::read_to_string(&cache_file).map_err(|e| {
        log::error!(
            "Failed to read cache file '{}': {}",
            cache_file.display(),
            e
        );
        GitNavigatorError::cache_read_failed(&cache_file, e)
    })?;

    let cache: StateCache = serde_json::from_str(&content).map_err(|e| {
        log::error!(
            "Failed to parse cache file '{}': {}",
            cache_file.display(),
            e
        );
        GitNavigatorError::cache_parse_failed(&cache_file, e)
    })?;

    log::debug!("Successfully loaded {} files from cache", cache.files.len());

    if cache.files.is_empty() {
        log::debug!("Cache file exists but contains no files");
        return Err(GitNavigatorError::NoCachedFiles);
    }

    Ok(cache.files)
}

fn print_grouped_status_sections(files: &[crate::core::state::FileEntry]) {
    let mut staged_files = Vec::new();
    let mut unstaged_files = Vec::new();
    let mut untracked_files = Vec::new();
    let mut unmerged_files = Vec::new();

    // Group files by type
    for file in files {
        match file.status {
            GitStatus::Unmerged => unmerged_files.push(file),
            GitStatus::Untracked => untracked_files.push(file),
            _ if file.staged => staged_files.push(file),
            _ => unstaged_files.push(file),
        }
    }

    // Print unmerged files first
    if !unmerged_files.is_empty() {
        println!(
            "{}",
            render_template(TEMPLATES.section_unmerged, &TemplateContext::default())
        );
        for file in &unmerged_files {
            print_status_line(file, "both modified");
        }
        println!(
            "{}",
            render_template(TEMPLATES.section_spacing, &TemplateContext::default())
        );
    }

    // Print staged files
    if !staged_files.is_empty() {
        println!(
            "{}",
            render_template(TEMPLATES.section_staged, &TemplateContext::default())
        );
        for file in &staged_files {
            let description = file.status.description();
            print_status_line(file, description);
        }
        println!(
            "{}",
            render_template(TEMPLATES.section_spacing, &TemplateContext::default())
        );
    }

    // Print unstaged files
    if !unstaged_files.is_empty() {
        println!(
            "{}",
            render_template(TEMPLATES.section_unstaged, &TemplateContext::default())
        );
        for file in &unstaged_files {
            let description = file.status.description();
            print_status_line(file, description);
        }
        println!(
            "{}",
            render_template(TEMPLATES.section_spacing, &TemplateContext::default())
        );
    }

    // Print untracked files
    if !untracked_files.is_empty() {
        println!(
            "{}",
            render_template(TEMPLATES.section_untracked, &TemplateContext::default())
        );
        for file in &untracked_files {
            print_status_line(file, "untracked");
        }
        println!(
            "{}",
            render_template(TEMPLATES.section_spacing, &TemplateContext::default())
        );
    }
}

/// Print just the file sections without header information (for use in other commands)
pub fn print_files_only(files: &[crate::core::state::FileEntry]) {
    if files.is_empty() {
        return;
    }
    print_grouped_status_sections(files);
}

fn print_status_line(file: &crate::core::state::FileEntry, description: &str) {
    // Convert PathBuf to str efficiently, avoiding allocation when possible
    let filename = file.path.to_string_lossy();
    let context = TemplateContext {
        file_status: Some(description),
        n: Some(file.index),
        filename: Some(&filename),
        git_status: Some(file.status),
        ..Default::default()
    };
    println!("{}", render_template(TEMPLATES.file_line, &context));
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn setup_test_repo() -> Result<(TempDir, PathBuf)> {
        let temp_dir = TempDir::new().map_err(|e| GitNavigatorError::Io(e))?;
        let repo_path = temp_dir.path().to_path_buf();

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

        Ok((temp_dir, repo_path))
    }

    #[test]
    fn test_execute_status_empty_repo() -> Result<()> {
        let (_temp_dir, repo_path) = setup_test_repo()?;

        // Test that we can open the repo without changing directories
        let git_repo = GitRepo::open(&repo_path)?;
        let files = git_repo.get_status()?;

        // Should succeed with empty file list for empty repo
        assert!(files.is_empty());
        Ok(())
    }

    #[test]
    fn test_execute_status_with_files() -> Result<()> {
        let (_temp_dir, repo_path) = setup_test_repo()?;

        // Create a test file
        fs::write(repo_path.join("test.txt"), "test content")?;

        // Test that we can detect the untracked file without changing directories
        let git_repo = GitRepo::open(&repo_path)?;
        let files = git_repo.get_status()?;

        // Should find the untracked file
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].status, GitStatus::Untracked);
        assert_eq!(files[0].path, std::path::PathBuf::from("test.txt"));
        Ok(())
    }

    #[test]
    fn test_execute_status_not_in_git_repo() -> Result<()> {
        let temp_dir = TempDir::new().map_err(|e| GitNavigatorError::Io(e))?;
        let non_repo_path = temp_dir.path();

        // Test that we get an error when trying to open a non-git directory
        let result = GitRepo::open(non_repo_path);

        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_get_cache_dir() -> Result<()> {
        let repo_path = PathBuf::from("/test/repo/path");
        let cache_dir = get_cache_dir(&repo_path)?;

        assert!(cache_dir.to_string_lossy().contains("git-navigator"));
        assert!(cache_dir.is_absolute());
        Ok(())
    }

    #[test]
    fn test_load_files_cache_nonexistent_file() {
        // Use a non-existent path without creating actual temp directories
        let fake_repo_path = PathBuf::from("/non/existent/repo/path");

        let result = load_files_cache(&fake_repo_path);
        assert!(result.is_err());

        let error = result.unwrap_err();
        match error {
            GitNavigatorError::CacheFileNotFound { path } => {
                assert!(path.to_string_lossy().contains("files.json"));
            }
            _ => panic!("Expected CacheFileNotFound error, got: {}", error),
        }
    }

    #[test]
    fn test_save_files_cache_creates_directory() -> Result<()> {
        let temp_dir = TempDir::new().map_err(|e| GitNavigatorError::Io(e))?;
        let repo_path = temp_dir.path().to_path_buf();

        // Create some test files to cache
        use crate::core::git_status::GitStatus;
        let test_files = vec![crate::core::state::FileEntry {
            index: 1,
            status: GitStatus::Modified,
            path: PathBuf::from("test.txt"),
            staged: false,
        }];

        // Temporarily change the cache home directory to our temp dir
        let original_cache_home = std::env::var("XDG_CACHE_HOME").ok();
        std::env::set_var("XDG_CACHE_HOME", temp_dir.path());

        let result = save_files_cache(&test_files, repo_path.clone());

        // Restore environment
        match original_cache_home {
            Some(val) => std::env::set_var("XDG_CACHE_HOME", val),
            None => std::env::remove_var("XDG_CACHE_HOME"),
        }

        // Should succeed in creating and saving cache
        assert!(result.is_ok(), "Failed to save cache: {:?}", result);

        Ok(())
    }

    #[test]
    #[ignore] // Disabled due to environment variable race conditions with parallel tests
    fn test_load_files_cache_corrupted_json() -> Result<()> {
        // NOTE: This test has environment variable isolation issues when run in parallel
        // It should pass when run with --test-threads=1
        // TODO: Refactor to avoid global environment state
        let temp_dir = TempDir::new().map_err(|e| GitNavigatorError::Io(e))?;
        let repo_path = temp_dir.path().to_path_buf();

        // Temporarily change the cache home directory to our temp dir
        let original_cache_home = std::env::var("XDG_CACHE_HOME").ok();
        let cache_home = temp_dir.path().join("cache");
        std::env::set_var("XDG_CACHE_HOME", &cache_home);

        // Get consistent cache directory and create it
        let cache_dir = get_cache_dir(&repo_path)?;
        fs::create_dir_all(&cache_dir)?;

        let cache_file = cache_dir.join("files.json");
        // Write invalid JSON
        fs::write(&cache_file, "{ invalid json")?;

        // Ensure the file exists before calling load_files_cache
        assert!(cache_file.exists(), "Cache file should exist");

        let result = load_files_cache(&repo_path);

        // Restore environment
        match original_cache_home {
            Some(val) => std::env::set_var("XDG_CACHE_HOME", val),
            None => std::env::remove_var("XDG_CACHE_HOME"),
        }

        assert!(result.is_err());

        let error = result.unwrap_err();
        match error {
            GitNavigatorError::CacheParseFailed { path, .. } => {
                assert_eq!(path, cache_file);
            }
            _ => panic!("Expected CacheParseFailed error, got: {}", error),
        }

        Ok(())
    }

    #[test]
    fn test_print_status_line_logic() {
        // Test the core logic of print_status_line without actual printing
        use crate::core::git_status::GitStatus;
        use std::path::PathBuf;

        let file_entry = crate::core::state::FileEntry {
            index: 1,
            status: GitStatus::Modified,
            path: PathBuf::from("test.txt"),
            staged: false,
        };

        // This test ensures the function doesn't panic and can handle different file entries
        // In a real scenario this would print, but the logic itself is testable
        let filename = file_entry.path.to_string_lossy();
        assert_eq!(filename, "test.txt");
        assert_eq!(file_entry.status.description(), "modified");
        assert_eq!(file_entry.index, 1);
        assert!(!file_entry.staged);
    }

    #[test]
    fn test_file_grouping_logic() {
        // Test the file grouping logic without actual printing
        use crate::core::git_status::GitStatus;
        use std::path::PathBuf;

        let files = vec![
            crate::core::state::FileEntry {
                index: 1,
                status: GitStatus::Modified,
                path: PathBuf::from("modified.txt"),
                staged: false,
            },
            crate::core::state::FileEntry {
                index: 2,
                status: GitStatus::Added,
                path: PathBuf::from("staged.txt"),
                staged: true,
            },
            crate::core::state::FileEntry {
                index: 3,
                status: GitStatus::Untracked,
                path: PathBuf::from("untracked.txt"),
                staged: false,
            },
            crate::core::state::FileEntry {
                index: 4,
                status: GitStatus::Unmerged,
                path: PathBuf::from("conflict.txt"),
                staged: false,
            },
        ];

        // Test the grouping logic that happens in print_grouped_status_sections
        let mut staged_files = Vec::new();
        let mut unstaged_files = Vec::new();
        let mut untracked_files = Vec::new();
        let mut unmerged_files = Vec::new();

        for file in &files {
            match file.status {
                GitStatus::Unmerged => unmerged_files.push(file),
                GitStatus::Untracked => untracked_files.push(file),
                _ if file.staged => staged_files.push(file),
                _ => unstaged_files.push(file),
            }
        }

        assert_eq!(staged_files.len(), 1);
        assert_eq!(unstaged_files.len(), 1);
        assert_eq!(untracked_files.len(), 1);
        assert_eq!(unmerged_files.len(), 1);

        assert_eq!(staged_files[0].path, PathBuf::from("staged.txt"));
        assert_eq!(unstaged_files[0].path, PathBuf::from("modified.txt"));
        assert_eq!(untracked_files[0].path, PathBuf::from("untracked.txt"));
        assert_eq!(unmerged_files[0].path, PathBuf::from("conflict.txt"));
    }

    #[test]
    #[ignore] // Disabled due to environment variable race conditions with parallel tests
    fn test_load_files_cache_empty_files() -> Result<()> {
        // NOTE: This test has environment variable isolation issues when run in parallel
        // It should pass when run with --test-threads=1
        // TODO: Refactor to avoid global environment state
        let temp_dir = TempDir::new().map_err(|e| GitNavigatorError::Io(e))?;
        let repo_path = temp_dir.path().to_path_buf();

        // Temporarily change the cache home directory to our temp dir
        let original_cache_home = std::env::var("XDG_CACHE_HOME").ok();
        std::env::set_var("XDG_CACHE_HOME", temp_dir.path());

        // Debug: Print the environment variable
        eprintln!(
            "DEBUG empty_files: XDG_CACHE_HOME = {:?}",
            std::env::var("XDG_CACHE_HOME")
        );
        eprintln!("DEBUG empty_files: temp_dir = {:?}", temp_dir.path());

        let cache_dir = get_cache_dir(&repo_path)?;
        eprintln!("DEBUG empty_files: cache_dir = {:?}", cache_dir);
        fs::create_dir_all(&cache_dir)?;

        let cache_file = cache_dir.join("files.json");

        // Create valid JSON but with empty files
        let empty_cache = StateCache {
            files: Vec::new(),
            branches: Vec::new(),
            last_updated: std::time::SystemTime::now(),
            repo_path: repo_path.clone(),
        };
        let json = serde_json::to_string_pretty(&empty_cache)?;
        fs::write(&cache_file, json)?;
        eprintln!(
            "DEBUG empty_files: cache_file exists = {}",
            cache_file.exists()
        );

        let result = load_files_cache(&repo_path);
        eprintln!("DEBUG empty_files: result = {:?}", result);

        // Restore environment
        match original_cache_home {
            Some(val) => std::env::set_var("XDG_CACHE_HOME", val),
            None => std::env::remove_var("XDG_CACHE_HOME"),
        }

        assert!(result.is_err());

        let error = result.unwrap_err();
        match error {
            GitNavigatorError::NoCachedFiles => {
                // Expected
            }
            _ => panic!("Expected NoCachedFiles error, got: {}", error),
        }

        Ok(())
    }
}
