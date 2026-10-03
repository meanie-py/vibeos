#!/bin/bash
# Preflight checks for the kernel-debugging infrastructure.
#
# Usage:
#   qemu/preflight.sh                    # check all 3 arches
#   qemu/preflight.sh <arch>              # check a single arch
#   qemu/preflight.sh --json              # machine-readable summary
#
# Each failure prints the EXACT command to fix it. The agent should never
# have to round-trip "what does this error mean?" — preflight tells you.

set -u

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
. "$SCRIPT_DIR/lib/common.sh"

# DWARF richness threshold on the linked ntoskrnl.exe. Empirically the i386
# kernel currently has 7459 DW_TAG entries; 100 is well above the noise floor
# and well below the lowest expected real value.
DWARF_RICHNESS_THRESHOLD=100

JSON_MODE=0
SINGLE_ARCH=""
for arg in "$@"; do
    case "$arg" in
        --json) JSON_MODE=1 ;;
        i386|amd64|arm64) SINGLE_ARCH="$arg" ;;
        --help|-h)
            sed -n '2,11p' "$0"
            exit 0
            ;;
        *) die "unknown argument: $arg" ;;
    esac
done

# --- check helpers ----------------------------------------------------------
# Each check_X function returns 0 on pass, nonzero on fail. On failure it
# prints a fix-it command (or hint) to stderr. JSON mode collects pass/fail
# state into the FAILURES array.

FAILURES=()  # "<arch>:<check>:<fix-it>" tuples, joined by tab

record_fail() {
    local arch="$1" check="$2" fixit="$3"
    FAILURES+=("$arch"$'\t'"$check"$'\t'"$fixit")
    [ "$JSON_MODE" -eq 1 ] || {
        log_error "[$arch] $check FAILED"
        log_error "        fix: $fixit"
    }
}

record_pass() {
    [ "$JSON_MODE" -eq 1 ] || log_info "[${1:-host}] ${2} OK"
}

check_gdb() {
    # Need a gdb that speaks all three architectures. gdb-multiarch is the
    # standard choice on Debian/Ubuntu; on macOS or other systems users may
    # need separate gdbs.
    if command -v gdb-multiarch >/dev/null 2>&1; then
        record_pass host "gdb-multiarch"
        return 0
    fi
    if command -v gdb >/dev/null 2>&1; then
        # plain gdb on x86 Linux can usually do x86 + amd64 but not aarch64.
        # We accept it for now but warn.
        record_fail host "gdb-multiarch" "sudo apt-get install -y gdb-multiarch"
        return 1
    fi
    record_fail host "gdb" "sudo apt-get install -y gdb-multiarch"
    return 1
}


check_dwarf_tool() {
    if [ -n "$DWARF_DUMP" ]; then
        record_pass host "llvm-dwarfdump ($DWARF_DUMP)"
        return 0
    fi
    if command -v objdump >/dev/null 2>&1; then
        # Acceptable for i386/amd64 only; arm64 will fail later.
        record_pass host "objdump (x86 only — install llvm-dwarfdump for arm64 support)"
        return 0
    fi
    record_fail host "DWARF dump tool" \
        "sudo apt-get install -y llvm  (provides llvm-dwarfdump for all archs)"
    return 1
}

check_qemu() {
    local arch="$1"
    local bin
    case "$arch" in
        i386)  bin="qemu-system-i386" ;;
        amd64) bin="qemu-system-x86_64" ;;
        arm64) bin="qemu-system-aarch64" ;;
    esac
    if command -v "$bin" >/dev/null 2>&1; then
        record_pass "$arch" "qemu ($bin)"
        return 0
    fi
    record_fail "$arch" "qemu ($bin)" "sudo apt-get install -y qemu-system-x86 qemu-system-arm"
    return 1
}

check_kernel_built() {
    local arch="$1"
    local img
    img="$(arch_kernel_image "$arch")"
    if [ -f "$img" ]; then
        record_pass "$arch" "kernel image"
        return 0
    fi
    record_fail "$arch" "kernel image" "ninja -C $(arch_build_dir "$arch") ntoskrnl"
    return 1
}

# Count DW_TAG_variable + DW_TAG_subprogram in DWARF. Used both for the
# linked-binary primary check and for per-axis diagnosis.
#
# Tool selection: llvm-dwarfdump handles all 3 PE arches uniformly. System
# objdump only handles x86 — it returns "file format not recognized" for arm64
# PE-COFF. We prefer llvm-dwarfdump when available, fall back to objdump for
# x86 only.
DWARF_DUMP="$(find_llvm_dwarfdump || true)"

count_dwarf_tags() {
    local file="$1" arch="$2"
    # NB: `grep -c` already prints the count (0 on no match) AND exits 1 when the
    # count is 0 — a trailing `|| echo 0` therefore printed a SECOND "0", so the
    # caller saw "0\n0" and `[ "$count" -ge N ]` died with "integer expression
    # expected". `|| true` swallows grep's exit-1 and keeps its single-line count.
    if [ -n "$DWARF_DUMP" ]; then
        "$DWARF_DUMP" --debug-info "$file" 2>/dev/null \
            | grep -cE 'DW_TAG_variable|DW_TAG_subprogram' || true
    elif [ "$arch" != "arm64" ]; then
        objdump --dwarf=info "$file" 2>/dev/null \
            | grep -cE 'DW_TAG_variable|DW_TAG_subprogram' || true
    else
        echo "0"   # no DWARF tool available for the arm64 PE
    fi
}

