# Manifest read/write helpers. Source after lib/common.sh.
#
# Each run lives at $RUNS_DIR/<run-id>/, with manifest.json holding metadata.
# Schema (canonical):
#
#   {
#     "run_id":         "amd64-20260502-093456",
#     "arch":           "amd64",
#     "qemu_pid":       12345,
#     "qemu_cmd":       "qemu-system-x86_64 -M pc ...",
#     "owner_pid":      8529,
#     "owner_comm":     "claude",
#     "deadline":       1748550000,   # epoch secs; hard 30-min lifetime cap
#     "gdb_port":       12001,
#     "qmp_socket":     "/abs/path/to/qmp.sock",
#     "serial_socket":  "/abs/path/to/serial.sock",
#     "start_time":     "2026-05-02T09:34:56Z",
#     "kernel_image":   "/abs/path/to/build-amd64/ntoskrnl/ntoskrnl.exe",
#     "kernel_mtime":   "2026-05-02T09:30:00Z",
#     "git_sha":        "a2d5a6e4...",
#     "git_dirty":      false
#   }
#
# owner_pid is the PID of the first non-shell ancestor of start.sh — the
# long-lived session that owns this run (claude, tmux, sshd, terminal
# emulator, etc.). See owner_pid_for() in lib/common.sh for the rationale.
# qemu/reap.sh uses this to identify orphans: when owner_pid is dead but
# qemu_pid is alive, the owning session died without stopping QEMU and the
# run should be reaped. owner_comm is the recorded command name at launch
# time, for forensic clarity in stale manifests.
#
# deadline is launch-time + the hard lifetime cap (default 30 min, override via
# $VIBEOS_QEMU_MAX_LIFETIME). qemu/reap.sh kills any VM past its deadline even
# if the owner is still alive — the cap is an absolute ceiling, not an orphan
# check. Set by start.sh after launch (not in manifest_init, which runs before
# the QEMU clock starts).
#
# Fields populated by start.sh; other tools read but never modify (except
# stop.sh, which sets a "stopped_at" field). The schema is intentionally flat:
# nesting would only make field lookups longer.

manifest_path() {
    local run_id="$1"
    echo "$RUNS_DIR/$run_id/manifest.json"
}

manifest_exists() {
    [ -f "$(manifest_path "$1")" ]
}

# Read a single field. Usage: manifest_get <run-id> <field>
manifest_get() {
    local run_id="$1" field="$2"
    local path
    path="$(manifest_path "$run_id")"
    [ -f "$path" ] || die "no manifest for run-id '$run_id'"
    vjson get "$path" "$field"
}

# Write the initial manifest at run-creation time. Only fields known up-front
# are populated; gdb_port etc. are filled by manifest_set after port allocation
# and qemu_pid is filled after launching qemu.
manifest_init() {
    local run_id="$1" arch="$2" kernel_image="$3" owner_pid="${4:-}"
    local run_dir="$RUNS_DIR/$run_id"
    mkdir -p "$run_dir"

    # Default to the first-non-shell-ancestor walk if no owner explicitly
    # passed; start.sh always passes one so this fallback rarely fires.
    # Seed from $PPID (manifest.sh is sourced into start.sh, so $$ would be
    # start.sh's own pid → walk would land on start.sh itself, which dies
    # immediately and would mark the run orphaned).
    [ -z "$owner_pid" ] && owner_pid="$(owner_pid_for "$PPID")"
    local owner_comm
    owner_comm=""
    { read -r owner_comm < "/proc/$owner_pid/comm"; } 2>/dev/null

    local kernel_mtime=""
    if [ -f "$kernel_image" ]; then
        kernel_mtime="$(date -u -d "@$(stat -c %Y "$kernel_image")" +%FT%TZ 2>/dev/null || echo "")"
    fi

    local git_sha="" git_dirty="false"
    if git -C "$VIBEOS_ROOT" rev-parse HEAD >/dev/null 2>&1; then
        git_sha="$(git -C "$VIBEOS_ROOT" rev-parse HEAD)"
        if [ -n "$(git -C "$VIBEOS_ROOT" status --porcelain 2>/dev/null)" ]; then
            git_dirty="true"
        fi
    fi

    vjson write "$(manifest_path "$run_id")" \
        --arg run_id "$run_id" \
        --arg arch "$arch" \
        --argjson qemu_pid null \
        --argjson qemu_cmd null \
        --argjson owner_pid "${owner_pid:-null}" \
        --arg owner_comm "${owner_comm:-}" \
        --argjson gdb_port null \
        --argjson qmp_socket null \
        --argjson serial_socket null \
        --arg start_time "$(date -u +%FT%TZ)" \
        --arg kernel_image "$kernel_image" \
        --arg kernel_mtime "$kernel_mtime" \
        --arg git_sha "$git_sha" \
        --argjson git_dirty "$git_dirty"
}

# Set a single field. Numeric values pass --argjson; strings pass --arg.
# Usage: manifest_set <run-id> <field> <value> [--json]
manifest_set() {
    local run_id="$1" field="$2" value="$3" mode="${4:---arg}"
    local path
    path="$(manifest_path "$run_id")"
    [ -f "$path" ] || die "no manifest for run-id '$run_id'"
    if [ "$mode" = "--json" ]; then
        vjson set "$path" "$field" "$value" --json
    else
        vjson set "$path" "$field" "$value"
    fi
}
