//! Service: the build's use cases.

mod input;
mod settings;

pub use input::InputService;
pub(crate) use settings::{ConfigProblems, SettingsService};
