//! Policy: the tables of `tag_links` in a configuration file, each with its pattern, address and text.

use regex_automata::meta::Regex;
use toml::Spanned;
use toml::de::{DeTable, DeValue};

use super::{problem, text};
use crate::entities::{ConfigProblem, ConfigProblemKind, TagLink, ValueType};
use crate::policies::{PatternProblem, unknown_group, whole_tag_pattern};

/// The keys of a table of `tag_links`, all needed.
const TAG_LINK_KEYS: [&str; 3] = ["pattern", "address", "text"];

/// A key's text and the offset of its value, or `None` when the key is missing, empty or not
/// text.
type Given<'a> = Option<(&'a str, usize)>;

/// Every table of `tag_links`, or every problem in them.
pub(super) fn tag_links(
    source: &str,
    value: &Spanned<DeValue<'_>>,
) -> Result<Vec<TagLink>, Vec<ConfigProblem>> {
    let not_list = || {
        vec![problem(
            source,
            value.span().start,
            ConfigProblemKind::NotTableList,
        )]
    };
    let tables: Vec<_> = value
        .get_ref()
        .as_array()
        .ok_or_else(not_list)?
        .iter()
        .map(table_at)
        .collect::<Option<_>>()
        .ok_or_else(not_list)?;
    let (links, failed): (Vec<_>, Vec<_>) = tables
        .iter()
        .map(|(start, table)| tag_link(source, *start, table))
        .partition(Result::is_ok);
    if failed.is_empty() {
        Ok(links.into_iter().filter_map(Result::ok).collect())
    } else {
        Err(failed
            .into_iter()
            .filter_map(Result::err)
            .flatten()
            .collect())
    }
}

/// The entry's offset and its table, when it is one.
fn table_at<'a, 'b>(entry: &'a Spanned<DeValue<'b>>) -> Option<(usize, &'a DeTable<'b>)> {
    Some((entry.span().start, entry.get_ref().as_table()?))
}

/// The table of `tag_links` at `start`, or every problem in it.
fn tag_link(
    source: &str,
    start: usize,
    table: &DeTable<'_>,
) -> Result<TagLink, Vec<ConfigProblem>> {
    let mut problems: Vec<_> = table
        .iter()
        .filter_map(|(key, value)| key_problem(source, key.get_ref(), key.span().start, value))
        .collect();
    problems.extend(
        TAG_LINK_KEYS
            .iter()
            .filter(|key| lacks(table, key))
            .map(|key| problem(source, start, ConfigProblemKind::Missing { key: full(key) })),
    );
    let [pattern, address, text] = TAG_LINK_KEYS.map(|key| given(table, key));
    let compiled = match pattern.map(|given| compile(source, given)).transpose() {
        Ok(compiled) => compiled,
        Err(found) => {
            problems.push(found);
            None
        }
    };
    if let Some(compiled) = &compiled {
        problems.extend(
            [("address", address), ("text", text)]
                .into_iter()
                .filter_map(|(key, given)| group_problem(source, key, given, compiled)),
        );
    }
    match (problems.is_empty(), compiled, pattern, address, text) {
        (true, Some(compiled), Some((pattern, _)), Some((address, _)), Some((text, _))) => {
            Ok(TagLink {
                source: pattern.to_string(),
                pattern: compiled,
                address: address.to_string(),
                text: text.to_string(),
            })
        }
        _ => Err(problems),
    }
}

fn full(key: &str) -> String {
    format!("tag_links.{key}")
}

/// What is wrong with the key `name` at `at` of a table, or with its value.
fn key_problem(
    source: &str,
    name: &str,
    at: usize,
    value: &Spanned<DeValue<'_>>,
) -> Option<ConfigProblem> {
    let key = full(name);
    if !TAG_LINK_KEYS.contains(&name) {
        return Some(problem(source, at, ConfigProblemKind::UnknownKey { key }));
    }
    let kind = match text(value) {
        Some("") => ConfigProblemKind::Empty { key },
        Some(_) => return None,
        None => ConfigProblemKind::WrongType {
            key,
            expected: ValueType::Text,
        },
    };
    Some(problem(source, value.span().start, kind))
}

fn lacks(table: &DeTable<'_>, key: &str) -> bool {
    !table.contains_key(key)
}

/// The text `key` gives, unless it is empty or not text.
fn given<'a>(table: &'a DeTable<'_>, key: &str) -> Given<'a> {
    let value = table.get(key)?;
    text(value)
        .filter(is_written)
        .map(|text| (text, value.span().start))
}

/// Whether a text says anything.
fn is_written(text: &&str) -> bool {
    !text.is_empty()
}

fn compile(source: &str, (pattern, at): (&str, usize)) -> Result<Regex, ConfigProblem> {
    whole_tag_pattern(pattern).map_err(|reason| pattern_problem(source, at, reason))
}

fn pattern_problem(source: &str, at: usize, reason: PatternProblem) -> ConfigProblem {
    let key = full("pattern");
    let kind = match reason {
        PatternProblem::TooLarge => ConfigProblemKind::RegexTooLarge { key },
        PatternProblem::Syntax(reason) => ConfigProblemKind::NotRegex { key, reason },
    };
    problem(source, at, kind)
}

/// The problem of a replacement that names a group the pattern does not have.
fn group_problem(
    source: &str,
    key: &str,
    given: Given<'_>,
    compiled: &Regex,
) -> Option<ConfigProblem> {
    let (replacement, at) = given?;
    let group = unknown_group(replacement, compiled)?;
    Some(problem(
        source,
        at,
        ConfigProblemKind::UnknownGroup {
            key: full(key),
            group,
        },
    ))
}
