//! Policy: how a build is run, the command line over the configuration file over the defaults.

use std::path::{Path, PathBuf};

use crate::entities::{Accents, BuildOptions, Config, PageOptions, Settings, Unnamed};
use crate::policies::DEFAULT_ACCENTS;

/// The input folder when neither the command line nor the file names one,
/// relative to the configuration file's folder.
pub(crate) const DEFAULT_INPUT: &str = "features";

/// The output folder when neither the command line nor the file names one,
/// relative to the configuration file's folder.
pub(crate) const DEFAULT_OUTPUT: &str = "site";

/// `folder` is the configuration file's folder, the current folder when the
/// file is `ghg.toml` there or there is none; paths from the command line are
/// kept as given, relative to the current folder.
pub(crate) fn resolve_settings(options: &BuildOptions, folder: &Path, config: &Config) -> Settings {
    let path = |option: &Option<PathBuf>, set: &Option<PathBuf>, default: &str| match (option, set)
    {
        (Some(given), _) => given.clone(),
        (None, Some(set)) => folder.join(set),
        (None, None) => folder.join(default),
    };
    let input = path(&options.input, &config.input, DEFAULT_INPUT);
    let output = path(&options.output, &config.output, DEFAULT_OUTPUT);
    let unnamed = match config.allow_unnamed {
        Some(false) => Unnamed::Refused,
        Some(true) | None => Unnamed::Allowed,
    };
    Settings {
        input,
        output,
        unnamed,
        pages: PageOptions {
            title: options.title.clone().or_else(|| config.title.clone()),
            source_link: config.source_link.clone(),
            collapse_rules: config.collapse_rules.unwrap_or(false),
            collapse_scenarios: config.collapse_scenarios.unwrap_or(false),
            show_tree: config.show_tree.unwrap_or(true),
            inline_backgrounds: config.inline_backgrounds.unwrap_or(true),
            tag_links: config.tag_links.clone().unwrap_or_default(),
            accents: Accents {
                light: config.accent_light.unwrap_or(DEFAULT_ACCENTS.light),
                dark: config.accent_dark.unwrap_or(DEFAULT_ACCENTS.dark),
            },
        },
    }
}

/// The folder a configuration file's relative paths start from: an empty
/// path, the current folder, for `ghg.toml`.
pub(crate) fn config_folder(config: &Path) -> PathBuf {
    config.parent().map(Path::to_path_buf).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::Colour;

    fn input(options: (Option<&str>, Option<&str>), file: &str, set: Option<&str>) -> PathBuf {
        let options = BuildOptions {
            config: options.0.map(PathBuf::from),
            input: options.1.map(PathBuf::from),
            ..BuildOptions::default()
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
    fn the_title_option_wins_over_the_file_s() {
        let title = |option: Option<&str>, set: Option<&str>| {
            let options = BuildOptions {
                title: option.map(str::to_string),
                ..BuildOptions::default()
            };
            let config = Config {
                title: set.map(str::to_string),
                ..Config::default()
            };
            resolve_settings(&options, Path::new(""), &config)
                .pages
                .title
        };
        assert_eq!(title(Some("cli"), Some("file")).as_deref(), Some("cli"));
        assert_eq!(title(None, Some("file")).as_deref(), Some("file"));
        assert_eq!(title(None, None), None);
    }

    #[test]
    fn the_output_resolves_as_the_input_does() {
        let resolve = |output: Option<&str>, set: Option<&str>| {
            let options = BuildOptions {
                output: output.map(PathBuf::from),
                ..BuildOptions::default()
            };
            let config = Config {
                output: set.map(PathBuf::from),
                ..Config::default()
            };
            resolve_settings(&options, Path::new("ci"), &config).output
        };
        assert_eq!(resolve(None, None), Path::new("ci").join("site"));
        assert_eq!(resolve(None, Some("out")), Path::new("ci").join("out"));
        assert_eq!(resolve(Some("x"), Some("out")), PathBuf::from("x"));
    }

    #[test]
    fn pages_start_expanded_with_backgrounds_inline_and_no_source_link_unless_the_file_says_otherwise()
     {
        let pages = |config: Config| {
            resolve_settings(&BuildOptions::default(), Path::new(""), &config).pages
        };
        assert_eq!(pages(Config::default()), PageOptions::default());
        let set = pages(Config {
            source_link: Some("https://x/{path}".into()),
            collapse_rules: Some(true),
            collapse_scenarios: Some(true),
            show_tree: Some(false),
            inline_backgrounds: Some(false),
            ..Config::default()
        });
        assert_eq!(set.source_link.as_deref(), Some("https://x/{path}"));
        assert!(set.collapse_rules && set.collapse_scenarios);
        assert!(!set.show_tree);
        assert!(pages(Config::default()).show_tree);
        assert!(!set.inline_backgrounds);
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

    #[test]
    fn each_accent_the_file_does_not_set_takes_its_default() {
        let light = Colour::new(0, 0, 0x80);
        let config = Config {
            accent_light: Some(light),
            ..Config::default()
        };
        let accents = resolve_settings(&BuildOptions::default(), Path::new(""), &config)
            .pages
            .accents;
        assert_eq!(
            accents,
            Accents {
                light,
                dark: DEFAULT_ACCENTS.dark
            }
        );
    }
}
