//! Controller: the words of every problem a run reports, from the interface text catalogue.

use i18n_embed_fl::fl;

use crate::entities::{
    ArgumentProblem, ConfigProblemKind, Problem, ProblemKind, Problems, ValueType,
};
use crate::policies::CATALOGUE;
use crate::services::ConfigProblems;

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
    }
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
