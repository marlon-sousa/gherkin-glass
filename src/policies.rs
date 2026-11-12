//! Policy: the pure decisions of a run of ghg, each area in a module of its own.

mod configuration;
mod feature_pages;
mod input;
mod interface_text;
mod pages;
mod rendering;
mod search;
mod sentences;
mod serving;
mod tags;
mod versions;

pub(crate) use configuration::*;
pub(crate) use feature_pages::*;
pub(crate) use input::*;
pub(crate) use interface_text::CATALOGUE;
pub(crate) use pages::*;
pub(crate) use rendering::*;
pub(crate) use search::*;
pub(crate) use sentences::*;
pub(crate) use serving::*;
pub(crate) use tags::*;
pub(crate) use versions::*;
