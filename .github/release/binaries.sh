#!/usr/bin/env bash
# Prints the name of every binary the gherkin-glass package declares, one per
# line, as Cargo.toml's [[bin]] tables give them.
set -euo pipefail
cargo metadata --no-deps --format-version 1 \
    | jq -r '.packages[] | select(.name == "gherkin-glass") | .targets[] | select(.kind | index("bin")) | .name' \
    | tr -d '\r' \
    | LC_ALL=C sort
