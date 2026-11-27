//! Policy: reading the command line's arguments into what they ask for.

mod arguments;
mod clap_errors;
mod option_values;
mod tag_link_option;

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
/// the first argument names the command, when it is one, and the second its subcommand, when
/// the command has them and it is one.
fn command_named_by(arguments: &[OsString]) -> &'static str {
    let mut named = arguments.iter().map(|argument| argument.to_str());
    match (named.next().flatten(), named.next().flatten()) {
        (Some("build"), _) => "ghg build",
        (Some("serve"), _) => "ghg serve",
        (Some("version"), Some("add")) => "ghg version add",
        (Some("version"), Some("remove")) => "ghg version remove",
        (Some("version"), _) => "ghg version",
        (Some("preview"), Some("add")) => "ghg preview add",
        (Some("preview"), Some("remove")) => "ghg preview remove",
        (Some("preview"), _) => "ghg preview",
        _ => "ghg",
    }
}

#[cfg(test)]
mod tests;
