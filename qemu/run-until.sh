#!/bin/bash
# Run a kernel until a named symbol is hit, then run inspection commands at
# the breakpoint, all in one batch GDB invocation.
#
# Usage:
#   qemu/run-until.sh <run-id> --break <symbol> --timeout <secs>
#                              [--then <inspect-command>]...
#                              [--json]
#
# This is the bounded primitive that replaces the fragile "set breakpoint in
# one GDB session, hope it persists, attach later" pattern. It always issues
# `delete breakpoints` before exiting — on hit AND on timeout — so stale
# breakpoint state can't leak between batch invocations.
#
# --then can be specified multiple times. Each value is one of the same
# command names accepted by inspect.sh (bt, regs, decode-bugcheck, etc.).
# Args to chained commands go after the command name in the same argument:
#   --then "bt 50"
#   --then "dump-thread 0xfffffa80015c92f0"

set -u

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
. "$SCRIPT_DIR/lib/common.sh"
. "$SCRIPT_DIR/lib/manifest.sh"
. "$SCRIPT_DIR/lib/runid.sh"

GDB_HELPERS_DIR="$SCRIPT_DIR/gdb-helpers"

RUN_ID="${1:-}"
BRK=""
TIMEOUT=120
THENS=()
JSON=0
shift 2>/dev/null || true
while [ $# -gt 0 ]; do
    case "$1" in
        --break)   shift; BRK="$1" ;;
        --timeout) shift; TIMEOUT="$1" ;;
        --then)    shift; THENS+=("$1") ;;
        --json)    JSON=1 ;;
        *) die "unknown argument: $1" ;;
    esac
    shift
done

[ -n "$RUN_ID" ] || die "usage: run-until.sh <run-id> --break <symbol> --timeout <secs> [--then <cmd>]"
[ -n "$BRK" ]    || die "--break required"
manifest_exists "$RUN_ID" || die "no such run-id: $RUN_ID"

ARCH="$(manifest_get "$RUN_ID" arch)"
GDB_PORT="$(manifest_get "$RUN_ID" gdb_port)"
KERNEL="$(manifest_get "$RUN_ID" kernel_image)"
GDB_LOG="$(runid_gdb_log "$RUN_ID")"

[ -n "$GDB_PORT" ] && [ "$GDB_PORT" != "null" ] || die \
    "run-id $RUN_ID has no gdb_port — restart with --gdb"

GDB_ARCH="$(arch_gdb_name "$ARCH")"

# Map a --then token to the gdb command. Names match inspect.sh's --command
# vocabulary; argument tail is forwarded verbatim.
expand_then() {
    local raw="$1" cmd="${1%% *}" rest=""
    [ "$cmd" != "$raw" ] && rest=" ${raw#* }"
    case "$cmd" in
        bt|bt-all-cpus)   echo "bt-tagged${rest:- 30}" ;;
        regs)             echo "info registers" ;;
        decode-bugcheck)  echo "decode-bugcheck" ;;
        triage-bugcheck)  echo "triage-bugcheck" ;;
        walk-waitlist)    echo "walk-waitlist${rest}" ;;
        dump-thread)      echo "dump-thread${rest}" ;;
        dump-trapframe)   echo "dump-trapframe${rest}" ;;
        pool-tag)         echo "pool-tag${rest}" ;;
        identify-object)  echo "identify-object${rest}" ;;
        symbolicate-pc)   echo "info symbol${rest}" ;;
        raw)              echo "${raw#raw }" ;;
        *) die "unknown --then command: $cmd" ;;
    esac
}

# --- assemble the GDB script ----------------------------------------------
# We use a single batch invocation. The flow:
#   1. Connect, source helpers, find kbase + relocate symbols
#   2. Set breakpoint, continue (with a host-side timeout wrapping the whole
#      gdb invocation — gdb itself doesn't timeout-abort `continue`)
#   3. Once the breakpoint hits, run each --then command
#   4. Always `delete breakpoints` and detach before quit
CACHED_KBASE="$(manifest_get "$RUN_ID" kernel_runtime_base 2>/dev/null || true)"

EX_ARGS=(
    -ex "set architecture $GDB_ARCH"
    -ex "file $KERNEL"
    -ex "target remote 127.0.0.1:$GDB_PORT"
    # `target remote` to a KVM-accelerated QEMU does not reliably halt the
    # vCPUs — find-kbase's memory scan then races a moving address space and
    # `break`/`continue` fail with "target is running". Force a stop up front.
    -ex "interrupt"
    -ex "source $GDB_HELPERS_DIR/preamble.gdb"
    -ex "source $GDB_HELPERS_DIR/find-kbase.py"
    -ex "source $GDB_HELPERS_DIR/bt-tagged.py"
    -ex "source $GDB_HELPERS_DIR/decode-bugcheck.py"
    -ex "source $GDB_HELPERS_DIR/dump-thread.py"
    -ex "source $GDB_HELPERS_DIR/dump-trapframe.py"
    -ex "source $GDB_HELPERS_DIR/pool-tag.py"
    -ex "source $GDB_HELPERS_DIR/identify-object.py"
    -ex "source $GDB_HELPERS_DIR/triage-bugcheck.py"
    -ex "source $GDB_HELPERS_DIR/walk-waitlist.py"
)

if [ -n "$CACHED_KBASE" ] && [ "$CACHED_KBASE" != "null" ]; then
    EX_ARGS+=(-ex "find-kbase $CACHED_KBASE")
else
    EX_ARGS+=(-ex "find-kbase")
fi

EX_ARGS+=(
    -ex "break $BRK"
    -ex "continue"
    -ex "echo \\n=== BREAKPOINT HIT ===\\n"
)

for t in "${THENS[@]:-}"; do
    [ -z "$t" ] && continue
    cmd="$(expand_then "$t")"
    EX_ARGS+=(-ex "echo --- ${t} ---\\n" -ex "$cmd")
done

# Always clean up: delete breakpoints, detach, quit. Reach this on hit OR on
# host-side timeout-kill of the gdb process (gdb finalize handlers run).
EX_ARGS+=(
    -ex "delete breakpoints"
    -ex "detach"
    -ex "quit"
)

# --- invoke with timeout wrapper ------------------------------------------
# Capture this invocation's output to a temp file (so success detection can't
# match a prior run's `BREAKPOINT HIT` in the appended gdb.log) and tee both
# the temp file and the running gdb.log + stdout.
TMP_OUT="$(mktemp)"
trap 'rm -f "$TMP_OUT"' EXIT

{
    echo "=== run-until.sh $RUN_ID --break $BRK timeout=$TIMEOUT $(date -u +%FT%TZ) ==="
    timeout --foreground "$TIMEOUT" \
        env VIBEOS_ARCH="$ARCH" VIBEOS_KERNEL="$KERNEL" \
        gdb-multiarch -batch "${EX_ARGS[@]}" 2>&1
    rc=$?
    echo "=== gdb exit rc=$rc ==="
} | tee "$TMP_OUT" | tee -a "$GDB_LOG"

if grep -q '=== BREAKPOINT HIT ===' "$TMP_OUT"; then
    rc=0
else
    rc=124
fi

if [ "$JSON" -eq 1 ]; then
    hit=false
    [ "$rc" -eq 0 ] && hit=true
    jq -n --argjson hit "$hit" --arg brk "$BRK" --argjson timeout "$TIMEOUT" \
        '{hit: $hit, "break": $brk, timeout_secs: $timeout}'
fi

exit $rc
