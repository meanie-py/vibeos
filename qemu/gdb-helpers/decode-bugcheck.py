# decode-bugcheck: pretty-print KeBugCheckEx arguments at the current frame.
#
# Assumes the gdbstub is halted at or near a KeBugCheckEx call. Walks up to
# 16 frames looking for one whose function name contains "BugCheck", then
# reads the 5 args (BugCheckCode + 4 parameters) from the right registers
# per ABI.
#
# Output format: a small structured block with the stop code, its symbolic
# name, and the parameters formatted as hex.

import gdb
import os


# Subset that's actually shown up in our smoke runs. Add more as we hit them.
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


def _arch_args(arch: str) -> list[str]:
    if arch == "amd64":
        return ["$rcx", "$rdx", "$r8",  "$r9",  "*(unsigned long long *)($rsp+0x28)"]
    if arch == "arm64":
        return ["$x0",  "$x1",  "$x2",  "$x3",  "$x4"]
    if arch == "i386":
        return [
            "*(unsigned int *)($esp+4)",
            "*(unsigned int *)($esp+8)",
            "*(unsigned int *)($esp+12)",
            "*(unsigned int *)($esp+16)",
            "*(unsigned int *)($esp+20)",
        ]
    raise gdb.error(f"unknown arch: {arch}")


def _read_unsigned(expr: str) -> int:
    val = gdb.parse_and_eval(expr)
    return int(val) & 0xFFFFFFFFFFFFFFFF


def _find_bugcheck_frame() -> gdb.Frame | None:
    f = gdb.newest_frame()
    seen = 0
    while f is not None and seen < 16:
        name = f.name() or ""
        if "BugCheck" in name:
            return f
        try:
            f = f.older()
        except gdb.error:
            break
        seen += 1
    return None


class DecodeBugcheckCmd(gdb.Command):
    """decode-bugcheck: read KeBugCheckEx args at the current frame."""

    def __init__(self):
        super().__init__("decode-bugcheck", gdb.COMMAND_USER)

    def invoke(self, arg: str, from_tty: bool) -> None:
        arch = os.environ.get("VIBEOS_ARCH", "")
        if arch not in ("amd64", "i386", "arm64"):
            gdb.write(f"decode-bugcheck: unknown arch '{arch}'\n", gdb.STDERR)
            return

        bf = _find_bugcheck_frame()
        if bf is not None:
            bf.select()
            gdb.write(f"decode-bugcheck: frame = {bf.name()}\n")

        # Force C language: amd64/i386 arg exprs are C casts (`*(unsigned ... *)
        # ($rsp+0x28)`) which fail when stopped inside a Rust frame (Rust mode
        # rejects C type names). Restored to auto afterward.
        gdb.execute("set language c", to_string=True)
        try:
            arg_exprs = _arch_args(arch)
            vals = [_read_unsigned(e) for e in arg_exprs]
        except gdb.error as e:
            gdb.write(f"decode-bugcheck: failed to read args: {e}\n", gdb.STDERR)
            return
        finally:
            gdb.execute("set language auto", to_string=True)

        code = vals[0] & 0xFFFFFFFF
        name = BUGCHECK_NAMES.get(code, "UNKNOWN")
        gdb.write(f"BugCheckCode: {code:#010x}  {name}\n")
        for i, v in enumerate(vals[1:], 1):
            gdb.write(f"  Param{i}: {v:#018x}\n")


DecodeBugcheckCmd()
