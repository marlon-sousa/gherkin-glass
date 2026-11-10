#!/usr/bin/env bash
# Checks cliff.toml against docs/BEHAVIOUR.md, "Releases", rule 41: the
# commits of tests/fixtures/release are made into a throwaway repository,
# before/ then the tag v0.1.0, since/ then the tag v0.2.0, then later/. The
# section git-cliff writes for v0.2.0 must be notes.md, and the one it writes
# for the commits after v0.2.0, as v0.3.0, must be notes-later.md.
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

failures=0
# expect <version> <file the notes must be>
expect() {
    (cd "$work" && git cliff --config "$root/cliff.toml" --unreleased --tag "v$1" --strip all) > "$work/$2"
    if diff -u "$fixture/$2" "$work/$2"; then
        echo "cliff.toml writes the notes for v$1 that tests/fixtures/release/$2 expects."
    else
        echo "cliff.toml does not write the notes for v$1 that tests/fixtures/release/$2 expects; the difference is above." >&2
        failures=$((failures + 1))
    fi
}

expect 0.2.0 notes.md
git -C "$work" tag v0.2.0
commit_each "$fixture/later"
expect 0.3.0 notes-later.md
[ "$failures" -eq 0 ]
