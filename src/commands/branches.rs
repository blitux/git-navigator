use crate::core::{
    error::{GitNavigatorError, Result},
    git::GitRepo,
    print_info, print_section_header,
    state::{BranchEntry, StateCache},
};
use colored::*;
use std::env;
use std::fs;
use std::path::PathBuf;

pub fn execute_branches(branch_index: Option<usize>) -> Result<()> {
    // Check if we're in a git repository
    let current_dir = env::current_dir()?;
    let git_repo = GitRepo::open(&current_dir).map_err(|_| GitNavigatorError::NotInGitRepo)?;

    if let Some(index) = branch_index {
        // Switch to branch by index
        checkout_branch_by_index(&git_repo, index)
    } else {
        // List branches with indices
        list_branches(&git_repo)
    }
}

fn list_branches(git_repo: &GitRepo) -> Result<()> {
    // Get all local branches
    let branches = get_local_branches(git_repo)?;

    if branches.is_empty() {
        print_info("No branches found. Make your first commit to create one.");
        return Ok(());
    }

    // Display section header using unified formatter
    print_section_header("Local Branches");

    // Display branches with proper formatting and colors
    for branch in &branches {
        if branch.is_current {
            // Current branch format: [*] branch-name (+ahead/-behind)
            let ahead_behind_text = match git_repo.get_ahead_behind() {
                Ok(Some((ahead, behind))) => {
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

            println!(
                "{}{}{} {}{}",
                "[".bright_black(),
                "*".white(),
                "]".bright_black(),
                branch.name.blue(),
                ahead_behind_text
            );
        } else {
            // Other branches format: [index] branch-name
            println!(
                "{}{}{} {}",
                "[".bright_black(),
                branch.index.to_string().white(),
                "]".bright_black(),
                branch.name.blue()
            );
        }
    }

    // Add spacing after branch list
    println!();

    // Save to cache for branch checkout command
    #[cfg(not(test))]
    {
        if let Err(e) = save_branches_cache(&branches, git_repo.get_repo_path()) {
            // Log cache errors but don't fail the command
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

    // Find branch by index
    let target_branch = branches
        .iter()
        .find(|branch| branch.index == index)
        .ok_or_else(|| {
            GitNavigatorError::custom_empty_files_error(&format!(
                "Branch index {} not found",
                index
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
        .map_err(|e| GitNavigatorError::Io(e))?;

    if output.status.success() {
        println!("Switched to branch '{}'", target_branch.name);
        Ok(())
    } else {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        Err(GitNavigatorError::custom_empty_files_error(&format!(
            "Failed to checkout branch '{}': {}",
            target_branch.name,
            error_msg.trim()
        )))
    }
}

fn get_local_branches(git_repo: &GitRepo) -> Result<Vec<BranchEntry>> {
    let repo = git_repo.get_repository();
    let mut branches = Vec::new();

    // Get current branch
    let current_branch = git_repo
        .get_current_branch()
        .unwrap_or_else(|_| "unknown".to_string());

    // List all local branches
    let branch_iter = repo.branches(Some(git2::BranchType::Local)).map_err(|e| {
        GitNavigatorError::custom_empty_files_error(&format!("Failed to list branches: {}", e))
    })?;

    let mut branch_names = Vec::new();
    for branch in branch_iter {
        let branch = branch.map_err(|e| {
            GitNavigatorError::custom_empty_files_error(&format!("Failed to read branch: {}", e))
        })?;
        let name = branch
            .0
            .name()
            .map_err(|e| {
                GitNavigatorError::custom_empty_files_error(&format!(
                    "Failed to get branch name: {}",
                    e
                ))
            })?
            .ok_or_else(|| {
                GitNavigatorError::custom_empty_files_error("Branch name is not valid UTF-8")
            })?
            .to_string();
        branch_names.push(name);
    }

    // Sort branch names for consistent ordering
    branch_names.sort();

    // Add current branch first (not numbered)
    if branch_names.contains(&current_branch) {
        branches.push(BranchEntry {
            index: 0, // Not used for current branch
            name: current_branch.clone(),
            is_current: true,
        });
    }

    // Add other branches with indices
    let mut index = 1;
    for branch_name in branch_names {
        if branch_name != current_branch {
            branches.push(BranchEntry {
                index,
                name: branch_name,
                is_current: false,
            });
            index += 1;
        }
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
