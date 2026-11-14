//! Policy: the blocks of an outline's examples, each with its table and the runs its rows make.

use gherkin::{Examples, Scenario, Table};
use i18n_embed_fl::fl;

use super::block_building::{Builder, description, worded_heading};
use super::heading_words::{
    Kind, TEXT_MARKS, examples_heading, isolated, kind_alone, plain, source_note, worded_parts,
};
use super::step_views::{filled_step_contents, table};
use crate::entities::{
    Anchored, AnchoredScenario, ExampleScenario, ExamplesBlock, ExamplesContent, ExamplesRuns,
    HeadingLevel, HeadingPart, plain_parts,
};
use crate::policies::CATALOGUE;
use crate::policies::site::feature_pages::{ExampleRow, block_rows};

impl Builder<'_> {
    /// The block at `place` among its outline's, from 1.
    fn examples(&self, anchored: &Anchored<'_, Examples>, place: usize) -> ExamplesBlock {
        let examples = anchored.block;
        let label = examples_heading(examples.name.as_deref().unwrap_or(""), place);
        let has_rows = examples.table.as_ref().is_some_and(has_rows);
        ExamplesBlock {
            anchor: anchored.anchor.clone(),
            label: if has_rows {
                label
            } else {
                let words = fl!(CATALOGUE, "nothing-to-run", heading = TEXT_MARKS[0]);
                worded_parts(&words, &[label])
            },
            source_note: source_note(Kind::Examples, &examples.keyword),
            tags: self.tags(&examples.tags),
            description: description(examples.description.as_deref()),
            table: examples.table.as_ref().map(table),
        }
    }

    /// The examples blocks of `anchored`, each with its runs at `level`, the
    /// level below the outline's, when it has a row.
    pub(super) fn examples_contents(
        &self,
        anchored: &AnchoredScenario<'_>,
        level: HeadingLevel,
    ) -> Vec<ExamplesContent> {
        let mut row_anchors = anchored.rows.iter().map(String::as_str);
        anchored
            .examples
            .iter()
            .zip(1..)
            .map(|(examples, place)| {
                self.examples_content(anchored.scenario, examples, place, &mut row_anchors, level)
            })
            .collect()
    }

    /// The examples block at `place`, with its runs, which take their
    /// anchors from the outline's `row_anchors` still unused.
    fn examples_content<'r>(
        &self,
        outline: &Scenario,
        examples: &Anchored<'_, Examples>,
        place: usize,
        row_anchors: &mut impl Iterator<Item = &'r str>,
        level: HeadingLevel,
    ) -> ExamplesContent {
        let rows = block_rows(examples.block);
        let anchors: Vec<&str> = row_anchors.take(rows.len()).collect();
        ExamplesContent {
            block: self.examples(examples, place),
            runs: self.runs(outline, examples, &rows, &anchors, level),
        }
    }

    /// The runs of `examples`' `rows`, each with its anchor; `None` when it
    /// has no row.
    fn runs(
        &self,
        outline: &Scenario,
        examples: &Anchored<'_, Examples>,
        rows: &[ExampleRow<'_>],
        anchors: &[&str],
        level: HeadingLevel,
    ) -> Option<ExamplesRuns> {
        if rows.is_empty() {
            return None;
        }
        let scenarios = rows
            .iter()
            .zip(anchors)
            .map(|(row, anchor)| self.example_scenario(outline, anchor, row, level.below()))
            .collect();
        Some(ExamplesRuns {
            heading: worded_heading(
                level,
                &format!("{}_runs", examples.anchor),
                plain_parts(&fl!(CATALOGUE, "runs-one-per-row")),
                true,
            ),
            scenarios,
        })
    }

    fn example_scenario(
        &self,
        outline: &Scenario,
        anchor: &str,
        row: &ExampleRow<'_>,
        level: HeadingLevel,
    ) -> ExampleScenario {
        let value_words = fl!(
            CATALOGUE,
            "example-value",
            column = TEXT_MARKS[0],
            value = TEXT_MARKS[1]
        );
        let separator = fl!(CATALOGUE, "example-values-separator");
        let values = row
            .cells()
            .map(|(column, value)| worded_parts(&value_words, &[isolated(column), isolated(value)]))
            .collect::<Vec<_>>()
            .join(&plain(&separator));
        let words = fl!(
            CATALOGUE,
            "example-scenario-name",
            name = TEXT_MARKS[0],
            values = TEXT_MARKS[1]
        );
        ExampleScenario {
            heading: worded_heading(
                level,
                anchor,
                worded_parts(&words, &[outline_name(outline), values]),
                self.options.collapse_scenarios,
            ),
            steps: filled_step_contents(self.parsed, &outline.steps, row),
        }
    }
}

/// The outline's name, isolated, or its kind in gherkin-glass's words when
/// it has none, whatever keyword the file writes.
fn outline_name(outline: &Scenario) -> Vec<HeadingPart> {
    match outline.name.trim() {
        "" => kind_alone(Kind::ScenarioOutline),
        name => isolated(name),
    }
}

/// Whether `table` has a row below its header.
fn has_rows(table: &Table) -> bool {
    table.rows.get(1).is_some()
}
