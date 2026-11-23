//! Handler: how `ghg build` ended, worded for a program: what a person reads, and the one JSON object that `--message-format json` prints on standard output.

use serde::Serialize;

use super::build_notes::{BuildNote, notes_of};
use super::build_reporter::BuildReporter;
use super::human_report::HumanReport;
use super::{Reply, lines};
use crate::entities::{BuildReport, VersionName};
use crate::errors::{BuildError, LocatedError};

/// Every key is always there, `null` when it does not apply, so that a program reads one shape.
#[derive(Debug, Serialize)]
struct JsonObject {
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

/// The reporter of `--message-format json`.
#[derive(Debug, Clone, Copy)]
pub(super) struct JsonReport;

/// What a person reads is still on standard error, as without the option, and the exit status
/// is the same, so that a program can read either.
impl BuildReporter for JsonReport {
    fn report(&self, ended: &Result<BuildReport, BuildError>) -> Reply {
        Reply {
            stdout: lines(json_report(ended)),
            ..HumanReport.report(ended)
        }
    }
}

/// The report of a build that ended as `ended` says, on one line.
fn json_report(ended: &Result<BuildReport, BuildError>) -> String {
    let notes = ended.as_ref().map(notes_of).unwrap_or_default();
    let report = match ended {
        Ok(BuildReport::Written {
            site,
            output,
            version,
        }) => JsonObject {
            output: Some(output.to_string_lossy().into_owned()),
            version: version
                .as_ref()
                .map(VersionName::as_str)
                .map(str::to_string),
            features: Some(site.features),
            scenarios: Some(site.scenarios),
            ..JsonObject::of(Outcome::Built, notes)
        },
        Ok(BuildReport::NothingToBuild { .. }) => JsonObject::of(Outcome::NothingToBuild, notes),
        Err(error) => JsonObject {
            errors: error.problems().into_iter().map(JsonError::from).collect(),
            ..JsonObject::of(Outcome::Stopped, notes)
        },
    };
    json(&report)
}

#[expect(
    clippy::expect_used,
    reason = "strings, numbers, lists of them and options of them always serialise"
)]
fn json(report: &JsonObject) -> String {
    serde_json::to_string(report)
        .expect("strings, numbers, lists of them and options of them always serialise")
}

impl JsonObject {
    /// The report of `outcome` with `notes`, and nothing else in it.
    fn of(outcome: Outcome, notes: Vec<BuildNote>) -> Self {
        JsonObject {
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
