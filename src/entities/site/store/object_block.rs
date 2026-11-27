//! Entity: a block of a feature file as its object holds it, below the feature: a background, a rule, a scenario or a scenario outline, with what each kind holds, and an outline's examples blocks with their runs.

use crate::entities::{BlockKind, ObjectStep};

/// What every block, the feature's included, says of itself as the file writes it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BlockText {
    /// As written, without its colon.
    pub(crate) keyword: String,
    /// Whether the keyword is another word than the kind's.
    pub(crate) note: bool,
    /// Empty for a block with no name.
    pub(crate) name: String,
    /// Empty for none.
    pub(crate) description: String,
    /// Each with its `@`; none for a background.
    pub(crate) tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ObjectBlock {
    pub(crate) text: BlockText,
    pub(crate) anchor: String,
    pub(crate) content: BlockContent,
}

/// What a block holds besides its text, by its kind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum BlockContent {
    Background {
        steps: Vec<ObjectStep>,
    },
    /// Its background and scenarios, in file order.
    Rule {
        blocks: Vec<ObjectBlock>,
    },
    Scenario {
        steps: Vec<ObjectStep>,
    },
    Outline {
        steps: Vec<ObjectStep>,
        examples: Vec<ObjectExamples>,
    },
}

impl ObjectBlock {
    pub(crate) fn kind(&self) -> BlockKind {
        match self.content {
            BlockContent::Background { .. } => BlockKind::Background,
            BlockContent::Rule { .. } => BlockKind::Rule,
            BlockContent::Scenario { .. } => BlockKind::Scenario,
            BlockContent::Outline { .. } => BlockKind::ScenarioOutline,
        }
    }
}

/// An examples block of an outline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ObjectExamples {
    pub(crate) text: BlockText,
    pub(crate) anchor: String,
    /// Its place among its outline's examples blocks, from 1.
    pub(crate) place: usize,
    /// Its rows, its header first; `None` without a table.
    pub(crate) table: Option<Vec<Vec<String>>>,
    /// One for each row of its table but the header.
    pub(crate) runs: Vec<ObjectRun>,
}

/// The scenario one row of an outline's examples makes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ObjectRun {
    pub(crate) anchor: String,
    /// The outline's steps with the row's values put in their places.
    pub(crate) steps: Vec<ObjectStep>,
}
