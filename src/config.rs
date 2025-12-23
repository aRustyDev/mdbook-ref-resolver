//! Configuration for the ref-resolver plugin.

use serde::Deserialize;
use std::collections::HashMap;
use toml::value::Table;

/// Plugin configuration from `book.toml`.
///
/// Configured in `book.toml` under `[preprocessor.ref-resolver]`.
///
/// # Example
///
/// ```toml
/// [preprocessor.ref-resolver]
/// github_repo = "owner/repo"
/// gitlab_repo = "owner/repo"
///
/// [preprocessor.ref-resolver.prefixes]
/// adr = "docs/adr/{id}-*.md"
/// docs = "docs/src/{path}.md"
/// ```
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct Config {
    /// GitHub repository in "owner/repo" format for gh: references.
    /// If not set, attempts to detect from git remote.
    pub github_repo: Option<String>,

    /// GitLab repository in "owner/repo" format for gl: references.
    pub gitlab_repo: Option<String>,

    /// Custom prefix patterns. Maps prefix name to path pattern.
    /// Pattern placeholders:
    /// - `{id}` - The reference ID (e.g., "0042" in adr:0042)
    /// - `{path}` - The full reference path (e.g., "user/auth" in docs:user/auth)
    /// - `*` - Glob wildcard for matching files
    pub prefixes: HashMap<String, String>,

    /// Path to UUID index file for uuid: references.
    pub uuid_index: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        let mut prefixes = HashMap::new();
        prefixes.insert("adr".to_string(), "docs/adr/{id}-*.md".to_string());
        prefixes.insert("docs".to_string(), "docs/src/{path}.md".to_string());
        prefixes.insert("blog".to_string(), "docs/blog/*{path}*.md".to_string());
        prefixes.insert("notes".to_string(), "docs/notes/{path}.md".to_string());

        Self {
            github_repo: None,
            gitlab_repo: None,
            prefixes,
            uuid_index: None,
        }
    }
}

impl Config {
    /// Parse configuration from mdbook's preprocessor config table.
    pub fn from_table(table: &Table) -> Result<Self, crate::Error> {
        let value = toml::Value::Table(table.clone());
        value
            .try_into()
            .map_err(|e| crate::Error::Config(format!("Invalid configuration: {}", e)))
    }

    /// Get the pattern for a given prefix.
    pub fn get_pattern(&self, prefix: &str) -> Option<&String> {
        self.prefixes.get(prefix)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.github_repo.is_none());
        assert!(config.prefixes.contains_key("adr"));
        assert!(config.prefixes.contains_key("docs"));
        assert!(config.prefixes.contains_key("blog"));
        assert!(config.prefixes.contains_key("notes"));
    }

    #[test]
    fn test_default_adr_pattern() {
        let config = Config::default();
        assert_eq!(
            config.get_pattern("adr"),
            Some(&"docs/adr/{id}-*.md".to_string())
        );
    }

    #[test]
    fn test_config_from_empty_table() {
        let table = Table::new();
        let config = Config::from_table(&table);
        assert!(config.is_ok());
    }

    #[test]
    fn test_config_with_github_repo() {
        let mut table = Table::new();
        table.insert(
            "github_repo".to_string(),
            toml::Value::String("owner/repo".to_string()),
        );
        let config = Config::from_table(&table).unwrap();
        assert_eq!(config.github_repo, Some("owner/repo".to_string()));
    }

    #[test]
    fn test_config_with_custom_prefixes() {
        let mut table = Table::new();
        let mut prefixes = Table::new();
        prefixes.insert(
            "custom".to_string(),
            toml::Value::String("custom/{path}.md".to_string()),
        );
        table.insert("prefixes".to_string(), toml::Value::Table(prefixes));

        let config = Config::from_table(&table).unwrap();
        assert_eq!(
            config.get_pattern("custom"),
            Some(&"custom/{path}.md".to_string())
        );
    }
}
