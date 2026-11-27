use rstest::rstest;

use super::*;
use crate::entities::BuildTarget;
use crate::entities::Colour;
use crate::test_support::name;

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
    resolve_settings(
        &options,
        &config_folder(Path::new(file)),
        &config,
        Destination::Folder,
    )
    .input
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
        resolve_settings(&options, Path::new(""), &config, Destination::Folder)
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
        resolve_settings(&options, Path::new("ci"), &config, Destination::Folder).output
    };
    assert_eq!(resolve(None, None), Path::new("ci").join("site"));
    assert_eq!(resolve(None, Some("out")), Path::new("ci").join("out"));
    assert_eq!(resolve(Some("x"), Some("out")), PathBuf::from("x"));
}

#[test]
fn pages_start_expanded_with_backgrounds_inline_and_no_source_link_unless_the_file_says_otherwise()
{
    let pages = |config: Config| {
        resolve_settings(
            &BuildOptions::default(),
            Path::new(""),
            &config,
            Destination::Folder,
        )
        .pages
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
        resolve_settings(
            &BuildOptions::default(),
            Path::new(""),
            &config,
            Destination::Folder,
        )
        .unnamed
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
    let accents = resolve_settings(
        &BuildOptions::default(),
        Path::new(""),
        &config,
        Destination::Folder,
    )
    .pages
    .accents;
    assert_eq!(
        accents,
        Accents {
            light,
            dark: Accents::DEFAULT.dark
        }
    );
}

#[test]
fn the_replace_option_wins_and_builds_are_replaced_by_default() {
    let replace = |option: Option<bool>, set: Option<bool>| {
        let options = BuildOptions {
            replace_existing_version: option,
            ..BuildOptions::default()
        };
        let config = Config {
            replace_existing_version: set,
            ..Config::default()
        };
        resolve_settings(&options, Path::new(""), &config, Destination::Folder).replace
    };
    assert!(!replace(None, Some(false)));
    assert!(replace(None, None));
    assert!(replace(Some(true), Some(false)));
}

#[rstest]
#[case::a_site(BuildTarget::Site)]
#[case::a_version(BuildTarget::Version(name("main")))]
#[case::a_preview(BuildTarget::Preview(name("pr-1")))]
fn the_target_is_the_command_s(#[case] target: BuildTarget) {
    let options = BuildOptions {
        target: target.clone(),
        ..BuildOptions::default()
    };
    let settings = resolve_settings(
        &options,
        Path::new(""),
        &Config::default(),
        Destination::Folder,
    );
    assert_eq!(settings.target, target);
}

mod key_options;
