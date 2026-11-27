//! Handler: how a build or a removal ended, worded for a program: what a person reads, and the one JSON object that `--message-format json` prints on standard output.

use serde::Serialize;

use super::build_notes::{BuildNote, notes_of, removal_notes};
use super::build_reporter::BuildReporter;
use super::human_reporter::HumanReporter;
use super::{Reply, lines};
use crate::entities::{BuildReport, BuildTarget, RemoveReport, VersionName};
use crate::errors::{BuildError, LocatedError};

/// Every key is always there, `null` when it does not apply, so that a program reads one shape.
#[derive(Debug, Serialize)]
struct JsonObject {
    outcome: Outcome,
    /// The output folder, from the current folder, as the report's paths are, when a target was
    /// written or removed.
    output: Option<String>,
    version: Option<String>,
    preview: Option<String>,
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
    Removed,
    NothingToRemove,
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
pub(super) struct JsonReporter;

/// What a person reads is still on standard error, as without the option, and the exit status
/// is the same, so that a program can read either.
impl BuildReporter for JsonReporter {
    fn report(&self, ended: &Result<BuildReport, BuildError>) -> Reply {
        Reply {
            stdout: lines(json_report(ended)),
            ..HumanReporter.report(ended)
        }
    }

    fn report_removal(&self, ended: &Result<RemoveReport, BuildError>) -> Reply {
        Reply {
            stdout: lines(json_removal(ended)),
            ..HumanReporter.report_removal(ended)
        }
    }
}

/// The report of a removal that ended as `ended` says, on one line.
fn json_removal(ended: &Result<RemoveReport, BuildError>) -> String {
    let notes = ended.as_ref().map(removal_notes).unwrap_or_default();
    let report = match ended {
        Ok(RemoveReport::Removed { output, target }) => JsonObject {
            output: Some(output.to_string_lossy().into_owned()),
            ..JsonObject::of(Outcome::Removed, notes).naming(target)
        },
        Ok(RemoveReport::NothingToRemove { .. }) => JsonObject::of(Outcome::NothingToRemove, notes),
        Err(error) => JsonObject::stopped(error, notes),
    };
    json(&report)
}

/// The report of a build that ended as `ended` says, on one line.
fn json_report(ended: &Result<BuildReport, BuildError>) -> String {
    let notes = ended.as_ref().map(notes_of).unwrap_or_default();
    let report = match ended {
        Ok(BuildReport::Written {
            site,
            output,
            target,
        }) => JsonObject {
            output: Some(output.to_string_lossy().into_owned()),
            features: Some(site.features),
            scenarios: Some(site.scenarios),
            ..JsonObject::of(Outcome::Built, notes).naming(target)
        },
        Ok(BuildReport::NothingToBuild { .. }) => JsonObject::of(Outcome::NothingToBuild, notes),
        Err(error) => JsonObject::stopped(error, notes),
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
            preview: None,
            features: None,
            scenarios: None,
            notes,
            errors: Vec::new(),
        }
    }

    /// The report of a command `error` stopped, with `notes`.
    fn stopped(error: &BuildError, notes: Vec<BuildNote>) -> Self {
        JsonObject {
            errors: error.problems().into_iter().map(JsonError::from).collect(),
            ..JsonObject::of(Outcome::Stopped, notes)
        }
    }

    /// This report, naming the version or the preview `target` is.
    fn naming(self, target: &BuildTarget) -> Self {
        let name = |name: Option<&VersionName>| name.map(VersionName::as_str).map(str::to_string);
        JsonObject {
            version: name(target.version()),
            preview: name(target.preview()),
            ..self
        }
    }
}
