//! Entity: the output folder and its versions, what is at a folder's place, its marker, the folder a build writes into, the versions and their names, and what stops a build writing.

mod build_folder;
mod folder_state;
mod marker;
mod output_problem;
mod version_list;
mod version_name;

pub(crate) use build_folder::BuildFolder;
pub(crate) use folder_state::FolderState;
pub(crate) use marker::Marker;
pub(crate) use output_problem::{OutputProblem, OutputProblemKind};
pub(crate) use version_list::VersionList;
pub(crate) use version_name::VersionName;
