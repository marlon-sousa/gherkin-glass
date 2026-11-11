//! Policy: reading the command line's arguments into what they ask for.

use std::ffi::OsString;
use std::path::PathBuf;

use lexopt::{Arg, Error, Parser};

use crate::entities::{ArgumentProblem, BuildOptions, Invocation};

/// The arguments after the command's own name.
pub(crate) fn parse_arguments(
    arguments: impl IntoIterator<Item = OsString>,
) -> Result<Invocation, ArgumentProblem> {
    let mut parser = Parser::from_args(arguments);
    let command = match parser.next().map_err(|error| problem(error, "ghg"))? {
        None => return Err(ArgumentProblem::MissingCommand),
        Some(Arg::Short('h')) => return Ok(Invocation::Help),
        Some(Arg::Long("help")) => return flag(&mut parser, "--help", Invocation::Help),
        Some(Arg::Short('V')) => return Ok(Invocation::Version),
        Some(Arg::Long("version")) => {
            return flag(&mut parser, "--version", Invocation::Version);
        }
        Some(Arg::Value(name)) => name,
        Some(option) => return Err(unknown_option("ghg", &option)),
    };
    match command.to_str() {
        Some("build") => parse_build(&mut parser),
        _ => Err(ArgumentProblem::UnknownCommand {
            name: command.to_string_lossy().into_owned(),
        }),
    }
}

fn parse_build(parser: &mut Parser) -> Result<Invocation, ArgumentProblem> {
    const COMMAND: &str = "ghg build";
    let mut options = BuildOptions::default();
    while let Some(argument) = parser.next().map_err(|error| problem(error, COMMAND))? {
        let slot = match argument {
            Arg::Short('h') => return Ok(Invocation::Help),
            Arg::Long("help") => return flag(parser, "--help", Invocation::Help),
            Arg::Short('V') => return Ok(Invocation::Version),
            Arg::Long("version") => return flag(parser, "--version", Invocation::Version),
            Arg::Long("config") => (&mut options.config, "--config"),
            Arg::Long("input") => (&mut options.input, "--input"),
            Arg::Value(value) => {
                return Err(ArgumentProblem::UnexpectedArgument {
                    command: COMMAND.to_string(),
                    argument: value.to_string_lossy().into_owned(),
                });
            }
            option => return Err(unknown_option(COMMAND, &option)),
        };
        let (slot, option) = slot;
        if slot.is_some() {
            return Err(ArgumentProblem::RepeatedOption {
                option: option.to_string(),
            });
        }
        let value = parser.value().map_err(|error| problem(error, COMMAND))?;
        // An empty path names nothing, and would report a line with no path.
        if value.is_empty() {
            return Err(ArgumentProblem::MissingValue {
                option: option.to_string(),
            });
        }
        *slot = Some(PathBuf::from(value));
    }
    Ok(Invocation::Build(options))
}

/// A long option that takes no value, such as `--help`, answers at once,
/// unless a value is attached to it, as in `--help=yes`. A short one, such as
/// `-h`, needs no such check: lexopt reads `-hV` as `-h` then `-V`.
fn flag(
    parser: &mut Parser,
    option: &str,
    invocation: Invocation,
) -> Result<Invocation, ArgumentProblem> {
    match parser.optional_value() {
        Some(_) => Err(ArgumentProblem::UnexpectedValue {
            option: option.to_string(),
        }),
        None => Ok(invocation),
    }
}

fn unknown_option(command: &str, option: &Arg) -> ArgumentProblem {
    let option = match option {
        Arg::Short(letter) => format!("-{letter}"),
        Arg::Long(name) => format!("--{name}"),
        Arg::Value(value) => value.to_string_lossy().into_owned(),
    };
    ArgumentProblem::UnknownOption {
        command: command.to_string(),
        option,
    }
}

