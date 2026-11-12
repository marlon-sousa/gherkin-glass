//! Policy: the site stage, the site tree built from the features: a feature's page, the folder pages and the folder tree, search's documents, the step sentences, the tag pages and the versions page, with what several of them use: every page's breadcrumb, its language and the site's title, the anchor of a text, the tags' names and their links to other sites.

mod anchor_text;
mod breadcrumb_trail;
mod feature_pages;
mod folder_pages;
mod folder_tree;
mod page_language;
mod search;
mod sentences;
mod site_title;
mod site_tree;
mod tag_linking;
mod tag_naming;
mod tag_pages;
mod versions;

pub(crate) use site_tree::site_tree;

use anchor_text::{anchor_of, unique};
use breadcrumb_trail::breadcrumb;
use feature_pages::build_page;
use folder_pages::build_folder_pages;
use folder_tree::build_folder_tree;
use page_language::page_language;
use search::search_documents;
use sentences::build_sentences_page;
use site_title::site_title;
#[cfg(test)]
use site_tree::site_with_feature_pages;
use tag_linking::outside_link;
use tag_naming::name_tags;
use tag_pages::build_tag_pages;
use versions::build_versions_page;
