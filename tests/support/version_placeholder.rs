//! Test support: the version of ghg in a page's footer, and the placeholder the committed expected sites hold in its place, as docs/BEHAVIOUR.md, rule 195, describes. The tests in `tests/` and `cargo xtask build` both bring it in with `#[path]`.

/// What the expected sites hold where a page's footer names the version.
const VERSION_PLACEHOLDER: &str = "{version}";

/// `page` with the version in its footer put back as the placeholder.
pub(crate) fn with_version_placeholder(page: &str, version: &str) -> String {
    page.replace(&footer_end(version), &footer_end(VERSION_PLACEHOLDER))
}

/// How a page's footer ends, after its link to gherkin-glass's repository.
fn footer_end(version: &str) -> String {
    format!("</a> {version}</p>\n</footer>")
}
