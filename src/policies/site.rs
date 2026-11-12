//! Policy: the site stage, the site tree built from the features: a feature's page, the folder pages and the folder tree, search's documents, the step sentences and the tags, with what every page of the site has, its breadcrumb, its language and the site's title.

mod breadcrumb_trail;
mod feature_pages;
mod page_language;
mod pages;
mod search;
mod sentences;
mod site_title;
mod site_tree;
mod tags;

pub(crate) use site_tree::site_tree;

use breadcrumb_trail::breadcrumb;
use feature_pages::{anchor_feature, anchor_of, build_page, heading, unique};
use page_language::page_language;
use pages::{build_folder_pages, build_folder_tree};
use search::search_documents;
use sentences::{build_sentences_page, find_step_sentences};
use site_title::site_title;
use tags::{build_tag_index, build_tag_page, name_tags, outside_link, tag_index_address, tags_of};
