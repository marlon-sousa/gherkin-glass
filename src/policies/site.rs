//! Policy: the site stage, the site tree built from the features: a feature's page, the folder pages and the frame every page has, search's documents, the step sentences and the tags.

mod feature_pages;
mod pages;
mod search;
mod sentences;
mod tags;

pub(crate) use feature_pages::*;
pub(crate) use pages::*;
pub(crate) use search::*;
pub(crate) use sentences::*;
pub(crate) use tags::*;
