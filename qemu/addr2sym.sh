#!/bin/bash
# Offline address -> symbol lookup against a cached kernel index.
#
# Usage:
#   qemu/addr2sym.sh <arch> [--reloc <hex> | --kernel-base <hex>] [--mp | --index <tsv>] <addr>...
#
# Reads qemu/symbols-<arch>.tsv (built by build-index.sh). Output one line
# per address: <addr>\t<symbol>+<offset>\t<source>

set -u

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
. "$SCRIPT_DIR/lib/common.sh"

ARCH="${1:-}"
shift 2>/dev/null || true
arch_validate "$ARCH"

# Optional --reloc <hex>: subtract this from each input address before lookup.
# Use it when the input is a runtime VA (e.g. PC from a bugcheck Param2)
# and the index holds file VAs. The right value is `kernel_runtime_base
# - 0x400000` for ntoskrnl. inspect.sh's symbolicate-pc command is the
# online equivalent and finds this automatically; this flag is for offline
# triage where you only have a hex address from a serial log.
RELOC=0
FLAVOR=""
INDEX=""
while :; do
    case "${1:-}" in
        --reloc)
            RELOC="${2:-}"
            shift 2
            [ -n "$RELOC" ] || die "--reloc requires an address argument"
            ;;
        # --kernel-base <hex>: the value printed by the kernel's own
        # "[KRNLBASE] ntoskrnl runtime base=<hex>" serial line. Pass it verbatim; we
        # do the `- 0x400000` here so nobody has to remember which direction it goes.
        # Doing that subtraction by hand is exactly the kind of remembered recipe
        # that produces a confidently wrong base, and a wrong base relocates the hunt
        # instead of failing loudly.
        --kernel-base)
            KBASE="${2:-}"
            shift 2
            [ -n "$KBASE" ] || die "--kernel-base requires an address argument"
            [ "$RELOC" = "0" ] || die "pass --reloc OR --kernel-base, not both"
            RELOC=$(python3 -c "import sys; print(format(int(sys.argv[1],16) - 0x400000, 'x'))" "$KBASE")
            ;;
        # --mp: resolve against ntkrnlmp.exe's index. An `--smp N` boot runs a
        # DIFFERENT binary from ntoskrnl.exe, so the UP index names the wrong
        # function for every MP crash — quietly, since some symbol always sorts
        # below the address. Callers that know which kernel booted must say so.
        --mp) FLAVOR="-mp"; shift ;;
        # --index <tsv>: resolve against a specific index instead of this tree's
        # qemu/symbols-<arch>.tsv. A crash from another build (a farm archive under
        # ~/vibeos-flake-farm/kernels/<sha>/, named by the run's manifest.json
        # kernels_dir) must be read against ITS index: the live one names some
        # unrelated function at every address, and reads exactly like a right answer.
        --index)
            INDEX="${2:-}"
            shift 2
            [ -n "$INDEX" ] || die "--index requires a path to a symbols tsv"
            [ -f "$INDEX" ] || die "--index: no such file: $INDEX"
            ;;
        # An unknown flag used to fall through to the address list and be looked up as
        # an address, while the lookup itself ran against the default index. A
        # misspelled index option thereby produced plausible wrong symbols.
        -*) die "unknown option: $1" ;;
        *) break ;;
    esac
done

[ "$#" -ge 1 ] || die "usage: addr2sym.sh <arch> [--reloc <hex> | --kernel-base <hex>] [--mp | --index <tsv>] <addr> [<addr>...]"

if [ -n "$INDEX" ]; then
    [ -z "$FLAVOR" ] || die "pass --index OR --mp, not both (--index names the exact tsv)"
else
    INDEX="$QEMU_DIR/symbols-$ARCH$FLAVOR.tsv"
fi
if [ ! -f "$INDEX" ]; then
    if [ -n "$FLAVOR" ]; then
        # Never silently fall back to the UP index: a wrong symbol is worse than
        # no symbol, because it reads exactly like a right one.
        die "missing $INDEX — the MP kernel was not indexed. Build ntkrnlmp (./vibeos-cli build --arch $ARCH) then re-run build-index.sh $ARCH"
    fi
    die "missing $INDEX — run build-index.sh $ARCH first"
fi

# Lookup uses awk: linear scan with early exit when address exceeds target.
# Index is sorted by lowercase fixed-width hex, so string compare == numeric
# IFF both sides are the same width — pad the target accordingly. We sample
# field 1 of the first non-empty line to learn the index width.
INDEX_WIDTH=$(awk -F'\t' '$1 != "" {print length($1); exit}' "$INDEX")

for raw in "$@"; do
    addr="${raw#0x}"; addr="${addr#0X}"
    addr="$(echo -n "$addr" | tr '[:upper:]' '[:lower:]')"
    [[ "$addr" =~ ^[0-9a-f]+$ ]] || die "not a hex address: $raw"
    # Apply --reloc subtraction (in Python because bash arithmetic is 64-bit
    # signed and chokes on values like 0xfffff80000000000).
    if [ "$RELOC" != "0" ]; then
        addr=$(python3 -c "import sys; print(format(int(sys.argv[1],16) - int(sys.argv[2],16), 'x'))" "$addr" "$RELOC")
    fi
    # Zero-pad to index width.
    while [ "${#addr}" -lt "$INDEX_WIDTH" ]; do addr="0$addr"; done

    awk -F'\t' -v target="$addr" '
        function hex2dec(h,    i, c, v) {
            v = 0
            for (i = 1; i <= length(h); i++) {
                c = tolower(substr(h, i, 1))
                v = v * 16 + (index("0123456789abcdef", c) - 1)
            }
            return v
        }
        # awk auto-coerces strings that look numeric (including hex with "e"
        # exponents like 000002e8) to numbers, ruining comparisons. Force
        # string compare by concatenating empty strings on both sides.
        {
            if (($1 "") <= (target "")) { ba = $1; bs = $2; bf = $3 }
            else                        { exit }
        }
        END {
            if (bs == "") { printf "%s\t<no symbol>\n", target; exit }
            off = hex2dec(target) - hex2dec(ba)
            printf "%s\t%s+%d\t%s\n", target, bs, off, bf
        }
    ' "$INDEX"
done
