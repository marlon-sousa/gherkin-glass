#!/usr/bin/env bash
# Reads the paths a pull request changes, one on each line, and prints true
# when any of them changes how a release is made, or false: the release
# workflow builds and checks every archive only then (CONTRIBUTING.md,
# "Releases").
# Usage: git diff --name-only <base>...<head> | release-paths.sh
set -euo pipefail

releasing='^(\.github/workflows/release\.yml|\.github/release/.*|tests/fixtures/release/.*|cliff\.toml|Cargo\.toml|Cargo\.lock|rust-toolchain\.toml|build\.rs|\.cargo/.*|src/main\.rs|src/cargo_ghg\.rs)$'
# grep reads every line, so that the command writing them never meets a
# closed pipe.
if grep -E "$releasing" > /dev/null; then
    echo true
else
    echo false
fi
