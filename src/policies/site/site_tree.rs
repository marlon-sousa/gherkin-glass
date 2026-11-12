//! Policy: the site tree of a site's features, made anew from the last one: its version, its folders with their pages, each feature's page kept from the last site tree when its file and the names of its tags are the same, its page of step sentences, its tag pages, its folder tree's and search's data, and its assets, and for a site with versions, the versions the output folder holds and the versions page.

use std::borrow::Cow;
use std::collections::{BTreeSet, HashMap};

use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};

use crate::entities::{
    FolderId, PageContent, PageOptions, RevisedFeature, Revision, SiteAssets, SiteFeaturePage,
    SiteTree, SiteTreeBuilder, SiteVersion, TagNames, VersionName,
};
use crate::policies::site::{
    build_folder_pages, build_folder_tree, build_page, build_sentences_page, build_tag_pages,
    build_versions_page, name_tags, same_tag_names, search_documents,
};
use crate::policies::{page_address, page_frame};

/// The tree of the site of `features`, each feature's page in its folder,
/// the folders made once from the pages' addresses, and the views of the
/// feature pages, such as the tag pages, made from the pages in the tree.
/// A feature's page is copied from `last`, the last site tree, when it was
/// converted from the same revision and every tag it writes keeps its name.
/// `version` names the version the build makes, or is `None` for a site
/// without versions, and `built_versions` are those the output folder
/// already holds.
pub(crate) fn site_tree(
    features: &[RevisedFeature<'_>],
    last: Option<&SiteTree>,
    options: &PageOptions,
    version: Option<&VersionName>,
    built_versions: &BTreeSet<VersionName>,
) -> SiteTree {
    let names = name_tags(features.iter().map(|revised| revised.feature));
    let mut site = site_with_feature_pages(features, last, options, &names, version);
    let feature_pages: Vec<_> = site.every_feature_page().collect();
    let sentences = build_sentences_page(&feature_pages, options);
    let tag_pages = build_tag_pages(&feature_pages, &names, options);
    let documents = search_documents(&feature_pages);
    let folder_pages = build_folder_pages(site.folders(), options);
    site.folder_pages(folder_pages);
    let folder_tree = build_folder_tree(site.input_folder());
    site.sentences_page(sentences);
    site.folder_tree(folder_tree);
    site.tag_pages(tag_pages);
    site.search_documents(documents);
    site.assets(SiteAssets::iter().map(Cow::into_owned));
    if let Some(version) = version {
        add_versions(&mut site, version, built_versions);
    }
    site.build()
}

/// The site tree being built, for the version `version` makes, with the page
/// of each of `features` in its folder, its tags linked as `names` says,
/// kept from `last` as `site_tree` says.
pub(crate) fn site_with_feature_pages(
    features: &[RevisedFeature<'_>],
    last: Option<&SiteTree>,
    options: &PageOptions,
    names: &TagNames,
    version: Option<&VersionName>,
) -> SiteTreeBuilder {
    let site_version = SiteVersion {
        name: version.cloned(),
        tag_names: names.clone(),
    };
    let mut site = SiteTreeBuilder::new(page_frame(options), site_version);
    let kept = KeptPages::of(last);
    let folders: Vec<_> = features
        .iter()
        .map(|revised| folder_of(&mut site, revised))
        .collect();
    // Collecting into a `Vec` keeps file order, whatever order the threads finish in.
    let pages: Vec<_> = features
        .par_iter()
        .map(|revised| kept.feature_page(*revised, options, names))
        .collect();
    for (folder, page) in folders.into_iter().zip(pages) {
        match page {
            FeaturePage::Kept(page) => site.kept_feature_page(folder, page),
            FeaturePage::Built(content) => site.feature_page(folder, *content),
        }
    }
    site
}

/// A feature's page, kept from the last site tree or built.
enum FeaturePage<'a> {
    Kept(SiteFeaturePage<'a>),
    Built(Box<PageContent>),
}

/// The feature pages of the last site tree, by the revision they were converted from, with the
/// names of its tags' pages.
struct KeptPages<'a> {
    pages: HashMap<Revision, SiteFeaturePage<'a>>,
    tag_names: Option<&'a TagNames>,
}

impl<'a> KeptPages<'a> {
    fn of(last: Option<&'a SiteTree>) -> Self {
        KeptPages {
            pages: last
                .into_iter()
                .flat_map(SiteTree::feature_pages)
                .map(keyed_by_revision)
                .collect(),
            tag_names: last
                .and_then(SiteTree::version)
                .map(|version| &version.tag_names),
        }
    }

    /// The page of `revised`: the last site tree's, as `page_of` finds it, or else built with
    /// `options`, its tags linked as `names` says.
    fn feature_page(
        &self,
        revised: RevisedFeature<'_>,
        options: &PageOptions,
        names: &TagNames,
    ) -> FeaturePage<'a> {
        match self.page_of(revised, names) {
            Some(page) => FeaturePage::Kept(page),
            None => FeaturePage::Built(Box::new(build_page(revised, options, names))),
        }
    }

    /// The page of the last site tree converted from `revised`'s revision, when every tag it
    /// writes has the same name in the last site tree as in `names`.
    fn page_of(
        &self,
        revised: RevisedFeature<'_>,
        names: &TagNames,
    ) -> Option<SiteFeaturePage<'a>> {
        let last_names = self.tag_names?;
        let page = self.pages.get(&revised.revision)?;
        same_tag_names(revised.feature, last_names, names).then_some(*page)
    }
}

