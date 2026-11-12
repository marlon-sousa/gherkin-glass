//! Policy: the site's data for the pages' script, the folder tree's and the documents search reads, rendered as JSON.

use serde::Serialize;

use super::node_rendering::{Render, RenderFile};
use crate::entities::{ASSETS, SearchDocument, SearchKind, SiteAddress, TreeItem};

/// In the folder of the assets, where the pages' script fetches them from.
pub(super) const TREE_DATA: &str = "tree.json";
pub(super) const SEARCH_DOCUMENTS: &str = "search.json";

/// An item of the folder tree, as the script reads it.
#[derive(Serialize)]
struct TreeData<'a> {
    text: &'a str,
    address: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    lang: Option<&'a str>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    items: Vec<TreeData<'a>>,
}

/// A document, as search reads it.
#[derive(Serialize)]
struct SearchData<'a> {
    kind: KindData,
    name: &'a str,
    feature: Option<&'a str>,
    rule: Option<&'a str>,
    steps: &'a [String],
    tags: &'a [String],
    page: &'a str,
    anchor: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
enum KindData {
    Feature,
    Rule,
    Scenario,
    ScenarioOutline,
}

/// The folder tree's items, on one line that ends in a line break.
impl Render for [TreeItem] {
    fn render(&self) -> String {
        let items: Vec<_> = self.iter().map(tree_data).collect();
        let mut json = json(&items);
        json.push('\n');
        json
    }
}

impl RenderFile for [TreeItem] {
    fn address(&self) -> SiteAddress {
        asset(TREE_DATA)
    }
}

impl Render for [SearchDocument] {
    fn render(&self) -> String {
        let documents: Vec<_> = self.iter().map(search_data).collect();
        json(&documents)
    }
}

impl RenderFile for [SearchDocument] {
    fn address(&self) -> SiteAddress {
        asset(SEARCH_DOCUMENTS)
    }
}

fn tree_data(item: &TreeItem) -> TreeData<'_> {
    TreeData {
        text: &item.text,
        address: item.address.to_string(),
        lang: item.lang.as_deref(),
        items: item.items.iter().map(tree_data).collect(),
    }
}

fn search_data(document: &SearchDocument) -> SearchData<'_> {
    SearchData {
        kind: kind_data(document.kind),
        name: &document.name,
        feature: document.feature.as_deref(),
        rule: document.rule.as_deref(),
        steps: &document.steps,
        tags: &document.tags,
        page: document.page.as_str(),
        anchor: &document.anchor,
    }
}

fn kind_data(kind: SearchKind) -> KindData {
    match kind {
        SearchKind::Feature => KindData::Feature,
        SearchKind::Rule => KindData::Rule,
        SearchKind::Scenario => KindData::Scenario,
        SearchKind::ScenarioOutline => KindData::ScenarioOutline,
    }
}

#[expect(
    clippy::expect_used,
    reason = "strings, lists of strings and options of them always serialise"
)]
fn json(data: &impl Serialize) -> String {
    serde_json::to_string(data)
        .expect("strings, lists of strings and options of them always serialise")
}

/// The address of the file named `name` in the folder of the assets.
pub(super) fn asset(name: &str) -> SiteAddress {
    SiteAddress::of(&[ASSETS], name)
}

#[cfg(test)]
mod tests;
