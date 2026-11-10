#!/usr/bin/env bash
# Checks the archives of a release, and what Cargo.toml tells cargo binstall
# about them, against docs/BEHAVIOUR.md, "Releases", rules 37 and 44: there is
# one archive for each of the five targets, and for each target the address
# [package.metadata.binstall] gives is that archive's on the release of the
# tag, with every binary at the archive's root.
# Usage: check-binstall.sh <tag> <folder holding the archives>
# Run from the repository's root.
set -euo pipefail

tag=$1 dist=$2
targets="aarch64-apple-darwin aarch64-unknown-linux-musl x86_64-apple-darwin x86_64-pc-windows-msvc x86_64-unknown-linux-musl"
package=$(cargo metadata --no-deps --format-version 1 | jq '.packages[] | select(.name == "gherkin-glass")')
version=$(jq -r .version <<< "$package")
repository=$(jq -r .repository <<< "$package")
failures=0

held=$(cd "$dist" && printf "%s\n" ghg-* | LC_ALL=C sort)
expected=$(for target in $targets; do
    case $target in *-windows-*) echo "ghg-$tag-$target.zip" ;; *) echo "ghg-$tag-$target.tar.gz" ;; esac
done | LC_ALL=C sort)
if [ "$held" != "$expected" ]; then
    echo "The release has these archives:" >&2
    echo "$held" >&2
    echo "It should have exactly one for each target:" >&2
    echo "$expected" >&2
    failures=$((failures + 1))
fi

# A target's setting is its override's, if it has one, or else the general one.
setting() {
    jq -r --arg target "$1" --arg key "$2" \
        '.metadata.binstall | (.overrides[$target][$key] // .[$key]) // empty' <<< "$package"
}

for target in $targets; do
    url=$(setting "$target" pkg-url)
    url=${url//"{ repo }"/$repository}
    url=${url//"{ version }"/$version}
    url=${url//"{ target }"/$target}
    case $target in *-windows-*) format=zip ;; *) format=tgz ;; esac
    archive=$(grep -F -- "-$target." <<< "$expected")
    if [ "$url" != "$repository/releases/download/$tag/$archive" ]; then
        echo "cargo binstall looks for $target at $url, not at $repository/releases/download/$tag/$archive." >&2
        failures=$((failures + 1))
    fi
    if [ "$(setting "$target" pkg-fmt)" != "$format" ]; then
        echo "cargo binstall reads the archive for $target as $(setting "$target" pkg-fmt), not as $format." >&2
        failures=$((failures + 1))
    fi
    if [ "$(setting "$target" bin-dir)" != "{ bin }{ binary-ext }" ]; then
        echo "cargo binstall looks for the binaries for $target at $(setting "$target" bin-dir), not at the archive's root." >&2
        failures=$((failures + 1))
    fi
done

if [ "$failures" -gt 0 ]; then
    exit 1
fi
echo "There is an archive for each target, where cargo binstall looks for it."
