//! flightrec — sync flight recorder (sync-instrumentation §1).
//!
//! A global lock-free ring of the last 16K sync events: `{seq, thread, object,
//! op, data}`. A lost wakeup is an ORDERING bug — by attach time the
//! interesting event is long gone; the ring keeps the recent history so a
//! wedge report (census §2) or a GDB session (§5 decoder) can read off the
//! last signal vs. last park on the stuck object, with the signalling thread.
//!
//! Design points:
//! - **No serial I/O on the record path** (prints perturb timing and mask
//!   races — the arm64 winsrv BAD-gate Heisenbug). Relaxed atomics, ~10ns.
//! - Ordering = ring sequence (the fetch_add on the write cursor IS the
//!   global order). No TSC needed — UP kernel, one timeline.
//! - Tear detection: `seq` is stored LAST with Release; a reader seeing
//!   `seq != expected` for a slot knows it raced the writer.
//! - All-zero static => .bss (zero image cost, 512KB virtual).
//!
//! Tapped from ke/synctrace.rs (one integration point: every accounting call
//! also records an event). Readers: census (dump events for a stuck object)
//! and qemu/gdb-helpers/debug_block.py (`vibeos-flightrec`).

#![allow(non_snake_case)]

use core::sync::atomic::{AtomicU64, AtomicUsize, Ordering::Relaxed, Ordering::Release};

/// Ring size — power of two. 16K × 32B = 512KB .bss.
pub const FLIGHTREC_ENTRIES: usize = 16 * 1024;

// Event ops (u8 range; keep in sync with debug_block.py OP_NAMES).
pub const OP_GATE_PARK: u32 = 1;
pub const OP_GATE_SATISFIED: u32 = 2;
pub const OP_GATE_SIGNAL: u32 = 3;
pub const OP_PL_PARK: u32 = 4;
pub const OP_PL_SATISFIED: u32 = 5;
pub const OP_PL_WAKE: u32 = 6;
// Working-set EXCLUSIVE lock/unlock (from ntkrnl-sys inline/mm.rs via the
// VibeosFlightrecLog extern below; data: 0=guarded 1=unsafe 2=mmsupport).
// Exclusive-only: the fault path takes shared (too hot for the ring), and a
// leaked WS lock that blocks a shared waiter is by definition exclusive.
pub const OP_WS_LOCK_EXCL: u32 = 16;
pub const OP_WS_UNLOCK_EXCL: u32 = 17;
// Lockdep (§4) self-deadlock: a thread about to block on a pushlock/gmutex it
// already holds. data = wanted mode (1=shared 2=excl 3=gmutex). See ke/lockdep.rs.
pub const OP_LOCKDEP_SELFDEADLOCK: u32 = 32;
// Lockdep (§4) Phase B ABBA: the lock pair was taken in both orders. Logged on
// the lock being acquired; data = its mode. Details in VIBEOS_LOCKDEP.violations.
pub const OP_LOCKDEP_ABBA: u32 = 33;
// Page table map/unmap (arm64/page.rs). `object` is the PAGE-ALIGNED VA, not a
// dispatcher object, so `flightrec_dump_object(va)` reads out that one VA's
// map/unmap history — which is exactly what the "old PTE was non-zero!" double-map
// bugcheck needs and what a whole-ring tail dump cannot give: on a real crash the
// last 32 events were 100% working-set lock churn (643K events banked, ring is 16K).
// data = the PFN, truncated to the ring's 24 data bits.
pub const OP_PAGE_MAP: u32 = 48;
pub const OP_PAGE_UNMAP: u32 = 49;
/// The RAW low 32 bits of the descriptor that OP_PAGE_UNMAP just swapped out,
/// logged beside it. The PFN alone cannot say whether it came from a hardware
/// descriptor or from a software PTE decoded with the hardware layout — and
/// `valid()` is only bit 0, so it cannot settle that either.
pub const OP_PAGE_PTE: u32 = 50;

