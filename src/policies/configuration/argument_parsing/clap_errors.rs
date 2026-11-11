//! Policy: clap's errors, as the mistakes on the command line they are.

use clap::Error;
use clap::error::{ContextKind, ErrorKind};

use crate::entities::ArgumentProblem;

use super::option_values::Rejected;

/// The mistake `error` reports, made in `command`, `ghg`, `ghg build` or
/// `ghg serve`. clap's words are never shown: only what was typed is kept.
pub(super) fn problem(error: &Error, command: &str) -> ArgumentProblem {
    let command = command.to_string();
    match error.kind() {
        ErrorKind::InvalidSubcommand => ArgumentProblem::UnknownCommand {
            name: context(error, ContextKind::InvalidSubcommand),
        },
        ErrorKind::UnknownArgument => {
            let typed = context(error, ContextKind::InvalidArg);
            if typed.starts_with('-') && typed != "-" {
                ArgumentProblem::UnknownOption {
                    command,
                    option: typed,
                }
            } else {
                ArgumentProblem::UnexpectedArgument {
                    command,
                    argument: typed,
                }
            }
        }
        // A path's value given empty, or a value not given at all.
        ErrorKind::InvalidValue => ArgumentProblem::MissingValue {
            option: option(error),
        },
        ErrorKind::ValueValidation => rejected(error),
        ErrorKind::TooManyValues => ArgumentProblem::UnexpectedValue {
            option: option(error),
        },
        ErrorKind::ArgumentConflict => ArgumentProblem::RepeatedOption {
            option: option(error),
        },
        // No other kind is raised for these commands; should one be, it is
        // still a mistake, named by what clap says was typed, if anything.
        _ => ArgumentProblem::UnexpectedArgument {
            command,
            argument: context(error, ContextKind::InvalidArg),
        },
    }
}

/// A value one of the checks of [`super::option_values`] refused.
fn rejected(error: &Error) -> ArgumentProblem {
    let value = context(error, ContextKind::InvalidValue);
    let reason = std::error::Error::source(error)
        .and_then(|source| source.downcast_ref::<Rejected>())
        .copied();
    match reason {
        Some(Rejected::NotAPort) => ArgumentProblem::NotAPort { value },
        Some(Rejected::NotAVersionName) => ArgumentProblem::InvalidVersionName { name: value },
        Some(Rejected::Empty) | None => ArgumentProblem::MissingValue {
            option: option(error),
        },
    }
}

/// The option an error is about, with its dashes, without the value's name
/// clap shows after it, as in `--input <INPUT>`.
fn option(error: &Error) -> String {
    let shown = context(error, ContextKind::InvalidArg);
    shown
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .to_string()
}

fn context(error: &Error, kind: ContextKind) -> String {
    error.get(kind).map(ToString::to_string).unwrap_or_default()
}
