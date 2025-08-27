use crate::core::{
    error::{GitNavigatorError, Result},
    git::GitRepo,
    print_info,
    state::{BranchEntry, StateCache},
};
use colored::*;
use std::env;
use std::fs;
use std::path::PathBuf;
use tabled::{Table, settings::{Style, Remove, object::Rows}};

pub fn execute_branches(show_remote: bool, branch_index: Option<usize>) -> Result<()> {
    // Check if we're in a git repository
    let current_dir = env::current_dir()?;
    let git_repo = GitRepo::open(&current_dir).map_err(|_| GitNavigatorError::NotInGitRepo)?;

    if let Some(index) = branch_index {
        // Switch to branch by index
        checkout_branch_by_index(&git_repo, index)
    } else {
        // List branches with indices
        list_branches(&git_repo, show_remote)
    }
}

fn list_branches(git_repo: &GitRepo, show_remote: bool) -> Result<()> {
    let branches = if show_remote {
        get_remote_branches(git_repo)?
    } else {
        get_local_branches_with_tracking(git_repo)?
    };

    if branches.is_empty() {
        if show_remote {
            print_info("No remote branches found. Run `git fetch` to update remotes.");
        } else {
            print_info("No branches found. Make your first commit to create one.");
        }
        return Ok(());
    }

    // Follow template: \n(title/header)\n(body)\n(help)\n
    println!(); // Opening whitespace
    
    // Show title/header
    if show_remote {
        println!("{}", "Remote Branches".blue());
    } else {
        println!("{}", "Local Branches".blue());
    }
    println!(); // Spacing after title
    
    // Create table with tabled - no borders, no headers
    let mut table = Table::new(&branches);
    table.with(Style::blank()); // No borders
    table.with(Remove::row(Rows::first())); // Remove header row

    println!("{}", table);
    
    println!(); // Spacing before help
    
    // Show help message in bright_black
    if show_remote {
        println!("{}", "Use gb <index> to checkout or gb without --remote for local branches.".bright_black());
    } else {
        println!("{}", "Use gb --remote to list remote branches.".bright_black());
    }
    
    println!(); // Closing whitespace

    // Save to cache for branch checkout command (only local branches)
    #[cfg(not(test))]
    if !show_remote {
        if let Err(e) = save_branches_cache(&branches, git_repo.get_repo_path()) {
            log::warn!("Branch cache save failed: {e}");
            #[cfg(debug_assertions)]
            eprintln!("Warning: Branch cache save failed: {e}");
        }
    }

    Ok(())
}

fn checkout_branch_by_index(git_repo: &GitRepo, index: usize) -> Result<()> {
    // Load cached branches from previous gb command
    let branches = load_branches_cache(&git_repo.get_repo_path()).map_err(|e| {
        log::warn!("Failed to load branch cache: {e}");
        GitNavigatorError::custom_cache_error(
            "Cannot load branch cache. Run 'gb' first to list branches.",
            e,
        )
    })?;

    if branches.is_empty() {
        return Err(GitNavigatorError::custom_empty_files_error(
            "No branches found in cache",
        ));
    }

    // Find branch by index - parse the index string directly
    let target_branch = branches
        .iter()
        .find(|branch| {
            branch.index.parse::<usize>().ok() == Some(index)
        })
        .ok_or_else(|| {
            GitNavigatorError::custom_empty_files_error(format!(
                "Branch index {index} not found"
            ))
        })?;

    // Check if trying to switch to current branch
    if target_branch.is_current {
        return Err(GitNavigatorError::custom_empty_files_error(
            "Cannot switch to current branch",
        ));
    }

    // Execute git checkout command
    let workdir = git_repo
        .get_repository()
        .workdir()
        .ok_or_else(|| GitNavigatorError::custom_empty_files_error("No workdir found"))?;

    let output = std::process::Command::new("git")
        .arg("checkout")
        .arg(&target_branch.name)
        .current_dir(workdir)
        .output()
        .map_err(GitNavigatorError::Io)?;

    if output.status.success() {
        println!("Switched to branch '{}'", target_branch.name);
        Ok(())
    } else {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        Err(GitNavigatorError::custom_empty_files_error(format!(
            "Failed to checkout branch '{}': {}",
            target_branch.name,
            error_msg.trim()
        )))
    }
}

