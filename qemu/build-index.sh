#!/bin/bash
# Build a sorted symbol index for a kernel build, for fast offline sym/addr
# lookup via sym.sh and addr.sh.
#
# Usage:
#   qemu/build-index.sh <arch>
#
# Writes:
#   qemu/symbols-<arch>.tsv     # tab-separated: <hex_addr>\t<symbol>\t<source>
#
# Source column comes from `nm --line-numbers` when DWARF has it (most C and
# Rust functions), else "-".

set -u

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
. "$SCRIPT_DIR/lib/common.sh"

ARCH="${1:-}"
[ -n "$ARCH" ] || die "usage: build-index.sh <arch>"
arch_validate "$ARCH"

NM="$(arch_nm "$ARCH")"
[ -x "$NM" ] || die "missing $NM"

index_one() {
    local img="$1" out="$2"
    log_info "indexing $img -> $out"
    # nm output:
    #   <addr> <type> <name>
    # Some nm variants accept --line-numbers to add <source>:<line>; llvm-nm
    # silently emits the same 3 columns either way. We tolerate either width.
    "$NM" --line-numbers "$img" 2>/dev/null \
        | awk '
            # field 1 = addr (hex); field 2 = type; field 3 = name; field 4 (if present) = source:line
            NF >= 3 && $1 ~ /^[0-9a-fA-F]+$/ {
                addr = $1
                name = $3
                src  = (NF >= 4) ? $4 : "-"
                print addr "\t" name "\t" src
            }
        ' \
        | sort -k1,1 \
        | uniq > "$out"
    log_info "wrote $(wc -l < "$out") symbols to $out"
}

IMG="$(arch_kernel_image "$ARCH")"
[ -f "$IMG" ] || die "missing $IMG — build first"
index_one "$IMG" "$QEMU_DIR/symbols-$ARCH.tsv"

# The MP kernel is a SEPARATE binary with its own layout, and every `--smp N`
# boot runs it. Without its own index an MP crash gets symbolicated against
# ntoskrnl.exe and silently names the wrong function. Best-effort: ntkrnlmp is an
# optional ninja target, so a build dir that never built it just has no MP index
# (addr2sym --mp then says so rather than falling back to the UP one).
MPIMG="$(arch_kernel_image "$ARCH" mp)"
if [ -f "$MPIMG" ]; then
    index_one "$MPIMG" "$QEMU_DIR/symbols-$ARCH-mp.tsv"
else
    log_info "no $MPIMG — skipping MP symbol index"
fi
