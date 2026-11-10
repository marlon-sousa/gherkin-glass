#!/usr/bin/env bash
# Packs the binaries built for one target into its release archive, named and
# filled as docs/BEHAVIOUR.md, "Releases", rules 130 and 131, say.
# Usage: pack.sh <tag> <target> <folder to write the archive in>
# Run from the repository's root, after cargo build --release --target <target>.
set -euo pipefail

tag=$1 target=$2
mkdir -p "$3"
out=$(cd "$3" && pwd)

case $target in
    *-windows-*) ext=.exe format=zip ;;
    *) ext="" format=tar.gz ;;
esac

stage=$(mktemp -d)
trap 'rm -rf "$stage"' EXIT
for bin in $(.github/release/binaries.sh); do
    cp "target/$target/release/$bin$ext" "$stage/"
done
cp LICENSE-MIT LICENSE-APACHE "$stage/"

archive="$out/ghg-$tag-$target.$format"
rm -f "$archive"
cd "$stage"
if [ "$format" = zip ]; then
    7z a -tzip -bso0 "$archive" -- *
else
    # Keeps macOS's tar from adding AppleDouble files for extended attributes.
    COPYFILE_DISABLE=1 tar -czf "$archive" -- *
fi
echo "Packed $archive."
