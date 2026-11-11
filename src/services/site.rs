//! Service: building a site, from reading its input to writing its pages, its folder pages, its page of step sentences, its folder tree's data and its assets.

use std::path::Path;
use std::sync::Arc;

use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};

use crate::entities::{
    ASSETS, BuiltSite, Problem, ProblemKind, Problems, Settings, SiteAddress, SiteAssets, SiteFile,
};
use crate::policies::{
    LARGE_SEARCH_DOCUMENTS, SENTENCES_PAGE, anchor_feature, build_folder_pages, build_folder_tree,
    build_page, build_sentences_page, build_tag_index, build_tag_page, find_step_sentences,
    name_tags, render_folder_page, render_folder_tree, render_page, render_sentences_page,
    render_tag_index, render_tag_page, search_documents, search_json, tag_index_address, tags_of,
};
use crate::ports::SiteWriter;
use crate::services::InputService;

/// In the folder of the assets, where the pages' script fetches them from.
const TREE_DATA: &str = "tree.json";
const SEARCH_DOCUMENTS: &str = "search.json";

/// Every file of a site, and what the build has to say of them.
pub(crate) struct SiteFiles {
    pub(crate) files: Vec<SiteFile>,
    pub(crate) built: BuiltSite,
}

pub(crate) struct SiteService {
    input: InputService,
    writer: Arc<dyn SiteWriter>,
}

impl SiteService {
    pub(crate) fn new(input: InputService, writer: Arc<dyn SiteWriter>) -> Self {
        SiteService { input, writer }
    }

    /// Writes into `output`, any folder, over what is there, and only once the
    /// whole input is read. Whether a folder on disk may be written into is
    /// [`crate::services::OutputService`]'s to decide.
    pub(crate) fn build(&self, settings: &Settings, output: &Path) -> Result<BuiltSite, Problems> {
        let site = self.files(settings)?;
        self.write(output, site.files)?;
        Ok(site.built)
    }

    /// Every file of the site, pages first, in file order. Nothing is written.
    pub(crate) fn files(&self, settings: &Settings) -> Result<SiteFiles, Problems> {
        let features = self.input.read(&settings.input, settings.unnamed)?;
        let tags = tags_of(&features);
        let names = name_tags(&tags);
        let anchored: Vec<_> = features.iter().map(anchor_feature).collect();
        // Collecting into a `Vec` keeps file order, whatever order the threads finish in.
        let pages: Vec<_> = anchored
            .par_iter()
            .map(|feature| build_page(feature, &settings.pages, &names))
            .collect();
        let folders = build_folder_pages(&pages, &settings.pages);
        let mut files: Vec<SiteFile> = pages
            .par_iter()
            .map(|page| SiteFile::text(page.address.clone(), render_page(page)))
            .collect();
        files.extend(
            folders
                .par_iter()
                .map(|folder| SiteFile::text(folder.address.clone(), render_folder_page(folder)))
                .collect::<Vec<_>>(),
        );
        let sentences = find_step_sentences(&anchored);
        files.push(SiteFile::text(
            SiteAddress::at_root(SENTENCES_PAGE),
            render_sentences_page(&build_sentences_page(
                &features,
                &sentences,
                &settings.pages,
            )),
        ));
        files.push(SiteFile::text(
            asset(TREE_DATA),
            render_folder_tree(&build_folder_tree(&folders)),
        ));
        files.push(SiteFile::text(
            tag_index_address(),
            render_tag_index(&build_tag_index(&tags, &pages, &names, &settings.pages)),
        ));
        files.extend(
            tags.par_iter()
                .map(|tag| {
                    let page = build_tag_page(tag, &pages, &names, &settings.pages);
                    SiteFile::text(page.address.clone(), render_tag_page(&page))
                })
                .collect::<Vec<_>>(),
        );
        let search = search_json(&search_documents(&anchored));
        let built = BuiltSite {
            large_search_documents: Some(search.len())
                .filter(|&size| size > LARGE_SEARCH_DOCUMENTS),
        };
        files.push(SiteFile::text(asset(SEARCH_DOCUMENTS), search));
        files.extend(assets());
        Ok(SiteFiles { files, built })
    }

    /// Writes `files` into `output`, stopping at the first that cannot be written.
    pub(crate) fn write(
        &self,
        output: &Path,
        files: impl IntoIterator<Item = SiteFile>,
    ) -> Result<(), Problems> {
        for file in files {
            let address = file.address.as_str();
            self.writer
                .write(output, address, &file.contents)
                .map_err(|error| {
                    Problems::new(vec![Problem::of_site(
                        output,
                        address,
                        ProblemKind::CannotBeWritten {
                            reason: error.to_string(),
                        },
                    )])
                })?;
        }
        Ok(())
    }
}

/// The assets every site has.
pub(crate) fn assets() -> impl Iterator<Item = SiteFile> {
    SiteAssets::iter().filter_map(|name| {
        SiteAssets::get(&name).map(|file| SiteFile {
            address: asset(&name),
            contents: file.data.into_owned(),
        })
    })
}

/// The address of the file named `name` in the folder of the assets.
fn asset(name: &str) -> SiteAddress {
    SiteAddress::of(&[ASSETS], name)
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

    fn build(paths: Vec<&'static str>, writer: Arc<FakeWriter>) -> Result<BuiltSite, Problems> {
        let input = InputService::new(Arc::new(FakeCollector(paths)), Arc::new(FakeParser));
        let settings = Settings {
            input: PathBuf::from("in"),
            output: PathBuf::from("site"),
            unnamed: Unnamed::Allowed,
            pages: PageOptions::default(),
            version: None,
            replace: true,
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
                "assets/search.json",
                "assets/tree.json",
                "index.html",
                "steps.html",
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
    fn only_search_documents_above_the_size_are_reported() {
        let built = build(vec!["a.feature"], Arc::default()).expect("built");
        assert_eq!(built.large_search_documents, None);
        // The feature's name is its path, and its page's address almost the same.
        let long: &'static str = String::leak("a".repeat(LARGE_SEARCH_DOCUMENTS / 2));
        let built = build(vec![long], Arc::default()).expect("built");
        let size = built.large_search_documents.expect("reported");
        assert!(size > LARGE_SEARCH_DOCUMENTS);
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
    fn a_file_whose_page_is_the_page_of_step_sentences_is_reported() {
        let writer = Arc::new(FakeWriter::default());
        let problems = build(
            vec![
                "Steps.feature",
                "a.feature",
                "steps.feature",
                "sub/steps.feature",
            ],
            writer.clone(),
        )
        .expect_err("the same page");
        assert_eq!(
            problems.to_string(),
            "Steps.feature: has the same page, steps.html, as the step sentences\n\
             steps.feature: has the same page, steps.html, as the step sentences\n"
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
