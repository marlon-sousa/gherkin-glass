//! Entity: a block of a feature file as its object holds it, with the blocks inside it, its steps, and an outline's examples with their runs.

use crate::entities::{BlockKind, ObjectStep};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ObjectBlock {
    pub(crate) kind: BlockKind,
    /// As written, without its colon.
    pub(crate) keyword: String,
    /// Whether the keyword is another word than the kind's.
    pub(crate) note: bool,
    /// Empty for a block with no name.
    pub(crate) name: String,
    /// Empty for none.
    pub(crate) description: String,
    /// Each with its `@`.
    pub(crate) tags: Vec<String>,
    /// Empty for the feature, which is its page.
    pub(crate) anchor: String,
    /// A feature's background, scenarios and rules, or a rule's background and scenarios, in
    /// file order; empty for every other block.
    pub(crate) blocks: Vec<ObjectBlock>,
    pub(crate) steps: Vec<ObjectStep>,
    /// An outline's examples blocks.
    pub(crate) examples: Vec<ObjectBlock>,
    /// An examples block's place among its outline's, from 1; `None` for every other block.
    pub(crate) place: Option<usize>,
    /// An examples block's rows, its header first; `None` without a table, and for every
    /// other block.
    pub(crate) table: Option<Vec<Vec<String>>>,
    /// One for each row of an examples block's table but the header.
    pub(crate) runs: Vec<ObjectRun>,
}

/// The scenario one row of an outline's examples makes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ObjectRun {
    pub(crate) anchor: String,
    /// The outline's steps with the row's values put in their places.
    pub(crate) steps: Vec<ObjectStep>,
}
