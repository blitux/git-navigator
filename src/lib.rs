//! Git Navigator - A lightweight Rust CLI tool for numbered file navigation in Git operations.
//!
//! This library provides the core functionality for git-navigator, including git operations,
//! file indexing, state management, and UI formatting. It is designed to be fast, type-safe,
//! and user-friendly.
//!
//! # Public API
//! The main public interface is re-exported from the [`core`] module, which provides:
//! - Git repository operations
//! - File status management and caching  
//! - Index parsing and validation
//! - Error handling and result types
//! - UI templates and color system

pub mod commands;
pub mod core;

// Re-export the core public API for external users
pub use core::{
    format_file_status,
    get_aligned_status,
    get_colored_path,
    get_legend_status,
    // Color system (core functions)
    get_status_color_style,
    render_template,
    render_template_plain,
    strip_ansi_codes,

    ArgsParser,

    BranchEntry,
    // State management
    FileEntry,
    // Error handling
    GitNavigatorError,
    // Git operations
    GitRepo,
    GitStatus,

    IndexCommandContext,

    // Command initialization
    IndexCommandInit,
    // Index parsing and argument handling
    IndexParser,
    IndexRange,
    Result,

    StateCache,

    TemplateContext,
    // UI and formatting
    Templates,
    TEMPLATES,
};
