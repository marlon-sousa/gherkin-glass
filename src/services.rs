//! Service: the stages of the pipeline, which the controller runs in order.

mod input;
mod output;
mod server;
mod settings;
mod site;

pub(crate) use input::InputService;
pub(crate) use output::OutputService;
pub(crate) use server::ServerService;
pub(crate) use settings::SettingsService;
pub(crate) use site::SiteService;
