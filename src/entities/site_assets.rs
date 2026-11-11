//! Entity: the files every site has beside its pages, built into the binary.

use rust_embed::RustEmbed;

/// `assets/` of the repository, written as `assets/` of every site.
#[derive(RustEmbed)]
#[folder = "assets/"]
pub(crate) struct SiteAssets;
