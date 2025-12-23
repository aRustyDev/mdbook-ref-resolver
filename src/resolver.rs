//! Reference resolution logic.

use crate::{Config, Error};
use regex::Regex;
use std::path::{Path, PathBuf};

/// A parsed reference from markdown content.
#[derive(Debug, Clone, PartialEq)]
pub struct Reference {
    /// The original reference string (e.g., "adr:0042")
    pub original: String,
    /// The prefix (e.g., "adr", "docs", "gh")
    pub prefix: String,
    /// The reference value (e.g., "0042", "user/auth", "::issue/123")
    pub value: String,
}

impl Reference {
    /// Parse a reference string into its components.
    pub fn parse(reference: &str) -> Option<Self> {
        let parts: Vec<&str> = reference.splitn(2, ':').collect();
        if parts.len() != 2 {
            return None;
        }

        let prefix = parts[0].to_string();
        let value = parts[1].to_string();

        // Validate prefix is alphanumeric
        if prefix.is_empty() || !prefix.chars().all(|c| c.is_alphanumeric()) {
            return None;
        }

        Some(Self {
            original: reference.to_string(),
            prefix,
            value,
        })
    }
}

/// Resolves references to their full paths.
pub struct ReferenceResolver {
    config: Config,
    root: PathBuf,
}

impl ReferenceResolver {
    /// Create a new resolver with the given configuration.
    pub fn new(config: Config, root: PathBuf) -> Self {
        Self { config, root }
    }

    /// Resolve a reference to its full path or URL.
    pub fn resolve(
        &self,
        reference: &Reference,
        chapter_path: Option<&Path>,
    ) -> Result<String, Error> {
        match reference.prefix.as_str() {
            "gh" => self.resolve_github(&reference.value),
            "gl" => self.resolve_gitlab(&reference.value),
            "uuid" => self.resolve_uuid(&reference.value),
            prefix => self.resolve_file_reference(prefix, &reference.value, chapter_path),
        }
    }

    /// Resolve a GitHub reference.
    /// Format: gh:owner:repo:type/id or gh:::type/id (uses configured repo)
    fn resolve_github(&self, value: &str) -> Result<String, Error> {
        let (owner, repo, path) = self.parse_forge_reference(value, "gh")?;

        let (owner, repo) = if owner.is_empty() && repo.is_empty() {
            // Use configured GitHub repo
            self.config
                .github_repo
                .as_ref()
                .and_then(|r| {
                    let parts: Vec<&str> = r.splitn(2, '/').collect();
                    if parts.len() == 2 {
                        Some((parts[0].to_string(), parts[1].to_string()))
                    } else {
                        None
                    }
                })
                .ok_or_else(|| Error::Resolution {
                    reference: format!("gh:{}", value),
                    reason: "No GitHub repository configured".to_string(),
                })?
        } else if repo.is_empty() {
            // owner provided, repo from config
            self.config
                .github_repo
                .as_ref()
                .and_then(|r| r.split('/').nth(1))
                .map(|r| (owner, r.to_string()))
                .ok_or_else(|| Error::Resolution {
                    reference: format!("gh:{}", value),
                    reason: "No repository specified and none configured".to_string(),
                })?
        } else {
            (owner, repo)
        };

        // Convert path type to GitHub URL format
        let github_path = self.convert_forge_path(&path, "github");
        Ok(format!(
            "https://github.com/{}/{}/{}",
            owner, repo, github_path
        ))
    }

    /// Resolve a GitLab reference.
    fn resolve_gitlab(&self, value: &str) -> Result<String, Error> {
        let (owner, repo, path) = self.parse_forge_reference(value, "gl")?;

        let (owner, repo) = if owner.is_empty() && repo.is_empty() {
            self.config
                .gitlab_repo
                .as_ref()
                .and_then(|r| {
                    let parts: Vec<&str> = r.splitn(2, '/').collect();
                    if parts.len() == 2 {
                        Some((parts[0].to_string(), parts[1].to_string()))
                    } else {
                        None
                    }
                })
                .ok_or_else(|| Error::Resolution {
                    reference: format!("gl:{}", value),
                    reason: "No GitLab repository configured".to_string(),
                })?
        } else if repo.is_empty() {
            self.config
                .gitlab_repo
                .as_ref()
                .and_then(|r| r.split('/').nth(1))
                .map(|r| (owner, r.to_string()))
                .ok_or_else(|| Error::Resolution {
                    reference: format!("gl:{}", value),
                    reason: "No repository specified and none configured".to_string(),
                })?
        } else {
            (owner, repo)
        };

        let gitlab_path = self.convert_forge_path(&path, "gitlab");
        Ok(format!(
            "https://gitlab.com/{}/{}/{}",
            owner, repo, gitlab_path
        ))
    }

