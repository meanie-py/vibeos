# list-threads: walk PsActiveProcessHead and print every (process, thread)
# pair with state, wait reason, image name, and addresses.
#
# Cooperates with find-kbase: kernel symbols (PsActiveProcessHead, struct
# layouts) are resolved via DWARF after `symbol-file ... -o <reloc>` ran.
#
# DWARF-only — no raw-offset fallback yet. If `(struct _EPROCESS *)` casts
# stop working, the bindgen DWO referenced in ntoskrnl.exe is missing.
# The DWARF path is robust enough for now because this kernel ships with
# rich struct info.

import gdb
import os
import re


I386_RUST_SYMBOL_SHIFT = 0x291000


# KTHREAD.State values (from ntdef.h KTHREAD_STATE enum).
KTHREAD_STATE = {
    0: "Initialized",
    1: "Ready",
    2: "Running",
    3: "Standby",
    4: "Terminated",
    5: "Waiting",
    6: "Transition",
    7: "DeferredReady",
    8: "GateWaitObsolete",
    9: "WaitingForProcessInSwap",
}


# KWAIT_REASON values — abbreviated for display.
KWAIT_REASON = {
    0: "Executive", 1: "FreePage", 2: "PageIn", 3: "PoolAllocation",
    4: "DelayExecution", 5: "Suspended", 6: "UserRequest", 7: "WrExecutive",
    8: "WrFreePage", 9: "WrPageIn", 10: "WrPoolAllocation", 11: "WrDelayExecution",
    12: "WrSuspended", 13: "WrUserRequest", 14: "WrEventPair", 15: "WrQueue",
    16: "WrLpcReceive", 17: "WrLpcReply", 18: "WrVirtualMemory", 19: "WrPageOut",
    20: "WrRendezvous", 21: "WrKeyedEvent", 22: "WrTerminated", 23: "WrProcessInSwap",
    24: "WrCpuRateControl", 25: "WrCalloutStack", 26: "WrKernel", 27: "WrResource",
    28: "WrPushLock", 29: "WrMutex", 30: "WrQuantumEnd", 31: "WrDispatchInt",
    32: "WrPreempted", 33: "WrYieldExecution", 34: "WrFastMutex", 35: "WrGuardedMutex",
    36: "WrRundown", 37: "MaximumWaitReason",
}


def _lt_arch() -> str:
    return os.environ.get("VIBEOS_ARCH", "amd64")


def _lt_read_imagename(eproc_addr: int) -> str:
    """Read EPROCESS.ImageFileName (CHAR[16]) as a Python string."""
    off = _lt_checked_offsets().get("_EPROCESS.ImageFileName")
    if off is not None:
        out = []
        for i in range(16):
            b = _lt_read_byte(eproc_addr + off + i)
            if b is None:
                return "<unreadable>" if not out else "".join(out)
            if b == 0:
                break
            out.append(chr(b) if 0x20 <= b < 0x7F else "?")
        return "".join(out) or "<empty>"
    try:
        s = gdb.parse_and_eval(
            f"((struct _EPROCESS *){eproc_addr:#x})->ImageFileName"
        )
        # CHAR[16] -> bytes
        out = []
        for i in range(16):
            try:
                b = int(s[i]) & 0xFF
            except gdb.error:
                break
            if b == 0:
                break
            if 0x20 <= b < 0x7F:
                out.append(chr(b))
            else:
                out.append("?")
        return "".join(out) or "<empty>"
    except gdb.error:
        return "<unreadable>"


def _lt_read_handle(addr: int, ptr_size: int) -> int | None:
    ctype = "unsigned long long" if ptr_size == 8 else "unsigned int"
    try:
        return int(gdb.parse_and_eval(f"*({ctype} *)({addr:#x})"))
    except gdb.error:
        return None


def _lt_parse_u64(expr: str) -> int | None:
    try:
        return int(gdb.parse_and_eval(expr)) & 0xFFFFFFFFFFFFFFFF
    except gdb.error:
        return None


def _lt_read_byte(addr: int) -> int | None:
    try:
        return int(gdb.parse_and_eval(f"*(unsigned char *)({addr:#x})")) & 0xFF
    except gdb.error:
        return None


def _lt_ptr_size() -> int:
    return 4 if _lt_arch() == "i386" else 8


