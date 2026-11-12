//! Policy: the files the last site rendered, by their address, each with the node it was rendered from, which a file node of the new site tree keeps its file from, when the two nodes have the same subtree.

use std::collections::HashMap;

use super::file_renderer;
use crate::entities::{SiteAddress, SiteFile, SiteTreeNode};

/// The files of one part of the last site, the version's or those at the output folder's root.
pub(super) struct LastFiles<'a> {
    by_address: HashMap<SiteAddress, (SiteTreeNode<'a>, &'a SiteFile)>,
}

impl<'a> LastFiles<'a> {
    /// The files among `files` rendered from `nodes`, the file nodes of the last site tree they
    /// were rendered from; an asset, which is never rendered, is left out.
    pub(super) fn of(
        nodes: impl Iterator<Item = SiteTreeNode<'a>>,
        files: &'a [SiteFile],
    ) -> LastFiles<'a> {
        let files: HashMap<_, _> = files.iter().map(by_its_address).collect();
        let by_address = nodes
            .filter_map(|node| with_its_file(node, &files))
            .collect();
        LastFiles { by_address }
    }

    /// The file at `address` of the last site, shared, when its node has the same subtree as
    /// `node`.
    pub(super) fn file(&self, address: &SiteAddress, node: SiteTreeNode<'_>) -> Option<SiteFile> {
        let (last, file) = self.by_address.get(address)?;
        last.same_subtree_as(node).then(|| SiteFile::clone(file))
    }
}

/// The address of the file `node` renders, with `node` and that file among `files`, when
/// `node` renders a file there.
fn with_its_file<'a>(
    node: SiteTreeNode<'a>,
    files: &HashMap<&SiteAddress, &'a SiteFile>,
) -> Option<(SiteAddress, (SiteTreeNode<'a>, &'a SiteFile))> {
    let address = file_renderer(node)?.address();
    let file = *files.get(&address)?;
    Some((address, (node, file)))
}

fn by_its_address(file: &SiteFile) -> (&SiteAddress, &SiteFile) {
    (&file.address, file)
}
