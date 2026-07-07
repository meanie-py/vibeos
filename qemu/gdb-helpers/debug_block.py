# debug_block: decode VIBEOS_DEBUG_BLOCK — the kernel's exported, versioned
# block of struct offsets + instrumentation-global addresses
# (sdk/lib/rust/ntoskrnl-rust/src/debugblock.rs, sync-instrumentation §5).
#
# This is the DWARF-free path the Rust kernel needs: one symbol lookup, then
# every KTHREAD/EPROCESS offset is data. Other helpers should prefer
# `DebugBlock.load()` over hand-pinned offsets (which rot on layout changes —
# the field-manual §9 pins were per-incident workarounds for exactly this).
#
# Also decodes the synctrace wait/signal table (§3) reachable through the
# block: `vibeos-synctrace [addr]` prints counters for one object or every
# populated slot.
#
# Usage (inside inspect.sh / any attached gdb):
#   source debug_block.py
#   vibeos-debug-block            # print the whole block
#   vibeos-synctrace              # dump populated synctrace slots
#   vibeos-synctrace 0xADDR       # counters for one object
#   python db = DebugBlock.load(); print(hex(db.off("kthread_state")))

import struct

import gdb

MAGIC = 0x3142_4456_5342_4956  # "VIBSVDB1" little-endian
SUPPORTED_VERSIONS = (1,)

# (name, kind) in exact struct order — kind: Q=u64, I=u32, P=ptr-sized atomic.
# Append-only, mirroring debugblock.rs; bump SUPPORTED_VERSIONS on change.
_FIELDS = [
    ("magic", "Q"),
    ("version", "I"),
    ("size", "I"),
    ("arch", "I"),
    ("pointer_width", "I"),
    ("kthread_size", "I"),
    ("kthread_state", "I"),
    ("kthread_wait_irql", "I"),
    ("kthread_wait_reason", "I"),
    ("kthread_kernel_stack", "I"),
    ("kthread_stack_base", "I"),
    ("kthread_gate_object", "I"),
    ("kthread_wait_block", "I"),
    ("kthread_thread_list_entry", "I"),
    ("kthread_queue", "I"),
    ("ethread_size", "I"),
    ("ethread_cid", "I"),
    ("ethread_thread_list_entry", "I"),
    ("eprocess_size", "I"),
    ("eprocess_unique_process_id", "I"),
    ("eprocess_active_process_links", "I"),
    ("eprocess_image_file_name", "I"),
    ("eprocess_pcb_thread_list_head", "I"),
    ("eprocess_thread_list_head", "I"),
    ("kprcb_current_thread", "I"),
    ("_pad", "I"),
    ("ps_active_process_head", "P"),
    ("ps_initial_system_process", "P"),
    ("synctrace_table", "P"),
    ("census_scan_count", "P"),
    ("flightrec_ring", "P"),
]

ARCH_NAMES = {1: "i386", 2: "amd64", 3: "arm64"}


def _read(addr, size):
    return bytes(gdb.selected_inferior().read_memory(addr, size))


class DebugBlock:
    """Decoded VIBEOS_DEBUG_BLOCK. `off(name)` for offsets, `addr(name)` for
    runtime-stamped global addresses (0 until Phase-1 ran)."""

    def __init__(self, base, values):
        self.base = base
        self.values = values

    @classmethod
    def load(cls):
        sym = gdb.lookup_global_symbol("VIBEOS_DEBUG_BLOCK")
        if sym is not None:
            base = int(sym.value().address)
        else:
            # Fall back to the minimal-symbol path (symbol-file loaded but no
            # global symbol object — depends on how syms were added).
            base = int(gdb.parse_and_eval("&VIBEOS_DEBUG_BLOCK"))
        values = {}
        pos = 0
        # header first to learn pointer width
        hdr = _read(base, 24)
        magic, version, size, arch, ptr_width = struct.unpack("<QIIII", hdr)
        if magic != MAGIC:
            raise gdb.GdbError(
                f"VIBEOS_DEBUG_BLOCK magic mismatch at {base:#x}: {magic:#x}"
            )
        if version not in SUPPORTED_VERSIONS:
            raise gdb.GdbError(f"unsupported debug-block version {version}")
        psz = ptr_width // 8
        raw = _read(base, size)
        for name, kind in _FIELDS:
            if kind == "Q":
                values[name] = struct.unpack_from("<Q", raw, pos)[0]
                pos += 8
            elif kind == "I":
                values[name] = struct.unpack_from("<I", raw, pos)[0]
                pos += 4
            else:  # P — align then read pointer-sized
                if pos % psz:
                    pos += psz - (pos % psz)
                fmt = "<Q" if psz == 8 else "<I"
                values[name] = struct.unpack_from(fmt, raw, pos)[0]
                pos += psz
        return cls(base, values)

    def off(self, name):
        return self.values[name]

    def addr(self, name):
        return self.values[name]

    @property
    def ptr_size(self):
        return self.values["pointer_width"] // 8


