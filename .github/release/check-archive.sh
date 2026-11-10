#!/usr/bin/env bash
# Checks a release archive against docs/BEHAVIOUR.md, "Releases", rules 130
# and 131: its name, and that it holds every binary the package declares and
# the licence files at its root, and nothing else.
# Usage: check-archive.sh <archive> <tag> <target>
# Run from the repository's root.
set -euo pipefail

archive=$1 tag=$2 target=$3
case $target in
    *-windows-*) ext=.exe format=zip ;;
    *) ext="" format=tar.gz ;;
esac

expected_name="ghg-$tag-$target.$format"
if [ "$(basename "$archive")" != "$expected_name" ]; then
    echo "The archive is named $(basename "$archive"), not $expected_name." >&2
    exit 1
fi

expected=$({ .github/release/binaries.sh | sed "s/\$/$ext/"; printf 'LICENSE-APACHE\nLICENSE-MIT\n'; } | LC_ALL=C sort)
if [ "$format" = zip ]; then
    # 7-Zip ends its lines in a carriage return on Windows, and may list the
    # archive itself, by the path it was given, among what it holds.
    held=$(7z l -ba -slt "$archive" | tr -d '\r' | sed -n 's/^Path = //p' \
        | { grep -vxF -e "$archive" -e "${archive//\//\\}" || true; } | LC_ALL=C sort)
else
    held=$(tar -tzf "$archive" | LC_ALL=C sort)
fi
if [ "$held" != "$expected" ]; then
    echo "$expected_name holds:" >&2
    echo "$held" >&2
    echo "It should hold exactly:" >&2
    echo "$expected" >&2
    exit 1
fi
echo "$expected_name holds every binary and the licence files, and nothing else."
