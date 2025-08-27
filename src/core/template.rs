//! Command template system for consistent output formatting.
//!
//! This module provides a template-based approach to command output that ensures
//! consistent spacing, coloring, and layout across all git-navigator commands.
//!
//! # Usage Example
//! ```rust
//! use git_navigator::core::CommandTemplate;
//! 
//! let table = "  [1] main\n  [2] feature-branch";
//! CommandTemplate::new()
//!     .title("Local Branches")
//!     .body(table)
//!     .help("Use gb --remote to list remote branches.")
//!     .print();
//! ```
//!
//! # Template Format
//! All commands follow this consistent layout:
//! - Opening whitespace
//! - Title (blue colored)  
//! - Spacing after title
//! - Body content (preserves original formatting)
//! - Spacing before help
//! - Help text (bright black)
//! - Closing whitespace

use colored::*;
use std::fmt::Display;

/// Represents different types of content sections in a command template
#[derive(Debug, Clone)]
pub enum TemplateSection {
    /// Main title/header - rendered in blue
    Title(String),
    /// Main content body - preserves original formatting
    Body(String),
    /// Help message - rendered in bright black
    Help(String),
    /// Warning message - rendered in yellow
    Warning(String),
    /// Error message - rendered in red
    Error(String),
    /// Success message - rendered in green
    Success(String),
    /// Custom section with optional color override
    Custom { content: String, color: Option<String> },
}

/// Template for consistent command output formatting
#[derive(Debug, Default)]
pub struct CommandTemplate {
    sections: Vec<TemplateSection>,
}

impl CommandTemplate {
    /// Create a new empty template
    pub fn new() -> Self {
        Self {
            sections: Vec::new(),
        }
    }

    /// Add a title section (rendered in blue)
    pub fn title(mut self, text: impl Into<String>) -> Self {
        self.sections.push(TemplateSection::Title(text.into()));
        self
    }

    /// Add a body section (preserves original formatting)
    pub fn body(mut self, content: impl Display) -> Self {
        self.sections.push(TemplateSection::Body(content.to_string()));
        self
    }

    /// Add a help section (rendered in bright black)
    pub fn help(mut self, text: impl Into<String>) -> Self {
        self.sections.push(TemplateSection::Help(text.into()));
        self
    }

    /// Add a warning section (rendered in yellow)
    pub fn warning(mut self, text: impl Into<String>) -> Self {
        self.sections.push(TemplateSection::Warning(text.into()));
        self
    }

    /// Add an error section (rendered in red)
    pub fn error(mut self, text: impl Into<String>) -> Self {
        self.sections.push(TemplateSection::Error(text.into()));
        self
    }

    /// Add a success section (rendered in green)
    pub fn success(mut self, text: impl Into<String>) -> Self {
        self.sections.push(TemplateSection::Success(text.into()));
        self
    }

    /// Add a custom section with optional color
    pub fn custom(mut self, content: impl Into<String>, color: Option<impl Into<String>>) -> Self {
        self.sections.push(TemplateSection::Custom {
            content: content.into(),
            color: color.map(|c| c.into()),
        });
        self
    }

    /// Render the template to a string (useful for testing)
    pub fn render(self) -> String {
        let mut output = String::new();
        
        // Opening whitespace
        output.push('\n');

        for (index, section) in self.sections.iter().enumerate() {
            match section {
                TemplateSection::Title(text) => {
                    output.push_str(&text.blue().to_string());
                    output.push('\n');
                    // Add spacing after title if there are more sections
                    if index < self.sections.len() - 1 {
                        output.push('\n');
                    }
                }
                TemplateSection::Body(content) => {
                    output.push_str(content);
                    output.push('\n');
                }
                TemplateSection::Help(text) => {
                    // Add spacing before help if not first section
                    if index > 0 {
                        output.push('\n');
                    }
                    output.push_str(&text.bright_black().to_string());
                    output.push('\n');
                }
                TemplateSection::Warning(text) => {
                    if index > 0 {
                        output.push('\n');
                    }
                    output.push_str(&text.yellow().to_string());
                    output.push('\n');
                }
                TemplateSection::Error(text) => {
                    if index > 0 {
                        output.push('\n');
                    }
                    output.push_str(&text.red().to_string());
                    output.push('\n');
                }
                TemplateSection::Success(text) => {
                    if index > 0 {
                        output.push('\n');
                    }
                    output.push_str(&text.green().to_string());
                    output.push('\n');
                }
                TemplateSection::Custom { content, color } => {
                    if index > 0 {
                        output.push('\n');
                    }
                    match color {
                        Some(color_name) => {
                            // For now, support basic colors - can be extended
                            let colored_content = match color_name.as_str() {
                                "blue" => content.blue().to_string(),
                                "red" => content.red().to_string(),
                                "green" => content.green().to_string(),
                                "yellow" => content.yellow().to_string(),
                                "bright_black" => content.bright_black().to_string(),
                                _ => content.clone(),
                            };
                            output.push_str(&colored_content);
                        }
                        None => output.push_str(content),
                    }
                    output.push('\n');
                }
            }
        }

        // Closing whitespace
        output.push('\n');

        output
    }

    /// Print the template to stdout
    pub fn print(self) {
        print!("{}", self.render());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_template() {
        let template = CommandTemplate::new();
        let output = template.render();
        assert_eq!(output, "\n\n"); // Just opening and closing whitespace
    }

    #[test]
    fn test_title_only() {
        let template = CommandTemplate::new()
            .title("Test Title");
        let output = template.render();
        // Should contain title (we can't easily test colors in unit tests)
        assert!(output.contains("Test Title"));
        assert!(output.starts_with('\n')); // Opening whitespace
        assert!(output.ends_with('\n')); // Closing whitespace
    }

    #[test]
    fn test_full_template() {
        let template = CommandTemplate::new()
            .title("Local Branches")
            .body("branch list content")
            .help("Use gb --remote for remote branches");
        
        let output = template.render();
        assert!(output.contains("Local Branches"));
        assert!(output.contains("branch list content"));
        assert!(output.contains("Use gb --remote"));
        assert!(output.starts_with('\n'));
        assert!(output.ends_with('\n'));
    }

    #[test]
    fn test_builder_pattern() {
        let template = CommandTemplate::new()
            .title("Title")
            .body("Body")
            .warning("Warning")
            .error("Error")
            .success("Success")
            .help("Help");
        
        assert_eq!(template.sections.len(), 6);
    }
}