    /// Parse a forge reference in the format owner:repo:path
    /// Supports: ::path (both empty), :repo:path (owner empty), owner::path (repo empty)
    fn parse_forge_reference(
        &self,
        value: &str,
        prefix: &str,
    ) -> Result<(String, String, String), Error> {
        let parts: Vec<&str> = value.splitn(3, ':').collect();

        match parts.len() {
            1 => {
                // Just path, no colons - invalid for forge refs
                Err(Error::InvalidSyntax {
                    reference: format!("{}:{}", prefix, value),
                    reason: "Forge references must use format owner:repo:path or ::path"
                        .to_string(),
                })
            }
            2 => {
                // owner:path or :path - assume empty repo
                Ok((parts[0].to_string(), String::new(), parts[1].to_string()))
            }
            3 => {
                // Full format: owner:repo:path
                Ok((
                    parts[0].to_string(),
                    parts[1].to_string(),
                    parts[2].to_string(),
                ))
            }
            _ => unreachable!(),
        }
    }

    /// Convert forge path shorthand to full URL path.
    fn convert_forge_path(&self, path: &str, platform: &str) -> String {
        // Handle common shortcuts
        if let Some(stripped) = path.strip_prefix("issue/") {
            return format!("issues/{}", stripped);
        }
        if let Some(stripped) = path.strip_prefix("pr/") {
            if platform == "github" {
                return format!("pull/{}", stripped);
            } else {
                return format!("-/merge_requests/{}", stripped);
            }
        }
        if let Some(stripped) = path.strip_prefix("mr/") {
            if platform == "gitlab" {
                return format!("-/merge_requests/{}", stripped);
            } else {
                return format!("pull/{}", stripped);
            }
        }
        if let Some(stripped) = path.strip_prefix("commit/") {
            if platform == "gitlab" {
                return format!("-/commit/{}", stripped);
            }
            return format!("commit/{}", stripped);
        }
        if let Some(stripped) = path.strip_prefix("blob/") {
            if platform == "gitlab" {
                return format!("-/blob/{}", stripped);
            }
            return format!("blob/{}", stripped);
        }

        // Return path as-is if no shorthand matched
        path.to_string()
    }

    /// Resolve a UUID reference (lookup in index file).
    fn resolve_uuid(&self, _value: &str) -> Result<String, Error> {
        // TODO: Implement UUID index lookup
        Err(Error::Resolution {
            reference: format!("uuid:{}", _value),
            reason: "UUID resolution not yet implemented".to_string(),
        })
    }

    /// Resolve a file-based reference using glob patterns.
    fn resolve_file_reference(
        &self,
        prefix: &str,
        value: &str,
        chapter_path: Option<&Path>,
    ) -> Result<String, Error> {
        let pattern = self
            .config
            .get_pattern(prefix)
            .ok_or_else(|| Error::UnknownPrefix {
                prefix: prefix.to_string(),
                reference: format!("{}:{}", prefix, value),
            })?;

        // Build the glob pattern by replacing placeholders
        let glob_pattern = pattern.replace("{id}", value).replace("{path}", value);

        let full_pattern = self.root.join(&glob_pattern);
        let pattern_str = full_pattern.to_string_lossy();

        // Find matching files
        let matches: Vec<PathBuf> = glob::glob(&pattern_str)
            .map_err(Error::from)?
            .filter_map(Result::ok)
            .collect();

        match matches.len() {
            0 => Err(Error::FileNotFound {
                pattern: glob_pattern,
                reference: format!("{}:{}", prefix, value),
            }),
            1 => {
                let target = &matches[0];
                // Calculate relative path from chapter to target
                if let Some(chapter) = chapter_path {
                    let chapter_dir = chapter.parent().unwrap_or(Path::new(""));
                    let relative = self.relative_path(chapter_dir, target);
                    Ok(relative)
                } else {
                    // No chapter path, return path relative to root
                    Ok(target
                        .strip_prefix(&self.root)
                        .unwrap_or(target)
                        .to_string_lossy()
                        .to_string())
                }
            }
            _ => Err(Error::AmbiguousReference {
                reference: format!("{}:{}", prefix, value),
                files: matches
                    .iter()
                    .map(|p| p.to_string_lossy().to_string())
                    .collect(),
            }),
        }
    }

