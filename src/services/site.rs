//! Service: building a site, reading its input, making its files and writing them.

use std::path::Path;
use std::sync::Arc;

use crate::entities::{BuiltSite, Problem, ProblemKind, Problems, Settings, SiteFile, SiteFiles};
use crate::policies::site_files;
use crate::ports::SiteWriter;
use crate::services::InputService;

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
        Ok(site_files(&features, &settings.pages))
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
}