class VibeosDebugBlockCmd(gdb.Command):
    """vibeos-debug-block: print the decoded kernel debug block."""

    def __init__(self):
        super().__init__("vibeos-debug-block", gdb.COMMAND_USER)

    def invoke(self, arg, from_tty):
        db = DebugBlock.load()
        v = db.values
        print(f"VIBEOS_DEBUG_BLOCK @ {db.base:#x}")
        print(
            f"  version={v['version']} arch={ARCH_NAMES.get(v['arch'], v['arch'])}"
            f" ptr={v['pointer_width']}b size={v['size']}"
        )
        for name, _ in _FIELDS:
            if name in ("magic", "version", "size", "arch", "pointer_width", "_pad"):
                continue
            tag = "addr" if _ == "P" else "off"
            print(f"  {name:34s} {tag}={v[name]:#x}")


# ── synctrace table decoding (§3) ────────────────────────────────────
# struct SyncTraceTable { magic u32; slot_count u32; dropped u32; _pad u32;
#                         slots[slot_count] }
# struct SyncTraceEntry { key usize; parked u32; satisfied u32; signals u32;
#                         _pad u32; last_waiter usize; last_signaller usize }
SYNCTRACE_MAGIC = 0x53545431  # 'STT1'


def _synctrace_entries(db):
    tbl = db.addr("synctrace_table")
    if not tbl:
        raise gdb.GdbError("synctrace_table address not stamped (pre-Phase-1?)")
    psz = db.ptr_size
    magic, slot_count, dropped, _ = struct.unpack("<IIII", _read(tbl, 16))
    if magic not in (SYNCTRACE_MAGIC, 0):  # 0 = pre-init; counters still valid
        raise gdb.GdbError(f"synctrace magic mismatch: {magic:#x}")
    if slot_count == 0:
        slot_count = 4096  # pre-init default (SYNCTRACE_SLOTS)
    ptr = "Q" if psz == 8 else "I"
    entry_fmt = f"<{ptr}IIII{ptr}{ptr}"
    entry_size = struct.calcsize(entry_fmt)
    raw = _read(tbl + 16, slot_count * entry_size)
    out = []
    for i in range(slot_count):
        key, parked, sat, sig, _pad, lw, ls = struct.unpack_from(
            entry_fmt, raw, i * entry_size
        )
        if key:
            out.append((key, parked, sat, sig, lw, ls))
    return dropped, out


class VibeosSynctraceCmd(gdb.Command):
    """vibeos-synctrace [addr]: wait/signal counters (all slots or one object)."""

    def __init__(self):
        super().__init__("vibeos-synctrace", gdb.COMMAND_USER)

    def invoke(self, arg, from_tty):
        db = DebugBlock.load()
        dropped, entries = _synctrace_entries(db)
        want = int(arg, 0) if arg.strip() else None
        print(f"synctrace: {len(entries)} object(s), dropped={dropped}")
        shown = 0
        for key, parked, sat, sig, lw, ls in sorted(entries):
            if want is not None and key != want:
                continue
            verdict = ""
            if parked > sat:  # someone is (or was) parked unsatisfied
                verdict = (
                    " <== HOLDER-STUCK(no signal)"
                    if sig == 0
                    else (" <== LOST-WAKEUP(sig>=parked)" if sig >= parked else " <== UNDER-SIGNALLED")
                )
            print(
                f"  obj={key:#x} parked={parked} sat={sat} sig={sig}"
                f" last_waiter={lw:#x} last_sig={ls:#x}{verdict}"
            )
            shown += 1
        if want is not None and shown == 0:
            print(f"  no entry for {want:#x}")


