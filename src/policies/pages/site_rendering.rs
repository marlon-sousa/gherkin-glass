//! Policy: every file of a site rendered from its tree, and what the build has to say of them.

use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};

use crate::entities::{
    ASSETS, BuiltSite, FileNode, PageOptions, SiteAddress, SiteAssets, SiteFile, SiteFiles,
    SiteTree,
};
use crate::policies::{
    LARGE_SEARCH_DOCUMENTS, SENTENCES_PAGE, render_folder_page, render_folder_tree, render_page,
    render_sentences_page, render_tag_index, render_tag_page, search_json, tag_index_address,
};

/// In the folder of the assets, where the pages' script fetches them from.
const TREE_DATA: &str = "tree.json";
const SEARCH_DOCUMENTS: &str = "search.json";

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
    let (address, text) = match node {
        FileNode::Feature(page) => (page.address.clone(), render_page(page)),
        FileNode::Folder(folder) => (folder.address.clone(), render_folder_page(folder)),
        FileNode::Sentences(page) => (
            SiteAddress::at_root(SENTENCES_PAGE),
            render_sentences_page(page),
        ),
        FileNode::FolderTree(items) => (asset(TREE_DATA), render_folder_tree(items)),
        FileNode::TagIndex(index) => (tag_index_address(), render_tag_index(index)),
        FileNode::Tag(page) => (page.address.clone(), render_tag_page(page)),
        FileNode::SearchDocuments(documents) => (asset(SEARCH_DOCUMENTS), search_json(documents)),
        FileNode::Asset(name) => return asset_file(name),
    };
    Some(SiteFile::text(address, text))
}

/// The assets every site has.
pub(crate) fn assets() -> impl Iterator<Item = SiteFile> {
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

/// The address of the file named `name` in the folder of the assets.
fn asset(name: &str) -> SiteAddress {
    SiteAddress::of(&[ASSETS], name)
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
