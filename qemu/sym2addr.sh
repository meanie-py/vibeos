#!/bin/bash
# Offline symbol -> address lookup against a cached kernel index.
#
# Usage:
#   qemu/sym2addr.sh <arch> <symbol>           # exact match
#   qemu/sym2addr.sh <arch> <symbol> --regex   # regex match (multi-result)

set -u

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
. "$SCRIPT_DIR/lib/common.sh"

ARCH="${1:-}"
SYM="${2:-}"
MODE="${3:-exact}"
arch_validate "$ARCH"
[ -n "$SYM" ] || die "usage: sym2addr.sh <arch> <symbol> [--regex]"

INDEX="$QEMU_DIR/symbols-$ARCH.tsv"
[ -f "$INDEX" ] || die "missing $INDEX — run build-index.sh $ARCH first"

if [ "$MODE" = "--regex" ]; then
    awk -F'\t' -v re="$SYM" '$2 ~ re { print $1 "\t" $2 "\t" $3 }' "$INDEX"
else
    # i386 PE decoration zoo:
    #   cdecl:     _<name>
    #   stdcall:   _<name>@<bytes>
    #   fastcall:  @<name>@<bytes>
    # Match the bare symbol or any of these decorated forms; amd64/arm64 just
    # match bare. The regex anchors prevent accidental substring hits.
    awk -F'\t' -v s="$SYM" '
        $2 == s ||
        $2 == "_" s ||
        $2 ~ ("^_" s "@[0-9]+$") ||
        $2 ~ ("^@" s "@[0-9]+$") {
            print $1 "\t" $2 "\t" $3
        }
    ' "$INDEX"
fi
