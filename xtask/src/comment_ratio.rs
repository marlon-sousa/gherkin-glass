//! Policy: how many lines of a Rust or TypeScript source are comments, and how many are code.

/// The comment lines and the code lines of a source, leaving out blank lines.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Lines {
    pub(crate) comment: usize,
    pub(crate) code: usize,
}

impl Lines {
    /// Counts `source`'s lines, a line that starts with `//` or that is in a `/* */` block being
    /// a comment.
    pub(crate) fn of(source: &str) -> Lines {
        let (lines, _) = source
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .fold((Lines::default(), false), count_line);
        lines
    }

    /// The lines of all of `counts`.
    pub(crate) fn total(counts: &[Lines]) -> Lines {
        Lines {
            comment: counts.iter().map(|lines| lines.comment).sum(),
            code: counts.iter().map(|lines| lines.code).sum(),
        }
    }

    /// `name: 3 comment, 40 code, 0.07 per code line`.
    pub(crate) fn line(self, name: &str) -> String {
        let ratio = self.comment as f64 / self.code.max(1) as f64;
        format!(
            "{name}: {} comment, {} code, {ratio:.2} per code line",
            self.comment, self.code
        )
    }
}

/// `lines` with `line` counted, and whether a `/* */` block is still open after it, given
/// whether one was open before it.
fn count_line((lines, in_block): (Lines, bool), line: &str) -> (Lines, bool) {
    let comment = Lines {
        comment: lines.comment + 1,
        ..lines
    };
    if in_block {
        (comment, !line.contains("*/"))
    } else if line.starts_with("//") {
        (comment, false)
    } else if let Some(rest) = line.strip_prefix("/*") {
        (comment, !rest.contains("*/"))
    } else {
        let code = Lines {
            code: lines.code + 1,
            ..lines
        };
        (code, false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn comment_lines_are_those_of_line_and_block_comments() {
        let source = "//! Role.\n\nfn a() {}\n/* one\n two */\n/// Doc.\nfn b() {} // trailing\n";
        assert_eq!(
            Lines::of(source),
            Lines {
                comment: 4,
                code: 2
            }
        );
    }

    #[test]
    fn a_one_line_block_comment_ends_where_it_starts() {
        assert_eq!(
            Lines::of("/* one */\nfn a() {}\n"),
            Lines {
                comment: 1,
                code: 1
            }
        );
    }

    #[test]
    fn the_total_adds_each_kind_of_line() {
        let counts = [
            Lines {
                comment: 1,
                code: 2,
            },
            Lines {
                comment: 3,
                code: 4,
            },
        ];
        assert_eq!(
            Lines::total(&counts),
            Lines {
                comment: 4,
                code: 6
            }
        );
    }

    #[test]
    fn the_line_shows_the_comment_lines_for_each_line_of_code() {
        let lines = Lines {
            comment: 3,
            code: 40,
        };
        assert_eq!(
            lines.line("a.rs"),
            "a.rs: 3 comment, 40 code, 0.07 per code line"
        );
    }

    #[test]
    fn a_source_with_no_code_counts_its_comments_for_one_line_of_code() {
        let lines = Lines {
            comment: 2,
            code: 0,
        };
        assert_eq!(
            lines.line("b.rs"),
            "b.rs: 2 comment, 0 code, 2.00 per code line"
        );
    }
}
