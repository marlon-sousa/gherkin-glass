//! gherkin-glass turns a folder of Gherkin feature files into a static website.
//!
//! The build runs in four steps, as issue #3 sets out: collect the files to
//! read, parse them, build the model, and render the pages. This library has
//! the first two, which [`read_input`] runs; docs/BEHAVIOUR.md, "Reading the
//! input", says what they promise.

mod collect;
mod doc_string;
mod parse;
mod problem;

use std::path::Path;

pub use parse::ParsedFeature;
pub use problem::{Problem, Problems};

/// Collects the feature files under `input` and parses every one of them.
///
/// Returns the features sorted by path, or, if anything cannot be read, every
/// problem found, sorted by path. A problem found while collecting does not
/// stop the files already found from being parsed, so that one run reports
/// everything.
pub fn read_input(input: &Path) -> Result<Vec<ParsedFeature>, Problems> {
    let (files, mut problems) = collect::collect(input);
    let features = parse::parse(input, &files, &mut problems);
    if problems.is_empty() {
        Ok(features)
    } else {
        Err(Problems::new(problems))
    }
}