# ── offset-free process/thread walk (replaces hand-pinned walk-procs) ─
KTHREAD_STATE_NAMES = {
    0: "Initialized", 1: "Ready", 2: "Running", 3: "Standby", 4: "Terminated",
    5: "Waiting", 6: "Transition", 7: "DeferredReady", 8: "GateWait",
}


def _ptr(db, addr):
    raw = _read(addr, db.ptr_size)
    return struct.unpack("<Q" if db.ptr_size == 8 else "<I", raw)[0]


class VibeosThreadsCmd(gdb.Command):
    """vibeos-threads [gatewait]: walk every process/thread via the debug
    block (no DWARF, no hand-pinned offsets). Optional arg filters to
    GateWait-state threads (the wedge signature)."""

    def __init__(self):
        super().__init__("vibeos-threads", gdb.COMMAND_USER)

    def invoke(self, arg, from_tty):
        db = DebugBlock.load()
        only_gatewait = arg.strip().lower() == "gatewait"
        head = db.addr("ps_active_process_head")
        if not head:
            raise gdb.GdbError("ps_active_process_head not stamped (pre-Phase-1?)")
        plink = _ptr(db, head)
        nprocs = 0
        while plink and plink != head and nprocs < 512:
            nprocs += 1
            eproc = plink - db.off("eprocess_active_process_links")
            img = _read(eproc + db.off("eprocess_image_file_name"), 16)
            img = img.split(b"\0")[0].decode("ascii", "replace") or "?"
            pid = _ptr(db, eproc + db.off("eprocess_unique_process_id"))
            thead = eproc + db.off("eprocess_pcb_thread_list_head")
            tlink = _ptr(db, thead)
            rows = []
            nthreads = 0
            while tlink and tlink != thead and nthreads < 2048:
                nthreads += 1
                kt = tlink - db.off("kthread_thread_list_entry")
                state = _read(kt + db.off("kthread_state"), 1)[0]
                if not only_gatewait or state == 8:
                    reason = _read(kt + db.off("kthread_wait_reason"), 1)[0]
                    # CLIENT_ID = {UniqueProcess, UniqueThread} — tid is 2nd.
                    tid = _ptr(db, kt + db.off("ethread_cid") + db.ptr_size)
                    obj = _ptr(db, kt + db.off("kthread_gate_object"))
                    rows.append(
                        f"    tid={tid:<6} kthread={kt:#x} "
                        f"{KTHREAD_STATE_NAMES.get(state, state)!s:<14} "
                        f"reason={reason:<3} waitobj={obj:#x}"
                    )
                tlink = _ptr(db, tlink)
            if rows or not only_gatewait:
                print(f"{img} pid={pid} eprocess={eproc:#x} ({len(rows)} shown)")
                for r in rows:
                    print(r)
            plink = _ptr(db, plink)


# ── flight recorder decoding (§1) ────────────────────────────────────
# struct FlightRecRing { cursor u64; entry_count u64; entries[] }
# struct FlightRecEntry { seq u64; thread usize; object usize; opdata u64 }
OP_NAMES = {
    1: "gate-park", 2: "gate-satisfied", 3: "gate-signal",
    4: "pushlock-park", 5: "pushlock-satisfied", 6: "pushlock-wake",
    16: "ws-lock-EXCL", 17: "ws-unlock-EXCL",
    32: "LOCKDEP-SELFDEADLOCK", 33: "LOCKDEP-ABBA",
}

# lockdep §4 (feature = "lockdep"): held-lock modes / violation kinds.
LOCKDEP_MAGIC = 0x4C4B4432  # 'LKD2' (Phase B layout: sites + edge table)
MODE_NAMES = {1: "shared", 2: "excl", 3: "gmutex"}
KIND_SELF, KIND_ABBA = 1, 2


