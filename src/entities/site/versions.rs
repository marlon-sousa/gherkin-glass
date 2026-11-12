//! Entity: the output folder and its versions, what is in it and what its marker records, what a build writes into it and where, the versions and their names, and the versions page.

mod build_folder;
mod build_target;
mod output_contents;
mod record;
mod version_list;
mod version_name;
mod versions_page;

pub(crate) use build_folder::BuildFolder;
pub(crate) use build_target::BuildTarget;
pub(crate) use output_contents::OutputContents;
pub(crate) use record::{FilePath, Record};
pub(crate) use version_list::VersionList;
pub(crate) use version_name::VersionName;
pub(crate) use versions_page::{VersionEntry, VersionsPage, VersionsWords};
