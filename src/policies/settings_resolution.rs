//! Policy: how a build is run, the command line over the configuration file over the defaults.

use std::path::{Path, PathBuf};

use crate::entities::{BuildOptions, Config, Settings, Unnamed};

/// The input folder when neither the command line nor the file names one,
/// relative to the configuration file's folder.
pub(crate) const DEFAULT_INPUT: &str = "features";

/// `folder` is the configuration file's folder, the current folder when the
/// file is `ghg.toml` there or there is none; paths from the command line are
/// kept as given, relative to the current folder.
pub(crate) fn resolve_settings(options: &BuildOptions, folder: &Path, config: &Config) -> Settings {
    let input = match (&options.input, &config.input) {
        (Some(input), _) => input.clone(),
        (None, Some(input)) => folder.join(input),
        (None, None) => folder.join(DEFAULT_INPUT),
    };
    let unnamed = match config.allow_unnamed {
        Some(false) => Unnamed::Refused,
        Some(true) | None => Unnamed::Allowed,
    };
    Settings { input, unnamed }
}

/// The folder a configuration file's relative paths start from: an empty
/// path, the current folder, for `ghg.toml`.
pub(crate) fn config_folder(config: &Path) -> PathBuf {
    config.parent().map(Path::to_path_buf).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(options: (Option<&str>, Option<&str>), file: &str, set: Option<&str>) -> PathBuf {
        let options = BuildOptions {
            config: options.0.map(PathBuf::from),
            input: options.1.map(PathBuf::from),
        };
        let config = Config {
            input: set.map(PathBuf::from),
            ..Config::default()
        };
        resolve_settings(&options, &config_folder(Path::new(file)), &config).input
    }

    #[test]
    fn the_option_wins_over_the_file_and_the_file_over_the_default() {
        assert_eq!(
            input((None, Some("cli")), "ghg.toml", Some("file")),
            PathBuf::from("cli")
        );
        assert_eq!(
            input((None, None), "ghg.toml", Some("file")),
            PathBuf::from("file")
        );
        assert_eq!(
            input((None, None), "ghg.toml", None),
            PathBuf::from("features")
        );
    }

    #[test]
    fn the_file_s_paths_start_from_its_folder_and_the_option_s_from_the_current_one() {
        let file = Path::new("ci").join("ghg.toml");
        let file = file.to_str().expect("UTF-8");
        assert_eq!(
            input((Some(file), None), file, Some("specs")),
            Path::new("ci").join("specs")
        );
        assert_eq!(
            input((Some(file), None), file, None),
            Path::new("ci").join("features")
        );
        assert_eq!(
            input((Some(file), Some("x")), file, Some("specs")),
            PathBuf::from("x")
        );
    }

    #[test]
    fn unnamed_blocks_are_allowed_unless_the_file_says_otherwise() {
        let unnamed = |allow_unnamed| {
            let config = Config {
                allow_unnamed,
                ..Config::default()
            };
            resolve_settings(&BuildOptions::default(), Path::new(""), &config).unnamed
        };
        assert_eq!(unnamed(None), Unnamed::Allowed);
        assert_eq!(unnamed(Some(true)), Unnamed::Allowed);
        assert_eq!(unnamed(Some(false)), Unnamed::Refused);
    }
}
