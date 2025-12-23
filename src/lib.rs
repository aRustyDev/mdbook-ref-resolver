//! MDBook preprocessor to expand short references to full paths.
//!
//! # Overview
//!
//! This preprocessor expands short reference syntax in markdown links to full paths.
//! It supports multiple reference schemes for different content types.
//!
//! # Supported Reference Schemes
//!
//! | Prefix | Resolves To | Example |
//! |--------|-------------|---------|
//! | `adr:` | `docs/adr/{id}-*.md` | `adr:0042` → `docs/adr/0042-session-timeout.md` |
//! | `docs:` | `docs/src/{path}.md` | `docs:user/auth` → `docs/src/user/auth.md` |
//! | `blog:` | `docs/blog/{slug}.md` | `blog:auth-journey` → `docs/blog/2024-01-15-auth-journey.md` |
//! | `notes:` | `docs/notes/{path}.md` | `notes:analysis/perf` → `docs/notes/analysis/perf.md` |
//! | `uuid:` | Lookup in index | `uuid:20ae...` → resolved path |
//! | `gh:` | GitHub URL | `gh:::issue/123` → `https://github.com/{repo}/issues/123` |
//! | `gl:` | GitLab URL | `gl:::issue/123` → `https://gitlab.com/{repo}/issues/123` |
//!
//! # Configuration
//!
//! Add to your `book.toml`:
//!
//! ```toml
//! [preprocessor.ref-resolver]
//! # GitHub repository for gh: references (auto-detected from git remote)
//! github_repo = "owner/repo"
//!
//! # GitLab repository for gl: references
//! gitlab_repo = "owner/repo"
//!
//! # Custom prefix patterns
//! [preprocessor.ref-resolver.prefixes]
//! adr = "docs/adr/{id}-*.md"
//! docs = "docs/src/{path}.md"
//! ```
//!
//! # Usage
//!
//! ```markdown
//! <!-- Input -->
//! See [ADR-0042](adr:0042) for details.
//! Check the [auth docs](docs:user/auth).
//! Related to [Issue #123](gh:::issue/123).
//!
//! <!-- Output -->
//! See [ADR-0042](../adr/0042-session-timeout.md) for details.
//! Check the [auth docs](../user/auth.md).
//! Related to [Issue #123](https://github.com/owner/repo/issues/123).
//! ```

pub mod config;
pub mod error;
pub mod preprocessor;
pub mod resolver;

pub use config::Config;
pub use error::Error;
pub use preprocessor::RefResolverPreprocessor;
pub use resolver::{Reference, ReferenceResolver};
