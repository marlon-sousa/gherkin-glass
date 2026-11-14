use std::collections::{BTreeMap, BTreeSet};
use std::io::Result as IoResult;
use std::path::Path;

use rstest::rstest;

use super::*;
use crate::entities::SiteScope;

/// Configuration files held in memory, each with its text, or with `None` when it is there but
/// cannot be read; any other path is not found.
struct ConfigFiles {
    files: BTreeMap<PathBuf, Option<String>>,
}

impl FileReader for ConfigFiles {
    fn read(&self, path: &Path) -> IoResult<String> {
        match self.files.get(path) {
            Some(Some(text)) => Ok(text.clone()),
            Some(None) => Err(IoError::new(
                ErrorKind::PermissionDenied,
                "permission denied",
            )),
            None => Err(IoError::new(ErrorKind::NotFound, "not found")),
        }
    }
}

/// Standard input's text, or `None` when it cannot be read.
struct FakeStandardInput {
    text: Option<String>,
}

impl StandardInput for FakeStandardInput {
    fn read(&self) -> IoResult<String> {
        self.text
            .clone()
            .ok_or_else(|| IoError::new(ErrorKind::InvalidData, "not UTF-8"))
    }
}

fn config_file((path, text): &(&str, Option<&str>)) -> (PathBuf, Option<String>) {
    (PathBuf::from(path), text.map(str::to_string))
}

/// The settings of a build over the configuration `files`, with the file `--config` names, if
/// any, or the report of what stops them.
fn load(files: &[(&str, Option<&str>)], config: Option<&str>) -> Result<Settings, String> {
    let options = BuildOptions {
        config: config.map(PathBuf::from),
        ..BuildOptions::default()
    };
    load_with(files, &options, None)
}

/// The settings of a build over the configuration and list `files`, with `options` and
/// `standard_input`, or the report of what stops them.
fn load_with(
    files: &[(&str, Option<&str>)],
    options: &BuildOptions,
    standard_input: Option<&str>,
) -> Result<Settings, String> {
    let files = files.iter().map(config_file).collect();
    let standard_input = FakeStandardInput {
        text: standard_input.map(str::to_string),
    };
    SettingsService::new(Arc::new(ConfigFiles { files }), Arc::new(standard_input))
        .load(options, Destination::Folder)
        .map_err(|errors| errors.to_string())
}

fn only(list: PathList) -> BuildOptions {
    BuildOptions {
        only: Some(list),
        ..BuildOptions::default()
    }
}

#[rstest]
// Rules 21, 23 and 24.
#[case::no_file_and_every_default(&[], None, "features")]
#[case::ghg_toml_in_the_current_folder(&[("ghg.toml", Some("input = \"specs\"\n"))], None, "specs")]
#[case::the_file_asked_for_instead(
    &[("ghg.toml", Some("input = \"specs\"\n")), ("ci/ghg.toml", Some("input = \"specs\"\n"))],
    Some("ci/ghg.toml"),
    "ci/specs"
)]
fn the_file_read_is_the_one_asked_for_or_ghg_toml_which_may_be_missing(
    #[case] files: &[(&str, Option<&str>)],
    #[case] config: Option<&str>,
    #[case] input: &str,
) {
    let settings = load(files, config).expect("loads");
    assert_eq!(settings.input, PathBuf::from(input));
}

#[rstest]
// Rules 20, 21 and 23.
#[case::a_file_asked_for_that_does_not_exist(&[], Some("ci.toml"), "ci.toml: cannot be read: not found\n")]
#[case::a_ghg_toml_that_cannot_be_read(
    &[("ghg.toml", None)],
    None,
    "ghg.toml: cannot be read: permission denied\n"
)]
#[case::every_mistake_of_the_file(
    &[("ghg.toml", Some("inputs = \"x\"\ninput = 1\n"))],
    None,
    "ghg.toml:1: ghg has no setting named inputs.\nghg.toml:2: input must be text, in quotes.\n"
)]
fn a_configuration_that_cannot_be_used_is_reported_whole(
    #[case] files: &[(&str, Option<&str>)],
    #[case] config: Option<&str>,
    #[case] report: &str,
) {
    assert_eq!(load(files, config), Err(report.to_string()));
}

// Rule 204.
#[rstest]
#[case::a_file(only(PathList::File(PathBuf::from("changed.txt"))))]
#[case::standard_input(only(PathList::StandardInput))]
fn the_list_of_only_is_read_from_its_file_or_standard_input(#[case] options: BuildOptions) {
    let files = [("changed.txt", Some("features/a.feature\nsrc/main.rs\n"))];
    let settings =
        load_with(&files, &options, Some("features/a.feature\nsrc/main.rs\n")).expect("loads");
    let listed = settings.only.expect("a list");
    assert_eq!(Some(listed.list), options.only);
    let expected: BTreeSet<PathBuf> = ["features/a.feature", "src/main.rs"]
        .into_iter()
        .map(PathBuf::from)
        .collect();
    assert_eq!(listed.paths, expected);
    assert_eq!(settings.pages.scope, SiteScope::OnlyListed);
}

// Rule 204.
#[rstest]
#[case::a_file_that_does_not_exist(
    only(PathList::File(PathBuf::from("changed.txt"))),
    "changed.txt: cannot be read: not found\n"
)]
#[case::standard_input_that_is_not_text(
    only(PathList::StandardInput),
    "-: cannot be read: not UTF-8\n"
)]
fn a_list_that_cannot_be_read_stops_the_run(#[case] options: BuildOptions, #[case] report: &str) {
    assert_eq!(load_with(&[], &options, None), Err(report.to_string()));
}

#[test]
fn without_only_the_site_shows_every_feature_file() {
    let settings = load(&[], None).expect("loads");
    assert_eq!(settings.only, None);
    assert_eq!(settings.pages.scope, SiteScope::EveryFeature);
}
