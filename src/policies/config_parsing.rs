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
        match name {
            "input" => match text(value) {
                // An empty path names nothing, and would report a line with no path.
                Some("") => problems.push(ConfigProblem {
                    line: line(value.span().start),
                    kind: ConfigProblemKind::Empty {
                        key: name.to_string(),
                    },
                }),
                Some(text) => config.input = Some(PathBuf::from(text)),
                None => problems.push(ConfigProblem {
                    line: line(value.span().start),
                    kind: ConfigProblemKind::WrongType {
                        key: name.to_string(),
                        expected: ValueType::Text,
                    },
                }),
            },
            _ => problems.push(ConfigProblem {
                line: line(key.span().start),
                kind: ConfigProblemKind::UnknownKey {
                    key: name.to_string(),
                },
            }),
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
                input: Some(PathBuf::from("specs/all"))
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
