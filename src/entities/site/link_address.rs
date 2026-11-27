//! Entity: a link's address in a shell: `#` and the name of a page, percent-encoded.

use derive_more::Display;
/// Always percent-encoded, as [`percent_encoded`] says.
#[derive(Debug, Clone, Default, PartialEq, Eq, Display)]
pub(crate) struct LinkAddress {
    address: String,
}

impl LinkAddress {
    /// The address of the page named `page`, as written, in the shell that links it: `#` and the
    /// name, percent-encoded.
    pub(crate) fn of_page(page: &str) -> LinkAddress {
        LinkAddress {
            address: format!("#{}", percent_encoded(page)),
        }
    }
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
    use super::*;

    #[test]
    fn a_link_s_address_keeps_only_unreserved_bytes_and_slashes() {
        assert_eq!(
            percent_encoded("a b/ç?#%.html"),
            "a%20b/%C3%A7%3F%23%25.html"
        );
    }

    // Rule 235: a page's address is `#` and its name, percent-encoded.
    #[test]
    fn a_page_s_address_is_a_hash_and_its_name() {
        assert_eq!(
            LinkAddress::of_page("billing/a b").to_string(),
            "#billing/a%20b"
        );
    }
}
