//! The map of anchors, checked by building `tests/fixtures/anchor-map` as a site, a version and a
//! preview, and comparing each map with the one committed beside its features, `anchors.json`.

#![expect(
    clippy::expect_used,
    reason = "a test file's helpers are test code, which clippy.toml lets panic"
)]

use std::fs::{read, remove_dir_all};
use std::path::{Path, PathBuf};
use std::process::Command;

use rstest::rstest;

fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/anchor-map")
}

/// What the JSON file at `path` holds.
fn json(path: &Path) -> serde_json::Value {
    let bytes = read(path).expect("the map is there");
    serde_json::from_slice(&bytes).expect("the file is JSON")
}

// Rules 263 to 265: a site's, a version's and a preview's map is in its own folder's `assets/`,
// with every block's line, page and anchor, and every row's own line.
#[rstest]
#[case::a_site(&["build"], "assets/anchors.json")]
#[case::a_version(&["version", "add", "v1.0.0"], "v1.0.0/assets/anchors.json")]
#[case::a_preview(&["preview", "add", "pr-1"], "previews/pr-1/assets/anchors.json")]
fn the_map_is_in_its_target_s_folder_and_maps_every_block(
    #[case] command: &[&str],
    #[case] map: &str,
) {
    let output = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("anchor-map")
        .join(command.join("-"));
    let _ = remove_dir_all(&output);
    let result = Command::new(env!("CARGO_BIN_EXE_ghg"))
        .args(command)
        .args(["--input", "features", "--title", "Library", "--output"])
        .arg(&output)
        .current_dir(fixture())
        .output()
        .expect("the command runs");
    assert_eq!(result.status.code(), Some(0), "{result:?}");
    assert_eq!(
        json(&output.join(map)),
        json(&fixture().join("anchors.json"))
    );
}
