use clap::{Parser, Subcommand};
use git_navigator::commands::*;
use git_navigator::core::{
    error::{GitNavigatorError, Result},
    print_error,
};
use std::env;

#[derive(Parser)]
#[command(name = "git-navigator")]
#[command(about = "A lightweight and efficient Git navigation tool")]
#[command(version = "0.1.0")]
struct Cli {
    /// Enable debug logging
    #[arg(long, global = true)]
    debug: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Show numbered git status (gs alias)
    Status,
    /// Add files by index (ga alias)
    Add {
        /// File indices to add (e.g., "1 3-5,8")
        indices: Vec<String>,
    },
    /// Show diff for files by index (gd alias)
    Diff {
        /// File indices to diff (e.g., "1 3-5,8")
        indices: Vec<String>,
    },
    /// Reset files by index (grs alias)
    Reset {
        /// File indices to reset (e.g., "1 3-5,8")
        indices: Vec<String>,
    },
    /// Checkout files by index or switch to branch (gco alias)
    Checkout {
        /// Create and switch to a new branch
        #[arg(short = 'b', long = "create")]
        create_branch: bool,
        /// File indices (e.g., "1 3-5,8") OR branch name (e.g., "main") OR branch name to create
        indices: Vec<String>,
    },
    /// Show numbered branches or switch to a branch (gb alias)
    Branches {
        /// Branch index to checkout (if provided)
        index: Option<usize>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Configure logging based on --debug flag
    if cli.debug {
        env::set_var("RUST_LOG", "debug");
    } else {
        env::set_var("RUST_LOG", "info");
    }
    env_logger::init();

    match cli.command {
        Commands::Status => {
            if let Err(e) = execute_status() {
                if let GitNavigatorError::NotInGitRepo = e {
                    print_error("Not in a git repository");
                } else {
                    print_error(&e.to_string());
                }
                std::process::exit(1);
            }
        }
        Commands::Add { indices } => {
            if let Err(e) = execute_add(indices) {
                if let GitNavigatorError::NotInGitRepo = e {
                    print_error("Not in a git repository");
                } else {
                    print_error(&e.to_string());
                }
                std::process::exit(1);
            }
        }
        Commands::Diff { indices } => {
            if let Err(e) = execute_diff(indices) {
                if let GitNavigatorError::NotInGitRepo = e {
                    print_error("Not in a git repository");
                } else {
                    print_error(&e.to_string());
                }
                std::process::exit(1);
            }
        }
        Commands::Reset { indices } => {
            if let Err(e) = execute_reset(indices) {
                if let GitNavigatorError::NotInGitRepo = e {
                    print_error("Not in a git repository");
                } else {
                    print_error(&e.to_string());
                }
                std::process::exit(1);
            }
        }
        Commands::Checkout {
            create_branch,
            indices,
        } => {
            if let Err(e) = execute_checkout_with_flags(create_branch, indices) {
                if let GitNavigatorError::NotInGitRepo = e {
                    print_error("Not in a git repository");
                } else {
                    print_error(&e.to_string());
                }
                std::process::exit(1);
            }
        }
        Commands::Branches { index } => {
            if let Err(e) = execute_branches(index) {
                if let GitNavigatorError::NotInGitRepo = e {
                    print_error("Not in a git repository");
                } else {
                    print_error(&e.to_string());
                }
                std::process::exit(1);
            }
        }
    }

    Ok(())
}
