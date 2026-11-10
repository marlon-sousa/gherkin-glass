//! Service: reading the input, the collect and parse steps of a build.

use std::path::Path;
use std::sync::Arc;

use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};

use crate::entities::{ParsedFeature, Problem, Problems};
use crate::policies::correct_doc_strings;
use crate::ports::{FeatureParser, InputCollector};

pub struct InputService {
    collector: Arc<dyn InputCollector>,
    parser: Arc<dyn FeatureParser>,
}

impl InputService {
    pub fn new(collector: Arc<dyn InputCollector>, parser: Arc<dyn FeatureParser>) -> Self {
        InputService { collector, parser }
    }

    /// The features under `input`, sorted by path, or every problem found,
    /// those met while collecting and those of every file parsed.
    pub fn read(&self, input: &Path) -> Result<Vec<ParsedFeature>, Problems> {
        let collected = self.collector.collect(input);
        let mut problems = collected.problems;
        // Collecting into a `Vec` keeps file order, whatever order the threads finish in.
        let results: Vec<_> = collected
            .paths
            .par_iter()
            .map(|path| self.parse(input, path))
            .collect();
        let mut features = Vec::with_capacity(results.len());
        for result in results {
            match result {
                Ok(feature) => features.push(feature),
                Err(problem) => problems.push(problem),
            }
        }
        if problems.is_empty() {
            Ok(features)
        } else {
            Err(Problems::new(problems))
        }
    }

    fn parse(&self, input: &Path, path: &Path) -> Result<ParsedFeature, Problem> {
        let problem = |line, message: String| Problem::new(input, path, line, message);
        let mut raw = self
            .parser
            .parse(path)
            .map_err(|failure| problem(failure.line, failure.message))?;
        // Only a file changed between the parse and the read of its source fails here.
        let media_types = correct_doc_strings(&mut raw.feature, &raw.source)
            .ok_or_else(|| problem(None, "changed while it was being read".to_string()))?;
        let relative = path.strip_prefix(input).unwrap_or(path).to_path_buf();
        Ok(ParsedFeature::new(relative, raw.feature, media_types))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    use gherkin::{Feature, GherkinEnv};

    use super::*;
    use crate::entities::{CollectedInput, ParseFailure, RawFeature};

    struct FakeCollector(CollectedInput);

    impl InputCollector for FakeCollector {
        fn collect(&self, _input: &Path) -> CollectedInput {
            self.0.clone()
        }
    }

    /// Parses the text held for a path; a path with no text cannot be opened.
    struct FakeParser(BTreeMap<PathBuf, String>);

    impl FeatureParser for FakeParser {
        fn parse(&self, path: &Path) -> Result<RawFeature, ParseFailure> {
            let source = self.0.get(path).cloned().ok_or_else(|| ParseFailure {
                line: None,
                message: "cannot be read: permission denied".to_string(),
            })?;
            let feature =
                Feature::parse(&source, GherkinEnv::default()).map_err(|_| ParseFailure {
                    line: Some(1),
                    message: "Syntax error.".to_string(),
                })?;
            Ok(RawFeature { feature, source })
        }
    }

    fn read(
        paths: &[&str],
        problems: Vec<Problem>,
        texts: &[(&str, &str)],
    ) -> Result<Vec<ParsedFeature>, Problems> {
        let collector = FakeCollector(CollectedInput {
            paths: paths.iter().map(PathBuf::from).collect(),
            problems,
        });
        let parser = FakeParser(
            texts
                .iter()
                .map(|(path, text)| (PathBuf::from(path), text.to_string()))
                .collect(),
        );
        InputService::new(Arc::new(collector), Arc::new(parser)).read(Path::new("in"))
    }

    #[test]
    fn the_problems_of_collecting_and_parsing_are_reported_together_sorted() {
        // How the service combines the collector's problems with the parser's;
        // the messages are the fakes' own.
        let locked = Problem::new(
            Path::new("in"),
            Path::new("in/locked"),
            None,
            "cannot be listed: permission denied",
        );
        let problems = read(
            &["in/a.feature", "in/b.feature", "in/c.feature"],
            vec![locked],
            &[
                ("in/a.feature", "Feature: A\n"),
                ("in/c.feature", "nothing\n"),
            ],
        )
        .expect_err("problems");
        assert_eq!(
            problems.to_string(),
            "b.feature: cannot be read: permission denied\n\
             c.feature:1: Syntax error.\n\
             locked: cannot be listed: permission denied\n"
        );
    }

    #[test]
    fn features_keep_the_order_they_were_collected_in() {
        let texts: Vec<_> = (0..50)
            .map(|n| (format!("in/{n:02}.feature"), format!("Feature: {n}\n")))
            .collect();
        let paths: Vec<_> = texts.iter().map(|(path, _)| path.as_str()).collect();
        let texts: Vec<_> = texts
            .iter()
            .map(|(p, t)| (p.as_str(), t.as_str()))
            .collect();
        let features = read(&paths, Vec::new(), &texts).expect("read");
        let names: Vec<_> = features.iter().map(|f| f.feature.name.clone()).collect();
        let expected: Vec<_> = (0..50).map(|n| n.to_string()).collect();
        assert_eq!(names, expected);
    }

    #[test]
    fn a_doc_string_is_corrected_and_its_path_made_relative() {
        let features = read(
            &["in/sub/a.feature"],
            Vec::new(),
            &[(
                "in/sub/a.feature",
                "Feature: A\n  Scenario: S\n    Given text\n      \"\"\"md\n      Hi\n      \"\"\"\n",
            )],
        )
        .expect("read");
        let parsed = &features[0];
        let step = &parsed.feature.scenarios[0].steps[0];
        assert_eq!(parsed.path, Path::new("sub/a.feature"));
        assert_eq!(step.docstring.as_deref(), Some("Hi"));
        assert_eq!(parsed.media_type(step), Some("md"));
    }
}
