#!/usr/bin/env bash
# Prints the version Cargo.toml gives the gherkin-glass package.
set -euo pipefail
cargo metadata --no-deps --format-version 1 \
    | jq -r '.packages[] | select(.name == "gherkin-glass") | .version' \
    | tr -d '\r'
