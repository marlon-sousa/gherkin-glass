//! Policy: the address of a feature file's page, relative to the site's root, and how a link holds it.

use std::path::{Component, Path};

/// A folder's page, in its folder of the site.
pub(crate) const FOLDER_PAGE: &str = "index.html";

/// `path` is relative to the input folder. Its parts are joined by `/`, and
/// its last extension is replaced by `html`, or `.html` added when it has none.
pub(crate) fn page_address(path: &Path) -> String {
    let parts: Vec<String> = path
        .components()
        .filter_map(|part| match part {
            Component::Normal(name) => Some(name.to_string_lossy().into_owned()),
            _ => None,
        })
        .collect();
    let Some((name, folders)) = parts.split_last() else {
        return String::new();
    };
    let page = match name.rfind('.') {
        // A name that starts with its only dot, such as `.feature`, has no extension.
        Some(dot) if dot > 0 => format!("{}.html", &name[..dot]),
        _ => format!("{name}.html"),
    };
    folders
        .iter()
        .map(String::as_str)
        .chain([page.as_str()])
        .collect::<Vec<_>>()
        .join("/")
}

/// The relative address from the page at `address` back to the site's root,
/// such as `../` for a page one folder down, and empty for a page at the root.
pub(crate) fn root_from(address: &str) -> String {
    "../".repeat(address.matches('/').count())
}

/// `path` with every byte but an ASCII letter, a digit, `-`, `.`, `_`, `~`
/// and `/` percent-encoded, as a link's address holds it.
pub(crate) fn percent_encoded(path: &str) -> String {
    path.bytes()
        .map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' | b'/' => {
                char::from(byte).to_string()
            }
            _ => format!("%{byte:02X}"),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn the_last_extension_becomes_html() {
        assert_eq!(page_address(Path::new("refunds.feature")), "refunds.html");
        assert_eq!(
            page_address(&PathBuf::from_iter(["billing", "refunds.feature"])),
            "billing/refunds.html"
        );
        assert_eq!(
            page_address(&PathBuf::from_iter(["stories", "read.story"])),
            "stories/read.html"
        );
        assert_eq!(page_address(Path::new("a.b.feature")), "a.b.html");
    }

    #[test]
    fn a_name_without_an_extension_gets_one() {
        assert_eq!(page_address(Path::new("README")), "README.html");
        assert_eq!(page_address(Path::new(".feature")), ".feature.html");
    }

    #[test]
    fn a_link_s_address_keeps_only_unreserved_bytes_and_slashes() {
        assert_eq!(
            percent_encoded("a b/ç?#%.html"),
            "a%20b/%C3%A7%3F%23%25.html"
        );
    }

    #[test]
    fn the_root_is_one_step_up_for_each_folder() {
        assert_eq!(root_from("a.html"), "");
        assert_eq!(root_from("billing/refunds/a.html"), "../../");
    }
}
