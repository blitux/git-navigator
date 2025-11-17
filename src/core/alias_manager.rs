//! Alias management for git-navigator shell integration.
//!
//! This module provides comprehensive shell alias management including:
//! - Shell detection (bash, zsh, fish, sh)
//! - Alias registry with canonical definitions
//! - Template system for different shell syntaxes
//! - Config file parsing and updating
//! - Diff generation for changes

use crate::core::error::{GitNavigatorError, Result};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

/// Supported shell types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
    Sh,
}

impl Shell {
    /// Detect current shell from environment
    pub fn detect() -> Result<Self> {
        let shell = env::var("SHELL").unwrap_or_default();

        if shell.contains("bash") {
            Ok(Shell::Bash)
        } else if shell.contains("zsh") {
            Ok(Shell::Zsh)
        } else if shell.contains("fish") {
            Ok(Shell::Fish)
        } else if shell.contains("sh") {
            Ok(Shell::Sh)
        } else {
            Err(GitNavigatorError::config_error(format!(
                "Unsupported shell: {shell}"
            )))
        }
    }

    /// Get the config file path for this shell
    pub fn config_path(&self) -> Result<PathBuf> {
        let home = env::var("HOME").map_err(|_| {
            GitNavigatorError::config_error("HOME environment variable not set")
        })?;

        let config_file = match self {
            Shell::Bash => {
                let bash_profile = Path::new(&home).join(".bash_profile");
                let bashrc = Path::new(&home).join(".bashrc");

                // On macOS, prefer .bash_profile if it exists
                if cfg!(target_os = "macos") && bash_profile.exists() {
                    bash_profile
                } else {
                    bashrc
                }
            }
            Shell::Zsh => Path::new(&home).join(".zshrc"),
            Shell::Fish => Path::new(&home).join(".config/fish/config.fish"),
            Shell::Sh => Path::new(&home).join(".profile"),
        };

        Ok(config_file)
    }

    /// Get the name of the shell
    pub fn name(&self) -> &str {
        match self {
            Shell::Bash => "bash",
            Shell::Zsh => "zsh",
            Shell::Fish => "fish",
            Shell::Sh => "sh",
        }
    }

    /// Format an alias for this shell
    pub fn format_alias(&self, name: &str, command: &str) -> String {
        match self {
            Shell::Fish => format!("alias {name} \"{command}\""),
            _ => format!("alias {name}=\"{command}\""),
        }
    }

    /// Parse an alias line for this shell
    pub fn parse_alias(&self, line: &str) -> Option<(String, String)> {
        let line = line.trim();

        if !line.starts_with("alias ") {
            return None;
        }

        let rest = line.strip_prefix("alias ")?.trim();

        match self {
            Shell::Fish => {
                // Fish: alias name "command"
                let parts: Vec<&str> = rest.splitn(2, ' ').collect();
                if parts.len() == 2 {
                    let name = parts[0].to_string();
                    let command = parts[1].trim_matches('"').to_string();
                    Some((name, command))
                } else {
                    None
                }
            }
            _ => {
                // Bash/Zsh/Sh: alias name="command"
                let parts: Vec<&str> = rest.splitn(2, '=').collect();
                if parts.len() == 2 {
                    let name = parts[0].to_string();
                    let command = parts[1].trim_matches('"').trim_matches('\'').to_string();
                    Some((name, command))
                } else {
                    None
                }
            }
        }
    }
}

/// Definition of a single alias
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AliasDefinition {
    pub name: String,
    pub command: String,
    pub description: String,
}

/// Status of an alias in the current configuration
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AliasStatus {
    Installed,
    Missing,
    Outdated { current: String },
}

/// A comparison result for a single alias
#[derive(Debug, Clone)]
pub struct AliasComparison {
    pub alias: AliasDefinition,
    pub status: AliasStatus,
}

/// Marker comment for git-navigator alias block
pub const ALIAS_MARKER: &str = "# Git Navigator aliases";

/// Registry of all git-navigator aliases
pub struct AliasRegistry;

