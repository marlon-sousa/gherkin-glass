//! Test support: the files of a site written by a test, compared with a committed expected site. The tests in `tests/` bring it in with `#[path]`, beside `version_placeholder.rs`.

use std::collections::BTreeMap;
use std::fs::{read, read_dir};
use std::path::Path;

use super::version_placeholder::with_version_placeholder;

/// Every file below `folder`, by its `/`-joined path in it, the version in
/// its footer put back as the placeholder.
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
                found.insert(
                    name,
                    with_version_placeholder(file, env!("CARGO_PKG_VERSION")),
                );
            }
        }
    }
    found
}

/// Checks that `written` holds the files of the committed site `expected`, and only those.
pub(crate) fn assert_same_site(written: &Path, expected: &Path) {
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
