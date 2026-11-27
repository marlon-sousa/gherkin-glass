//! Entity: the object of one feature file's content, which depends on the file alone, never on a build's settings.

use crate::entities::ObjectBlock;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FeatureObject {
    /// As Gherkin names it, such as `en` or `pt`.
    pub(crate) language: String,
    /// The page's `lang`, made from the language.
    pub(crate) lang: String,
    pub(crate) feature: ObjectBlock,
}
