//! Adapter: the reload script's element, added to a page served over HTTP under `--watch`.

use super::request_route::RELOAD_ADDRESS;

/// `page` with the reload script's element just before its last `</body>`,
/// in any case, or at its end when it has none; `build` is the number of
/// the version the page comes from.
pub(super) fn add_reload_script(page: &[u8], build: u64) -> Vec<u8> {
    let element =
        format!(r#"<script type="module" src="{RELOAD_ADDRESS}?build={build}"></script>"#);
    let at = page
        .windows(7)
        .rposition(|window| window.eq_ignore_ascii_case(b"</body>"))
        .unwrap_or(page.len());
    let mut added = Vec::with_capacity(page.len() + element.len());
    let (head, tail) = page.split_at(at);
    added.extend_from_slice(head);
    added.extend_from_slice(element.as_bytes());
    added.extend_from_slice(tail);
    added
}

#[cfg(test)]
mod tests {
    use super::*;

    const ELEMENT: &str = r#"<script type="module" src="/__ghg/reload.js?build=7"></script>"#;

    fn added(page: &str) -> String {
        String::from_utf8(add_reload_script(page.as_bytes(), 7)).expect("UTF-8")
    }

    #[test]
    fn the_element_goes_before_the_last_closing_body() {
        assert_eq!(
            added("<body><pre></body></pre></BODY>\n"),
            format!("<body><pre></body></pre>{ELEMENT}</BODY>\n")
        );
    }

    #[test]
    fn a_page_without_one_has_it_at_its_end() {
        assert_eq!(added("<p>hi"), format!("<p>hi{ELEMENT}"));
        assert_eq!(added(""), ELEMENT);
    }
}
