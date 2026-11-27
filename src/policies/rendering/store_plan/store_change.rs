//! Policy: what a command changes in the output folder's store, and what the folder holds before it: the input of the store plan.

use std::collections::BTreeSet;

use crate::entities::{
    BuildFolder, BuildTarget, FilePath, PublishedStore, SiteFile, StoreFiles, VersionName,
    VersionsPageRecord,
};

/// What a command does to the output folder's store.
pub(crate) struct StoreChange {
    pub(crate) target: BuildTarget,
    /// `None` when the target is removed.
    pub(crate) written: Option<WrittenTarget>,
    /// The versions page, written again when a version is added or removed and another stays.
    pub(crate) versions_page: Option<VersionsPageWrite>,
    /// The files of this ghg's assets.
    pub(crate) assets: Vec<SiteFile>,
}

/// The files of a target a build writes.
pub(crate) struct WrittenTarget {
    pub(crate) store: StoreFiles,
    pub(crate) shell: SiteFile,
    /// The version of the ghg that writes it.
    pub(crate) ghg: String,
}

/// The versions page, rendered, and what it is written with.
pub(crate) struct VersionsPageWrite {
    pub(crate) record: VersionsPageRecord,
    pub(crate) page: SiteFile,
}

/// What the folder holds before the command writes anything.
pub(crate) struct FolderBefore<'a> {
    /// Every file its marker lists.
    pub(crate) listed: &'a BTreeSet<FilePath>,
    /// Every file in it, but what is kept.
    pub(crate) found: &'a BTreeSet<FilePath>,
    pub(crate) published: &'a PublishedStore,
}

impl<'a> FolderBefore<'a> {
    /// What `folder` holds, as the output check found it.
    pub(crate) fn of(folder: &'a BuildFolder) -> Self {
        FolderBefore {
            listed: folder.listed(),
            found: folder.found(),
            published: folder.published(),
        }
    }
}

/// The versions the store holds once `target` is written, or removed when `removed`.
pub(crate) fn versions_once_changed(
    published: &PublishedStore,
    target: &BuildTarget,
    removed: bool,
) -> Vec<VersionName> {
    let mut targets: BTreeSet<&BuildTarget> = published.descriptors.keys().collect();
    if removed {
        targets.remove(target);
    } else {
        targets.insert(target);
    }
    targets
        .into_iter()
        .filter_map(BuildTarget::version)
        .cloned()
        .collect()
}
