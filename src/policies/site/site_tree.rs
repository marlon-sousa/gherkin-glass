//! Policy: the site tree of a site's features: its version, its folders with their pages, its page of step sentences, its tag pages, its folder tree's and search's data, and its assets, and for a site with versions, the versions the output folder holds and the versions page.

use std::borrow::Cow;
use std::collections::BTreeSet;

use rayon::prelude::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};

use crate::entities::{
    FolderId, PageOptions, ParsedFeature, SiteAssets, SiteTree, SiteTreeBuilder, TagNames,
    VersionName,
};
use crate::policies::site::{
    anchor_feature, build_folder_pages, build_folder_tree, build_page, build_sentences_page,
    build_tag_index, build_tag_page, build_versions_page, find_step_sentences, name_tags,
    search_documents, tags_of,
};
use crate::policies::{page_address, page_frame};

/// The tree of the site of `features`, each feature's page in its folder,
/// the folders made once from the pages' addresses, and the views of the
/// feature pages, such as the tag pages, made from the pages in the tree. `version` names the
/// version the build makes, or is `None` for a site without versions, and
/// `built_versions` are those the output folder already holds.
pub(crate) fn site_tree(
    features: &[ParsedFeature],
    options: &PageOptions,
    version: Option<&VersionName>,
    built_versions: &BTreeSet<VersionName>,
) -> SiteTree {
    let tags = tags_of(features);
    let names = name_tags(&tags);
    let mut site = site_with_feature_pages(features, options, &names, version);
    let feature_pages: Vec<_> = site.every_feature_page().collect();
    let sentences = build_sentences_page(&find_step_sentences(&feature_pages), options);
    let tag_index = build_tag_index(&tags, &feature_pages, &names, options);
    let tag_pages: Vec<_> = tags
        .par_iter()
        .map(|tag| build_tag_page(tag, &feature_pages, &names, options))
        .collect();
    let documents = search_documents(&feature_pages);
    let folder_pages = build_folder_pages(site.folders(), options);
    site.folder_pages(folder_pages);
    let folder_tree = build_folder_tree(site.input_folder());
    site.sentences_page(sentences);
    site.folder_tree(folder_tree);
    site.tag_index(tag_index);
    site.tag_pages(tag_pages);
    site.search_documents(documents);
    site.assets(SiteAssets::iter().map(Cow::into_owned));
    if let Some(version) = version {
        add_versions(&mut site, version, built_versions);
    }
    site.build()
}

/// The site tree being built, for the version `version` makes, with the page
/// of each of `features` in its folder, its tags linked as `names` says.
pub(crate) fn site_with_feature_pages(
    features: &[ParsedFeature],
    options: &PageOptions,
    names: &TagNames,
    version: Option<&VersionName>,
) -> SiteTreeBuilder {
    let mut site = SiteTreeBuilder::new(page_frame(options), version.cloned());
    let folders: Vec<_> = features
        .iter()
        .map(|feature| folder_of(&mut site, feature))
        .collect();
    // Collecting into a `Vec` keeps file order, whatever order the threads finish in.
    let pages: Vec<_> = features
        .par_iter()
        .zip(folders.par_iter())
        .map(|(feature, folder)| {
            let anchored = anchor_feature(feature);
            build_page(
                &anchored,
                options,
                names,
                &site.site_folder(*folder).names(),
            )
        })
        .collect();
    site.feature_pages(folders.into_iter().zip(pages));
    site
}

/// The versions `built_versions` the output folder holds, but `version`,
/// which the build makes, and the page that lists them all, with its assets.
fn add_versions(
    site: &mut SiteTreeBuilder,
    version: &VersionName,
    built_versions: &BTreeSet<VersionName>,
) {
    let made = BTreeSet::from([version.clone()]);
    site.kept_versions(built_versions.difference(&made).cloned());
    site.versions_page(build_versions_page(built_versions.union(&made).cloned()));
    site.versions_page_assets(SiteAssets::iter().map(Cow::into_owned));
}

/// The folder node of `feature`'s page, made when it is not there yet.
fn folder_of(site: &mut SiteTreeBuilder, feature: &ParsedFeature) -> FolderId {
    site.folder_at(&page_address(&feature.path).folders())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::{SiteNode, VersionEntry};
    use crate::test_support::name;

    #[test]
    fn an_empty_input_still_has_its_home_page() {
        let tree = site_tree(&[], &PageOptions::default(), None, &BTreeSet::new());
        assert!(tree.files().any(|node| is_the_home_page(node.value())));
        assert_eq!(tree.root_files().count(), 0);
    }

    #[test]
    fn a_versions_page_lists_every_version_once_the_one_built_among_them() {
        let built = BTreeSet::from([name("main"), name("v1.0.0")]);
        let tree = site_tree(&[], &PageOptions::default(), Some(&name("main")), &built);
        let Some(SiteNode::VersionsPage(page)) = tree.root_files().next().map(|node| node.value())
        else {
            panic!("the versions page first");
        };
        let names = |entries: &[VersionEntry]| -> Vec<VersionName> {
            entries.iter().map(|entry| entry.name.clone()).collect()
        };
        assert_eq!(names(&page.development), [name("main")]);
        assert_eq!(names(&page.releases), [name("v1.0.0")]);
        assert!(page.pre_releases.is_empty());
        assert!(
            tree.root_files()
                .skip(1)
                .all(|node| is_an_asset(node.value()))
        );
        assert!(tree.files().any(|node| is_the_home_page(node.value())));
    }

    fn is_an_asset(node: &SiteNode) -> bool {
        matches!(node, SiteNode::Asset(_))
    }

    fn is_the_home_page(node: &SiteNode) -> bool {
        matches!(node, SiteNode::FolderPage(folder) if folder.address.as_str() == "index.html")
    }
}
