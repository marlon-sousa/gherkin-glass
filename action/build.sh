#!/usr/bin/env bash
# Adapter: runs ghg build, for the GitHub Action in action.yml.
#
# Environment: GHG_BIN, the ghg that install.sh installed, and one
# GHG_OPTION_<NAME> for each input that is an option of ghg build, its name in
# capitals with _ for -. An empty one is left out. Exits with ghg's status.

set -euo pipefail

arguments=()
for variable in $(compgen -e | grep '^GHG_OPTION_' | sort); do
  value=${!variable}
  [ -n "$value" ] || continue
  name=${variable#GHG_OPTION_}
  name=$(printf '%s' "$name" | tr 'A-Z_' 'a-z-')
  arguments+=("--$name=$value")
done

exec "$GHG_BIN" build ${arguments[@]+"${arguments[@]}"}