fn keyed_by_revision(page: SiteFeaturePage<'_>) -> (Revision, SiteFeaturePage<'_>) {
    (page.page().revision, page)
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

/// The folder node of `revised`'s page, made when it is not there yet.
fn folder_of(site: &mut SiteTreeBuilder, revised: &RevisedFeature<'_>) -> FolderId {
    site.folder_at(&page_address(&revised.feature.path).folders())
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::entities::{SiteNode, SiteTreeNode, VersionsList};
    use crate::test_support::{name, parsed_at};

    /// The revision after the first, `steps` times.
    fn revision(steps: usize) -> Revision {
        (0..steps).fold(Revision::default(), |revision, _| revision.next())
    }

    #[rstest]
    // A page converted from the revision its file still has, whose tags keep their names, is
    // copied from the last site tree, so it still shows what the last text said.
    #[case::the_same_revision(0, "@slow", "Last")]
    #[case::a_new_revision(1, "@slow", "New")]
    #[case::a_tag_renamed_by_another_file(0, "@Slow", "New")]
    fn a_page_is_kept_from_the_last_site_tree_unless_its_file_or_its_tags_names_changed(
        #[case] steps: usize,
        #[case] other_tag: &str,
        #[case] shown: &str,
    ) {
        let options = PageOptions::default();
        let last_a = parsed_at("a.feature", "@slow\nFeature: Last\n");
        let new_a = parsed_at("a.feature", "@slow\nFeature: New\n");
        let last_b = parsed_at("b.feature", "@other\nFeature: B\n");
        let new_b = parsed_at("b.feature", &format!("{other_tag}\nFeature: B\n"));
        let revised = |feature, steps| RevisedFeature {
            feature,
            revision: revision(steps),
        };
        let last = site_tree(
            &[revised(&last_a, 0), revised(&last_b, 5)],
            None,
            &options,
            None,
            &BTreeSet::new(),
        );
        let new = site_tree(
            &[revised(&new_a, steps), revised(&new_b, 6)],
            Some(&last),
            &options,
            None,
            &BTreeSet::new(),
        );
        let names: Vec<_> = new
            .feature_pages()
            .map(|page| page.page().name.clone())
            .collect();
        assert_eq!(names, [shown, "B"]);
    }

    #[test]
    fn an_empty_input_still_has_its_home_page() {
        let tree = site_tree(&[], None, &PageOptions::default(), None, &BTreeSet::new());
        assert!(tree.files().any(|node| is_the_home_page(node.value())));
        assert_eq!(tree.root_files().count(), 0);
    }

    #[test]
    fn a_versions_page_lists_every_version_once_the_one_built_among_them() {
        let built = BTreeSet::from([name("main"), name("v1.0.0")]);
        let tree = site_tree(
            &[],
            None,
            &PageOptions::default(),
            Some(&name("main")),
            &built,
        );
        let Some(page) = tree.root_files().next() else {
            panic!("the versions page first");
        };
        assert!(matches!(page.value(), SiteNode::VersionsPage(_)));
        let lists: Vec<_> = page
            .children()
            .filter_map(|list| match list.value() {
                SiteNode::VersionsList(kind) => Some((*kind, names(list))),
                _ => None,
            })
            .collect();
        assert_eq!(
            lists,
            [
                (VersionsList::Development, vec![name("main")]),
                (VersionsList::Releases, vec![name("v1.0.0")]),
                (VersionsList::PreReleases, vec![]),
            ]
        );
        assert!(
            tree.root_files()
                .skip(1)
                .all(|node| is_an_asset(node.value()))
        );
        assert!(tree.files().any(|node| is_the_home_page(node.value())));
    }

    /// The names of the versions whose entries are in `list`.
    fn names(list: SiteTreeNode<'_>) -> Vec<VersionName> {
        list.children()
            .filter_map(|entry| match entry.value() {
                SiteNode::VersionEntry(entry) => Some(entry.name.clone()),
                _ => None,
            })
            .collect()
    }

    fn is_an_asset(node: &SiteNode) -> bool {
        matches!(node, SiteNode::Asset(_))
    }

    fn is_the_home_page(node: &SiteNode) -> bool {
        matches!(node, SiteNode::FolderPage(folder) if folder.address.as_str() == "index.html")
    }
}
