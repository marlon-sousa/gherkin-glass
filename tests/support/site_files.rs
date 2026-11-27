//! Test support: the files of a site written by a test, compared with a committed expected site. The tests in `tests/` bring it in with `#[path]`, beside `version_placeholder.rs` and `store_json.rs`.

use std::collections::BTreeMap;
use std::fs::{read, read_dir};
use std::path::Path;

use sha2::{Digest, Sha256};

use super::store_json::indented;
use super::version_placeholder::{with_folder_placeholder, with_version_placeholder};

/// Every file below `folder`, by its `/`-joined path in it, the version it and its path name
/// put back as the placeholder, and each file of the store indented.
pub(crate) fn files(folder: &Path) -> BTreeMap<String, Vec<u8>> {
    let mut found = BTreeMap::new();
    let mut folders = vec![(folder.to_path_buf(), String::new())];
    while let Some((path, prefix)) = folders.pop() {
        for entry in read_dir(&path).expect("the folder is listed") {
            let entry = entry.expect("the folder is listed");
            let name = format!("{prefix}{}", entry.file_name().to_string_lossy());
            if entry.path().is_dir() {
                folders.push((entry.path(), format!("{name}/")));
            } else {
                let file = read(entry.path()).expect("the file is read");
                let file = with_version_placeholder(file, env!("CARGO_PKG_VERSION"));
                let name = with_folder_placeholder(&name, env!("CARGO_PKG_VERSION"));
                let file = if is_store_file(&name) {
                    indented(&String::from_utf8(file).expect("UTF-8")).into_bytes()
                } else {
                    file
                };
                found.insert(name, file);
            }
        }
    }
    found
}

/// Checks that `written` holds the files of the committed site `expected`, and only those, and
/// that each object `written` holds is named by the SHA-256 of its bytes.
pub(crate) fn assert_same_site(written: &Path, expected: &Path) {
    assert_same_store(written, expected);
    let expected = files(expected);
    let written = files(written);
    assert_eq!(
        written.keys().collect::<Vec<_>>(),
        expected.keys().collect::<Vec<_>>(),
        "the files written"
    );
    for (name, contents) in &expected {
        assert!(
            written.get(name) == Some(contents),
            "{name} differs from the expected file:\n{}",
            written
                .get(name)
                .map(|bytes| String::from_utf8_lossy(bytes).into_owned())
                .unwrap_or_default()
        );
    }
}

/// Checks that the store of the site `written`, its files below `data/`, is the store of the
/// committed site `expected`, which may hold nothing else, and that each object is named by
/// the SHA-256 of its bytes.
pub(crate) fn assert_same_store(written: &Path, expected: &Path) {
    assert_objects_named_by_their_bytes(written);
    let written: BTreeMap<_, _> = files(written)
        .into_iter()
        .filter(|(path, _)| is_store_file(path))
        .collect();
    let expected: BTreeMap<_, _> = files(expected)
        .into_iter()
        .filter(|(path, _)| is_store_file(path))
        .collect();
    assert_eq!(
        written.keys().collect::<Vec<_>>(),
        expected.keys().collect::<Vec<_>>(),
        "the store's files"
    );
    let differing: Vec<_> = expected
        .iter()
        .filter(|(path, contents)| written.get(*path) != Some(*contents))
        .map(|(path, _)| path)
        .collect();
    assert!(
        differing.is_empty(),
        "{differing:?} differ from the expected store"
    );
}

/// Whether the file at `path`, a `/`-joined path below the output folder, is one of the store's.
fn is_store_file(path: &str) -> bool {
    path.starts_with("data/") && path.ends_with(".json")
}

// Rule 247: an object is named by the SHA-256 of its own bytes, as they are written.
fn assert_objects_named_by_their_bytes(site: &Path) {
    let Ok(objects) = read_dir(site.join("data/objects")) else {
        return;
    };
    for entry in objects {
        let path = entry.expect("the folder is listed").path();
        let bytes = read(&path).expect("the object is read");
        let hash: String = Sha256::digest(&bytes)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();
        assert_eq!(
            path.file_name()
                .map(|name| name.to_string_lossy().into_owned()),
            Some(format!("{hash}.json")),
            "the object's name"
        );
    }
}
