//! Policy: which parts of a step are highlighted.

use std::collections::HashSet;

use gherkin::{Scenario, Step};
use unicode_normalization::char::is_combining_mark;

use crate::entities::{HighlightedStep, StepPart};

/// `scenario` is the scenario or outline that holds `step`, or `None` for a
/// background's step; only an outline's steps have placeholders.
#[expect(
    clippy::indexing_slicing,
    reason = "every position is inside `chars`, which the scan never passes"
)]
pub(crate) fn highlight_step(step: &Step, scenario: Option<&Scenario>) -> HighlightedStep {
    let columns: HashSet<&str> = scenario
        .into_iter()
        .flat_map(|scenario| &scenario.examples)
        .filter_map(|examples| examples.table.as_ref())
        .filter_map(|table| table.rows.first())
        .flatten()
        .map(String::as_str)
        .collect();
    let chars: Vec<char> = step.value.chars().collect();
    let mut parts = Parts::default();
    let mut at = 0;
    while at < chars.len() {
        if let Some(end) = parameter_end(&chars, at) {
            parts.push(StepPart::Parameter(placeholders_in(
                &chars[at..end],
                &columns,
            )));
            at = end;
        } else if let Some(end) = placeholder_end(&chars, at, &columns) {
            parts.push(StepPart::Placeholder(chars[at..end].iter().collect()));
            at = end;
        } else if let Some(end) = number_end(&chars, at) {
            parts.push(StepPart::Number(chars[at..end].iter().collect()));
            at = end;
        } else {
            let end = word_end(&chars, at);
            parts.plain(&chars[at..end]);
            at = end;
        }
    }
    HighlightedStep {
        keyword: step.keyword.trim_end().to_string(),
        parts: parts.0,
    }
}

/// Parts in order, with neighbouring plain text joined into one part.
#[derive(Default)]
struct Parts(Vec<StepPart>);

impl Parts {
    fn push(&mut self, part: StepPart) {
        self.0.push(part);
    }

    fn plain(&mut self, chars: &[char]) {
        if let Some(StepPart::Plain(text)) = self.0.last_mut() {
            text.extend(chars);
        } else {
            self.0.push(StepPart::Plain(chars.iter().collect()));
        }
    }
}

/// The end, past its closing quote, of a quoted parameter opening at `at`.
fn parameter_end(chars: &[char], at: usize) -> Option<usize> {
    let quote = |index: usize| {
        chars.get(index) == Some(&'"') && (index == 0 || chars.get(index - 1) != Some(&'\\'))
    };
    if !quote(at) {
        return None;
    }
    (at + 1..chars.len())
        .find(|&index| quote(index))
        .map(|index| index + 1)
}

/// The end, past its `>`, of a placeholder naming one of `columns`, opening at `at`.
fn placeholder_end(chars: &[char], at: usize, columns: &HashSet<&str>) -> Option<usize> {
    if chars.get(at) != Some(&'<') {
        return None;
    }
    let rest = chars.get(at + 1..)?;
    let (name, after) = rest.split_at(rest.iter().position(|&c| c == '<' || c == '>')?);
    let end = at + name.len() + 2;
    let name: String = name.iter().collect();
    (after.first() == Some(&'>') && !name.is_empty() && columns.contains(name.as_str()))
        .then_some(end)
}

#[expect(
    clippy::indexing_slicing,
    reason = "every position is inside `chars`, which the scan never passes"
)]
fn placeholders_in(chars: &[char], columns: &HashSet<&str>) -> Vec<StepPart> {
    let mut parts = Parts::default();
    let mut at = 0;
    while at < chars.len() {
        if let Some(end) = placeholder_end(chars, at, columns) {
            parts.push(StepPart::Placeholder(chars[at..end].iter().collect()));
            at = end;
        } else {
            parts.plain(&chars[at..=at]);
            at += 1;
        }
    }
    parts.0
}

/// The end of a number starting at `at`, which stands apart from any word.
fn number_end(chars: &[char], at: usize) -> Option<usize> {
    if at > 0 && chars.get(at - 1).copied().is_some_and(in_word) {
        return None;
    }
    let mut index = at;
    if chars.get(index) == Some(&'-') {
        index += 1;
    }
    if !chars.get(index).is_some_and(char::is_ascii_digit) {
        return None;
    }
    loop {
        while chars.get(index).is_some_and(char::is_ascii_digit) {
            index += 1;
        }
        let separator = chars.get(index).is_some_and(|&c| c == '.' || c == ',');
        let after = chars.get(index + 1).copied();
        if !separator || !after.is_some_and(in_word) {
            break;
        }
        // A point or comma between a digit and a letter makes one word, such as `2.x`.
        if !after.is_some_and(|c| c.is_ascii_digit()) {
            return None;
        }
        index += 1;
    }
    (!chars.get(index).copied().is_some_and(in_word)).then_some(index)
}

/// The end of the text from `at` that is not highlighted: a whole word, with
/// any digits, points and commas inside it, so that no number is found
/// inside a word, or one character.
#[expect(
    clippy::indexing_slicing,
    reason = "every position is inside `chars`, which the scan never passes"
)]
fn word_end(chars: &[char], at: usize) -> usize {
    if !in_word(chars[at]) {
        return at + 1;
    }
    let mut index = at + 1;
    while index < chars.len() {
        let inside = in_word(chars[index])
            || (matches!(chars[index], '.' | ',')
                && chars.get(index + 1).copied().is_some_and(in_word));
        if !inside {
            break;
        }
        index += 1;
    }
    index
}

fn in_word(c: char) -> bool {
    c.is_alphanumeric() || is_combining_mark(c) || c == '_'
}

#[cfg(test)]
mod tests;
