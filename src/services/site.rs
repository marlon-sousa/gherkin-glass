//! Service: building a site, from reading its input to writing its pages, its folder pages, its folder tree's data and its assets.

use std::path::Path;
use std::sync::Arc;

use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};

use crate::entities::{Problem, ProblemKind, Problems, Settings, SiteAssets};
use crate::policies::{
    build_folder_pages, build_folder_tree, build_page, build_tag_index, build_tag_page, name_tags,
    render_folder_page, render_folder_tree, render_page, render_tag_index, render_tag_page,
    tag_index_address, tags_of,
};
use crate::entities::{ParsedFeature, Problem, ProblemKind, Problems, Settings, SiteAssets};
use crate::policies::{build_page, page_address, render_page, theme_stylesheet};
use crate::ports::SiteWriter;
use crate::services::InputService;

/// Beside the assets, where the pages' script fetches it from.
const TREE_DATA: &str = "assets/tree.json";

pub struct SiteService {
    input: InputService,
    writer: Arc<dyn SiteWriter>,
}

impl SiteService {
    pub fn new(input: InputService, writer: Arc<dyn SiteWriter>) -> Self {
        SiteService { input, writer }
    }

    /// Writes into `output`, any folder, over what is there, and only once the whole input is read.
    pub fn build(&self, settings: &Settings, output: &Path) -> Result<(), Problems> {
        let features = self.input.read(&settings.input, settings.unnamed)?;
        let tags = tags_of(&features);
        let names = name_tags(&tags);
        same_pages(&settings.input, &features)?;
        let theme = theme_stylesheet(&settings.accents);
        // Collecting into a `Vec` keeps file order, whatever order the threads finish in.
        let pages: Vec<_> = features
            .par_iter()
            .map(|parsed| build_page(parsed, &settings.pages, &names))
            .collect();
        let folders = build_folder_pages(&pages, &settings.pages);
        let mut rendered: Vec<(String, String)> = pages
            .par_iter()
            .map(|page| (page.address.clone(), render_page(page)))
            .map(|parsed| {
                let page = build_page(parsed, &settings.pages);
                let html = render_page(&page, &theme);
                (page.address, html)
            })
            .collect();
        rendered.extend(
            folders
                .par_iter()
                .map(|folder| (folder.address.clone(), render_folder_page(folder)))
                .collect::<Vec<_>>(),
        );
        rendered.push((
            TREE_DATA.to_string(),
            render_folder_tree(&build_folder_tree(&folders)),
        ));
        rendered.push((
            tag_index_address(),
            render_tag_index(&build_tag_index(&tags, &pages, &names, &settings.pages)),
        ));
        rendered.extend(
            tags.par_iter()
                .map(|tag| {
                    let page = build_tag_page(tag, &pages, &names, &settings.pages);
                    (page.address.clone(), render_tag_page(&page))
                })
                .collect::<Vec<_>>(),
        );
        let assets = SiteAssets::iter().filter_map(|name| {
            SiteAssets::get(&name).map(|file| (format!("assets/{name}"), file.data.into_owned()))
        });
        let files = rendered
            .into_iter()
            .map(|(address, html)| (address, html.into_bytes()))
            .chain(assets);
        for (file, contents) in files {
            self.writer
                .write(output, &file, &contents)
                .map_err(|error| {
                    Problems::new(vec![Problem::of_site(
                        output,
                        &file,
                        ProblemKind::CannotBeWritten {
                            reason: error.to_string(),
                        },
                    )])
                })?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::io::{Error as IoError, Result as IoResult};
    use std::path::PathBuf;
    use std::sync::Mutex;

    use gherkin::{Feature, GherkinEnv};

    use super::*;
    use crate::entities::{CollectedInput, PageOptions, ParseFailure, RawFeature, Unnamed};
    use crate::ports::{FeatureParser, InputCollector};

    struct FakeCollector(Vec<&'static str>);

    impl InputCollector for FakeCollector {
        fn collect(&self, input: &Path) -> CollectedInput {
            CollectedInput {
                paths: self.0.iter().map(|path| input.join(path)).collect(),
                problems: Vec::new(),
            }
        }
    }

    struct FakeParser;

    impl FeatureParser for FakeParser {
        fn parse(&self, path: &Path) -> Result<RawFeature, ParseFailure> {
            let source = format!("Feature: {}\n", path.display());
            let feature = Feature::parse(&source, GherkinEnv::default()).expect("parses");
            Ok(RawFeature { feature, source })
        }
    }

    /// Records every file, and fails the one named `fails`.
    #[derive(Default)]
    struct FakeWriter {
        written: Mutex<BTreeMap<String, PathBuf>>,
        fails: Option<&'static str>,
    }

    impl SiteWriter for FakeWriter {
        fn write(&self, output: &Path, file: &str, _contents: &[u8]) -> IoResult<()> {
            if self.fails == Some(file) {
                return Err(IoError::other("disk full"));
            }
            let mut written = self.written.lock().expect("not poisoned");
            written.insert(file.to_string(), output.to_path_buf());
            Ok(())
        }
    }

    fn build(paths: Vec<&'static str>, writer: Arc<FakeWriter>) -> Result<(), Problems> {
        let input = InputService::new(Arc::new(FakeCollector(paths)), Arc::new(FakeParser));
        let settings = Settings {
            input: PathBuf::from("in"),
            output: PathBuf::from("site"),
            unnamed: Unnamed::Allowed,
            pages: PageOptions::default(),
        };
        SiteService::new(input, writer).build(&settings, Path::new("elsewhere"))
    }

    #[test]
    fn every_page_and_asset_is_written_into_the_folder_given() {
        let writer = Arc::new(FakeWriter::default());
        build(vec!["a.feature", "sub/b.feature"], writer.clone()).expect("built");
        let written = writer.written.lock().expect("not poisoned");
        let files: Vec<_> = written.keys().map(String::as_str).collect();
        assert_eq!(
            files,
            [
                "a.html",
                "assets/gherkin-glass.css",
                "assets/gherkin-glass.js",
                "assets/tree.json",
                "index.html",
                "sub/b.html",
                "sub/index.html",
                "tags/index.html"
            ]
        );
        assert!(
            written
                .values()
                .all(|output| output == Path::new("elsewhere"))
        );
    }

    #[test]
    fn a_file_that_cannot_be_written_is_reported_with_the_output_folder() {
        let writer = Arc::new(FakeWriter {
            fails: Some("sub/b.html"),
            ..FakeWriter::default()
        });
        let problems = build(vec!["a.feature", "sub/b.feature"], writer).expect_err("fails");
        assert_eq!(
            problems.to_string(),
            "elsewhere/sub/b.html: cannot be written: disk full\n"
        );
    }

    #[test]
    fn a_page_in_the_folder_of_the_tag_pages_is_reported_and_nothing_is_written() {
        let writer = Arc::new(FakeWriter::default());
        let problems = build(
            vec!["Tags/a.feature", "tags.feature", "b/tags/c.feature"],
            writer.clone(),
        )
        .expect_err("in the tags folder");
        assert_eq!(
            problems.to_string(),
            "Tags/a.feature: has its page, Tags/a.html, in the folder tags\n"
        );
        assert!(writer.written.lock().expect("not poisoned").is_empty());
    }

    #[test]
    fn files_with_the_same_page_are_reported_and_nothing_is_written() {
        let writer = Arc::new(FakeWriter::default());
        let problems =
            build(vec!["a.feature", "a.story", "A"], writer.clone()).expect_err("the same page");
        assert_eq!(
            problems.to_string(),
            "A: has the same page, A.html, as a.feature\na.story: has the same page, a.html, as a.feature\n"
        );
        assert!(writer.written.lock().expect("not poisoned").is_empty());
    }

    #[test]
    fn an_empty_input_still_has_its_home_page() {
        let writer = Arc::new(FakeWriter::default());
        build(Vec::new(), writer.clone()).expect("built");
        let written = writer.written.lock().expect("not poisoned");
        assert!(written.contains_key("index.html"));
    }

    #[test]
    fn a_folder_whose_page_differs_only_in_case_is_reported_once() {
        let writer = Arc::new(FakeWriter::default());
        let problems = build(
            vec!["A/x.feature", "a/y.feature", "a/z.feature", "a/b/w.feature"],
            writer.clone(),
        )
        .expect_err("the same folder page");
        assert_eq!(
            problems.to_string(),
            "a: has the same page, a/index.html, as the folder A\n"
        );
        assert!(writer.written.lock().expect("not poisoned").is_empty());
    }

    #[test]
    fn a_feature_whose_page_is_a_folder_s_is_reported_in_any_case() {
        let writer = Arc::new(FakeWriter::default());
        let problems = build(vec!["INDEX.feature", "a/b/index.story"], writer.clone())
            .expect_err("a folder's page");
        assert_eq!(
            problems.as_slice()[0].kind,
            ProblemKind::SameAsFolderPage {
                page: "INDEX.html".into(),
                folder: None
            }
        );
        assert_eq!(
            problems.as_slice()[1].kind,
            ProblemKind::SameAsFolderPage {
                page: "a/b/index.html".into(),
                folder: Some("a/b".into())
            }
        );
        assert!(writer.written.lock().expect("not poisoned").is_empty());
    }
}
