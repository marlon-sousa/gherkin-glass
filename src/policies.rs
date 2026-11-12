//! Policy: the pure decisions of a run of ghg, each stage of the pipeline in a module of its own, and what more than one stage uses: the interface text, the addresses of the pages that both the input and the site name, the frame every page has, and the themes' colours.

mod collecting;
mod configuration;
mod interface_text;
mod page_address;
mod page_frame;
mod palettes;
mod rendering;
mod serving;
mod site;

pub(crate) use collecting::*;
pub(crate) use configuration::*;
pub(crate) use interface_text::CATALOGUE;
pub(crate) use page_address::{FOLDER_PAGE, SENTENCES_PAGE, page_address};
pub(crate) use page_frame::page_frame;
pub(crate) use palettes::palette;
pub(crate) use rendering::*;
pub(crate) use serving::*;
pub(crate) use site::*;
