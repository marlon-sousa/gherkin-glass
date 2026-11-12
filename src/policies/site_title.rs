//! Policy: the site's title, the configuration's or the catalogue's default, which the site stage and every page's frame use.

use i18n_embed_fl::fl;

use crate::entities::PageOptions;
use crate::policies::CATALOGUE;

pub(crate) fn site_title(options: &PageOptions) -> String {
    options
        .title
        .clone()
        .unwrap_or_else(|| fl!(CATALOGUE, "default-title"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_configured_title_wins_over_the_default() {
        assert_eq!(site_title(&PageOptions::default()), "Feature files");
        let options = PageOptions {
            title: Some("Banking".into()),
            ..PageOptions::default()
        };
        assert_eq!(site_title(&options), "Banking");
    }
}
