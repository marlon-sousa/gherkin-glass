//! Service: the stages of the pipeline, which the controller runs in order, each behind the trait the controller holds it by.

mod converter;
mod input;
mod server;
mod settings;
mod site_writer;

pub(crate) use converter::{Converter, ConverterApi};
pub(crate) use input::{InputService, ParserApi};
pub(crate) use server::{ServerApi, ServerService};
pub(crate) use settings::{SettingsApi, SettingsService};
pub(crate) use site_writer::{SiteWriter, SiteWriterApi};
