//! Policy: every file of a site rendered from its tree, each file node through the rendering trait unless the last site rendered holds its file, and what the build has to say of them.

mod last_files;

use std::sync::Arc;

use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};

use super::node_rendering::{NodeOf, RenderFile};
use super::page_layout::SitePieces;
use super::site_data::{SEARCH_DOCUMENTS, asset};
use crate::entities::{
    BuiltSite, PageOptions, RenderedSite, SiteAssets, SiteFile, SiteFiles, SiteNode, SiteTree,
    SiteTreeNode,
};
use last_files::LastFiles;

/// Above this size, search can be slow to start. MiniSearch 7.2.0, in Node 22
/// on a four-core server, took about 70 ms to index each million bytes.
const LARGE_SEARCH_DOCUMENTS: usize = 5_000_000;

/// Every file of `tree`, in its order, in the site that `options` make: the
/// version's, and those beside the versions, at the output folder's root.
/// A file whose node has the same subtree as the node of the file at the same
/// address in `last`, the site the last build rendered, is kept from it rather
/// than rendered again.
pub(crate) fn render_site(
    tree: &SiteTree,
    last: Option<&RenderedSite>,
    options: &PageOptions,
) -> SiteFiles {
    let site = SitePieces::of(tree.frame());
    let last_files = last.map(|last| LastFiles::of(last.tree.files(), &last.files.files));
    let last_root_files =
        last.map(|last| LastFiles::of(last.tree.root_files(), &last.files.root_files));
    let files = render_files(tree.files(), last_files.as_ref(), &site);
    let root_files = render_files(tree.root_files(), last_root_files.as_ref(), &site);
    let built = BuiltSite {
        large_search_documents: files
            .iter()
            .find(is_search_documents)
            .map(|file| file.contents.len())
            .filter(is_large),
        untitled: options.title.is_none(),
    };
    SiteFiles {
        files,
        root_files,
        built,
    }
}

/// The file of each of `nodes`, in their order, rendered in parallel, unless
/// it is kept from `last`.
fn render_files<'a>(
    nodes: impl Iterator<Item = SiteTreeNode<'a>>,
    last: Option<&LastFiles<'_>>,
    site: &SitePieces<'_>,
) -> Vec<SiteFile> {
    let nodes: Vec<_> = nodes.collect();
    // Collecting into a `Vec` keeps the files' order, whatever order the threads finish in.
    nodes
        .par_iter()
        .filter_map(|node| render_file(*node, last, site))
        .collect()
}

/// The file of `node`, kept from `last` when its node there is unchanged; `None` for the
/// site, a version, a folder and a part of a page, which make no file of their
/// own, and for an asset that is not there.
fn render_file(
    node: SiteTreeNode<'_>,
    last: Option<&LastFiles<'_>>,
    site: &SitePieces<'_>,
) -> Option<SiteFile> {
    if let SiteNode::Asset(name) = node.value() {
        return asset_file(name);
    }
    let renderer = file_renderer(node)?;
    let address = renderer.address();
    let kept_file = last.and_then(|last| last.file(&address, node));
    Some(kept_file.unwrap_or_else(|| SiteFile::text(address, renderer.render_file(site))))
}

/// What renders the file of `node`; `None` for an asset, which is not
/// rendered, and for the site, a version, a folder and a part of a page.
fn file_renderer<'a>(node: SiteTreeNode<'a>) -> Option<Box<dyn RenderFile + 'a>> {
    match node.value() {
        SiteNode::FeaturePage(_) => node.feature_page().map(boxed),
        SiteNode::FolderPage(_) => node.folder_page().map(boxed),
        SiteNode::SentencesPage(value) => Some(boxed(NodeOf { value, node })),
        SiteNode::FolderTree(items) => Some(boxed(items.as_slice())),
        SiteNode::TagIndex(value) => Some(boxed(NodeOf { value, node })),
        SiteNode::TagPage(value) => Some(boxed(NodeOf { value, node })),
        SiteNode::SearchDocuments(documents) => Some(boxed(documents.as_slice())),
        SiteNode::VersionsPage(value) => Some(boxed(NodeOf { value, node })),
        SiteNode::Asset(_)
        | SiteNode::Site(_)
        | SiteNode::Version(_)
        | SiteNode::KeptVersion(_)
        | SiteNode::InputFolder
        | SiteNode::Folder(_)
        | SiteNode::Background(_)
        | SiteNode::Rule(_)
        | SiteNode::Scenario(_)
        | SiteNode::Examples(_)
        | SiteNode::ExamplesAsScenarios(_)
        | SiteNode::ExampleScenario(_)
        | SiteNode::Step(_)
        | SiteNode::StepTable(_)
        | SiteNode::DocString(_)
        | SiteNode::TagIndexEntry(_)
        | SiteNode::TagFeature(_)
        | SiteNode::TagScenario(_)
        | SiteNode::SentencesGroup(_)
        | SiteNode::Sentence(_)
        | SiteNode::SentenceUse(_)
        | SiteNode::SubfolderLink(_)
        | SiteNode::FeatureEntry(_)
        | SiteNode::VersionsList(_)
        | SiteNode::VersionEntry(_) => None,
    }
}

fn boxed<'a>(file: impl RenderFile + 'a) -> Box<dyn RenderFile + 'a> {
    Box::new(file)
}

/// The asset named `name`, as a file of the site.
fn asset_file(name: &str) -> Option<SiteFile> {
    let file = SiteAssets::get(name)?;
    Some(SiteFile {
        address: asset(name),
        contents: Arc::from(file.data),
    })
}

fn is_search_documents(file: &&SiteFile) -> bool {
    file.address == asset(SEARCH_DOCUMENTS)
}

/// Whether search documents of `size` bytes can be slow to start.
fn is_large(size: &usize) -> bool {
    *size > LARGE_SEARCH_DOCUMENTS
}

#[cfg(test)]
mod tests;
