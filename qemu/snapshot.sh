#!/bin/bash
# Post-boot VM snapshots for fast debug iteration.
#
# Saving a running VM's full state to a file and restoring it lets a debug cycle
# skip the cold boot (~50s on arm64 TCG → ~1-2s restore). Restore lands at the
# exact captured point; attach gdb / run-until / inspect against it unchanged.
#
# Usage:
#   qemu/snapshot.sh save <run-id> [--name <name>]   # capture a running VM
#   qemu/snapshot.sh restore <name> [--gdb] [--allow-stale]
#   qemu/snapshot.sh list                            # list snapshots + staleness
#   qemu/snapshot.sh rm <name>
#
# IMPORTANT — snapshot at a QUIESCENT point, not mid-churn. Migration-to-file is
# faithful when the guest is settled (idle loop, well past "shell ready"), but
# capturing during high activity (e.g. exactly at shell-ready, while explorer is
# spawning threads) perturbs ~20% of restores into a spurious post-resume fault.
# Wait for the system to go quiet before `save`. (arm64-virt finding, 2026-06.)
#
# Mechanism: QMP `stop` + `migrate "exec:cat > file"` (RAM + device state). On
# restore, start.sh launches QEMU with `-incoming "exec:cat < file"` and the same
# boot media (re-opened read-consistent), then we issue the post-load `cont`.

set -u

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
. "$SCRIPT_DIR/lib/common.sh"
. "$SCRIPT_DIR/lib/manifest.sh"
. "$SCRIPT_DIR/lib/runid.sh"

SNAP_DIR="$SCRIPT_DIR/snapshots"
mkdir -p "$SNAP_DIR"

# sha256 of a file, or "missing".
_sha() { [ -f "$1" ] && sha256sum "$1" 2>/dev/null | cut -d' ' -f1 || echo missing; }

# Wait for the QMP socket, drive the incoming-migration load, then `cont`.
# Args: <qmp-sock>. Prints final status.
_qmp_wait_and_cont() {
    python3 - "$1" <<'PY'
import socket, json, sys, time
qmp = sys.argv[1]
for _ in range(150):
    try:
        s = socket.socket(socket.AF_UNIX); s.connect(qmp); break
    except OSError:
        time.sleep(0.2)
else:
    print("ERROR: QMP socket never appeared"); sys.exit(1)
s.settimeout(60); f = s.makefile('rw'); f.readline()
def cmd(d):
    f.write(json.dumps(d) + "\n"); f.flush()
    while True:
        l = f.readline()
        if not l: return None
        m = json.loads(l)
        if 'return' in m or 'error' in m: return m
cmd({"execute": "qmp_capabilities"})
# Wait for the incoming stream to finish loading (status leaves 'inmigrate').
st = None
for _ in range(240):
    r = cmd({"execute": "query-status"})
    st = (r or {}).get('return', {}).get('status')
    if st in ('paused', 'running'): break
    time.sleep(0.25)
if st == 'paused':
    cmd({"execute": "cont"})
print((cmd({"execute": "query-status"}) or {}).get('return', {}).get('status', '?'))
PY
}

# QMP stop + migrate-to-file. Args: <qmp-sock> <out-file>. Prints completed/failed.
_qmp_save() {
    python3 - "$1" "$2" <<'PY'
import socket, json, sys, time
qmp, out = sys.argv[1], sys.argv[2]
s = socket.socket(socket.AF_UNIX); s.connect(qmp); s.settimeout(120)
f = s.makefile('rw'); f.readline()
def cmd(d):
    f.write(json.dumps(d) + "\n"); f.flush()
    while True:
        l = f.readline()
        if not l: return None
        m = json.loads(l)
        if 'return' in m or 'error' in m: return m
cmd({"execute": "qmp_capabilities"})
cmd({"execute": "stop"})
cmd({"execute": "migrate-set-parameters", "arguments": {"max-bandwidth": 4 << 30}})
r = cmd({"execute": "migrate", "arguments": {"uri": "exec:cat > %s" % out}})
if r and 'error' in r:
    print("failed: %s" % r['error'].get('desc', '?')); sys.exit(1)
for _ in range(600):
    r = cmd({"execute": "query-migrate"})
    st = (r or {}).get('return', {}).get('status')
    if st in ('completed', 'failed', 'cancelled'):
        print(st)
        if st != 'completed':
            sys.exit(1)
        break
    time.sleep(0.25)
PY
}

cmd_save() {
    local run_id="" name=""
    while [ $# -gt 0 ]; do
        case "$1" in
            --name) shift; name="$1" ;;
            *) [ -z "$run_id" ] && run_id="$1" || die "unexpected arg: $1" ;;
        esac
        shift
    done
    [ -n "$run_id" ] || die "save: usage: snapshot.sh save <run-id> [--name <name>]"
    manifest_exists "$run_id" || die "no such run-id: $run_id"

    local arch qmp kernel
    arch="$(manifest_get "$run_id" arch)"
    qmp="$(manifest_get "$run_id" qmp_socket)"
    kernel="$(manifest_get "$run_id" kernel_image)"
    [ -S "$qmp" ] || die "run $run_id has no live QMP socket ($qmp)"

    [ -n "$name" ] || name="${arch}-postboot"
    local mig="$SNAP_DIR/$name.mig" meta="$SNAP_DIR/$name.meta"

    log_info "saving VM state of $run_id → $mig (this captures full RAM; ~350M)"
    local res; res="$(_qmp_save "$qmp" "$mig")"
    if [ "$res" != "completed" ]; then
        rm -f "$mig"
        die "migration failed ($res) — VM left stopped; check qemu/runs/$run_id"
    fi

    # Stamp provenance so a stale restore (kernel rebuilt since) is detectable.
    {
        echo "name=$name"
        echo "arch=$arch"
        echo "kernel_image=$kernel"
        echo "kernel_sha=$(_sha "$kernel")"
        echo "src_run=$run_id"
        echo "size=$(stat -c%s "$mig" 2>/dev/null || echo 0)"
    } > "$meta"

    log_info "snapshot '$name' saved ($(numfmt --to=iec "$(stat -c%s "$mig")" 2>/dev/null || echo '?'))"
    log_info "source VM $run_id is now stopped; reclaiming it"
    bash "$SCRIPT_DIR/stop.sh" "$run_id" >&2 2>&1 || true
    echo "$name"
}

