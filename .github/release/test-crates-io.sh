#!/usr/bin/env bash
# Checks crates-io.sh against docs/BEHAVIOUR.md, "Releases", rule 167, with
# the tags below. Run from the repository's root.
set -euo pipefail

failures=0
# expect <tag> <expected output>
expect() {
    local tag=$1 expected=$2 actual
    actual=$(.github/release/crates-io.sh "$tag")
    if [ "$actual" != "$expected" ]; then
        echo "$tag: printed \"$actual\", expected \"$expected\"" >&2
        failures=$((failures + 1))
    fi
}

# A version without a pre-release part is published.
expect v0.1.0 true
expect v1.2.0 true
expect v10.20.30 true
# So is a release candidate.
expect v0.2.0-rc.1 true
expect v1.0.0-rc.12 true
expect v1.0.0-rc true
# An alpha and a beta are not.
expect v0.1.0-alpha.1 false
expect v0.1.0-alpha false
expect v0.1.0-beta.1 false
expect v1.0.0-beta.2 false
# Nor is any other pre-release, even one whose name starts with rc.
expect v1.0.0-dev.1 false
expect v1.0.0-rc1 false
expect v1.0.0-rcx.1 false
expect v1.0.0-alpha.rc.1 false
# A pre-release part may follow rc only after a dot.
expect v1.0.0-rc. false

if [ "$failures" -gt 0 ]; then
    echo "$failures cases of crates-io.sh failed." >&2
    exit 1
fi
echo "crates-io.sh tells every release whether it goes to crates.io."
