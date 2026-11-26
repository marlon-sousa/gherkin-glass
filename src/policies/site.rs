//! Policy: the site stage, the site tree built from the features: a feature's page, the folder pages and the folder tree, search's documents and the page of its results, the step sentences, the tag pages and the versions page, with what several of them use: a page's language, the anchor of a text, the tags' names and their links to other sites.

mod anchor_rule;
mod anchor_text;
mod block_words;
mod example_rows;
mod feature_pages;
mod folder_pages;
mod folder_tree;
mod page_language;
mod search;
mod search_page;
mod sentence_slots;
mod sentences;
mod site_tree;
mod step_highlighting;
mod tag_linking;
mod tag_naming;
mod tag_pages;
mod versions;

pub(crate) use site_tree::site_tree;

use anchor_rule::anchor_feature;
use anchor_text::{anchor_of, unique};
use block_words::writes_another_keyword;
use example_rows::{ExampleRow, block_rows, example_rows};
use feature_pages::build_page;
use folder_pages::build_folder_pages;
use folder_tree::build_folder_tree;
use page_language::page_language;
use search::search_documents;
use search_page::build_search_page;
use sentence_slots::sentence_of;
use sentences::build_sentences_page;
#[cfg(test)]
use site_tree::site_with_feature_pages;
use step_highlighting::highlight_step;
use tag_linking::outside_link;
use tag_naming::{name_tags, same_tag_names};
use tag_pages::build_tag_pages;
use versions::build_versions_page;
