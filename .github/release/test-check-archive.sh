#!/usr/bin/env bash
# Checks check-archive.sh against docs/BEHAVIOUR.md, "Releases", rules 37 and
# 38, with archives the test makes: one named and filled as the rules say,
# which must pass, and others that break them, which must fail.
# Needs cargo, jq and tar. Run from the repository's root.
set -euo pipefail

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
target=x86_64-unknown-linux-musl
failures=0

# archive <name> <file>...: packs the files, made in a folder of their own, into <name>.
archive() {
    local name=$1 stage file
    shift
    stage=$(mktemp -d -p "$work")
    for file in "$@"; do
        mkdir -p "$stage/$(dirname "$file")"
        echo "$file" > "$stage/$file"
    done
    (cd "$stage" && tar -czf "$work/$name" -- *)
}

# expect <status> <archive> <what it is>
expect() {
    local actual=0
    .github/release/check-archive.sh "$work/$2" v1.2.3 "$target" > /dev/null 2>&1 || actual=$?
    if [ "$actual" != "$1" ]; then
        echo "check-archive.sh exited with $actual, not $1, for $3." >&2
        failures=$((failures + 1))
    fi
}

mapfile -t binaries < <(.github/release/binaries.sh)
right="ghg-v1.2.3-$target.tar.gz"
mkdir -p "$work/extra" "$work/folder" "$work/missing" "$work/zip"

archive "$right" "${binaries[@]}" LICENSE-MIT LICENSE-APACHE
expect 0 "$right" "an archive as the rules say"

archive "extra/$right" "${binaries[@]}" LICENSE-MIT LICENSE-APACHE README.md
expect 1 "extra/$right" "an archive holding a file more"

archive "folder/$right" "${binaries[@]/#/ghg/}" ghg/LICENSE-MIT ghg/LICENSE-APACHE
expect 1 "folder/$right" "an archive whose files are inside a folder"

archive "missing/$right" "${binaries[@]}" LICENSE-MIT
expect 1 "missing/$right" "an archive without a licence file"

archive "ghg-1.2.3-$target.tar.gz" "${binaries[@]}" LICENSE-MIT LICENSE-APACHE
expect 1 "ghg-1.2.3-$target.tar.gz" "an archive named without the tag's v"

archive "zip/ghg-v1.2.3-$target.zip" "${binaries[@]}" LICENSE-MIT LICENSE-APACHE
expect 1 "zip/ghg-v1.2.3-$target.zip" "a Linux archive named as a zip"

if [ "$failures" -gt 0 ]; then
    echo "$failures checks of check-archive.sh failed." >&2
    exit 1
fi
echo "check-archive.sh accepts the archive the rules describe and refuses each that breaks them."
