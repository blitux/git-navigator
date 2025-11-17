//! Alias management command for shell integration.
//!
//! Provides functionality to show, update, and manage git-navigator shell aliases.

use crate::core::{
    alias_manager::{AliasManager, AliasStatus},
    error::Result,
    print_info, print_success,
};
use colored::Colorize;

/// Show current alias status
pub fn show_aliases() -> Result<()> {
    let manager = AliasManager::new()?;
    let comparisons = manager.compare_aliases()?;

    // Print header
    println!();
    println!(
        "{} ({})",
        "Git Navigator Aliases".bold(),
        manager.shell().name()
    );
    println!("{}", "━".repeat(70));
    println!(
        "{:<8} {:<8} {:<30} {}",
        "Status".bold(),
        "Alias".bold(),
        "Command".bold(),
        "Description".bold()
    );
    println!("{}", "━".repeat(70));

    // Print each alias
    let mut installed_count = 0;
    let mut missing_count = 0;
    let mut outdated_count = 0;

    for comp in &comparisons {
        let (status_icon, status_color) = match &comp.status {
            AliasStatus::Installed => {
                installed_count += 1;
                ("✓", "green")
            }
            AliasStatus::Missing => {
                missing_count += 1;
                ("✗", "red")
            }
            AliasStatus::Outdated { .. } => {
                outdated_count += 1;
                ("⚠", "yellow")
            }
        };

        let status_str = match status_color {
            "green" => status_icon.green(),
            "red" => status_icon.red(),
            "yellow" => status_icon.yellow(),
            _ => status_icon.normal(),
        };

        let alias_name = comp.alias.name.cyan();
        let command = if comp.alias.command.len() > 28 {
            format!("{}...", &comp.alias.command[..25])
        } else {
            comp.alias.command.clone()
        };

        println!(
            "{:<8} {:<8} {:<30} {}",
            status_str, alias_name, command, comp.alias.description
        );

        // Show current value if outdated
        if let AliasStatus::Outdated { current } = &comp.status {
            println!(
                "         {}  {} {}",
                " ".repeat(8),
                "current:".bright_black(),
                current.bright_black()
            );
        }
    }

    println!("{}", "━".repeat(70));

    // Print summary
    println!();
    println!(
        "{}: {}",
        "Config file".bright_black(),
        manager.config_path().display().to_string().bright_black()
    );
    println!(
        "{}: {} installed, {} missing, {} outdated",
        "Summary".bright_black(),
        installed_count.to_string().green(),
        missing_count.to_string().red(),
        outdated_count.to_string().yellow()
    );

    // Print action hint
    if missing_count > 0 || outdated_count > 0 {
        println!();
        print_info("Run 'git-navigator alias --update' to install/update aliases");
    }

    println!();
    Ok(())
}

/// Update aliases in shell configuration
pub fn update_aliases() -> Result<()> {
    let manager = AliasManager::new()?;

    print_info(&format!(
        "Updating aliases in {}...",
        manager.config_path().display()
    ));
    println!();

    // Get status before update
    let before = manager.compare_aliases()?;

    // Perform update
    manager.update_aliases()?;

    // Show changes
    println!("{}", "Changes:".bold());

    let mut added = 0;
    let mut updated = 0;
    let mut unchanged = 0;

    for comp in &before {
        let (symbol, color, change_type) = match &comp.status {
            AliasStatus::Missing => {
                added += 1;
                ("+", "green", "added")
            }
            AliasStatus::Outdated { .. } => {
                updated += 1;
                ("~", "yellow", "updated")
            }
            AliasStatus::Installed => {
                unchanged += 1;
                ("=", "bright_black", "unchanged")
            }
        };

        let symbol_colored = match color {
            "green" => symbol.green(),
            "yellow" => symbol.yellow(),
            "bright_black" => symbol.bright_black(),
            _ => symbol.normal(),
        };

        let change_colored = match color {
            "green" => change_type.green(),
            "yellow" => change_type.yellow(),
            "bright_black" => change_type.bright_black(),
            _ => change_type.normal(),
        };

        println!(
            "  {} {:<6} {} ({})",
            symbol_colored,
            comp.alias.name.cyan(),
            comp.alias.command,
            change_colored
        );
    }

    println!();

    // Print success message
    if added > 0 || updated > 0 {
        print_success(&format!(
            "{} aliases added/updated in {}",
            added + updated,
            manager.config_path().display()
        ));

        // Print shell-specific reload instruction
        let reload_cmd = match manager.shell().name() {
            "bash" => "source ~/.bashrc",
            "zsh" => "source ~/.zshrc",
            "fish" => "source ~/.config/fish/config.fish",
            "sh" => "source ~/.profile",
            _ => "restart your terminal",
        };

        print_info(&format!("Please restart your terminal or run: {reload_cmd}"));
    } else {
        print_success("All aliases are already up to date");
    }

    println!();
    Ok(())
}

/// Execute alias command with given action
pub fn execute_alias(_show: bool, update: bool) -> Result<()> {
    if update {
        update_aliases()
    } else {
        // Default is show
        show_aliases()
    }
}
