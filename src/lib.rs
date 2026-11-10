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

pub use adapters::{GherkinParser, InputWalker};
pub use entities::{CollectedInput, ParseFailure, ParsedFeature, Problem, Problems, RawFeature};
pub use ports::{FeatureParser, InputCollector};
pub use services::InputService;
