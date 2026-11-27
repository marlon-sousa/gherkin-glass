//! Policy: the steps of a feature's object, each with what ghg works out of it, and the runs an outline's rows make, whose steps have the row's values in place of their placeholders.

use gherkin::{Scenario, Step, Table};

use crate::entities::{DocString, ObjectRun, ObjectStep, ParsedFeature};
use crate::policies::site::{ExampleRow, highlight_step, sentence_of, step_kind};

/// Builds a feature's steps, which need its doc strings' media types.
pub(super) struct ObjectSteps<'a> {
    pub(super) parsed: &'a ParsedFeature,
}

impl ObjectSteps<'_> {
    /// `scenario` holds the steps, or is `None` for a background's.
    pub(super) fn steps(&self, steps: &[Step], scenario: Option<&Scenario>) -> Vec<ObjectStep> {
        let columns = columns_of(scenario);
        steps
            .iter()
            .map(|step| self.step(step, scenario, &columns))
            .collect()
    }

    /// The scenario `row` of `outline`'s examples makes, at `anchor`.
    pub(super) fn run(&self, outline: &Scenario, row: &ExampleRow<'_>, anchor: &str) -> ObjectRun {
        ObjectRun {
            anchor: anchor.to_string(),
            steps: outline
                .steps
                .iter()
                .map(|step| self.filled_step(step, row))
                .collect(),
        }
    }

    fn step(&self, step: &Step, scenario: Option<&Scenario>, columns: &[&str]) -> ObjectStep {
        let (sentence, slots) = sentence_of(&step.value, columns);
        ObjectStep {
            keyword: step.keyword.clone(),
            text: step.value.clone(),
            kind: step_kind(step.ty),
            sentence,
            slots,
            parts: highlight_step(step, scenario).parts,
            table: step.table.as_ref().map(rows_of),
            doc_string: self.doc_string(step),
        }
    }

    fn doc_string(&self, step: &Step) -> Option<DocString> {
        Some(DocString {
            media_type: self.parsed.media_type(step).map(str::to_string),
            content: step.docstring.clone()?,
        })
    }

    /// The outline's step with `row`'s values in place of its placeholders, in its text, its
    /// table and its doc string, so that none of them is a placeholder any more.
    fn filled_step(&self, step: &Step, row: &ExampleRow<'_>) -> ObjectStep {
        let filled = Step {
            value: row.fill(&step.value),
            ..step.clone()
        };
        ObjectStep {
            table: step.table.as_ref().map(|table| filled_rows(table, row)),
            doc_string: self
                .doc_string(step)
                .map(|doc_string| filled_doc_string(doc_string, row)),
            ..self.step(&filled, None, &[])
        }
    }
}

fn rows_of(table: &Table) -> Vec<Vec<String>> {
    table.rows.clone()
}

fn filled_doc_string(doc_string: DocString, row: &ExampleRow<'_>) -> DocString {
    DocString {
        media_type: doc_string.media_type.map(|text| row.fill(&text)),
        content: row.fill(&doc_string.content),
    }
}

fn filled_rows(table: &Table, row: &ExampleRow<'_>) -> Vec<Vec<String>> {
    table
        .rows
        .iter()
        .map(|cells| row.fill_cells(cells))
        .collect()
}

/// The names in the header rows of an outline's examples; none for a plain scenario or a
/// background.
fn columns_of(scenario: Option<&Scenario>) -> Vec<&str> {
    scenario
        .into_iter()
        .flat_map(|scenario| &scenario.examples)
        .filter_map(|examples| examples.table.as_ref())
        .filter_map(|table| table.rows.first())
        .flatten()
        .map(String::as_str)
        .collect()
}
