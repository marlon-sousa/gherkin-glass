//! Entity: the values a run of ghg reads, computes and reports, each stage of the pipeline in a module of its own, what a build made, which holds the parts of several stages, and the colours of each theme, which every stage that paints a page uses.

mod build_snapshot;
mod collecting;
mod configuration;
mod palette;
mod serving;
mod site;

pub(crate) use build_snapshot::BuildSnapshot;
pub(crate) use collecting::*;
pub(crate) use configuration::*;
pub(crate) use palette::Palette;
pub(crate) use serving::*;
pub(crate) use site::*;
