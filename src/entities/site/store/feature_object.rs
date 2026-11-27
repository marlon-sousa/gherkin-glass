//! Entity: the object of one feature file's content, which depends on the file alone, never on a build's settings.

use crate::entities::{BlockText, ObjectBlock};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FeatureObject {
    /// As Gherkin names it, such as `en` or `pt`.
    pub(crate) language: String,
    /// The page's `lang`, made from the language.
    pub(crate) lang: String,
    /// The feature's own text; the feature has no anchor, since it is its page.
    pub(crate) text: BlockText,
    /// Its background, scenarios and rules, in file order.
    pub(crate) blocks: Vec<ObjectBlock>,
}