# Per-axis DWARF diagnosis. Only called when the linked-binary check fails.
# Returns 0/1 as "C objects are rich" / "Rust objects are rich". Side effect:
# emits fix-it for whichever axis is thin.
diagnose_dwarf_axes() {
    local arch="$1"

    # Sample one C .obj file from CMakeFiles/ntoskrnl.dir.
    local c_obj
    c_obj="$(find "$(arch_build_dir "$arch")/ntoskrnl/CMakeFiles/ntoskrnl.dir" \
        -name '*.c.obj' -print -quit 2>/dev/null)"
    local c_tags=0
    [ -n "$c_obj" ] && c_tags=$(count_dwarf_tags "$c_obj" "$arch")

    # Sample one rcgu.o from inside the rlib by extracting to a temp dir.
    local rust_dir
    rust_dir="$(arch_rust_target_dir "$arch")"
    local rlib
    rlib="$(find "$rust_dir/deps" -name 'libntoskrnl_rust-*.rlib' -print -quit 2>/dev/null)"
    local r_tags=0
    if [ -n "$rlib" ]; then
        local tmp
        tmp="$(mktemp -d)"
        ( cd "$tmp" && ar x "$rlib" 2>/dev/null )
        local rcgu
        rcgu="$(find "$tmp" -name '*.rcgu.o' -print -quit 2>/dev/null)"
        [ -n "$rcgu" ] && r_tags=$(count_dwarf_tags "$rcgu" "$arch")
        rm -rf "$tmp"
    fi

    log_info "[$arch] dwarf axes: C=$c_tags Rust=$r_tags (sample objs)"

    if [ "$c_tags" -lt 1 ]; then
        record_fail "$arch" "DWARF (C axis)" \
            "rebuild C with debug info — set CMAKE_C_FLAGS_RELEASE='-O2 -g3' in $(arch_build_dir "$arch")/ via 'cmake -DCMAKE_C_FLAGS_RELEASE=\"-O2 -g3\" .' then 'ninja ntoskrnl'"
    fi
    if [ "$r_tags" -lt 1 ]; then
        record_fail "$arch" "DWARF (Rust axis)" \
            "rebuild Rust with debug info — set [profile.release] debug = true in sdk/lib/rust/Cargo.toml or export RUSTFLAGS='-C debuginfo=2' before building"
    fi
}

check_dwarf_richness() {
    local arch="$1"
    local img
    img="$(arch_kernel_image "$arch")"
    [ -f "$img" ] || return 1   # already reported by check_kernel_built

    local count
    count=$(count_dwarf_tags "$img" "$arch")
    if [ "$count" -ge "$DWARF_RICHNESS_THRESHOLD" ]; then
        record_pass "$arch" "DWARF richness ($count tags)"
        return 0
    fi
    record_fail "$arch" "DWARF richness" \
        "linked ntoskrnl.exe has only $count DW_TAGs (threshold $DWARF_RICHNESS_THRESHOLD); see per-axis diagnosis below"
    diagnose_dwarf_axes "$arch"
    return 1
}

check_kd_debugger_data_block() {
    local arch="$1"
    local img
    img="$(arch_kernel_image "$arch")"
    [ -f "$img" ] || return 1

    # We don't need the value, just that the symbol exists. Use the per-arch
    # nm from llvm-mingw — system objdump can't read arm64 PE-COFF.
    # i386 PE adds a leading underscore (`_KdDebuggerDataBlock`); amd64/arm64
    # do not. The regex below matches both.
    local nm
    nm="$(arch_nm "$arch")"
    if [ ! -x "$nm" ]; then
        record_fail "$arch" "KdDebuggerDataBlock symbol" \
            "missing $nm — re-extract llvm-mingw into .build-tools/"
        return 1
    fi
    if "$nm" "$img" 2>/dev/null | grep -qE '(^| )_?KdDebuggerDataBlock( |$)'; then
        record_pass "$arch" "KdDebuggerDataBlock symbol"
        return 0
    fi
    record_fail "$arch" "KdDebuggerDataBlock symbol" \
        "symbol missing from $img — confirm KD64 is enabled in the build (it should be by default)"
    return 1
}

# --- main -------------------------------------------------------------------
declare -a ARCHES
if [ -n "$SINGLE_ARCH" ]; then
    arch_validate "$SINGLE_ARCH"
    ARCHES=("$SINGLE_ARCH")
else
    ARCHES=("${SUPPORTED_ARCHES[@]}")
fi

# Host-level checks, run once.
check_gdb || true
check_dwarf_tool || true

# Per-arch checks.
for arch in "${ARCHES[@]}"; do
    check_qemu "$arch" || true
    if check_kernel_built "$arch"; then
        check_dwarf_richness "$arch" || true
        check_kd_debugger_data_block "$arch" || true
    fi
done

# Summary.
if [ "$JSON_MODE" -eq 1 ]; then
    # Build {"failures":[{"axis":"<arch>","check":"...","fix":"..."}, ...]}
    {
        printf '{"failures":['
        local_first=1
        for f in "${FAILURES[@]:-}"; do
            [ -z "$f" ] && continue
            arch="${f%%$'\t'*}"
            rest="${f#*$'\t'}"
            check="${rest%%$'\t'*}"
            fix="${rest#*$'\t'}"
            if [ "$local_first" -eq 1 ]; then
                local_first=0
            else
                printf ','
            fi
            vjson obj -c --arg axis "$arch" --arg check "$check" --arg fix "$fix"
        done
        printf ']}\n'
    }
fi

if [ "${#FAILURES[@]}" -gt 0 ]; then
    [ "$JSON_MODE" -eq 1 ] || log_error "preflight: ${#FAILURES[@]} failure(s)"
    exit 1
fi

[ "$JSON_MODE" -eq 1 ] || log_info "preflight: all checks passed"
exit 0
