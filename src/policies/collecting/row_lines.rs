//! Policy: the line of each row of an outline's examples, which the `gherkin` crate does not give, found in the feature file's text.

use gherkin::{Feature, Table};

/// The line of each row of every examples block's table but its header, counted from 1,
/// outline by outline in file order, the feature's scenarios first and then each rule's, as
/// the runs of an outline are numbered.
pub(crate) fn examples_row_lines(feature: &Feature, source: &str) -> Vec<usize> {
    let lines: Vec<&str> = source.lines().collect();
    feature
        .scenarios
        .iter()
        .chain(feature.rules.iter().flat_map(|rule| &rule.scenarios))
        .flat_map(|scenario| &scenario.examples)
        .filter_map(|examples| examples.table.as_ref())
        .flat_map(|table| table_row_lines(&lines, table).into_iter().skip(1))
        .collect()
}

/// The line of each row of `table`, its header first, among `lines`: a table's rows are its
/// first lines from its own on that are neither blank nor a comment.
fn table_row_lines(lines: &[&str], table: &Table) -> Vec<usize> {
    lines
        .iter()
        .zip(1..)
        .skip(table.position.line.saturating_sub(1))
        .filter(|(line, _)| is_row(line))
        .take(table.rows.len())
        .map(|(_, number)| number)
        .collect()
}

fn is_row(line: &str) -> bool {
    let text = line.trim_start();
    !text.is_empty() && !text.starts_with('#')
}

#[cfg(test)]
mod tests {
    use gherkin::GherkinEnv;

    use super::*;

    fn row_lines(source: &str) -> Vec<usize> {
        let feature = Feature::parse(source, GherkinEnv::default()).expect("parses");
        examples_row_lines(&feature, source)
    }

    // Rule 265: a row's line is its own, past the comments and blank lines between rows.
    #[test]
    fn a_row_s_line_is_its_own_past_comments_and_blank_lines() {
        let source = "Feature: F\n  Scenario Outline: O\n    Given <a>\n    Examples:\n      | a |\n      | 1 |\n      # A comment.\n\n      | 2 |\n";
        assert_eq!(row_lines(source), [6, 9]);
    }

    // Rule 33: the rows are those of every examples block, header left out, the feature's
    // outlines first and then each rule's.
    #[test]
    fn rows_are_counted_outline_by_outline_without_headers() {
        let source = "Feature: F\n  Scenario Outline: O\n    Given <a>\n    Examples:\n      | a |\n      | 1 |\n    Examples:\n      | a |\n    Examples:\n      | a |\n      | 2 |\n  Rule: R\n    Scenario Outline: P\n      Given <b>\n      Examples:\n        | b |\n        | 3 |\n";
        assert_eq!(row_lines(source), [6, 11, 17]);
    }

    #[test]
    fn a_feature_without_examples_has_no_rows() {
        assert!(row_lines("Feature: F\n  Scenario: S\n    Given a\n      | x |\n").is_empty());
    }
}
