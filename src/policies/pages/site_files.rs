//! Policy: every file of a site from its features: its pages, its folder pages, its page of step sentences, its tag pages, its folder tree's and search's data, and its assets.

use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};

use crate::entities::{
    ASSETS, BuiltSite, PageOptions, ParsedFeature, SiteAddress, SiteAssets, SiteFile, SiteFiles,
};
use crate::policies::{
    LARGE_SEARCH_DOCUMENTS, SENTENCES_PAGE, anchor_feature, build_folder_pages, build_folder_tree,
    build_page, build_sentences_page, build_tag_index, build_tag_page, find_step_sentences,
    name_tags, render_folder_page, render_folder_tree, render_page, render_sentences_page,
    render_tag_index, render_tag_page, search_documents, search_json, tag_index_address, tags_of,
};

/// In the folder of the assets, where the pages' script fetches them from.
const TREE_DATA: &str = "tree.json";
const SEARCH_DOCUMENTS: &str = "search.json";

/// Every file of the site of `features`, pages first, in file order.
pub(crate) fn site_files(features: &[ParsedFeature], options: &PageOptions) -> SiteFiles {
    let tags = tags_of(features);
    let names = name_tags(&tags);
    let anchored: Vec<_> = features.iter().map(anchor_feature).collect();
    // Collecting into a `Vec` keeps file order, whatever order the threads finish in.
    let pages: Vec<_> = anchored
        .par_iter()
        .map(|feature| build_page(feature, options, &names))
        .collect();
    let folders = build_folder_pages(&pages, options);
    let mut files: Vec<SiteFile> = pages
        .par_iter()
        .map(|page| SiteFile::text(page.address.clone(), render_page(page)))
        .collect();
    files.extend(
        folders
            .par_iter()
            .map(|folder| SiteFile::text(folder.address.clone(), render_folder_page(folder)))
            .collect::<Vec<_>>(),
    );
    let sentences = find_step_sentences(&anchored);
    files.push(SiteFile::text(
        SiteAddress::at_root(SENTENCES_PAGE),
        render_sentences_page(&build_sentences_page(features, &sentences, options)),
    ));
    files.push(SiteFile::text(
        asset(TREE_DATA),
        render_folder_tree(&build_folder_tree(&folders)),
    ));
    files.push(SiteFile::text(
        tag_index_address(),
        render_tag_index(&build_tag_index(&tags, &pages, &names, options)),
    ));
    files.extend(
        tags.par_iter()
            .map(|tag| build_tag_page(tag, &pages, &names, options))
            .map(|page| SiteFile::text(page.address.clone(), render_tag_page(&page)))
            .collect::<Vec<_>>(),
    );
    let search = search_json(&search_documents(&anchored));
    let built = BuiltSite {
        large_search_documents: Some(search.len()).filter(is_large),
        untitled: options.title.is_none(),
    };
    files.push(SiteFile::text(asset(SEARCH_DOCUMENTS), search));
    files.extend(assets());
    SiteFiles { files, built }
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
        site_files(&[parsed], &PageOptions::default()).built
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
