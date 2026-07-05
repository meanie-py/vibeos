# walk-procs: offset-free EPROCESS list walk for the Rust kernel.
#
# The Rust-translated ntoskrnl emits no usable C struct DWARF (only stray
# anonymous sub-structs), so list-threads.py's `(struct _EPROCESS *)` casts
# fail.  This walker needs NO struct offsets: it follows the raw
# PsActiveProcessHead LIST_ENTRY chain (Flink is offset-free) and derives the
# ImageFileName offset empirically by scanning each node for a NUL-terminated
# printable name.  Runs at source-time; expects find-kbase to have relocated
# symbols already.
import gdb


def _u64(addr):
    try:
        b = gdb.selected_inferior().read_memory(addr, 8)
        return int.from_bytes(bytes(b), "little")
    except gdb.MemoryError:
        return None


def _scan_name(node_addr):
    # ImageFileName is CHAR[15] a few hundred bytes past ActiveProcessLinks on
    # amd64 (~0x5a8-ish across ROS revisions).  Scan a window for the longest
    # plausible NUL-terminated ASCII run; return (name, delta) or (None, None).
    lo, hi = 0x100, 0x700
    try:
        buf = bytes(gdb.selected_inferior().read_memory(node_addr + lo, hi - lo))
    except gdb.MemoryError:
        return None, None
    best = None
    i = 0
    while i < len(buf):
        if 0x20 <= buf[i] < 0x7F:
            j = i
            while j < len(buf) and 0x20 <= buf[j] < 0x7F:
                j += 1
            run = buf[i:j]
            # process-image names: 4..15 chars, NUL-terminated, has a dot or is a known bare name
            if 4 <= len(run) <= 15 and j < len(buf) and buf[j] == 0:
                s = run.decode("ascii", "replace")
                if (".exe" in s.lower()) or s in ("System", "Idle", "Registry", "MemCompression"):
                    if best is None or len(s) > len(best[0]):
                        best = (s, lo + i)
            i = j
        else:
            i += 1
    if best:
        return best[0], best[1]
    return None, None


def main():
    try:
        head = int(gdb.parse_and_eval("(unsigned long long)&PsActiveProcessHead"))
    except gdb.error as e:
        print(f"walk-procs: cannot resolve PsActiveProcessHead: {e}")
        return
    print(f"walk-procs: PsActiveProcessHead @ {head:#x}")
    first = _u64(head)
    if not first:
        print("walk-procs: empty/unreadable list head")
        return

    # Calibrate delta on the first few nodes until a name is found.
    delta = None
    cur = first
    for _ in range(6):
        nm, d = _scan_name(cur)
        if d is not None:
            delta = d
            print(f"walk-procs: calibrated ImageFileName delta = {delta:#x} (node {cur:#x} = {nm!r})")
            break
        cur = _u64(cur) or 0
        if not cur or cur == head:
            break
    if delta is None:
        print("walk-procs: could not calibrate ImageFileName offset")
        return

    # Walk and print every node's name via the calibrated delta.
    cur = first
    seen = set()
    n = 0
    while cur and cur != head and cur not in seen and n < 128:
        seen.add(cur)
        try:
            raw = bytes(gdb.selected_inferior().read_memory(cur + delta, 15))
            name = raw.split(b"\x00")[0].decode("ascii", "replace") or "<empty>"
        except gdb.MemoryError:
            name = "<unreadable>"
        eproc = cur - delta + 0  # node addr; EPROCESS base ~= cur - ActiveProcessLinks_off (unknown, informational)
        print(f"  [{n:2}] links@{cur:#x}  name={name}")
        cur = _u64(cur)
        n += 1
    print(f"walk-procs: {n} processes")


main()
