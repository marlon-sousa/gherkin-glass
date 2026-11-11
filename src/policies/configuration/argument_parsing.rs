//! Policy: reading the command line's arguments into what they ask for.

mod arguments;
mod clap_errors;
mod option_values;

use std::ffi::OsString;

use clap::Parser;
use clap::error::ErrorKind;

use crate::entities::Invocation;
use crate::errors::ArgumentError;

use arguments::Arguments;

/// The arguments after the command's own name.
pub(crate) fn parse_arguments(
    arguments: impl IntoIterator<Item = OsString>,
) -> Result<Invocation, ArgumentError> {
    let arguments = ended_after_the_command(arguments.into_iter().collect());
    let command = command_named_by(&arguments);
    let program = OsString::from("ghg");
    match Arguments::try_parse_from(std::iter::once(program).chain(arguments)) {
        Ok(parsed) => Invocation::try_from(parsed),
        Err(error) => match error.kind() {
            ErrorKind::DisplayHelp => Ok(Invocation::Help),
            ErrorKind::DisplayVersion => Ok(Invocation::Version),
            _ => Err(clap_errors::argument_error(&error, command)),
        },
    }
}

/// `ghg -- build` names the command `build` after the end of the options,
/// which clap would take for an argument; it is read as `ghg build --`,
/// whose every argument also comes after the end of the options.
fn ended_after_the_command(mut arguments: Vec<OsString>) -> Vec<OsString> {
    let ended_first = arguments.first().is_some_and(|first| first == "--");
    let command = arguments.get(1).and_then(|second| second.to_str());
    if ended_first && matches!(command, Some("build" | "serve")) {
        arguments.swap(0, 1);
    }
    arguments
}

/// The command a mistake is made in. ghg's own options take no value, so
/// the first argument names the command, when it is one.
fn command_named_by(arguments: &[OsString]) -> &'static str {
    match arguments.first().and_then(|first| first.to_str()) {
        Some("build") => "ghg build",
        Some("serve") => "ghg serve",
        _ => "ghg",
    }
}

#[cfg(test)]
mod tests;
