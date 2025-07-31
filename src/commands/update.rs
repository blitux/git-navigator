use std::io::{self, Write};
use clap::Parser;
use semver::Version;
use crate::core::error::GitNavigatorError;
use crate::core::config::InstallConfig;
use crate::core::{print_info, print_section_header, print_success};
use colored::*;

// Repository configuration constants
const REPO_OWNER: &str = "blitux";
const REPO_NAME: &str = "git-navigator";
const BIN_NAME: &str = "git-navigator";

#[derive(Parser)]
pub struct UpdateArgs {
    /// Check for updates without installing
    #[arg(long)]
    pub check: bool,
    
    /// Show current version and exit
    #[arg(long)]
    pub version: bool,
    
    /// Skip confirmation prompts
    #[arg(long)]
    pub yes: bool,
    
    /// Show verbose update information
    #[arg(short, long)]
    pub verbose: bool,
}

pub fn execute_update(args: UpdateArgs) -> Result<(), GitNavigatorError> {
    let current_version = env!("CARGO_PKG_VERSION");
    
    if args.version {
        print_info(&format!("git-navigator v{current_version}"));
        return Ok(());
    }
    
    print_info("Checking for updates...");
    
    // Load config to get repository settings, fallback to constants if config fails
    let config = InstallConfig::load_or_create().unwrap_or_else(|_| InstallConfig {
        installed_version: current_version.to_string(),
        install_date: chrono::Utc::now(),
        binary_path: std::env::current_exe().unwrap_or_default(),
        repository: crate::core::config::RepositoryConfig {
            owner: REPO_OWNER.to_string(),
            name: REPO_NAME.to_string(),
            bin_name: BIN_NAME.to_string(),
        },
        update_config: crate::core::config::UpdateConfig::default(),
    });
    
    if args.check {
        let latest = self_update::backends::github::Update::configure()
            .repo_owner(&config.repository.owner)
            .repo_name(&config.repository.name)
            .bin_name(&config.repository.bin_name)
            .show_download_progress(true)
            .show_output(args.verbose)
            .current_version(current_version)
            .build()?
            .get_latest_release()?;
        display_update_check(current_version, &latest)?;
        return Ok(());
    }
    
    let latest = self_update::backends::github::Update::configure()
        .repo_owner(&config.repository.owner)
        .repo_name(&config.repository.name)
        .bin_name(&config.repository.bin_name)
        .show_download_progress(true)
        .show_output(args.verbose)
        .current_version(current_version)
        .build()?
        .get_latest_release()?;
    let needs_update = needs_update(current_version, &latest.version)?;
    
    if !needs_update {
        print_success(&format!("Already up to date (v{current_version})\n"));
        return Ok(());
    }
    
    if !args.yes && !confirm_update(current_version, &latest.version) {
        return Err(GitNavigatorError::UpdateCanceled);
    }
    
    print_info("Downloading update...");
    let status = self_update::backends::github::Update::configure()
        .repo_owner(&config.repository.owner)
        .repo_name(&config.repository.name)
        .bin_name(&config.repository.bin_name)
        .show_download_progress(true)
        .show_output(args.verbose)
        .current_version(current_version)
        .build()?
        .update()?;
    
    match status.updated() {
        true => {
            print_success(&format!("Successfully updated to v{}\n", status.version()));
            update_config_after_update(&status.version())?;
        },
        false => {
            print_success(&format!("Already up to date (v{current_version})\n"));
        }
    }    
    Ok(())
}

fn display_update_check(current: &str, latest: &self_update::update::Release) -> Result<(), GitNavigatorError> {
    print_section_header("Version information");
    println!("   Current: {}", format!("v{current}").blue());
    println!("   Latest:  {}", format!("v{}", latest.version).blue());
    
    if needs_update(current, &latest.version)? {
        println!("   Status:  {}", "Update available".yellow());
        
        if let Some(notes) = &latest.body {
            print_section_header("What's new");
            for line in notes.lines().take(5) {
                let clean_line = line.trim_start_matches("- ").trim_start_matches("* ");
                if !clean_line.is_empty() {
                    println!("   {} {}", "•".bright_black(), clean_line.white());
                }
            }
        }
        
        print_info("Run 'git-navigator update' to install the update");
    } else {
        println!("   Status:  {}\n", "Up to date".green());
    }
    
    Ok(())
}

fn confirm_update(current: &str, latest: &str) -> bool {
    print_section_header("Update process");
    println!("   {}. Download git-navigator {} from GitHub Releases", "1".bright_black(), format!("v{latest}").blue());
    println!("   {}. Verify download integrity with checksums", "2".bright_black());
    println!("   {}. Backup current binary ({})", "3".bright_black(), format!("v{current}").blue());
    println!("   {}. Replace binary atomically", "4".bright_black());
    println!("   {}. Verify installation", "5".bright_black());
    
    print!("\n{} ", "Proceed with update? [y/N]:".blue());
    io::stdout().flush().unwrap();
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    
    matches!(input.trim().to_lowercase().as_str(), "y" | "yes")
}

fn needs_update(current: &str, latest: &str) -> Result<bool, GitNavigatorError> {
    let current_version = Version::parse(current)
        .map_err(|e| GitNavigatorError::config_error(format!("Invalid current version: {e}")))?;
    let latest_version = Version::parse(latest)
        .map_err(|e| GitNavigatorError::config_error(format!("Invalid latest version: {e}")))?;
    
    Ok(latest_version > current_version)
}

fn update_config_after_update(new_version: &str) -> Result<(), GitNavigatorError> {
    let mut config = InstallConfig::load_or_create()?;
    config.update_version(new_version)?;
    Ok(())
}