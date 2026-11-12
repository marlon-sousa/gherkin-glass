//! Policy: a page's breadcrumb, from the home page through the folders it is in, the input's found above its node, then those of its trail, to the page itself, rendered from its template, and the path from the page back to the site's root.

use askama::Template;

use super::node_rendering::rendered;
use crate::entities::{BreadcrumbWords, LinkAddress, OwnCrumb, SiteTreeNode, Trail};
use crate::policies::FOLDER_PAGE;

/// Where a page is in the site: the folders it is in, from the outermost, and its own crumb.
pub(super) struct Place<'a> {
    folders: Vec<&'a str>,
    page: Option<&'a OwnCrumb>,
}

#[derive(Template)]
#[template(path = "breadcrumb.html")]
struct BreadcrumbTemplate<'a> {
    /// The landmark's name.
    label: &'a str,
    /// From the home page in; the last is the page itself.
    crumbs: Vec<Crumb<'a>>,
}

struct Crumb<'a> {
    /// Relative to the page the breadcrumb is on.
    address: LinkAddress,
    text: &'a str,
}

impl<'a> Place<'a> {
    /// The place of the page whose node is `node` and whose trail is `trail`.
    pub(super) fn of(node: SiteTreeNode<'a>, trail: &'a Trail) -> Self {
        let mut folders = node.folders();
        folders.extend(trail.folders.iter().map(String::as_str));
        Place {
            folders,
            page: trail.page.as_ref(),
        }
    }

    /// From the page back to the site's root.
    pub(super) fn root(&self) -> String {
        "../".repeat(self.folders.len())
    }

    /// The page's breadcrumb, in `words`, rendered.
    pub(super) fn breadcrumb(&self, words: &BreadcrumbWords) -> String {
        let depth = self.folders.len();
        let home = Crumb {
            address: up_to_folder_page(depth),
            text: &words.home,
        };
        // The innermost folder is the one the page is in, zero levels up.
        let folders = self
            .folders
            .iter()
            .copied()
            .zip((0..depth).rev())
            .map(folder_crumb);
        let own = self.page.map(own_crumb);
        rendered(&BreadcrumbTemplate {
            label: &words.label,
            crumbs: [home].into_iter().chain(folders).chain(own).collect(),
        })
    }
}

/// The crumb of the folder named `text`, `levels` folders above the page.
fn folder_crumb((text, levels): (&str, usize)) -> Crumb<'_> {
    Crumb {
        address: up_to_folder_page(levels),
        text,
    }
}

fn own_crumb(page: &OwnCrumb) -> Crumb<'_> {
    Crumb {
        address: LinkAddress::to(&page.file),
        text: &page.text,
    }
}

/// The link to the page of the folder `levels` folders above.
fn up_to_folder_page(levels: usize) -> LinkAddress {
    LinkAddress::to(&format!("{}{FOLDER_PAGE}", "../".repeat(levels)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn trail(folders: &[&str], page: Option<(&str, &str)>) -> String {
        let page = page.map(|(text, file)| OwnCrumb {
            text: text.to_string(),
            file: file.to_string(),
        });
        let place = Place {
            folders: folders.to_vec(),
            page: page.as_ref(),
        };
        let words = BreadcrumbWords {
            label: "Breadcrumb".to_string(),
            home: "Site".to_string(),
        };
        place.breadcrumb(&words)
    }

    /// The links of a breadcrumb with `crumbs`, each an address and a text, the last the page's.
    fn links(crumbs: &[(&str, &str)]) -> String {
        let items: Vec<_> = crumbs
            .iter()
            .enumerate()
            .map(|(place, (address, text))| {
                let current = if place + 1 == crumbs.len() {
                    " aria-current=\"page\""
                } else {
                    ""
                };
                format!("<li><a href=\"{address}\"{current}>{text}</a></li>")
            })
            .collect();
        format!(
            "<nav class=\"ghg-breadcrumb\" aria-label=\"Breadcrumb\">\n<ol>\n{}\n</ol>\n</nav>",
            items.join("\n")
        )
    }

    #[test]
    fn the_home_page_s_trail_is_itself() {
        assert_eq!(trail(&[], None), links(&[("index.html", "Site")]));
    }

    #[test]
    fn a_folder_page_ends_at_its_own_index() {
        assert_eq!(
            trail(&["billing", "archive"], None),
            links(&[
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
            links(&[
                ("../index.html", "Site"),
                ("index.html", "billing"),
                ("refunds.html", "Feature: Refunds"),
            ])
        );
        assert_eq!(
            trail(&[], Some(("Feature: A", "a b.html"))),
            links(&[("index.html", "Site"), ("a%20b.html", "Feature: A")])
        );
    }

    #[test]
    fn a_page_s_root_is_one_level_up_for_each_folder_it_is_in() {
        let place = Place {
            folders: vec!["billing", "archive"],
            page: None,
        };
        assert_eq!(place.root(), "../../");
    }
}
