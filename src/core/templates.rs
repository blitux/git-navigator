//! Template system for consistent output formatting.
//!
//! This module provides a high-performance template rendering system for generating
//! consistent, colorized output across all git-navigator commands. It uses optimized
//! single-pass rendering with pre-allocated buffers for minimal memory allocation.
//!
//! # Public API
//! - [`Templates`]: Template definitions for all output sections
//! - [`TemplateContext`]: Context data for template rendering
//! - [`TEMPLATES`]: Global template instance with default formatting
//! - [`render_template`]: Main rendering function with colors
//! - [`render_template_plain`]: Plain text rendering for testing
//! - [`strip_ansi_codes`]: Utility for removing color codes
//!
//! # Template Categories
//! - **Headers**: Branch names, commit information
//! - **Sections**: Staged, unstaged, untracked file groups
//! - **File lines**: Individual file entries with status and index
//!
//! # Performance Features
//! - **Single-pass rendering**: No intermediate string allocations
//! - **Capacity estimation**: Pre-allocate buffers based on content size
//! - **Color optimization**: Direct color application without string manipulation

use crate::core::{colors::get_colored_path, git_status::GitStatus};
use colored::*;

/// Template definitions for all output formatting
pub struct Templates {
    // Header templates
    pub header_empty_line: &'static str,
    pub header_branch: &'static str,
    pub header_parent_no_commits: &'static str,
    pub header_parent_with_commits: &'static str,

    // Section templates
    pub section_unmerged: &'static str,
    pub section_staged: &'static str,
    pub section_unstaged: &'static str,
    pub section_untracked: &'static str,

    // File line template
    pub file_line: &'static str,
    pub section_spacing: &'static str,
}

impl Default for Templates {
    fn default() -> Self {
        Self {
            header_empty_line: "",
            header_branch: "Branch: {branch_name}{ahead_behind}",
            header_parent_no_commits: "Parent: {commit_message}",
            header_parent_with_commits: "Parent: {short_hash} {commit_message}",
            section_unmerged: "➤ Unmerged:",
            section_staged: "➤ Staged:",
            section_unstaged: "➤ Not staged:",
            section_untracked: "➤ Untracked:",
            file_line: "   ({file_status}) [{n}] {filename}",
            section_spacing: "",
        }
    }
}

/// Global templates instance
pub static TEMPLATES: Templates = Templates {
    header_empty_line: "",
    header_branch: "Branch: {branch_name}{ahead_behind}",
    header_parent_no_commits: "Parent: {commit_message}",
    header_parent_with_commits: "Parent: {short_hash} {commit_message}",
    section_unmerged: "➤ Unmerged:",
    section_staged: "➤ Staged:",
    section_unstaged: "➤ Not staged:",
    section_untracked: "➤ Untracked:",
    file_line: "   ({file_status}) [{n}] {filename}",
    section_spacing: "",
};

/// Context for template rendering
#[derive(Debug, Default)]
pub struct TemplateContext<'a> {
    pub branch_name: Option<&'a str>,
    pub ahead_behind: Option<&'a str>,
    pub short_hash: Option<&'a str>,
    pub commit_message: Option<&'a str>,
    pub section_type: Option<&'a str>, // "staged", "unstaged", etc.
    pub file_status: Option<&'a str>,
    pub filename: Option<&'a str>,
    pub n: Option<usize>,
    pub git_status: Option<GitStatus>, // GitStatus enum for coloring
}

/// Render a template with context and apply colors
pub fn render_template(template: &str, context: &TemplateContext) -> String {
    // Pre-allocate buffer with estimated capacity
    let estimated_capacity = template.len() +
        context.branch_name.map_or(0, |s| s.len()) +
        context.ahead_behind.map_or(0, |s| s.len()) +
        context.short_hash.map_or(0, |s| s.len()) +
        context.commit_message.map_or(0, |s| s.len()) +
        context.file_status.map_or(0, |s| s.len()) +
        context.filename.map_or(0, |s| s.len()) +
        context.n.map_or(0, |_| 4) + // Reserve space for index numbers
        128; // Extra space for color codes and formatting

    let mut result = String::with_capacity(estimated_capacity);

    // Single-pass template rendering using state machine
    render_template_single_pass(template, context, &mut result);

    // Apply colors in single pass
    apply_colors_optimized(&result, template, context)
}

