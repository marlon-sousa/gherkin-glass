//! Policy: the page of search results, with the words the script fills in.

use i18n_embed::LanguageLoader;
use i18n_embed_fl::fl;

use crate::entities::{
    KindHeadings, OwnCrumb, PageOptions, SearchPage, SearchPageWords, Trail, plain_parts,
};
use crate::policies::{CATALOGUE, SEARCH_PAGE, site_title};

/// The page of search results of a site with `options`.
pub(crate) fn build_search_page(options: &PageOptions) -> SearchPage {
    let heading = fl!(CATALOGUE, "search-page-heading");
    let site = site_title(options);
    SearchPage {
        trail: Trail {
            folders: Vec::new(),
            page: Some(OwnCrumb {
                text: plain_parts(&heading),
                file: SEARCH_PAGE.to_string(),
            }),
        },
        language: CATALOGUE.current_language().to_string(),
        title: fl!(CATALOGUE, "search-page-title", site = site.as_str()),
        heading,
        words: SearchPageWords {
            heading: fl!(CATALOGUE, "search-page-heading-for", words = "{words}"),
            title: fl!(
                CATALOGUE,
                "search-page-title-for",
                words = "{words}",
                site = site.as_str()
            ),
            loading: fl!(CATALOGUE, "search-loading"),
            failed: fl!(CATALOGUE, "search-failed"),
            none: fl!(CATALOGUE, "search-none"),
            count: fl!(
                CATALOGUE,
                "search-page-count",
                count = "{count}",
                features = "{features}"
            ),
            within: fl!(CATALOGUE, "search-page-within", blocks = "{blocks}"),
            separator: fl!(CATALOGUE, "search-page-separator"),
            matching_steps: fl!(CATALOGUE, "search-page-matching-steps"),
            matching_tags: fl!(CATALOGUE, "search-page-matching-tags"),
            matching_cells: fl!(CATALOGUE, "search-page-matching-cells"),
            headings: kind_headings(),
        },
    }
}

fn kind_headings() -> KindHeadings {
    KindHeadings {
        feature: named("feature"),
        unnamed_feature: unnamed("feature"),
        rule: named("rule"),
        unnamed_rule: unnamed("rule"),
        background: named("background"),
        unnamed_background: unnamed("background"),
        scenario: named("scenario"),
        unnamed_scenario: unnamed("scenario"),
        scenario_outline: named("scenario-outline"),
        unnamed_scenario_outline: unnamed("scenario-outline"),
        examples: named("examples"),
        numbered_examples: fl!(CATALOGUE, "numbered-examples", place = "{place}"),
    }
}

/// The heading of a block of `kind`, as the catalogue's selector names it, with `{name}` for its
/// name.
fn named(kind: &str) -> String {
    fl!(CATALOGUE, "named-heading", kind = kind, name = "{name}")
}

fn unnamed(kind: &str) -> String {
    fl!(CATALOGUE, "unnamed-heading", kind = kind)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Rule 216: the page's heading, title and crumb, and the words for those searched for.
    #[test]
    fn the_page_is_titled_search_results_and_the_site_s() {
        let options = PageOptions {
            title: Some("Banking".into()),
            ..PageOptions::default()
        };
        let page = build_search_page(&options);
        assert_eq!(page.heading, "Search results");
        assert_eq!(page.title, "Search results, Banking");
        assert_eq!(page.words.heading, "Results for \"{words}\"");
        assert_eq!(page.words.title, "Results for \"{words}\", Banking");
        assert_eq!(
            page.trail.page.map(|crumb| crumb.file),
            Some("search.html".into())
        );
    }

    // Rule 219: each kind's heading, as rule 43 makes it.
    #[test]
    fn each_kind_s_heading_is_worded_as_on_its_page() {
        let headings = build_search_page(&PageOptions::default()).words.headings;
        assert_eq!(headings.scenario_outline, "Scenario outline: {name}");
        assert_eq!(headings.unnamed_background, "Background");
        assert_eq!(headings.numbered_examples, "Examples {place}");
    }
}
