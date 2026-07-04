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
// Lockdep (§4) self-deadlock: a thread about to block on a pushlock it already
// holds. data = wanted mode (1=shared 2=excl). See ke/lockdep.rs.
pub const OP_LOCKDEP_SELFDEADLOCK: u32 = 32;

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
        let name: *const u8 = match op {
            OP_GATE_PARK => b"gate-park\0".as_ptr(),
            OP_GATE_SATISFIED => b"gate-satisfied\0".as_ptr(),
            OP_GATE_SIGNAL => b"gate-signal\0".as_ptr(),
            OP_PL_PARK => b"pushlock-park\0".as_ptr(),
            OP_PL_SATISFIED => b"pushlock-satisfied\0".as_ptr(),
            OP_PL_WAKE => b"pushlock-wake\0".as_ptr(),
            _ => b"?\0".as_ptr(),
        };
        crate::dbg_print!(
            b"[FLIGHTREC]   seq=%u %s obj=%p thr=%p data=%u\n\0",
            seq as u32,
            name,
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
