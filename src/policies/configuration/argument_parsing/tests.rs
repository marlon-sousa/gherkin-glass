mod key_option_mistakes;
mod key_options;
mod mistakes;

use std::collections::BTreeSet;
use std::path::PathBuf;

use clap::CommandFactory;
use rstest::rstest;

use super::*;
use crate::entities::{
    BuildOptions, BuildTarget, KeptInOutput, KeptPattern, MessageFormat, PathList, RemoveOptions,
    ServeMode, ServeOptions,
};
use crate::test_support::name;

fn parse(arguments: &[&str]) -> Result<Invocation, ArgumentError> {
    parse_arguments(arguments.iter().map(OsString::from))
}

fn build(options: BuildOptions) -> Result<Invocation, ArgumentError> {
    Ok(Invocation::Build {
        options,
        message_format: MessageFormat::Human,
    })
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

/// The long options of `ghg build` that set how a site is built, which `ghg serve` takes too:
/// all but `--message-format`, which sets how `ghg build` says how it ended.
fn building_options() -> BTreeSet<String> {
    let mut options = long_options("build");
    options.remove("message-format");
    options
}

fn is_every_command_s(long: &str) -> bool {
    ["help", "version"].contains(&long)
}

// Rule 18.
#[test]
fn the_help_names_every_option() {
    let help = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/messages/en/gherkin-glass.ftl"
    ));
    for option in long_options("build") {
        let with_a_value = format!("  --{option} <");
        let without = format!("  --{option}:");
        assert!(
            help.contains(&with_a_value) || help.contains(&without),
            "the help names --{option}"
        );
    }
}

#[test]
fn clap_accepts_the_commands_as_they_are_declared() {
    Arguments::command().debug_assert();
}

// Rule 56: ghg serve has the options of ghg build that set how a site is built.
#[test]
fn serve_takes_every_option_of_build_that_sets_how_a_site_is_built() {
    let serve = long_options("serve");
    assert!(
        building_options().is_subset(&serve),
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
// Rule 204.
#[case::a_list_of_files(&["build", "--only", "changed.txt"], BuildOptions {
    only: Some(PathList::File(PathBuf::from("changed.txt"))),
    ..BuildOptions::default()
})]
#[case::a_list_on_standard_input(&["build", "--only=-"], BuildOptions {
    only: Some(PathList::StandardInput),
    ..BuildOptions::default()
})]
// Rule 257.
#[case::a_version(&["version", "add", "v1.2.0", "--input=specs"], BuildOptions {
    input: Some(PathBuf::from("specs")),
    target: BuildTarget::Version(name("v1.2.0")),
    ..BuildOptions::default()
})]
#[case::a_version_not_replaced(&["version", "add", "--replace-existing-version", "false", "main"], BuildOptions {
    target: BuildTarget::Version(name("main")),
    replace_existing_version: Some(false),
    ..BuildOptions::default()
})]
// Rule 258.
#[case::a_preview(&["preview", "add", "pr-131", "--output", "site"], BuildOptions {
    output: Some(PathBuf::from("site")),
    target: BuildTarget::Preview(name("pr-131")),
    ..BuildOptions::default()
})]
// Rule 224.
#[case::patterns_to_keep(&["build", "--keep-in-output", ".nojekyll", "--keep-in-output=CNAME"], BuildOptions {
    keep_in_output: Some(KeptInOutput::new(vec![
        KeptPattern::new(".nojekyll").expect("valid"),
        KeptPattern::new("CNAME").expect("valid"),
    ])),
    ..BuildOptions::default()
})]
fn build_takes_its_options_in_either_spelling_and_order(
    #[case] arguments: &[&str],
    #[case] options: BuildOptions,
) {
    assert_eq!(parse(arguments), build(options));
}

// Rules 257 and 258.
#[rstest]
#[case::a_version(
    &["version", "remove", "main", "--output", "site", "--message-format=json"],
    BuildTarget::Version(name("main")),
    MessageFormat::Json
)]
#[case::a_preview(
    &["preview", "remove", "--output=site", "pr-1"],
    BuildTarget::Preview(name("pr-1")),
    MessageFormat::Human
)]
fn remove_takes_a_name_and_the_output_folder(
    #[case] arguments: &[&str],
    #[case] target: BuildTarget,
    #[case] message_format: MessageFormat,
) {
    assert_eq!(
        parse(arguments),
        Ok(Invocation::Remove {
            options: RemoveOptions {
                output: Some(PathBuf::from("site")),
                ..RemoveOptions::default()
            },
            target,
            message_format,
        })
    );
}

#[test]
fn remove_takes_the_configuration_and_what_to_keep() {
    let arguments = [
        "version",
        "remove",
        "v1",
        "--config",
        "ghg.toml",
        "--keep-in-output",
        "CNAME",
    ];
    let Ok(Invocation::Remove { options, .. }) = parse(&arguments) else {
        panic!("a removal");
    };
    assert_eq!(options.config, Some(PathBuf::from("ghg.toml")));
    assert_eq!(
        options.keep_in_output,
        Some(KeptInOutput::new(vec![
            KeptPattern::new("CNAME").expect("valid")
        ]))
    );
}

#[rstest]
// Rule 231.
#[case::by_default(&["build"], MessageFormat::Human)]
#[case::for_a_person(&["build", "--message-format", "human"], MessageFormat::Human)]
#[case::for_a_program(&["build", "--message-format=json"], MessageFormat::Json)]
fn build_takes_its_message_format(#[case] arguments: &[&str], #[case] format: MessageFormat) {
    let Ok(Invocation::Build { message_format, .. }) = parse(arguments) else {
        panic!("{arguments:?} is not a build");
    };
    assert_eq!(message_format, format);
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
