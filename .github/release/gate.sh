#!/usr/bin/env bash
# Refuses a tag that may not become a release, as docs/BEHAVIOUR.md,
# "Releases", rule 36, says: the tag must be "v" and the version in
# Cargo.toml, its commit must be on main, and CHANGELOG.md must have notes for
# the version. Every reason is reported, not only the first.
# Usage: gate.sh <tag> <commit>
# Run from the repository's root, in a clone that has fetched main.
set -euo pipefail

tag=$1 commit=$2
version=$("$(dirname "$0")/version.sh")
problems=()

if [ "$tag" != "v$version" ]; then
    problems+=("The tag $tag does not match the version $version in Cargo.toml; it must be v$version.")
fi
if ! git merge-base --is-ancestor "$commit" origin/main; then
    problems+=("The tagged commit $commit is not on main.")
fi
# The section runs from its heading to the next heading of level 1 or 2, or
# to the link definitions that close the file, as create-release reads it.
notes=$(awk -v heading="## [$version]" '
    $0 == heading || index($0, heading " ") == 1 { inside = 1; next }
    inside && (/^##?[ \t]/ || /^\[[^]]+\]:[ \t]/) { exit }
    inside && NF { print }
' CHANGELOG.md)
if [ -z "$notes" ]; then
    problems+=("CHANGELOG.md has no notes for $version, under the heading \"## [$version]\".")
fi

if [ ${#problems[@]} -gt 0 ]; then
    echo "The release gate refused $tag:"
    for problem in "${problems[@]}"; do
        echo "::error::$problem"
    done
    exit 1
fi
echo "$tag may be released."
