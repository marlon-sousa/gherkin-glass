#!/usr/bin/env bash
# Adds one version of the project's site to the gh-pages branch, leaving every
# other version as it was published. docs/BEHAVIOUR.md, "The project's own
# site". Run from inside the repository's checkout, whose origin is GitHub.
#
# Usage:
#   project-site/publish.sh open <pages>
#     Checks out the gh-pages branch into <pages>, a new git worktree,
#     creating the branch, with only a .nojekyll file, when origin has none.
#     ghg then builds the version straight into <pages>, keeping .nojekyll and
#     .git, the worktree's own file, with --keep-in-output.
#   project-site/publish.sh publish <pages> <version>
#     After ghg has built <version> into <pages>, commits <version>/, the
#     versions page, its assets/, the site's data/ and the marker, and pushes
#     gh-pages to origin. It fails, committing nothing, when anything else has changed.

set -euo pipefail

# What a versioned build writes at the root of its output, besides the
# version's own folder: the versions page, the assets it links, the site's
# data, and the marker that lists what ghg wrote.
versions_page=index.html
versions_assets=assets
store=data
marker=ghg-versions.json

usage() {
  echo "Usage: project-site/publish.sh open <pages> | project-site/publish.sh publish <pages> <version>" >&2
  exit 2
}

open() {
  local folder=$1
  local found=0
  git ls-remote --exit-code origin refs/heads/gh-pages >/dev/null || found=$?
  if [ "$found" -eq 0 ]; then
    git fetch --depth=1 origin refs/heads/gh-pages
    git worktree add -B gh-pages "$folder" FETCH_HEAD
  elif [ "$found" -ne 2 ]; then
    echo "origin cannot be asked whether it has a gh-pages branch." >&2
    exit 1
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
    "$versions_page" | "$versions_assets" | "$store" | "$marker")
      echo "$version: a version may not be named as the versions page, its assets, the site's data or the marker." >&2
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

  # The push is made from the checkout, whose credentials a worktree may not see.
  local checkout=$PWD
  cd "$folder"
  git add --all -- "$version" "$versions_page"
  local path
  for path in "$versions_assets" "$store" "$marker"; do
    if [ -e "$path" ] || git ls-files --error-unmatch -- "$path" >/dev/null 2>&1; then
      git add --all -- "$path"
    fi
  done
  if [ -e .nojekyll ]; then
    git add -- .nojekyll
  fi
  local others
  others=$(git status --porcelain --untracked-files=all)
  others=$(grep -v '^[MADR] ' <<<"$others" || true)
  if [ -n "$others" ]; then
    echo "The build changed more than $version/, $versions_page, $versions_assets/, $store/ and $marker, so nothing is published:" >&2
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
  git -C "$checkout" push origin gh-pages
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
