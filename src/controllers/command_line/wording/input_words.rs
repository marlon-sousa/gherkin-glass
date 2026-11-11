//! Controller: the words of every problem found in the input, from the interface text catalogue.

use i18n_embed_fl::fl;

use super::line;
use crate::entities::{Problem, ProblemKind, Problems, UnnamedBlock};
use crate::policies::CATALOGUE;

/// One line per problem found in the input.
pub(crate) fn problems(problems: &Problems) -> String {
    problems.as_slice().iter().map(input_problem).collect()
}

fn input_problem(problem: &Problem) -> String {
    let message = match &problem.kind {
        ProblemKind::InputMissing => fl!(CATALOGUE, "input-missing"),
        ProblemKind::CannotBeListed { reason } => {
            fl!(
                CATALOGUE,
                "input-cannot-be-listed",
                reason = reason.as_str()
            )
        }
        ProblemKind::CannotBeRead { reason } => {
            fl!(CATALOGUE, "input-cannot-be-read", reason = reason.as_str())
        }
        ProblemKind::InvalidRule { reason } => reason.clone(),
        ProblemKind::Parser { message } => message.clone(),
        ProblemKind::Syntax => fl!(CATALOGUE, "input-syntax-error"),
        ProblemKind::Changed => fl!(CATALOGUE, "input-changed"),
        ProblemKind::Unnamed { block } => match block {
            UnnamedBlock::Feature => fl!(CATALOGUE, "unnamed-feature"),
            UnnamedBlock::Rule => fl!(CATALOGUE, "unnamed-rule"),
            UnnamedBlock::Scenario => fl!(CATALOGUE, "unnamed-scenario"),
            UnnamedBlock::ScenarioOutline => fl!(CATALOGUE, "unnamed-scenario-outline"),
        },
        ProblemKind::SamePage { page, first } => fl!(
            CATALOGUE,
            "same-page",
            page = page.as_str(),
            first = first.as_str()
        ),
        ProblemKind::SameFolderPage { page, first } => fl!(
            CATALOGUE,
            "same-folder-page",
            page = page.as_str(),
            first = first.as_str()
        ),
        ProblemKind::ReservedPage { page } => {
            fl!(CATALOGUE, "reserved-page", page = page.as_str())
        }
        ProblemKind::SameAsFolderPage { page, folder } => match folder {
            Some(folder) => fl!(
                CATALOGUE,
                "same-as-folder-page",
                page = page.as_str(),
                folder = folder.as_str()
            ),
            None => fl!(CATALOGUE, "same-as-home-page", page = page.as_str()),
        },
        ProblemKind::InTagsFolder { page } => {
            fl!(CATALOGUE, "in-tags-folder", page = page.as_str())
        }
        ProblemKind::CannotBeWritten { reason } => {
            fl!(CATALOGUE, "cannot-be-written", reason = reason.as_str())
        }
    };
    line(&problem.path, problem.line, message)
}
