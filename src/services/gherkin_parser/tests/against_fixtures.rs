//! The gherkin parser over the real file system, checked against the fixtures.

mod constructs;
mod doc_strings;
#[cfg(unix)]
mod unreadable;

use std::fs::{read_to_string, write};
use std::path::Path;

use gherkin::Step;

use crate::test_support::{fixture, read_features as read, scratch};
