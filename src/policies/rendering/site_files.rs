//! Policy: every file of a site rendered from its tree, each file node through the rendering trait, and what the build has to say of them.

use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};

use super::node_rendering::RenderFile;
use super::site_data::{SEARCH_DOCUMENTS, asset};
use crate::entities::{
    BuiltSite, FileNode, PageOptions, SiteAssets, SiteFile, SiteFiles, SiteTree,
};

/// Above this size, search can be slow to start. MiniSearch 7.2.0, in Node 22
/// on a four-core server, took about 70 ms to index each million bytes.
const LARGE_SEARCH_DOCUMENTS: usize = 5_000_000;

/// Every file of `tree`, in its order, in the site that `options` make.
pub(crate) fn render_site(tree: &SiteTree, options: &PageOptions) -> SiteFiles {
    let files: Vec<_> = tree.0.par_iter().filter_map(render_file).collect();
    let built = BuiltSite {
        large_search_documents: files
            .iter()
            .find(is_search_documents)
            .map(|file| file.contents.len())
            .filter(is_large),
        untitled: options.title.is_none(),
    };
    SiteFiles { files, built }
}

/// The file of `node`; `None` only for an asset that is not there.
fn render_file(node: &FileNode) -> Option<SiteFile> {
    match node {
        FileNode::Feature(page) => Some(file(page)),
        FileNode::Folder(folder) => Some(file(folder)),
        FileNode::Sentences(page) => Some(file(page)),
        FileNode::FolderTree(items) => Some(file(items.as_slice())),
        FileNode::TagIndex(index) => Some(file(index)),
        FileNode::Tag(page) => Some(file(page)),
        FileNode::SearchDocuments(documents) => Some(file(documents.as_slice())),
        FileNode::Asset(name) => asset_file(name),
    }
}

fn file(node: &(impl RenderFile + ?Sized)) -> SiteFile {
    SiteFile::text(node.address(), node.render())
}

/// The assets every site has.
pub(super) fn assets() -> impl Iterator<Item = SiteFile> {
    SiteAssets::iter().filter_map(|name| asset_file(&name))
}

/// The asset named `name`, as a file of the site.
fn asset_file(name: &str) -> Option<SiteFile> {
    let file = SiteAssets::get(name)?;
    Some(SiteFile {
        address: asset(name),
        contents: file.data.into_owned(),
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
mod tests {
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    use gherkin::{Feature, GherkinEnv};
    use rstest::rstest;

    use super::*;
    use crate::entities::ParsedFeature;
    use crate::policies::site_tree;

    /// The site of one feature named `name`.
    fn built(name: &str) -> BuiltSite {
        let feature =
            Feature::parse(format!("Feature: {name}\n"), GherkinEnv::default()).expect("parses");
        let parsed = ParsedFeature::new(
            PathBuf::from("a.feature"),
            feature,
            "en".to_string(),
            BTreeMap::new(),
        );
        let options = PageOptions::default();
        render_site(&site_tree(&[parsed], &options), &options).built
    }

    #[rstest]
    #[case::small(1, false)]
    #[case::large(LARGE_SEARCH_DOCUMENTS, true)]
    fn only_search_documents_above_the_size_are_reported(
        #[case] name_length: usize,
        #[case] reported: bool,
    ) {
        let built = built(&"a".repeat(name_length));
        assert_eq!(built.large_search_documents.is_some(), reported);
        assert!(
            built
                .large_search_documents
                .is_none_or(|size| is_large(&size))
        );
    }
}
