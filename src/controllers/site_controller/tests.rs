mod fakes;
mod serving;

use std::path::PathBuf;

use fakes::*;
use rstest::rstest;

use super::*;

/// The calls of a build from `input` with the settings `destination` gives, from the check of
/// the output folder on.
fn a_build(destination: &str, input: &str) -> Vec<String> {
    [
        format!("site_writer.check {destination}"),
        format!("input_collector.collect {input}"),
        "gherkin_parser.parse a.feature Refused".into(),
        "converter.convert a.feature beside v1.0.0".into(),
        "site_writer.write a.feature".into(),
    ]
    .into()
}

#[test]
fn a_build_loads_the_settings_checks_the_output_then_collects_parses_converts_and_writes() {
    let world = world();
    let options = BuildOptions {
        input: Some(PathBuf::from("specs")),
        ..BuildOptions::default()
    };
    assert_eq!(world.controller().build(&options), Ok(built_site()));
    assert_eq!(
        world.calls.taken(),
        [
            "settings.load Folder",
            "site_writer.check Folder",
            "input_collector.collect specs",
            "gherkin_parser.parse a.feature Refused",
            "converter.convert a.feature beside v1.0.0",
            "site_writer.write a.feature",
        ]
    );
}

#[rstest]
// Rules 20, 38 and 123: what stops a build is reported, and nothing after it runs, so a
// configuration file with a problem stops it before the input is read, and an input with a
// problem before any folder is written.
#[case::by_the_settings(Failing::Settings, 0, BuildError::Config(config_errors()))]
#[case::by_the_output_folder_s_check(Failing::OutputCheck, 1, BuildError::Output(output_error()))]
#[case::by_the_input(Failing::Input, 3, BuildError::Input(input_errors()))]
#[case::by_the_writing(Failing::Writing, 5, BuildError::Output(output_error()))]
fn what_stops_a_build_is_reported_and_ends_it(
    #[case] failing: Failing,
    #[case] calls_of_the_build: usize,
    #[case] error: BuildError,
) {
    let world = world();
    world.fail(failing);
    assert_eq!(
        world.controller().build(&BuildOptions::default()),
        Err(error)
    );
    let expected: Vec<_> = ["settings.load Folder".to_string()]
        .into_iter()
        .chain(
            a_build("Folder", "features")
                .into_iter()
                .take(calls_of_the_build),
        )
        .collect();
    assert_eq!(world.calls.taken(), expected);
}
