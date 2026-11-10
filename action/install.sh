#!/usr/bin/env bash
# Adapter: installs ghg on the runner, for the GitHub Action in action.yml.
#
# Arguments: the base address of GitHub's API, and that of its downloads.
# Environment: GHG_VERSION, GHG_BUILD and GHG_TOKEN, from action.yml;
# GHG_ACTION_PATH, the action's own folder, whose Cargo.toml gives its
# version; and the runner's RUNNER_OS, RUNNER_ARCH, RUNNER_TOOL_CACHE,
# RUNNER_TEMP, GITHUB_PATH and GITHUB_OUTPUT.
# Writes bin=<path of ghg> to GITHUB_OUTPUT.

set -euo pipefail

api_url=$1
download_url=$2
repository=marlon-sousa/gherkin-glass

fail() {
  echo "::error::$1"
  exit 1
}

case "${GHG_BUILD:-false}" in
  true | false) ;;
  *) fail "build must be true or false." ;;
esac

case "${RUNNER_OS:-}/${RUNNER_ARCH:-}" in
  Linux/X64) target=x86_64-unknown-linux-musl ;;
  Linux/ARM64) target=aarch64-unknown-linux-musl ;;
  macOS/X64) target=x86_64-apple-darwin ;;
  macOS/ARM64) target=aarch64-apple-darwin ;;
  Windows/X64) target=x86_64-pc-windows-msvc ;;
  *) fail "ghg has no build for ${RUNNER_OS:-this system} on ${RUNNER_ARCH:-this processor}." ;;
esac
if [ "$RUNNER_OS" = Windows ]; then
  binary=ghg.exe
  extension=zip
else
  binary=ghg
  extension=tar.gz
fi

fetch() {
  local headers=(-H "Accept: application/vnd.github+json")
  # The token is github.com's only on github.com, not on GitHub Enterprise Server.
  if [ -n "${GHG_TOKEN:-}" ] && [ "${GITHUB_SERVER_URL:-https://github.com}" = https://github.com ]; then
    headers+=(-H "Authorization: Bearer $GHG_TOKEN")
  fi
  curl --fail --silent --show-error --location --retry 3 "${headers[@]}" "$@"
}

# The newest release compatible with the action's own version: the same
# major version, or while that is 0, the same minor version, as Cargo counts
# compatibility. Only a tag that is a plain version, vX.Y.Z, counts. Sets
# version, empty when there is none.
newest_release() {
  local own major minor page=1 listing tags all=""
  own=$(sed -n 's/^version = "\(.*\)"$/\1/p' "$GHG_ACTION_PATH/Cargo.toml" | head -n 1)
  major=${own%%.*}
  minor=${own#*.}
  minor=${minor%%.*}
  while :; do
    listing=$(fetch "$api_url/repos/$repository/releases?per_page=100&page=$page") ||
      fail "The releases of ghg cannot be listed."
    # A release's own keys come before its assets, and a key in its notes
    # has its quotes escaped, so each tag_name is followed by its draft.
    tags=$(printf '%s\n' "$listing" |
      grep -Eo '"(tag_name|draft)": *("[^"]*"|true|false)' |
      awk -F '"' '$2 == "tag_name" { tag = $4 } $2 == "draft" && $0 ~ /false$/ && tag != "" { print tag } $2 == "draft" { tag = "" }' || true)
    [ -n "$tags" ] || break
    all+="$tags"$'\n'
    page=$((page + 1))
  done
  version=$(printf '%s' "$all" |
    grep -E '^v[0-9]+\.[0-9]+\.[0-9]+$' |
    sed 's/^v//' |
    awk -F . -v major="$major" -v minor="$minor" '$1 == major && (major != 0 || $2 == minor)' |
    sort -t . -k 1,1n -k 2,2n -k 3,3n |
    tail -n 1 || true)
}

version=${GHG_VERSION:-}
version=${version#v}
if [ -z "$version" ]; then
  newest_release
  [ -n "$version" ] || fail "ghg has no release compatible with this action's version."
elif ! printf '%s' "$version" | grep -Eq '^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)(-[0-9A-Za-z]+(\.[0-9A-Za-z]+)*)?$'; then
  fail "version must be a version of ghg, such as 1.2.0."
fi

cache=${RUNNER_TOOL_CACHE:-$RUNNER_TEMP}/ghg/$version
folder=$cache/$target
if [ ! -f "$cache/$target.complete" ]; then
  archive=ghg-v$version-$target.$extension
  release=$download_url/$repository/releases/download/v$version
  work=$(mktemp -d "${RUNNER_TEMP:-/tmp}/ghg.XXXXXX")
  trap 'rm -rf "$work"' EXIT
  fetch --output "$work/$archive" "$release/$archive" ||
    fail "$archive cannot be downloaded from the release v$version of ghg."
  fetch --output "$work/SHA256SUMS" "$release/SHA256SUMS" ||
    fail "SHA256SUMS cannot be downloaded from the release v$version of ghg."
  expected=$(awk -v name="$archive" '{ sub(/\r$/, "") } $2 == name || $2 == "*" name { print $1; exit }' "$work/SHA256SUMS")
  if command -v sha256sum > /dev/null; then
    actual=$(sha256sum "$work/$archive" | awk '{ print $1 }')
  else
    actual=$(shasum -a 256 "$work/$archive" | awk '{ print $1 }')
  fi
  [ -n "$expected" ] || fail "$archive is not in the release's SHA256SUMS."
  [ "$expected" = "$actual" ] || fail "$archive does not match its checksum in SHA256SUMS."
  rm -rf "$folder"
  mkdir -p "$folder"
  if [ "$extension" = zip ]; then
    if command -v unzip > /dev/null; then
      unzip -q "$work/$archive" -d "$folder"
    else
      powershell -NoProfile -Command "Expand-Archive -LiteralPath '$(cygpath -w "$work/$archive")' -DestinationPath '$(cygpath -w "$folder")'"
    fi
  else
    tar -xzf "$work/$archive" -C "$folder"
  fi
  [ -f "$folder/$binary" ] || fail "$archive has no $binary."
  touch "$cache/$target.complete"
fi

path_entry=$folder
bin=$folder/$binary
# On Windows, GITHUB_PATH takes a Windows path, and bash a Unix one.
if command -v cygpath > /dev/null; then
  path_entry=$(cygpath -w "$folder")
  bin=$(cygpath -u "$bin")
fi
echo "$path_entry" >> "$GITHUB_PATH"
echo "bin=$bin" >> "$GITHUB_OUTPUT"
echo "ghg $version is installed."
