//! Policy: reading a configuration file's text into the values it sets.

use std::path::PathBuf;

use toml::Spanned;
use toml::de::{DeTable, DeValue};

use crate::entities::{Config, ConfigProblem, ConfigProblemKind, TagLink, ValueType};
use crate::policies::{PatternProblem, unknown_group, whole_tag_pattern};

/// The keys of a table of `tag_links`, all needed.
const TAG_LINK_KEYS: [&str; 3] = ["pattern", "address", "text"];
use crate::entities::{Colour, Config, ConfigProblem, ConfigProblemKind, Theme, ValueType};
use crate::policies::check_accent;

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
        if name == "tag_links" {
            match tag_links(source, value) {
                Ok(links) => config.tag_links = Some(links),
                Err(found) => problems.extend(found),
            }
            continue;
        }
        let problem = |kind| ConfigProblem { line: at, kind };
        let wrong_type = |expected| {
            problem(ConfigProblemKind::WrongType {
                key: name.to_string(),
                expected,
            })
        };
        let found = match name {
            "input" | "output" | "source_link" | "title" => match text(value) {
                // An empty path names nothing, and would report a line with no path.
                Some("") => Err(problem(ConfigProblemKind::Empty {
                    key: name.to_string(),
                })),
                Some(text) => {
                    match name {
                        "input" => config.input = Some(PathBuf::from(text)),
                        "output" => config.output = Some(PathBuf::from(text)),
                        "title" => config.title = Some(text.to_string()),
                        _ => config.source_link = Some(text.to_string()),
                    }
                    Ok(())
                }
                None => Err(wrong_type(ValueType::Text)),
            },
            "allow_unnamed" | "collapse_rules" | "collapse_scenarios" | "show_tree"
            | "inline_backgrounds" => match value.get_ref().as_bool() {
                Some(set) => {
                    *match name {
                        "allow_unnamed" => &mut config.allow_unnamed,
                        "collapse_rules" => &mut config.collapse_rules,
                        "show_tree" => &mut config.show_tree,
                        "collapse_scenarios" => &mut config.collapse_scenarios,
                        _ => &mut config.inline_backgrounds,
                    } = Some(set);
                    Ok(())
                }
                None => Err(wrong_type(ValueType::Boolean)),
            },
            _ => Err(ConfigProblem {
            "accent_light" | "accent_dark" => {
                let theme = if name == "accent_light" {
                    Theme::Light
                } else {
                    Theme::Dark
                };
                let line = line(value.span().start);
                match text(value).and_then(Colour::from_hex) {
                    Some(colour) => match check_accent(theme, colour) {
                        None if theme == Theme::Light => config.accent_light = Some(colour),
                        None => config.accent_dark = Some(colour),
                        Some(shortfall) => problems.push(ConfigProblem {
                            line,
                            kind: ConfigProblemKind::TooLittleContrast {
                                key: name.to_string(),
                                shortfall,
                            },
                        }),
                    },
                    None => problems.push(ConfigProblem {
                        line,
                        kind: ConfigProblemKind::WrongType {
                            key: name.to_string(),
                            expected: ValueType::Colour,
                        },
                    }),
                }
            }
            _ => problems.push(ConfigProblem {
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

/// Every table of `tag_links`, or every problem in them.
fn tag_links(
    source: &str,
    value: &Spanned<DeValue<'_>>,
) -> Result<Vec<TagLink>, Vec<ConfigProblem>> {
    let line = |offset: usize| Some(line_of(source, offset));
    let not_list = || {
        vec![ConfigProblem {
            line: line(value.span().start),
            kind: ConfigProblemKind::NotTableList,
        }]
    };
    let tables: Vec<_> = value
        .get_ref()
        .as_array()
        .ok_or_else(not_list)?
        .iter()
        .map(|entry| {
            entry
                .get_ref()
                .as_table()
                .map(|table| (entry.span().start, table))
        })
        .collect::<Option<_>>()
        .ok_or_else(not_list)?;
    let mut links = Vec::new();
    let mut problems = Vec::new();
    for (start, table) in tables {
        let mut found: [Option<(&str, usize)>; 3] = [None; 3];
        for (key, value) in table {
            let name = key.get_ref().as_ref();
            let full = format!("tag_links.{name}");
            let at = line(value.span().start);
            let Some(slot) = TAG_LINK_KEYS.iter().position(|known| *known == name) else {
                problems.push(ConfigProblem {
                    line: line(key.span().start),
                    kind: ConfigProblemKind::UnknownKey { key: full },
                });
                continue;
            };
            match text(value) {
                Some("") => problems.push(ConfigProblem {
                    line: at,
                    kind: ConfigProblemKind::Empty { key: full },
                }),
                Some(text) => found[slot] = Some((text, value.span().start)),
                None => problems.push(ConfigProblem {
                    line: at,
                    kind: ConfigProblemKind::WrongType {
                        key: full,
                        expected: ValueType::Text,
                    },
                }),
            }
        }
        for key in TAG_LINK_KEYS {
            if !table.iter().any(|(name, _)| name.get_ref().as_ref() == key) {
                problems.push(ConfigProblem {
                    line: line(start),
                    kind: ConfigProblemKind::Missing {
                        key: format!("tag_links.{key}"),
                    },
                });
            }
        }
        let [pattern, address, text] = found;
        let compiled = pattern.and_then(|(pattern, at)| match whole_tag_pattern(pattern) {
            Ok(compiled) => Some(compiled),
            Err(reason) => {
                problems.push(ConfigProblem {
                    line: line(at),
                    kind: match reason {
                        PatternProblem::TooLarge => ConfigProblemKind::RegexTooLarge {
                            key: "tag_links.pattern".to_string(),
                        },
                        PatternProblem::Syntax(reason) => ConfigProblemKind::NotRegex {
                            key: "tag_links.pattern".to_string(),
                            reason,
                        },
                    },
                });
                None
            }
        });
        let Some(compiled) = compiled else {
            continue;
        };
        let mut sound = true;
        for (key, given) in [("address", address), ("text", text)] {
            if let Some((replacement, at)) = given
                && let Some(group) = unknown_group(replacement, &compiled)
            {
                sound = false;
                problems.push(ConfigProblem {
                    line: line(at),
                    kind: ConfigProblemKind::UnknownGroup {
                        key: format!("tag_links.{key}"),
                        group,
                    },
                });
            }
        }
        if let (true, Some((pattern, _)), Some((address, _)), Some((text, _))) =
            (sound, pattern, address, text)
        {
            links.push(TagLink {
                source: pattern.to_string(),
                pattern: compiled,
                address: address.to_string(),
                text: text.to_string(),
            });
        }
    }
    if problems.is_empty() {
        Ok(links)
    } else {
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
                (
                    Some(4),
                    ConfigProblemKind::Empty {
                        key: "title".into()
                    }
                ),
    fn an_accent_is_a_colour_in_hexadecimal() {
        assert_eq!(
            parse_config("accent_light = \"#000080\"\naccent_dark = \"#FF8\"\n")
                .map(|config| (config.accent_light, config.accent_dark)),
            Ok((
                Some(Colour::new(0, 0, 0x80)),
                Some(Colour::new(0xff, 0xff, 0x88))
            ))
        );
        for value in ["\"navy\"", "\"000080\"", "128", "\"\""] {
            assert_eq!(
                problems(&format!("\naccent_dark = {value}\n")),
                [(
                    Some(2),
                    ConfigProblemKind::WrongType {
                        key: "accent_dark".into(),
                        expected: ValueType::Colour
                    }
                )],
                "{value}"
            );
        }
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
