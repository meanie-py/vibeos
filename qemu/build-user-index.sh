#!/bin/bash
# Offline symbol index for USER-MODE PEs (ntdll, kernel32, user32, the wow64
# layer, applets...) — the half of the system build-index.sh does not cover.
#
# build-index.sh indexes exactly ntoskrnl.exe and ntkrnlmp.exe, so every
# user-mode address had to be resolved by hand with objdump plus a throwaway
# nearest-symbol script. Fine once, a tax forever: the WoW64 lane, the winetest
# corpus and every stock application live entirely up here.
#
# Usage:
#   qemu/build-user-index.sh <arch>
#
# Writes:
#   qemu/modules-<arch>.tsv       <base>\t<end>\t<module>\t<path>\t<unique|shared>
#   qemu/symbols-<arch>-user.tsv  <module>\t<addr>\t<symbol>\t<source>
#
# Two files, because one flat address table would be WRONG. A build-amd64 tree
# holds ~1450 PEs sharing far fewer distinct image bases: only the modules
# ReactOS assigns a base to get a unique one, while the rest sit on the linker
# default (90 at 0x11000, 42 at 0x140001000). A single addr->symbol table over
# that set would silently name a function in the wrong module, which is exactly
# the failure this tool exists to prevent.
#
# So: resolve the MODULE first, by containing range, and only then the symbol.
# Modules on a SHARED base are recorded but get no symbols — their runtime base
# is assigned at load time, so an address cannot identify them anyway, and
# indexing them would buy nothing but ambiguity. They are marked `shared` so the
# lookup can say WHY it declined instead of shrugging. Skipping them is also
# what makes this fast: ~136 nm calls instead of ~1450.
#
# ASLR is off and our PEs load at their link base, so a file VA is a runtime VA
# and no relocation bookkeeping is needed. That is what makes this cheap at all.

set -u

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
. "$SCRIPT_DIR/lib/common.sh"

ARCH="${1:-}"
arch_validate "$ARCH"

# --one <path>: dump ONE module's symbols to stdout and exit, in the same
# 4-column shape. This is what `sym --module` uses to resolve inside a module
# that shares an image base and therefore has no rows in the main index: the
# caller has supplied the identity the address could not, so the only thing
# missing is the symbols, and nm on a single PE costs ~0.1s. Without this,
# `--module` answered "no symbol table entry" for exactly the modules it exists
# to serve -- which is how it was found.
if [ "${2:-}" = "--one" ]; then
    NM="$(arch_nm "$ARCH")"
    one="${3:-}"
    [ -f "$one" ] || die "--one needs a readable module path"
    "$NM" --defined-only --line-numbers "$one" 2>/dev/null \
        | awk -v mod="$(basename "$one")" '
            $1 ~ /^[0-9a-fA-F]+$/ && NF >= 3 {
                if ($2 == "a" || $2 == "A" || $2 == "U") next
                print mod "\t" $1 "\t" $3 "\t" (NF >= 4 ? $4 : "-")
            }
        '
    exit 0
fi

NM="$(arch_nm "$ARCH")"
OBJDUMP="${NM%nm}objdump"
BUILD="$VIBEOS_ROOT/build-$ARCH"
[ -d "$BUILD" ] || die "missing $BUILD — build first"

MODS="$QEMU_DIR/modules-$ARCH.tsv"
SYMS="$QEMU_DIR/symbols-$ARCH-user.tsv"
HDRS="$(mktemp)"
trap 'rm -f "$HDRS"' EXIT

# Pass 1 — image bases. `boot/stripped/` holds hash-prefixed stripped COPIES of
# real modules; indexing them manufactures a duplicate of every module that has
# one (a `d60cb67db295-ntdll.dll` claiming the same VA range as ntdll.dll).
while IFS= read -r img; do
    hdr="$("$OBJDUMP" -p "$img" 2>/dev/null)" || continue
    base="$(printf '%s\n' "$hdr" | awk '/^ImageBase/{print $2; exit}')"
    size="$(printf '%s\n' "$hdr" | awk '/^SizeOfImage/{print $2; exit}')"
    [ -n "$base" ] && [ -n "$size" ] || continue
    printf '%016x\t%016x\t%s\t%s\n' \
        "$((16#$base))" "$((16#$base + 16#$size))" \
        "$(basename "$img")" "${img#"$VIBEOS_ROOT"/}" >> "$HDRS"
done < <(find "$BUILD" \
    \( -name '*.dll' -o -name '*.exe' -o -name '*.sys' -o -name '*.ocx' \
       -o -name '*.cpl' -o -name '*.drv' \) -type f 2>/dev/null \
    | grep -v '/CMakeFiles/' | grep -v '/stripped/')

# Pass 2 — mark unique vs shared, then index only what an address can identify.
awk -F'\t' 'NR==FNR{n[$1]++; next} {print $0 "\t" (n[$1] == 1 ? "unique" : "shared")}' \
    "$HDRS" "$HDRS" | sort -k1,1 | uniq > "$MODS"

: > "$SYMS.tmp"
count=0
while IFS=$'\t' read -r _base _end name path flag; do
    [ "$flag" = "unique" ] || continue
    "$NM" --defined-only --line-numbers "$VIBEOS_ROOT/$path" 2>/dev/null \
        | awk -v mod="$name" '
            $1 ~ /^[0-9a-fA-F]+$/ && NF >= 3 {
                if ($2 == "a" || $2 == "A" || $2 == "U") next
                print mod "\t" $1 "\t" $3 "\t" (NF >= 4 ? $4 : "-")
            }
        ' >> "$SYMS.tmp"
    count=$((count + 1))
done < "$MODS"

sort -k1,1 -k2,2 "$SYMS.tmp" | uniq > "$SYMS" && rm -f "$SYMS.tmp"
log_info "indexed $count/$(wc -l < "$MODS") user-mode modules (rest share a base): $(wc -l < "$SYMS") symbols"
