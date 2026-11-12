//! Policy: the site tree of a site's features: its pages, its folder pages, its page of step sentences, its tag pages, its folder tree's and search's data, and its assets.

use std::borrow::Cow;

use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};

use crate::entities::{FileNode, PageOptions, ParsedFeature, SiteAssets, SiteTree};
use crate::policies::{
    anchor_feature, build_folder_pages, build_folder_tree, build_page, build_sentences_page,
    build_tag_index, build_tag_page, find_step_sentences, name_tags, search_documents, tags_of,
};

/// The tree of the site of `features`, pages first, in file order.
pub(crate) fn site_tree(features: &[ParsedFeature], options: &PageOptions) -> SiteTree {
    let tags = tags_of(features);
    let names = name_tags(&tags);
    let anchored: Vec<_> = features.iter().map(anchor_feature).collect();
    // Collecting into a `Vec` keeps file order, whatever order the threads finish in.
    let pages: Vec<_> = anchored
        .par_iter()
        .map(|feature| build_page(feature, options, &names))
        .collect();
    let folders = build_folder_pages(&pages, options);
    let sentences = build_sentences_page(features, &find_step_sentences(&anchored), options);
    let folder_tree = build_folder_tree(&folders);
    let tag_index = build_tag_index(&tags, &pages, &names, options);
    let tag_pages: Vec<_> = tags
        .par_iter()
        .map(|tag| build_tag_page(tag, &pages, &names, options))
        .collect();
    let documents = search_documents(&anchored);
    let files = pages
        .into_iter()
        .map(FileNode::Feature)
        .chain(folders.into_iter().map(FileNode::Folder))
        .chain([
            FileNode::Sentences(sentences),
            FileNode::FolderTree(folder_tree),
            FileNode::TagIndex(tag_index),
        ])
        .chain(tag_pages.into_iter().map(FileNode::Tag))
        .chain([FileNode::SearchDocuments(documents)])
        .chain(SiteAssets::iter().map(Cow::into_owned).map(FileNode::Asset))
        .collect();
    SiteTree {
        files,
        untitled: options.title.is_none(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_empty_input_still_has_its_home_page() {
        let tree = site_tree(&[], &PageOptions::default());
        assert!(tree.files.iter().any(is_the_home_page));
    }

    fn is_the_home_page(node: &FileNode) -> bool {
        matches!(node, FileNode::Folder(folder) if folder.address.as_str() == "index.html")
    }
}
