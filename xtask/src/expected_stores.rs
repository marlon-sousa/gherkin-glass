//! Adapter: the expected stores `cargo xtask build` writes, of the fixtures that commit their store without their site, and of the history of versions.

#[path = "../../tests/support/history_builds.rs"]
mod history_builds;

use history_builds::HISTORY_BUILDS;

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

/// The fixture of a history of versions, which builds each of its suites in turn.
const HISTORY: &str = "tests/fixtures/versions/history";

/// Builds the history's versions in turn into a folder of the target's, and writes its store
/// as the history's expected store.
pub(crate) fn history_store() -> Result<(), String> {
    let history = root().join(HISTORY);
    let output = target().join("tmp/history");
    replace_folder(&output, None)?;
    let given = output.to_string_lossy().into_owned();
    for (suite, version) in HISTORY_BUILDS {
        cargo_in(
            &history.join(suite),
            &[
                "run",
                "--quiet",
                "--locked",
                "--package",
                "gherkin-glass",
                "--",
                "build",
                "--output",
                &given,
                "--version-name",
                version,
            ],
        )?;
    }
    replace_folder(&history.join("expected/data"), Some(&output.join("data")))
}
