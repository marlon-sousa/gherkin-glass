//! Entity: the page of step sentences, as its template shows it, every text in it as written or worded: its nodes and its content as the site stage builds it, with what is below each node.

use crate::entities::{FeaturePath, LinkAddress, Trail};

/// Every `String` is text, escaped when it is rendered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SentencesPage {
    pub(crate) trail: Trail,
    /// The `lang` of the page.
    pub(crate) language: String,
    pub(crate) title: String,
    pub(crate) heading: String,
    /// Shown only when there is no group.
    pub(crate) no_steps: String,
}

/// The heading of the sentences of one kind of step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GroupView {
    /// The kind of step, such as `Given`.
    pub(crate) heading: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SentenceView {
    pub(crate) text: String,
    /// The `lang` of the sentence's heading, when it differs from the page's.
    pub(crate) language: Option<String>,
    /// Such as `Used once.`
    pub(crate) uses_count: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct UseView {
    /// Relative to the site's root, where the page of step sentences is.
    pub(crate) address: LinkAddress,
    /// The `lang` of the feature's own text, when it differs from the page's.
    pub(crate) language: Option<String>,
    /// The link's words, around the block's heading and the file's path.
    pub(crate) link: Vec<Piece>,
    /// The item's words, around the link and the values; empty when the step has none.
    pub(crate) item: Vec<Piece>,
    pub(crate) heading: String,
    pub(crate) path: FeaturePath,
    /// As written.
    pub(crate) values: Vec<String>,
    /// Between two values, such as `, `.
    pub(crate) separator: String,
}

/// A part of a message: words of the catalogue, or the place of one of the
/// message's placeholders, by its number.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Piece {
    Words(String),
    Placeholder(usize),
}

impl Piece {
    pub(crate) fn words(&self) -> Option<&str> {
        match self {
            Piece::Words(words) => Some(words),
            Piece::Placeholder(_) => None,
        }
    }

    pub(crate) fn is_placeholder(&self, number: usize) -> bool {
        *self == Piece::Placeholder(number)
    }
}

/// The page with its groups, in the order of the kinds of step, as the site stage builds it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SentencesPageContent {
    pub(crate) page: SentencesPage,
    pub(crate) groups: Vec<GroupContent>,
}

/// A group with its sentences, sorted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GroupContent {
    pub(crate) group: GroupView,
    pub(crate) sentences: Vec<SentenceContent>,
}

/// A sentence with its uses, in the order of the paths, then of the lines.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SentenceContent {
    pub(crate) sentence: SentenceView,
    pub(crate) uses: Vec<UseView>,
}