def _lt_list_walk(head_addr: int, link_offset: int, ptr_size: int, max_iter: int = 256):
    """Yield struct addresses by walking a circular LIST_ENTRY chain.

    head_addr points at the LIST_ENTRY embedded in the *parent* (e.g. the
    global PsActiveProcessHead, or EPROCESS.ThreadListHead). For each Flink
    entry, the embedded struct address is `entry - link_offset`. Stops when
    we loop back to head or hit max_iter.
    """
    # First Flink
    flink_addr_loc = head_addr  # &LIST_ENTRY.Flink == &LIST_ENTRY
    cur = _lt_read_handle(flink_addr_loc, ptr_size)
    if cur is None:
        return
    cur &= (1 << (ptr_size * 8)) - 1
    seen = set()
    for _ in range(max_iter):
        if cur == head_addr:
            return  # full loop
        if cur in seen:
            return  # corrupted
        seen.add(cur)
        yield cur - link_offset
        # Walk to next
        cur = _lt_read_handle(cur, ptr_size)  # Flink at offset 0
        if cur is None:
            return
        cur &= (1 << (ptr_size * 8)) - 1


def _lt_current_thread_addr() -> int | None:
    """Per-arch current-thread pointer.  See sample-pc.py for derivation —
    arm64 needs raw KPCR offsets because DWARF lacks the struct."""
    ptr_size = _lt_ptr_size()
    arch = _lt_arch()

    # Per-arch raw KPCR offsets — DWARF lacks _KPCR/_KIPCR on arm64+i386.
    if arch == "arm64":
        try:
            pcr = int(gdb.parse_and_eval("(unsigned long)$TPIDR_EL1")) & 0xFFFFFFFFFFFFFFFF
        except gdb.error:
            return None
        if not pcr:
            return None
        prcb = _lt_read_handle(pcr + 0x20, ptr_size)
        if not prcb:
            return None
        return _lt_read_handle(prcb + 0x08, ptr_size)

    if arch == "i386":
        try:
            pcr = int(gdb.parse_and_eval("(unsigned long)$fs_base")) & 0xFFFFFFFF
        except gdb.error:
            return None
        if not pcr:
            return None
        prcb = _lt_read_handle(pcr + 0x20, ptr_size)
        if not prcb:
            return None
        return _lt_read_handle(prcb + 0x04, ptr_size)

    # amd64: DWARF works.
    for expr in (
        "((struct _KPCR *)$gs_base)->Prcb.CurrentThread",
        "((struct _KPCR *)$gs_base)->PrcbData.CurrentThread",
        "KeGetCurrentThread()",
    ):
        try:
            v = int(gdb.parse_and_eval(expr))
            if v:
                return v & 0xFFFFFFFFFFFFFFFF
        except gdb.error:
            continue
    return None


_LT_CHECKED_OFFSETS: dict[str, int] | None = None


def _lt_checked_offsets() -> dict[str, int]:
    """`Struct.Field` -> offset, from ntkrnl-sys's compiled layout asserts
    (`offset_check_<arch>.rs`: `assert!(offset_of!(super::ddk::_EPROCESS, Cid) == 888)`).

    The cargo-linked kernel's DWARF does not name its types `struct _EPROCESS`, so the
    DWARF lookup below finds nothing there and this command printed only "offsets
    unavailable". These asserts are COMPILED into ntkrnl-sys, so for a kernel that built
    they are the layout, not a guess."""
    global _LT_CHECKED_OFFSETS
    if _LT_CHECKED_OFFSETS is None:
        _LT_CHECKED_OFFSETS = {}
        here = os.path.dirname(os.path.abspath(globals().get("__file__", "qemu/gdb-helpers/x")))
        path = os.path.join(here, "..", "..", "sdk", "lib", "rust", "ntkrnl-sys", "src",
                            f"offset_check_{_lt_arch()}.rs")
        pat = re.compile(r"offset_of!\(super::\w+::(\w+),\s*([\w.]+)\)\s*==\s*(\d+)")
        try:
            with open(path, encoding="utf-8") as fh:
                for m in pat.finditer(fh.read()):
                    _LT_CHECKED_OFFSETS[f"{m.group(1)}.{m.group(2)}"] = int(m.group(3))
        except OSError:
            pass
    return _LT_CHECKED_OFFSETS


