//! Entity: the input, what collecting it found, each feature as the parser returned it and as it was read.

mod collected_input;
mod doc_string;
mod parsed_feature;
mod raw_feature;

pub(crate) use collected_input::CollectedInput;
pub(crate) use doc_string::DocString;
pub(crate) use parsed_feature::ParsedFeature;
pub(crate) use raw_feature::RawFeature;
