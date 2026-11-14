//! Policy: the blocks of an outline's examples, as their tables and as the scenarios their rows make.

use gherkin::{Examples, Scenario, Table};
use i18n_embed_fl::fl;

use super::block_building::{Builder, description, worded_heading};
use super::heading_words::{Kind, TEXT_MARKS, heading, isolated, kind_alone, plain, worded_parts};
use super::step_views::{filled_step_contents, table};
use crate::entities::{
    Anchored, AnchoredScenario, ExampleScenario, ExamplesAsScenarios, ExamplesBlock, HeadingLevel,
    HeadingPart,
};
use crate::policies::CATALOGUE;
use crate::policies::site::feature_pages::{ExampleRow, example_rows};

impl Builder<'_> {
    pub(super) fn examples(&self, anchored: &Anchored<'_, Examples>) -> ExamplesBlock {
        let examples = anchored.block;
        let label = heading(
            Kind::Examples,
            &examples.keyword,
            examples.name.as_deref().unwrap_or(""),
        );
        let has_rows = examples.table.as_ref().is_some_and(has_rows);
        ExamplesBlock {
            anchor: anchored.anchor.clone(),
            label: if has_rows {
                label
            } else {
                let words = fl!(CATALOGUE, "empty-examples", heading = TEXT_MARKS[0]);
                worded_parts(&words, &[label])
            },
            has_rows,
            tags: self.tags(&examples.tags),
            description: description(examples.description.as_deref()),
            table: examples.table.as_ref().map(table),
        }
    }

    /// `None` for a scenario, or an outline whose examples have no row.
    pub(super) fn examples_as_scenarios(
        &self,
        anchored: &AnchoredScenario<'_>,
        level: HeadingLevel,
    ) -> Option<ExamplesAsScenarios> {
        let outline = anchored.scenario;
        let rows = example_rows(outline);
        if rows.is_empty() {
            return None;
        }
        let outline_anchor = &anchored.anchor;
        let words = fl!(CATALOGUE, "examples-as-scenarios", name = TEXT_MARKS[0]);
        let scenarios = rows
            .iter()
            .zip(&anchored.rows)
            .map(|(row, anchor)| self.example_scenario(outline, anchor, row, level.below()))
            .collect();
        Some(ExamplesAsScenarios {
            heading: worded_heading(
                level,
                &format!("{outline_anchor}_scenarios"),
                worded_parts(&words, &[outline_name(outline)]),
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
