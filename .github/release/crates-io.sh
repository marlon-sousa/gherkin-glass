#!/usr/bin/env bash
# Prints true when the release of <tag> is published to crates.io, and false
# when it is not, as docs/BEHAVIOUR.md, "Releases", rule 167, says: only a
# version without a pre-release part, such as v1.2.0, or a release candidate,
# whose pre-release part starts with the identifier rc, such as v1.2.0-rc.1,
# is published. An alpha, a beta and any other pre-release are not.
# Usage: crates-io.sh <tag>
set -euo pipefail

tag=$1
if [[ $tag =~ ^v[0-9]+\.[0-9]+\.[0-9]+(-rc(\..+)?)?$ ]]; then
    echo true
else
    echo false
fi
