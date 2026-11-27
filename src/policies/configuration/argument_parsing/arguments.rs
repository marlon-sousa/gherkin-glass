//! Policy: the commands and options ghg takes, as clap reads them, and what they ask for.

use std::ffi::OsString;
use std::path::PathBuf;

use clap::builder::{BoolValueParser, OsStringValueParser, TypedValueParser};
use clap::{ArgAction, Args, Parser, Subcommand};

use crate::entities::{
    BuildOptions, BuildTarget, Colour, Invocation, KeptInOutput, KeptPattern, MessageFormat,
    PathList, RemoveOptions, ServeMode, ServeOptions, VersionName,
};
use crate::errors::ArgumentError;

use super::option_values::{
    dark_accent, kept_pattern, light_accent, message_format, path_list, port, text, version_name,
};
use super::tag_link_option::tag_link;

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

/// `ghg version`, whose subcommand is missing when it is `None`.
#[derive(Debug, Args)]
struct VersionArguments {
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
struct PreviewArguments {
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

#[derive(Debug, Args)]
struct ReportArgument {
    #[arg(long, allow_hyphen_values = true, value_parser = OsStringValueParser::new().try_map(message_format))]
    message_format: Option<MessageFormat>,
}

/// Whether a build replaces what an earlier one wrote, which `ghg preview add` always does.
#[derive(Debug, Args)]
struct ReplaceArgument {
    #[arg(long, value_parser = BoolValueParser::new())]
    replace_existing_version: Option<bool>,
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
    #[arg(long, allow_hyphen_values = true)]
    front_page: Option<PathBuf>,
    #[arg(long, allow_hyphen_values = true, value_parser = OsStringValueParser::new().try_map(path_list))]
    only: Option<PathList>,
    /// One of the two options that may be given more than once.
    #[arg(long, allow_hyphen_values = true, action = ArgAction::Append, value_parser = OsStringValueParser::new().try_map(kept_pattern))]
    keep_in_output: Vec<KeptPattern>,
    #[arg(long, allow_hyphen_values = true, value_parser = OsStringValueParser::new().try_map(text))]
    source_link: Option<String>,
    #[arg(long, value_parser = BoolValueParser::new())]
    allow_unnamed: Option<bool>,
    #[arg(long, value_parser = BoolValueParser::new())]
    collapse_rules: Option<bool>,
    #[arg(long, value_parser = BoolValueParser::new())]
    collapse_scenarios: Option<bool>,
    #[arg(long, value_parser = BoolValueParser::new())]
    show_tree: Option<bool>,
    #[arg(long, value_parser = BoolValueParser::new())]
    inline_backgrounds: Option<bool>,
    #[arg(long, allow_hyphen_values = true, value_parser = OsStringValueParser::new().try_map(light_accent))]
    accent_light: Option<Colour>,
    #[arg(long, allow_hyphen_values = true, value_parser = OsStringValueParser::new().try_map(dark_accent))]
    accent_dark: Option<Colour>,
    /// The other option that may be given more than once, each time with its three values, all
    /// in one list, three by three.
    #[arg(
        long,
        allow_hyphen_values = true,
        action = ArgAction::Append,
        num_args = 3,
        value_names = ["pattern", "address", "text"],
        value_parser = OsStringValueParser::new().try_map(text)
    )]
    tag_link: Vec<String>,
    /// What follows `--`, which no command takes.
    #[arg(last = true)]
    after_the_options: Vec<OsString>,
}

/// The options of `ghg build`: those it shares with `ghg serve`, and how it says how it ended.
#[derive(Debug, Args)]
struct BuildCommandArguments {
    #[command(flatten)]
    build: BuildArguments,
    #[command(flatten)]
    replace: ReplaceArgument,
    #[command(flatten)]
    report: ReportArgument,
}

#[derive(Debug, Args)]
struct ServeArguments {
    #[command(flatten)]
    build: BuildArguments,
    #[command(flatten)]
    replace: ReplaceArgument,
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
    type Error = ArgumentError;

