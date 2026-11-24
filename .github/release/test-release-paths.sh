#!/usr/bin/env bash
# Checks release-paths.sh with the lists of changed paths below.
# Run from the repository's root.
set -euo pipefail

failures=0
# expect <expected output> <path>...
expect() {
    local expected=$1 actual
    shift
    actual=$(printf '%s\n' "$@" | .github/release/release-paths.sh)
    if [ "$actual" != "$expected" ]; then
        echo "$*: printed \"$actual\", expected \"$expected\"" >&2
        failures=$((failures + 1))
    fi
}

# Each path that changes how a release is made.
expect true .github/workflows/release.yml
expect true .github/release/pack.sh
expect true tests/fixtures/release/changelogs/one.md
expect true cliff.toml
expect true Cargo.toml
expect true Cargo.lock
expect true rust-toolchain.toml
expect true build.rs
expect true .cargo/config.toml
expect true src/main.rs
expect true src/cargo_ghg.rs
# One among others is enough.
expect true README.md src/lib.rs Cargo.lock
# Paths that do not, those that only look alike included.
expect false README.md
expect false src/lib.rs src/services/site_writer.rs
expect false .github/workflows/rust.yml
expect false tests/fixtures/pages/a.feature
expect false xtask/Cargo.toml
expect false src/main.rs.orig
expect false docs/Cargo.toml.md
# No path at all.
expect false ""

if [ "$failures" -gt 0 ]; then
    echo "$failures cases of release-paths.sh failed." >&2
    exit 1
fi
echo "release-paths.sh tells every path list as expected."
