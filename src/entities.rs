//! Entity: the values the build reads and reports, those that say how it is run, and what ghg serve serves.
//! Entity: the values the build reads and reports, those that say how it is run, and the themes' colours.

mod accent_shortfall;
mod anchors;
mod breadcrumb;
mod collected_input;
mod colour;
mod config;
mod config_problem;
mod doc_string;
mod folder_page;
mod highlighted_step;
mod invocation;
mod listen_failure;
mod page;
mod page_frame;
mod page_options;
mod parse_failure;
mod parsed_feature;
mod preview;
mod problem;
mod raw_feature;
mod settings;
mod site_assets;
mod tag_link;
mod tag_names;
mod tag_page;
mod tree_item;
mod theme;
mod unnamed;
mod watch;

pub(crate) use accent_shortfall::AccentShortfall;
pub use anchors::Anchors;
pub(crate) use breadcrumb::{Breadcrumb, Crumb};
pub use collected_input::CollectedInput;
pub use colour::Colour;
pub(crate) use config::Config;
pub(crate) use config_problem::{ConfigProblem, ConfigProblemKind, ValueType};
pub(crate) use doc_string::DocString;
pub(crate) use folder_page::{FeatureEntry, FolderLink, FolderPage, FolderWords};
pub(crate) use invocation::{ArgumentProblem, BuildOptions, Invocation, ServeOptions};
pub use listen_failure::ListenFailure;
pub(crate) use page::{
    BackgroundsOption, Block, BlockKind, ContentsEntry, DocStringView, FromBackground, OutsideLink,
    Page, PageWords, Paragraph, SourceLink, StepView, TableView, TagView,
};
pub(crate) use page_frame::PageFrame;
pub use page_options::PageOptions;
pub use highlighted_step::{HighlightedStep, StepPart};
pub(crate) use invocation::{ArgumentProblem, BuildOptions, Invocation};
pub use parse_failure::ParseFailure;
pub use parsed_feature::ParsedFeature;
pub use preview::{Preview, ServedSite, ServerWords, Version};
pub use problem::{Problem, ProblemKind, Problems, UnnamedBlock};
pub use raw_feature::RawFeature;
pub use settings::Settings;
pub(crate) use site_assets::SiteAssets;
pub use tag_link::TagLink;
pub(crate) use tag_names::{TAGS_FOLDER, TagNames};
pub(crate) use tag_page::{TagFeature, TagIndex, TagIndexEntry, TagPage, TagScenario};
pub(crate) use tree_item::TreeItem;
pub(crate) use settings::Settings;
pub use theme::Accents;
pub(crate) use theme::{Palette, Theme};
pub use unnamed::Unnamed;
pub use watch::{Burst, Changes, WatchFailure, Watched};
