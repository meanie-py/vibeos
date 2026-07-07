//! synctrace — wait/signal accounting side-table (sync-instrumentation §3).
//!
//! A fixed open-addressing hash table keyed by dispatcher-object address,
//! counting `{waits_parked, waits_satisfied, signals}` per object with relaxed
//! atomics. It exists to answer, from a wedged boot, the question that costs a
//! GDB archaeology session every time: is a stuck gate/pushlock a **lost
//! wakeup** (`signals >= waits_parked` yet a waiter still parked) or a
//! **holder stuck** (no signal ever sent)?
//!
//! Constraints (sync-instrumentation-design.md):
//! - ABI-frozen kernel objects — this is a SIDE table, nothing grows.
//! - Callers hold dispatcher/thread locks at SYNCH_LEVEL: everything here is
//!   lock-free, allocation-free, print-free. Relaxed atomics only.
//! - Always-on Tier 0: a table miss costs one hash + ≤16 probes.
//!
//! Readers: `ke/census.rs` (the wedge self-census, prints counters for stuck
//! objects) and GDB via the exported `VIBEOS_SYNCTRACE` symbol (decoder:
//! `qemu/gdb-helpers/debug_block.py`).

#![allow(non_snake_case)]

use core::sync::atomic::{AtomicU32, AtomicUsize, Ordering::Relaxed};

/// Slot count — power of two. 4096 × 48B = 192KB of .bss (virtual, zero-fill;
/// no image-size cost). A boot touches a few hundred distinct gates/pushlocks.
pub const SYNCTRACE_SLOTS: usize = 4096;
const PROBE_LIMIT: usize = 16;

#[repr(C)]
pub struct SyncTraceEntry {
    /// Object address (gate, pushlock, …). 0 = empty slot. Set once by CAS;
    /// entries are never freed (objects are counted for the boot's lifetime —
    /// address reuse across object lifetimes is acceptable noise for a
    /// diagnostic; the census cross-checks against live waiters anyway).
    pub key: AtomicUsize,
    /// Waits that actually PARKED (inserted into the wait list and swapped
    /// out). Fast-path satisfied waits (SignalState already set) don't count —
    /// the lost-wakeup discrimination only cares about parked waiters.
    pub waits_parked: AtomicU32,
    /// Parked waits that were satisfied (returned from the swap).
    pub waits_satisfied: AtomicU32,
    /// Signals delivered to the object (KeSignalGateBoostPriority /
    /// pushlock wake, keyed per waiter woken).
    pub signals: AtomicU32,
    pub _pad: u32,
    /// Most recent parker / signaller (PKTHREAD) — from a wedge, "who was
    /// supposed to wake this" is half the diagnosis.
    pub last_waiter: AtomicUsize,
    pub last_signaller: AtomicUsize,
}

#[repr(C)]
pub struct SyncTraceTable {
    /// Version stamp for external decoders (bump on layout change). Stamped
    /// at Phase-1 by `synctrace_init` — the static must be ALL-ZERO at build
    /// time so its 192KB lands in .bss (zero-fill), not .data (image bytes).
    pub magic: AtomicU32, // 'STT1' once initialized
    pub slot_count: AtomicU32,
    /// Inserts dropped because the probe window was full — nonzero means the
    /// counters under-report and the census says so (fail loud, never silent).
    pub dropped: AtomicU32,
    pub _pad: u32,
    pub slots: [SyncTraceEntry; SYNCTRACE_SLOTS],
}

// SAFETY: all mutation is through atomics.
unsafe impl Sync for SyncTraceTable {}

#[allow(clippy::declare_interior_mutable_const)]
const EMPTY_ENTRY: SyncTraceEntry = SyncTraceEntry {
    key: AtomicUsize::new(0),
    waits_parked: AtomicU32::new(0),
    waits_satisfied: AtomicU32::new(0),
    signals: AtomicU32::new(0),
    _pad: 0,
    last_waiter: AtomicUsize::new(0),
    last_signaller: AtomicUsize::new(0),
};

/// Exported (unmangled) so GDB / the debugger data block can find it without
/// DWARF — the Rust kernel has none (field-manual: DWARF is dead here).
#[no_mangle]
pub static VIBEOS_SYNCTRACE: SyncTraceTable = SyncTraceTable {
    magic: AtomicU32::new(0),
    slot_count: AtomicU32::new(0),
    dropped: AtomicU32::new(0),
    _pad: 0,
    slots: [EMPTY_ENTRY; SYNCTRACE_SLOTS],
};

/// Stamp the decoder header. Called once from `ExpInitSystemPhase1`; counting
/// works before this (the table needs no init), the stamp is only for external
/// decoders to sanity-check they found the right symbol/layout.
pub fn synctrace_init() {
    VIBEOS_SYNCTRACE.slot_count.store(SYNCTRACE_SLOTS as u32, Relaxed);
    VIBEOS_SYNCTRACE.magic.store(0x5354_5431, Relaxed); // 'STT1'
}