/// Link-time tap for crates below ntoskrnl-rust in the dep graph (ntkrnl-sys
/// inlines can't `use` this crate — the dependency points the other way).
#[no_mangle]
pub extern "C" fn VibeosFlightrecLog(op: u32, object: usize, thread: usize, data: u32) {
    flightrec_log(op, object, thread, data);
}

#[repr(C)]
pub struct FlightRecEntry {
    /// Global sequence + 1 (0 = never written). Stored LAST (Release).
    pub seq: AtomicU64,
    pub thread: AtomicUsize,
    pub object: AtomicUsize,
    /// op in low 8 bits, caller-supplied data in the high 24.
    pub opdata: AtomicU64,
}

#[repr(C)]
pub struct FlightRecRing {
    /// Monotone write cursor; slot = cursor & (FLIGHTREC_ENTRIES-1).
    pub cursor: AtomicU64,
    pub entry_count: AtomicU64, // stamped at init for decoders
    pub entries: [FlightRecEntry; FLIGHTREC_ENTRIES],
}

unsafe impl Sync for FlightRecRing {}

#[allow(clippy::declare_interior_mutable_const)]
const EMPTY: FlightRecEntry = FlightRecEntry {
    seq: AtomicU64::new(0),
    thread: AtomicUsize::new(0),
    object: AtomicUsize::new(0),
    opdata: AtomicU64::new(0),
};

#[no_mangle]
pub static VIBEOS_FLIGHTREC: FlightRecRing = FlightRecRing {
    cursor: AtomicU64::new(0),
    entry_count: AtomicU64::new(0),
    entries: [EMPTY; FLIGHTREC_ENTRIES],
};

pub fn flightrec_init() {
    VIBEOS_FLIGHTREC.entry_count.store(FLIGHTREC_ENTRIES as u64, Relaxed);
}

/// Record one event. Safe at any IRQL, under any lock.
#[inline]
pub fn flightrec_log(op: u32, object: usize, thread: usize, data: u32) {
    let seq = VIBEOS_FLIGHTREC.cursor.fetch_add(1, Relaxed);
    let e = &VIBEOS_FLIGHTREC.entries[(seq as usize) & (FLIGHTREC_ENTRIES - 1)];
    e.thread.store(thread, Relaxed);
    e.object.store(object, Relaxed);
    e.opdata.store((op as u64 & 0xFF) | ((data as u64) << 8), Relaxed);
    e.seq.store(seq + 1, Release); // last: readers treat mismatched seq as torn
}

/// Human name for an op code — shared by every dumper. Kept in sync with the
/// OP_* constants above and debug_block.py OP_NAMES.
pub fn flightrec_op_name(op: u32) -> *const u8 {
    match op {
        OP_GATE_PARK => b"gate-park\0".as_ptr(),
        OP_GATE_SATISFIED => b"gate-satisfied\0".as_ptr(),
        OP_GATE_SIGNAL => b"gate-signal\0".as_ptr(),
        OP_PL_PARK => b"pushlock-park\0".as_ptr(),
        OP_PL_SATISFIED => b"pushlock-satisfied\0".as_ptr(),
        OP_PL_WAKE => b"pushlock-wake\0".as_ptr(),
        OP_WS_LOCK_EXCL => b"ws-lock-excl\0".as_ptr(),
        OP_WS_UNLOCK_EXCL => b"ws-unlock-excl\0".as_ptr(),
        OP_LOCKDEP_SELFDEADLOCK => b"lockdep-selfdeadlock\0".as_ptr(),
        OP_LOCKDEP_ABBA => b"lockdep-abba\0".as_ptr(),
        OP_PAGE_MAP => b"page-map\0".as_ptr(),
        OP_PAGE_UNMAP => b"page-unmap\0".as_ptr(),
        OP_PAGE_PTE => b"page-pte-raw\0".as_ptr(),
        _ => b"?\0".as_ptr(),
    }
}

