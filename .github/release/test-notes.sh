#!/usr/bin/env bash
# Checks cliff.toml against docs/BEHAVIOUR.md, "Releases", rule 41: the
# commits of tests/fixtures/release are made into a throwaway repository,
# those in before/ ending at the previous tag, and the section git-cliff
# writes for the next version must be tests/fixtures/release/notes.md.
# Needs git and git-cliff 2.14.2. Run from the repository's root.
set -euo pipefail

root=$PWD
fixture=$root/tests/fixtures/release
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

git -C "$work" init --quiet
git -C "$work" config user.name "Release fixture"
git -C "$work" config user.email "fixture@example.invalid"
git -C "$work" config commit.gpgsign false
git -C "$work" config tag.gpgsign false

commit_each() {
    local message
    for message in "$1"/*.txt; do
        git -C "$work" commit --quiet --allow-empty --cleanup=verbatim --file="$message"
    done
}

commit_each "$fixture/before"
git -C "$work" tag v0.1.0
commit_each "$fixture/since"

(cd "$work" && git cliff --config "$root/cliff.toml" --unreleased --tag v0.2.0 --strip all) > "$work/notes.md"
if diff -u "$fixture/notes.md" "$work/notes.md"; then
    echo "cliff.toml writes the notes tests/fixtures/release/notes.md expects."
else
    echo "cliff.toml does not write the notes tests/fixtures/release/notes.md expects; the difference is above." >&2
    exit 1
fi
