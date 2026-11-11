//! Policy: the blocks of an outline's examples, as their tables and as the scenarios their rows make.

use gherkin::{Examples, Scenario};
use i18n_embed_fl::fl;

use super::block_building::{Builder, block, description, worded_block};
use super::heading_words::{TEXT_MARKS, heading, isolated, plain, worded_parts};
use super::step_views::{filled_step_views, table};
use crate::entities::{Anchored, AnchoredScenario, Block, BlockKind, HeadingLevel};
use crate::policies::{CATALOGUE, ExampleRow, example_rows};

impl Builder<'_> {
    pub(super) fn examples(&self, anchored: &Anchored<'_, Examples>, level: HeadingLevel) -> Block {
        let examples = anchored.block;
        Block {
            tags: self.tags(&examples.tags),
            description: description(examples.description.as_deref()),
            table: examples.table.as_ref().map(table),
            ..block(
                BlockKind::Examples,
                level,
                &anchored.anchor,
                heading(&examples.keyword, examples.name.as_deref().unwrap_or("")),
                false,
            )
        }
    }

    pub(super) fn examples_as_scenarios(
        &self,
        anchored: &AnchoredScenario<'_>,
        level: HeadingLevel,
    ) -> Option<Block> {
        let outline = anchored.scenario;
        let rows = example_rows(outline);
        if rows.is_empty() {
            return None;
        }
        let outline_anchor = &anchored.anchor;
        let words = fl!(CATALOGUE, "examples-as-scenarios", name = TEXT_MARKS[0]);
        let blocks = rows
            .iter()
            .zip(&anchored.rows)
            .map(|(row, anchor)| self.example_scenario(outline, anchor, row, level.below()))
            .collect();
        Some(Block {
            blocks,
            ..worded_block(
                BlockKind::ExamplesAsScenarios,
                level,
                &format!("{outline_anchor}_scenarios"),
                worded_parts(&words, &[isolated(outline_name(outline))]),
                true,
            )
        })
    }

    fn example_scenario(
        &self,
        outline: &Scenario,
        anchor: &str,
        row: &ExampleRow<'_>,
        level: HeadingLevel,
    ) -> Block {
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
        Block {
            steps: filled_step_views(self.parsed, &outline.steps, row),
            ..worded_block(
                BlockKind::Scenario,
                level,
                anchor,
                worded_parts(&words, &[isolated(outline_name(outline)), values]),
                self.options.collapse_scenarios,
            )
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