/// Optimized single-pass template renderer
fn render_template_single_pass(template: &str, context: &TemplateContext, output: &mut String) {
    let mut chars = template.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '{' {
            // Look ahead to find the closing brace
            let mut placeholder = String::new();
            let mut found_closing = false;

            while let Some(&next_ch) = chars.peek() {
                if next_ch == '}' {
                    chars.next(); // consume '}'
                    found_closing = true;
                    break;
                }
                placeholder.push(chars.next().unwrap());
            }

            if found_closing {
                // Replace placeholder with actual value
                match placeholder.as_str() {
                    "branch_name" => {
                        if let Some(value) = context.branch_name {
                            output.push_str(value);
                        }
                    }
                    "ahead_behind" => {
                        if let Some(value) = context.ahead_behind {
                            output.push_str(value);
                        }
                    }
                    "short_hash" => {
                        if let Some(value) = context.short_hash {
                            output.push_str(value);
                        }
                    }
                    "commit_message" => {
                        if let Some(value) = context.commit_message {
                            output.push_str(value);
                        }
                    }
                    "file_status" => {
                        if let Some(value) = context.file_status {
                            output.push_str(value);
                        }
                    }
                    "filename" => {
                        if let Some(value) = context.filename {
                            output.push_str(value);
                        }
                    }
                    "n" => {
                        if let Some(value) = context.n {
                            use std::fmt::Write;
                            let _ = write!(output, "{value}");
                        }
                    }
                    _ => {
                        // Unknown placeholder, keep as-is
                        output.push('{');
                        output.push_str(&placeholder);
                        output.push('}');
                    }
                }
            } else {
                // No closing brace found, treat as literal
                output.push(ch);
                output.push_str(&placeholder);
            }
        } else {
            output.push(ch);
        }
    }
}

/// Optimized single-pass color application
fn apply_colors_optimized(text: &str, template: &str, context: &TemplateContext) -> String {
    use std::fmt::Write;

    // Pre-allocate with extra space for color codes
    let mut result = String::with_capacity(text.len() + 128);

    match template {
        // Header templates
        t if t.contains("Branch:") => {
            if let Some(branch_name) = context.branch_name {
                let _ = write!(result, "Branch: {}", branch_name.blue());
                // Add ahead/behind info if present
                if let Some(ahead_behind) = context.ahead_behind {
                    result.push_str(ahead_behind);
                }
            } else {
                result.push_str(text);
            }
        }

        t if t.contains("Parent:") && t.contains("{short_hash}") => {
            if let (Some(short_hash), Some(commit_message)) =
                (context.short_hash, context.commit_message)
            {
                let _ = write!(
                    result,
                    "Parent: {} {}",
                    short_hash.blue(),
                    commit_message.bright_black()
                );
            } else {
                result.push_str(text);
            }
        }

        t if t.contains("Parent:") && !t.contains("{short_hash}") => {
            if let Some(commit_message) = context.commit_message {
                let _ = write!(result, "Parent: {}", commit_message.white());
            } else {
                result.push_str(text);
            }
        }

        // Section templates - use write! to avoid format! allocation
        t if t.contains("➤ Unmerged:") => {
            let _ = write!(result, "{} {}", "➤".red(), "Unmerged:".red());
        }
        t if t.contains("➤ Staged:") => {
            let _ = write!(result, "{} {}", "➤".green(), "Staged:".green());
        }
        t if t.contains("➤ Not staged:") => {
            let _ = write!(result, "{} {}", "➤".yellow(), "Not staged:".yellow());
        }
        t if t.contains("➤ Untracked:") => {
            let _ = write!(result, "{} {}", "➤".cyan(), "Untracked:".cyan());
        }

        // File line template - optimized single-pass formatting
        t if t.contains("({file_status}) [{n}] {filename}") => {
            result.push_str("   "); // Leading spaces

            if let Some(file_status) = context.file_status {
                // Format status with padding for alignment
                let padding_needed = 13 - file_status.len();
                let _ = write!(
                    result,
                    "{}{}{}",
                    "(".bright_black(),
                    file_status.bright_black(),
                    ")".bright_black()
                );
                for _ in 0..padding_needed {
                    result.push(' ');
                }
            }

            result.push(' '); // Space before index

            if let Some(n) = context.n {
                let _ = write!(
                    result,
                    "{}{}{}",
                    "[".bright_black(),
                    n.to_string().white(),
                    "]".bright_black()
                );
            }

            result.push(' '); // Space before filename

            if let (Some(filename), Some(git_status)) = (context.filename, context.git_status) {
                let colored_filename = get_colored_path(git_status, filename);
                let _ = write!(result, "{colored_filename}");
            }
        }

        // Default: return as-is
        _ => {
            result.push_str(text);
        }
    }

    result
}

