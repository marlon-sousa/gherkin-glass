use std::collections::BTreeMap;
use std::io::{Error as IoError, Result as IoResult};
use std::path::Path;

use rstest::rstest;

use super::*;

/// Configuration files held in memory, each with its text, or with `None` when it is there but
/// cannot be read; any other path is not found.
struct ConfigFiles {
    files: BTreeMap<PathBuf, Option<String>>,
}

impl ConfigReader for ConfigFiles {
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

fn config_file((path, text): &(&str, Option<&str>)) -> (PathBuf, Option<String>) {
    (PathBuf::from(path), text.map(str::to_string))
}

/// The settings of a build over the configuration `files`, with the file `--config` names, if
/// any, or the report of what stops them.
fn load(files: &[(&str, Option<&str>)], config: Option<&str>) -> Result<Settings, String> {
    let files = files.iter().map(config_file).collect();
    let options = BuildOptions {
        config: config.map(PathBuf::from),
        ..BuildOptions::default()
    };
    SettingsService::new(Arc::new(ConfigFiles { files }))
        .load(&options, Destination::Folder)
        .map_err(|errors| errors.to_string())
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
