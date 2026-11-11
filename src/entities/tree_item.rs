//! Entity: one item of the folder tree, with the items inside it, as `assets/tree.json` holds it for the pages' script.

use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct TreeItem {
    pub(crate) text: String,
    /// Relative to the site's root, percent-encoded as a link's, with the block's anchor after `#` for a rule or scenario.
    pub(crate) address: String,
    /// Only a feature's: the `lang` of its page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) lang: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) items: Vec<TreeItem>,
}
