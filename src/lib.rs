//! Facade: gherkin-glass turns a folder of Gherkin feature files into a static website.
//!
//! docs/BEHAVIOUR.md says what it promises. The build runs in four steps, as
//! issue #3 sets out; [`InputService`] runs the first two, collecting and
//! parsing the input; [`make_anchors`] makes a feature's anchors, part of
//! the third; [`SiteService`] runs the whole build, writing each feature's
//! page; [`PreviewService`] previews a site, built and served; and
//! [`CommandLine`] is the `ghg` command, configured by `ghg.toml`.

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
