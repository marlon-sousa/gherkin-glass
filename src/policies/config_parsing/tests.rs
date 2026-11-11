use std::path::PathBuf;

use rstest::rstest;

use super::*;
use crate::entities::Theme;

fn problems(source: &str) -> Vec<(Option<usize>, ConfigProblemKind)> {
    parse_config(source)
        .expect_err("problems")
        .into_iter()
        .map(|problem| (problem.line, problem.kind))
        .collect()
}

fn unknown(key: &str) -> ConfigProblemKind {
    ConfigProblemKind::UnknownKey { key: key.into() }
}

fn wrong_type(key: &str, expected: ValueType) -> ConfigProblemKind {
    ConfigProblemKind::WrongType {
        key: key.into(),
        expected,
    }
}

fn empty(key: &str) -> ConfigProblemKind {
    ConfigProblemKind::Empty { key: key.into() }
}

#[test]
fn an_empty_file_sets_nothing() {
    assert_eq!(parse_config(""), Ok(Config::default()));
    assert_eq!(parse_config("# nothing\n"), Ok(Config::default()));
}

#[test]
fn input_is_read_as_written() {
    assert_eq!(
        parse_config("input = \"specs/all\"\n"),
        Ok(Config {
            input: Some(PathBuf::from("specs/all")),
            ..Config::default()
        })
    );
    assert_eq!(
        parse_config("input = 'specs'\n").map(|config| config.input),
        Ok(Some(PathBuf::from("specs")))
    );
}

#[test]
fn every_unknown_key_and_wrong_type_is_reported_on_its_line_in_order() {
    assert_eq!(
        problems("# settings\n\nzebra = 1\ninput = [\n  \"a\",\n]\napple = true\n"),
        [
            (Some(3), unknown("zebra")),
            (Some(4), wrong_type("input", ValueType::Text)),
            (Some(7), unknown("apple")),
        ]
    );
}

#[rstest]
#[case::a_flag_that_is_not_a_boolean(
    "\nallow_unnamed = \"no\"\n",
    2,
    wrong_type("allow_unnamed", ValueType::Boolean)
)]
#[case::an_accent_named(
    "\naccent_dark = \"navy\"\n",
    2,
    wrong_type("accent_dark", ValueType::Colour)
)]
#[case::an_accent_without_its_hash(
    "\naccent_dark = \"000080\"\n",
    2,
    wrong_type("accent_dark", ValueType::Colour)
)]
#[case::an_accent_that_is_a_number(
    "\naccent_dark = 128\n",
    2,
    wrong_type("accent_dark", ValueType::Colour)
)]
#[case::an_empty_accent(
    "\naccent_dark = \"\"\n",
    2,
    wrong_type("accent_dark", ValueType::Colour)
)]
#[case::an_empty_input("input = \"\"\n", 1, empty("input"))]
#[case::a_table_of_unknown_name("input = \"a\"\n\n[site]\ntitle = \"x\"\n", 3, unknown("site"))]
#[case::a_version_name_of_more_than_one_folder(
    "version_name = \"../v1\"\n",
    1,
    ConfigProblemKind::InvalidVersionName { name: "../v1".into() }
)]
fn a_wrong_value_is_reported_on_its_line(
    #[case] source: &str,
    #[case] line: usize,
    #[case] kind: ConfigProblemKind,
) {
    assert_eq!(problems(source), [(Some(line), kind)]);
}

#[test]
fn allow_unnamed_is_true_or_false() {
    assert_eq!(
        parse_config("allow_unnamed = false\n").map(|config| config.allow_unnamed),
        Ok(Some(false))
    );
}

#[test]
fn an_accent_is_a_colour_in_hexadecimal() {
    assert_eq!(
        parse_config("accent_light = \"#000080\"\naccent_dark = \"#FF8\"\n")
            .map(|config| (config.accent_light, config.accent_dark)),
        Ok((
            Some(Colour::new(0, 0, 0x80)),
            Some(Colour::new(0xff, 0xff, 0x88))
        ))
    );
}

#[test]
fn an_accent_short_of_7_to_1_in_its_theme_is_a_problem() {
    let found = problems("accent_light = \"#8cc4ff\"\n\naccent_dark = \"#0b4f9c\"\n");
    let shortfalls: Vec<_> = found
        .iter()
        .map(|(line, kind)| match kind {
            ConfigProblemKind::TooLittleContrast { key, shortfall } => {
                (*line, key.as_str(), shortfall.theme, shortfall.hundredths)
            }
            other => panic!("{other:?}"),
        })
        .collect();
    assert_eq!(
        shortfalls,
        [
            (Some(1), "accent_light", Theme::Light, 163),
            (Some(3), "accent_dark", Theme::Dark, 197),
        ]
    );
}

#[test]
fn the_keys_of_pages_are_read_as_written() {
    let config = parse_config(
        "output = \"out\"\nsource_link = \"https://x/{path}#L{line}\"\n\
         collapse_rules = true\ncollapse_scenarios = false\ntitle = \"Banking\"\n\
         show_tree = false\ninline_backgrounds = false\n",
    )
    .expect("valid");
    assert_eq!(config.output, Some(PathBuf::from("out")));
    assert_eq!(
        config.source_link.as_deref(),
        Some("https://x/{path}#L{line}")
    );
    assert_eq!(config.collapse_rules, Some(true));
    assert_eq!(config.collapse_scenarios, Some(false));
    assert_eq!(config.title.as_deref(), Some("Banking"));
    assert_eq!(config.show_tree, Some(false));
    assert_eq!(config.inline_backgrounds, Some(false));
    assert_eq!(
        problems("output = 1\nsource_link = \"\"\ncollapse_rules = \"yes\"\ntitle = \"\"\n"),
        [
            (Some(1), wrong_type("output", ValueType::Text)),
            (Some(2), empty("source_link")),
            (Some(3), wrong_type("collapse_rules", ValueType::Boolean)),
            (Some(4), empty("title")),
        ]
    );
}

#[test]
fn text_that_is_not_toml_is_reported_where_the_toml_crate_stopped() {
    let found = problems("\ninput = features\n");
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].0, Some(2));
    assert!(
        matches!(&found[0].1, ConfigProblemKind::Syntax { message } if !message.is_empty()),
        "{found:?}"
    );
}

#[test]
fn a_key_given_twice_is_not_toml() {
    let found = problems("input = \"a\"\ninput = \"b\"\n");
    assert_eq!(found[0].0, Some(2));
    assert!(matches!(found[0].1, ConfigProblemKind::Syntax { .. }));
}

#[test]
fn the_version_settings_are_read() {
    let config = parse_config("version_name = \"v1.2.0\"\nreplace_existing_version = false\n")
        .expect("valid");
    assert_eq!(config.version_name, VersionName::new("v1.2.0"));
    assert_eq!(config.replace_existing_version, Some(false));
    assert_eq!(config.allow_unnamed, None);
}

#[test]
fn every_problem_of_the_version_settings_is_reported() {
    assert_eq!(
        problems("version_name = \"\"\nversion_name2 = 1\n\n[x]\n"),
        [
            (Some(1), empty("version_name")),
            (Some(2), unknown("version_name2")),
            (Some(4), unknown("x")),
        ]
    );
    assert_eq!(
        problems("version_name = 1.0\nreplace_existing_version = \"yes\"\n"),
        [
            (Some(1), wrong_type("version_name", ValueType::Text)),
            (
                Some(2),
                wrong_type("replace_existing_version", ValueType::Boolean)
            ),
        ]
    );
}
