//! Policy: reading a configuration file's text into the values it sets.

use std::path::PathBuf;

use toml::Spanned;
use toml::de::{DeTable, DeValue};

use crate::entities::{Config, ConfigProblem, ConfigProblemKind, ValueType};

/// The values `source` sets, or every problem in it, sorted by line. A file
/// that is not TOML has one problem, where the `toml` crate stopped.
pub(crate) fn parse_config(source: &str) -> Result<Config, Vec<ConfigProblem>> {
    let line = |offset: usize| Some(line_of(source, offset));
    let table = DeTable::parse(source).map_err(|error| {
        vec![ConfigProblem {
            line: error.span().map(|span| line_of(source, span.start)),
            kind: ConfigProblemKind::Syntax {
                message: error.message().to_string(),
            },
        }]
    })?;
    let mut config = Config::default();
    let mut problems = Vec::new();
    for (key, value) in table.get_ref() {
        let name = key.get_ref().as_ref();
        let at = line(value.span().start);
        let problem = |kind| ConfigProblem { line: at, kind };
        let wrong_type = |expected| {
            problem(ConfigProblemKind::WrongType {
                key: name.to_string(),
                expected,
            })
        };
        let found = match name {
            "input" | "output" | "source_link" => match text(value) {
                // An empty path names nothing, and would report a line with no path.
                Some("") => Err(problem(ConfigProblemKind::Empty {
                    key: name.to_string(),
                })),
                Some(text) => {
                    match name {
                        "input" => config.input = Some(PathBuf::from(text)),
                        "output" => config.output = Some(PathBuf::from(text)),
                        _ => config.source_link = Some(text.to_string()),
                    }
                    Ok(())
                }
                None => Err(wrong_type(ValueType::Text)),
            },
            "allow_unnamed" | "collapse_rules" | "collapse_scenarios" => {
                match value.get_ref().as_bool() {
                    Some(set) => {
                        *match name {
                            "allow_unnamed" => &mut config.allow_unnamed,
                            "collapse_rules" => &mut config.collapse_rules,
                            _ => &mut config.collapse_scenarios,
                        } = Some(set);
                        Ok(())
                    }
                    None => Err(wrong_type(ValueType::Boolean)),
                }
            }
            _ => Err(ConfigProblem {
                line: line(key.span().start),
                kind: ConfigProblemKind::UnknownKey {
                    key: name.to_string(),
                },
            }),
        };
        if let Err(found) = found {
            problems.push(found);
        }
    }
    if problems.is_empty() {
        Ok(config)
    } else {
        problems.sort();
        Err(problems)
    }
}

fn text<'a>(value: &'a Spanned<DeValue<'_>>) -> Option<&'a str> {
    value.get_ref().as_str()
}

/// Counted from 1; an offset past the end is on the last line.
fn line_of(source: &str, offset: usize) -> usize {
    let before = source.get(..offset).unwrap_or(source);
    before.matches('\n').count() + 1
}

#[cfg(test)]
mod tests {
    use super::*;

    fn problems(source: &str) -> Vec<(Option<usize>, ConfigProblemKind)> {
        parse_config(source)
            .expect_err("problems")
            .into_iter()
            .map(|problem| (problem.line, problem.kind))
            .collect()
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
                (
                    Some(3),
                    ConfigProblemKind::UnknownKey {
                        key: "zebra".into()
                    }
                ),
                (
                    Some(4),
                    ConfigProblemKind::WrongType {
                        key: "input".into(),
                        expected: ValueType::Text
                    }
                ),
                (
                    Some(7),
                    ConfigProblemKind::UnknownKey {
                        key: "apple".into()
                    }
                ),
            ]
        );
    }

    #[test]
    fn allow_unnamed_is_true_or_false() {
        assert_eq!(
            parse_config("allow_unnamed = false\n").map(|config| config.allow_unnamed),
            Ok(Some(false))
        );
        assert_eq!(
            problems("\nallow_unnamed = \"no\"\n"),
            [(
                Some(2),
                ConfigProblemKind::WrongType {
                    key: "allow_unnamed".into(),
                    expected: ValueType::Boolean
                }
            )]
        );
    }

    #[test]
    fn the_keys_of_pages_are_read_as_written() {
        let config = parse_config(
            "output = \"out\"\nsource_link = \"https://x/{path}#L{line}\"\n\
             collapse_rules = true\ncollapse_scenarios = false\n",
        )
        .expect("valid");
        assert_eq!(config.output, Some(PathBuf::from("out")));
        assert_eq!(
            config.source_link.as_deref(),
            Some("https://x/{path}#L{line}")
        );
        assert_eq!(config.collapse_rules, Some(true));
        assert_eq!(config.collapse_scenarios, Some(false));
        assert_eq!(
            problems("output = 1\nsource_link = \"\"\ncollapse_rules = \"yes\"\n"),
            [
                (
                    Some(1),
                    ConfigProblemKind::WrongType {
                        key: "output".into(),
                        expected: ValueType::Text
                    }
                ),
                (
                    Some(2),
                    ConfigProblemKind::Empty {
                        key: "source_link".into()
                    }
                ),
                (
                    Some(3),
                    ConfigProblemKind::WrongType {
                        key: "collapse_rules".into(),
                        expected: ValueType::Boolean
                    }
                ),
            ]
        );
    }

    #[test]
    fn an_empty_input_is_a_problem() {
        assert_eq!(
            problems("input = \"\"\n"),
            [(
                Some(1),
                ConfigProblemKind::Empty {
                    key: "input".into()
                }
            )]
        );
    }

    #[test]
    fn a_table_of_unknown_name_is_an_unknown_key() {
        assert_eq!(
            problems("input = \"a\"\n\n[site]\ntitle = \"x\"\n"),
            [(
                Some(3),
                ConfigProblemKind::UnknownKey { key: "site".into() }
            )]
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
}
