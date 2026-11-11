//! Entity: the files every site has beside its pages, built into the binary.

use rust_embed::RustEmbed;

/// The folder of a site that holds its assets, in the site's root.
pub(crate) const ASSETS: &str = "assets";

/// `assets/` of the repository, written as `assets/` of every site.
#[derive(RustEmbed)]
#[folder = "assets/"]
pub(crate) struct SiteAssets;
