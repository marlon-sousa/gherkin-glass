//! Controller: the words of every problem a run reports, from the interface text catalogue.

use i18n_embed_fl::fl;

use crate::entities::{
    ArgumentProblem, ConfigProblemKind, ListenFailure, Problem, ProblemKind, Problems,
    UnnamedBlock, ValueType,
};
use crate::policies::CATALOGUE;
use crate::services::{ConfigProblems, RebuildFailure, ServeFailure};

pub(super) fn argument_problem(problem: &ArgumentProblem) -> String {
    match problem {
        ArgumentProblem::MissingCommand => fl!(CATALOGUE, "missing-command"),
        ArgumentProblem::UnknownCommand { name } => {
            fl!(CATALOGUE, "unknown-command", name = name.as_str())
        }
        ArgumentProblem::UnknownOption { command, option } => fl!(
            CATALOGUE,
            "unknown-option",
            command = command.as_str(),
            option = option.as_str()
        ),
        ArgumentProblem::MissingValue { option } => {
            fl!(CATALOGUE, "missing-value", option = option.as_str())
        }
        ArgumentProblem::UnexpectedValue { option } => {
            fl!(CATALOGUE, "unexpected-value", option = option.as_str())
        }
        ArgumentProblem::RepeatedOption { option } => {
            fl!(CATALOGUE, "repeated-option", option = option.as_str())
        }
        ArgumentProblem::UnexpectedArgument { command, argument } => fl!(
            CATALOGUE,
            "unexpected-argument",
            command = command.as_str(),
            argument = argument.as_str()
        ),
        ArgumentProblem::NotAPort { value } => {
            fl!(CATALOGUE, "not-a-port", value = value.as_str())
        }
        ArgumentProblem::WatchWithoutBuild => fl!(CATALOGUE, "watch-without-build"),
    }
}

/// What stops ghg serve before it serves, as one line, or the report of
/// the configuration or the input.
pub(super) fn serve_failure(failure: &ServeFailure) -> String {
    match failure {
        ServeFailure::Config(problems) => config_problems(problems),
        ServeFailure::Input(found) => problems(found),
        ServeFailure::Listen {
            port,
            failure: ListenFailure::PortTaken,
            ..
        } => fl!(CATALOGUE, "serve-port-taken", port = port.to_string()),
        ServeFailure::Listen {
            host,
            port,
            failure: ListenFailure::Other { reason },
        } => fl!(
            CATALOGUE,
            "serve-cannot-listen",
            host = host.as_str(),
            port = port.to_string(),
            reason = reason.as_str()
        ),
        ServeFailure::Watch(failure) => fl!(
            CATALOGUE,
            "serve-cannot-watch",
            path = failure.path.to_string_lossy(),
            reason = failure.reason.as_str()
        ),
    }
}

/// The report of a rebuild that stopped, then what is still served.
pub(super) fn rebuild_failure(failure: &RebuildFailure) -> String {
    let report = match failure {
        RebuildFailure::Config(problems) => config_problems(problems),
        RebuildFailure::Input(found) => problems(found),
    };
    report + &fl!(CATALOGUE, "serve-not-rebuilt") + "\n"
}

/// One line per problem, the file shown as it was given.
pub(super) fn config_problems(problems: &ConfigProblems) -> String {
    let path = problems.file.to_string_lossy();
    problems
        .problems
        .iter()
        .map(|problem| line(&path, problem.line, config_message(&problem.kind)))
        .collect()
}

/// The report of docs/BEHAVIOUR.md, "Files that cannot be read", one line per problem.
pub(super) fn problems(problems: &Problems) -> String {
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
        ProblemKind::SameAsFolderPage { page, folder } => match folder {
            Some(folder) => fl!(
                CATALOGUE,
                "same-as-folder-page",
                page = page.as_str(),
                folder = folder.as_str()
            ),
            None => fl!(CATALOGUE, "same-as-home-page", page = page.as_str()),
        },
        ProblemKind::CannotBeWritten { reason } => {
            fl!(CATALOGUE, "cannot-be-written", reason = reason.as_str())
        }
    };
    line(&problem.path, problem.line, message)
}

fn config_message(kind: &ConfigProblemKind) -> String {
    match kind {
        ConfigProblemKind::CannotBeRead { reason } => {
            fl!(CATALOGUE, "config-cannot-be-read", reason = reason.as_str())
        }
        ConfigProblemKind::Syntax { message } => message.clone(),
        ConfigProblemKind::UnknownKey { key } => {
            fl!(CATALOGUE, "config-unknown-key", key = key.as_str())
        }
        ConfigProblemKind::Empty { key } => fl!(CATALOGUE, "config-empty", key = key.as_str()),
        ConfigProblemKind::WrongType {
            key,
            expected: ValueType::Text,
        } => fl!(CATALOGUE, "config-not-text", key = key.as_str()),
        ConfigProblemKind::WrongType {
            key,
            expected: ValueType::Boolean,
        } => fl!(CATALOGUE, "config-not-boolean", key = key.as_str()),
    }
}

/// The line number is passed as text, so that Fluent never groups its digits.
fn line(path: &str, line: Option<usize>, message: String) -> String {
    let text = match line {
        Some(line) => fl!(
            CATALOGUE,
            "problem-with-line",
            path = path,
            line = line.to_string(),
            message = message
        ),
        None => fl!(CATALOGUE, "problem", path = path, message = message),
    };
    text + "\n"
}