class VibeosFlightrecCmd(gdb.Command):
    """vibeos-flightrec [N] [obj=0xADDR]: last N (default 64) sync events,
    optionally filtered to one object."""

    def __init__(self):
        super().__init__("vibeos-flightrec", gdb.COMMAND_USER)

    def invoke(self, arg, from_tty):
        db = DebugBlock.load()
        ring = db.addr("flightrec_ring")
        if not ring:
            raise gdb.GdbError("flightrec_ring not stamped (pre-Phase-1?)")
        limit, want = 64, None
        for tok in arg.split():
            if tok.startswith("obj="):
                want = int(tok[4:], 0)
            else:
                limit = int(tok, 0)
        psz = db.ptr_size
        cursor, entry_count = struct.unpack("<QQ", _read(ring, 16))
        if entry_count == 0:
            entry_count = 16 * 1024  # pre-init default (FLIGHTREC_ENTRIES)
        ptrc = "Q" if psz == 8 else "I"
        efmt = f"<Q{ptrc}{ptrc}Q"
        esz = struct.calcsize(efmt)
        span = min(cursor, entry_count)
        print(f"flightrec: cursor={cursor} showing<= {limit}"
              + (f" obj={want:#x}" if want else ""))
        shown = 0
        rows = []
        back = 0
        while back < span and shown < limit:
            seq = cursor - back  # 1-based stored seq
            slot = (seq - 1) % entry_count
            raw = _read(ring + 16 + slot * esz, esz)
            sseq, thread, obj, opdata = struct.unpack(efmt, raw)
            back += 1
            if sseq != seq:
                continue  # torn/overwritten
            if want is not None and obj != want:
                continue
            op = opdata & 0xFF
            rows.append(
                f"  seq={seq:<10} {OP_NAMES.get(op, op)!s:<20} obj={obj:#x}"
                f" thr={thread:#x} data={opdata >> 8}"
            )
            shown += 1
        for r in reversed(rows):
            print(r)


