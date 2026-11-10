//! Facade: gherkin-glass turns a folder of Gherkin feature files into a static website.
//!
//! docs/BEHAVIOUR.md says what it promises. The build runs in four steps, as
//! issue #3 sets out; [`InputService`] runs the first two, collecting and
//! parsing the input.

#![warn(unreachable_pub)]

mod adapters;
mod entities;
mod policies;
mod ports;
mod services;

pub use adapters::{FileSystem, GherkinParser};
pub use entities::{EntryKind, FolderEntry, ParseFailure, ParsedFeature, Problem, Problems};
pub use ports::{FeatureParser, InputFolder};
pub use services::InputService;
