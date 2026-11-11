//! Entity: the output folder and its versions, what is at a folder's place, its marker, the folder a build writes into, and the versions and their names.

mod build_folder;
mod folder_state;
mod marker;
mod version_list;
mod version_name;

pub(crate) use build_folder::BuildFolder;
pub(crate) use folder_state::FolderState;
pub(crate) use marker::Marker;
pub(crate) use version_list::VersionList;
pub(crate) use version_name::VersionName;
