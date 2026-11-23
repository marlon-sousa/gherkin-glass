//! Handler: how `ghg build` ended, as the one JSON object that `--message-format json` prints for a program to read.

use serde::Serialize;

use super::build_notes::{BuildNote, notes_of};
use crate::entities::{BuildReport, VersionName};
use crate::errors::{BuildError, LocatedError};

/// Every key is always there, `null` when it does not apply, so that a program reads one shape.
#[derive(Debug, Serialize)]
struct JsonReport {
    outcome: Outcome,
    /// The output folder, from the current folder, as the report's paths are, when a site was
    /// written.
    output: Option<String>,
    version: Option<String>,
    features: Option<usize>,
    scenarios: Option<usize>,
    notes: Vec<BuildNote>,
    errors: Vec<JsonError>,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "kebab-case")]
enum Outcome {
    Built,
    NothingToBuild,
    Stopped,
}

#[derive(Debug, Serialize)]
struct JsonError {
    path: String,
    line: Option<usize>,
    message: String,
}

impl From<LocatedError> for JsonError {
    fn from(problem: LocatedError) -> Self {
        JsonError {
            path: problem.path,
            line: problem.line,
            message: problem.message,
        }
    }
}

/// The report of a build that ended as `ended` says, on one line.
pub(super) fn json_report(ended: &Result<BuildReport, BuildError>) -> String {
    let notes = ended.as_ref().map(notes_of).unwrap_or_default();
    let report = match ended {
        Ok(BuildReport::Written {
            site,
            output,
            version,
        }) => JsonReport {
            output: Some(output.to_string_lossy().into_owned()),
            version: version
                .as_ref()
                .map(VersionName::as_str)
                .map(str::to_string),
            features: Some(site.features),
            scenarios: Some(site.scenarios),
            ..JsonReport::of(Outcome::Built, notes)
        },
        Ok(BuildReport::NothingToBuild { .. }) => JsonReport::of(Outcome::NothingToBuild, notes),
        Err(error) => JsonReport {
            errors: error.problems().into_iter().map(JsonError::from).collect(),
            ..JsonReport::of(Outcome::Stopped, notes)
        },
    };
    json(&report)
}

#[expect(
    clippy::expect_used,
    reason = "strings, numbers, lists of them and options of them always serialise"
)]
fn json(report: &JsonReport) -> String {
    serde_json::to_string(report)
        .expect("strings, numbers, lists of them and options of them always serialise")
}

impl JsonReport {
    /// The report of `outcome` with `notes`, and nothing else in it.
    fn of(outcome: Outcome, notes: Vec<BuildNote>) -> Self {
        JsonReport {
            outcome,
            output: None,
            version: None,
            features: None,
            scenarios: None,
            notes,
            errors: Vec::new(),
        }
    }
}
