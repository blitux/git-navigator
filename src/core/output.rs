//! Unified output formatting utilities for consistent CLI presentation.
//!
//! This module provides standardized formatting functions for all git-navigator output,
//! ensuring consistent colors, spacing, and message structure across commands.
//!
//! # Design Principles
//! - **Consistent color scheme**: Red for errors, blue for commands, bright_black for examples
//! - **Standardized spacing**: Newline before and after all command outputs
//! - **Context-aware messaging**: Command-specific usage examples and error messages
//! - **User-friendly formatting**: Clear visual hierarchy and readable output

use colored::*;

/// Formats and prints an error message with consistent styling
///
/// # Format
/// ```text
///
/// ✕ Error: <message>
///
/// ```
///
/// # Colors
/// - "✕ Error:" in red
/// - Message in white
/// - Newlines before and after for spacing
pub fn print_error(message: &str) {
    println!("\n{} {}\n", "✕ Error:".red(), message.white());
}

/// Formats and prints an error with structured usage information
///
/// # Format
/// ```text
///
/// ✕ Error: <message>.
/// Usage:
///   <usage_pattern1>
///   <usage_pattern2>
///   ...
///
/// Options:
///   <option1>  <description1>
///   <option2>  <description2>
///   ...
///
/// ```
///
/// # Colors
/// - Error prefix in red
/// - Message in white
/// - Usage patterns in blue
/// - Options in bright_black (muted)
pub fn print_error_with_structured_usage(
    message: &str,
    usage_patterns: &[&str],
    options: &[(&str, &str)],
) {
    println!("\n{} {}.\n", "✕ Error:".red(), message.white());
    println!("{}", "Usage:".blue());

    for pattern in usage_patterns {
        println!("  {}", pattern.white());
    }

    if !options.is_empty() {
        println!("\n{}", "Options:".blue());
        for (flag, description) in options {
            println!("  {}  {}", flag.bright_black(), description.bright_black());
        }
    }

    println!();
}

/// Formats and prints a success message with consistent styling
///
/// # Format
/// ```text
///
/// ✓ <message>
///
/// ```
///
/// # Colors
/// - Checkmark in green, message in white
/// - Newlines before and after for spacing
pub fn print_success(message: &str) {
    println!("\n{} {}", "✓".green(), message.white());
}

/// Formats and prints an informational message with consistent styling
///
/// # Format
/// ```text
///
/// <message>
///
/// ```
///
/// # Colors
/// - Message in white
/// - Newlines before and after for spacing
pub fn print_info(message: &str) {
    println!("\n{}\n", message.white());
}

/// Formats and prints a section header with consistent styling
///
/// # Format
/// ```text
///
/// <header>:
///
/// ```
///
/// # Colors
/// - Header in white
/// - Newlines before and after for spacing
pub fn print_section_header(header: &str) {
    println!("\n{}:\n", header.white());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print_error_does_not_panic() {
        print_error("Test error message");
    }

    #[test]
    fn test_print_success_does_not_panic() {
        print_success("Operation completed");
    }

    #[test]
    fn test_print_info_does_not_panic() {
        print_info("Information message");
    }

    #[test]
    fn test_print_section_header_does_not_panic() {
        print_section_header("Local Branches");
    }

    #[test]
    fn test_color_functions_available() {
        // Test that color functions are available and don't panic
        let _ = "test".red();
        let _ = "test".white();
        let _ = "test".blue();
        let _ = "test".bright_black();
    }
}
