//! Entity: the output folder and its versions, what is in it, what is kept there and what its marker records, what a command writes into it, the versions and their names, and the versions page.

mod build_folder;
mod build_target;
mod output_contents;
mod output_marker;
mod version_list;
mod version_name;
mod versions_page;

pub(crate) use build_folder::BuildFolder;
pub(crate) use build_target::BuildTarget;
pub(crate) use output_contents::OutputContents;
pub(crate) use output_marker::{FilePath, OutputMarker};
pub(crate) use version_list::VersionList;
pub(crate) use version_name::VersionName;
pub(crate) use versions_page::{VersionEntry, VersionsPage, VersionsPageContent, VersionsWords};