    /// Calculate relative path from source to target.
    fn relative_path(&self, from: &Path, to: &Path) -> String {
        // Normalize paths relative to root
        let from_rel = from.strip_prefix(&self.root).unwrap_or(from);
        let to_rel = to.strip_prefix(&self.root).unwrap_or(to);

        let from_components: Vec<_> = from_rel.components().collect();
        let to_components: Vec<_> = to_rel.components().collect();

        // Find common prefix length
        let common_len = from_components
            .iter()
            .zip(to_components.iter())
            .take_while(|(a, b)| a == b)
            .count();

        // Build relative path
        let mut result = PathBuf::new();

        // Add ".." for each remaining component in 'from'
        for _ in common_len..from_components.len() {
            result.push("..");
        }

        // Add remaining components from 'to'
        for component in to_components.iter().skip(common_len) {
            result.push(component);
        }

        result.to_string_lossy().to_string()
    }
}

/// Find all references in markdown content.
pub fn find_references(content: &str) -> Vec<(usize, usize, Reference)> {
    // Match markdown links with reference syntax: [text](prefix:value)
    let re = Regex::new(r"\]\(([a-zA-Z]+:[^)]+)\)").unwrap();

    re.captures_iter(content)
        .filter_map(|cap| {
            let ref_str = cap.get(1)?.as_str();

            // Skip URLs (they have :// after the scheme)
            if ref_str.contains("://") {
                return None;
            }

            let reference = Reference::parse(ref_str)?;

            // Return the position of just the reference part (inside parentheses)
            let ref_match = cap.get(1)?;
            Some((ref_match.start(), ref_match.end(), reference))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_adr_reference() {
        let ref_ = Reference::parse("adr:0042").unwrap();
        assert_eq!(ref_.prefix, "adr");
        assert_eq!(ref_.value, "0042");
    }

    #[test]
    fn test_parse_docs_reference() {
        let ref_ = Reference::parse("docs:user/auth").unwrap();
        assert_eq!(ref_.prefix, "docs");
        assert_eq!(ref_.value, "user/auth");
    }

    #[test]
    fn test_parse_github_reference() {
        let ref_ = Reference::parse("gh:::issue/123").unwrap();
        assert_eq!(ref_.prefix, "gh");
        assert_eq!(ref_.value, "::issue/123");
    }

    #[test]
    fn test_parse_invalid_reference() {
        assert!(Reference::parse("noprefix").is_none());
        assert!(Reference::parse(":noprefix").is_none());
    }

    #[test]
    fn test_find_references_in_content() {
        let content = r#"
See [ADR-0042](adr:0042) for details.
Check the [auth docs](docs:user/auth).
Related to [Issue #123](gh:::issue/123).
Regular [link](https://example.com) should not match.
"#;
        let refs = find_references(content);
        assert_eq!(refs.len(), 3);
        assert_eq!(refs[0].2.prefix, "adr");
        assert_eq!(refs[1].2.prefix, "docs");
        assert_eq!(refs[2].2.prefix, "gh");
    }

    #[test]
    fn test_github_url_resolution() {
        let config = Config {
            github_repo: Some("octocat/foobar".to_string()),
            ..Config::default()
        };

        let resolver = ReferenceResolver::new(config, PathBuf::from("/tmp"));

        let ref_ = Reference::parse("gh:::issue/123").unwrap();
        let result = resolver.resolve(&ref_, None).unwrap();
        assert_eq!(result, "https://github.com/octocat/foobar/issues/123");
    }

    #[test]
    fn test_github_pr_resolution() {
        let config = Config {
            github_repo: Some("octocat/foobar".to_string()),
            ..Config::default()
        };

        let resolver = ReferenceResolver::new(config, PathBuf::from("/tmp"));

        let ref_ = Reference::parse("gh:::pr/456").unwrap();
        let result = resolver.resolve(&ref_, None).unwrap();
        assert_eq!(result, "https://github.com/octocat/foobar/pull/456");
    }

    #[test]
    fn test_github_with_explicit_repo() {
        let config = Config::default();
        let resolver = ReferenceResolver::new(config, PathBuf::from("/tmp"));

        let ref_ = Reference::parse("gh:owner:repo:issue/789").unwrap();
        let result = resolver.resolve(&ref_, None).unwrap();
        assert_eq!(result, "https://github.com/owner/repo/issues/789");
    }

    #[test]
    fn test_relative_path_calculation() {
        let config = Config::default();
        let resolver = ReferenceResolver::new(config, PathBuf::from("/project"));

        // From docs/src/chapter1.md to docs/adr/0042-test.md
        let from = Path::new("docs/src");
        let to = Path::new("/project/docs/adr/0042-test.md");

        let result = resolver.relative_path(from, to);
        assert_eq!(result, "../adr/0042-test.md");
    }
}
