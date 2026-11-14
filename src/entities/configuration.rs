//! Entity: how a build is run, what the command line asks for, what a configuration file sets, and the settings resolved from them, among them where a build writes, where `--only` reads its list of feature files from, whether a version is a preview, the theme and the accent colours.

mod colour;
mod config;
mod destination;
mod invocation;
mod path_list;
mod settings;
mod site_scope;
mod theme;
mod unnamed;
mod version_listing;

pub(crate) use colour::Colour;
pub(crate) use config::Config;
pub(crate) use destination::Destination;
pub(crate) use invocation::{BuildOptions, Invocation, ServeMode, ServeOptions};
pub(crate) use path_list::PathList;
pub(crate) use settings::Settings;
pub(crate) use site_scope::SiteScope;
pub(crate) use theme::{Accents, Theme};
pub(crate) use unnamed::Unnamed;
pub(crate) use version_listing::VersionListing;
