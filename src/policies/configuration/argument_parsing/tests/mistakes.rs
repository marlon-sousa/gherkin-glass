//! Each mistake on the command line, named as the error it is.

use rstest::rstest;

use super::*;
use crate::entities::KeptPattern;

fn unknown_option(command: &str, option: &str) -> ArgumentError {
    ArgumentError::UnknownOption {
        command: command.into(),
        option: option.into(),
    }
}

fn unexpected_argument(command: &str, argument: &str) -> ArgumentError {
    ArgumentError::UnexpectedArgument {
        command: command.into(),
        argument: argument.into(),
    }
}

fn missing_value(option: &str) -> ArgumentError {
    ArgumentError::MissingValue {
        option: option.into(),
    }
}

fn unexpected_value(option: &str) -> ArgumentError {
    ArgumentError::UnexpectedValue {
        option: option.into(),
    }
}

fn repeated(option: &str) -> ArgumentError {
    ArgumentError::RepeatedOption {
        option: option.into(),
    }
}

fn unknown_command(name: &str) -> ArgumentError {
    ArgumentError::UnknownCommand { name: name.into() }
}

// Rules 19, 58, 68 and 121.
#[rstest]
#[case::no_command(&[], ArgumentError::MissingCommand)]
#[case::a_misspelt_command(&["biuld"], unknown_command("biuld"))]
#[case::help_as_a_command(&["help"], unknown_command("help"))]
#[case::an_option_of_build_before_it(&["--input", "x"], unknown_option("ghg", "--input"))]
#[case::a_short_option(&["build", "-x"], unknown_option("ghg build", "-x"))]
#[case::an_option_with_a_value(&["build", "--foo=bar"], unknown_option("ghg build", "--foo"))]
#[case::an_option_of_serve_given_to_build(&["build", "--watch"], unknown_option("ghg build", "--watch"))]
#[case::an_unknown_option_of_serve(&["serve", "-x"], unknown_option("ghg serve", "-x"))]
#[case::a_mistake_before_help(&["build", "-x", "--help"], unknown_option("ghg build", "-x"))]
#[case::an_argument(&["build", "features"], unexpected_argument("ghg build", "features"))]
#[case::a_lone_dash(&["build", "-"], unexpected_argument("ghg build", "-"))]
#[case::an_argument_after_a_command_after_the_end_of_the_options(&["--", "build", "x"], unexpected_argument("ghg build", "x"))]
#[case::no_command_after_the_end_of_the_options(&["--", "--help"], unknown_command("--help"))]
#[case::an_option_after_the_end_of_the_options(&["serve", "--", "-x"], unexpected_argument("ghg serve", "-x"))]
#[case::no_value(&["build", "--input"], missing_value("--input"))]
#[case::no_text(&["build", "--title"], missing_value("--title"))]
#[case::a_joined_empty_path(&["build", "--input="], missing_value("--input"))]
#[case::an_empty_path(&["build", "--config", ""], missing_value("--config"))]
#[case::an_empty_title(&["build", "--title="], missing_value("--title"))]
#[case::an_empty_list(&["build", "--only="], missing_value("--only"))]
#[case::an_empty_host(&["serve", "--host", ""], missing_value("--host"))]
#[case::an_empty_port(&["serve", "--port="], missing_value("--port"))]
#[case::a_value_for_help(&["build", "--help=yes"], unexpected_value("--help"))]
#[case::a_value_for_help_before_a_command(&["--help=yes"], unexpected_value("--help"))]
#[case::a_value_for_a_flag(&["serve", "--watch=1"], unexpected_value("--watch"))]
#[case::a_repeated_path(&["build", "--input", "a", "--input", "b"], repeated("--input"))]
#[case::a_repeated_output(&["build", "--output", "a", "--output", "b"], repeated("--output"))]
#[case::a_repeated_flag(&["serve", "--no-build", "--no-build"], repeated("--no-build"))]
#[case::a_port_with_a_sign(&["serve", "--port", "+80"], ArgumentError::NotAPort { value: "+80".into() })]
#[case::a_port_too_large(&["serve", "--port=65536"], ArgumentError::NotAPort { value: "65536".into() })]
#[case::watching_without_building(&["serve", "--no-build", "--watch"], ArgumentError::WatchWithoutBuild)]
// Rule 204.
#[case::a_list_without_building(&["serve", "--no-build", "--only", "-"], ArgumentError::OnlyWithoutBuild)]
// Rules 257 and 258: a version or a preview is named by its command, never by an option.
#[case::a_version_name_given_to_build(&["build", "--version-name", "v1"], unknown_option("ghg build", "--version-name"))]
#[case::a_preview_given_to_build(&["build", "--preview"], unknown_option("ghg build", "--preview"))]
#[case::no_subcommand_of_version(&["version"], ArgumentError::MissingSubcommand { command: "ghg version".into() })]
#[case::no_subcommand_of_preview(&["preview"], ArgumentError::MissingSubcommand { command: "ghg preview".into() })]
#[case::an_unknown_subcommand(
    &["version", "list"],
    ArgumentError::UnknownSubcommand { command: "ghg version".into(), name: "list".into() }
)]
#[case::no_version_named(&["version", "add"], ArgumentError::MissingName { command: "ghg version add".into() })]
#[case::no_preview_named(&["preview", "remove", "--output", "site"], ArgumentError::MissingName { command: "ghg preview remove".into() })]
#[case::a_second_name(&["version", "add", "a", "b"], unexpected_argument("ghg version add", "b"))]
#[case::a_preview_replaced(
    &["preview", "add", "pr-1", "--replace-existing-version", "true"],
    unknown_option("ghg preview add", "--replace-existing-version")
)]
#[case::an_input_to_remove(&["version", "remove", "v1", "--input", "x"], unknown_option("ghg version remove", "--input"))]
#[case::an_argument_after_a_removal(&["preview", "remove", "pr-1", "--", "x"], unexpected_argument("ghg preview remove", "x"))]
// Rule 226.
#[case::an_empty_pattern(&["build", "--keep-in-output="], missing_value("--keep-in-output"))]
#[case::a_pattern_that_is_not_valid(
    &["serve", "--keep-in-output", "{draft"],
    ArgumentError::InvalidKeptPattern {
        pattern: "{draft".into(),
        reason: KeptPattern::new("{draft").expect_err("not valid"),
    }
)]
// Rule 231.
#[case::a_message_format_ghg_does_not_have(
    &["build", "--message-format", "JSON"],
    ArgumentError::NotAMessageFormat { value: "JSON".into() }
)]
#[case::an_empty_message_format(&["build", "--message-format="], missing_value("--message-format"))]
#[case::a_message_format_for_serve(
    &["serve", "--message-format", "json"],
    ArgumentError::UnknownOption { command: "ghg serve".into(), option: "--message-format".into() }
)]
#[case::a_path_as_version_name(
    &["version", "add", "../v1"],
    ArgumentError::InvalidVersionName { name: "../v1".into() }
)]
#[case::a_path_as_preview_name(
    &["preview", "remove", "a/b"],
    ArgumentError::InvalidVersionName { name: "a/b".into() }
)]
fn each_mistake_is_named(#[case] arguments: &[&str], #[case] error: ArgumentError) {
    assert_eq!(parse(arguments), Err(error));
}
