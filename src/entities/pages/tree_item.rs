//! Entity: one item of the folder tree, with the items inside it, which `assets/tree.json` holds for the pages' script.

use crate::entities::LinkAddress;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TreeItem {
    pub(crate) text: String,
    /// Relative to the site's root, with the block's anchor for a rule or scenario.
    pub(crate) address: LinkAddress,
    /// Only a feature's: the `lang` of its page.
    pub(crate) lang: Option<String>,
    pub(crate) items: Vec<TreeItem>,
}
