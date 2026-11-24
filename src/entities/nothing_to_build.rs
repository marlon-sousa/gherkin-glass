//! Entity: a build that read no feature file, and why: an input folder that holds none, or a list of `--only` that names none, or no path at all.

use std::path::PathBuf;

use crate::entities::{InputTree, ListedFiles, PathList, Settings};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum NothingToBuild {
    /// The input folder, as the settings give it, holds no feature file that is read.
    InputHoldsNone { input: PathBuf },
    /// The list of `--only`, as it was given, names no feature file of the input.
    ListNamesNone { list: PathList },
    /// The list of `--only`, as it was given, names no path at all.
    ListNamesNoPath { list: PathList },
}

impl NothingToBuild {
    /// Why a build with `settings` reads no feature file, when `input_tree`, what it collected,
    /// holds no file that is read and no error.
    pub(crate) fn found(settings: &Settings, input_tree: &InputTree) -> Option<Self> {
        (input_tree.reads_no_file() && input_tree.input_errors().is_none())
            .then(|| Self::of(settings, input_tree))
    }

    /// Why a build with `settings` read no feature file, `input_tree` being what it collected.
    fn of(settings: &Settings, input_tree: &InputTree) -> Self {
        match (&settings.only, input_tree.listed()) {
            (None, _) => NothingToBuild::InputHoldsNone {
                input: settings.input.clone(),
            },
            (Some(list), Some(ListedFiles::NoPath)) => {
                NothingToBuild::ListNamesNoPath { list: list.clone() }
            }
            (Some(list), _) => NothingToBuild::ListNamesNone { list: list.clone() },
        }
    }
}
