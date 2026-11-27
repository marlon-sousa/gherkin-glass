//! Policy: the pure decisions of a run of ghg, each stage of the pipeline in a module of its own, and what more than one stage uses: the interface text, the name of a feature file's page, which both the input and the site name, the frame every page has, the site's title, the order of versions, and the themes' colours.

mod collecting;
mod configuration;
mod interface_text;
mod page_frame;
mod page_naming;
mod palettes;
mod rendering;
mod serving;
mod site;
mod site_title;
mod version_order;

pub(crate) use collecting::*;
pub(crate) use configuration::*;
pub(crate) use interface_text::CATALOGUE;
pub(crate) use page_frame::page_frame;
pub(crate) use page_naming::page_name;
pub(crate) use palettes::palette;
pub(crate) use rendering::*;
pub(crate) use serving::*;
pub(crate) use site::*;
pub(crate) use site_title::site_title;
pub(crate) use version_order::{order_versions, precedence_groups, version_kind};
