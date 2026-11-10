//! Policy: the content type a file is served with, from its extension.

/// `address` is a file's `/`-joined address; its extension is compared without regard to case.
pub(crate) fn content_type(address: &str) -> &'static str {
    let name = address.rsplit('/').next().unwrap_or(address);
    let extension = match name.rsplit_once('.') {
        Some((stem, extension)) if !stem.is_empty() => extension.to_ascii_lowercase(),
        _ => return "application/octet-stream",
    };
    match extension.as_str() {
        "html" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" | "mjs" => "text/javascript; charset=utf-8",
        "json" | "map" => "application/json",
        "txt" => "text/plain; charset=utf-8",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "ico" => "image/x-icon",
        "woff2" => "font/woff2",
        "woff" => "font/woff",
        _ => "application/octet-stream",
    }
}

/// Whether a file of this content type has the reload script added under `--watch`.
pub(crate) fn is_html(content_type: &str) -> bool {
    content_type.starts_with("text/html")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_extension_has_its_type() {
        assert_eq!(content_type("index.html"), "text/html; charset=utf-8");
        assert_eq!(content_type("a/B.HTML"), "text/html; charset=utf-8");
        assert_eq!(
            content_type("assets/gherkin-glass.js"),
            "text/javascript; charset=utf-8"
        );
        assert_eq!(content_type("m.mjs"), "text/javascript; charset=utf-8");
        assert_eq!(content_type("assets/tree.json"), "application/json");
        assert_eq!(content_type("s.css"), "text/css; charset=utf-8");
        assert_eq!(content_type("i.svg"), "image/svg+xml");
        assert_eq!(content_type("f.woff2"), "font/woff2");
    }

    #[test]
    fn a_file_without_a_known_extension_is_bytes() {
        assert_eq!(content_type("README"), "application/octet-stream");
        assert_eq!(content_type("a.feature"), "application/octet-stream");
        assert_eq!(content_type(".html"), "application/octet-stream");
        assert_eq!(content_type("html.d/file"), "application/octet-stream");
    }
}
