#!/usr/bin/env bash
# Checks action-tag.sh against docs/BEHAVIOUR.md, "The GitHub Action",
# rule 177, with the lists of tags below. Run from the repository's root.
set -euo pipefail

failures=0
# expect <tag> <expected output> <the repository's tags>...
expect() {
    local tag=$1 expected=$2 actual
    shift 2
    actual=$(printf '%s\n' "$@" | .github/release/action-tag.sh "$tag")
    if [ "$actual" != "$expected" ]; then
        echo "$tag among $*: printed \"$actual\", expected \"$expected\"" >&2
        failures=$((failures + 1))
    fi
}

expect v1.0.0 v1 v1.0.0
expect v1.2.0 v1 v1.0.0 v1.1.3 v1.2.0 v1 v0.9.0
expect v1.10.0 v1 v1.9.0 v1.10.0 v1
expect v2.0.0 v2 v1.4.0 v1 v2.0.0 v10.0.0
# A newer pre-release, or a newer version of another major, does not count.
expect v1.2.0 v1 v1.2.0 v1.3.0-rc.1
expect v1.3.0 v1 v1.3.0 v11.0.0 v10.1.0
# A version with leading zeros is no version.
expect v1.2.0 v1 v1.2.0 v1.02.5
expect v1.02.0 "" v1.02.0
# A release older than another of its major version moves nothing.
expect v1.2.1 "" v1.2.1 v1.3.0 v1
# Nor does a pre-release, a version before 1.0, or a tag that is no version.
expect v2.0.0-rc.1 "" v1.0.0 v2.0.0-rc.1
expect v0.3.0 "" v0.3.0
expect v1 "" v1.0.0 v1
expect 1.0.0 "" 1.0.0
# The tag being released is counted even if the list does not have it yet.
expect v1.5.0 v1 v1.4.0
# Tags listed with CRLF line endings.
expect v1.5.0 v1 $'v1.4.0\r' $'v1.5.0\r' $'v1.6.0-rc.1\r'
expect v1.5.0 "" $'v1.4.0\r' $'v1.6.0\r'

if [ "$failures" -gt 0 ]; then
    echo "$failures cases of action-tag.sh failed." >&2
    exit 1
fi
echo "action-tag.sh moves the right tag in every case."
