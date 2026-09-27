# who-holds <KTHREAD>: for a thread blocked in WrResource, name the ERESOURCE it waits on,
# the thread that owns it, and the code addresses on the owner's kernel stack.
#
# The 2026-09-27 amd64 appstorm hang took four hand-written gdb scripts to answer "why is
# explorer's folder window stuck in NtCreateFile": its thread waited on one event, the
# ERESOURCE whose ExclusiveWaiters that event is was found by walking
# ExpSystemResourcesList, the owner was a wordpad thread, and the owner's stack said it was
# spinning in FsRtlUninitializeFileLock inside fastfat's close. This is that recipe as a
# verb. Usage: list-threads (find the WrResource thread's ETHREAD), then who-holds <addr>.
#
# Offsets: ERESOURCE / LDR / KWAIT_BLOCK / KTHREAD.KernelStack come from ntkrnl-sys's
# compiled layout asserts (list-threads.py's _lt_checked_offsets); KTHREAD.WaitBlockList
# has no assert and comes from dump-thread.py's per-arch table.

import gdb


def _wh_ptr_size() -> int:
    return _lt_ptr_size()  # noqa: F821 -- list-threads.py, sourced earlier


def _wh_read(addr: int, size: int) -> int:
    return int.from_bytes(bytes(gdb.selected_inferior().read_memory(addr, size)), "little")


def _wh_ptr(addr: int) -> int:
    return _wh_read(addr, _wh_ptr_size())


def _wh_off(name: str) -> int:
    off = _lt_checked_offsets().get(name)  # noqa: F821
    if off is None:
        raise gdb.GdbError(f"who-holds: no compiled offset for {name}")
    return off


def _wh_waitblocklist_off() -> int:
    table = {"amd64": "KT_OFFSETS_AMD64", "i386": "KT_OFFSETS_I386", "arm64": "KT_OFFSETS_ARM64"}
    rows = globals().get(table.get(_lt_arch(), ""), [])  # noqa: F821
    for row in rows:
        if isinstance(row, tuple) and row[0] == "WaitBlockList":
            return row[1]
    raise gdb.GdbError(f"who-holds: dump-thread.py has no WaitBlockList offset for {_lt_arch()}")  # noqa: F821


def _wh_modules() -> list[tuple[int, int, str]]:
    head = int(gdb.parse_and_eval("(unsigned long long)&PsLoadedModuleList"))
    base_off = _wh_off("_LDR_DATA_TABLE_ENTRY.DllBase")
    size_off = _wh_off("_LDR_DATA_TABLE_ENTRY.SizeOfImage")
    name_off = _wh_off("_LDR_DATA_TABLE_ENTRY.BaseDllName")
    ps = _wh_ptr_size()
    mods, cur = [], _wh_ptr(head)
    while cur != head and len(mods) < 512:
        length, buf = _wh_read(cur + name_off, 2), _wh_ptr(cur + name_off + ps)
        try:
            name = bytes(gdb.selected_inferior().read_memory(buf, length)).decode("utf-16-le")
        except (gdb.MemoryError, UnicodeDecodeError):
            name = "?"
        mods.append((_wh_ptr(cur + base_off), _wh_read(cur + size_off, 4), name))
        cur = _wh_ptr(cur)
    return mods


def _wh_find_resource(obj: int) -> int | None:
    head = int(gdb.parse_and_eval("(unsigned long long)&ExpSystemResourcesList"))
    sw, ew = _wh_off("_ERESOURCE.SharedWaiters"), _wh_off("_ERESOURCE.ExclusiveWaiters")
    cur, n = _wh_ptr(head), 0
    while cur != head and n < 100000:
        # SystemResourcesList is the ERESOURCE's first field, so the link IS the resource.
        if obj in (_wh_ptr(cur + sw), _wh_ptr(cur + ew)):
            return cur
        cur, n = _wh_ptr(cur), n + 1
    return None


class WhoHoldsCmd(gdb.Command):
    """who-holds <KTHREAD>: the ERESOURCE a WrResource thread waits on, and its owner."""

    def __init__(self):
        super().__init__("who-holds", gdb.COMMAND_USER)

    def invoke(self, arg: str, from_tty: bool) -> None:
        if not arg.strip():
            raise gdb.GdbError("usage: who-holds <KTHREAD address>")
        thread = int(gdb.parse_and_eval(arg.strip()))
        ps = _wh_ptr_size()
        block = _wh_ptr(thread + _wh_waitblocklist_off())
        obj = _wh_ptr(block + _wh_off("_KWAIT_BLOCK.Object"))
        # WaitReason and WaitBlockList survive the wake: a thread that was signalled, or that
        # has TERMINATED, still names its last object. It is waiting only while its wait block
        # is linked on the object's DISPATCHER_HEADER.WaitListHead. Without this check an
        # exited hello32.exe read as "a lost wakeup" on a resource nobody held.
        head = obj + _wh_off("_DISPATCHER_HEADER.WaitListHead")
        entry = block + _wh_off("_KWAIT_BLOCK.WaitListEntry")
        cur, linked = _wh_ptr(head), False
        for _ in range(256):
            if cur in (head, 0):
                break
            if cur == entry:
                linked = True
                break
            cur = _wh_ptr(cur)
        gdb.write(
            f"thread {thread:#x} last waited on object {obj:#x}: "
            f"{'STILL BLOCKED (on its wait list)' if linked else 'not on its wait list'}\n"
        )
        if not linked:
            gdb.write("  stale wait block: the thread woke or terminated; nothing to chase here\n")
            return
        res = _wh_find_resource(obj)
        if res is None:
            gdb.write(
                "  not an ERESOURCE waiter event (no resource on ExpSystemResourcesList "
                "names it); try identify-object\n"
            )
            return
        owner = _wh_ptr(res + _wh_off("_ERESOURCE.OwnerEntry"))
        count = _wh_ptr(res + _wh_off("_ERESOURCE.OwnerEntry") + ps)
        active = _wh_read(res + _wh_off("_ERESOURCE.ActiveCount"), 2)
        flag = _wh_read(res + _wh_off("_ERESOURCE.Flag"), 2)
        mods = _wh_modules()

        def where(v: int) -> str | None:
            for base, size, name in mods:
                if base <= v < base + size:
                    return f"{name}+{v - base:#x}"
            return None

        gdb.write(
            f"ERESOURCE {res:#x} ({where(res) or 'pool'}): ActiveCount={active} "
            f"Flag={flag:#x} OwnerThread={owner:#x} OwnerCount={count:#x}\n"
        )
        if not owner or active == 0:
            gdb.write(
                "  NO OWNER while a thread waits: a lost wakeup (release did not "
                "signal the waiter) or a stale waiter count\n"
            )
            return
        stack = _wh_ptr(owner + _wh_off("_KTHREAD.KernelStack"))
        top = _wh_ptr(owner + _wh_off("_KTHREAD.InitialStack"))
        gdb.write(
            f"owner {owner:#x} kernel stack {stack:#x}..{top:#x}, code addresses "
            "(symbolize with llvm-symbolizer against the named module):\n"
        )
        for addr in range(stack, min(top, stack + 0x3000), ps):
            try:
                v = _wh_ptr(addr)
            except gdb.MemoryError:
                break
            m = where(v)
            if m and not m.endswith("+0x0"):
                gdb.write(f"  [{addr - stack:#06x}] {v:#x} {m}\n")


WhoHoldsCmd()
