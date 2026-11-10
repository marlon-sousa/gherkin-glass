#!/usr/bin/env bash
# Adds one version of the project's site to the gh-pages branch, leaving every
# other version as it was published. docs/BEHAVIOUR.md, "The project's own
# site". Run from inside the repository's checkout, whose origin is GitHub.
#
# Usage:
#   site/publish.sh open <folder>
#     Checks out the gh-pages branch into <folder>, a new git worktree,
#     creating the branch, with only a .nojekyll file, when origin has none.
#   site/publish.sh publish <folder> <version>
#     After ghg has built <version> into <folder>, commits <version>/ and the
#     versions page, and pushes gh-pages to origin. It fails, committing
#     nothing, when anything else in <folder> has changed.

set -euo pipefail

# What a versioned build writes at the root of its output, besides the
# version's own folder: the versions page of #12.
versions_page=index.html

usage() {
  echo "Usage: site/publish.sh open <folder> | site/publish.sh publish <folder> <version>" >&2
  exit 2
}

open() {
  local folder=$1
  if git ls-remote --exit-code --heads origin gh-pages >/dev/null; then
    git fetch --depth=1 origin gh-pages
    git worktree add -B gh-pages "$folder" FETCH_HEAD
  else
    git worktree add --orphan -b gh-pages "$folder"
    # GitHub Pages runs Jekyll on a branch unless this file is there, and
    # Jekyll leaves out every file and folder whose name starts with _.
    touch "$folder/.nojekyll"
  fi
}

publish() {
  local folder=$1 version=$2
  case "$version" in
    "" | . | .. | .* | */* | *\\*)
      echo "$version: a version is one folder name, not empty and not starting with a dot." >&2
      exit 1
      ;;
  esac
  if [ ! -d "$folder/$version" ]; then
    echo "$folder/$version: ghg wrote no folder for this version." >&2
    exit 1
  fi
  if [ ! -f "$folder/$versions_page" ]; then
    echo "$folder/$versions_page: ghg wrote no versions page." >&2
    exit 1
  fi

  cd "$folder"
  git add --all -- "$version" "$versions_page"
  if [ -e .nojekyll ]; then
    git add -- .nojekyll
  fi
  local others
  others=$(git status --porcelain --untracked-files=all)
  others=$(grep -v '^[MADR] ' <<<"$others" || true)
  if [ -n "$others" ]; then
    echo "The build changed more than $version/ and $versions_page, so nothing is published:" >&2
    echo "$others" >&2
    exit 1
  fi
  if git diff --cached --quiet; then
    echo "$version is already published as it was built."
    return
  fi
  git -c user.name="github-actions[bot]" \
    -c user.email="41898282+github-actions[bot]@users.noreply.github.com" \
    commit --quiet -m "site: $version"
  git push origin gh-pages
}

case "${1-}" in
  open)
    [ "$#" -eq 2 ] || usage
    open "$2"
    ;;
  publish)
    [ "$#" -eq 3 ] || usage
    publish "$2" "$3"
    ;;
  *) usage ;;
esac