def _lt_resolve_field_offset(struct_name: str, field: str) -> int | None:
    """Byte offset of a field: DWARF first, then the compiled layout asserts."""
    try:
        v = gdb.parse_and_eval(f"&((struct {struct_name} *)0)->{field}")
        return int(v)
    except gdb.error:
        return _lt_checked_offsets().get(f"{struct_name}.{field}")


def _lt_candidate_process_heads(symbol_head: int) -> list[tuple[str, int]]:
    """Return candidate PsActiveProcessHead runtime addresses.

    Most builds work with the relocated COFF symbol address. Some i386 Rust
    builds have had split PE/debug address spaces for Rust-owned data symbols,
    so keep the handoff's known shift as a validated fallback instead of
    hard-coding it as the only answer.
    """
    ptr_size = _lt_ptr_size()
    candidates: list[tuple[str, int]] = [("symbol", symbol_head)]

    kd_head = _lt_parse_u64(
        "((struct KDDEBUGGER_DATA64_FULL *)&KdDebuggerDataBlock)->PsActiveProcessHead"
    )
    if kd_head:
        candidates.append(("KdDebuggerDataBlock.PsActiveProcessHead", kd_head))

    shifts: list[int] = []
    env_shift = os.environ.get("VIBEOS_GDB_SYMBOL_SHIFT")
    if env_shift:
        try:
            shifts.append(int(env_shift, 0))
        except ValueError:
            gdb.write(f"list-threads: ignoring bad VIBEOS_GDB_SYMBOL_SHIFT={env_shift!r}\n",
                      gdb.STDERR)
    if _lt_arch() == "i386":
        shifts.append(I386_RUST_SYMBOL_SHIFT)

    for shift in shifts:
        candidates.append((f"symbol+{shift:#x}", symbol_head + shift))
        if kd_head:
            candidates.append((f"KdDebuggerDataBlock.PsActiveProcessHead+{shift:#x}",
                               kd_head + shift))

    # Preserve order while de-duplicating addresses.
    out: list[tuple[str, int]] = []
    seen: set[int] = set()
    mask = (1 << (ptr_size * 8)) - 1
    for label, addr in candidates:
        addr &= mask
        if addr and addr not in seen:
            seen.add(addr)
            out.append((label, addr))
    return out


def _lt_collect_processes(head_addr: int, eproc_links_off: int,
                          eproc_threadhead_off: int, ethread_listentry_off: int,
                          ptr_size: int) -> list[tuple[int, str, int, list[int]]]:
    processes = []
    for eproc in _lt_list_walk(head_addr, eproc_links_off, ptr_size):
        name = _lt_read_imagename(eproc)
        if name == "<unreadable>":
            continue
        pid = _lt_read_handle(eproc + _LT_OFFSETS["eproc_pid"], ptr_size) or 0
        thr_head = eproc + eproc_threadhead_off
        ethreads = list(_lt_list_walk(thr_head, ethread_listentry_off, ptr_size))
        processes.append((pid, name, eproc, ethreads))
        if len(processes) >= 256:
            break
    return processes


_LT_OFFSETS: dict[str, int] = {}


