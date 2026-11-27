//! Handler: how a build or a removal ended, worded for a person: the notes of a command that nothing stopped, or what stopped it, on standard error.

use super::build_notes::{BuildNote, notes_of, removal_notes};
use super::build_reporter::BuildReporter;
use super::exit_status::ExitStatus;
use super::{Reply, lines};
use crate::entities::{BuildReport, RemoveReport};
use crate::errors::BuildError;

/// The reporter of `--message-format human`, the default.
#[derive(Debug, Clone, Copy)]
pub(super) struct HumanReporter;

impl BuildReporter for HumanReporter {
    fn report(&self, ended: &Result<BuildReport, BuildError>) -> Reply {
        noted(ended.as_ref().map(notes_of))
    }

    fn report_removal(&self, ended: &Result<RemoveReport, BuildError>) -> Reply {
        noted(ended.as_ref().map(removal_notes))
    }
}

/// The notes of a command that ended as `ended` says, or what stopped it.
fn noted(ended: Result<Vec<BuildNote>, &BuildError>) -> Reply {
    match ended {
        Ok(notes) => Reply {
            stderr: notes.into_iter().map(|note| lines(note.message)).collect(),
            ..Reply::success(String::new())
        },
        Err(error) => Reply::failure(ExitStatus::Stopped, error.to_string()),
    }
}
