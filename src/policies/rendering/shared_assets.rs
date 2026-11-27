//! Policy: the files of `assets/<ghg>/` this ghg writes, built into it: the stylesheet, the script, the icon and the licences of the packages the script holds.

use std::sync::Arc;

use crate::entities::{SiteAddress, SiteAssets, SiteFile};
use crate::policies::rendering::{ASSET_FILES, GHG_VERSION, assets_folder};

/// Each file of this ghg's assets, at its path below the output folder.
pub(crate) fn shared_assets() -> Vec<SiteFile> {
    ASSET_FILES.iter().filter_map(|file| asset(file)).collect()
}

fn asset(file: &str) -> Option<SiteFile> {
    let embedded = SiteAssets::get(file)?;
    Some(SiteFile {
        address: SiteAddress::at_root(&format!("{}/{file}", assets_folder(GHG_VERSION))),
        contents: Arc::from(embedded.data.into_owned()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // Rule 39: every asset is built into ghg.
    #[test]
    fn every_asset_is_built_in_under_this_ghg_s_folder() {
        let paths: Vec<String> = shared_assets().iter().map(SiteFile::path).collect();
        let expected: Vec<String> = ASSET_FILES
            .iter()
            .map(|file| format!("assets/{GHG_VERSION}/{file}"))
            .collect();
        assert_eq!(paths, expected);
    }
}
