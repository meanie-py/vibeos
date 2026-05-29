#!/bin/bash
# Spawn a QEMU instance for kernel debugging.
#
# Usage:
#   qemu/start.sh <arch> [options]
#
# Options:
#   --gdb             Open gdbstub on auto-allocated port
#   --gdb-wait        Open gdbstub AND halt at first instruction (for boot-from-zero)
#   --livecd          Boot from livecd.iso (i386/amd64; default; ignored for arm64)
#   --skip-preflight  Skip preflight checks (use only when iterating preflight itself)
#
# On success: prints the run-id to stdout. All other output goes to stderr.
# On failure: nonzero exit, error to stderr.

set -u

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
. "$SCRIPT_DIR/lib/common.sh"
. "$SCRIPT_DIR/lib/manifest.sh"
. "$SCRIPT_DIR/lib/runid.sh"

# Resolve the owning session NOW, before anything else. owner_pid_for walks
# parents past shell processes (bash/zsh/sh/dash/fish/ksh) and returns the
# first long-lived non-shell ancestor — typically claude, tmux, sshd, or a
# terminal-emulator server. That's what qemu/reap.sh uses as the liveness
# key to decide whether this run is orphaned. We seed the walk at $PPID
# (not $$) because start.sh itself is short-lived; we want to skip past the
# calling bash (per-tool-call in claude, login in terminal, transient in $()).
# See lib/common.sh:owner_pid_for for the rationale in full.
OWNER_PID="$(owner_pid_for "$PPID")"

# --- arg parsing ------------------------------------------------------------
ARCH=""
GDB=0
GDB_WAIT=0
LIVECD=1
SKIP_PREFLIGHT=0

while [ $# -gt 0 ]; do
    case "$1" in
        --gdb)            GDB=1 ;;
        --gdb-wait)       GDB=1; GDB_WAIT=1 ;;
        --livecd)         LIVECD=1 ;;
        --skip-preflight) SKIP_PREFLIGHT=1 ;;
        --help|-h)
            sed -n '2,15p' "$0"
            exit 0
            ;;
        i386|amd64|arm64) ARCH="$1" ;;
        *) die "unknown argument: $1" ;;
    esac
    shift
done

[ -n "$ARCH" ] || die "arch required (i386 | amd64 | arm64)"
arch_validate "$ARCH"

# --- preflight --------------------------------------------------------------
if [ "$SKIP_PREFLIGHT" -eq 0 ]; then
    if ! "$SCRIPT_DIR/preflight.sh" "$ARCH" >&2; then
        die "preflight failed for $ARCH — fix issues above before starting"
    fi
fi

# --- parent-aware orphan reap ----------------------------------------------
# Sweep any prior QEMU (this arch or any other, this worktree or any other)
# whose owning shell/launcher is dead. Live owners — including other agents'
# active sessions — are NEVER touched. See qemu/reap.sh for the exact rules.
# reap.sh writes [reap] action lines to stderr (which flows through to the
# user) and the kill count to stdout (which we capture for the summary log).
if reaped="$("$SCRIPT_DIR/reap.sh")"; then
    [ "${reaped:-0}" -gt 0 ] && log_info "reaped $reaped orphan QEMU(s)"
else
    log_warn "qemu/reap.sh exited nonzero; continuing"
fi

# --- allocate run-id, ports, sockets ---------------------------------------
RUN_ID="$(runid_new "$ARCH")"
RUN_DIR="$RUNS_DIR/$RUN_ID"
mkdir -p "$RUN_DIR"

KERNEL_IMAGE="$(arch_kernel_image "$ARCH")"
manifest_init "$RUN_ID" "$ARCH" "$KERNEL_IMAGE" "$OWNER_PID"

GDB_PORT=""
if [ "$GDB" -eq 1 ]; then
    GDB_PORT="$(find_free_port 12000 12999)"
    manifest_set "$RUN_ID" gdb_port "$GDB_PORT" --json
fi

QMP_SOCK="$(runid_qmp_socket "$RUN_ID")"
SERIAL_LOG="$(runid_serial_log "$RUN_ID")"
PID_FILE="$(runid_pid_file "$RUN_ID")"
QEMU_LAUNCH_LOG="$RUN_DIR/qemu.launch.log"

manifest_set "$RUN_ID" qmp_socket "$QMP_SOCK"

# Pre-create empty serial.log so wait-for.sh can tail before QEMU writes.
: > "$SERIAL_LOG"

# --- build QEMU command line ----------------------------------------------
# We write serial straight to a file. Earlier we tried `-chardev socket,...,
# logfile=...` so a future interactive client could attach, but QEMU buffers
# socket output until a client connects — the logfile stayed empty. The file
# chardev guarantees flush-as-you-go. If interactive serial becomes useful
# later, expose it via a separate QMP-driven mechanism rather than reviving
# the socket+logfile combo.
SERIAL_FLAGS=(
    -serial "file:$SERIAL_LOG"
)

QMP_FLAGS=(
    -qmp "unix:$QMP_SOCK,server=on,wait=off"
)

GDB_FLAGS=()
if [ "$GDB" -eq 1 ]; then
    GDB_FLAGS+=(-gdb "tcp:127.0.0.1:$GDB_PORT")
    [ "$GDB_WAIT" -eq 1 ] && GDB_FLAGS+=(-S)
fi

DAEMON_FLAGS=(-display none)

