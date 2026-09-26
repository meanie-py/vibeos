#!/bin/bash
# Run a kernel until a named symbol is hit, then run inspection commands at
# the breakpoint, all in one batch GDB invocation.
#
# Usage:
#   qemu/run-until.sh <run-id> --break <symbol> --timeout <secs>
#                              [--break-if <gdb-condition>]
#                              [--skip <n>]
#                              [--then <inspect-command>]...
#                              [--json]
#
# --break-if takes a gdb condition expression, evaluated at the breakpoint:
#     --break "IopCompleteRequest" --break-if "Irp->IoStatus.Information > 0x40"
# --skip ignores the first <n> hits:
#     --break "ExAllocatePoolWithTag" --skip 5000
# Both exist for HOT sites. An unconditional breakpoint on a path that fires
# thousands of times per boot stops on the first, uninteresting hit; that made
# `--break` useless at exactly the sites worth watching.
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
BRK_IF=""
BRK_SKIP=""
TIMEOUT=120
THENS=()
JSON=0
shift 2>/dev/null || true
while [ $# -gt 0 ]; do
    case "$1" in
        --break)   shift; BRK="$1" ;;
        --break-if) shift; BRK_IF="$1" ;;
        --skip)    shift; BRK_SKIP="$1" ;;
        --timeout) shift; TIMEOUT="$1" ;;
        --then)    shift; THENS+=("$1") ;;
        --json)    JSON=1 ;;
        *) die "unknown argument: $1" ;;
    esac
    shift
done

[ -n "$RUN_ID" ] || die "usage: run-until.sh <run-id> --break <symbol> [--break-if <cond>] [--skip <n>] --timeout <secs> [--then <cmd>]"
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
    -ex "source $GDB_HELPERS_DIR/run-until-bp.py"
)

if [ -n "$CACHED_KBASE" ] && [ "$CACHED_KBASE" != "null" ]; then
    EX_ARGS+=(-ex "find-kbase $CACHED_KBASE")
else
    EX_ARGS+=(-ex "find-kbase")
fi

# bounded-continue interrupts cleanly from inside gdb after $TIMEOUT seconds if
# the breakpoint never fires (printing a NOT-HIT marker), so the --then/cleanup
# commands below always run against a STOPPED target rather than crashing with
# "Cannot execute while running". The host-side `timeout` (below) is a wider
# safety net only — it should never be what fires.
# --break-if / --skip exist because an unconditional breakpoint is unusable at a
# HOT site. The IRP buffered-completion copy-back fires thousands of times per
# boot, so `--break <symbol>` there stops on the first, uninteresting one and
# tells you nothing — the instrument did not fit the site even once breakpoints
# themselves were working. gdb has had the answer all along (`break X if COND`,
# `ignore N COUNT`); run-until.sh just never exposed it.
# `break X if COND` parses COND in the CURRENT frame, which at set time is
# wherever the guest happens to be — so a condition naming the callee's own
# parameters fails with "No symbol 'NumberOfBytes' in current context", and gdb
# then leaves NO breakpoint set while the script sails on into
# "Cannot execute this command while the target is running". `condition N COND`
# parses in the breakpoint's own scope instead, which is what the caller means.
EX_ARGS+=(-ex "break $BRK")
if [ -n "$BRK_IF" ]; then
    # If the expression does not resolve, gdb prints "No symbol 'X' in current
    # context" and LEAVES THE BREAKPOINT UNCONDITIONAL — it then fires on the
    # first hit and the caller believes it was filtered. That is a silent
    # degradation of exactly the kind that wastes a debugging session, so the
    # marker below is grepped for after the run and turned into a hard failure.
    # (Conditions naming a callee's parameters are fragile here: a breakpoint on
    # a Rust symbol often resolves into an INLINED helper whose scope has no such
    # name. Register conditions -- `$x1 > 0x4000` -- always parse.)
    EX_ARGS+=(-ex "condition 1 $BRK_IF" -ex "echo ===COND-SET===\n")
fi
# `ignore` needs the breakpoint number; ours is always 1 here because
# bounded-continue runs against a freshly-`delete`d breakpoint table.
[ -n "$BRK_SKIP" ] && EX_ARGS+=(-ex "ignore 1 $BRK_SKIP")
EX_ARGS+=(-ex "bounded-continue $TIMEOUT")

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

# Host timeout is a wider safety net than the in-gdb bounded-continue budget
# (which interrupts gracefully at $TIMEOUT). +30s covers gdb teardown/detach so
# this kill only fires if bounded-continue itself wedges.
HARD_TIMEOUT=$(( TIMEOUT + 30 ))
{
    echo "=== run-until.sh $RUN_ID --break $BRK timeout=$TIMEOUT $(date -u +%FT%TZ) ==="
    timeout --foreground "$HARD_TIMEOUT" \
        env VIBEOS_ARCH="$ARCH" VIBEOS_KERNEL="$KERNEL" \
        gdb-multiarch -batch "${EX_ARGS[@]}" 2>&1
    rc=$?
    echo "=== gdb exit rc=$rc ==="
} | tee "$TMP_OUT" | tee -a "$GDB_LOG"

if [ -n "$BRK_IF" ] && grep -q "No symbol .* in current context" "$TMP_OUT"; then
    log_error "--break-if '$BRK_IF' did not resolve; gdb left the breakpoint"
    log_error "UNCONDITIONAL, so any hit below is unfiltered. Refusing to report it."
    log_error "A breakpoint on a Rust symbol often resolves into an inlined helper"
    log_error "whose scope lacks the callee's parameter names — use a register"
    log_error "condition instead (e.g. --break-if '\$x1 > 0x4000')."
    exit 3
fi
if grep -q '=== BREAKPOINT HIT ===' "$TMP_OUT"; then
    rc=0
else
    rc=124
fi

if [ "$JSON" -eq 1 ]; then
    hit=false
    [ "$rc" -eq 0 ] && hit=true
    vjson obj --argjson hit "$hit" --arg break "$BRK" --argjson timeout_secs "$TIMEOUT"
fi

exit $rc
