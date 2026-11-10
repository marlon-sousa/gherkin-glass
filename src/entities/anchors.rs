//! Entity: the anchors of one feature's page, one for each block and each row of an outline's examples.

use std::collections::BTreeMap;

use gherkin::{Background, Examples, Rule, Scenario};

/// Each block's anchor is found by the byte offset at which the block starts,
/// so only the blocks of the feature the anchors were made from have one.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Anchors {
    blocks: BTreeMap<usize, String>,
    /// Keyed by the outline's offset and the row's number, counted from 1.
    rows: BTreeMap<(usize, usize), String>,
}

impl Anchors {
    pub(crate) fn insert_block(&mut self, start: usize, anchor: String) {
        self.blocks.insert(start, anchor);
    }

    pub(crate) fn insert_row(&mut self, outline_start: usize, row: usize, anchor: String) {
        self.rows.insert((outline_start, row), anchor);
    }

    pub fn background(&self, background: &Background) -> Option<&str> {
        self.block(background.span.start)
    }

    pub fn rule(&self, rule: &Rule) -> Option<&str> {
        self.block(rule.span.start)
    }

    /// A scenario's, or a scenario outline's.
    pub fn scenario(&self, scenario: &Scenario) -> Option<&str> {
        self.block(scenario.span.start)
    }

    pub fn examples(&self, examples: &Examples) -> Option<&str> {
        self.block(examples.span.start)
    }

    /// The scenario `outline` produces from its `row`th row, counted from 1
    /// across its examples blocks, without their header rows.
    pub fn row(&self, outline: &Scenario, row: usize) -> Option<&str> {
        self.rows
            .get(&(outline.span.start, row))
            .map(String::as_str)
    }

    fn block(&self, start: usize) -> Option<&str> {
        self.blocks.get(&start).map(String::as_str)
    }
}