class VibeosLockdepCmd(gdb.Command):
    """vibeos-lockdep [held|edges]: sync-instrumentation §4 deadlock detector.

    Prints recorded violations — kind 1 SELF-DEADLOCK (a thread blocked on a
    pushlock/gmutex it already holds — the Mm WS-pushlock wedge) and kind 2
    ABBA (Phase B: a lock pair taken in both orders by different paths).
    `held` also dumps every thread's currently-held lock set; `edges` dumps
    the full observed lock-order graph (from→to instance pairs + acquire
    sites — symbolize sites with qemu/addr2sym.sh). Requires a --lockdep
    build; on a normal build the VIBEOS_LOCKDEP symbol is absent."""

    def __init__(self):
        super().__init__("vibeos-lockdep", gdb.COMMAND_USER)

    def invoke(self, arg, from_tty):
        try:
            base = int(gdb.parse_and_eval("&VIBEOS_LOCKDEP"))
        except gdb.error:
            raise gdb.GdbError(
                "VIBEOS_LOCKDEP symbol absent — build with `vibeos-cli build --lockdep`"
            )
        psz = DebugBlock.load().ptr_size
        ptr = "Q" if psz == 8 else "I"
        # header: magic, thread_slots, held_depth, edge_slots, dropped,
        #         edges_dropped, edge_count, _pad (8×u32) + violation_count u64
        (magic, thread_slots, held_depth, edge_slots, dropped, edrops,
         ecount, _pad) = struct.unpack("<IIIIIIII", _read(base, 32))
        (vcount,) = struct.unpack("<Q", _read(base + 32, 8))
        if magic == 0x4C4B4431:
            raise gdb.GdbError("LKD1 (Phase A) kernel — this decoder needs the LKD2 build")
        if magic not in (LOCKDEP_MAGIC, 0):
            raise gdb.GdbError(f"lockdep magic mismatch: {magic:#x}")
        if thread_slots == 0:
            thread_slots, held_depth, edge_slots = 1024, 8, 32768  # pre-init defaults
        print(f"lockdep: violations={vcount} edges={ecount} "
              f"(thread-slot-drops={dropped} edge-drops={edrops} "
              f"slots={thread_slots} depth={held_depth} edge-slots={edge_slots})")

        # HeldLock = lock ptr + site ptr + mode u32 + pad u32
        held_sz = 2 * psz + 8
        th_sz = psz + 8 + held_depth * held_sz
        HDR = 40
        viol_off = HDR + thread_slots * th_sz
        # Violation = seq u64 + thread,lock,lock2,thread2,site,site2 (6 ptr)
        #             + held_mode,wanted_mode,kind,_pad (4 u32)
        viol_fmt = f"<Q{ptr * 6}IIII"
        viol_sz = struct.calcsize(viol_fmt)
        NVIOL = 16  # LOCKDEP_VIOLATIONS
        edge_off = viol_off + NVIOL * viol_sz

        shown = 0
        for i in range(NVIOL):
            (seq, thread, lock, lock2, thread2, site, site2,
             held_mode, wanted_mode, kind, _p) = struct.unpack(
                viol_fmt, _read(base + viol_off + i * viol_sz, viol_sz)
            )
            if seq == 0:
                continue
            if kind == KIND_ABBA:
                print(
                    f"  *** ABBA #{seq} thread={thread:#x} holds {lock2:#x}"
                    f"({MODE_NAMES.get(held_mode, held_mode)}) acquiring {lock:#x}"
                    f"({MODE_NAMES.get(wanted_mode, wanted_mode)}) site={site:#x}"
                )
                print(
                    f"        reverse order seen: thread={thread2:#x} "
                    f"took {lock:#x} before {lock2:#x} site={site2:#x}"
                )
            else:
                # site2 nonzero on a kind-1 record = the FAULT-UNDER-WS tripwire
                # (MmAccessFault entry): site2 carries the faulting address.
                fault = f" fault_addr={site2:#x}" if site2 else ""
                print(
                    f"  *** SELF-DEADLOCK #{seq} thread={thread:#x} "
                    f"WAIT({MODE_NAMES.get(wanted_mode, wanted_mode)}) on lock={lock:#x} "
                    f"already held({MODE_NAMES.get(held_mode, held_mode)}) site={site:#x}{fault}"
                )
            shown += 1
        if shown == 0:
            print("  no violations recorded")

        mode = arg.strip()
        if mode == "held":
            print("currently-held lock sets:")
            for s in range(thread_slots):
                th_base = base + HDR + s * th_sz
                thread, overflow, _ = struct.unpack(f"<{ptr}II", _read(th_base, psz + 8))
                if thread == 0:
                    continue
                locks = []
                for h in range(held_depth):
                    hb = th_base + psz + 8 + h * held_sz
                    lock, site, hmode, _p = struct.unpack(
                        f"<{ptr}{ptr}II", _read(hb, held_sz)
                    )
                    if lock:
                        st = f" site={site:#x}" if site else ""
                        locks.append(f"{lock:#x}({MODE_NAMES.get(hmode, hmode)}{st})")
                if locks or overflow:
                    ov = f" overflow={overflow}" if overflow else ""
                    print(f"  thread={thread:#x} holds {', '.join(locks) or '(none)'}{ov}")
        elif mode == "edges":
            # Edge = from,to,from_site,to_site,thread (5 ptr) + modes u32 + pad
            efmt = f"<{ptr * 5}II"
            esz = struct.calcsize(efmt)
            print(f"lock-order graph ({ecount} edges):")
            for s in range(edge_slots):
                frm, to, fs, ts, thread, modes, _p = struct.unpack(
                    efmt, _read(base + edge_off + s * esz, esz)
                )
                if frm == 0 or modes == 0:
                    continue
                fm, tm = (modes >> 8) & 0xFF, modes & 0xFF
                print(
                    f"  {frm:#x}({MODE_NAMES.get(fm, fm)}@{fs:#x}) -> "
                    f"{to:#x}({MODE_NAMES.get(tm, tm)}@{ts:#x}) thr={thread:#x}"
                )


# cswitchdep (feature = "cswitchdep"): context-switch hand-off verifier.
CSWITCH_MAGIC = 0x43535731  # 'CSW1'
CSWITCH_KINDS = {
    1: "HIGH_IRQL_AT_SWITCH", 2: "IRQL_UNBALANCED_SWITCH", 3: "SWAP_IRQL",
    4: "SWITCH_FROM_DPC", 5: "CURTHREAD_INCOHERENT", 6: "STACK_OOB",
    7: "SLEEP_WITH_SPINLOCK", 8: "DPC_DISPATCH_IRQL", 9: "INTR_STATE_UNBALANCED",
}


