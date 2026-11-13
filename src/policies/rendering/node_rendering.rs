//! Policy: the rendering trait, through which every node of the site tree renders its own piece of a page, and what renders a file.

use askama::Template;

use super::feature_block::{ExampleScenarioNode, ExamplesAsScenariosNode};
use super::page_layout::SitePieces;
use crate::entities::{SiteAddress, SiteNode, SiteTreeNode};
use crate::errors::FrontPageError;

/// A node of the site tree, rendered into its piece of a page. A piece has no
/// line break before or after it: the template that places it puts them.
pub(super) trait Render {
    fn render(&self) -> String;
}

/// A node that is a file of its own: where the file goes, and the whole file,
/// which a page makes with the site nodes rendered once for the build. Only the
/// front page, whose template may break its rules, can fail.
pub(super) trait RenderFile {
    fn address(&self) -> SiteAddress;
    fn render_file(&self, site: &SitePieces<'_>) -> Result<String, FrontPageError>;
}

/// A node's value, `value`, with its node, which holds the nodes below it
/// that its piece or its file is made of.
pub(super) struct NodeOf<'a, V> {
    pub(super) value: &'a V,
    pub(super) node: SiteTreeNode<'a>,
}

/// The pieces of `nodes`, each rendered, in their order.
pub(super) fn pieces<'a>(nodes: impl Iterator<Item = SiteTreeNode<'a>>) -> Vec<String> {
    nodes.map(piece).collect()
}

/// The piece of `node`, handed to the implementation of its kind; empty for
/// a node that is no piece of a page: the site, a version, a folder and a file,
/// and for a list of versions.
fn piece(node: SiteTreeNode<'_>) -> String {
    match node.value() {
        SiteNode::Background(value) => NodeOf { value, node }.render(),
        SiteNode::Rule(value) => NodeOf { value, node }.render(),
        SiteNode::Scenario(value) => NodeOf { value, node }.render(),
        SiteNode::Examples(value) => NodeOf { value, node }.render(),
        SiteNode::ExamplesAsScenarios(heading) => {
            ExamplesAsScenariosNode { heading, node }.render()
        }
        SiteNode::ExampleScenario(heading) => ExampleScenarioNode { heading, node }.render(),
        SiteNode::Step(value) => NodeOf { value, node }.render(),
        SiteNode::StepTable(table) => table.render(),
        SiteNode::DocString(doc_string) => doc_string.render(),
        SiteNode::SubfolderLink(link) => link.render(),
        SiteNode::FeatureEntry(entry) => entry.render(),
        SiteNode::SentencesGroup(value) => NodeOf { value, node }.render(),
        SiteNode::Sentence(value) => NodeOf { value, node }.render(),
        SiteNode::SentenceUse(found) => found.render(),
        SiteNode::TagIndexEntry(entry) => entry.render(),
        SiteNode::TagFeature(value) => NodeOf { value, node }.render(),
        SiteNode::TagScenario(value) => NodeOf { value, node }.render(),
        SiteNode::VersionEntry(entry) => entry.render(),
        SiteNode::Site(_)
        | SiteNode::Version(_)
        | SiteNode::KeptVersion(_)
        | SiteNode::InputFolder
        | SiteNode::Folder(_)
        | SiteNode::VersionsPage(_)
        // The versions page renders its lists itself, at the level of its own headings.
        | SiteNode::VersionsList(_)
        | SiteNode::FeaturePage(_)
        | SiteNode::FolderPage(_)
        | SiteNode::SentencesPage(_)
        | SiteNode::FolderTree(_)
        | SiteNode::TagIndex(_)
        | SiteNode::TagPage(_)
        | SiteNode::SearchDocuments(_)
        | SiteNode::Asset(_) => String::new(),
    }
}

/// The templates only write strings, which cannot fail.
#[expect(
    clippy::expect_used,
    reason = "a template that writes only strings renders"
)]
pub(super) fn rendered(template: &impl Template) -> String {
    template
        .render()
        .expect("a template that writes only strings renders")
}

/// A whole page, which ends in a line break.
pub(super) fn page(template: &impl Template) -> String {
    let mut html = rendered(template);
    if !html.ends_with('\n') {
        html.push('\n');
    }
    html
}