/// Flight-recorder tail dump for the crash sidecar: print the most recent
/// `limit` sync events across ALL objects, oldest-first (so the last line is
/// the final event before the crash). Intended to run from the bugcheck path —
/// we are already halting, so the serial I/O ban on the record path does not
/// apply. Best-effort: torn slots (racing a writer) are skipped, not trusted.
pub fn flightrec_dump_recent(limit: usize) {
    let cursor = VIBEOS_FLIGHTREC.cursor.load(Relaxed);
    if cursor == 0 {
        crate::dbg_print!(b"[FLIGHTREC] (empty -- no sync events recorded)\n\0");
        return;
    }
    let span = core::cmp::min(cursor, FLIGHTREC_ENTRIES as u64);
    let want = core::cmp::min(limit as u64, span);
    crate::dbg_print!(
        b"[FLIGHTREC] --- last %lu of %lu sync events (oldest first) ---\n\0",
        want as u32,
        cursor as u32,
    );
    // Stored seq is 1-based; the newest is `cursor`, so the window is
    // (cursor-want, cursor]. Walk it oldest-first.
    let mut seq = cursor - want + 1;
    while seq <= cursor {
        let e = &VIBEOS_FLIGHTREC.entries[((seq - 1) as usize) & (FLIGHTREC_ENTRIES - 1)];
        // Skip a slot the writer already recycled (its seq moved past ours).
        if e.seq.load(Relaxed) == seq {
            let opdata = e.opdata.load(Relaxed);
            let op = (opdata & 0xFF) as u32;
            crate::dbg_print!(
                b"[FLIGHTREC]   seq=%lu %s obj=%p thr=%p data=%lu\n\0",
                seq as u32,
                flightrec_op_name(op),
                e.object.load(Relaxed) as *const (),
                e.thread.load(Relaxed) as *const (),
                (opdata >> 8) as u32,
            );
        }
        seq += 1;
    }
}

/// Census helper: print (via dbg_print) the most recent `limit` events for
/// `object`, newest last. PASSIVE-level only (it prints).
pub fn flightrec_dump_object(object: usize, limit: usize) {
    let cursor = VIBEOS_FLIGHTREC.cursor.load(Relaxed);
    let span = core::cmp::min(cursor, FLIGHTREC_ENTRIES as u64);
    // Collect newest-first indices, then print oldest-first.
    let mut found: [u64; 16] = [0; 16];
    let limit = core::cmp::min(limit, 16);
    let mut n = 0usize;
    let mut back = 0u64;
    while back < span && n < limit {
        let seq_expect = cursor - back; // 1-based stored seq
        let e = &VIBEOS_FLIGHTREC.entries[((seq_expect - 1) as usize) & (FLIGHTREC_ENTRIES - 1)];
        if e.seq.load(Relaxed) == seq_expect && e.object.load(Relaxed) == object {
            found[n] = seq_expect;
            n += 1;
        }
        back += 1;
    }
    for i in (0..n).rev() {
        let seq = found[i];
        let e = &VIBEOS_FLIGHTREC.entries[((seq - 1) as usize) & (FLIGHTREC_ENTRIES - 1)];
        let opdata = e.opdata.load(Relaxed);
        let op = (opdata & 0xFF) as u32;
        crate::dbg_print!(
            b"[FLIGHTREC]   seq=%u %s obj=%p thr=%p data=%u\n\0",
            seq as u32,
            flightrec_op_name(op),
            e.object.load(Relaxed) as *const (),
            e.thread.load(Relaxed) as *const (),
            (opdata >> 8) as u32,
        );
    }
    if n == 0 {
        crate::dbg_print!(
            b"[FLIGHTREC]   no events for obj=%p in the last %u\n\0",
            object as *const (),
            span as u32,
        );
    }
}