class ListThreadsCmd(gdb.Command):
    """list-threads: dump every process and thread known to the kernel."""

    def __init__(self):
        super().__init__("list-threads", gdb.COMMAND_USER)

    def invoke(self, arg: str, from_tty: bool) -> None:
        ptr_size = _lt_ptr_size()

        # Resolve struct-relative offsets via DWARF.
        eproc_links_off = _lt_resolve_field_offset("_EPROCESS", "ActiveProcessLinks")
        eproc_threadhead_off = _lt_resolve_field_offset("_EPROCESS", "ThreadListHead")
        eproc_pid_off = _lt_resolve_field_offset("_EPROCESS", "UniqueProcessId")
        eproc_imgname_off = _lt_resolve_field_offset("_EPROCESS", "ImageFileName")
        ethread_listentry_off = _lt_resolve_field_offset("_ETHREAD", "ThreadListEntry")
        ethread_cid_off = _lt_resolve_field_offset("_ETHREAD", "Cid")
        kthread_state_off = _lt_resolve_field_offset("_KTHREAD", "State")
        kthread_waitreason_off = _lt_resolve_field_offset("_KTHREAD", "WaitReason")
        kthread_priority_off = _lt_resolve_field_offset("_KTHREAD", "Priority")

        if any(v is None for v in (
            eproc_links_off, eproc_threadhead_off, eproc_pid_off,
            ethread_listentry_off, ethread_cid_off,
        )):
            gdb.write("list-threads: DWARF struct offsets unavailable; "
                      "did find-kbase run?\n", gdb.STDERR)
            return
        _LT_OFFSETS["eproc_pid"] = eproc_pid_off

        # PsActiveProcessHead is a LIST_ENTRY; we need its address (not its value).
        try:
            head_addr = int(gdb.parse_and_eval("(unsigned long)&PsActiveProcessHead"))
        except gdb.error as e:
            gdb.write(f"list-threads: can't resolve PsActiveProcessHead: {e}\n",
                      gdb.STDERR)
            return

        selected_head = None
        selected_label = None
        selected_processes = []
        for label, candidate in _lt_candidate_process_heads(head_addr):
            processes = _lt_collect_processes(
                candidate, eproc_links_off, eproc_threadhead_off,
                ethread_listentry_off, ptr_size,
            )
            thread_total = sum(len(p[3]) for p in processes)
            has_systemish = any(
                name in ("System", "smss.exe", "csrss.exe", "winlogon.exe", "explorer.exe")
                for _, name, _, _ in processes
            )
            if processes and thread_total and has_systemish:
                selected_head = candidate
                selected_label = label
                selected_processes = processes
                break
            gdb.write(
                f"list-threads: rejected head {candidate:#018x} ({label}); "
                f"processes={len(processes)} threads={thread_total}\n"
            )

        if selected_head is None:
            gdb.write("list-threads: no candidate PsActiveProcessHead produced "
                      "a plausible process/thread list\n", gdb.STDERR)
            return
        head_addr = selected_head

        cur_thread = _lt_current_thread_addr()
        cur_thread_s = f"{cur_thread:#018x}" if cur_thread else "<unknown>"

        gdb.write(f"=== process list (head @ {head_addr:#018x}, "
                  f"source {selected_label}, current thread {cur_thread_s}) ===\n")
        gdb.write(f"{'Pid':>6}  {'Name':<16}  {'#Thr':>4}  EPROCESS\n")
        gdb.write("-" * 64 + "\n")

        process_count = 0
        thread_count = 0
        threads_to_print = []  # collected for second pass

        for pid, name, eproc, ethreads in selected_processes:
            process_count += 1
            n_thr = len(ethreads)
            gdb.write(f"{pid:>6}  {name:<16}  {n_thr:>4}  {eproc:#018x}\n")
            for ethread in ethreads:
                # Cid is a CLIENT_ID (UniqueProcess, UniqueThread).
                tid = _lt_read_handle(
                    ethread + ethread_cid_off + ptr_size,  # UniqueThread is second handle
                    ptr_size,
                ) or 0
                state = (
                    _lt_read_byte(ethread + kthread_state_off)
                    if kthread_state_off is not None else None
                )
                wreason = (
                    _lt_read_byte(ethread + kthread_waitreason_off)
                    if kthread_waitreason_off is not None else None
                )
                pri = (
                    _lt_read_byte(ethread + kthread_priority_off)
                    if kthread_priority_off is not None else None
                )
                threads_to_print.append((pid, name, tid, ethread, state, wreason, pri))
                thread_count += 1

        gdb.write("\n")
        gdb.write(f"=== threads ({thread_count} total) ===\n")
        gdb.write(f"{'Pid':>6}  {'Tid':>6}  {'Process':<16}  "
                  f"{'State':<11}  {'WaitReason':<14}  Pri  ETHREAD              Cur?\n")
        gdb.write("-" * 100 + "\n")

        for pid, pname, tid, ethread, state, wreason, pri in threads_to_print:
            sname = KTHREAD_STATE.get(state, f"?{state:#x}" if state is not None else "?")
            wname = (KWAIT_REASON.get(wreason, f"?{wreason:#x}")
                     if wreason is not None else "?")
            pri_s = f"{pri:>3}" if pri is not None else "  ?"
            cur_marker = "  *" if cur_thread and ethread == cur_thread else ""
            gdb.write(f"{pid:>6}  {tid:>6}  {pname:<16}  "
                      f"{sname:<11}  {wname:<14}  {pri_s}  {ethread:#018x}{cur_marker}\n")


ListThreadsCmd()