impl AliasRegistry {
    /// Get all canonical alias definitions
    pub fn definitions() -> Vec<AliasDefinition> {
        vec![
            AliasDefinition {
                name: "gs".to_string(),
                command: "git-navigator status".to_string(),
                description: "Show numbered git status".to_string(),
            },
            AliasDefinition {
                name: "ga".to_string(),
                command: "git-navigator add".to_string(),
                description: "Add files by index".to_string(),
            },
            AliasDefinition {
                name: "gd".to_string(),
                command: "git-navigator diff".to_string(),
                description: "Show diff by index".to_string(),
            },
            AliasDefinition {
                name: "grs".to_string(),
                command: "git-navigator reset".to_string(),
                description: "Reset files by index".to_string(),
            },
            AliasDefinition {
                name: "gco".to_string(),
                command: "git-navigator checkout".to_string(),
                description: "Checkout files by index".to_string(),
            },
            AliasDefinition {
                name: "gb".to_string(),
                command: "git-navigator branches".to_string(),
                description: "Show numbered branches".to_string(),
            },
            AliasDefinition {
                name: "cp".to_string(),
                command: "git-navigator copy".to_string(),
                description: "Copy files by index".to_string(),
            },
            AliasDefinition {
                name: "rm".to_string(),
                command: "git-navigator remove".to_string(),
                description: "Remove files by index".to_string(),
            },
            AliasDefinition {
                name: "gcb".to_string(),
                command: "git-navigator checkout-branch".to_string(),
                description: "Checkout branch by index".to_string(),
            },
            AliasDefinition {
                name: "gl".to_string(),
                command: "git log --graph --pretty=format:'%Cred%h%Creset -%C(yellow)%d%Creset %s %Cgreen(%cr) %C(bold blue)<%an>%Creset' --abbrev-commit".to_string(),
                description: "Visual git log".to_string(),
            },
        ]
    }

    /// Get alias definitions as a HashMap for quick lookup
    pub fn definitions_map() -> HashMap<String, AliasDefinition> {
        Self::definitions()
            .into_iter()
            .map(|def| (def.name.clone(), def))
            .collect()
    }
}

/// Manages shell alias operations
pub struct AliasManager {
    shell: Shell,
    config_path: PathBuf,
}

impl AliasManager {
    /// Create a new alias manager for the detected shell
    pub fn new() -> Result<Self> {
        let shell = Shell::detect()?;
        let config_path = shell.config_path()?;

        Ok(Self { shell, config_path })
    }

    /// Create a new alias manager for a specific shell and config path
    pub fn with_config(shell: Shell, config_path: PathBuf) -> Self {
        Self { shell, config_path }
    }

    /// Get the current shell
    pub fn shell(&self) -> Shell {
        self.shell
    }

    /// Get the config file path
    pub fn config_path(&self) -> &Path {
        &self.config_path
    }

    /// Check if a line is a git-navigator alias marker (supports old formats)
    fn is_alias_marker(line: &str) -> bool {
        let line = line.trim();
        line == ALIAS_MARKER
            || line.starts_with("# Git Navigator aliases -")
            || line == "# Git Navigator aliases - Cleaner, faster, leaner than SCM Breeze"
            || line == "# Git Navigator aliases - Clean, lean and fast git productivity tool"
    }

    /// Read current aliases from config file
    pub fn read_aliases(&self) -> Result<HashMap<String, String>> {
        if !self.config_path.exists() {
            return Ok(HashMap::new());
        }

        let content = fs::read_to_string(&self.config_path)?;
        let mut aliases = HashMap::new();
        let mut in_block = false;

        for line in content.lines() {
            if Self::is_alias_marker(line) {
                in_block = true;
                continue;
            }

            if in_block {
                // Check if we've left the alias block
                if line.trim().is_empty() || (!line.trim().starts_with("alias ") && !line.trim().starts_with('#')) {
                    break;
                }

                if let Some((name, command)) = self.shell.parse_alias(line) {
                    aliases.insert(name, command);
                }
            }
        }

        Ok(aliases)
    }

    /// Compare current aliases with canonical definitions
    pub fn compare_aliases(&self) -> Result<Vec<AliasComparison>> {
        let current = self.read_aliases()?;
        let canonical = AliasRegistry::definitions_map();

        let mut comparisons = Vec::new();

        for (name, definition) in canonical {
            let status = match current.get(&name) {
                Some(current_cmd) if current_cmd == &definition.command => {
                    AliasStatus::Installed
                }
                Some(current_cmd) => AliasStatus::Outdated {
                    current: current_cmd.clone(),
                },
                None => AliasStatus::Missing,
            };

            comparisons.push(AliasComparison {
                alias: definition,
                status,
            });
        }

        // Sort by alias name for consistent output
        comparisons.sort_by(|a, b| a.alias.name.cmp(&b.alias.name));

        Ok(comparisons)
    }

