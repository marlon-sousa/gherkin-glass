//! Policy: the site stage, the site tree built from the features: a feature's page, the folder pages and the folder tree, search's documents, the step sentences, the tags and the versions page, with what every page of the site has, its breadcrumb, its language and the site's title.

mod breadcrumb_trail;
mod feature_pages;
mod folders;
mod page_language;
mod search;
mod sentences;
mod site_title;
mod site_tree;
mod tags;
mod versions;

pub(crate) use site_tree::site_tree;

use breadcrumb_trail::breadcrumb;
use feature_pages::{anchor_feature, anchor_of, build_page, heading, unique};
use folders::{build_folder_pages, build_folder_tree};
use page_language::page_language;
use search::search_documents;
use sentences::{build_sentences_page, find_step_sentences};
use site_title::site_title;
use tags::{build_tag_index, build_tag_page, name_tags, outside_link, tag_index_address, tags_of};
use versions::build_versions_page;