/// Fibonacci-hash the object address into the table (low bits of kernel
/// addresses are alignment zeros — shift them out first).
#[inline]
fn slot_index(key: usize) -> usize {
    let h = (key >> 4).wrapping_mul(0x9E37_79B9_7F4A_7C15_u64 as usize);
    (h >> 16) & (SYNCTRACE_SLOTS - 1)
}

/// Find (or claim) the entry for `key`. Returns None when the probe window is
/// exhausted — counted in `dropped`, never silent.
#[inline]
fn entry_for(key: usize) -> Option<&'static SyncTraceEntry> {
    if key == 0 {
        return None;
    }
    let start = slot_index(key);
    for i in 0..PROBE_LIMIT {
        let e = &VIBEOS_SYNCTRACE.slots[(start + i) & (SYNCTRACE_SLOTS - 1)];
        let k = e.key.load(Relaxed);
        if k == key {
            return Some(e);
        }
        if k == 0 {
            match e.key.compare_exchange(0, key, Relaxed, Relaxed) {
                Ok(_) => return Some(e),
                Err(actual) if actual == key => return Some(e),
                Err(_) => continue, // raced with a different key — keep probing
            }
        }
    }
    VIBEOS_SYNCTRACE.dropped.fetch_add(1, Relaxed);
    None
}

/// The flight recorder (§1) taps every accounting call, so this module is the
/// single integration point for both tables. Gate call sites (gate.rs) use the
/// plain entry points; pushlock call sites (ex/pushlock.rs) the `_pl` ones —
/// same counters, distinct flight-recorder ops.
use super::flightrec::{
    flightrec_log, OP_GATE_PARK, OP_GATE_SATISFIED, OP_GATE_SIGNAL, OP_PL_PARK, OP_PL_SATISFIED,
    OP_PL_WAKE,
};

#[inline]
fn account_park(op: u32, object: usize, thread: usize) {
    flightrec_log(op, object, thread, 0);
    if let Some(e) = entry_for(object) {
        e.waits_parked.fetch_add(1, Relaxed);
        e.last_waiter.store(thread, Relaxed);
    }
}

#[inline]
fn account_satisfied(op: u32, object: usize) {
    flightrec_log(op, object, 0, 0);
    if let Some(e) = entry_for(object) {
        e.waits_satisfied.fetch_add(1, Relaxed);
    }
}

#[inline]
fn account_signal(op: u32, object: usize, thread: usize) {
    flightrec_log(op, object, thread, 0);
    if let Some(e) = entry_for(object) {
        e.signals.fetch_add(1, Relaxed);
        e.last_signaller.store(thread, Relaxed);
    }
}

/// A waiter is about to park on gate `object` (wait-list insert done, swap next).
#[inline]
pub fn synctrace_wait_park(object: *const core::ffi::c_void, thread: *const core::ffi::c_void) {
    account_park(OP_GATE_PARK, object as usize, thread as usize);
}

/// A parked gate waiter came back (its wait was satisfied).
#[inline]
pub fn synctrace_wait_satisfied(object: *const core::ffi::c_void) {
    account_satisfied(OP_GATE_SATISFIED, object as usize);
}

/// Gate `object` was signalled (one waiter's worth — call per waiter woken).
#[inline]
pub fn synctrace_signal(object: *const core::ffi::c_void, thread: *const core::ffi::c_void) {
    account_signal(OP_GATE_SIGNAL, object as usize, thread as usize);
}

/// Pushlock variants — identical accounting, pushlock-flavored recorder ops.
#[inline]
pub fn synctrace_pl_park(object: *const core::ffi::c_void, thread: *const core::ffi::c_void) {
    account_park(OP_PL_PARK, object as usize, thread as usize);
}

#[inline]
pub fn synctrace_pl_satisfied(object: *const core::ffi::c_void) {
    account_satisfied(OP_PL_SATISFIED, object as usize);
}

#[inline]
pub fn synctrace_pl_wake(object: *const core::ffi::c_void, thread: *const core::ffi::c_void) {
    account_signal(OP_PL_WAKE, object as usize, thread as usize);
}

/// Snapshot accessor for the census: counters for `object`, or None if the
/// object never hit the table. Returns (parked, satisfied, signals,
/// last_waiter, last_signaller).
pub fn synctrace_lookup(object: usize) -> Option<(u32, u32, u32, usize, usize)> {
    if object == 0 {
        return None;
    }
    let start = slot_index(object);
    for i in 0..PROBE_LIMIT {
        let e = &VIBEOS_SYNCTRACE.slots[(start + i) & (SYNCTRACE_SLOTS - 1)];
        let k = e.key.load(Relaxed);
        if k == object {
            return Some((
                e.waits_parked.load(Relaxed),
                e.waits_satisfied.load(Relaxed),
                e.signals.load(Relaxed),
                e.last_waiter.load(Relaxed),
                e.last_signaller.load(Relaxed),
            ));
        }
        if k == 0 {
            return None;
        }
    }
    None
}

/// Table-level drop counter for the census banner.
pub fn synctrace_dropped() -> u32 {
    VIBEOS_SYNCTRACE.dropped.fetch_add(0, Relaxed)
}
