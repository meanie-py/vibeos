# walk-waitlist <object-addr>: walk an object's Header.WaitListHead,
# pretty-printing every wait block (Thread, Object, NextWaitBlock,
# WaitType/WaitKey/BlockState).
#
# Use case: a dispatcher object got signaled but the wakeup chain blew up
# — you want to see what's actually queued on its wait list. Especially
# useful when investigating wait-block field corruption (e.g. the recurring
# "Thread field holds a Process pointer" bug).
#
# Layout assumes the WIN2003-era KWAIT_BLOCK (Thread @ +0x10, Object @ +0x18)
# matching ReactOS targets. KWAIT_BLOCK is layout-flipped on WIN8+ and the
# helper would need updating if NTDDI_VERSION is bumped past 0x0602.

import gdb
import os


# Per-arch offsets within KWAIT_BLOCK (legacy/WS03 layout).
WB_LAYOUT = {
    "amd64": {
        "ptr_size":          8,
        "WaitListEntry":     0x00,
        "Thread":            0x10,
        "Object":            0x18,
        "NextWaitBlock":     0x20,
        "WaitKey":           0x28,
        "WaitType":          0x2a,
        "BlockState":        0x2b,
    },
    "arm64": {
        "ptr_size":          8,
        "WaitListEntry":     0x00,
        "Thread":            0x10,
        "Object":            0x18,
        "NextWaitBlock":     0x20,
        "WaitKey":           0x28,
        "WaitType":          0x2a,
        "BlockState":        0x2b,
    },
    "i386": {
        "ptr_size":          4,
        "WaitListEntry":     0x00,
        "Thread":            0x08,
        "Object":            0x0c,
        "NextWaitBlock":     0x10,
        "WaitKey":           0x14,
        "WaitType":          0x16,
        "BlockState":        0x17,
    },
}


# Where Header.WaitListHead lives within DISPATCHER_HEADER. Same on all our
# arches: at the start of the body, +0x08 from the dispatcher header start.
WAITLISTHEAD_OFFSET = 0x08

WAIT_TYPES = {
    0: "WaitAny",
    1: "WaitAll",
}


def _arch() -> str:
    return os.environ.get("VIBEOS_ARCH", "amd64")


def _ww_read(addr: int, size: int) -> int | None:
    ctype = {1: "unsigned char", 2: "unsigned short",
             4: "unsigned int", 8: "unsigned long long"}[size]
    try:
        v = gdb.parse_and_eval(f"*({ctype} *)({addr:#x})")
        return int(v) & ((1 << (size * 8)) - 1)
    except gdb.error:
        return None


class WalkWaitListCmd(gdb.Command):
    """walk-waitlist <object-addr>: walk Header.WaitListHead of a dispatcher object."""

    MAX_ITERATIONS = 32  # safety cap — in case the list is corrupted into a longer cycle

    def __init__(self):
        super().__init__("walk-waitlist", gdb.COMMAND_USER)

    def invoke(self, arg: str, from_tty: bool) -> None:
        a = arg.strip()
        if not a:
            gdb.write("walk-waitlist: usage: walk-waitlist <object-addr>\n", gdb.STDERR)
            return
        try:
            obj = int(gdb.parse_and_eval(a)) & 0xFFFFFFFFFFFFFFFF
        except gdb.error as e:
            gdb.write(f"walk-waitlist: bad address: {e}\n", gdb.STDERR)
            return

        arch = _arch()
        if arch not in WB_LAYOUT:
            gdb.write(f"walk-waitlist: unsupported arch '{arch}'\n", gdb.STDERR)
            return
        layout = WB_LAYOUT[arch]
        psize = layout["ptr_size"]

        head = obj + WAITLISTHEAD_OFFSET
        flink = _ww_read(head, psize)
        blink = _ww_read(head + psize, psize)
        if flink is None or blink is None:
            gdb.write(f"walk-waitlist: WaitListHead unreadable at {head:#018x}\n",
                      gdb.STDERR)
            return

        # Header.Type for context
        type_byte = _ww_read(obj, 1)
        sig_state = _ww_read(obj + 0x04, 4) or 0
        gdb.write(
            f"Object @ {obj:#018x}  Header.Type={type_byte:#04x}  "
            f"SignalState={sig_state:#x}\n"
        )
        gdb.write(
            f"WaitListHead @ {head:#018x}  Flink={flink:#018x}  Blink={blink:#018x}\n"
        )

        if flink == head:
            gdb.write("  (empty list)\n")
            return

        seen = set()
        i = 0
        cur = flink
        while cur != head and i < self.MAX_ITERATIONS:
            if cur in seen:
                gdb.write(f"  *** CYCLE detected at {cur:#018x} — aborting walk\n")
                return
            seen.add(cur)

            wb = cur - layout["WaitListEntry"]
            thread = _ww_read(wb + layout["Thread"], psize)
            obj_field = _ww_read(wb + layout["Object"], psize)
            nxt = _ww_read(wb + layout["NextWaitBlock"], psize)
            key = _ww_read(wb + layout["WaitKey"], 2) or 0
            wtype = _ww_read(wb + layout["WaitType"], 1) or 0
            bstate = _ww_read(wb + layout["BlockState"], 1) or 0

            wt_name = WAIT_TYPES.get(wtype, f"({wtype:#x})")
            gdb.write(
                f"  [{i:02d}] WaitBlock @ {wb:#018x}\n"
                f"        Thread={thread:#018x}  Object={obj_field:#018x}\n"
                f"        NextWaitBlock={nxt:#018x}  "
                f"WaitKey={key:#x}  WaitType={wt_name}  BlockState={bstate:#x}\n"
            )

            # Sanity check: tag whether Thread looks like a Thread or not
            t_type = _ww_read(thread, 1) if thread else None
            if t_type is not None and t_type != 0x06:
                gdb.write(
                    f"        ** WARNING: Thread@{thread:#x}.Type={t_type:#x} "
                    f"(expected 0x06 ThreadObject)\n"
                )

            i += 1
            next_flink = _ww_read(cur, psize)
            if next_flink is None:
                gdb.write(f"  (next Flink unreadable at {cur:#018x})\n")
                return
            cur = next_flink

        if i == self.MAX_ITERATIONS:
            gdb.write(f"  (stopped after {self.MAX_ITERATIONS} iterations — possibly long or corrupted)\n")


WalkWaitListCmd()
