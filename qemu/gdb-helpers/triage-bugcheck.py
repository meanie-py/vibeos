# triage-bugcheck: decode KeBugCheckEx args, then drill into the trap frame
# (if applicable) and identify the kernel pointer that faulted.
#
# Headline command for the recurring "kernel BUGCHECKs in some flake" loop:
# one call gives you stop code, exception code, fault PC, fault VA, and —
# when applicable — what the bogus pointer in the asm operand actually was
# (process? thread? freed slab?).
#
# Tuned for KMODE_EXCEPTION_NOT_HANDLED (0x1E), where this tree raises with
# Param2 = fault PC and Param3 = KTRAP_FRAME pointer on ALL three arches
# (amd64 ke/amd64, i386 ke/i386/exp.rs, arm64 ke/arm64/stubs.rs). Other stop
# codes fall back to printing the bug args and stop short.

import gdb
import os


BUGCHECK_NAMES = {
    0x00000001: "APC_INDEX_MISMATCH",
    0x0000001E: "KMODE_EXCEPTION_NOT_HANDLED",
    0x0000003B: "SYSTEM_SERVICE_EXCEPTION",
    0x0000004A: "IRQL_GT_ZERO_AT_SYSTEM_SERVICE",
    0x0000004E: "PFN_LIST_CORRUPT",
    0x00000050: "PAGE_FAULT_IN_NONPAGED_AREA",
    0x0000007E: "SYSTEM_THREAD_EXCEPTION_NOT_HANDLED",
    0x0000007F: "UNEXPECTED_KERNEL_MODE_TRAP",
    0x000000C2: "BAD_POOL_CALLER",
    0x000000C5: "DRIVER_CORRUPTED_EXPOOL",
    0x000000D1: "DRIVER_IRQL_NOT_LESS_OR_EQUAL",
    # Synthetic codes used by in-tree assertions:
    0xBADBED1E: "WAIT_BLOCK_TYPE_CONFUSION (synthetic, KiUnwaitThread entry)",
}


def _arch() -> str:
    return os.environ.get("VIBEOS_ARCH", "amd64")


# Per-arch "what pointer did the faulting instruction touch" — read from the
# KTRAP_FRAME after we've located it. amd64/i386 surface the C-call first arg
# register (Rcx/Ecx) that asm-heavy sites indirect through; arm64 has no such
# convention, so we use the hardware FaultAddress (the data-abort VA) directly.
# (trapframe-relative offset, label)
BAD_PTR = {
    "amd64": (0x38, "Rcx"),
    "i386":  (0x40, "Ecx"),
    "arm64": (0x10, "FaultAddress"),
}


def _bugcheck_args() -> list[int] | None:
    arch = _arch()
    if arch == "amd64":
        exprs = ["$rcx", "$rdx", "$r8", "$r9",
                 "*(unsigned long long *)($rsp+0x28)"]
    elif arch == "arm64":
        exprs = ["$x0", "$x1", "$x2", "$x3", "$x4"]
    elif arch == "i386":
        exprs = [f"*(unsigned int *)($esp+{i*4})" for i in range(1, 6)]
    else:
        return None

    # Force C language: some exprs are C casts (`*(unsigned long long *)($rsp+
    # 0x28)`), which fail when gdb is stopped inside a Rust frame (Rust language
    # mode rejects C type names). Register convenience vars work in any language.
    gdb.execute("set language c", to_string=True)
    try:
        return [int(gdb.parse_and_eval(e)) & 0xFFFFFFFFFFFFFFFF for e in exprs]
    except gdb.error:
        return None
    finally:
        gdb.execute("set language auto", to_string=True)


def _find_bugcheck_frame() -> bool:
    """Walk up to a frame whose function name contains 'BugCheck'. True on success."""
    f = gdb.newest_frame()
    seen = 0
    while f is not None and seen < 16:
        name = f.name() or ""
        if "BugCheck" in name:
            f.select()
            return True
        try:
            f = f.older()
        except gdb.error:
            break
        seen += 1
    return False


class TriageBugcheckCmd(gdb.Command):
    """triage-bugcheck: decode KeBugCheckEx, drill trap frame + bad-pointer."""

    def __init__(self):
        super().__init__("triage-bugcheck", gdb.COMMAND_USER)

    def invoke(self, arg: str, from_tty: bool) -> None:
        arch = _arch()
        if arch not in BAD_PTR:
            gdb.write(f"triage-bugcheck: unsupported arch '{arch}'\n", gdb.STDERR)
            return

        if not _find_bugcheck_frame():
            gdb.write("triage-bugcheck: not at a KeBugCheckEx frame\n", gdb.STDERR)
            return

        args = _bugcheck_args()
        if args is None:
            gdb.write("triage-bugcheck: failed to read bugcheck args\n", gdb.STDERR)
            return

        code = args[0] & 0xFFFFFFFF
        name = BUGCHECK_NAMES.get(code, "UNKNOWN")
        gdb.write(f"=== BUGCHECK {code:#010x} ({name}) ===\n")
        for i, v in enumerate(args[1:], 1):
            gdb.write(f"  Param{i}: {v:#018x}\n")

        # Only KMODE_EXCEPTION_NOT_HANDLED gets the trap-frame drilldown.
        if code != 0x0000001E:
            gdb.write("triage-bugcheck: stop code is not 0x1E; no further drill\n")
            return

        exc_code = args[1] & 0xFFFFFFFF
        fault_pc = args[2]
        trap_frame = args[3]

        gdb.write(f"\nException: {exc_code:#010x}  ")
        if exc_code == 0xC0000005:
            gdb.write("(STATUS_ACCESS_VIOLATION)\n")
        elif exc_code == 0xC0000006:
            gdb.write("(STATUS_IN_PAGE_ERROR)\n")
        else:
            gdb.write("\n")
        gdb.write(f"Fault PC : {fault_pc:#018x}  ")
        try:
            sym = gdb.execute(f"info symbol {fault_pc:#x}", to_string=True).strip()
            gdb.write(f"({sym})\n")
        except gdb.error:
            gdb.write("\n")

        if trap_frame == 0:
            gdb.write("\nNo trap frame (Param3 == 0)\n")
            return

        gdb.write(f"\n--- KTRAP_FRAME @ {trap_frame:#018x} ---\n")
        try:
            gdb.execute(f"dump-trapframe {trap_frame:#x}")
        except gdb.error as e:
            gdb.write(f"  (dump-trapframe failed: {e})\n")
            return

        # Pull the per-arch "bad pointer" out of the trap frame and identify it:
        # amd64/i386 use the first C-arg register (Rcx/Ecx) that asm-heavy #PF
        # sites indirect through; arm64 uses the hardware FaultAddress (data VA).
        ptr_off, ptr_label = BAD_PTR[arch]
        sz = 4 if arch == "i386" else 8
        try:
            # Language-agnostic raw read (see dump-trapframe: C casts break in
            # Rust frames). Trap frames are little-endian on our targets.
            buf = gdb.selected_inferior().read_memory(trap_frame + ptr_off, sz)
            bad = int.from_bytes(bytes(buf), "little") & 0xFFFFFFFFFFFFFFFF
        except (gdb.error, gdb.MemoryError):
            return
        if bad == 0:
            return

        gdb.write(f"\n--- identify-object on {ptr_label} @ {bad:#018x} ---\n")
        try:
            gdb.execute(f"identify-object {bad:#x}")
        except gdb.error as e:
            gdb.write(f"  (identify-object failed: {e})\n")


TriageBugcheckCmd()
