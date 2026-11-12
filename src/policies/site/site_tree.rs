//! Policy: the site tree of a site's features: its version, its folders with their pages, its page of step sentences, its tag pages, its folder tree's and search's data, and its assets.

use std::borrow::Cow;

use rayon::prelude::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};

use crate::entities::{
    FolderId, PageOptions, ParsedFeature, SiteAssets, SiteTree, SiteTreeBuilder,
};
use crate::policies::site::{
    anchor_feature, build_folder_pages, build_folder_tree, build_page, build_sentences_page,
    build_tag_index, build_tag_page, find_step_sentences, name_tags, search_documents, tags_of,
};
use crate::policies::{page_address, page_frame};

/// The tree of the site of `features`, without versions, each feature's page
/// in its folder, the folders made once from the pages' addresses.
pub(crate) fn site_tree(features: &[ParsedFeature], options: &PageOptions) -> SiteTree {
    let tags = tags_of(features);
    let names = name_tags(&tags);
    let anchored: Vec<_> = features.iter().map(anchor_feature).collect();
    let mut site = SiteTreeBuilder::new(page_frame(options), None);
    let folders: Vec<_> = features
        .iter()
        .map(|feature| folder_of(&mut site, feature))
        .collect();
    // Collecting into a `Vec` keeps file order, whatever order the threads finish in.
    let pages: Vec<_> = anchored
        .par_iter()
        .zip(folders.par_iter())
        .map(|(feature, folder)| {
            build_page(feature, options, &names, &site.site_folder(*folder).names())
        })
        .collect();
    let sentences = build_sentences_page(features, &find_step_sentences(&anchored), options);
    let tag_index = build_tag_index(&tags, &pages, &names, options);
    let tag_pages: Vec<_> = tags
        .par_iter()
        .map(|tag| build_tag_page(tag, &pages, &names, options))
        .collect();
    let documents = search_documents(&anchored);
    site.feature_pages(folders.into_iter().zip(pages));
    let folder_pages = build_folder_pages(site.folders(), options);
    site.folder_pages(folder_pages);
    let folder_tree = build_folder_tree(site.input_folder());
    site.sentences_page(sentences);
    site.folder_tree(folder_tree);
    site.tag_index(tag_index);
    site.tag_pages(tag_pages);
    site.search_documents(documents);
    site.assets(SiteAssets::iter().map(Cow::into_owned));
    site.build()
}

/// The folder node of `feature`'s page, made when it is not there yet.
fn folder_of(site: &mut SiteTreeBuilder, feature: &ParsedFeature) -> FolderId {
    site.folder_at(&page_address(&feature.path).folders())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::SiteNode;

    #[test]
    fn an_empty_input_still_has_its_home_page() {
        let tree = site_tree(&[], &PageOptions::default());
        assert!(tree.files().any(is_the_home_page));
    }

    fn is_the_home_page(node: &SiteNode) -> bool {
        matches!(node, SiteNode::FolderPage(folder) if folder.address.as_str() == "index.html")
    }
}
