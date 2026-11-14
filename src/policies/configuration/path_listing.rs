//! Policy: the paths a list of feature files holds, one on each line.

use std::collections::BTreeSet;
use std::path::PathBuf;

/// Each line of `text` that is not empty, as written, without the carriage return before its
/// line feed.
pub(crate) fn listed_paths(text: &str) -> BTreeSet<PathBuf> {
    text.lines().filter(is_written).map(PathBuf::from).collect()
}

fn is_written(line: &&str) -> bool {
    !line.is_empty()
}

#[cfg(test)]
mod tests {
    use proptest::prelude::proptest;

    use super::*;

    // Rule 204.
    #[test]
    fn each_line_that_is_not_empty_is_a_path_as_written() {
        let paths = listed_paths("a.feature\r\n\n b c.feature \nsub/d.feature");
        let expected: BTreeSet<PathBuf> = ["a.feature", " b c.feature ", "sub/d.feature"]
            .into_iter()
            .map(PathBuf::from)
            .collect();
        assert_eq!(paths, expected);
    }

    proptest! {
        // Rule 204: any lines, joined by a carriage return and a line feed, are read as the
        // paths they write, but the empty ones.
        #[test]
        fn any_list_is_read_as_its_lines(lines in proptest::collection::vec("[^\r\n]{0,12}", 0..8)) {
            let paths = listed_paths(&lines.join("\r\n"));
            let expected: BTreeSet<PathBuf> = lines
                .iter()
                .filter(|line| !line.is_empty())
                .map(PathBuf::from)
                .collect();
            assert_eq!(paths, expected);
        }
    }
}
