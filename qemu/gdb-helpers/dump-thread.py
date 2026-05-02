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

import gdb


# Fields we care about for the wait-list debugging cases. Plus a few generic
# state fields. Listed in display order.
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


def _dump_one(addr_expr: str) -> None:
    cast = f"(struct _KTHREAD *)({addr_expr})"
    try:
        kt = gdb.parse_and_eval(cast)
    except gdb.error as e:
        gdb.write(f"dump-thread: cast failed: {e}\n", gdb.STDERR)
        return

    addr = int(kt) & 0xFFFFFFFFFFFFFFFF
    gdb.write(f"_KTHREAD @ {addr:#018x}\n")
    for f in THREAD_FIELDS:
        try:
            val = gdb.parse_and_eval(f"{cast}->{f}")
            gdb.write(f"  {f:<35} {val}\n")
        except gdb.error as e:
            gdb.write(f"  {f:<35} <unreadable: {e}>\n")


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
        _dump_one(a)


DumpThreadCmd()
