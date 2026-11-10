#!/usr/bin/env bash
# Checks a built binary against docs/BEHAVIOUR.md, "Releases", rule 40: it
# needs nothing installed beyond what every system of its kind has. When it
# was built for the machine running the check, it is also run.
# Usage: check-binary.sh <binary> <target>
set -euo pipefail

binary=$1 target=$2
name=$(basename "$binary")

case $target in
    *-linux-musl)
        description=$(file -b "$binary")
        case $description in
            *"statically linked"* | *"static-pie linked"*) ;;
            *)
                echo "$name is not statically linked: $description" >&2
                exit 1
                ;;
        esac
        ;;
    *-apple-darwin)
        # otool's first line names the binary; each other line is a library it loads.
        foreign=$(otool -L "$binary" | tail -n +2 | awk '{print $1}' | grep -Ev '^(/usr/lib/|/System/Library/)' || true)
        if [ -n "$foreign" ]; then
            echo "$name loads libraries that are not part of macOS:" >&2
            echo "$foreign" >&2
            exit 1
        fi
        ;;
    *-windows-msvc)
        readobj="$(cygpath -u "$(rustc --print sysroot)")/lib/rustlib/$target/bin/llvm-readobj.exe"
        runtime=$("$readobj" --coff-imports "$binary" | sed -n 's/^ *Name: *//p' \
            | grep -Ei '^(vcruntime|msvcp|ucrtbase|api-ms-win-crt-)' || true)
        if [ -n "$runtime" ]; then
            echo "$name needs the C runtime installed, from:" >&2
            echo "$runtime" >&2
            exit 1
        fi
        ;;
    *)
        echo "$target is not a target gherkin-glass releases for." >&2
        exit 1
        ;;
esac
echo "$name for $target needs nothing installed."

if [ "$(rustc -vV | sed -n 's/^host: //p' | tr -d '\r')" = "$target" ]; then
    "$binary" --version
fi
