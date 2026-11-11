//! Entity: the values a run of ghg reads, computes and reports, each area in a module of its own.

mod configuration;
mod feature_pages;
mod input;
mod pages;
mod runs;
mod search;
mod sentences;
mod serving;
mod tags;
mod themes;
mod versions;

pub(crate) use configuration::*;
pub(crate) use feature_pages::*;
pub(crate) use input::*;
pub(crate) use pages::*;
pub(crate) use runs::*;
pub(crate) use search::*;
pub(crate) use sentences::*;
pub(crate) use serving::*;
pub(crate) use tags::*;
pub(crate) use themes::*;
pub(crate) use versions::*;
