//! Core functionality for the git-navigator tool.
//!
//! This module provides the fundamental building blocks for git operations,
//! file indexing, error handling, and UI components.

pub mod args_parser;
pub mod colors;
pub mod command_init;
pub mod config;
pub mod dirs;
pub mod error;
pub mod git;
pub mod git_status;
pub mod index_parser;
pub mod output;
pub mod state;
pub mod templates;

// === Error handling ===
// Core error types and result type used throughout the application
pub use error::{GitNavigatorError, Result};

// === Git operations ===
// Main git repository interface for status, adding files, etc.
pub use git::GitRepo;

// === Git status types ===
// Type-safe git status enumeration to replace string-based status codes
pub use git_status::GitStatus;

// === State management ===
// Data structures for caching file and branch information
pub use state::{BranchEntry, FileEntry, StateCache};

// === Index parsing ===
// Parser for handling user input like "1 3-5,8" -> [1, 3, 4, 5, 8]
pub use index_parser::{IndexParser, IndexRange};

// === Argument parsing ===
// High-level command argument parsing that combines index parsing with validation
pub use args_parser::ArgsParser;

// === Command initialization ===
// Centralized initialization for commands that work with file indices
pub use command_init::{IndexCommandContext, IndexCommandInit};

// === UI templates ===
// Template system for consistent output formatting with colors
pub use templates::{
    render_template, render_template_plain, strip_ansi_codes, TemplateContext, Templates, TEMPLATES,
};

// === Color system ===
// Unified color system for consistent git status coloring
pub use colors::{
    format_file_status,
    get_aligned_status,
    get_aligned_status_legacy,
    get_colored_path,
    get_colored_path_legacy,
    get_legend_status,
    get_legend_status_legacy,
    get_status_color_style,
    // Legacy functions for backward compatibility during migration
    get_status_color_style_legacy,
};

// === Output formatting ===
// Unified output formatting for consistent CLI presentation
pub use output::{
    print_error, print_error_with_structured_usage, print_info, print_section_header, print_success,
};
