//! Policy: a feature's doc strings corrected from its source, as docs/BEHAVIOUR.md, "Doc strings", says.
//!
//! The `gherkin` crate returns a doc string with its media type glued to the
//! content, a line break before and after it and its escapes kept, and takes
//! the indentation away by the least indented line rather than by the
//! delimiter. It gives no position for the doc string itself, so it is found
//! after the end of its step, which the crate does give.

use std::collections::BTreeMap;

use gherkin::{Feature, Step};

use crate::entities::DocString;

/// Corrects every doc string of `feature` in place, from the `source` it was
/// parsed from, and returns their media types by the byte offset at which
/// their step starts. `None` if `source` is not what `feature` was parsed
/// from.
pub(crate) fn correct_doc_strings(
    feature: &mut Feature,
    source: &str,
) -> Option<BTreeMap<usize, String>> {
    let mut media_types = BTreeMap::new();
    for step in steps_mut(feature) {
        if step.docstring.is_none() {
            continue;
        }
        let read = read_doc_string(source, step.span.end)?;
        if let Some(media_type) = read.media_type {
            media_types.insert(step.span.start, media_type);
        }
        step.docstring = Some(read.content);
    }
    Some(media_types)
}

/// Of its background, its scenarios, and its rules' backgrounds and scenarios.
fn steps_mut(feature: &mut Feature) -> impl Iterator<Item = &mut Step> {
    let Feature {
        background,
        scenarios,
        rules,
        ..
    } = feature;
    let rule_steps = rules.iter_mut().flat_map(|rule| {
        rule.background
            .iter_mut()
            .flat_map(|background| &mut background.steps)
            .chain(
                rule.scenarios
                    .iter_mut()
                    .flat_map(|scenario| &mut scenario.steps),
            )
    });
    background
        .iter_mut()
        .flat_map(|background| &mut background.steps)
        .chain(
            scenarios
                .iter_mut()
                .flat_map(|scenario| &mut scenario.steps),
        )
        .chain(rule_steps)
}

/// Reads the doc string of the step that ends at byte `step_end` of `source`.
///
/// The crate's grammar puts only blank lines and comments between a step and
/// its doc string, and ends the doc string at the next occurrence of its
/// delimiter; the same is done here. Returns `None` if no doc string follows,
/// which cannot happen for a step the crate gave a doc string to.
fn read_doc_string(source: &str, step_end: usize) -> Option<DocString> {
    // The rest of the step's line, then line after line until the delimiter.
    let mut line_start = source.get(step_end..)?.find('\n')? + step_end + 1;
    let (indent, delimiter, after_open) = loop {
        let line_end = source[line_start..]
            .find('\n')
            .map_or(source.len(), |end| line_start + end);
        let line = &source[line_start..line_end];
        let text = line.trim_start_matches([' ', '\t']);
        let indent = line.len() - text.len();
        if let Some(delimiter) = ["\"\"\"", "```"]
            .into_iter()
            .find(|delimiter| text.starts_with(delimiter))
        {
            break (
                line[..indent].chars().count(),
                delimiter,
                line_start + indent + delimiter.len(),
            );
        }
        let rest = text.trim_end_matches('\r');
        if !(rest.is_empty() || rest.starts_with('#')) || line_end == source.len() {
            return None;
        }
        line_start = line_end + 1;
    };
    let close = closing_delimiter(source, after_open, delimiter)?;
    let raw = &source[after_open..close];

    let mut lines = raw
        .split('\n')
        .map(|line| line.strip_suffix('\r').unwrap_or(line));
    let media_type = lines.next().unwrap_or("").trim();
    let mut lines: Vec<&str> = lines.collect();
    // What stands before the closing delimiter on its own line is not content.
    if lines
        .last()
        .is_some_and(|last| last.trim_matches([' ', '\t']).is_empty())
    {
        lines.pop();
    }
    let escaped = match delimiter {
        "```" => "\\`\\`\\`",
        _ => "\\\"\\\"\\\"",
    };
    let content = lines
        .iter()
        .map(|line| dedent(line, indent).replace(escaped, delimiter))
        .collect::<Vec<_>>()
        .join("\n");

    Some(DocString {
        media_type: (!media_type.is_empty()).then(|| media_type.to_string()),
        content,
    })
}

/// Where the doc string opened before byte `from` closes, the way the
/// crate's grammar finds it: at the next occurrence of `delimiter`, except
/// that from a `#` anywhere in a line, with any spaces and tabs before it,
/// the rest of the line is passed over, delimiter or not, provided the line
/// ends with a line break.
fn closing_delimiter(source: &str, from: usize, delimiter: &str) -> Option<usize> {
    let mut at = from;
    while at < source.len() {
        let rest = &source[at..];
        if rest.starts_with(delimiter) {
            return Some(at);
        }
        let blank = rest.len() - rest.trim_start_matches([' ', '\t']).len();
        if rest[blank..].starts_with('#')
            && let Some(end) = rest.find('\n')
        {
            at += end + 1;
            continue;
        }
        at += rest.chars().next().map_or(1, char::len_utf8);
    }
    None
}

/// `line` without up to `indent` characters of leading spaces and tabs.
fn dedent(line: &str, indent: usize) -> &str {
    let mut rest = line;
    for _ in 0..indent {
        match rest.strip_prefix([' ', '\t']) {
            Some(shorter) => rest = shorter,
            None => break,
        }
    }
    rest
}

#[cfg(test)]
mod tests {
    use super::*;

    fn doc_string(source: &str) -> DocString {
        let step_end = source.find("step").expect("a step") + "step".len();
        read_doc_string(source, step_end).expect("a doc string")
    }

    #[test]
    fn a_closing_delimiter_on_the_last_line_without_a_line_break() {
        let read = doc_string("    Given a step\n      \"\"\"\n      text\n      \"\"\"");
        assert_eq!(read.content, "text");
        assert_eq!(read.media_type, None);
    }

    #[test]
    fn an_empty_doc_string() {
        let read = doc_string("    Given a step\n      \"\"\"\n      \"\"\"\n");
        assert_eq!(read.content, "");
    }

    #[test]
    fn a_delimiter_after_a_hash_does_not_close_the_doc_string() {
        let read = doc_string(
            "    Given a step\n      \"\"\"\n      a # b \"\"\"\n      more\n      \"\"\"\n",
        );
        assert_eq!(read.content, "a # b \"\"\"\nmore");
    }

    #[test]
    fn no_doc_string_after_the_step() {
        let source = "    Given a step\n    When another\n";
        assert_eq!(
            read_doc_string(source, source.find("step").unwrap() + 4),
            None
        );
    }
}
