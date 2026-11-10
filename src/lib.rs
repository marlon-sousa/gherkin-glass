//! Facade: gherkin-glass turns a folder of Gherkin feature files into a static website.
//!
//! docs/BEHAVIOUR.md says what it promises. The build runs in four steps, as
//! issue #3 sets out; [`InputService`] runs the first two, collecting and
//! parsing the input; [`make_anchors`] makes a feature's anchors, part of
//! the third; [`SiteService`] runs the whole build, writing each feature's
//! page; [`PreviewService`] previews a site, built and served; and
//! [`CommandLine`] is the `ghg` command, configured by `ghg.toml`.
//! the third, as [`highlight_step`] splits a step into the parts its highlighting
//! tells apart; [`theme_stylesheet`] writes the themes' colours, part of the
//! fourth; and [`CommandLine`] is the `ghg` command, configured by `ghg.toml`.

#![warn(unreachable_pub)]

mod adapters;
mod controllers;
mod entities;
mod policies;
mod ports;
mod services;

pub use adapters::{
    ConfigFileReader, FileWatcher, GherkinParser, HttpPreviewServer, InputWalker, MemorySite,
    SiteFiles, StandardStreams,
};
pub use controllers::{CommandLine, Reply};
pub use entities::{
    Anchors, Burst, Changes, CollectedInput, ListenFailure, PageOptions, ParseFailure,
    ParsedFeature, Preview, Problem, ProblemKind, Problems, RawFeature, ServedSite, ServerWords,
    Settings, TagLink, Unnamed, UnnamedBlock, Version, WatchFailure, Watched,
};
pub use policies::make_anchors;
pub use ports::{
    ChangeWatcher, ConfigReader, FeatureParser, InputCollector, PreviewServer, SiteRecorder,
    SiteWriter, Terminal,
};
pub use services::{InputService, PreviewService, SiteService};
    Accents, Anchors, CollectedInput, Colour, HighlightedStep, ParseFailure, ParsedFeature,
    Problem, ProblemKind, Problems, RawFeature, StepPart, Unnamed, UnnamedBlock,
};
pub use policies::{DEFAULT_ACCENTS, highlight_step, make_anchors, theme_stylesheet};
pub use ports::{ConfigReader, FeatureParser, InputCollector};
pub use services::InputService;
pub use policies::{DEFAULT_ACCENTS, highlight_step, make_anchors};
pub use ports::{ConfigReader, FeatureParser, InputCollector, SiteWriter};
pub use services::{InputService, SiteService};
