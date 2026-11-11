//! Entity: the values the build reads and reports, those that say how it is run, and what ghg serve serves.

mod anchors;
mod collected_input;
mod config;
mod config_problem;
mod doc_string;
mod invocation;
mod listen_failure;
mod page;
mod page_options;
mod parse_failure;
mod parsed_feature;
mod preview;
mod problem;
mod raw_feature;
mod settings;
mod site_assets;
mod unnamed;
mod watch;

pub use anchors::Anchors;
pub use collected_input::CollectedInput;
pub(crate) use config::Config;
pub(crate) use config_problem::{ConfigProblem, ConfigProblemKind, ValueType};
pub(crate) use doc_string::DocString;
pub(crate) use invocation::{ArgumentProblem, BuildOptions, Invocation, ServeOptions};
pub use listen_failure::ListenFailure;
pub(crate) use page::{
    Block, BlockKind, ContentsEntry, DocStringView, Page, PageWords, Paragraph, SourceLink,
    StepView, TableView,
};
pub use page_options::PageOptions;
pub use parse_failure::ParseFailure;
pub use parsed_feature::ParsedFeature;
pub use preview::{Preview, ServedSite, ServerWords, Version};
pub use problem::{Problem, ProblemKind, Problems, UnnamedBlock};
pub use raw_feature::RawFeature;
pub use settings::Settings;
pub(crate) use site_assets::SiteAssets;
pub use unnamed::Unnamed;
pub use watch::{Burst, Changes, WatchFailure, Watched};