    /// Update aliases in config file
    pub fn update_aliases(&self) -> Result<Vec<AliasComparison>> {
        // Get comparison before update
        let comparisons = self.compare_aliases()?;

        // Read existing config
        let content = if self.config_path.exists() {
            fs::read_to_string(&self.config_path)?
        } else {
            // Create parent directory if needed
            if let Some(parent) = self.config_path.parent() {
                fs::create_dir_all(parent)?;
            }
            String::new()
        };

        // Build new alias block
        let mut alias_lines = vec![ALIAS_MARKER.to_string()];
        for def in AliasRegistry::definitions() {
            alias_lines.push(self.shell.format_alias(&def.name, &def.command));
        }

        // Replace or append alias block
        let new_content = if content.contains(ALIAS_MARKER) {
            self.replace_alias_block(&content, &alias_lines)
        } else {
            self.append_alias_block(&content, &alias_lines)
        };

        // Write back to file
        fs::write(&self.config_path, new_content)?;

        Ok(comparisons)
    }

    /// Replace existing alias block in content (handles old marker formats)
    fn replace_alias_block(&self, content: &str, new_lines: &[String]) -> String {
        let mut result = Vec::new();
        let mut in_block = false;
        let mut block_replaced = false;

        for line in content.lines() {
            // Check for any old or new marker format
            if Self::is_alias_marker(line) {
                if !block_replaced {
                    // Insert new alias block with new marker
                    for alias_line in new_lines {
                        result.push(alias_line.clone());
                    }
                    block_replaced = true;
                    in_block = true;
                }
                continue;
            }

            if in_block {
                // Skip old alias lines until we hit a non-alias line
                if line.trim().starts_with("alias ") || line.trim().is_empty() {
                    if line.trim().is_empty() {
                        in_block = false;
                    }
                    continue;
                } else {
                    in_block = false;
                }
            }

            result.push(line.to_string());
        }

        result.join("\n") + "\n"
    }

    /// Append alias block to content
    fn append_alias_block(&self, content: &str, new_lines: &[String]) -> String {
        let mut result = content.to_string();

        // Ensure there's a blank line before the block
        if !result.is_empty() && !result.ends_with('\n') {
            result.push('\n');
        }
        if !result.is_empty() {
            result.push('\n');
        }

        // Add alias block
        for line in new_lines {
            result.push_str(line);
            result.push('\n');
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_format_alias_bash() {
        let shell = Shell::Bash;
        assert_eq!(
            shell.format_alias("gs", "git-navigator status"),
            "alias gs=\"git-navigator status\""
        );
    }

    #[test]
    fn test_shell_format_alias_fish() {
        let shell = Shell::Fish;
        assert_eq!(
            shell.format_alias("gs", "git-navigator status"),
            "alias gs \"git-navigator status\""
        );
    }

    #[test]
    fn test_shell_parse_alias_bash() {
        let shell = Shell::Bash;
        assert_eq!(
            shell.parse_alias("alias gs=\"git-navigator status\""),
            Some(("gs".to_string(), "git-navigator status".to_string()))
        );
    }

    #[test]
    fn test_shell_parse_alias_fish() {
        let shell = Shell::Fish;
        assert_eq!(
            shell.parse_alias("alias gs \"git-navigator status\""),
            Some(("gs".to_string(), "git-navigator status".to_string()))
        );
    }

    #[test]
    fn test_alias_registry_count() {
        let defs = AliasRegistry::definitions();
        assert_eq!(defs.len(), 10);
    }

    #[test]
    fn test_alias_registry_map() {
        let map = AliasRegistry::definitions_map();
        assert!(map.contains_key("gs"));
        assert!(map.contains_key("cp"));
        assert!(map.contains_key("rm"));
    }

    #[test]
    fn test_replace_alias_block() {
        let shell = Shell::Bash;
        let manager = AliasManager::with_config(shell, PathBuf::from("/tmp/test"));

        let content = r#"# User config
export PATH=/custom:$PATH

# Git Navigator aliases
alias gs="git-navigator status"
alias ga="git-navigator add"

# More config
alias ll="ls -la"
"#;

        let new_lines = vec![
            "# Git Navigator aliases".to_string(),
            "alias gs=\"git-navigator status\"".to_string(),
            "alias cp=\"git-navigator copy\"".to_string(),
        ];

        let result = manager.replace_alias_block(content, &new_lines);

        assert!(result.contains("# Git Navigator aliases"));
        assert!(result.contains("alias cp=\"git-navigator copy\""));
        assert!(result.contains("alias ll=\"ls -la\""));
        assert_eq!(result.matches("# Git Navigator aliases").count(), 1);
    }
}
