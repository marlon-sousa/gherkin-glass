//! Policy: the files of a build's target, its store, its shell, with the home page in its `main` and its front page from the template, and the shared assets, rendered from the site's store, and the plan by which they are written into the output folder, with the versions page beside a version.

use std::collections::BTreeSet;

use crate::entities::{
    BuildFolder, BuildTarget, Destination, FilePath, FrontPageTemplate, PageFrame, Settings,
    SiteFile, SiteStore, StoreFiles, StorePlan, VersionsPageRecord,
};
use crate::errors::FrontPageError;
use crate::policies::rendering::{
    DescriptorSettings, FolderBefore, GHG_VERSION, StoreChange, WrittenTarget, plan_store,
    render_shell, render_store, shared_assets, versions_once_changed, versions_page_write,
};
use crate::policies::{build_home_page, build_shell_page, page_frame, site_title};

/// What a build renders before it looks at the output folder again.
pub(crate) struct TargetFiles {
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
    pub(crate) fn render(
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

    /// The size in bytes of the objects, only when it is above the size at which the pages and
    /// search can be slow to start.
    pub(crate) fn large_site_data(&self) -> Option<usize> {
        self.store.large_site_data
    }

    /// Every file a site in memory is served from: the objects, the descriptor, the shell and the
    /// assets.
    pub(crate) fn in_memory(&self) -> Vec<SiteFile> {
        self.store
            .objects
            .iter()
            .chain([&self.store.descriptor, &self.shell])
            .chain(&self.assets)
            .cloned()
            .collect()
    }

    /// The path of each file of [`TargetFiles::in_memory`].
    pub(crate) fn paths(&self) -> BTreeSet<FilePath> {
        self.in_memory().iter().map(SiteFile::path).collect()
    }

    /// The plan by which the target is written into `folder`, with the versions page, written
    /// from `template`, beside a version.
    pub(crate) fn plan(
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
        Ok(plan_store(&FolderBefore::of(folder), change))
    }
}
