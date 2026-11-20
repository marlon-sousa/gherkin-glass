#!/usr/bin/env bash
# Adapter: runs ghg build, for the GitHub Action in action.yml.
#
# Environment: GHG_BIN, the ghg that install.sh installed; GHG_OPTIONS, the
# names of the inputs that are options of ghg build; GHG_FLAGS, those of them
# that take no value; GHG_LISTS, those that take one value on each line, each
# given as its own option; and for each name, its input in GHG_OPTION_<NAME>,
# the name in capitals with _ for -. An empty input is left out, and so are a
# flag that is false and a line of white space only. Exits with ghg's status.

set -euo pipefail

arguments=()
for name in $GHG_OPTIONS; do
  variable=GHG_OPTION_$(printf '%s' "$name" | tr 'a-z-' 'A-Z_')
  value=${!variable:-}
  [ -n "$value" ] || continue
  if [[ " ${GHG_FLAGS:-} " == *" $name "* ]]; then
    case $value in
      true) arguments+=("--$name") ;;
      false) ;;
      *)
        printf '%s must be true or false.\n' "$name" >&2
        exit 1
        ;;
    esac
  elif [[ " ${GHG_LISTS:-} " == *" $name "* ]]; then
    while IFS= read -r line || [ -n "$line" ]; do
      line=${line%$'\r'}
      [[ $line =~ [^[:space:]] ]] || continue
      arguments+=("--$name=$line")
    done <<<"$value"
  else
    arguments+=("--$name=$value")
  fi
done

exec "$GHG_BIN" build ${arguments[@]+"${arguments[@]}"}
