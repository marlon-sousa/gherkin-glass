//! Entity: the values the build reads and reports, and those that say how it is run.

mod anchors;
mod collected_input;
mod config;
mod config_problem;
mod doc_string;
mod invocation;
mod page;
mod page_options;
mod parse_failure;
mod parsed_feature;
mod problem;
mod raw_feature;
mod settings;
mod site_assets;
mod unnamed;

pub use anchors::Anchors;
pub use collected_input::CollectedInput;
pub(crate) use config::Config;
pub(crate) use config_problem::{ConfigProblem, ConfigProblemKind, ValueType};
pub(crate) use doc_string::DocString;
pub(crate) use invocation::{ArgumentProblem, BuildOptions, Invocation};
pub(crate) use page::{
    Block, BlockKind, ContentsEntry, DocStringView, Page, PageWords, Paragraph, SourceLink,
    StepView, TableView,
};
pub use page_options::PageOptions;
pub use parse_failure::ParseFailure;
pub use parsed_feature::ParsedFeature;
pub use problem::{Problem, ProblemKind, Problems, UnnamedBlock};
pub use raw_feature::RawFeature;
pub use settings::Settings;
pub(crate) use site_assets::SiteAssets;
pub use unnamed::Unnamed;
