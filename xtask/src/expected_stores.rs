//! Adapter: the expected stores `cargo xtask build` writes, of the fixtures that commit their store without their site, and the versions of `tests/fixtures/versions`: its output folder and its history's store.

use std::path::Path;

#[path = "../../tests/support/history_builds.rs"]
mod history_builds;

use history_builds::{HISTORY_BUILDS, VERSION_BUILDS};

use crate::expected_sites::replace_folder;
use crate::{cargo_in, root, target};

/// The fixtures whose expected store is committed without their site, each with the arguments
/// of its `ghg build`, run in its folder.
const STORE_BUILDS: [(&str, &[&str]); 5] = [
    ("constructs", &["--input", "."]),
    ("anchors", &["--input", "."]),
    ("doc-strings", &["--input", "."]),
    ("step-sentences", &["--input", "."]),
    ("configuration/configured", &[]),
];

/// Builds each fixture of `STORE_BUILDS` into a folder of the target's, and writes its store as
/// the fixture's expected store, `expected-store/data`.
pub(crate) fn fixture_stores() -> Result<(), String> {
    for (fixture, arguments) in STORE_BUILDS {
        let folder = root().join("tests/fixtures").join(fixture);
        let output = target().join("tmp/stores").join(fixture);
        replace_folder(&output, None)?;
        let given = output.to_string_lossy().into_owned();
        let mut build = vec![
            "run",
            "--quiet",
            "--locked",
            "--package",
            "gherkin-glass",
            "--",
            "build",
            "--output",
            &given,
        ];
        build.extend(arguments);
        cargo_in(&folder, &build)?;
        replace_folder(
            &folder.join("expected-store/data"),
            Some(&output.join("data")),
        )?;
    }
    Ok(())
}

/// The fixture of versions, which adds each of its suites in turn.
const VERSIONS: &str = "tests/fixtures/versions";

/// Adds the fixture's versions in turn into its output folder, `expected`, emptied first, and
/// removes the version `v2.0.0-rc.1` from a copy of it, `expected-removed`.
pub(crate) fn versions_site() -> Result<(), String> {
    let versions = root().join(VERSIONS);
    let output = versions.join("expected");
    replace_folder(&output, None)?;
    add_versions(&versions, &VERSION_BUILDS, &output)?;
    let removed = versions.join("expected-removed");
    replace_folder(&removed, Some(&output))?;
    cargo_in(
        &versions,
        &[
            "run",
            "--quiet",
            "--locked",
            "--package",
            "gherkin-glass",
            "--",
            "version",
            "remove",
            "v2.0.0-rc.1",
            "--output",
            "expected-removed",
        ],
    )
}

/// Adds the history's versions in turn into a folder of the target's, and writes its store
/// as the history's expected store.
pub(crate) fn history_store() -> Result<(), String> {
    let history = root().join(VERSIONS).join("history");
    let output = target().join("tmp/history");
    replace_folder(&output, None)?;
    add_versions(&history, &HISTORY_BUILDS, &output)?;
    replace_folder(&history.join("expected/data"), Some(&output.join("data")))
}

/// Adds each suite of `builds`, a folder of `fixture`, as its version into `output`.
fn add_versions(fixture: &Path, builds: &[(&str, &str)], output: &Path) -> Result<(), String> {
    let given = output.to_string_lossy().into_owned();
    builds.iter().try_for_each(|(suite, version)| {
        cargo_in(
            &fixture.join(suite),
            &[
                "run",
                "--quiet",
                "--locked",
                "--package",
                "gherkin-glass",
                "--",
                "version",
                "add",
                version,
                "--output",
                &given,
            ],
        )
    })
}
