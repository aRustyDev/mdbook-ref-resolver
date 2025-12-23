# mdbook-ref-resolver

MDBook preprocessor to expand short references to full paths.

## Overview

This preprocessor expands short reference syntax in markdown links to full paths or URLs. It supports multiple reference schemes for different content types.

## Supported Reference Schemes

| Prefix | Resolves To | Example |
|--------|-------------|---------|
| `adr:` | `docs/adr/{id}-*.md` | `adr:0042` → `docs/adr/0042-session-timeout.md` |
| `docs:` | `docs/src/{path}.md` | `docs:user/auth` → `docs/src/user/auth.md` |
| `blog:` | `docs/blog/{slug}.md` | `blog:auth-journey` → `docs/blog/2024-01-15-auth-journey.md` |
| `notes:` | `docs/notes/{path}.md` | `notes:analysis/perf` → `docs/notes/analysis/perf.md` |
| `uuid:` | Lookup in index | `uuid:20ae...` → resolved path |
| `gh:` | GitHub URL | `gh:::issue/123` → `https://github.com/{repo}/issues/123` |
| `gl:` | GitLab URL | `gl:::issue/123` → `https://gitlab.com/{repo}/issues/123` |

## Installation

```bash
cargo install mdbook-ref-resolver
```

## Configuration

Add to your `book.toml`:

```toml
[preprocessor.ref-resolver]
# GitHub repository for gh: references (auto-detected from git remote)
github_repo = "owner/repo"

# GitLab repository for gl: references
gitlab_repo = "owner/repo"

# Custom prefix patterns (optional - these are the defaults)
[preprocessor.ref-resolver.prefixes]
adr = "docs/adr/{id}-*.md"
docs = "docs/src/{path}.md"
blog = "docs/blog/*{path}*.md"
notes = "docs/notes/{path}.md"
```

## Usage

### File References

```markdown
<!-- Input -->
See [ADR-0042](adr:0042) for details.
Check the [auth docs](docs:user/auth).

<!-- Output -->
See [ADR-0042](../adr/0042-session-timeout.md) for details.
Check the [auth docs](../user/auth.md).
```

### GitHub/GitLab References

The forge reference format is `gh:owner:repo:type/id`. Empty segments use configured defaults:

```markdown
<!-- These are equivalent if github_repo = "octocat/foobar" -->
[Issue #123](gh:::issue/123)
[Issue #123](gh::foobar:issue/123)
[Issue #123](gh:octocat::issue/123)
[Issue #123](gh:octocat:foobar:issue/123)

<!-- All resolve to -->
[Issue #123](https://github.com/octocat/foobar/issues/123)
```

Supported shortcuts:
- `issue/N` → `issues/N`
- `pr/N` → `pull/N` (GitHub) or `-/merge_requests/N` (GitLab)
- `mr/N` → `-/merge_requests/N` (GitLab) or `pull/N` (GitHub)
- `commit/SHA` → `commit/SHA` or `-/commit/SHA`
- `blob/path` → `blob/path` or `-/blob/path`

## Custom Prefixes

You can define custom reference prefixes:

```toml
[preprocessor.ref-resolver.prefixes]
# RFC documents
rfc = "docs/rfcs/RFC-{id}.md"

# API documentation
api = "docs/api/{path}.md"

# Meeting notes with date pattern
meeting = "docs/meetings/*{id}*.md"
```

Pattern placeholders:
- `{id}` - The reference value (e.g., "0042" from `adr:0042`)
- `{path}` - The reference value (e.g., "user/auth" from `docs:user/auth`)
- `*` - Glob wildcard for file matching

## Troubleshooting

### Reference not resolving

Enable debug logging:

```bash
RUST_LOG=debug mdbook build
```

### Multiple files match pattern

If a glob pattern matches multiple files, you'll get an error. Make your patterns more specific or use more precise references.

### Unknown prefix

Add the prefix to your configuration:

```toml
[preprocessor.ref-resolver.prefixes]
myprefix = "path/to/{path}.md"
```

## License

GPL-3.0-or-later
