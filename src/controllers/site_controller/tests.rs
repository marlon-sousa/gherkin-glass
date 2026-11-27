mod building_from_last;
mod fakes;
mod serving;

use std::path::PathBuf;
use std::sync::atomic::Ordering;

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
        "converter.convert a.feature".into(),
        "site_writer.write".into(),
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
    assert_eq!(
        world.controller().build(&options),
        Ok(BuildReport::Written {
            site: built_site(),
            output: PathBuf::from("site"),
            target: BuildTarget::Site,
        })
    );
    assert_eq!(
        world.calls.taken(),
        [
            "settings.load Folder",
            "site_writer.check Folder",
            "input_collector.collect specs",
            "gherkin_parser.parse a.feature Refused",
            "converter.convert a.feature",
            "site_writer.write",
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

// Rule 229: a build that reads no feature file checks the output folder and collects the input,
// then writes nothing, saying where it read from.
#[test]
fn a_build_that_reads_no_feature_file_writes_nothing() {
    let world = world();
    world.input_collector.no_file.store(true, Ordering::SeqCst);
    assert_eq!(
        world.controller().build(&BuildOptions::default()),
        Ok(BuildReport::NothingToBuild(
            NothingToBuild::InputHoldsNone {
                input: PathBuf::from("features"),
            }
        ))
    );
    assert_eq!(
        world.calls.taken(),
        [
            "settings.load Folder",
            "site_writer.check Folder",
            "input_collector.collect features",
        ]
    );
}

// Rules 16 and 229: an input that holds an error stops the build, even with no feature file.
#[test]
fn an_input_with_an_error_and_no_feature_file_stops_the_build() {
    let world = world();
    world.input_collector.no_file.store(true, Ordering::SeqCst);
    world
        .input_collector
        .input_missing
        .store(true, Ordering::SeqCst);
    assert!(matches!(
        world.controller().build(&BuildOptions::default()),
        Err(BuildError::Input(_))
    ));
}
