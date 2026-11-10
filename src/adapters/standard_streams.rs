//! Adapter: the terminal port over standard output and standard error.

use std::io::{Write, stderr, stdout};

use crate::ports::Terminal;

#[derive(Debug, Default, Clone, Copy)]
pub struct StandardStreams;

impl Terminal for StandardStreams {
    fn say(&self, text: &str) {
        // A closed pipe or a full disk leaves nothing more to say.
        let mut out = stdout().lock();
        let _ = out.write_all(text.as_bytes());
        let _ = out.flush();
    }

    fn warn(&self, text: &str) {
        let _ = stderr().lock().write_all(text.as_bytes());
    }
}
