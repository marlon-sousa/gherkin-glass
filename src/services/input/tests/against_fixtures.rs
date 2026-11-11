//! The input service over the real file system, checked against the fixtures.

mod constructs;
mod doc_strings;
mod unreadable;
mod which_files;

#[cfg(unix)]
use std::fs::{Permissions, set_permissions};
use std::fs::{copy, create_dir_all, read_dir, read_to_string, write};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use gherkin::Step;

use crate::entities::{ParsedFeature, ProblemKind};
use crate::fixtures::{copy_folder, fixture, read_features as read, read_input, scratch};

/// The paths read, with `/` between folders.
fn paths(features: &[ParsedFeature]) -> Vec<String> {
    features
        .iter()
        .map(|feature| {
            let parts: Vec<_> = feature
                .path
                .components()
                .map(|part| part.as_os_str().to_string_lossy().into_owned())
                .collect();
            parts.join("/")
        })
        .collect()
}