    fn try_from(arguments: Arguments) -> Result<Self, Self::Error> {
        match arguments.command {
            None => Err(ArgumentError::MissingCommand),
            Some(Command::Build(build)) => build.invocation("ghg build", BuildTarget::Site),
            Some(Command::Serve(serve)) => Ok(Invocation::Serve(serve.try_into()?)),
            Some(Command::Version(version)) => match version.action {
                None => Err(missing_subcommand("ghg version")),
                Some(VersionAction::Add(add)) => {
                    let command = "ghg version add";
                    let name = add.name.name(command)?;
                    add.build.invocation(command, BuildTarget::Version(name))
                }
                Some(VersionAction::Remove(remove)) => {
                    remove.invocation("ghg version remove", BuildTarget::Version)
                }
            },
            Some(Command::Preview(preview)) => match preview.action {
                None => Err(missing_subcommand("ghg preview")),
                Some(PreviewAction::Add(add)) => {
                    let command = "ghg preview add";
                    let target = BuildTarget::Preview(add.name.name(command)?);
                    Ok(Invocation::Build {
                        options: add.build.options(command, target, None)?,
                        message_format: add.report.message_format.unwrap_or_default(),
                    })
                }
                Some(PreviewAction::Remove(remove)) => {
                    remove.invocation("ghg preview remove", BuildTarget::Preview)
                }
            },
        }
    }
}

fn missing_subcommand(command: &str) -> ArgumentError {
    ArgumentError::MissingSubcommand {
        command: command.to_string(),
    }
}

impl BuildCommandArguments {
    /// The build of `target` that `command` asks for.
    fn invocation(self, command: &str, target: BuildTarget) -> Result<Invocation, ArgumentError> {
        Ok(Invocation::Build {
            options: self
                .build
                .options(command, target, self.replace.replace_existing_version)?,
            message_format: self.report.message_format.unwrap_or_default(),
        })
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
            message_format: self.report.message_format.unwrap_or_default(),
        })
    }
}

/// Nothing, unless an argument follows `--` in `command`.
fn unexpected(command: &str, after_the_options: &[OsString]) -> Result<(), ArgumentError> {
    match after_the_options.first() {
        Some(argument) => Err(ArgumentError::UnexpectedArgument {
            command: command.to_string(),
            argument: argument.to_string_lossy().into_owned(),
        }),
        None => Ok(()),
    }
}

impl BuildArguments {
    /// The options of a build of `target`, which `replace_existing_version` replaces or not,
    /// unless an argument follows `--` in `command`.
    fn options(
        self,
        command: &str,
        target: BuildTarget,
        replace_existing_version: Option<bool>,
    ) -> Result<BuildOptions, ArgumentError> {
        unexpected(command, &self.after_the_options)?;
        let tag_links = if self.tag_link.is_empty() {
            None
        } else {
            Some(
                self.tag_link
                    .chunks(3)
                    .map(tag_link)
                    .collect::<Result<_, _>>()?,
            )
        };
        Ok(BuildOptions {
            config: self.config,
            input: self.input,
            target,
            output: self.output,
            title: self.title,
            front_page: self.front_page,
            only: self.only,
            keep_in_output: (!self.keep_in_output.is_empty())
                .then(|| KeptInOutput::new(self.keep_in_output)),
            source_link: self.source_link,
            allow_unnamed: self.allow_unnamed,
            collapse_rules: self.collapse_rules,
            collapse_scenarios: self.collapse_scenarios,
            show_tree: self.show_tree,
            inline_backgrounds: self.inline_backgrounds,
            replace_existing_version,
            accent_light: self.accent_light,
            accent_dark: self.accent_dark,
            tag_links,
        })
    }
}

impl TryFrom<ServeArguments> for ServeOptions {
    type Error = ArgumentError;

    fn try_from(arguments: ServeArguments) -> Result<Self, Self::Error> {
        let build = arguments.build.options(
            "ghg serve",
            BuildTarget::Site,
            arguments.replace.replace_existing_version,
        )?;
        let mode = match (arguments.no_build, arguments.watch) {
            (true, true) => return Err(ArgumentError::WatchWithoutBuild),
            (true, false) if build.only.is_some() => return Err(ArgumentError::OnlyWithoutBuild),
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
