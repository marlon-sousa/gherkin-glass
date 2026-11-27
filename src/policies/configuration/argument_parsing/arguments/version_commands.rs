//! Policy: the commands of versions and previews, `ghg version add` and `remove` and `ghg preview add` and `remove`, as clap reads them, and what they ask for.

use std::ffi::OsString;
use std::path::PathBuf;

use clap::builder::{OsStringValueParser, TypedValueParser};
use clap::{ArgAction, Args, Subcommand};

use crate::entities::{
    BuildTarget, Invocation, KeptInOutput, KeptPattern, RemoveOptions, VersionName,
};
use crate::errors::ArgumentError;

use super::super::option_values::{kept_pattern, version_name};
use super::build_arguments::{BuildArguments, BuildCommandArguments, ReportArgument, unexpected};

/// `ghg version`, whose subcommand is missing when it is `None`.
#[derive(Debug, Args)]
pub(super) struct VersionArguments {
    #[command(subcommand)]
    action: Option<VersionAction>,
}

#[derive(Debug, Subcommand)]
enum VersionAction {
    Add(VersionAddArguments),
    Remove(RemoveArguments),
}

/// `ghg preview`, whose subcommand is missing when it is `None`.
#[derive(Debug, Args)]
pub(super) struct PreviewArguments {
    #[command(subcommand)]
    action: Option<PreviewAction>,
}

#[derive(Debug, Subcommand)]
enum PreviewAction {
    Add(PreviewAddArguments),
    Remove(RemoveArguments),
}

/// The name of a version or a preview, missing when it is `None`.
#[derive(Debug, Args)]
struct NameArgument {
    #[arg(value_parser = OsStringValueParser::new().try_map(version_name))]
    name: Option<VersionName>,
}

impl NameArgument {
    fn name(self, command: &str) -> Result<VersionName, ArgumentError> {
        self.name.ok_or_else(|| ArgumentError::MissingName {
            command: command.to_string(),
        })
    }
}

/// `ghg version add`: a name, the options of `ghg build`, and how it says how it ended.
#[derive(Debug, Args)]
struct VersionAddArguments {
    #[command(flatten)]
    name: NameArgument,
    #[command(flatten)]
    build: BuildCommandArguments,
}

/// `ghg preview add`: a name, the options of `ghg build` but `--replace-existing-version`, and
/// how it says how it ended.
#[derive(Debug, Args)]
struct PreviewAddArguments {
    #[command(flatten)]
    name: NameArgument,
    #[command(flatten)]
    build: BuildArguments,
    #[command(flatten)]
    report: ReportArgument,
}

/// `ghg version remove` and `ghg preview remove`: a name, the output folder, how it is found and
/// what it keeps, and how it says how it ended.
#[derive(Debug, Args)]
struct RemoveArguments {
    #[command(flatten)]
    name: NameArgument,
    #[arg(long, allow_hyphen_values = true)]
    config: Option<PathBuf>,
    #[arg(long, allow_hyphen_values = true)]
    output: Option<PathBuf>,
    #[arg(long, allow_hyphen_values = true, action = ArgAction::Append, value_parser = OsStringValueParser::new().try_map(kept_pattern))]
    keep_in_output: Vec<KeptPattern>,
    #[command(flatten)]
    report: ReportArgument,
    /// What follows `--`, which no command takes.
    #[arg(last = true)]
    after_the_options: Vec<OsString>,
}

impl VersionArguments {
    /// What `ghg version` and its subcommand ask for.
    pub(super) fn invocation(self) -> Result<Invocation, ArgumentError> {
        match self.action {
            None => Err(missing_subcommand("ghg version")),
            Some(VersionAction::Add(add)) => {
                let command = "ghg version add";
                let name = add.name.name(command)?;
                add.build.invocation(command, BuildTarget::Version(name))
            }
            Some(VersionAction::Remove(remove)) => {
                remove.invocation("ghg version remove", BuildTarget::Version)
            }
        }
    }
}

impl PreviewArguments {
    /// What `ghg preview` and its subcommand ask for.
    pub(super) fn invocation(self) -> Result<Invocation, ArgumentError> {
        match self.action {
            None => Err(missing_subcommand("ghg preview")),
            Some(PreviewAction::Add(add)) => {
                let command = "ghg preview add";
                let target = BuildTarget::Preview(add.name.name(command)?);
                Ok(Invocation::Build {
                    options: add.build.options(command, target, None)?,
                    message_format: add.report.message_format(),
                })
            }
            Some(PreviewAction::Remove(remove)) => {
                remove.invocation("ghg preview remove", BuildTarget::Preview)
            }
        }
    }
}

fn missing_subcommand(command: &str) -> ArgumentError {
    ArgumentError::MissingSubcommand {
        command: command.to_string(),
    }
}

impl RemoveArguments {
    /// The removal `command` asks for, of the target `named` names.
    fn invocation(
        self,
        command: &str,
        named: fn(VersionName) -> BuildTarget,
    ) -> Result<Invocation, ArgumentError> {
        let name = self.name.name(command)?;
        unexpected(command, &self.after_the_options)?;
        Ok(Invocation::Remove {
            options: RemoveOptions {
                config: self.config,
                output: self.output,
                keep_in_output: (!self.keep_in_output.is_empty())
                    .then(|| KeptInOutput::new(self.keep_in_output)),
            },
            target: named(name),
            message_format: self.report.message_format(),
        })
    }
}
