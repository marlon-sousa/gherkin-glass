//! Adapter: the content type a file is served with over HTTP, from its extension, with mime_guess.

use mime_guess::mime::TEXT;

/// `address` is a file's `/`-joined address. Text is said to be UTF-8,
/// as every file ghg writes is; an unknown extension, or none, is bytes.
pub(super) fn content_type(address: &str) -> String {
    let guess = mime_guess::from_path(address).first_or_octet_stream();
    if guess.type_() == TEXT {
        format!("{guess}; charset=utf-8")
    } else {
        guess.to_string()
    }
}

/// Whether a file of this content type has the reload script added under `--watch`.
pub(super) fn is_html(content_type: &str) -> bool {
    content_type.starts_with("text/html")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_extension_a_site_holds_has_its_type() {
        let cases = [
            ("index.html", "text/html; charset=utf-8"),
            ("a/B.HTML", "text/html; charset=utf-8"),
            ("assets/gherkin-glass.js", "text/javascript; charset=utf-8"),
            ("assets/tree.json", "application/json"),
            ("s.css", "text/css; charset=utf-8"),
            ("i.svg", "image/svg+xml"),
            ("f.woff2", "font/woff2"),
            ("a.feature", "text/x-gherkin; charset=utf-8"),
        ];
        for (address, expected) in cases {
            assert_eq!(content_type(address), expected, "{address}");
        }
    }

    #[test]
    fn a_file_without_a_known_extension_is_bytes() {
        for address in ["README", "a.unknown-extension", ".html", "html.d/file"] {
            assert_eq!(
                content_type(address),
                "application/octet-stream",
                "{address}"
            );
        }
    }
}
