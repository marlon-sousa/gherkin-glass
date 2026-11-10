//! Policy: the rows of an outline's examples, and the placeholders each row fills.

use gherkin::Scenario;

/// One row of an outline's examples, with the header of its table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ExampleRow<'a> {
    pub(crate) header: &'a [String],
    pub(crate) values: &'a [String],
}

/// Every row of `outline`'s examples blocks, in file order, without their
/// header rows: the `n`th is the row the anchor rule numbers `n + 1`.
pub(crate) fn example_rows(outline: &Scenario) -> Vec<ExampleRow<'_>> {
    outline
        .examples
        .iter()
        .filter_map(|examples| examples.table.as_ref())
        .flat_map(|table| {
            let (header, rows) = table
                .rows
                .split_first()
                .map_or((&[][..], &[][..]), |(h, r)| (&h[..], r));
            rows.iter().map(move |values| ExampleRow { header, values })
        })
        .collect()
}

impl ExampleRow<'_> {
    /// Each column's name and its value in this row, in the table's order.
    pub(crate) fn cells(&self) -> impl Iterator<Item = (&str, &str)> {
        self.header
            .iter()
            .zip(self.values)
            .map(|(column, value)| (column.as_str(), value.as_str()))
    }

    /// `text` with every `<column>` replaced by this row's value, the first
    /// column of that name when several share it. A placeholder no column
    /// names stays as written, and a value put in is not searched again.
    pub(crate) fn fill(&self, text: &str) -> String {
        let mut filled = String::with_capacity(text.len());
        let mut rest = text;
        while let Some(open) = rest.find('<') {
            filled.push_str(&rest[..open]);
            let after = &rest[open + 1..];
            let value = after.find('>').and_then(|close| {
                let name = &after[..close];
                self.value(name).map(|value| (value, close))
            });
            match value {
                Some((value, close)) => {
                    filled.push_str(value);
                    rest = &after[close + 1..];
                }
                None => {
                    filled.push('<');
                    rest = after;
                }
            }
        }
        filled.push_str(rest);
        filled
    }

    fn value(&self, column: &str) -> Option<&str> {
        if column.is_empty() {
            return None;
        }
        self.cells()
            .find(|(name, _)| *name == column)
            .map(|(_, value)| value)
    }
}

#[cfg(test)]
mod tests {
    use gherkin::{Feature, GherkinEnv};

    use super::*;

    fn outline(text: &str) -> Scenario {
        let feature = Feature::parse(text, GherkinEnv::default()).expect("parses");
        feature.scenarios[0].clone()
    }

    #[test]
    fn rows_run_across_examples_blocks_without_their_headers() {
        let outline = outline(
            "Feature: F\n\
             \x20 Scenario Outline: O\n\
             \x20   Given <a>\n\
             \x20   Examples:\n\
             \x20     | a |\n\
             \x20     | 1 |\n\
             \x20     | 2 |\n\
             \x20   Examples:\n\
             \x20     | a |\n\
             \x20   Examples:\n\
             \x20   Examples:\n\
             \x20     | b | a |\n\
             \x20     | 3 | 4 |\n",
        );
        let found = example_rows(&outline);
        let rows: Vec<Vec<(&str, &str)>> = found.iter().map(|row| row.cells().collect()).collect();
        assert_eq!(
            rows,
            [
                vec![("a", "1")],
                vec![("a", "2")],
                vec![("b", "3"), ("a", "4")]
            ]
        );
    }

    #[test]
    fn a_placeholder_takes_the_first_column_of_its_name_and_an_unknown_one_stays() {
        let header = ["x".to_string(), "y".to_string(), "x".to_string()];
        let values = ["1".to_string(), "<x>".to_string(), "3".to_string()];
        let row = ExampleRow {
            header: &header,
            values: &values,
        };
        assert_eq!(row.fill("<x> and <y>, <z> < <x>"), "1 and <x>, <z> < 1");
        assert_eq!(row.fill("<x"), "<x");
        assert_eq!(row.fill("<<x>>"), "<1>");
        let header = [String::new()];
        let values = ["1".to_string()];
        let row = ExampleRow {
            header: &header,
            values: &values,
        };
        assert_eq!(row.fill("<> stays"), "<> stays");
    }
}
