//! Entity: a link's address, a path of the site percent-encoded, with a block's anchor after `#` when it links to one.

use derive_more::Display;
/// Always percent-encoded, as [`percent_encoded`] says, except for the anchor.
#[derive(Debug, Clone, Default, PartialEq, Eq, Display)]
pub(crate) struct LinkAddress {
    address: String,
}

impl LinkAddress {
    /// The address of `path`, a path of the site as written, relative to the page that links it.
    pub(crate) fn to(path: &str) -> LinkAddress {
        LinkAddress {
            address: percent_encoded(path),
        }
    }

    /// The address of the block whose anchor is `anchor`, on the page this address names.
    pub(crate) fn at(&self, anchor: &str) -> LinkAddress {
        LinkAddress {
            address: format!("{}#{anchor}", self.address),
        }
    }

    /// This address followed by `inner`: when this one names a folder, written with its last `/`,
    /// and `inner` is relative to that folder, the address of what `inner` names.
    pub(crate) fn then(&self, inner: &LinkAddress) -> LinkAddress {
        LinkAddress {
            address: format!("{}{}", self.address, inner.address),
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
}
