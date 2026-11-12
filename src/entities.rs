//! Entity: the values a run of ghg reads, computes and reports, each stage of the pipeline in a module of its own.

mod collecting;
mod configuration;
mod serving;
mod site;

pub(crate) use collecting::*;
pub(crate) use configuration::*;
pub(crate) use serving::*;
pub(crate) use site::*;
