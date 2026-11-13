#!/usr/bin/env bash
# Writes the project's front page from the README's marked parts:
# docs/BEHAVIOUR.md, "The project's own site", rule 193.
#
# Usage:
#   project-site/front-page.sh <readme> <front page>
#     Writes <front page>, or nothing when a mark is wrong, which it reports
#     as <readme>:<line>: <message> and fails.

set -euo pipefail

if [ "$#" -ne 2 ]; then
  echo "Usage: project-site/front-page.sh <readme> <front page>" >&2
  exit 2
fi
readme=$1
front_page=$2

written=$(awk -v readme="$readme" '
  function wrong(message) {
    printf "%s:%d: %s\n", readme, NR, message > "/dev/stderr"
    failed = 1
    exit 1
  }
  $0 == "<!-- site-only" {
    if (part != "") wrong("a site-only part opens inside another part.")
    part = "site"; opened = NR; next
  }
  $0 == "<!-- github-only -->" {
    if (part != "") wrong("a GitHub-only part opens inside another part.")
    part = "github"; opened = NR; next
  }
  $0 == "-->" {
    if (part != "site") wrong("--> closes no site-only part.")
    part = ""; next
  }
  $0 == "<!-- end github-only -->" {
    if (part != "github") wrong("<!-- end github-only --> closes no GitHub-only part.")
    part = ""; next
  }
  part != "github" { print }
  END {
    if (failed) exit 1
    if (part != "") {
      printf "%s:%d: this part is never closed.\n", readme, opened > "/dev/stderr"
      exit 1
    }
  }
' "$readme")
printf '%s\n' "$written" >"$front_page"
