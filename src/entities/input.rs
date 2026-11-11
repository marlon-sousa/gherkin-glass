//! Entity: the input, what collecting it found, each feature as the parser returned it and as it was read, and the problems that stop a build.

mod collected_input;
mod doc_string;
mod parse_failure;
mod parsed_feature;
mod problem;
mod raw_feature;

pub(crate) use collected_input::CollectedInput;
pub(crate) use doc_string::DocString;
pub(crate) use parse_failure::ParseFailure;
pub(crate) use parsed_feature::ParsedFeature;
pub(crate) use problem::{Problem, ProblemKind, Problems, UnnamedBlock};
pub(crate) use raw_feature::RawFeature;
