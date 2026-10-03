# Run-id allocation and port allocation helpers.
# Source after lib/common.sh.

# Generate a fresh run-id of the form <arch>-YYYYMMDDHHMMSS. If a directory
# already exists at that timestamp (rare, sub-second double-call), suffix with
# the PID of this script to disambiguate.
runid_new() {
    local arch="$1"
    local stamp
    stamp="$(date +%Y%m%d-%H%M%S)"
    local id="${arch}-${stamp}"
    if [ -e "$RUNS_DIR/$id" ]; then
        id="${id}-$$"
    fi
    echo "$id"
}

# True if a TCP port is free on localhost. Uses /dev/tcp (no extra deps).
port_is_free() {
    local port="$1"
    # bash /dev/tcp returns 0 if connected, nonzero on refused/timeout.
    # We treat "refused" as free. Use a short timeout to dodge hangs.
    if timeout 0.2 bash -c "exec 3<>/dev/tcp/127.0.0.1/$port" 2>/dev/null; then
        # Connection succeeded → something is listening → not free.
        exec 3>&- 3<&- 2>/dev/null || true
        return 1
    fi
    return 0
}

# Find a free TCP port starting from $1, scanning upward. We use the gdbstub
# range 12000-12999 by default to avoid collisions with anything else on the
# host (QEMU's default :1234 is collision-prone with other tooling).
find_free_port() {
    local start="${1:-12000}" end="${2:-12999}"
    local p="$start"
    while [ "$p" -le "$end" ]; do
        if port_is_free "$p"; then
            echo "$p"
            return 0
        fi
        p=$((p + 1))
    done
    die "no free port in range $start-$end"
}

# Per-run socket paths. Always absolute so QEMU and GDB resolve identically.
runid_qmp_socket()    { echo "$RUNS_DIR/$1/qmp.sock"; }
runid_serial_socket() { echo "$RUNS_DIR/$1/serial.sock"; }
runid_serial_log()    { echo "$RUNS_DIR/$1/serial.log"; }
runid_gdb_log()       { echo "$RUNS_DIR/$1/gdb.log"; }

# runid_kernel_image <run-id> — the kernel image freeldr ACTUALLY loaded for this run, from
# its "[FLDR] load ...\<image>" serial line; the manifest's guess until that line exists.
# The UP and MP kernels are separate images whose data symbols sit 0x4000 apart, so a gdb
# session on the wrong one reads every global from the wrong address: PsLoadedModuleList
# "held" a physical-looking 0x52229358 and lm/who-holds walked garbage on an MP arm64 boot.
runid_kernel_image() {
    local manifest serial loaded
    manifest="$(manifest_get "$1" kernel_image)"
    serial="$(runid_serial_log "$1")"
    loaded="$(grep -a -o -m1 -E 'FLDR\] load .*\\(ntoskrnl|ntkrnlmp)\.exe' "$serial" 2>/dev/null \
              | grep -o -E '(ntoskrnl|ntkrnlmp)\.exe$')"
    case "$loaded" in
        ntkrnlmp.exe) arch_kernel_image "$(manifest_get "$1" arch)" mp ;;
        ntoskrnl.exe) arch_kernel_image "$(manifest_get "$1" arch)" up ;;
        *) echo "$manifest" ;;
    esac
}
# runid_kernel_base <run-id> — the running kernel's image base: the manifest's cached value,
# else the guest's own "[KRNLBASE] ntoskrnl runtime base=0x..." serial line. find-kbase's
# memory scan cannot see the kernel when the vCPU is stopped at EL0 (a user-mode spin is
# exactly when you attach), and answered "not found in expected ranges" while the base sat in
# the serial log all along.
runid_kernel_base() {
    local cached
    cached="$(manifest_get "$1" kernel_runtime_base 2>/dev/null || true)"
    if [ -n "$cached" ] && [ "$cached" != "null" ]; then
        echo "$cached"
        return
    fi
    grep -a -o -m1 -E 'KRNLBASE\] ntoskrnl runtime base=0x[0-9A-Fa-f]+' "$(runid_serial_log "$1")" 2>/dev/null \
        | grep -o -E '0x[0-9A-Fa-f]+$' | tr 'A-F' 'a-f' | sed 's/^0X/0x/'
}
runid_pid_file()      { echo "$RUNS_DIR/$1/pid"; }
