//! Entity: the values the build reads and reports.

mod collected_input;
mod doc_string;
mod parse_failure;
mod parsed_feature;
mod problem;
mod raw_feature;

pub use collected_input::CollectedInput;
pub(crate) use doc_string::DocString;
pub use parse_failure::ParseFailure;
pub use parsed_feature::ParsedFeature;
pub use problem::{Problem, ProblemKind, Problems};
pub use raw_feature::RawFeature;
