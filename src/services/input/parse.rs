//! Service: parsing the collected files in parallel, and correcting their doc strings.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use gherkin::{Feature, Step};
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};

use crate::entities::{ParsedFeature, Problem};
use crate::policies::read_doc_string;
use crate::ports::{FeatureParser, InputFolder};

/// The features in the order of `files`, whatever order the threads finish
/// in, with a problem added for each file that cannot be read.
pub(super) fn parse(
    folder: &dyn InputFolder,
    parser: &dyn FeatureParser,
    input: &Path,
    files: &[PathBuf],
    problems: &mut Vec<Problem>,
) -> Vec<ParsedFeature> {
    let results: Vec<_> = files
        .par_iter()
        .map(|path| parse_file(folder, parser, input, path))
        .collect();
    let mut features = Vec::with_capacity(results.len());
    for result in results {
        match result {
            Ok(feature) => features.push(feature),
            Err(problem) => problems.push(problem),
        }
    }
    features
}

fn parse_file(
    folder: &dyn InputFolder,
    parser: &dyn FeatureParser,
    input: &Path,
    path: &Path,
) -> Result<ParsedFeature, Problem> {
    let problem = |line, message: String| Problem::new(input, path, line, message);
    let mut feature = parser
        .parse(path)
        .map_err(|failure| problem(failure.line, failure.message))?;

    // The source is read again only for a feature with doc strings.
    let mut source = None;
    let mut media_types = BTreeMap::new();
    for step in steps_mut(&mut feature) {
        if step.docstring.is_none() {
            continue;
        }
        if source.is_none() {
            let text = folder
                .read_text(path)
                .map_err(|error| problem(None, format!("cannot be read: {error}")))?;
            source = Some(text);
        }
        let source = source.as_deref().expect("read above");
        // Only a file changed since the parser read it has no doc string here.
        let read = read_doc_string(source, step.span.end)
            .ok_or_else(|| problem(None, "changed while it was being read".to_string()))?;
        if let Some(media_type) = read.media_type {
            media_types.insert(step.span.start, media_type);
        }
        step.docstring = Some(read.content);
    }

    let relative = path.strip_prefix(input).unwrap_or(path).to_path_buf();
    Ok(ParsedFeature::new(relative, feature, media_types))
}

/// Of its background, its scenarios, and its rules' backgrounds and scenarios.
fn steps_mut(feature: &mut Feature) -> impl Iterator<Item = &mut Step> {
    let Feature {
        background,
        scenarios,
        rules,
        ..
    } = feature;
    let rule_steps = rules.iter_mut().flat_map(|rule| {
        rule.background
            .iter_mut()
            .flat_map(|background| &mut background.steps)
            .chain(
                rule.scenarios
                    .iter_mut()
                    .flat_map(|scenario| &mut scenario.steps),
            )
    });
    background
        .iter_mut()
        .flat_map(|background| &mut background.steps)
        .chain(
            scenarios
                .iter_mut()
                .flat_map(|scenario| &mut scenario.steps),
        )
        .chain(rule_steps)
}
