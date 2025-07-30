//! Flexible parsing of user-provided file indices.
//!
//! This module provides [`IndexParser`] which handles the complex task of parsing
//! user input like "1 3-5,8" into validated index lists. It supports multiple
//! input formats and provides comprehensive validation.
//!
//! # Public API
//! - [`IndexParser`]: Main parser with static methods for parsing and validation
//! - [`IndexRange`]: Simple struct representing a numeric range
//!
//! # Supported Formats
//! - **Single indices**: `1`, `3`, `5`
//! - **Space-separated**: `1 3 5`
//! - **Comma-separated**: `1,3,5`  
//! - **Ranges**: `3-6` (expands to 3,4,5,6)
//! - **Mixed combinations**: `1 3-5,8` (expands to 1,3,4,5,8)
//!
//! # Features
//! - **Deduplication**: Automatically removes duplicate indices
//! - **Validation**: Ensures indices are within valid bounds
//! - **Error handling**: Detailed error messages for invalid input

use crate::core::error::{GitNavigatorError, Result};
use std::collections::HashSet;

#[derive(Debug, PartialEq)]
pub struct IndexRange {
    pub start: usize,
    pub end: usize,
}

pub struct IndexParser;

impl IndexParser {
    pub fn parse(input: &str) -> Result<Vec<usize>> {
        if input.trim().is_empty() {
            return Ok(Vec::new());
        }

        let mut indices = HashSet::new();

        // Split by spaces and commas
        let parts: Vec<&str> = input
            .split([' ', ','])
            .filter(|s| !s.trim().is_empty())
            .collect();

        for part in parts {
            let part = part.trim();
            if part.contains('-') {
                // Handle range like "3-6"
                let range_parts: Vec<&str> = part.split('-').collect();
                if range_parts.len() != 2 {
                    return Err(GitNavigatorError::invalid_range_format(part));
                }

                let start: usize = range_parts[0]
                    .parse()
                    .map_err(|_| GitNavigatorError::invalid_range_number(range_parts[0]))?;
                let end: usize = range_parts[1]
                    .parse()
                    .map_err(|_| GitNavigatorError::invalid_range_number(range_parts[1]))?;

                if start > end {
                    return Err(GitNavigatorError::invalid_range_order(start, end));
                }

                for i in start..=end {
                    indices.insert(i);
                }
            } else {
                // Handle single number
                let num: usize = part
                    .parse()
                    .map_err(|_| GitNavigatorError::invalid_number(part))?;
                indices.insert(num);
            }
        }

        let mut result: Vec<usize> = indices.into_iter().collect();
        result.sort();
        Ok(result)
    }

    pub fn validate(indices: &[usize], max_index: usize) -> Result<()> {
        if max_index == 0 {
            return Err(GitNavigatorError::NoFilesAvailable);
        }

        for &index in indices {
            if index == 0 {
                return Err(GitNavigatorError::ZeroIndex);
            }
            if index > max_index {
                return Err(GitNavigatorError::index_out_of_range(index, max_index));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_single_number() -> Result<()> {
        let result = IndexParser::parse("5")?;
        assert_eq!(result, vec![5]);
        Ok(())
    }

    #[test]
    fn test_parse_multiple_numbers() -> Result<()> {
        let result = IndexParser::parse("1 3 5")?;
        assert_eq!(result, vec![1, 3, 5]);
        Ok(())
    }

    #[test]
    fn test_parse_range() -> Result<()> {
        let result = IndexParser::parse("3-6")?;
        assert_eq!(result, vec![3, 4, 5, 6]);
        Ok(())
    }

    #[test]
    fn test_parse_comma_separated() -> Result<()> {
        let result = IndexParser::parse("1,3,5")?;
        assert_eq!(result, vec![1, 3, 5]);
        Ok(())
    }

    #[test]
    fn test_parse_mixed_format() -> Result<()> {
        let result = IndexParser::parse("1 3-5,8")?;
        assert_eq!(result, vec![1, 3, 4, 5, 8]);
        Ok(())
    }

    #[test]
    fn test_parse_duplicates_removed() -> Result<()> {
        let result = IndexParser::parse("1,1,2,2,3")?;
        assert_eq!(result, vec![1, 2, 3]);
        Ok(())
    }

    #[test]
    fn test_parse_empty_input() -> Result<()> {
        let result = IndexParser::parse("")?;
        assert!(result.is_empty());
        Ok(())
    }

    #[test]
    fn test_parse_whitespace_only() -> Result<()> {
        let result = IndexParser::parse("   ")?;
        assert!(result.is_empty());
        Ok(())
    }

    #[test]
    fn test_parse_invalid_number() {
        let result = IndexParser::parse("abc");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid number: 'abc'"));
    }

    #[test]
    fn test_parse_invalid_range() {
        let result = IndexParser::parse("5-3");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("start (5) must be <= end (3)"));
    }

    #[test]
    fn test_parse_malformed_range() {
        let result = IndexParser::parse("1-2-3");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid range format"));
    }

    #[test]
    fn test_validate_valid_indices() -> Result<()> {
        IndexParser::validate(&[1, 2, 3], 5)?;
        Ok(())
    }

    #[test]
    fn test_validate_index_too_large() {
        let result = IndexParser::validate(&[1, 2, 6], 5);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Index 6 is out of range (1-5 available)"));
    }

    #[test]
    fn test_validate_zero_index() {
        let result = IndexParser::validate(&[0, 1, 2], 5);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Index must be positive (got 0)"));
    }

    #[test]
    fn test_validate_no_files_available() {
        let result = IndexParser::validate(&[1], 0);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("No files available to operate on"));
    }
}
