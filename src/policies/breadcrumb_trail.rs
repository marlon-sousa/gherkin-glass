//! Policy: a page's breadcrumb, from the home page through the folders above it.

use i18n_embed_fl::fl;

use crate::entities::{Breadcrumb, Crumb};
use crate::policies::{CATALOGUE, FOLDER_PAGE, percent_encoded};

/// `folders` are those the page is in, from the outermost. `page` is the
/// page's own text and file name, as written, or `None` for a folder's page, which is the
/// innermost of `folders`, or the home page when there is none.
pub(crate) fn breadcrumb(
    title: &str,
    folders: &[String],
    page: Option<(&str, &str)>,
) -> Breadcrumb {
    let depth = folders.len();
    let mut crumbs = vec![Crumb {
        address: format!("{}{FOLDER_PAGE}", "../".repeat(depth)),
        text: title.to_string(),
    }];
    crumbs.extend(folders.iter().enumerate().map(|(index, name)| Crumb {
        address: format!("{}{FOLDER_PAGE}", "../".repeat(depth - index - 1)),
        text: name.clone(),
    }));
    if let Some((text, file)) = page {
        crumbs.push(Crumb {
            address: percent_encoded(file),
            text: text.to_string(),
        });
    }
    Breadcrumb {
        label: fl!(CATALOGUE, "breadcrumb"),
        crumbs,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn trail(folders: &[&str], page: Option<(&str, &str)>) -> Vec<(String, String)> {
        let folders: Vec<String> = folders.iter().map(|name| name.to_string()).collect();
        breadcrumb("Site", &folders, page)
            .crumbs
            .into_iter()
            .map(|crumb| (crumb.address, crumb.text))
            .collect()
    }

    fn crumbs(pairs: &[(&str, &str)]) -> Vec<(String, String)> {
        pairs
            .iter()
            .map(|(address, text)| (address.to_string(), text.to_string()))
            .collect()
    }

    #[test]
    fn the_home_page_s_trail_is_itself() {
        assert_eq!(trail(&[], None), crumbs(&[("index.html", "Site")]));
    }

    #[test]
    fn a_folder_page_ends_at_its_own_index() {
        assert_eq!(
            trail(&["billing", "archive"], None),
            crumbs(&[
                ("../../index.html", "Site"),
                ("../index.html", "billing"),
                ("index.html", "archive"),
            ])
        );
    }

    #[test]
    fn a_feature_page_ends_at_itself_after_its_folders() {
        assert_eq!(
            trail(&["billing"], Some(("Feature: Refunds", "refunds.html"))),
            crumbs(&[
                ("../index.html", "Site"),
                ("index.html", "billing"),
                ("refunds.html", "Feature: Refunds"),
            ])
        );
        assert_eq!(
            trail(&[], Some(("Feature: A", "a b.html"))),
            crumbs(&[("index.html", "Site"), ("a%20b.html", "Feature: A")])
        );
    }
}
