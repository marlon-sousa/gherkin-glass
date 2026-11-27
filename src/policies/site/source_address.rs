//! Policy: the address of a feature file's source, from the template of `source_link`, which a feature's page links and the store records.

use crate::entities::{FeaturePath, percent_encoded};

/// `template` with every `{path}` replaced by `path`, percent-encoded, and every `{line}` by
/// `line`, the line of the feature's keyword.
pub(crate) fn source_address(template: &str, path: &FeaturePath, line: usize) -> String {
    template
        .replace("{path}", &percent_encoded(path.as_str()))
        .replace("{line}", &line.to_string())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    #[test]
    fn the_path_is_encoded_and_the_line_put_in_every_place() {
        let path = FeaturePath::below_input(Path::new("billing/a b#ç.feature"));
        assert_eq!(
            source_address("https://x/{path}#L{line}?p={path}", &path, 3),
            "https://x/billing/a%20b%23%C3%A7.feature#L3?p=billing/a%20b%23%C3%A7.feature"
        );
        assert_eq!(source_address("https://x/", &path, 3), "https://x/");
    }
}
