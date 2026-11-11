//! Policy: reading the command line's arguments into what they ask for.

use std::ffi::OsString;
use std::path::PathBuf;

use lexopt::{Arg, Error, Parser};

use crate::entities::{ArgumentProblem, Invocation, ServeOptions, VersionName};

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
    let (name, serving) = match command.to_str() {
        Some("build") => ("ghg build", false),
        Some("serve") => ("ghg serve", true),
        _ => {
            return Err(ArgumentProblem::UnknownCommand {
                name: command.to_string_lossy().into_owned(),
            });
        }
    };
    let options = match parse_options(&mut parser, name, serving) {
        Ok(options) => options,
        Err(Answer::Now(invocation)) => return Ok(*invocation),
        Err(Answer::Mistake(problem)) => return Err(problem),
    };
    if !serving {
        Ok(Invocation::Build(options.build))
    } else if options.watch && options.no_build {
        Err(ArgumentProblem::WatchWithoutBuild)
    } else {
        Ok(Invocation::Serve(options))
    }
}

/// The options of `command`, `ghg build` or `ghg serve`; only `ghg serve`,
/// `serving`, has those beyond `ghg build`'s.
fn parse_options(
    parser: &mut Parser,
    command: &str,
    serving: bool,
) -> Result<ServeOptions, Answer> {
    let mut options = ServeOptions::default();
    while let Some(argument) = parser.next().map_err(|error| problem(error, command))? {
        let slot = match argument {
            Arg::Short('h') => return Err(Answer::Now(Box::new(Invocation::Help))),
            Arg::Long("help") => return Err(answer(parser, "--help", Invocation::Help)),
            Arg::Short('V') => return Err(Answer::Now(Box::new(Invocation::Version))),
            Arg::Long("version") => return Err(answer(parser, "--version", Invocation::Version)),
            Arg::Long("config") => Slot::Path(&mut options.build.config, "--config"),
            Arg::Long("input") => Slot::Path(&mut options.build.input, "--input"),
            Arg::Long("output") => Slot::Path(&mut options.build.output, "--output"),
            Arg::Long("title") => Slot::Text(&mut options.build.title, "--title"),
            Arg::Long("version-name") => Slot::Version(&mut options.build.version_name),
            Arg::Long("host") if serving => Slot::Host(&mut options.host),
            Arg::Long("port") if serving => Slot::Port(&mut options.port),
            Arg::Long("no-build") if serving => Slot::Flag(&mut options.no_build, "--no-build"),
            Arg::Long("watch") if serving => Slot::Flag(&mut options.watch, "--watch"),
            Arg::Value(value) => {
                return Err(ArgumentProblem::UnexpectedArgument {
                    command: command.to_string(),
                    argument: value.to_string_lossy().into_owned(),
                }
                .into());
            }
            option => return Err(unknown_option(command, &option).into()),
        };
        slot.fill(parser, command)?;
    }
    Ok(options)
}

