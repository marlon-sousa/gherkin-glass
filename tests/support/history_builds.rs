//! Test support: the builds of the history of versions, `tests/fixtures/versions/history`, in order: each suite, and the version it is built as. `tests/versions.rs` and `cargo xtask build` both bring it in with `#[path]`.

pub(crate) const HISTORY_BUILDS: [(&str, &str); 5] = [
    ("first", "v1.0.0"),
    ("moved", "v1.1.0"),
    ("changed", "v2.0.0"),
    ("draft", "main"),
    ("first", "main"),
];
