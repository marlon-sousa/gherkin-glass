#!/usr/bin/env bash
# Adapter: runs ghg build, for the GitHub Action in action.yml.
#
# Environment: GHG_BIN, the ghg that install.sh installed; GHG_OPTIONS, the
# names of the inputs that are options of ghg build; and for each name, its
# input in GHG_OPTION_<NAME>, the name in capitals with _ for -. An empty
# input is left out. Exits with ghg's status.

set -euo pipefail

arguments=()
for name in $GHG_OPTIONS; do
  variable=GHG_OPTION_$(printf '%s' "$name" | tr 'a-z-' 'A-Z_')
  value=${!variable:-}
  [ -n "$value" ] || continue
  arguments+=("--$name=$value")
done

exec "$GHG_BIN" build ${arguments[@]+"${arguments[@]}"}
