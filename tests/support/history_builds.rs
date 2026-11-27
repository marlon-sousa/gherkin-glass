//! Test support: the versions the fixture `tests/fixtures/versions` adds in turn, each suite with the version it is added as: those of its output folder, `expected`, and those of its history, `history`. `tests/versions.rs` and `cargo xtask build` both bring it in with `#[path]`.

pub(crate) const VERSION_BUILDS: [(&str, &str); 3] = [
    ("suite", "v1.0.0"),
    ("next", "v2.0.0-rc.1"),
    ("next", "main"),
];

pub(crate) const HISTORY_BUILDS: [(&str, &str); 5] = [
    ("first", "v1.0.0"),
    ("moved", "v1.1.0"),
    ("changed", "v2.0.0"),
    ("draft", "main"),
    ("first", "main"),
];
