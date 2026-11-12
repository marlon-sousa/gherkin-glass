//! Policy: the pure decisions of a run of ghg, each stage of the pipeline in a module of its own, and what more than one stage uses: the interface text, the addresses of the pages that both the input and the site name, the frame every page has, the site's title, and the themes' colours.

mod collecting;
mod configuration;
mod interface_text;
mod page_address;
mod page_frame;
mod palettes;
mod rendering;
mod serving;
mod site;
mod site_title;

pub(crate) use collecting::*;
pub(crate) use configuration::*;
pub(crate) use interface_text::CATALOGUE;
pub(crate) use page_address::{
    FOLDER_PAGE, SENTENCES_PAGE, TAG_INDEX_NAME, page_address, tag_index_address,
};
pub(crate) use page_frame::page_frame;
pub(crate) use palettes::palette;
pub(crate) use rendering::*;
pub(crate) use serving::*;
pub(crate) use site::*;
pub(crate) use site_title::site_title;