class VibeosCswitchCmd(gdb.Command):
    """vibeos-cswitch [threads]: cswitchdep context-switch verifier tables.

    Prints the per-CPU shadow (switch census, spinlock held-counter + its
    self-calibrating floor) and the violation ring (kinds above — symbolize
    pc with qemu/addr2sym.sh). `threads` also dumps the live ThreadShadow
    slots (gen=0 = birth-branded, never suspended). Requires a --cswitchdep
    build; on a normal build the VIBEOS_CSWITCHDEP symbol is absent.
    Layout is pinned by compile-time asserts in ke/cswitchdep.rs — keep the
    offsets here in sync with them."""

    def __init__(self):
        super().__init__("vibeos-cswitch", gdb.COMMAND_USER)

    def invoke(self, arg, from_tty):
        try:
            base = int(gdb.parse_and_eval("&VIBEOS_CSWITCHDEP"))
        except gdb.error:
            raise gdb.GdbError(
                "VIBEOS_CSWITCHDEP symbol absent — build with `vibeos-cli build --cswitchdep`"
            )
        psz = DebugBlock.load().ptr_size
        ptr = "Q" if psz == 8 else "I"
        # header: magic, thread_slots, dropped, _pad (4×u32) + violation_count u64
        magic, thread_slots, dropped, _pad = struct.unpack("<IIII", _read(base, 16))
        (vcount,) = struct.unpack("<Q", _read(base + 16, 8))
        if magic not in (CSWITCH_MAGIC, 0):
            raise gdb.GdbError(f"cswitchdep magic mismatch: {magic:#x}")
        if thread_slots == 0:
            thread_slots = 1024  # pre-init default (CSWITCH_THREADS)
        # CpuShadow @24 (48 bytes): switch_irql u32, held i32, baseline i32,
        # (4 pad), switches u64, spin_acquires u64, resume_no_record u32,
        # first_runs u32, evictions u32, _pad u32.
        (sw_irql, held, floor, switches, spin_acq, no_rec, first_runs,
         evictions, _p2) = struct.unpack("<Iii4xQQIIII", _read(base + 24, 48))
        print(
            f"cswitchdep: switches={switches} violations={vcount} "
            f"spin_acq={spin_acq} held_now={held} floor={floor} "
            f"(first_runs={first_runs} no_record={no_rec} evictions={evictions} "
            f"drops={dropped} last_switch_irql={sw_irql})"
        )

        HDR = 72  # offset_of!(CswitchTable, threads)
        th_sz = psz + 8
        viol_off = HDR + thread_slots * th_sz
        # Violation: seq,expected,actual (3×u64) + kind,_pad (2×u32) +
        # pc,old,new (3×ptr), tail-padded to 8 (AtomicU64 align).
        viol_fmt = f"<QQQII{ptr * 3}"
        viol_sz = (struct.calcsize(viol_fmt) + 7) & ~7
        NVIOL = 16  # CSWITCH_VIOLATIONS
        shown = 0
        for i in range(NVIOL):
            raw = _read(base + viol_off + i * viol_sz, struct.calcsize(viol_fmt))
            seq, expected, actual, kind, _p, pc, old, new = struct.unpack(viol_fmt, raw)
            if seq == 0:
                continue
            print(
                f"  *** #{seq} {CSWITCH_KINDS.get(kind, kind)} old={old:#x} "
                f"new={new:#x} expected={expected} actual={actual} pc={pc:#x}"
            )
            shown += 1
        if shown == 0:
            print("  no violations recorded")

        if arg.strip() == "threads":
            print("live ThreadShadow slots:")
            for s in range(thread_slots):
                tb = base + HDR + s * th_sz
                kthread, gen, susp_irql = struct.unpack(f"<{ptr}II", _read(tb, th_sz))
                if kthread == 0:
                    continue
                tag = "first-run" if gen == 0 else f"gen={gen} suspend_irql={susp_irql}"
                print(f"  kthread={kthread:#x} {tag}")


VibeosDebugBlockCmd()
VibeosSynctraceCmd()
VibeosThreadsCmd()
VibeosFlightrecCmd()
VibeosLockdepCmd()
VibeosCswitchCmd()
