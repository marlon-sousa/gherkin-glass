//! Policy: the blocks of an outline's examples, as their tables and as the scenarios their rows make.

use gherkin::{Examples, Scenario};
use i18n_embed_fl::fl;

use super::block_building::{Builder, block_heading, description, worded_heading};
use super::heading_words::{TEXT_MARKS, heading, isolated, plain, worded_parts};
use super::step_views::{filled_step_contents, table};
use crate::entities::{
    Anchored, AnchoredScenario, ExampleScenario, ExamplesAsScenarios, ExamplesBlock, HeadingLevel,
};
use crate::policies::CATALOGUE;
use crate::policies::site::feature_pages::{ExampleRow, example_rows};

impl Builder<'_> {
    pub(super) fn examples(
        &self,
        anchored: &Anchored<'_, Examples>,
        level: HeadingLevel,
    ) -> ExamplesBlock {
        let examples = anchored.block;
        ExamplesBlock {
            heading: block_heading(
                level,
                &anchored.anchor,
                heading(&examples.keyword, examples.name.as_deref().unwrap_or("")),
                false,
            ),
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
                worded_parts(&words, &[isolated(outline_name(outline))]),
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
                worded_parts(&words, &[isolated(outline_name(outline)), values]),
                self.options.collapse_scenarios,
            ),
            steps: filled_step_contents(self.parsed, &outline.steps, row),
        }
    }
}

/// The outline's name, or its keyword as written when it has none.
fn outline_name(outline: &Scenario) -> &str {
    match outline.name.trim() {
        "" => outline.keyword.trim(),
        name => name,
    }
}
