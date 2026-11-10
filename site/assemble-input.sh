#!/usr/bin/env bash
# Writes the input of the project's site into a new folder: the example suite
# at its root, and every fixture not named in fixtures-left-out in
# fixtures/<name>/. docs/BEHAVIOUR.md, "The project's own site".
#
# Usage: site/assemble-input.sh <folder that does not exist yet>

set -euo pipefail

if [ "$#" -ne 1 ] || [ -z "$1" ]; then
  echo "Usage: site/assemble-input.sh <folder that does not exist yet>" >&2
  exit 2
fi
destination=$1
site=$(cd "$(dirname "$0")" && pwd)
fixtures=$(dirname "$site")/tests/fixtures

if [ -e "$destination" ]; then
  echo "$destination: already exists. Name a folder that does not exist yet." >&2
  exit 1
fi
if [ -e "$site/features/fixtures" ]; then
  echo "site/features/fixtures: the example suite may not have a folder named fixtures, since the fixtures go there." >&2
  exit 1
fi

# The names in fixtures-left-out, without comments, blank lines and spaces.
left_out=$(sed -e 's/#.*//' -e 's/^[[:space:]]*//' -e 's/[[:space:]]*$//' -e '/^$/d' "$site/fixtures-left-out")
while IFS= read -r name; do
  if [ -n "$name" ] && [ ! -d "$fixtures/$name" ]; then
    echo "site/fixtures-left-out: $name is not a folder of tests/fixtures." >&2
    exit 1
  fi
done <<<"$left_out"

mkdir -p "$destination/fixtures"
cp -R "$site/features/." "$destination/"
for folder in "$fixtures"/*/; do
  name=$(basename "$folder")
  if ! grep -qxF -- "$name" <<<"$left_out"; then
    cp -R "$folder" "$destination/fixtures/$name"
  fi
done
