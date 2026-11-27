//! Policy: the store's format, which every need of the store shares: its number, where each file of the store, each shell, each map of anchors and the shared assets go below the output folder, and how a file is written, as compact JSON, and named, by its SHA-256.

use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::entities::{BuildTarget, FilePath};

/// The number of the store's format, which every file of the store holds.
pub(crate) const STORE_FORMAT: u32 = 1;

/// The version of the ghg that writes the store.
pub(crate) const GHG_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Where the objects are.
pub(crate) const OBJECTS: &str = "data/objects/";
/// Where the versions' descriptors are.
pub(crate) const VERSIONS: &str = "data/versions/";
/// Where the previews' descriptors are.
pub(crate) const PREVIEWS: &str = "data/previews/";
pub(crate) const SITE_DESCRIPTOR: &str = "data/site.json";
pub(crate) const VERSIONS_LIST: &str = "data/versions.json";
pub(crate) const PAGES_INDEX: &str = "data/pages.json";
pub(crate) const JSON: &str = ".json";

/// The shell of a site without versions, and the versions page, at the output folder's root.
pub(crate) const ROOT_PAGE: &str = "index.html";

/// The folder of the store.
pub(crate) const DATA: &str = "data";

/// The folder of every ghg's assets.
pub(crate) const ASSETS: &str = "assets";

/// The map of anchors, below the folder of its shell.
const ANCHOR_MAP: &str = "assets/anchors.json";

/// The files of a ghg's assets, which the output folder has in `assets/<ghg>/`.
pub(crate) const ASSET_FILES: [&str; 4] = [
    "gherkin-glass.css",
    "gherkin-glass.js",
    "gherkin-glass.svg",
    "THIRD-PARTY-LICENSES.txt",
];

/// The path of the descriptor of `target`.
pub(crate) fn descriptor_path(target: &BuildTarget) -> FilePath {
    match target {
        BuildTarget::Site => SITE_DESCRIPTOR.to_string(),
        BuildTarget::Version(name) => format!("{VERSIONS}{name}{JSON}"),
        BuildTarget::Preview(name) => format!("{PREVIEWS}{name}{JSON}"),
    }
}

/// The path of the shell of `target`.
pub(crate) fn shell_path(target: &BuildTarget) -> FilePath {
    match target.folder() {
        Some(folder) => format!("{folder}/{ROOT_PAGE}"),
        None => ROOT_PAGE.to_string(),
    }
}

/// The path of the map of anchors of `target`, in the folder of its shell.
pub(crate) fn anchor_map_path(target: &BuildTarget) -> FilePath {
    match target.folder() {
        Some(folder) => format!("{folder}/{ANCHOR_MAP}"),
        None => ANCHOR_MAP.to_string(),
    }
}

/// The folder of the assets of the ghg `ghg`, such as `assets/2.0.0`.
pub(crate) fn assets_folder(ghg: &str) -> FilePath {
    format!("{ASSETS}/{ghg}")
}

/// The path of each file of the assets of the ghg `ghg`.
pub(crate) fn asset_paths(ghg: &str) -> impl Iterator<Item = FilePath> + '_ {
    ASSET_FILES
        .iter()
        .map(move |file| format!("{}/{file}", assets_folder(ghg)))
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