fn get_local_branches_with_tracking(git_repo: &GitRepo) -> Result<Vec<BranchEntry>> {
    let repo = git_repo.get_repository();
    let mut branches = Vec::new();

    // Get current branch
    let current_branch = git_repo
        .get_current_branch()
        .unwrap_or_else(|_| "unknown".to_string());

    // List all local branches with tracking information
    let branch_iter = repo.branches(Some(git2::BranchType::Local)).map_err(|e| {
        GitNavigatorError::custom_empty_files_error(format!("Failed to list branches: {e}"))
    })?;

    let mut branch_data = Vec::new();
    for branch_result in branch_iter {
        let (branch, _) = branch_result.map_err(|e| {
            GitNavigatorError::custom_empty_files_error(format!("Failed to read branch: {e}"))
        })?;
        
        let name = branch
            .name()
            .map_err(|e| {
                GitNavigatorError::custom_empty_files_error(format!(
                    "Failed to get branch name: {e}"
                ))
            })?
            .ok_or_else(|| {
                GitNavigatorError::custom_empty_files_error("Branch name is not valid UTF-8")
            })?
            .to_string();

        // Get tracking branch information
        let tracking_branch = branch.upstream()
            .ok()
            .and_then(|upstream| {
                upstream.name().ok().flatten().map(|s| s.to_string())
            });

        branch_data.push((name, tracking_branch));
    }

    // Sort by branch name for consistent ordering
    branch_data.sort_by(|a, b| a.0.cmp(&b.0));

    // Add current branch first with special formatting
    if let Some((_, tracking)) = branch_data.iter().find(|(name, _)| name == &current_branch) {
        branches.push(BranchEntry {
            index: "*".to_string(), // Special marker for current branch
            name: current_branch.clone(),
            is_current: true,
            tracking_info: tracking.clone().unwrap_or_else(|| "(no upstream)".to_string()),
            is_remote: false,
        });
    }

    // Add other branches with indices
    let mut index = 1;
    for (branch_name, tracking) in branch_data {
        if branch_name != current_branch {
            branches.push(BranchEntry {
                index: index.to_string(),
                name: branch_name,
                is_current: false,
                tracking_info: tracking.unwrap_or_else(|| "(no upstream)".to_string()),
                is_remote: false,
            });
            index += 1;
        }
    }

    Ok(branches)
}

fn get_remote_branches(git_repo: &GitRepo) -> Result<Vec<BranchEntry>> {
    let repo = git_repo.get_repository();
    let mut branches = Vec::new();

    // List all remote branches
    let branch_iter = repo.branches(Some(git2::BranchType::Remote)).map_err(|e| {
        GitNavigatorError::custom_empty_files_error(format!("Failed to list remote branches: {e}"))
    })?;

    let mut branch_data = Vec::new();
    for branch_result in branch_iter {
        let (branch, _) = branch_result.map_err(|e| {
            GitNavigatorError::custom_empty_files_error(format!("Failed to read remote branch: {e}"))
        })?;
        
        let full_name = branch
            .name()
            .map_err(|e| {
                GitNavigatorError::custom_empty_files_error(format!(
                    "Failed to get remote branch name: {e}"
                ))
            })?
            .ok_or_else(|| {
                GitNavigatorError::custom_empty_files_error("Remote branch name is not valid UTF-8")
            })?
            .to_string();

        // Parse remote name from full branch name (e.g., "origin/main" -> remote: "origin")
        let remote_name = if let Some(slash_pos) = full_name.find('/') {
            full_name[..slash_pos].to_string()
        } else {
            "unknown".to_string()
        };

        branch_data.push((full_name, remote_name));
    }

    // Sort by branch name for consistent ordering
    branch_data.sort_by(|a, b| a.0.cmp(&b.0));

    // Add remote branches with indices
    for (index, (branch_name, remote_name)) in branch_data.into_iter().enumerate() {
        branches.push(BranchEntry {
            index: (index + 1).to_string(), // Start from 1 for remote branches
            name: branch_name,
            is_current: false,
            tracking_info: remote_name, // Store remote name for remote branches
            is_remote: true,
        });
    }

    Ok(branches)
}

