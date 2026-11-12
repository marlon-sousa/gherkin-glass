//! Entity: how a build is run, what the command line asks for, what a configuration file sets, and the settings resolved from them, among them where a build writes, the theme and the accent colours.

mod colour;
mod config;
mod destination;
mod invocation;
mod settings;
mod theme;
mod unnamed;

pub(crate) use colour::Colour;
pub(crate) use config::Config;
pub(crate) use destination::Destination;
pub(crate) use invocation::{BuildOptions, Invocation, ServeMode, ServeOptions};
pub(crate) use settings::Settings;
pub(crate) use theme::{Accents, Theme};
pub(crate) use unnamed::Unnamed;
