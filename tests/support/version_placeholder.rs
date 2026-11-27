//! Test support: the version of ghg in a page's footer, in the `ghg` member of the store's files and of the marker, and in the folder of its assets, `assets/<ghg>/`, and the placeholder `{version}` the committed expected sites hold in its place, so that a release changes no committed file. The tests in `tests/` and `cargo xtask build` both bring it in with `#[path]`.

/// What the expected sites hold where a page's footer names the version.
const VERSION_PLACEHOLDER: &str = "{version}";

/// `file` with the version in a page's footer, in the `ghg` member of a file written as compact
/// JSON and in every path into the folder of the assets, put back as the placeholder; a file
/// that is not text is left as it is.
pub(crate) fn with_version_placeholder(file: Vec<u8>, version: &str) -> Vec<u8> {
    match String::from_utf8(file) {
        Ok(text) => with_folder_placeholder(&text, version)
            .replace(&footer_end(version), &footer_end(VERSION_PLACEHOLDER))
            .replace(&ghg_member(version), &ghg_member(VERSION_PLACEHOLDER))
            .into_bytes(),
        Err(error) => error.into_bytes(),
    }
}

/// `path` with the folder of the assets of `version` put back as the placeholder's.
pub(crate) fn with_folder_placeholder(path: &str, version: &str) -> String {
    path.replace(&assets_folder(version), &assets_folder(VERSION_PLACEHOLDER))
}

fn assets_folder(version: &str) -> String {
    format!("assets/{version}/")
}

/// How a page's footer ends, after its link to gherkin-glass's repository.
fn footer_end(version: &str) -> String {
    format!("</a> {version}</p>\n</footer>")
}

/// The `ghg` member of a descriptor or of the versions page's settings, as compact JSON.
fn ghg_member(version: &str) -> String {
    format!("\"ghg\":\"{version}\"")
}
