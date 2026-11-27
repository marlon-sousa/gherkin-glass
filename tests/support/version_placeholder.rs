//! Test support: the version of ghg in a page's footer and in the `ghg` member of the store's files, and the placeholder `{version}` the committed expected sites hold in its place, so that a release changes no committed page. The tests in `tests/` and `cargo xtask build` both bring it in with `#[path]`.

/// What the expected sites hold where a page's footer names the version.
const VERSION_PLACEHOLDER: &str = "{version}";

/// `file` with the version in a page's footer, and in the `ghg` member of a file of the store
/// written as compact JSON, put back as the placeholder; a file that is not text is left as it
/// is.
pub(crate) fn with_version_placeholder(file: Vec<u8>, version: &str) -> Vec<u8> {
    match String::from_utf8(file) {
        Ok(text) => text
            .replace(&footer_end(version), &footer_end(VERSION_PLACEHOLDER))
            .replace(&ghg_member(version), &ghg_member(VERSION_PLACEHOLDER))
            .into_bytes(),
        Err(error) => error.into_bytes(),
    }
}

/// How a page's footer ends, after its link to gherkin-glass's repository.
fn footer_end(version: &str) -> String {
    format!("</a> {version}</p>\n</footer>")
}

/// The `ghg` member of a descriptor or of the versions page's settings, as compact JSON.
fn ghg_member(version: &str) -> String {
    format!("\"ghg\":\"{version}\"")
}
