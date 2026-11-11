//! Policy: the commands and options ghg takes, as clap reads them, and what they ask for.

use std::ffi::OsString;
use std::path::PathBuf;

use clap::builder::{OsStringValueParser, TypedValueParser};
use clap::{ArgAction, Args, Parser, Subcommand};

use crate::entities::{
    ArgumentProblem, BuildOptions, Invocation, ServeMode, ServeOptions, VersionName,
};

use super::option_values::{port, text, version_name};

/// clap prints nothing: `--help` and `--version` are answered with the
/// catalogue's words, and its mistakes become [`ArgumentProblem`]s.
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
    Build(BuildArguments),
    Serve(ServeArguments),
}

/// A value may start with a dash, as in `--title -x`, and is never taken for an option.
#[derive(Debug, Args)]
struct BuildArguments {
    #[arg(long, allow_hyphen_values = true)]
    config: Option<PathBuf>,
    #[arg(long, allow_hyphen_values = true)]
    input: Option<PathBuf>,
    #[arg(long, allow_hyphen_values = true)]
    output: Option<PathBuf>,
    #[arg(long, allow_hyphen_values = true, value_parser = OsStringValueParser::new().try_map(text))]
    title: Option<String>,
    #[arg(long, allow_hyphen_values = true, value_parser = OsStringValueParser::new().try_map(version_name))]
    version_name: Option<VersionName>,
    /// What follows `--`, which no command takes.
    #[arg(last = true)]
    after_the_options: Vec<OsString>,
}

#[derive(Debug, Args)]
struct ServeArguments {
    #[command(flatten)]
    build: BuildArguments,
    #[arg(long, allow_hyphen_values = true, value_parser = OsStringValueParser::new().try_map(text))]
    host: Option<String>,
    #[arg(long, allow_hyphen_values = true, value_parser = OsStringValueParser::new().try_map(port))]
    port: Option<u16>,
    #[arg(long)]
    no_build: bool,
    #[arg(long)]
    watch: bool,
}

impl TryFrom<Arguments> for Invocation {
    type Error = ArgumentProblem;

    fn try_from(arguments: Arguments) -> Result<Self, Self::Error> {
        match arguments.command {
            None => Err(ArgumentProblem::MissingCommand),
            Some(Command::Build(build)) => Ok(Invocation::Build(build.options("ghg build")?)),
            Some(Command::Serve(serve)) => Ok(Invocation::Serve(serve.try_into()?)),
        }
    }
}

impl BuildArguments {
    /// The options, unless an argument follows `--` in `command`.
    fn options(self, command: &str) -> Result<BuildOptions, ArgumentProblem> {
        if let Some(argument) = self.after_the_options.first() {
            return Err(ArgumentProblem::UnexpectedArgument {
                command: command.to_string(),
                argument: argument.to_string_lossy().into_owned(),
            });
        }
        Ok(BuildOptions {
            config: self.config,
            input: self.input,
            version_name: self.version_name,
            output: self.output,
            title: self.title,
        })
    }
}

impl TryFrom<ServeArguments> for ServeOptions {
    type Error = ArgumentProblem;

    fn try_from(arguments: ServeArguments) -> Result<Self, Self::Error> {
        let build = arguments.build.options("ghg serve")?;
        let mode = match (arguments.no_build, arguments.watch) {
            (true, true) => return Err(ArgumentProblem::WatchWithoutBuild),
            (true, false) => ServeMode::Folder,
            (false, true) => ServeMode::Watch,
            (false, false) => ServeMode::Build,
        };
        Ok(ServeOptions {
            build,
            host: arguments.host,
            port: arguments.port,
            mode,
        })
    }
}
