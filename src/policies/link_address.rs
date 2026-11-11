//! Policy: how a link's address holds a path of the site.

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
    use super::*;

    #[test]
    fn a_link_s_address_keeps_only_unreserved_bytes_and_slashes() {
        assert_eq!(
            percent_encoded("a b/ç?#%.html"),
            "a%20b/%C3%A7%3F%23%25.html"
        );
    }
}
