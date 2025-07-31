use std::path::PathBuf;
use std::io::{self, Write};
use clap::Parser;
use semver::Version;
use crate::core::error::GitNavigatorError;
use crate::core::dirs::get_config_directory;
use crate::core::{print_info, print_section_header, print_success};
use colored::*;

#[derive(Parser)]
pub struct RollbackArgs {
    /// Show available backup versions
    #[arg(long)]
    pub list: bool,
    
    /// Restore specific version
    #[arg(long)]
    pub version: Option<String>,
}

pub fn execute_rollback(args: RollbackArgs) -> Result<(), GitNavigatorError> {
    if args.list {
        list_available_backups()?;
        return Ok(());
    }
    
    if let Some(version) = args.version {
        restore_version(&version)?;
    } else {
        interactive_rollback()?;
    }
    
    Ok(())
}

fn list_available_backups() -> Result<(), GitNavigatorError> {
    let config_dir = get_config_directory()?;
    let backup_dir = config_dir.join("backups");
    
    if !backup_dir.exists() {
        print_info("No backups available");
        return Ok(());
    }
    
    print_section_header("Available backups");
    
    let mut backups = Vec::new();
    for entry in std::fs::read_dir(backup_dir)? {
        if let Ok(entry) = entry {
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with("git-navigator-v") {
                    let version = name.strip_prefix("git-navigator-v").unwrap();
                    let metadata = entry.metadata()?;
                    backups.push(BackupInfo {
                        version: version.to_string(),
                        path,
                        size: metadata.len(),
                        created: metadata.modified()?,
                    });
                }
            }
        }
    }
    
    backups.sort_by(|a, b| {
        Version::parse(&b.version).unwrap_or_else(|_| Version::new(0, 0, 0))
            .cmp(&Version::parse(&a.version).unwrap_or_else(|_| Version::new(0, 0, 0)))
    });
    
    for (i, backup) in backups.iter().enumerate() {
        let size = backup.size / 1024;
        let date = backup.created
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        println!("  {} {} ({} KB, created {})", 
                 format!("[{}]", i + 1).bright_black(),
                 format!("v{}", backup.version).blue(), 
                 size.to_string().bright_black(),
                 chrono::DateTime::from_timestamp(date as i64, 0)
                     .unwrap()
                     .format("%Y-%m-%d %H:%M")
                     .to_string().bright_black());
    }
    
    Ok(())
}

fn interactive_rollback() -> Result<(), GitNavigatorError> {
    let config_dir = get_config_directory()?;
    let backup_dir = config_dir.join("backups");
    
    if !backup_dir.exists() {
        return Err(GitNavigatorError::rollback_failed("No backups available"));
    }
    
    let mut backups = Vec::new();
    for entry in std::fs::read_dir(backup_dir)? {
        if let Ok(entry) = entry {
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with("git-navigator-v") {
                    let version = name.strip_prefix("git-navigator-v").unwrap();
                    backups.push((version.to_string(), path));
                }
            }
        }
    }
    
    if backups.is_empty() {
        return Err(GitNavigatorError::rollback_failed("No backups available"));
    }
    
    backups.sort_by(|a, b| {
        Version::parse(&b.0).unwrap_or_else(|_| Version::new(0, 0, 0))
            .cmp(&Version::parse(&a.0).unwrap_or_else(|_| Version::new(0, 0, 0)))
    });
    
    print_section_header("Select version to restore");
    for (i, (version, _)) in backups.iter().enumerate() {
        println!("  {} {}", format!("[{}]", i + 1).bright_black(), format!("v{}", version).blue());
    }
    
    print!("\n{} ", format!("Enter selection (1-{}):", backups.len()).blue());
    io::stdout().flush().unwrap();
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    
    let selection: usize = input.trim().parse()
        .map_err(|_| GitNavigatorError::rollback_failed("Invalid selection"))?;
    
    if selection < 1 || selection > backups.len() {
        return Err(GitNavigatorError::rollback_failed("Selection out of range"));
    }
    
    let (selected_version, _) = &backups[selection - 1];
    restore_version(selected_version)?;
    
    Ok(())
}

fn restore_version(version: &str) -> Result<(), GitNavigatorError> {
    let config_dir = get_config_directory()?;
    let backup_dir = config_dir.join("backups");
    let backup_file = backup_dir.join(format!("git-navigator-v{version}"));
    
    if !backup_file.exists() {
        return Err(GitNavigatorError::version_not_found(version));
    }
    
    let current_exe = std::env::current_exe()
        .map_err(|e| GitNavigatorError::rollback_failed(format!("Cannot determine current executable: {e}")))?;
    
    print_info(&format!("Restoring git-navigator v{version}..."));
    
    let backup_content = std::fs::read(&backup_file)
        .map_err(|e| GitNavigatorError::rollback_failed(format!("Failed to read backup: {e}")))?;
    
    std::fs::write(&current_exe, backup_content)
        .map_err(|e| GitNavigatorError::rollback_failed(format!("Failed to restore binary: {e}")))?;
    
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&current_exe)?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&current_exe, perms)?;
    }
    
    print_success(&format!("Successfully restored to v{version}"));
    
    Ok(())
}

#[derive(Debug)]
struct BackupInfo {
    version: String,
    #[allow(dead_code)]
    path: PathBuf,
    size: u64,
    created: std::time::SystemTime,
}