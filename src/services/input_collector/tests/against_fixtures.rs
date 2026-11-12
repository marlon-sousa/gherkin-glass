//! The input collector over the real file system, checked against the fixtures.

mod unreadable;
mod which_files;

use std::fs::{copy, create_dir_all, write};
use std::path::Path;

use crate::adapters::InputFolderReader;
use crate::entities::InputTree;
use crate::errors::{InputError, InputErrorKind};
use crate::test_support::{copy_folder, fixture, scratch};

use super::*;

fn collected(input: &Path) -> InputTree {
    InputCollector::new(Arc::new(InputFolderReader)).collect(input)
}

/// The paths of the files the input tree of `input` allows, relative to it, with `/` between
/// folders.
fn allowed(input: &Path) -> Vec<String> {
    collected(input)
        .allowed_files()
        .iter()
        .map(|path| InputError::shown_path(input, path))
        .collect()
}

/// The errors of the input tree of `input`, each as its path and what is wrong.
fn errors(input: &Path) -> Vec<(String, InputErrorKind)> {
    let errors = collected(input).input_errors().expect("errors");
    errors.as_slice().iter().map(path_and_kind).collect()
}
