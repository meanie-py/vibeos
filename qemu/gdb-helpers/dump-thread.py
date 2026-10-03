# dump-thread [addr]: pretty-print a _KTHREAD with named fields.
#
# Defaults to the current thread (KPCR.Prcb.CurrentThread on amd64/i386,
# similar on arm64). With an explicit address, dumps that pool block as a
# KTHREAD — even if the address is invalid, the layout is shown so the agent
# can see *whether* it looks like a real KTHREAD or random garbage.
#
# Specifically tuned for the recurring KiUnwaitThread BUGCHECK on amd64,
# where Thread->WaitBlockList is dereferenced; we surface WaitStatus,
# WaitBlockList, WaitBlock[0], State, and ApcState.Process.
#
# Two access modes:
#   1. DWARF-typed: cast `(struct _KTHREAD *)addr` and walk named fields.
#   2. Raw-offset fallback: if the cast fails (e.g. bindgen DWO CU missing
#      from ntoskrnl.exe), read raw memory at hardcoded offsets per arch.

import gdb
import os


# Per-arch KTHREAD field offsets — the subset we surface in this view.
# Keys mirror the field names in DWARF mode so the output is comparable.
# Updated from sdk/lib/rust/ntkrnl-sys/src/bindgen_<arch>.rs ; if KTHREAD
# layout changes (new bitfield, struct rearrangement) bump these too.
KT_OFFSETS_AMD64 = [
    ("Header.Type",                 0x000, 1, "u"),
    ("Header.SignalState",          0x004, 4, "i"),
    ("Header.WaitListHead.Flink",   0x008, 8, "p"),
    ("Header.WaitListHead.Blink",   0x010, 8, "p"),
    ("KernelStack",                 0x038, 8, "p"),
    ("ApcState.Process",            0x068, 8, "p"),
    ("WaitStatus",                  0x080, 8, "x"),
    ("WaitBlockList",               0x088, 8, "p"),
    ("WaitBlock[0].WaitListEntry.Flink", 0x0f8, 8, "p"),
    ("WaitBlock[0].WaitListEntry.Blink", 0x100, 8, "p"),
    ("WaitBlock[0].Thread",         0x108, 8, "p"),
    ("WaitBlock[0].Object",         0x110, 8, "p"),
    ("WaitBlock[0].NextWaitBlock",  0x118, 8, "p"),
    ("State",                       0x164, 1, "u"),  # observed via asm; verify
    ("Process",                     0x210, 8, "p"),  # KTHREAD.Process (different from ApcState.Process)
]

# Conservative i386 / arm64 sets — if you start using these, validate offsets
# against the live kernel before trusting them.
KT_OFFSETS_I386 = [
    ("Header.Type",                 0x000, 1, "u"),
    ("Header.SignalState",          0x004, 4, "i"),
    ("Header.WaitListHead.Flink",   0x008, 4, "p"),
    ("Header.WaitListHead.Blink",   0x00c, 4, "p"),
    ("InitialStack",                0x018, 4, "p"),
    ("StackLimit",                  0x01c, 4, "p"),
    ("KernelStack",                 0x020, 4, "p"),
    ("ApcState.Process",            0x038, 4, "p"),
    ("State",                       0x04c, 1, "u"),
    ("WaitMode",                    0x04f, 1, "u"),
    ("WaitStatus",                  0x050, 4, "x"),
    ("WaitBlockList",               0x054, 4, "p"),
    ("WaitReason",                  0x05a, 1, "u"),
    ("Priority",                    0x05b, 1, "u"),
    ("Teb",                         0x074, 4, "p"),
    ("WaitBlock[0].WaitListEntry.Flink", 0x0a8, 4, "p"),
    ("WaitBlock[0].WaitListEntry.Blink", 0x0ac, 4, "p"),
    ("WaitBlock[0].Thread",         0x0b0, 4, "p"),
    ("WaitBlock[0].Object",         0x0b4, 4, "p"),
    ("WaitBlock[0].NextWaitBlock",  0x0b8, 4, "p"),
    ("TrapFrame",                   0x110, 4, "p"),
    ("CallbackStack",               0x114, 4, "p"),
    ("Process",                     0x128, 4, "p"),
]

# arm64's KTHREAD is amd64's up to 0x2f0: ntkrnl-sys's compiled asserts
# (offset_check_{amd64,arm64}.rs) agree on every field below that, and differ only in the
# I/O counters and StateSaveArea past it. The header-only table this replaced left
# who-holds unable to name an arm64 thread's wait block.
KT_OFFSETS_ARM64 = KT_OFFSETS_AMD64


