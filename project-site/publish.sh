#!/usr/bin/env bash
# Adds one version of the project's site to the gh-pages branch, leaving every
# other version as it was published. docs/BEHAVIOUR.md, "The project's own
# site". Run from inside the repository's checkout, whose origin is GitHub,
# with every tag fetched and ghg on the path.
#
# Usage:
#   project-site/publish.sh open <pages>
#     Checks out the gh-pages branch into <pages>, a new git worktree,
#     creating the branch, with only a .nojekyll file, when origin has none.
#     When it holds a site of gherkin-glass 1, which ghg 2 refuses to add to,
#     it empties it but for a new .nojekyll, and prints on standard output the
#     versions that site held, one name a line, for rebuild to add again.
#   project-site/publish.sh build <pages> <version> <input> <front page>
#     Builds <version> from <input> with ghg version add straight into
#     <pages>, keeping .nojekyll and .git, the worktree's own file.
#   project-site/publish.sh rebuild <pages> <front page>
#     For each version named on standard input, builds it into <pages> from
#     the example suite and fixtures of its own tag, gathered by that tag's
#     assemble-input.sh, and commits it as commit does. It fails, committing
#     nothing more, at the first version that has no tag.
#   project-site/publish.sh commit <pages> <version>
#     After ghg has built <version> into <pages>, commits <version>/, the
#     versions page, its ghg's folder of assets/ when it is new, the site's
#     data/ and the marker. It fails, committing nothing, when anything else
#     has changed.
#   project-site/publish.sh publish <pages> <version>
#     Commits <version> as commit does, then pushes gh-pages to origin, with
#     every version committed before it.

set -euo pipefail

# What a versioned build writes at the root of its output, besides the
# version's own folder: the versions page, the assets it links, the site's
# data, and the marker that lists what ghg wrote.
versions_page=index.html
versions_assets=assets
store=data
marker=ghg-site.json
title="gherkin-glass examples"

usage() {
  echo "Usage: project-site/publish.sh open <pages> | build <pages> <version> <input> <front page> | rebuild <pages> <front page> | commit <pages> <version> | publish <pages> <version>" >&2
  exit 2
}

# Whether the output folder <folder> was marked by gherkin-glass 1: with the
# marker of its versions, or with a site's marker that names no format of the
# site's data, as docs/BEHAVIOUR.md, rule 256, tells them.
holds_gherkin_glass_1() {
  local folder=$1
  [ -e "$folder/ghg-versions.json" ] && return 0
  [ -f "$folder/$marker" ] && ! grep -q '"format"' "$folder/$marker"
}

open() {
  local folder=$1
  local found=0
  git ls-remote --exit-code origin refs/heads/gh-pages >/dev/null || found=$?
  if [ "$found" -eq 0 ]; then
    git fetch --quiet --depth=1 origin refs/heads/gh-pages
    git worktree add --quiet -B gh-pages "$folder" FETCH_HEAD
    if holds_gherkin_glass_1 "$folder"; then
      echo "gh-pages holds a site of gherkin-glass 1, which ghg 2 cannot add to, so its versions are built again." >&2
      if [ -f "$folder/ghg-versions.json" ]; then
        # A version of gherkin-glass 1 is a folder at the root that its marker
        # names, as "<version>": [ followed by the files it wrote there.
        local name
        while IFS= read -r name; do
          if grep -qF "\"$name\":" "$folder/ghg-versions.json"; then
            echo "$name"
          fi
        done < <(git -C "$folder" ls-tree -d --name-only HEAD)
      fi
      git -C "$folder" rm -r --quiet -- .
      touch "$folder/.nojekyll"
    fi
  elif [ "$found" -ne 2 ]; then
    echo "origin cannot be asked whether it has a gh-pages branch." >&2
    exit 1
  else
    git worktree add --quiet --orphan -b gh-pages "$folder"
    # GitHub Pages runs Jekyll on a branch unless this file is there, and
    # Jekyll leaves out every file and folder whose name starts with _.
    touch "$folder/.nojekyll"
  fi
}

build() {
  local folder=$1 version=$2 input=$3 front_page=$4
  ghg version add "$version" \
    --title "$title" \
    --input "$input" \
    --output "$folder" \
    --front-page "$front_page" \
    --keep-in-output .nojekyll \
    --keep-in-output .git
}

rebuild() {
  local folder=$1 front_page=$2
  local tags
  tags=$(mktemp -d)
  local versions version
  versions=$(cat)
  # The names come through file descriptor 3, so that nothing run for each
  # version reads them from standard input.
  while IFS= read -r version <&3; do
    [ -n "$version" ] || continue
    if ! git rev-parse --quiet --verify "refs/tags/$version^{commit}" >/dev/null; then
      echo "$version: the site held this version, but the repository has no such tag to build it again from." >&2
      exit 1
    fi
    git worktree add --quiet --detach "$tags/$version" "refs/tags/$version"
    "$tags/$version/project-site/assemble-input.sh" "$tags/input-$version"
    build "$folder" "$version" "$tags/input-$version" "$front_page"
    (commit "$folder" "$version")
  done 3<<<"$versions"
}

commit() {
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
  # A folder of assets/ is its ghg's, written once: added when it is new, or
  # removed when nothing links it any more, never changed.
  local status path folder_of
  while IFS=$'\t' read -r status path; do
    folder_of=${path#"$versions_assets"/}
    folder_of=${folder_of%%/*}
    if [ "$status" != D ] && git cat-file -e "HEAD:$versions_assets/$folder_of" 2>/dev/null; then
      echo "The build changed $path, in a folder of $versions_assets/ that was already published, so nothing is published." >&2
      exit 1
    fi
  done < <(git diff --cached --name-status --no-renames -- "$versions_assets")
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
}

publish() {
  # The push is made from the checkout, whose credentials a worktree may not see.
  local checkout=$PWD
  commit "$1" "$2"
  git -C "$checkout" push origin gh-pages
}

case "${1-}" in
  open)
    [ "$#" -eq 2 ] || usage
    open "$2"
    ;;
  commit)
    [ "$#" -eq 3 ] || usage
    commit "$2" "$3"
    ;;
  publish)
    [ "$#" -eq 3 ] || usage
    publish "$2" "$3"
    ;;
  build)
    [ "$#" -eq 5 ] || usage
    build "$2" "$3" "$4" "$5"
    ;;
  rebuild)
    [ "$#" -eq 3 ] || usage
    rebuild "$2" "$3"
    ;;
  *) usage ;;
esac
