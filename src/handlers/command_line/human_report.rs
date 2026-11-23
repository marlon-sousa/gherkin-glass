//! Handler: how `ghg build` ended, worded for a person: the notes of a build that nothing stopped, or what stopped it, on standard error.

use super::build_notes::notes_of;
use super::build_reporter::BuildReporter;
use super::exit_status::ExitStatus;
use super::{Reply, lines};
use crate::entities::BuildReport;
use crate::errors::BuildError;

/// The reporter of `--message-format human`, the default.
#[derive(Debug, Clone, Copy)]
pub(super) struct HumanReport;

impl BuildReporter for HumanReport {
    fn report(&self, ended: &Result<BuildReport, BuildError>) -> Reply {
        match ended {
            Ok(report) => Reply {
                stderr: notes_of(report)
                    .into_iter()
                    .map(|note| lines(note.message))
                    .collect(),
                ..Reply::success(String::new())
            },
            Err(error) => Reply::failure(ExitStatus::Stopped, error.to_string()),
        }
    }
}
