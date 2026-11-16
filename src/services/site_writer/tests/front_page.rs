//! The site writer's tests of the front page's template, read as the site is written.

use std::path::PathBuf;

use rstest::rstest;

use super::*;
use crate::entities::{IgnoreRules, InputTreeBuilder, OutputContents, Revision};
use crate::errors::BuildError;
use crate::services::{Converter, ConverterApi};
/// Writes the site of `a.feature` from the front page's template `text`, `None` when it
/// cannot be read, into `folders`, and returns what it rendered.
fn build_with_template(
    text: Option<&'static str>,
    folders: Arc<FakeFolders>,
    writer: Arc<FakeWriter>,
) -> Result<RenderedSite, BuildError> {
    let site_writer = SiteWriter::new(
        Arc::new(FakeTemplate { text }),
        folders,
        writer,
        Arc::new(FakeWriter::default()),
    );
    let settings = Settings {
        front_page: Some(PathBuf::from("docs/front.md")),
        ..settings(Destination::Folder, None)
    };
    let empty =
        InputTreeBuilder::listed("in".into(), IgnoreRules::default(), Revision::default()).build();
    let built_versions = site_writer.check(&settings)?;
    let tree = Converter.convert(&empty, None, &settings, &built_versions);
    let (_, rendered) = site_writer.write(&settings, tree, None)?;
    Ok(rendered)
}

#[test]
fn the_front_page_s_template_is_read_as_the_site_is_written() {
    // Rule 186: the template is read through the file reader as the site is written.
    let folders = FakeFolders::holding(OutputContents::Missing);
    let writer = Arc::new(FakeWriter::default());
    let rendered = build_with_template(
        Some("# Front\n<!-- ghg:contents -->\n"),
        folders,
        writer.clone(),
    )
    .expect("built");
    let home = rendered
        .files
        .files
        .iter()
        .find(|file| file.address.as_str() == "index.html")
        .map(|file| String::from_utf8_lossy(&file.contents).into_owned())
        .unwrap_or_default();
    assert!(home.contains("<h1>Front</h1>"), "{home}");
    assert!(written(&writer).contains(&"site/index.html".to_string()));
}

#[rstest]
// Rule 186: a template that cannot be read, or that breaks a rule, stops the build before
// anything is cleared or written.
#[case::cannot_be_read(None, "docs/front.md: cannot be read: entity not found")]
#[case::breaks_a_rule(
    Some("<!-- ghg:contents -->\n"),
    "docs/front.md: has no level-one heading, written with one #, for the front page\'s title."
)]
fn a_template_that_stops_the_build_leaves_the_output_folder_as_it_is(
    #[case] text: Option<&'static str>,
    #[case] report: &str,
) {
    let folders = FakeFolders::holding(OutputContents::Missing);
    let writer = Arc::new(FakeWriter::default());
    let outcome = build_with_template(text, folders.clone(), writer.clone());
    let Err(BuildError::FrontPage(error)) = outcome else {
        panic!("{outcome:?}");
    };
    assert_eq!(error.to_string(), report);
    assert!(folders.done().is_empty());
    assert!(written(&writer).is_empty());
}