cmd_restore() {
    local name="" gdb=0 allow_stale=0
    while [ $# -gt 0 ]; do
        case "$1" in
            --gdb) gdb=1 ;;
            --allow-stale) allow_stale=1 ;;
            *) [ -z "$name" ] && name="$1" || die "unexpected arg: $1" ;;
        esac
        shift
    done
    [ -n "$name" ] || die "restore: usage: snapshot.sh restore <name> [--gdb] [--allow-stale]"
    local mig="$SNAP_DIR/$name.mig" meta="$SNAP_DIR/$name.meta"
    [ -f "$mig" ] || die "no such snapshot: $name (looked for $mig)"
    [ -f "$meta" ] || die "snapshot $name has no .meta sidecar"

    local arch kernel kernel_sha
    arch="$(sed -n 's/^arch=//p' "$meta")"
    kernel="$(sed -n 's/^kernel_image=//p' "$meta")"
    kernel_sha="$(sed -n 's/^kernel_sha=//p' "$meta")"

    # Staleness: the snapshot embeds the OLD kernel in RAM; gdb symbolication
    # uses the ON-DISK image. If the kernel was rebuilt since the snapshot, the
    # two diverge and symbols mislead. Fail loud (mirrors smoke's stale-media
    # refusal); --allow-stale to override knowingly.
    local now_sha; now_sha="$(_sha "$kernel")"
    if [ "$now_sha" != "$kernel_sha" ]; then
        log_warn "STALE snapshot: $kernel changed since '$name' was saved"
        log_warn "  snapshot kernel sha: $kernel_sha"
        log_warn "  on-disk  kernel sha: $now_sha"
        log_warn "  → the restored RAM runs the OLD kernel; gdb symbols (on-disk) WON'T match."
        if [ "$allow_stale" -eq 0 ]; then
            die "refusing stale restore; re-save the snapshot, or pass --allow-stale"
        fi
        log_warn "  --allow-stale: proceeding anyway"
    fi

    local gdbflag=(); [ "$gdb" -eq 1 ] && gdbflag=(--gdb)
    log_info "restoring '$name' ($arch) via start.sh --incoming"
    local run_id
    run_id="$(bash "$SCRIPT_DIR/start.sh" "$arch" "${gdbflag[@]}" --incoming "$mig" --skip-preflight)"
    [ -n "$run_id" ] || die "start.sh --incoming failed"

    local qmp; qmp="$(runid_qmp_socket "$run_id")"
    local status; status="$(_qmp_wait_and_cont "$qmp")"
    if [ "$status" != "running" ]; then
        log_warn "post-restore status='$status' (expected running) — VM may not have resumed"
    else
        log_info "restored '$name' → run-id $run_id (resumed, status=running)"
    fi
    echo "$run_id"
}

cmd_list() {
    local found=0
    for meta in "$SNAP_DIR"/*.meta; do
        [ -f "$meta" ] || continue
        found=1
        local name arch ksha src mig now stale
        name="$(sed -n 's/^name=//p' "$meta")"
        arch="$(sed -n 's/^arch=//p' "$meta")"
        ksha="$(sed -n 's/^kernel_sha=//p' "$meta")"
        src="$(sed -n 's/^src_run=//p' "$meta")"
        mig="$SNAP_DIR/$name.mig"
        now="$(_sha "$(sed -n 's/^kernel_image=//p' "$meta")")"
        stale="fresh"; [ "$now" != "$ksha" ] && stale="STALE (kernel rebuilt)"
        local sz="-"; [ -f "$mig" ] && sz="$(numfmt --to=iec "$(stat -c%s "$mig")" 2>/dev/null || echo '?')"
        printf '%-24s %-6s %-8s src=%s  %s\n' "$name" "$arch" "$sz" "$src" "$stale"
    done
    [ "$found" -eq 1 ] || log_info "no snapshots in $SNAP_DIR"
}

cmd_rm() {
    local name="${1:-}"
    [ -n "$name" ] || die "rm: usage: snapshot.sh rm <name>"
    rm -f "$SNAP_DIR/$name.mig" "$SNAP_DIR/$name.meta"
    log_info "removed snapshot '$name'"
}

SUB="${1:-}"; shift 2>/dev/null || true
case "$SUB" in
    save)    cmd_save "$@" ;;
    restore) cmd_restore "$@" ;;
    list)    cmd_list "$@" ;;
    rm)      cmd_rm "$@" ;;
    --help|-h|"") sed -n '2,30p' "$0"; exit 0 ;;
    *) die "unknown subcommand: $SUB (save|restore|list|rm)" ;;
esac
