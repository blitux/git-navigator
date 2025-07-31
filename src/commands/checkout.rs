use crate::commands::status::{execute_status, print_files_only};
use crate::core::{
    command_init::IndexCommandInit,
    error::{GitNavigatorError, Result},
    git::GitRepo,
    print_error, print_error_with_structured_usage, print_info, print_success,
};

pub fn execute_checkout_with_flags(create_branch: bool, indices_args: Vec<String>) -> Result<()> {
    // Handle branch creation flag
    if create_branch {
        if indices_args.is_empty() {
            print_error_with_structured_usage(
                "Branch name required with -b flag",
                &["gco -b <branch-name>"],
                &[
                    ("-b, --create", "Create and switch to a new branch"),
                    ("-h, --help", "Show this help message"),
                ],
            );
            return Ok(());
        }
        if indices_args.len() > 1 {
            print_error_with_structured_usage(
                "Only one branch name allowed with -b flag",
                &["gco -b <branch-name>"],
                &[
                    ("-b, --create", "Create and switch to a new branch"),
                    ("-h, --help", "Show this help message"),
                ],
            );
            return Ok(());
        }
        return create_and_checkout_branch(&indices_args[0]);
    }

    // Delegate to original function for backward compatibility
    execute_checkout(indices_args)
}

pub fn execute_checkout(indices_args: Vec<String>) -> Result<()> {
    // If no arguments provided, show usage
    if indices_args.is_empty() {
        print_error_with_structured_usage(
            "No file indices or branch name provided",
            &["gco <index>...", "gco <branch>", "gco -b <branch-name>"],
            &[
                ("-b, --create", "Create and switch to a new branch"),
                ("-h, --help", "Show this help message"),
            ],
        );
        return Ok(());
    }

    // Check if this might be a branch name (single argument, not numeric)
    if indices_args.len() == 1 {
        let arg = &indices_args[0];

        // Check for branch creation syntax (-b flag)
        if arg == "-b" {
            print_error_with_structured_usage(
                "Branch name required after -b flag",
                &["gco -b <branch-name>"],
                &[
                    ("-b, --create", "Create and switch to a new branch"),
                    ("-h, --help", "Show this help message"),
                ],
            );
            return Ok(());
        }

        // If it's not a pure number or range, treat as potential branch name
        if !is_numeric_index(arg) {
            return checkout_branch_by_name(arg);
        }
    }

    // Check for branch creation syntax (-b branch_name)
    if indices_args.len() == 2 && indices_args[0] == "-b" {
        return create_and_checkout_branch(&indices_args[1]);
    }

    // Otherwise, treat as file indices
    checkout_files_by_indices(indices_args)
}

fn is_numeric_index(arg: &str) -> bool {
    // Check if the argument contains only digits, commas, dashes, and spaces
    // This covers: "1", "1,2", "1-3", "1 2", "1-3,5"
    arg.chars()
        .all(|c| c.is_ascii_digit() || c == ',' || c == '-' || c == ' ')
}

fn checkout_files_by_indices(indices_args: Vec<String>) -> Result<()> {
    // Initialize everything needed for this index-based command
    let context = match IndexCommandInit::initialize_with_messages(
        indices_args,
        "Cannot load file cache",
        "No files available to checkout",
    ) {
        Ok(context) => context,
        Err(GitNavigatorError::NoIndicesProvided) => {
            print_error_with_structured_usage(
                "No file indices provided",
                &["gco <index>..."],
                &[("-h, --help", "Show this help message")],
            );
            return Ok(());
        }
        Err(e) => return Err(e),
    };

    // Check if there are any changes available to checkout
    let current_status = context.git_repo.get_status()?;
    if current_status.is_empty() {
        print_error("There are no changes to checkout");
        print_info("Current status:");
        execute_status()?;
        return Ok(());
    }

    // Get the selected files and prepare them for checkout
    let selected_files = context.get_selected_files();

    // Extract paths for checkout
    let paths_to_checkout: Vec<_> = selected_files
        .iter()
        .map(|file| &file.path)
        .cloned()
        .collect();

    if paths_to_checkout.is_empty() {
        return Err(GitNavigatorError::NoValidFilesSelected);
    }

    // Checkout files using git
    match context.git_repo.checkout_files(&paths_to_checkout) {
        Ok(()) => {
            print_success(&format!(
                "Successfully checked out {} file(s).",
                selected_files.len()
            ));
        }
        Err(e) => {
            return Err(e);
        }
    }

    // Show updated status
    print_info("Updated status:");
    let updated_files = context.git_repo.get_status()?;
    print_files_only(&updated_files);

    Ok(())
}

fn checkout_branch_by_name(branch_name: &str) -> Result<()> {
    let git_repo = GitRepo::open(".")?;

    match git_repo.checkout_branch(branch_name) {
        Ok(()) => {
            print_success(&format!(
                "Successfully switched to branch '{branch_name}'"
            ));
        }
        Err(e) => {
            print_error(&format!(
                "Failed to checkout branch '{branch_name}': {e}"
            ));
            return Err(e);
        }
    }

    Ok(())
}

fn create_and_checkout_branch(branch_name: &str) -> Result<()> {
    let git_repo = GitRepo::open(".")?;

    match git_repo.create_branch(branch_name) {
        Ok(()) => {
            print_success(&format!(
                "Successfully created and switched to branch '{branch_name}'"
            ));
        }
        Err(e) => {
            print_error(&format!("Failed to create branch '{branch_name}': {e}"));
            return Err(e);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_numeric_index() {
        assert!(is_numeric_index("1"));
        assert!(is_numeric_index("1,2,3"));
        assert!(is_numeric_index("1-3"));
        assert!(is_numeric_index("1 2 3"));
        assert!(is_numeric_index("1-3,5"));
        assert!(is_numeric_index("1 3-5,8"));

        assert!(!is_numeric_index("main"));
        assert!(!is_numeric_index("feature-branch"));
        assert!(!is_numeric_index("fix/bug-123"));
        assert!(!is_numeric_index("-b"));
        assert!(!is_numeric_index("abc"));
    }

    #[test]
    fn test_execute_checkout_no_args() {
        let result = execute_checkout(vec![]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_checkout_branch_creation_incomplete() {
        let result = execute_checkout(vec!["-b".to_string()]);
        assert!(result.is_ok());
    }
}
