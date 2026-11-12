//! Entity: the values a run of ghg reads, computes and reports, each stage of the pipeline in a module of its own, and the colours of each theme, which every stage that paints a page uses.

mod collecting;
mod configuration;
mod palette;
mod serving;
mod site;

pub(crate) use collecting::*;
pub(crate) use configuration::*;
pub(crate) use palette::Palette;
pub(crate) use serving::*;
pub(crate) use site::*;
