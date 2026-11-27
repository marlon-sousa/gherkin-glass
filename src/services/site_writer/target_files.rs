//! Service: the site writer's files of a target, the store, the shell and the shared assets, rendered from the site's store, and the plans by which a target is written into the output folder, with the versions page beside a version, or removed from it.

use std::collections::BTreeSet;

use crate::entities::{
    BuildFolder, BuildTarget, BuiltSite, Destination, FilePath, FrontPageTemplate, PageFrame,
    PageOptions, Settings, SiteFile, SiteStore, StoreFiles, StorePlan, VersionName,
    VersionsPageRecord,
};
use crate::errors::FrontPageError;
use crate::policies::{
    DescriptorSettings, FolderBefore, GHG_VERSION, StoreChange, VersionsPageWrite, WrittenTarget,
    build_home_page, build_shell_page, build_versions_page, describe_site, page_frame, plan_store,
    render_shell, render_store, render_versions_page, shared_assets, site_title,
    versions_once_changed,
};

/// What a build renders before it looks at the output folder again.
pub(super) struct TargetFiles {
    /// The settings' target, or a site without versions in memory.
    target: BuildTarget,
    store: StoreFiles,
    shell: SiteFile,
    assets: Vec<SiteFile>,
    frame: PageFrame,
}

impl TargetFiles {
    /// The files of `store`'s target, its shell's front page written from `template`, or from
    /// ghg's own without one.
    pub(super) fn render(
        settings: &Settings,
        store: &SiteStore,
        template: Option<&FrontPageTemplate>,
    ) -> Result<Self, FrontPageError> {
        let target = match settings.destination {
            Destination::Folder => settings.target.clone(),
            Destination::Memory => BuildTarget::Site,
        };
        let options = &settings.pages;
        let store_files = render_store(
            store,
            &DescriptorSettings {
                target: &target,
                options,
                front_page: template.map(|template| template.text.as_str()),
            },
        );
        let frame = page_frame(options);
        let home = build_home_page(store, options);
        let shell_page = build_shell_page(&target, options, GHG_VERSION);
        let shell = render_shell(&target, &shell_page, &frame, &home, template)?;
        Ok(TargetFiles {
            target,
            store: store_files,
            shell,
            assets: shared_assets(),
            frame,
        })
    }

    /// What the build says of the site of `store`.
    pub(super) fn built_site(&self, store: &SiteStore, options: &PageOptions) -> BuiltSite {
        BuiltSite {
            large_site_data: self.store.large_site_data,
            ..describe_site(store, options)
        }
    }

    /// Every file a site in memory is served from: the objects, the descriptor, the shell and the
    /// assets.
    pub(super) fn in_memory(&self) -> Vec<SiteFile> {
        self.store
            .objects
            .iter()
            .chain([&self.store.descriptor, &self.shell])
            .chain(&self.assets)
            .cloned()
            .collect()
    }

    /// The path of each file of [`TargetFiles::in_memory`].
    pub(super) fn paths(&self) -> BTreeSet<FilePath> {
        self.in_memory().iter().map(SiteFile::path).collect()
    }

    /// The plan by which the target is written into `folder`, with the versions page, written
    /// from `template`, beside a version.
    pub(super) fn plan(
        self,
        folder: &BuildFolder,
        settings: &Settings,
        template: Option<&FrontPageTemplate>,
    ) -> Result<StorePlan, FrontPageError> {
        let options = &settings.pages;
        let versions_page = match &self.target {
            BuildTarget::Version(_) => {
                let record = VersionsPageRecord {
                    ghg: GHG_VERSION.to_string(),
                    title: site_title(options),
                    accents: options.accents,
                    front_page: template.map(|template| template.text.clone()),
                };
                let names = versions_once_changed(folder.published(), &self.target, false);
                Some(versions_page_write(record, names, &self.frame, template)?)
            }
            BuildTarget::Site | BuildTarget::Preview(_) => None,
        };
        let change = StoreChange {
            target: self.target,
            written: Some(WrittenTarget {
                store: self.store,
                shell: self.shell,
                ghg: GHG_VERSION.to_string(),
            }),
            versions_page,
            assets: self.assets,
        };
        Ok(plan_store(&before(folder), change))
    }
}

/// The plan by which the settings' target, a version or a preview, is removed from `folder`,
/// with the versions page written again, by this ghg, from what the store says it was last
/// written with, or from the settings when it says nothing, while another version stays.
pub(super) fn removal_plan(
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
            // The template was read when the page was last written, and is named by the output
            // folder, which keeps it.
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
    Ok(plan_store(&before(folder), change))
}

/// The versions page listing `names`, written with `record`.
fn versions_page_write(
    record: VersionsPageRecord,
    names: Vec<VersionName>,
    frame: &PageFrame,
    template: Option<&FrontPageTemplate>,
) -> Result<VersionsPageWrite, FrontPageError> {
    let content = build_versions_page(names, &record.title);
    let page = render_versions_page(&content, frame, &record.ghg, template)?;
    Ok(VersionsPageWrite { record, page })
}

fn before(folder: &BuildFolder) -> FolderBefore<'_> {
    FolderBefore {
        listed: folder.listed(),
        found: folder.found(),
        published: folder.published(),
    }
}
