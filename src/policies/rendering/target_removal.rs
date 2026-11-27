//! Policy: the plan by which a version or a preview is removed from the output folder, with the versions page written again while another version stays.

use crate::entities::{
    BuildFolder, BuildTarget, FrontPageTemplate, PageOptions, Settings, StorePlan,
    VersionsPageRecord,
};
use crate::errors::FrontPageError;
use crate::policies::rendering::{
    FolderBefore, GHG_VERSION, StoreChange, plan_store, shared_assets, versions_once_changed,
    versions_page_write,
};
use crate::policies::{page_frame, site_title};

/// The plan by which the settings' target, a version or a preview, is removed from `folder`,
/// with the versions page written again, by this ghg, from what the store says it was last
/// written with, or from the settings when it says nothing, while another version stays.
pub(crate) fn removal_plan(
    folder: &BuildFolder,
    settings: &Settings,
) -> Result<StorePlan, FrontPageError> {
    let target = &settings.target;
    let published = folder.published();
    let names = versions_once_changed(published, target, true);
    let versions_page = match target {
        BuildTarget::Version(_) if !names.is_empty() => {
            let options = &settings.pages;
            let record = match &published.versions_page {
                Some(record) => VersionsPageRecord {
                    ghg: GHG_VERSION.to_string(),
                    ..record.clone()
                },
                None => VersionsPageRecord {
                    ghg: GHG_VERSION.to_string(),
                    title: site_title(options),
                    accents: options.accents,
                    front_page: None,
                },
            };
            let template = record.front_page.clone().map(|text| FrontPageTemplate {
                file: settings.output.clone(),
                text,
            });
            let frame = page_frame(&PageOptions {
                accents: record.accents,
                ..options.clone()
            });
            Some(versions_page_write(
                record,
                names,
                &frame,
                template.as_ref(),
            )?)
        }
        BuildTarget::Site | BuildTarget::Version(_) | BuildTarget::Preview(_) => None,
    };
    let change = StoreChange {
        target: target.clone(),
        written: None,
        versions_page,
        assets: shared_assets(),
    };
    Ok(plan_store(&FolderBefore::of(folder), change))
}
