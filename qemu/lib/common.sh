# Shared helpers for vibeos kernel-debugging infra.
# Source from other scripts: . "$(dirname "$0")/lib/common.sh"
#
# Provides: path constants, logging functions, arch validation.

# Resolve project root from this file's location, regardless of caller cwd.
_LIB_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
QEMU_DIR="$(dirname "$_LIB_DIR")"
VIBEOS_ROOT="$(dirname "$QEMU_DIR")"
RUNS_DIR="$QEMU_DIR/runs"

SUPPORTED_ARCHES=(i386 amd64 arm64)

# Hard wall-clock ceiling (seconds) on any QEMU we launch. Mirrors
# tests/qemu/vm.py:MAX_VM_LIFETIME_SECONDS. A wedged guest / halted-at-gdb
# debug VM / hung run must never hold a KVM slot or GBs of RAM forever.
# Override for an unusually long interactive debug session via the env var.
QEMU_MAX_LIFETIME_SECONDS="${VIBEOS_QEMU_MAX_LIFETIME:-1800}"

# Global concurrency cap: at most this many QEMUs running at once across ALL
# worktrees/sessions. Enforced with a pool of flock'd slot files (below).
# Mirrors tests/qemu/vm.py:QEMU_MAX_CONCURRENT. The cap exists so N parallel
# smoke runs queue instead of all thrashing the host's KVM/RAM at once.
QEMU_MAX_CONCURRENT="${VIBEOS_QEMU_MAX_CONCURRENT:-10}"
# How long a launcher waits for a free slot before giving up.
QEMU_QUEUE_TIMEOUT_SECONDS="${VIBEOS_QEMU_QUEUE_TIMEOUT:-1800}"
# Slot-file pool lives alongside the run lockfiles. Files are persistent
# (slot-00 .. slot-NN); the flock state is what's ephemeral — released
# automatically when the holding process (the QEMU that inherited the fd) dies.
QEMU_SLOT_DIR="/tmp/vibeos-qemu-locks/slots"

# Acquire one of QEMU_MAX_CONCURRENT global concurrency slots. The slot is held
# on fd 200 of the CALLING shell (so call directly — never in a subshell, or
# the fd closes when the subshell exits). The QEMU launched afterwards inherits
# fd 200, and the slot releases automatically when QEMU exits — this survives
# start.sh's own immediate exit via the inherited fd (flock OFD semantics,
# verified). Blocks until a slot frees or the queue timeout elapses. Sets
# SLOT_INDEX and returns 0 on success; returns 1 on timeout.
#
# CRITICAL: time spent queued here is BEFORE the caller computes the VM's
# deadline, so queue wait never counts against the 30-min lifetime cap.
# CRITICAL: any OTHER long-lived child the caller spawns after acquiring (e.g.
# the watchdog) MUST close fd 200 (`200>&-`) or it will pin the slot too.
acquire_qemu_slot() {
    mkdir -p "$QEMU_SLOT_DIR" 2>/dev/null || true
    local n="$QEMU_MAX_CONCURRENT" i waited=0
    local q_deadline=$(( $(date +%s) + QEMU_QUEUE_TIMEOUT_SECONDS ))
    while :; do
        for (( i=0; i<n; i++ )); do
            # `>>` opens for append: creates if absent, never truncates (we
            # never write — the file is purely a lock token).
            exec 200>>"$QEMU_SLOT_DIR/slot-$(printf '%02d' "$i")"
            if flock -n 200; then
                SLOT_INDEX="$i"
                [ "$waited" -gt 0 ] && log_info "acquired qemu slot $i after ${waited}s queued"
                return 0
            fi
            exec 200>&-   # busy — drop and try the next slot
        done
        if [ "$(date +%s)" -ge "$q_deadline" ]; then
            log_warn "no free qemu slot after ${QEMU_QUEUE_TIMEOUT_SECONDS}s queued (cap=$n)"
            return 1
        fi
        [ "$waited" -eq 0 ] && log_info "all $n qemu slots busy; queuing (queue time excluded from lifetime cap)..."
        sleep 2
        waited=$(( waited + 2 ))
    done
}

log_info()  { printf '[INFO]  %s\n' "$*" >&2; }
log_warn()  { printf '[WARN]  %s\n' "$*" >&2; }
log_error() { printf '[ERROR] %s\n' "$*" >&2; }
die()       { log_error "$*"; exit 1; }

