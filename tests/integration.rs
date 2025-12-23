//! Integration tests for mdbook-ref-resolver.

use mdbook::preprocess::Preprocessor;
use mdbook_ref_resolver::RefResolverPreprocessor;

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
