use clap::{Parser, Subcommand};
use git_navigator::commands::*;
use git_navigator::core::{
    error::{GitNavigatorError, Result},
    print_error, print_success,
};
use std::env;

/// Helper function to handle common error patterns and exit appropriately
fn handle_command_error(result: Result<()>) {
    if let Err(e) = result {
        match e {
            GitNavigatorError::NotInGitRepo => {
                print_error("Not in a git repository");
            }
            _ => {
                print_error(&e.to_string());
            }
        }
        std::process::exit(1);
    }
}

#[derive(Parser)]
#[command(name = "git-navigator")]
#[command(about = "A lightweight and efficient Git navigation tool")]
#[command(version = env!("CARGO_PKG_VERSION"))]
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
        /// Show remote branches instead of local branches
        #[arg(long)]
        remote: bool,
        /// Branch index to checkout (if provided)
        index: Option<usize>,
    },
    /// Update git-navigator to the latest version
    Update {
        #[command(flatten)]
        args: update::UpdateArgs,
    },
    /// Rollback to a previous version
    Rollback {
        #[command(flatten)]
        args: rollback::RollbackArgs,
    },
    /// Copy files by index (wrapper for cp)
    Copy {
        /// Arguments: indices and/or paths (e.g., "1 3-5 /dest")
        args: Vec<String>,
    },
    /// Remove files by index (wrapper for rm)
    Remove {
        /// Arguments: indices and/or paths (e.g., "1 3-5")
        args: Vec<String>,
    },
    /// Manage shell aliases
    Alias {
        /// Show current alias status
        #[arg(long, default_value_t = true)]
        show: bool,
        /// Update aliases in shell config
        #[arg(long)]
        update: bool,
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
            handle_command_error(execute_status());
        }
        Commands::Add { indices } => {
            handle_command_error(execute_add(indices));
        }
        Commands::Diff { indices } => {
            handle_command_error(execute_diff(indices));
        }
        Commands::Reset { indices } => {
            handle_command_error(execute_reset(indices));
        }
        Commands::Checkout {
            create_branch,
            indices,
        } => {
            handle_command_error(execute_checkout_with_flags(create_branch, indices));
        }
        Commands::Branches { remote, index } => {
            handle_command_error(execute_branches(remote, index));
        }
        Commands::Update { args } => {
            if let Err(e) = update::execute_update(args) {
                match e {
                    GitNavigatorError::AlreadyUpToDate { current } => {
                        print_success(&format!("Already up to date (v{current})"));
                    }
                    GitNavigatorError::UpdateCanceled => {
                        print_error("Update canceled");
                    }
                    _ => {
                        print_error(&e.to_string());
                        std::process::exit(1);
                    }
                }
            }
        }
        Commands::Rollback { args } => {
            if let Err(e) = rollback::execute_rollback(args) {
                print_error(&e.to_string());
                std::process::exit(1);
            }
        }
        Commands::Copy { args } => {
            handle_command_error(execute_copy(args));
        }
        Commands::Remove { args } => {
            handle_command_error(execute_remove(args));
        }
        Commands::Alias { show, update } => {
            handle_command_error(execute_alias(show, update));
        }
    }

    Ok(())
}
