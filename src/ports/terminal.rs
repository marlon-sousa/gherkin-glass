//! Port: what a command that keeps running says while it runs.

pub(crate) trait Terminal: Send + Sync {
    /// To standard output; `text` ends in a line break.
    fn say(&self, text: &str);
    /// To standard error; `text` ends in a line break.
    fn warn(&self, text: &str);
}