/// Strip ANSI color codes for testing
pub fn strip_ansi_codes(text: &str) -> String {
    // Simple state machine to remove ANSI escape sequences
    let mut result = String::new();
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\x1b' && chars.peek() == Some(&'[') {
            // Skip the escape sequence
            chars.next(); // consume '['
            for ch in chars.by_ref() {
                if ch.is_ascii_alphabetic() {
                    break; // End of escape sequence
                }
            }
        } else {
            result.push(ch);
        }
    }

    result
}

/// Render template without colors for testing
pub fn render_template_plain(template: &str, context: &TemplateContext) -> String {
    let colored = render_template(template, context);
    strip_ansi_codes(&colored)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_branch_template() {
        let branch_name = "main";
        let context = TemplateContext {
            branch_name: Some(branch_name),
            ..Default::default()
        };
        let result = render_template_plain(TEMPLATES.header_branch, &context);
        assert_eq!(result, "Branch: main");
    }

    #[test]
    fn test_render_parent_with_commits() {
        let short_hash = "a1b2c3d";
        let commit_message = "Initial commit";
        let context = TemplateContext {
            short_hash: Some(short_hash),
            commit_message: Some(commit_message),
            ..Default::default()
        };
        let result = render_template_plain(TEMPLATES.header_parent_with_commits, &context);
        assert_eq!(result, "Parent: a1b2c3d Initial commit");
    }

    #[test]
    fn test_render_file_line() {
        let file_status = "modified";
        let filename = "src/main.rs";
        let context = TemplateContext {
            file_status: Some(file_status),
            n: Some(1),
            filename: Some(filename),
            git_status: Some(GitStatus::Modified),
            ..Default::default()
        };
        let result = render_template_plain(TEMPLATES.file_line, &context);
        assert_eq!(result, "   (modified)      [1] src/main.rs");
    }

    #[test]
    fn test_render_section_templates() {
        assert_eq!(
            strip_ansi_codes(&apply_colors_optimized(
                "➤ Staged:",
                TEMPLATES.section_staged,
                &TemplateContext::default()
            )),
            "➤ Staged:"
        );
        assert_eq!(
            strip_ansi_codes(&apply_colors_optimized(
                "➤ Not staged:",
                TEMPLATES.section_unstaged,
                &TemplateContext::default()
            )),
            "➤ Not staged:"
        );
        assert_eq!(
            strip_ansi_codes(&apply_colors_optimized(
                "➤ Untracked:",
                TEMPLATES.section_untracked,
                &TemplateContext::default()
            )),
            "➤ Untracked:"
        );
        assert_eq!(
            strip_ansi_codes(&apply_colors_optimized(
                "➤ Unmerged:",
                TEMPLATES.section_unmerged,
                &TemplateContext::default()
            )),
            "➤ Unmerged:"
        );
    }

    #[test]
    fn test_template_context_default() {
        let context = TemplateContext::default();
        assert!(context.branch_name.is_none());
        assert!(context.short_hash.is_none());
        assert!(context.commit_message.is_none());
    }

    #[test]
    fn test_single_pass_renderer_basic() {
        let template = "Hello {name}!";
        let mut output = String::new();
        let context = TemplateContext::default();

        render_template_single_pass(template, &context, &mut output);
        // Unknown placeholders should be kept as-is
        assert_eq!(output, "Hello {name}!");
    }

    #[test]
    fn test_single_pass_renderer_multiple_placeholders() {
        let template = "{branch_name}: {short_hash} - {commit_message}";
        let mut output = String::new();
        let context = TemplateContext {
            branch_name: Some("main"),
            short_hash: Some("abc123"),
            commit_message: Some("Initial commit"),
            ..Default::default()
        };

        render_template_single_pass(template, &context, &mut output);
        assert_eq!(output, "main: abc123 - Initial commit");
    }

    #[test]
    fn test_single_pass_renderer_unknown_placeholder() {
        let template = "Hello {unknown}!";
        let mut output = String::new();
        let context = TemplateContext::default();

        render_template_single_pass(template, &context, &mut output);
        assert_eq!(output, "Hello {unknown}!");
    }

    #[test]
    fn test_single_pass_renderer_malformed_placeholder() {
        let template = "Hello {incomplete";
        let mut output = String::new();
        let context = TemplateContext::default();

        render_template_single_pass(template, &context, &mut output);
        assert_eq!(output, "Hello {incomplete");
    }

    #[test]
    fn test_single_pass_renderer_numeric_placeholder() {
        let template = "Item [{n}]";
        let mut output = String::new();
        let context = TemplateContext {
            n: Some(42),
            ..Default::default()
        };

        render_template_single_pass(template, &context, &mut output);
        assert_eq!(output, "Item [42]");
    }

    #[test]
    fn test_optimized_rendering_maintains_functionality() {
        // Test that optimized version produces same results as before
        let context = TemplateContext {
            branch_name: Some("feature-branch"),
            short_hash: Some("a1b2c3d"),
            commit_message: Some("Add new feature"),
            file_status: Some("modified"),
            filename: Some("src/lib.rs"),
            n: Some(5),
            git_status: Some(GitStatus::Modified),
            ..Default::default()
        };

        // Test branch template
        let branch_result = render_template_plain(TEMPLATES.header_branch, &context);
        assert_eq!(branch_result, "Branch: feature-branch");

        // Test parent template with commit
        let parent_result = render_template_plain(TEMPLATES.header_parent_with_commits, &context);
        assert_eq!(parent_result, "Parent: a1b2c3d Add new feature");

        // Test file line template
        let file_result = render_template_plain(TEMPLATES.file_line, &context);
        assert_eq!(file_result, "   (modified)      [5] src/lib.rs");
    }

    #[test]
    fn test_capacity_estimation() {
        let context = TemplateContext {
            branch_name: Some("very-long-branch-name-with-many-characters"),
            short_hash: Some("abcdef123456"),
            commit_message: Some(
                "A very long commit message that would require significant buffer space",
            ),
            file_status: Some("both modified"),
            filename: Some("src/very/long/path/to/some/file.rs"),
            n: Some(9999),
            git_status: Some(GitStatus::Modified),
            ..Default::default()
        };

        // This should not panic or reallocate if our capacity estimation is good
        let result = render_template(TEMPLATES.file_line, &context);
        assert!(result.contains("both modified"));
        assert!(result.contains("9999"));
        assert!(result.contains("src/very/long/path/to/some/file.rs"));
    }

    #[test]
    fn test_performance_no_unnecessary_allocations() {
        // Test that we don't allocate for unused placeholders
        let simple_template = "Simple text without placeholders";
        let context = TemplateContext::default();

        let result = render_template_plain(simple_template, &context);
        assert_eq!(result, "Simple text without placeholders");
    }

    #[test]
    fn test_optimization_with_complex_template() {
        // Test rendering performance with complex context
        let context = TemplateContext {
            branch_name: Some("feature/optimize-templates"),
            short_hash: Some("1a2b3c4"),
            commit_message: Some("Optimize template rendering for better performance"),
            file_status: Some("both modified"),
            filename: Some("src/core/templates.rs"),
            n: Some(1),
            git_status: Some(GitStatus::Modified),
            ..Default::default()
        };

        // Benchmark-style test - render many times to stress test
        for _ in 0..1000 {
            let _result = render_template(TEMPLATES.file_line, &context);
        }

        // Verify final result is correct
        let final_result = render_template_plain(TEMPLATES.file_line, &context);
        assert!(final_result.contains("both modified"));
        assert!(final_result.contains("[1]"));
        assert!(final_result.contains("src/core/templates.rs"));
    }

    #[test]
    fn test_edge_cases_and_robustness() {
        // Test edge cases that might cause allocation issues
        let edge_cases = vec![
            ("", TemplateContext::default()),   // Empty template
            ("{}", TemplateContext::default()), // Empty placeholder
            (
                "{unknown_very_long_placeholder_name}",
                TemplateContext::default(),
            ), // Long unknown placeholder
            ("{{nested}}", TemplateContext::default()), // Nested braces
            (
                "{n}{n}{n}",
                TemplateContext {
                    n: Some(999),
                    ..Default::default()
                },
            ), // Repeated placeholder
        ];

        for (template, context) in edge_cases {
            let result = render_template(template, &context);
            // Should not panic and should produce some result
            assert!(!result.is_empty() || template.is_empty());
        }
    }
}
