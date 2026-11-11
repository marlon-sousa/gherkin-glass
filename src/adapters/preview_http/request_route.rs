//! Adapter: what a request's path asks ghg serve for over HTTP.

use std::path::{Component, Path};

use crate::entities::ServeMode;

/// The address `/__ghg/build`, answered only under `--watch`.
const BUILD_NUMBER: &str = "/__ghg/build";
/// The address `/__ghg/reload.js`, answered only under `--watch`.
pub(super) const RELOAD_ADDRESS: &str = "/__ghg/reload.js";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Route {
    BuildNumber,
    ReloadScript,
    /// A file, by its `/`-joined address below the site's root. When there
    /// is no such file but a folder of that address, `folder_redirect` is
    /// where to send the browser instead, if anywhere.
    File {
        address: String,
        folder_redirect: Option<String>,
    },
    NotFound,
}

/// `url` is the request's target as the browser sent it, its path and query.
pub(super) fn route(url: &str, mode: ServeMode) -> Route {
    let watching = mode.watches();
    // A browser never sends the fragment, but another client may.
    let url = url.split_once('#').map_or(url, |(url, _)| url);
    let (path, query) = match url.split_once('?') {
        Some((path, query)) => (path, Some(query)),
        None => (url, None),
    };
    if watching && path == BUILD_NUMBER {
        return Route::BuildNumber;
    }
    if watching && path == RELOAD_ADDRESS {
        return Route::ReloadScript;
    }
    let Some(below_root) = path.strip_prefix('/') else {
        return Route::NotFound;
    };
    let (below_root, folder) = match below_root.strip_suffix('/') {
        Some(folder) => (folder, true),
        None => (below_root, below_root.is_empty()),
    };
    let mut parts = Vec::new();
    if !below_root.is_empty() {
        for part in below_root.split('/') {
            match decode(part) {
                Some(part) if is_one_name(&part) => parts.push(part),
                _ => return Route::NotFound,
            }
        }
    }
    if folder {
        parts.push("index.html".to_string());
        return Route::File {
            address: parts.join("/"),
            folder_redirect: None,
        };
    }
    let redirect = match query {
        Some(query) => format!("{path}/?{query}"),
        None => format!("{path}/"),
    };
    let redirect = ascii(&redirect);
    Route::File {
        address: parts.join("/"),
        folder_redirect: Some(redirect),
    }
}

/// `text` with every byte outside printable ASCII percent-encoded, as a
/// header's value must be, whatever the browser sent.
fn ascii(text: &str) -> String {
    let mut encoded = String::with_capacity(text.len());
    for byte in text.bytes() {
        if byte.is_ascii_graphic() {
            encoded.push(char::from(byte));
        } else {
            encoded.push_str(&format!("%{byte:02X}"));
        }
    }
    encoded
}

/// Percent-decoded, as UTF-8; `None` for a broken escape or bytes that are not UTF-8.
fn decode(part: &str) -> Option<String> {
    let bytes = part.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while let Some(&byte) = bytes.get(index) {
        if byte == b'%' {
            let hex = bytes.get(index + 1..index + 3)?;
            let hex = std::str::from_utf8(hex).ok()?;
            decoded.push(u8::from_str_radix(hex, 16).ok()?);
            index += 3;
        } else {
            decoded.push(byte);
            index += 1;
        }
    }
    String::from_utf8(decoded).ok()
}

/// One plain name on this platform: not empty, `.` or `..`, and holding no
/// separator, drive or other prefix, so that joined to the served folder it
/// stays inside it.
fn is_one_name(part: &str) -> bool {
    if part.contains(['/', '\\', '\0']) {
        return false;
    }
    let mut components = Path::new(part).components();
    matches!(
        (components.next(), components.next()),
        (Some(Component::Normal(name)), None) if name == part
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file(address: &str, folder_redirect: Option<&str>) -> Route {
        Route::File {
            address: address.to_string(),
            folder_redirect: folder_redirect.map(str::to_string),
        }
    }

    #[test]
    fn a_path_names_a_file_and_a_slash_its_folder_s_index() {
        assert_eq!(route("/", ServeMode::Build), file("index.html", None));
        assert_eq!(route("/?x=1", ServeMode::Build), file("index.html", None));
        assert_eq!(
            route("/banking/", ServeMode::Build),
            file("banking/index.html", None)
        );
        assert_eq!(
            route("/banking/withdraw.html?q=1#x", ServeMode::Build),
            file("banking/withdraw.html", Some("/banking/withdraw.html/?q=1"))
        );
        assert_eq!(
            route("/banking", ServeMode::Build),
            file("banking", Some("/banking/"))
        );
    }

    #[test]
    fn parts_are_percent_decoded() {
        assert_eq!(
            route("/p%C3%A1gina%20um.html", ServeMode::Build),
            file("página um.html", Some("/p%C3%A1gina%20um.html/"))
        );
        assert_eq!(
            route("/página.html", ServeMode::Build),
            file("página.html", Some("/p%C3%A1gina.html/"))
        );
    }

    #[test]
    fn nothing_outside_the_folder_is_reached() {
        for url in [
            "/../secret",
            "/a/../../b",
            "/%2e%2e/b",
            "/a%2Fb",
            "/a%5Cb",
            "/./a",
            "/a//b",
            "/%zz",
            "/%C3",
            "/%00",
            "relative",
            "*",
        ] {
            assert_eq!(route(url, ServeMode::Build), Route::NotFound, "{url}");
        }
    }

    #[test]
    fn only_watching_answers_the_reload_addresses() {
        assert_eq!(route("/__ghg/build", ServeMode::Watch), Route::BuildNumber);
        assert_eq!(
            route("/__ghg/reload.js?build=3", ServeMode::Watch),
            Route::ReloadScript
        );
        assert_eq!(
            route("/__ghg/build", ServeMode::Build),
            file("__ghg/build", Some("/__ghg/build/"))
        );
    }
}
