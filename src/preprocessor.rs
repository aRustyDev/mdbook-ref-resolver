//! Preprocessor implementation.

use crate::resolver::{find_references, ReferenceResolver};
use crate::{Config, Error};
use mdbook::book::{Book, Chapter};
use mdbook::preprocess::{Preprocessor, PreprocessorContext};
use mdbook::BookItem;
use std::path::PathBuf;

/// Reference resolver preprocessor for MDBook.
pub struct RefResolverPreprocessor;

impl RefResolverPreprocessor {
    pub fn new() -> Self {
        Self
    }
}

impl Default for RefResolverPreprocessor {
    fn default() -> Self {
        Self::new()
    }
}

impl Preprocessor for RefResolverPreprocessor {
    fn name(&self) -> &str {
        "ref-resolver"
    }

    fn run(&self, ctx: &PreprocessorContext, mut book: Book) -> mdbook::errors::Result<Book> {
        // Get preprocessor config
        let config = ctx
            .config
            .get_preprocessor(self.name())
            .map(Config::from_table)
            .transpose()
            .map_err(|e| mdbook::errors::Error::msg(e.to_string()))?
            .unwrap_or_default();

        let root = ctx.root.clone();
        let resolver = ReferenceResolver::new(config, root);

        // Process each chapter
        let mut errors: Vec<String> = Vec::new();

        book.for_each_mut(|item| {
            if let BookItem::Chapter(ref mut chapter) = item {
                if let Err(e) = process_chapter(chapter, &resolver) {
                    errors.push(e.to_string());
                }
            }
        });

        if !errors.is_empty() {
            return Err(mdbook::errors::Error::msg(format!(
                "Reference resolution errors:\n{}",
                errors.join("\n")
            )));
        }

        Ok(book)
    }

    fn supports_renderer(&self, renderer: &str) -> bool {
        // Support all renderers - we're just rewriting links
        renderer != "not-supported"
    }
}

/// Process a single chapter, resolving all references.
fn process_chapter(chapter: &mut Chapter, resolver: &ReferenceResolver) -> Result<(), Error> {
    let chapter_path = chapter.path.as_ref().map(PathBuf::from);

    // Find all references in the chapter content
    let references = find_references(&chapter.content);

    if references.is_empty() {
        return Ok(());
    }

    // Process references in reverse order to preserve positions
    let mut content = chapter.content.clone();
    for (start, end, reference) in references.into_iter().rev() {
        match resolver.resolve(&reference, chapter_path.as_deref()) {
            Ok(resolved) => {
                content.replace_range(start..end, &resolved);
            }
            Err(e) => {
                log::warn!(
                    "Failed to resolve reference '{}' in chapter '{}': {}",
                    reference.original,
                    chapter.name,
                    e
                );
                // Optionally, we could continue and just leave the reference as-is
                // For now, return the error
                return Err(e);
            }
        }
    }

    chapter.content = content;
    Ok(())
}

/// Process chapter content, resolving all references (for testing).
pub fn process_content(content: &str, resolver: &ReferenceResolver) -> Result<String, Error> {
    let references = find_references(content);

    if references.is_empty() {
        return Ok(content.to_string());
    }

    let mut result = content.to_string();
    for (start, end, reference) in references.into_iter().rev() {
        match resolver.resolve(&reference, None) {
            Ok(resolved) => {
                result.replace_range(start..end, &resolved);
            }
            Err(e) => return Err(e),
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preprocessor_name() {
        let preprocessor = RefResolverPreprocessor::new();
        assert_eq!(preprocessor.name(), "ref-resolver");
    }

    #[test]
    fn test_supports_html_renderer() {
        let preprocessor = RefResolverPreprocessor::new();
        assert!(preprocessor.supports_renderer("html"));
    }

    #[test]
    fn test_supports_epub_renderer() {
        let preprocessor = RefResolverPreprocessor::new();
        assert!(preprocessor.supports_renderer("epub"));
    }

    #[test]
    fn test_does_not_support_not_supported() {
        let preprocessor = RefResolverPreprocessor::new();
        assert!(!preprocessor.supports_renderer("not-supported"));
    }

    #[test]
    fn test_process_content_with_github_refs() {
        let config = Config {
            github_repo: Some("octocat/foobar".to_string()),
            ..Config::default()
        };

        let resolver = ReferenceResolver::new(config, PathBuf::from("/tmp"));

        let input = "See [Issue #123](gh:::issue/123) for details.";
        let result = process_content(input, &resolver).unwrap();

        assert_eq!(
            result,
            "See [Issue #123](https://github.com/octocat/foobar/issues/123) for details."
        );
    }

    #[test]
    fn test_process_content_no_references() {
        let config = Config::default();
        let resolver = ReferenceResolver::new(config, PathBuf::from("/tmp"));

        let input = "Just regular [markdown](https://example.com) content.";
        let result = process_content(input, &resolver).unwrap();

        assert_eq!(result, input);
    }

    #[test]
    fn test_process_content_multiple_refs() {
        let config = Config {
            github_repo: Some("owner/repo".to_string()),
            ..Config::default()
        };

        let resolver = ReferenceResolver::new(config, PathBuf::from("/tmp"));

        let input = r#"See [Issue](gh:::issue/1) and [PR](gh:::pr/2)."#;
        let result = process_content(input, &resolver).unwrap();

        assert_eq!(
            result,
            "See [Issue](https://github.com/owner/repo/issues/1) and [PR](https://github.com/owner/repo/pull/2)."
        );
    }
}
