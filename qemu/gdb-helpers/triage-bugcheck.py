# triage-bugcheck: decode KeBugCheckEx args, then drill into the trap frame
# (if applicable) and identify the kernel pointer that faulted.
#
# Headline command for the recurring "kernel BUGCHECKs in some flake" loop:
# one call gives you stop code, exception code, fault PC, fault VA, and —
# when applicable — what the bogus pointer in the asm operand actually was
# (process? thread? freed slab?).
#
# Currently tuned for amd64 KMODE_EXCEPTION_NOT_HANDLED (0x1E) where Param3
# is a KTRAP_FRAME pointer. Other variants fall back to printing the bug
# args and stop short.

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
}


def _arch() -> str:
    return os.environ.get("VIBEOS_ARCH", "amd64")


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

    try:
        return [int(gdb.parse_and_eval(e)) & 0xFFFFFFFFFFFFFFFF for e in exprs]
    except gdb.error:
        return None


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
        if arch != "amd64":
            gdb.write(f"triage-bugcheck: only amd64 supported (got '{arch}')\n",
                      gdb.STDERR)
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

        # For amd64 #PF in KiUnwaitThread-style sites, Rcx holds the pointer
        # the asm was indirecting through. Pull it out and identify it.
        try:
            rcx = int(gdb.parse_and_eval(
                f"*(unsigned long long *)({trap_frame:#x} + 0x38)"
            )) & 0xFFFFFFFFFFFFFFFF
        except gdb.error:
            return
        if rcx == 0:
            return

        gdb.write(f"\n--- identify-object on Rcx @ {rcx:#018x} ---\n")
        try:
            gdb.execute(f"identify-object {rcx:#x}")
        except gdb.error as e:
            gdb.write(f"  (identify-object failed: {e})\n")


TriageBugcheckCmd()