/// Where an option's value goes.
enum Slot<'a> {
    Path(&'a mut Option<PathBuf>, &'static str),
    Text(&'a mut Option<String>, &'static str),
    Host(&'a mut Option<String>),
    Port(&'a mut Option<u16>),
    Flag(&'a mut bool, &'static str),
    Version(&'a mut Option<VersionName>),
}

impl Slot<'_> {
    fn fill(self, parser: &mut Parser, command: &str) -> Result<(), ArgumentProblem> {
        let repeated = |option: &str| ArgumentProblem::RepeatedOption {
            option: option.to_string(),
        };
        match self {
            Slot::Flag(slot, option) => {
                if *slot {
                    return Err(repeated(option));
                }
                if parser.optional_value().is_some() {
                    return Err(ArgumentProblem::UnexpectedValue {
                        option: option.to_string(),
                    });
                }
                *slot = true;
            }
            Slot::Path(slot, option) => {
                if slot.is_some() {
                    return Err(repeated(option));
                }
                *slot = Some(PathBuf::from(value(parser, command, option)?));
            }
            Slot::Text(slot, option) => {
                if slot.is_some() {
                    return Err(repeated(option));
                }
                let text = value(parser, command, option)?;
                *slot = Some(text.to_string_lossy().into_owned());
            }
            Slot::Version(slot) => {
                if slot.is_some() {
                    return Err(repeated("--version-name"));
                }
                let text = value(parser, command, "--version-name")?;
                let text = text.to_string_lossy();
                *slot = Some(VersionName::new(&text).ok_or_else(|| {
                    ArgumentProblem::InvalidVersionName {
                        name: text.into_owned(),
                    }
                })?);
            }
            Slot::Host(slot) => {
                if slot.is_some() {
                    return Err(repeated("--host"));
                }
                let host = value(parser, command, "--host")?;
                *slot = Some(host.to_string_lossy().into_owned());
            }
            Slot::Port(slot) => {
                if slot.is_some() {
                    return Err(repeated("--port"));
                }
                let port = value(parser, command, "--port")?;
                let port = port.to_string_lossy();
                // Only digits, so that `+80` or ` 80` is not taken for 80.
                let number = port
                    .bytes()
                    .all(|byte| byte.is_ascii_digit())
                    .then(|| port.parse().ok())
                    .flatten();
                *slot = Some(number.ok_or_else(|| ArgumentProblem::NotAPort {
                    value: port.into_owned(),
                })?);
            }
        }
        Ok(())
    }
}

/// An option's value, which may not be empty: an empty path names nothing,
/// and would report a line with no path.
fn value(parser: &mut Parser, command: &str, option: &str) -> Result<OsString, ArgumentProblem> {
    let value = parser.value().map_err(|error| problem(error, command))?;
    if value.is_empty() {
        return Err(ArgumentProblem::MissingValue {
            option: option.to_string(),
        });
    }
    Ok(value)
}

/// What stops reading the options early: help or version, answered at once, or a mistake.
enum Answer {
    Now(Box<Invocation>),
    Mistake(ArgumentProblem),
}

impl From<ArgumentProblem> for Answer {
    fn from(problem: ArgumentProblem) -> Self {
        Answer::Mistake(problem)
    }
}

fn answer(parser: &mut Parser, option: &str, invocation: Invocation) -> Answer {
    match flag(parser, option, invocation) {
        Ok(invocation) => Answer::Now(Box::new(invocation)),
        Err(problem) => Answer::Mistake(problem),
    }
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
    use crate::entities::BuildOptions;

    fn parse(arguments: &[&str]) -> Result<Invocation, ArgumentProblem> {
        parse_arguments(arguments.iter().map(OsString::from))
    }

    fn build(config: Option<&str>, input: Option<&str>) -> Invocation {
        Invocation::Build(BuildOptions {
            config: config.map(PathBuf::from),
            input: input.map(PathBuf::from),
            ..BuildOptions::default()
        })
    }

    #[test]
    fn build_takes_a_title_which_is_not_empty() {
        assert_eq!(
            parse(&["build", "--title", "Banking & more"]),
            Ok(Invocation::Build(BuildOptions {
                title: Some("Banking & more".into()),
                ..BuildOptions::default()
            }))
        );
        assert_eq!(
            parse(&["build", "--title="]),
            Err(ArgumentProblem::MissingValue {
                option: "--title".into()
            })
        );
    }

    #[test]
    fn build_takes_an_output_folder() {
        assert_eq!(
            parse(&["build", "--output=out"]),
            Ok(Invocation::Build(BuildOptions {
                output: Some(PathBuf::from("out")),
                ..BuildOptions::default()
            }))
        );
        assert_eq!(
            parse(&["build", "--output", "a", "--output", "b"]),
            Err(ArgumentProblem::RepeatedOption {
                option: "--output".into()
            })
        );
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
    fn a_version_name_is_taken_in_either_spelling() {
        let versioned = |name| {
            Ok(Invocation::Build(BuildOptions {
                version_name: VersionName::new(name),
                input: Some(PathBuf::from("x")),
                ..BuildOptions::default()
            }))
        };
        assert_eq!(
            parse(&["build", "--version-name", "v1.2.0", "--input", "x"]),
            versioned("v1.2.0")
        );
        assert_eq!(
            parse(&["build", "--input=x", "--version-name=main"]),
            versioned("main")
        );
    }

    #[test]
    fn a_version_name_that_cannot_be_one_is_a_mistake() {
        use ArgumentProblem::*;
        assert_eq!(
            parse(&["build", "--version-name", "../v1"]),
            Err(InvalidVersionName {
                name: "../v1".into()
            })
        );
        assert_eq!(
            parse(&["build", "--version-name="]),
            Err(MissingValue {
                option: "--version-name".into()
            })
        );
        assert_eq!(
            parse(&["build", "--version-name"]),
            Err(MissingValue {
                option: "--version-name".into()
            })
        );
        assert_eq!(
            parse(&["build", "--version-name", "a", "--version-name", "b"]),
            Err(RepeatedOption {
                option: "--version-name".into()
            })
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
