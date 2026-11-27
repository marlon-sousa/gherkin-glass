//! Policy: the commands and options ghg takes, as clap reads them, and what they ask for.

mod build_arguments;
mod version_commands;

use clap::{ArgAction, Parser, Subcommand};

use crate::entities::{BuildTarget, Invocation};
use crate::errors::ArgumentError;

use build_arguments::{BuildCommandArguments, ServeArguments};
use version_commands::{PreviewArguments, VersionArguments};

/// clap prints nothing: `--help` and `--version` are answered with the
/// catalogue's words, and its mistakes become [`ArgumentError`]s.
#[derive(Debug, Parser)]
#[command(
    name = "ghg",
    version,
    propagate_version = true,
    disable_help_flag = true,
    disable_version_flag = true,
    disable_help_subcommand = true
)]
pub(super) struct Arguments {
    #[arg(short, long, action = ArgAction::Help, global = true)]
    help: Option<bool>,
    #[arg(short = 'V', long, action = ArgAction::Version, global = true)]
    version: Option<bool>,
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    Build(BuildCommandArguments),
    Serve(ServeArguments),
    Version(VersionArguments),
    Preview(PreviewArguments),
}

impl TryFrom<Arguments> for Invocation {
    type Error = ArgumentError;

    fn try_from(arguments: Arguments) -> Result<Self, Self::Error> {
        match arguments.command {
            None => Err(ArgumentError::MissingCommand),
            Some(Command::Build(build)) => build.invocation("ghg build", BuildTarget::Site),
            Some(Command::Serve(serve)) => Ok(Invocation::Serve(serve.try_into()?)),
            Some(Command::Version(version)) => version.invocation(),
            Some(Command::Preview(preview)) => preview.invocation(),
        }
    }
}
