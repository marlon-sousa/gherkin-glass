#!/usr/bin/env bash
# Writes the crate's README, which crates.io and docs.rs show, from the
# repository's README: docs/BEHAVIOUR.md, "The front page", rule 223.
#
# Usage:
#   project-site/crate-readme.sh <readme> <crate readme>
#     Writes <crate readme>, or nothing when a mark of <readme> is wrong,
#     which project-site/front-page.sh reports, and fails.
set -euo pipefail
if [ "$#" -ne 2 ]; then
  echo "Usage: project-site/crate-readme.sh <readme> <crate readme>" >&2
  exit 2
fi
readme=$1
crate_readme=$2
front_page=$(mktemp)
trap 'rm -f "$front_page"' EXIT
"$(dirname "$0")/front-page.sh" "$readme" "$front_page"
{
  echo "<!-- Written from README.md by project-site/crate-readme.sh, which cargo xtask build runs. Edit README.md, then run it again. -->"
  echo
  # The marker outside a fenced code block, where the README may show it as an example.
  awk '
    /^```/ { fenced = !fenced }
    !fenced && $0 == "<!-- ghg:contents -->" {
      print "See them on [the project'"'"'s site](https://marlon-sousa.github.io/gherkin-glass/)."
      next
    }
    { print }
  ' "$front_page"
} >"$crate_readme.new"
mv "$crate_readme.new" "$crate_readme"