/// lexopt's own errors, as the problems they are.
fn problem(error: Error, command: &str) -> ArgumentProblem {
    match error {
        Error::MissingValue { option } => ArgumentProblem::MissingValue {
            option: option.unwrap_or_default(),
        },
        Error::UnexpectedValue { option, .. } => ArgumentProblem::UnexpectedValue { option },
        Error::UnexpectedOption(option) => ArgumentProblem::UnknownOption {
            command: command.to_string(),
            option,
        },
        Error::UnexpectedArgument(argument) | Error::NonUnicodeValue(argument) => {
            ArgumentProblem::UnexpectedArgument {
                command: command.to_string(),
                argument: argument.to_string_lossy().into_owned(),
            }
        }
        // Only `value.parse()` and custom errors raise these, and neither is used.
        Error::ParsingFailed { value, .. } => ArgumentProblem::UnexpectedArgument {
            command: command.to_string(),
            argument: value,
        },
        Error::Custom(error) => ArgumentProblem::UnexpectedArgument {
            command: command.to_string(),
            argument: error.to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(arguments: &[&str]) -> Result<Invocation, ArgumentProblem> {
        parse_arguments(arguments.iter().map(OsString::from))
    }

    fn build(config: Option<&str>, input: Option<&str>) -> Invocation {
        Invocation::Build(BuildOptions {
            config: config.map(PathBuf::from),
            input: input.map(PathBuf::from),
        })
    }

    #[test]
    fn build_takes_its_options_in_either_spelling_and_order() {
        assert_eq!(parse(&["build"]), Ok(build(None, None)));
        assert_eq!(
            parse(&["build", "--input", "specs", "--config=ci/ghg.toml"]),
            Ok(build(Some("ci/ghg.toml"), Some("specs")))
        );
        assert_eq!(
            parse(&["build", "--config", "a.toml", "--input=b"]),
            Ok(build(Some("a.toml"), Some("b")))
        );
    }

    #[test]
    fn help_and_version_answer_wherever_they_are() {
        assert_eq!(parse(&["--help"]), Ok(Invocation::Help));
        assert_eq!(parse(&["-h"]), Ok(Invocation::Help));
        assert_eq!(parse(&["build", "--help"]), Ok(Invocation::Help));
        assert_eq!(parse(&["--version"]), Ok(Invocation::Version));
        assert_eq!(parse(&["-V"]), Ok(Invocation::Version));
        assert_eq!(parse(&["build", "-V"]), Ok(Invocation::Version));
        assert_eq!(parse(&["-hV"]), Ok(Invocation::Help));
    }

    #[test]
    fn each_mistake_is_named() {
        use ArgumentProblem::*;
        assert_eq!(parse(&[]), Err(MissingCommand));
        assert_eq!(
            parse(&["biuld"]),
            Err(UnknownCommand {
                name: "biuld".into()
            })
        );
        assert_eq!(
            parse(&["--input", "x"]),
            Err(UnknownOption {
                command: "ghg".into(),
                option: "--input".into()
            })
        );
        assert_eq!(
            parse(&["build", "-x"]),
            Err(UnknownOption {
                command: "ghg build".into(),
                option: "-x".into()
            })
        );
        assert_eq!(
            parse(&["build", "--input"]),
            Err(MissingValue {
                option: "--input".into()
            })
        );
        assert_eq!(
            parse(&["build", "--input="]),
            Err(MissingValue {
                option: "--input".into()
            })
        );
        assert_eq!(
            parse(&["build", "--config", ""]),
            Err(MissingValue {
                option: "--config".into()
            })
        );
        assert_eq!(
            parse(&["build", "--help=yes"]),
            Err(UnexpectedValue {
                option: "--help".into()
            })
        );
        assert_eq!(
            parse(&["build", "--input", "a", "--input", "b"]),
            Err(RepeatedOption {
                option: "--input".into()
            })
        );
        assert_eq!(
            parse(&["build", "features"]),
            Err(UnexpectedArgument {
                command: "ghg build".into(),
                argument: "features".into()
            })
        );
    }
}
