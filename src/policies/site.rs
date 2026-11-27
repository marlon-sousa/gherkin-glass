//! Policy: the site stage, what a build makes of the features: the site's store, the home page a shell holds, what the build says of the site, what a shell says of itself, and the versions page, with what several of them use: a page's language, the anchor of a text and the anchors of a feature's blocks, the words of a block's heading, an outline's rows, a step's highlighting, kind and sentence, a block's words, a link to a feature's source, the tags' names and their links to other sites.

mod anchor_rule;
mod anchor_text;
mod block_counts;
mod block_words;
mod built_site;
mod description_lines;
mod example_rows;
mod heading_words;
mod home_page;
mod page_language;
mod sentence_slots;
mod shell_page;
mod source_address;
mod step_highlighting;
mod step_kinds;
mod store;
mod tag_linking;
mod tag_naming;
mod versions;

pub(crate) use built_site::describe_site;
pub(crate) use home_page::build_home_page;
pub(crate) use shell_page::build_shell_page;
pub(crate) use store::build_store;
pub(crate) use versions::build_versions_page;

use anchor_rule::anchor_feature;
use anchor_text::{anchor_of, unique};
use block_words::writes_another_keyword;
use description_lines::description_lines;
use example_rows::{ExampleRow, block_rows, example_rows};
use heading_words::heading;
use page_language::page_language;
use sentence_slots::{SentenceKey, sentence_of};
use source_address::source_address;
use step_highlighting::highlight_step;
use step_kinds::step_kind;
use tag_linking::outside_link;
use tag_naming::name_stored_tags;
