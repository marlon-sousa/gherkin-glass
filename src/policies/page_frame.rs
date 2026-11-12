//! Policy: what every page has around its content, from the settings of every page.

use i18n_embed_fl::fl;

use crate::entities::{PageFrame, PageOptions};
use crate::policies::CATALOGUE;

mod search_wording;

use search_wording::search_words;

pub(crate) fn page_frame(options: &PageOptions) -> PageFrame {
    PageFrame {
        skip_link: fl!(CATALOGUE, "skip-to-content"),
        show_tree: fl!(CATALOGUE, "show-folder-tree"),
        tree: fl!(CATALOGUE, "folder-tree"),
        tree_shown: options.show_tree,
        theme: fl!(CATALOGUE, "theme"),
        theme_system: fl!(CATALOGUE, "theme-system"),
        theme_light: fl!(CATALOGUE, "theme-light"),
        theme_dark: fl!(CATALOGUE, "theme-dark"),
        accents: options.accents,
        search: search_words(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_tree_is_shown_as_the_configuration_says() {
        assert!(page_frame(&PageOptions::default()).tree_shown);
        let hidden = PageOptions {
            show_tree: false,
            ..PageOptions::default()
        };
        assert!(!page_frame(&hidden).tree_shown);
        assert_eq!(page_frame(&hidden).skip_link, "Skip to content");
    }
}
