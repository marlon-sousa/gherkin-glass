//! Policy: the options of `ghg build` and `ghg serve`, which `ghg version add` and `ghg preview add` take too, as clap reads them, and the options of a build they ask for.

use std::ffi::OsString;
use std::path::PathBuf;

use clap::builder::{BoolValueParser, OsStringValueParser, TypedValueParser};
use clap::{ArgAction, Args};

use crate::entities::{
    BuildOptions, BuildTarget, Colour, Invocation, KeptInOutput, KeptPattern, MessageFormat,
    PathList, ServeMode, ServeOptions,
};
use crate::errors::ArgumentError;

use super::super::option_values::{
    dark_accent, kept_pattern, light_accent, message_format, path_list, port, text,
};
use super::super::tag_link_option::tag_link;

#[derive(Debug, Args)]
pub(super) struct ReportArgument {
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
pub(super) struct BuildArguments {
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
pub(super) struct BuildCommandArguments {
    #[command(flatten)]
    build: BuildArguments,
    #[command(flatten)]
    replace: ReplaceArgument,
    #[command(flatten)]
    report: ReportArgument,
}

#[derive(Debug, Args)]
pub(super) struct ServeArguments {
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

impl ReportArgument {
    /// How the command says how it ended, human words unless it is given.
    pub(super) fn message_format(&self) -> MessageFormat {
        self.message_format.unwrap_or_default()
    }
}

impl BuildCommandArguments {
    /// The build of `target` that `command` asks for.
    pub(super) fn invocation(
        self,
        command: &str,
        target: BuildTarget,
    ) -> Result<Invocation, ArgumentError> {
        Ok(Invocation::Build {
            options: self
                .build
                .options(command, target, self.replace.replace_existing_version)?,
            message_format: self.report.message_format(),
        })
    }
}

/// Nothing, unless an argument follows `--` in `command`.
pub(super) fn unexpected(
    command: &str,
    after_the_options: &[OsString],
) -> Result<(), ArgumentError> {
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
    pub(super) fn options(
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