arch_validate() {
    local arch="$1"
    for a in "${SUPPORTED_ARCHES[@]}"; do
        [ "$a" = "$arch" ] && return 0
    done
    die "unsupported arch '$arch'. supported: ${SUPPORTED_ARCHES[*]}"
}

# Paths derived from arch.
arch_build_dir()      { echo "$VIBEOS_ROOT/build-$1"; }
arch_kernel_image()   { echo "$VIBEOS_ROOT/build-$1/ntoskrnl/ntoskrnl.exe"; }

# Rust target dir for an arch. Cargo writes here under `ntoskrnl/rust-kernel-target/`.
arch_rust_target_dir() {
    case "$1" in
        i386)  echo "$VIBEOS_ROOT/ntoskrnl/rust-kernel-target/i686-pc-windows-gnu/release" ;;
        amd64) echo "$VIBEOS_ROOT/ntoskrnl/rust-kernel-target/x86_64-pc-windows-gnu/release" ;;
        arm64) echo "$VIBEOS_ROOT/ntoskrnl/rust-kernel-target/aarch64-pc-windows-gnullvm/release" ;;
    esac
}

# Map vibeos arch name to GDB architecture name (for `set architecture`).
arch_gdb_name() {
    case "$1" in
        i386)  echo "i386:intel" ;;
        amd64) echo "i386:x86-64" ;;
        arm64) echo "aarch64" ;;
    esac
}

# Path to llvm-mingw bin/ (per-arch objdump, nm, readelf live here).
LLVM_MINGW_BIN="$VIBEOS_ROOT/.build-tools/llvm-mingw/bin"

# Per-arch nm binary (handles PE-COFF for that arch).
arch_nm() {
    case "$1" in
        i386)  echo "$LLVM_MINGW_BIN/i686-w64-mingw32-nm" ;;
        amd64) echo "$LLVM_MINGW_BIN/x86_64-w64-mingw32-nm" ;;
        arm64) echo "$LLVM_MINGW_BIN/aarch64-w64-mingw32-nm" ;;
    esac
}

# Resolve the "owner" pid for an orphan-detection record: the first ancestor
# of `start_pid` that ISN'T a shell. This is the long-lived process that
# really owns the run (claude, tmux, sshd, gnome-terminal-server, etc.).
#
# Why not $PPID? In Claude's Bash tool, each tool call spawns a fresh per-call
# bash that dies the moment the call returns; using PPID would make every
# QEMU launched by a tool call look orphaned to the next call's reaper.
# In `$(qemu/start.sh ...)` pipelines, PPID is the transient $() subshell.
# In a normal terminal flow this lookup just walks past the user's bash and
# lands on tmux / the terminal emulator, which is exactly the long-lived
# parent we want to track for liveness.
#
# Falls back to start_pid itself if no non-shell ancestor exists (PID 1).
owner_pid_for() {
    local p="${1:-$$}"
    local pp comm
    for _ in 1 2 3 4 5 6 7 8 9 10 11 12; do
        [ -z "$p" ] && break
        [ "$p" = "1" ] && break
        comm="$(ps -o comm= -p "$p" 2>/dev/null | tr -d ' ')"
        [ -z "$comm" ] && break
        case "$comm" in
            bash|sh|dash|zsh|fish|ksh|-bash|-sh|-zsh|-fish|-ksh|-dash)
                pp="$(ps -o ppid= -p "$p" 2>/dev/null | tr -d ' ')"
                [ -z "$pp" ] && break
                p="$pp"
                ;;
            *)
                # Found first non-shell ancestor.
                echo "$p"
                return 0
                ;;
        esac
    done
    # Fell off the top — return the original.
    echo "${1:-$$}"
}

# Locate llvm-dwarfdump. Prefers PATH; falls back to known install locations.
# Empty output if not found — caller decides what to do.
find_llvm_dwarfdump() {
    if command -v llvm-dwarfdump >/dev/null 2>&1; then
        command -v llvm-dwarfdump
        return 0
    fi
    # Common Ubuntu / ROCm install paths.
    for c in \
        /opt/rocm-*/lib/llvm/bin/llvm-dwarfdump \
        /usr/lib/llvm-*/bin/llvm-dwarfdump
    do
        [ -x "$c" ] && { echo "$c"; return 0; }
    done
    return 1
}