#[cfg(not(test))]
fn save_branches_cache(branches: &[BranchEntry], repo_path: PathBuf) -> Result<()> {
    use crate::core::error::GitNavigatorError;

    log::debug!("Attempting to save {} branches to cache", branches.len());

    // Get cache directory
    let cache_dir = get_cache_dir(&repo_path).map_err(|e| {
        log::warn!("Failed to determine cache directory: {e}");
        e
    })?;

    log::debug!("Using cache directory: {}", cache_dir.display());

    // Create cache directory
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

    let cache_file = cache_dir.join("branches.json");
    log::debug!("Cache file path: {}", cache_file.display());

    let cache = StateCache {
        files: Vec::new(), // Not used for branches command
        branches: branches.to_vec(),
        last_updated: std::time::SystemTime::now(),
        repo_path,
    };

    // Serialize cache data
    let json = serde_json::to_string_pretty(&cache).map_err(|e| {
        log::error!("Failed to serialize branch cache data: {e}");
        GitNavigatorError::cache_serialization_failed(e)
    })?;

    // Write cache file
    if let Err(e) = fs::write(&cache_file, json) {
        log::error!(
            "Failed to write branch cache file '{}': {}",
            cache_file.display(),
            e
        );
        return Err(GitNavigatorError::cache_write_failed(&cache_file, e));
    }

    log::debug!("Successfully cached {} branches", branches.len());
    Ok(())
}

fn load_branches_cache(repo_path: &PathBuf) -> Result<Vec<BranchEntry>> {
    use crate::core::error::GitNavigatorError;

    log::debug!(
        "Attempting to load branch cache for repo: {}",
        repo_path.display()
    );

    let cache_dir = get_cache_dir(repo_path).map_err(|e| {
        log::warn!("Failed to determine cache directory: {e}");
        e
    })?;
    log::debug!("load_branches_cache: cache_dir = {cache_dir:?}");

    let cache_file = cache_dir.join("branches.json");
    log::debug!("Looking for branch cache file: {}", cache_file.display());
    log::debug!(
        "load_branches_cache: cache_file = {:?}, exists = {}",
        cache_file,
        cache_file.exists()
    );

    if !cache_file.exists() {
        log::debug!("Branch cache file does not exist: {}", cache_file.display());
        return Err(GitNavigatorError::cache_file_not_found(&cache_file));
    }

    let content = fs::read_to_string(&cache_file).map_err(|e| {
        log::error!(
            "Failed to read branch cache file '{}': {}",
            cache_file.display(),
            e
        );
        GitNavigatorError::cache_read_failed(&cache_file, e)
    })?;

    let cache: StateCache = serde_json::from_str(&content).map_err(|e| {
        log::error!(
            "Failed to parse branch cache file '{}': {}",
            cache_file.display(),
            e
        );
        GitNavigatorError::cache_parse_failed(&cache_file, e)
    })?;

    log::debug!(
        "Successfully loaded {} branches from cache",
        cache.branches.len()
    );

    if cache.branches.is_empty() {
        log::debug!("Branch cache file exists but contains no branches");
        return Err(GitNavigatorError::NoCachedFiles);
    }

    Ok(cache.branches)
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

#[cfg(test)]
mod tests {
    use super::*;
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
    fn test_execute_branches_empty_repo() -> Result<()> {
        let (_temp_dir, repo_path) = setup_test_repo()?;

        // Test that we can open the repo without changing directories
        let git_repo = GitRepo::open(&repo_path)?;
        let branches = get_local_branches(&git_repo)?;

        // Verify no branches exist
        assert!(branches.is_empty());
        Ok(())
    }

    #[test]
    fn test_execute_branches_not_in_git_repo() -> Result<()> {
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
    fn test_load_branches_cache_nonexistent_file() {
        // Use a non-existent path without creating actual temp directories
        let fake_repo_path = PathBuf::from("/non/existent/repo/path");

        let result = load_branches_cache(&fake_repo_path);
        assert!(result.is_err());

        let error = result.unwrap_err();
        match error {
            GitNavigatorError::CacheFileNotFound { path } => {
                assert!(path.to_string_lossy().contains("branches.json"));
            }
            _ => panic!("Expected CacheFileNotFound error, got: {}", error),
        }
    }
}
