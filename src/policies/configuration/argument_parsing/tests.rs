mod action_inputs;
mod mistakes;

use std::collections::BTreeSet;
use std::path::PathBuf;

use clap::CommandFactory;
use rstest::rstest;

use super::*;
use crate::entities::{BuildOptions, ServeMode, ServeOptions, VersionName};

fn parse(arguments: &[&str]) -> Result<Invocation, ArgumentError> {
    parse_arguments(arguments.iter().map(OsString::from))
}

fn build(options: BuildOptions) -> Result<Invocation, ArgumentError> {
    Ok(Invocation::Build(options))
}

fn paths(config: Option<&str>, input: Option<&str>) -> BuildOptions {
    BuildOptions {
        config: config.map(PathBuf::from),
        input: input.map(PathBuf::from),
        ..BuildOptions::default()
    }
}

/// The long options `command` takes, without their dashes, but for
/// `--help` and `--version`, which every command takes.
fn long_options(command: &str) -> BTreeSet<String> {
    let mut arguments = Arguments::command();
    arguments.build();
    arguments
        .find_subcommand(command)
        .into_iter()
        .flat_map(|command| command.get_arguments())
        .filter_map(|argument| argument.get_long())
        .filter(|long| !is_every_command_s(long))
        .map(str::to_string)
        .collect()
}

fn is_every_command_s(long: &str) -> bool {
    ["help", "version"].contains(&long)
}

#[test]
fn clap_accepts_the_commands_as_they_are_declared() {
    Arguments::command().debug_assert();
}

// Rule 56: ghg serve has the options of ghg build.
#[test]
fn serve_takes_every_option_of_build() {
    let serve = long_options("serve");
    assert!(
        long_options("build").is_subset(&serve),
        "ghg serve takes {serve:?}"
    );
}

#[rstest]
#[case::nothing(&["build"], paths(None, None))]
#[case::input_then_joined_config(
    &["build", "--input", "specs", "--config=ci/ghg.toml"],
    paths(Some("ci/ghg.toml"), Some("specs"))
)]
#[case::config_then_joined_input(
    &["build", "--config", "a.toml", "--input=b"],
    paths(Some("a.toml"), Some("b"))
)]
#[case::a_value_that_starts_with_a_dash(&["build", "--input", "--help"], paths(None, Some("--help")))]
#[case::nothing_after_the_end_of_the_options(&["build", "--"], paths(None, None))]
#[case::the_command_after_the_end_of_the_options(&["--", "build"], paths(None, None))]
#[case::an_output_folder(&["build", "--output=out"], BuildOptions {
    output: Some(PathBuf::from("out")),
    ..BuildOptions::default()
})]
#[case::a_title(&["build", "--title", "Banking & more"], BuildOptions {
    title: Some("Banking & more".into()),
    ..BuildOptions::default()
})]
#[case::a_version_name(&["build", "--version-name", "v1.2.0"], BuildOptions {
    version_name: VersionName::new("v1.2.0"),
    ..BuildOptions::default()
})]
#[case::a_joined_version_name(&["build", "--version-name=main"], BuildOptions {
    version_name: VersionName::new("main"),
    ..BuildOptions::default()
})]
fn build_takes_its_options_in_either_spelling_and_order(
    #[case] arguments: &[&str],
    #[case] options: BuildOptions,
) {
    assert_eq!(parse(arguments), build(options));
}

#[rstest]
#[case::building_first(&["serve"], ServeMode::Build)]
#[case::without_building(&["serve", "--no-build"], ServeMode::Folder)]
#[case::watching(&["serve", "--watch"], ServeMode::Watch)]
fn serve_s_flags_say_its_mode(#[case] arguments: &[&str], #[case] mode: ServeMode) {
    let Ok(Invocation::Serve(options)) = parse(arguments) else {
        panic!("{arguments:?} is not a serve");
    };
    assert_eq!(options.mode, mode);
}

#[test]
fn serve_takes_the_options_of_build_and_its_own() {
    let arguments = ["serve", "--input=specs", "--host", "0.0.0.0", "--port", "0"];
    assert_eq!(
        parse(&arguments),
        Ok(Invocation::Serve(ServeOptions {
            build: paths(None, Some("specs")),
            host: Some("0.0.0.0".into()),
            port: Some(0),
            mode: ServeMode::Build,
        }))
    );
}

// Rule 18.
#[rstest]
#[case::help(&["--help"], Invocation::Help)]
#[case::h(&["-h"], Invocation::Help)]
#[case::help_after_build(&["build", "--help"], Invocation::Help)]
#[case::help_after_options(&["serve", "--input", "x", "-h"], Invocation::Help)]
#[case::version(&["--version"], Invocation::Version)]
#[case::v(&["-V"], Invocation::Version)]
#[case::version_after_build(&["build", "-V"], Invocation::Version)]
#[case::the_first_of_both(&["-hV"], Invocation::Help)]
fn help_and_version_answer_wherever_they_are(
    #[case] arguments: &[&str],
    #[case] invocation: Invocation,
) {
    assert_eq!(parse(arguments), Ok(invocation));
}

#[cfg(unix)]
#[test]
fn a_path_is_kept_as_it_is_and_a_title_is_read_as_text() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;
    let not_unicode = OsStr::from_bytes(b"a\xff");
    let arguments = [
        OsStr::new("build"),
        OsStr::new("--input"),
        not_unicode,
        OsStr::new("--title"),
        not_unicode,
    ];
    assert_eq!(
        parse_arguments(arguments.map(OsString::from)),
        build(BuildOptions {
            input: Some(PathBuf::from(not_unicode)),
            title: Some("a\u{fffd}".into()),
            ..BuildOptions::default()
        })
    );
}
