//! High-level command argument parsing with validation.
//!
//! This module provides [`ArgsParser`] which combines index parsing with file count
//! validation to provide a complete argument processing solution for commands that
//! work with file indices.
//!
//! # Public API
//! - [`ArgsParser`]: Main parser that validates indices against available files
//!
//! # Features
//! - **Unified parsing**: Handles all argument formats in one call
//! - **Validation**: Ensures indices are within valid file bounds
//! - **Error handling**: Provides user-friendly error messages
//! - **Convenience methods**: Helper functions for argument inspection

use crate::core::{
    error::{GitNavigatorError, Result},
    index_parser::IndexParser,
};

/// Centralized argument parsing for commands that take file indices
pub struct ArgsParser;

impl ArgsParser {
    /// Parse command line arguments into validated indices
    ///
    /// Takes a Vec<String> from clap and returns validated indices ready to use.
    /// Handles all the conversion and validation logic in one place.
    ///
    /// # Arguments
    /// * `args` - Command line arguments from clap (e.g., ["1", "3-5", "8"])
    /// * `file_count` - Total number of files available for validation
    ///
    /// # Returns
    /// * `Ok(Vec<usize>)` - Parsed and validated indices (1-based)
    /// * `Err` - If no arguments provided, parsing failed, or validation failed
    ///
    /// # Examples
    /// ```no_run
    /// use git_navigator::core::args_parser::ArgsParser;
    ///
    /// // Parse space-separated arguments: ["1", "3", "5"]
    /// let indices = ArgsParser::parse_indices(vec!["1".to_string(), "3".to_string(), "5".to_string()], 10)?;
    /// assert_eq!(indices, vec![1, 3, 5]);
    ///
    /// // Parse range arguments: ["1-3", "5"]
    /// let indices = ArgsParser::parse_indices(vec!["1-3".to_string(), "5".to_string()], 10)?;
    /// assert_eq!(indices, vec![1, 2, 3, 5]);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn parse_indices(args: Vec<String>, file_count: usize) -> Result<Vec<usize>> {
        // Check if arguments were provided
        if args.is_empty() {
            return Err(GitNavigatorError::NoIndicesProvided);
        }

        // Join all arguments with spaces to create a single string for IndexParser
        // This handles cases like: ["1", "3-5,8"] -> "1 3-5,8"
        let indices_str = args.join(" ");

        // Parse the indices string using the existing IndexParser
        let indices = IndexParser::parse(&indices_str)
            .map_err(|e| GitNavigatorError::invalid_index_format(e.to_string()))?;

        // Check if parsing resulted in empty indices (could happen with empty strings)
        if indices.is_empty() {
            return Err(GitNavigatorError::NoValidIndices);
        }

        // Validate that indices are within bounds
        IndexParser::validate(&indices, file_count)?;

        // Return the validated indices
        Ok(indices)
    }

    /// Check if arguments were provided (for better error messages)
    pub fn has_args(args: &[String]) -> bool {
        !args.is_empty()
    }

    /// Get argument count for logging/debugging
    pub fn arg_count(args: &[String]) -> usize {
        args.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_single_index() -> Result<()> {
        let args = vec!["1".to_string()];
        let result = ArgsParser::parse_indices(args, 5)?;
        assert_eq!(result, vec![1]);
        Ok(())
    }

    #[test]
    fn test_parse_multiple_indices() -> Result<()> {
        let args = vec!["1".to_string(), "3".to_string(), "5".to_string()];
        let result = ArgsParser::parse_indices(args, 5)?;
        assert_eq!(result, vec![1, 3, 5]);
        Ok(())
    }

    #[test]
    fn test_parse_range() -> Result<()> {
        let args = vec!["1-3".to_string()];
        let result = ArgsParser::parse_indices(args, 5)?;
        assert_eq!(result, vec![1, 2, 3]);
        Ok(())
    }

    #[test]
    fn test_parse_mixed_format() -> Result<()> {
        let args = vec!["1".to_string(), "3-5".to_string(), "8".to_string()];
        let result = ArgsParser::parse_indices(args, 10)?;
        assert_eq!(result, vec![1, 3, 4, 5, 8]);
        Ok(())
    }

    #[test]
    fn test_parse_comma_separated_as_single_arg() -> Result<()> {
        let args = vec!["1,3,5".to_string()];
        let result = ArgsParser::parse_indices(args, 5)?;
        assert_eq!(result, vec![1, 3, 5]);
        Ok(())
    }

    #[test]
    fn test_parse_empty_args() {
        let args = vec![];
        let result = ArgsParser::parse_indices(args, 5);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("No file indices provided"));
    }

    #[test]
    fn test_parse_invalid_index() {
        let args = vec!["abc".to_string()];
        let result = ArgsParser::parse_indices(args, 5);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid index format"));
    }

    #[test]
    fn test_parse_index_out_of_bounds() {
        let args = vec!["10".to_string()];
        let result = ArgsParser::parse_indices(args, 5);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("out of range"));
    }

    #[test]
    fn test_has_args() {
        assert!(ArgsParser::has_args(&vec!["1".to_string()]));
        assert!(!ArgsParser::has_args(&vec![]));
    }

    #[test]
    fn test_arg_count() {
        assert_eq!(
            ArgsParser::arg_count(&vec!["1".to_string(), "2".to_string()]),
            2
        );
        assert_eq!(ArgsParser::arg_count(&vec![]), 0);
    }
}
