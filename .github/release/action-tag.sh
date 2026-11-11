#!/usr/bin/env bash
# Prints the tag of the GitHub Action's major version that the release of
# <tag> moves, such as v1 for v1.2.0, or nothing when it moves none, as
# docs/BEHAVIOUR.md, "The GitHub Action", rule 177, says. The repository's
# tags, one per line, come on standard input.
# Usage: action-tag.sh <tag> < tags
set -euo pipefail

tag=$1
if ! [[ $tag =~ ^v([1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$ ]]; then
    exit 0
fi
major=${BASH_REMATCH[1]}
newest=$( (cat; echo "$tag") |
    tr -d '\r' |
    grep -E "^v$major\\.(0|[1-9][0-9]*)\\.(0|[1-9][0-9]*)$" |
    sed 's/^v//' |
    sort -t . -k 1,1n -k 2,2n -k 3,3n |
    tail -n 1)
if [ "v$newest" = "$tag" ]; then
    echo "v$major"
fi
