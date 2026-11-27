//! Policy: the store's format, which every need of the store shares: its number, where each file of the store goes below the output folder, and how a file is written, as compact JSON, and named, by its SHA-256.

use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::entities::{FilePath, VersionName};

/// The number of the store's format, which every file of the store holds.
pub(crate) const STORE_FORMAT: u32 = 1;

/// The version of the ghg that writes the store.
pub(crate) const GHG_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Where the objects are.
pub(crate) const OBJECTS: &str = "data/objects/";
/// Where the versions' descriptors are.
pub(crate) const VERSIONS: &str = "data/versions/";
pub(crate) const SITE_DESCRIPTOR: &str = "data/site.json";
pub(crate) const VERSIONS_LIST: &str = "data/versions.json";
pub(crate) const PAGES_INDEX: &str = "data/pages.json";
pub(crate) const JSON: &str = ".json";

/// The path of the descriptor of `version`, or of a site without versions.
pub(crate) fn descriptor_path(version: Option<&VersionName>) -> FilePath {
    match version {
        Some(name) => format!("{VERSIONS}{name}{JSON}"),
        None => SITE_DESCRIPTOR.to_string(),
    }
}

/// The path of the object named `name`.
pub(crate) fn object_path(name: &str) -> FilePath {
    format!("{OBJECTS}{name}{JSON}")
}

/// The SHA-256 of `bytes`, in lower-case hexadecimal.
pub(crate) fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes).iter().map(hex_pair).collect()
}

fn hex_pair(byte: &u8) -> String {
    format!("{byte:02x}")
}

/// `data` as compact JSON, as every file of the store is written.
#[expect(
    clippy::expect_used,
    reason = "structs of strings, booleans, whole numbers, lists and options of them always serialise"
)]
pub(crate) fn json_bytes(data: &impl Serialize) -> Vec<u8> {
    serde_json::to_vec(data).expect(
        "structs of strings, booleans, whole numbers, lists and options of them always serialise",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    // Rule 247: only `"`, `\` and the characters below U+0020 are escaped.
    #[test]
    fn text_escapes_only_quotes_backslashes_and_control_characters() {
        let written = json_bytes(&"é \"a\" \\ \u{1}\t\n<&>");
        assert_eq!(written, "\"é \\\"a\\\" \\\\ \\u0001\\t\\n<&>\"".as_bytes());
    }

    #[test]
    fn a_hash_is_lower_case_hexadecimal() {
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
}
