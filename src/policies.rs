//! Policy: the pure decisions of a run of ghg, each stage of the pipeline in a module of its own.

mod collecting;
mod configuration;
mod interface_text;
mod palettes;
mod rendering;
mod serving;
mod site;

pub(crate) use collecting::*;
pub(crate) use configuration::*;
pub(crate) use interface_text::CATALOGUE;
pub(crate) use palettes::palette;
pub(crate) use rendering::*;
pub(crate) use serving::*;
pub(crate) use site::*;