KT_SCHEMA = {
    "amd64": KT_OFFSETS_AMD64,
    "i386":  KT_OFFSETS_I386,
    "arm64": KT_OFFSETS_ARM64,
}


# Fields we care about for the wait-list debugging cases. Plus a few generic
# state fields. Listed in display order.  Used only in DWARF mode.
THREAD_FIELDS = [
    "Header.Type",
    "Header.SignalState",
    "ApcState.Process",
    "WaitStatus",
    "WaitBlockList",
    "WaitBlock[0].Object",
    "WaitBlock[0].Thread",
    "WaitBlock[0].WaitListEntry.Flink",
    "WaitBlock[0].WaitType",
    "WaitBlock[0].BlockState",
    "State",
    "WaitMode",
    "WaitReason",
    "Process",
]


def _arch() -> str:
    return os.environ.get("VIBEOS_ARCH", "amd64")


def _dt_read(addr: int, size: int, kind: str) -> int | None:
    ctype = {1: "unsigned char", 2: "unsigned short",
             4: "unsigned int", 8: "unsigned long long"}[size]
    if kind == "i":
        ctype = ctype.replace("unsigned ", "")
    try:
        v = gdb.parse_and_eval(f"*({ctype} *)({addr:#x})")
        return int(v)
    except gdb.error:
        return None


def _dt_format(val: int | None, size: int, kind: str) -> str:
    if val is None:
        return "<unreadable>"
    width = size * 2
    if kind == "p":
        return f"{val & ((1 << (size * 8)) - 1):#0{width + 2}x}"
    if kind == "x":
        return f"{val & ((1 << (size * 8)) - 1):#0{width + 2}x}"
    if kind == "u":
        return f"{val & ((1 << (size * 8)) - 1):#x}"
    return str(val)


def _dump_dwarf(addr_expr: str) -> bool:
    """Try DWARF-typed dump. Returns False if the cast itself failed."""
    cast = f"(struct _KTHREAD *)({addr_expr})"
    try:
        kt = gdb.parse_and_eval(cast)
    except gdb.error:
        return False

    addr = int(kt) & 0xFFFFFFFFFFFFFFFF

    # The cast can succeed but field accesses fail when the bindgen DWO CU
    # is missing — probe one access first; if it fails, signal raw-fallback
    # without printing a misleading header line.
    try:
        _ = gdb.parse_and_eval(f"{cast}->Header.Type")
    except gdb.error:
        return False

    gdb.write(f"_KTHREAD @ {addr:#018x}  (DWARF-typed)\n")
    for f in THREAD_FIELDS:
        try:
            val = gdb.parse_and_eval(f"{cast}->{f}")
            gdb.write(f"  {f:<35} {val}\n")
        except gdb.error as e:
            gdb.write(f"  {f:<35} <unreadable: {e}>\n")
    return True


def _dump_raw(addr: int) -> None:
    arch = _arch()
    schema = KT_SCHEMA.get(arch, [])
    if not schema:
        gdb.write(f"dump-thread: no raw-offset schema for arch '{arch}'\n", gdb.STDERR)
        return

    gdb.write(f"_KTHREAD @ {addr:#018x}  (raw-offset, arch={arch})\n")
    for name, off, size, kind in schema:
        val = _dt_read(addr + off, size, kind)
        gdb.write(f"  +{off:#05x}  {name:<35} {_dt_format(val, size, kind)}\n")


class DumpThreadCmd(gdb.Command):
    """dump-thread [addr]: print _KTHREAD at addr (default: current thread)."""

    def __init__(self):
        super().__init__("dump-thread", gdb.COMMAND_USER)

    def invoke(self, arg: str, from_tty: bool) -> None:
        a = arg.strip()
        if not a:
            # Best-effort current thread: try common per-arch lookups.
            for expr in (
                "KeGetCurrentThread()",
                "((struct _KPCR *)$gs_base)->Prcb.CurrentThread",
                "((struct _KPCR *)$fs_base)->PrcbData.CurrentThread",
            ):
                try:
                    a = str(int(gdb.parse_and_eval(expr)))
                    break
                except gdb.error:
                    continue
            if not a:
                gdb.write("dump-thread: could not resolve current thread; pass an address\n",
                          gdb.STDERR)
                return

        try:
            addr = int(gdb.parse_and_eval(a)) & 0xFFFFFFFFFFFFFFFF
        except gdb.error as e:
            gdb.write(f"dump-thread: bad address: {e}\n", gdb.STDERR)
            return

        if not _dump_dwarf(str(addr)):
            _dump_raw(addr)


DumpThreadCmd()
