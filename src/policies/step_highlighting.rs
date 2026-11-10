//! Policy: which parts of a step are highlighted, as docs/BEHAVIOUR.md, "Highlighting", says.

use std::collections::HashSet;

use gherkin::{Scenario, Step};

use crate::entities::{HighlightedStep, StepPart};

/// `scenario` is the scenario or outline that holds `step`, or `None` for a
/// background's step; only an outline's steps have placeholders.
pub fn highlight_step(step: &Step, scenario: Option<&Scenario>) -> HighlightedStep {
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

/// Rule 39: the end, past its closing quote, of a quoted parameter opening at `at`.
fn parameter_end(chars: &[char], at: usize) -> Option<usize> {
    if chars[at] != '"' || (at > 0 && chars[at - 1] == '\\') {
        return None;
    }
    let mut index = at + 1;
    while index < chars.len() {
        match chars[index] {
            '\\' => index += 2,
            '"' => return Some(index + 1),
            _ => index += 1,
        }
    }
    None
}

/// Rule 40: the end, past its `>`, of a placeholder naming one of `columns`, opening at `at`.
fn placeholder_end(chars: &[char], at: usize, columns: &HashSet<&str>) -> Option<usize> {
    if chars[at] != '<' {
        return None;
    }
    let close = at + 1 + chars[at + 1..].iter().position(|&c| c == '<' || c == '>')?;
    let name: String = chars[at + 1..close].iter().collect();
    (chars[close] == '>' && columns.contains(name.as_str())).then_some(close + 1)
}

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

/// Rule 41: the end of a number starting at `at`, which stands apart from any word.
fn number_end(chars: &[char], at: usize) -> Option<usize> {
    if at > 0 && in_word(chars[at - 1]) {
        return None;
    }
    let mut index = at;
    if chars[index] == '-' {
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
        if separator && chars.get(index + 1).is_some_and(char::is_ascii_digit) {
            index += 1;
        } else {
            break;
        }
    }
    (!chars.get(index).copied().is_some_and(in_word)).then_some(index)
}

/// The end of the text from `at` that is not highlighted: a whole word, with
/// any digits, points and commas inside it, so that no number is found
/// inside a word, or one character.
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
    c.is_alphanumeric() || c == '_'
}

#[cfg(test)]
mod tests {
    use gherkin::{Feature, GherkinEnv};

    use super::*;
    use StepPart::{Number, Parameter, Placeholder, Plain};

    /// The steps of the first scenario of a feature whose first scenario has
    /// `steps` and, when `columns` is not empty, examples with those columns.
    fn highlight(step: &str, columns: &[&str]) -> Vec<StepPart> {
        let mut source = format!("Feature: F\n  Scenario: S\n    Given {step}\n");
        if !columns.is_empty() {
            source += &format!(
                "    Examples:\n      | {} |\n      | {} |\n",
                columns.join(" | "),
                columns.iter().map(|_| "x").collect::<Vec<_>>().join(" | ")
            );
        }
        let feature = Feature::parse(source, GherkinEnv::default()).expect("parses");
        let scenario = &feature.scenarios[0];
        let highlighted = highlight_step(&scenario.steps[0], Some(scenario));
        assert_eq!(highlighted.keyword, "Given");
        assert_eq!(highlighted.text(), step, "the text is unchanged");
        highlighted.parts
    }

    fn plain(text: &str) -> StepPart {
        Plain(text.into())
    }

    #[test]
    fn plain_text_is_one_part() {
        assert_eq!(highlight("a customer", &[]), [plain("a customer")]);
    }

    #[test]
    fn a_quoted_parameter_keeps_its_quotes() {
        assert_eq!(
            highlight(r#"the user types "hello 42" twice"#, &[]),
            [
                plain("the user types "),
                Parameter(vec![plain("\"hello 42\"")]),
                plain(" twice"),
            ]
        );
        assert_eq!(
            highlight(r#"an empty "" and "a \"b\"" end"#, &[]),
            [
                plain("an empty "),
                Parameter(vec![plain("\"\"")]),
                plain(" and "),
                Parameter(vec![plain(r#""a \"b\"""#)]),
                plain(" end"),
            ]
        );
    }

    #[test]
    fn an_unclosed_or_escaped_quote_is_plain_and_single_quotes_are_not_parameters() {
        assert_eq!(
            highlight(r#"a "lone quote"#, &[]),
            [plain(r#"a "lone quote"#)]
        );
        assert_eq!(highlight(r#"a \"b" c"#, &[]), [plain(r#"a \"b" c"#)]);
        assert_eq!(
            highlight("the user's 'name'", &[]),
            [plain("the user's 'name'")]
        );
    }

    #[test]
    fn numbers_stand_apart_from_words() {
        assert_eq!(
            highlight("pay 10, -3.5 and 1.000,50 or $7 on 2026-11-10.", &[]),
            [
                plain("pay "),
                Number("10".into()),
                plain(", "),
                Number("-3.5".into()),
                plain(" and "),
                Number("1.000,50".into()),
                plain(" or $"),
                Number("7".into()),
                plain(" on "),
                Number("2026".into()),
                plain("-"),
                Number("11".into()),
                plain("-"),
                Number("10".into()),
                plain("."),
            ]
        );
        assert_eq!(
            highlight("R2D2 is 3rd in v1.2 and x_1, item-4", &[]),
            [
                plain("R2D2 is 3rd in v1.2 and x_1, item-"),
                Number("4".into()),
            ]
        );
        assert_eq!(highlight("٣ apples", &[]), [plain("٣ apples")]);
    }

    #[test]
    fn a_placeholder_names_a_column_of_the_outline_s_examples() {
        assert_eq!(
            highlight(
                "<amount> of <currency> in <unknown> < 5 >",
                &["amount", "currency"]
            ),
            [
                Placeholder("<amount>".into()),
                plain(" of "),
                Placeholder("<currency>".into()),
                plain(" in <unknown> < "),
                Number("5".into()),
                plain(" >"),
            ]
        );
        assert_eq!(
            highlight("<a<amount>>", &["amount"]),
            [plain("<a"), Placeholder("<amount>".into()), plain(">")]
        );
    }

    #[test]
    fn a_placeholder_inside_a_parameter_is_highlighted_and_a_number_is_not() {
        assert_eq!(
            highlight(r#"types "<name> 3" now"#, &["name"]),
            [
                plain("types "),
                Parameter(vec![
                    plain("\""),
                    Placeholder("<name>".into()),
                    plain(" 3\""),
                ]),
                plain(" now"),
            ]
        );
    }

    #[test]
    fn a_step_outside_an_outline_has_no_placeholders() {
        let feature = Feature::parse(
            "Feature: F\n  Background:\n    Given <amount>\n  Scenario: S\n    Given <amount>\n",
            GherkinEnv::default(),
        )
        .expect("parses");
        let background = &feature.background.as_ref().expect("a background").steps[0];
        assert_eq!(highlight_step(background, None).parts, [plain("<amount>")]);
        let scenario = &feature.scenarios[0];
        assert_eq!(
            highlight_step(&scenario.steps[0], Some(scenario)).parts,
            [plain("<amount>")]
        );
    }
}
