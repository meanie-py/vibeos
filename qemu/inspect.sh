#!/bin/bash
# Single-shot batch GDB invocation against a running QEMU.
#
# Usage:
#   qemu/inspect.sh <run-id> --command <name> [args...] [--json]
#
# Built-in commands:
#   regs              -- info registers
#   bt [N]            -- backtrace with [Rust]/[C]/[ASM]/[???] tags
#   bt-all-cpus       -- bt for every vCPU (currently same as bt; we run 1 vCPU)
#   symbolicate-pc <addr>
#                     -- print symbol+offset+source for a single address
#   dump-thread [addr]
#                     -- pretty-print _KTHREAD (current thread by default)
#   decode-bugcheck   -- read KeBugCheckEx args at the bugcheck frame
#   find-kbase        -- scan for runtime kernel base, store in $kbase
#   raw <gdb-cmd>     -- run a raw GDB command (escape hatch)
#
# Always issues `delete breakpoints` on attach (defense-in-depth against
# stale breakpoints leaking between batch invocations).

set -u

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
. "$SCRIPT_DIR/lib/common.sh"
. "$SCRIPT_DIR/lib/manifest.sh"
. "$SCRIPT_DIR/lib/runid.sh"

GDB_HELPERS_DIR="$SCRIPT_DIR/gdb-helpers"

RUN_ID="${1:-}"
COMMAND=""
ARGS=()
JSON=0
shift 2>/dev/null || true
while [ $# -gt 0 ]; do
    case "$1" in
        --command) shift; COMMAND="$1" ;;
        --json)    JSON=1 ;;
        --)        shift; ARGS+=("$@"); break ;;
        *)         ARGS+=("$1") ;;
    esac
    shift
done

[ -n "$RUN_ID" ]   || die "usage: inspect.sh <run-id> --command <name> [args]"
[ -n "$COMMAND" ]  || die "--command required"
manifest_exists "$RUN_ID" || die "no such run-id: $RUN_ID"

ARCH="$(manifest_get "$RUN_ID" arch)"
GDB_PORT="$(manifest_get "$RUN_ID" gdb_port)"
KERNEL="$(manifest_get "$RUN_ID" kernel_image)"

[ -n "$GDB_PORT" ] && [ "$GDB_PORT" != "null" ] || die \
    "run-id $RUN_ID has no gdb_port — restart with --gdb"
[ -f "$KERNEL" ] || die "kernel image missing: $KERNEL"

GDB_LOG="$(runid_gdb_log "$RUN_ID")"

# --- assemble GDB command list ---------------------------------------------
GDB_ARCH="$(arch_gdb_name "$ARCH")"

EX_ARGS=(
    -ex "set architecture $GDB_ARCH"
    -ex "file $KERNEL"
    -ex "target remote 127.0.0.1:$GDB_PORT"
    -ex "source $GDB_HELPERS_DIR/preamble.gdb"
    -ex "source $GDB_HELPERS_DIR/find-kbase.py"
    -ex "source $GDB_HELPERS_DIR/bt-tagged.py"
    -ex "source $GDB_HELPERS_DIR/decode-bugcheck.py"
    -ex "source $GDB_HELPERS_DIR/dump-thread.py"
)

# Auto-resolve kbase + relocate symbols. If we already cached it in the
# manifest, pass it to find-kbase to skip the scan; otherwise let the helper
# scan memory and write back to the manifest after.
CACHED_KBASE="$(manifest_get "$RUN_ID" kernel_runtime_base 2>/dev/null || true)"
if [ -n "$CACHED_KBASE" ] && [ "$CACHED_KBASE" != "null" ]; then
    EX_ARGS+=(-ex "find-kbase $CACHED_KBASE")
else
    EX_ARGS+=(-ex "find-kbase")
fi

case "$COMMAND" in
    regs)
        EX_ARGS+=(-ex "info registers")
        ;;
    bt)
        N="${ARGS[0]:-30}"
        EX_ARGS+=(-ex "bt-tagged $N")
        ;;
    bt-all-cpus)
        # Single vCPU today. When we run multi-vCPU we'll iterate "thread N"
        # gdbstub-style. For now this is a synonym for bt.
        N="${ARGS[0]:-30}"
        EX_ARGS+=(-ex "bt-tagged $N")
        ;;
    symbolicate-pc)
        ADDR="${ARGS[0]:-}"
        [ -n "$ADDR" ] || die "symbolicate-pc requires an address"
        EX_ARGS+=(-ex "info symbol $ADDR")
        ;;
    dump-thread)
        EX_ARGS+=(-ex "dump-thread ${ARGS[*]:-}")
        ;;
    decode-bugcheck)
        EX_ARGS+=(-ex "decode-bugcheck")
        ;;
    find-kbase)
        EX_ARGS+=(-ex "find-kbase")
        ;;
    raw)
        [ "${#ARGS[@]}" -ge 1 ] || die "raw requires a command"
        EX_ARGS+=(-ex "${ARGS[*]}")
        ;;
    *)
        die "unknown --command: $COMMAND"
        ;;
esac

EX_ARGS+=(-ex "detach" -ex "quit")

# --- invoke gdb-multiarch as one batch -------------------------------------
# Pass arch + kernel image via env so the Python helpers don't need to
# re-derive them from argv.
{
    echo "=== inspect.sh $RUN_ID --command $COMMAND $(date -u +%FT%TZ) ==="
    VIBEOS_ARCH="$ARCH" VIBEOS_KERNEL="$KERNEL" \
        gdb-multiarch -batch "${EX_ARGS[@]}" 2>&1
} | tee -a "$GDB_LOG"

# Persist the runtime kernel base to the manifest if we just discovered it.
if [ -z "$CACHED_KBASE" ] || [ "$CACHED_KBASE" = "null" ]; then
    discovered="$(grep -oE 'kbase=0x[0-9a-fA-F]+' "$GDB_LOG" | tail -1 | cut -d= -f2)"
    if [ -n "$discovered" ]; then
        manifest_set "$RUN_ID" kernel_runtime_base "$discovered"
    fi
fi