# --- per-arch QEMU command -------------------------------------------------
declare -a QEMU_CMD

case "$ARCH" in
    i386)
        ISO="$(arch_build_dir i386)/livecd.iso"
        [ -f "$ISO" ] || die "missing $ISO — run: ninja -C $(arch_build_dir i386) livecd"
        KVM_FLAGS=()
        if [ -r /dev/kvm ]; then
            KVM_FLAGS=(-accel kvm -cpu host)
        else
            KVM_FLAGS=(-cpu Westmere)
        fi
        QEMU_CMD=(
            qemu-system-i386 -M pc "${KVM_FLAGS[@]}" -m 512M
            -cdrom "$ISO" -no-reboot
            "${SERIAL_FLAGS[@]}" "${QMP_FLAGS[@]}" "${GDB_FLAGS[@]}" "${DAEMON_FLAGS[@]}"
        )
        ;;
    amd64)
        ISO="$(arch_build_dir amd64)/livecd.iso"
        [ -f "$ISO" ] || die "missing $ISO — run: ninja -C $(arch_build_dir amd64) livecd"
        KVM_FLAGS=(-accel kvm -cpu host)
        [ -r /dev/kvm ] || die "amd64 requires /dev/kvm but it's not readable; check kvm group membership"
        QEMU_CMD=(
            qemu-system-x86_64 -M pc "${KVM_FLAGS[@]}" -m 512M
            -cdrom "$ISO" -no-reboot
            "${SERIAL_FLAGS[@]}" "${QMP_FLAGS[@]}" "${GDB_FLAGS[@]}" "${DAEMON_FLAGS[@]}"
        )
        ;;
    arm64)
        UEFI_FW="$QEMU_DIR/QEMU_EFI.fd"
        EFIVARS="$QEMU_DIR/efivars.fd"
        BOOT_IMG="$QEMU_DIR/boot.img"
        [ -f "$BOOT_IMG" ] || die "missing $BOOT_IMG — run: bash $QEMU_DIR/mkramdisk.sh"
        [ -f "$UEFI_FW" ] || die "missing $UEFI_FW (UEFI firmware)"
        QEMU_CMD=(
            qemu-system-aarch64 -M virt -cpu cortex-a72 -m 1G
            -drive "if=pflash,format=raw,file=$UEFI_FW,readonly=on"
            -drive "if=pflash,format=raw,file=$EFIVARS"
            -drive "file=$BOOT_IMG,format=raw,if=virtio"
            -device ramfb
            "${SERIAL_FLAGS[@]}" "${QMP_FLAGS[@]}" "${GDB_FLAGS[@]}" "${DAEMON_FLAGS[@]}"
        )
        ;;
esac

# --- launch QEMU as a daemon ----------------------------------------------
# Detach from terminal: stdin from /dev/null, stdout+stderr to qemu.launch.log.
# disown so the shell exit doesn't hand it a SIGHUP.
log_info "launching QEMU for $ARCH (run-id=$RUN_ID)"
QEMU_CMD_STR="${QEMU_CMD[*]}"
log_info "command: $QEMU_CMD_STR"

setsid "${QEMU_CMD[@]}" </dev/null >"$QEMU_LAUNCH_LOG" 2>&1 &
QEMU_PID=$!
disown "$QEMU_PID" 2>/dev/null || true

echo "$QEMU_PID" > "$PID_FILE"
manifest_set "$RUN_ID" qemu_pid "$QEMU_PID" --json
manifest_set "$RUN_ID" qemu_cmd "$QEMU_CMD_STR"

# Drop a shared lockfile so the reaper can reach this run cross-worktree.
# RUNS_DIR is per-worktree (qemu/runs/) so a sibling worktree's reaper can't
# see our manifest, but /tmp/vibeos-qemu-locks/ is global. Schema mirrors
# tests/qemu/vm.py: launcher_pid = the long-lived owner whose death means
# orphan (claude / tmux / sshd / terminal).
LOCK_DIR="/tmp/vibeos-qemu-locks"
mkdir -p "$LOCK_DIR" 2>/dev/null || true
LOCK_FILE="$LOCK_DIR/${OWNER_PID}-${QEMU_PID}.json"
jq -n \
    --argjson launcher_pid "$OWNER_PID" \
    --argjson qemu_pid "$QEMU_PID" \
    --arg arch "$ARCH" \
    --arg run_id "$RUN_ID" \
    --arg manifest_path "$(manifest_path "$RUN_ID")" \
    --arg source "qemu/start.sh" \
    '{launcher_pid: $launcher_pid, qemu_pid: $qemu_pid, arch: $arch,
      run_id: $run_id, manifest_path: $manifest_path, source: $source}' \
    > "$LOCK_FILE" 2>/dev/null || log_warn "could not write $LOCK_FILE"

# --- wait briefly for QEMU to actually start ------------------------------
# If it dies in the first 1s, almost always a bad command line — surface the
# launch log so the agent doesn't have to dig for it.
for _ in 1 2 3 4 5; do
    if ! kill -0 "$QEMU_PID" 2>/dev/null; then
        log_error "QEMU exited within 1s — launch log:"
        cat "$QEMU_LAUNCH_LOG" >&2 || true
        die "QEMU failed to start (run-id=$RUN_ID)"
    fi
    sleep 0.2
done

log_info "QEMU running: pid=$QEMU_PID gdb_port=${GDB_PORT:-none}"
echo "$RUN_ID"
