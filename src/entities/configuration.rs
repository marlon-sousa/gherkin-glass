//! Entity: how a build is run, what the command line asks for, what a configuration file sets, and the settings resolved from them, among them where a build writes, what it leaves alone in its output folder, where `--only` reads its list of feature files from, whether the site is a preview, whose pages ask not to be indexed, the theme and the accent colours.

mod colour;
mod config;
mod destination;
mod invocation;
mod kept_in_output;
mod path_list;
mod search_indexing;
mod settings;
mod site_scope;
mod theme;
mod unnamed;

pub(crate) use colour::Colour;
pub(crate) use config::Config;
pub(crate) use destination::Destination;
pub(crate) use invocation::{BuildOptions, Invocation, ServeMode, ServeOptions};
pub(crate) use kept_in_output::{KeptInOutput, KeptPattern};
pub(crate) use path_list::PathList;
pub(crate) use search_indexing::SearchIndexing;
pub(crate) use settings::Settings;
pub(crate) use site_scope::SiteScope;
pub(crate) use theme::{Accents, Theme};
pub(crate) use unnamed::Unnamed;
