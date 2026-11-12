//! Service: the gherkin parser, parsing the feature files it is given, each on its own.

use std::path::Path;
use std::sync::Arc;

use rayon::prelude::{IntoParallelIterator, ParallelIterator};

use crate::entities::{FileToParse, ParsedFeature, ParsedFile, Unnamed};
use crate::errors::{InputError, InputErrorKind};
use crate::policies::{correct_doc_strings, correct_step_keywords, file_language, unnamed_blocks};
use crate::ports::FeatureParser;

/// What the controller asks of the gherkin parser.
pub(crate) trait GherkinParserApi: Send + Sync {
    /// What parsing each of `files`, under `input`, gave, in their order: its feature, or its
    /// errors, among them, when `unnamed` is refused, every block that needs a name and has none.
    fn parse(&self, input: &Path, files: Vec<FileToParse>, unnamed: Unnamed) -> Vec<ParsedFile>;
}

#[derive(Clone)]
pub(crate) struct GherkinParser {
    feature_parser: Arc<dyn FeatureParser>,
}

impl GherkinParser {
    pub(crate) fn new(feature_parser: Arc<dyn FeatureParser>) -> Self {
        GherkinParser { feature_parser }
    }

    fn parse_file(
        &self,
        input: &Path,
        path: &Path,
        unnamed: Unnamed,
    ) -> Result<ParsedFeature, Vec<InputError>> {
        let input_error = |line, kind| InputError::new(input, path, line, kind);
        let mut raw = self
            .feature_parser
            .parse(path)
            .map_err(|error| vec![input_error(error.line, error.kind)])?;
        // Only a file changed between the parse and the read of its source fails here.
        let media_types = correct_doc_strings(&mut raw.feature, &raw.source)
            .ok_or_else(|| vec![input_error(None, InputErrorKind::Changed)])?;
        if unnamed == Unnamed::Refused {
            let found: Vec<_> = unnamed_blocks(&raw.feature)
                .into_iter()
                .map(|(line, block)| input_error(Some(line), InputErrorKind::Unnamed { block }))
                .collect();
            if !found.is_empty() {
                return Err(found);
            }
        }
        let relative = path.strip_prefix(input).unwrap_or(path).to_path_buf();
        correct_step_keywords(&mut raw.feature, &raw.source);
        let language = file_language(&raw.source);
        Ok(ParsedFeature::new(
            relative,
            raw.feature,
            language,
            media_types,
        ))
    }

    fn parsed_file(&self, input: &Path, file: FileToParse, unnamed: Unnamed) -> ParsedFile {
        let result = self.parse_file(input, &file.path, unnamed);
        ParsedFile { file, result }
    }
}

impl GherkinParserApi for GherkinParser {
    fn parse(&self, input: &Path, files: Vec<FileToParse>, unnamed: Unnamed) -> Vec<ParsedFile> {
        // Collecting into a `Vec` keeps file order, whatever order the threads finish in.
        files
            .into_par_iter()
            .map(|file| self.parsed_file(input, file, unnamed))
            .collect()
    }
}

#[cfg(test)]
mod tests;
