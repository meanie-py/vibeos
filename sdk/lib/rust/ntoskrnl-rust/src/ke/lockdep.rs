//! lockdep — Phase A: self-deadlock / recursive-pushlock detector
//! (sync-instrumentation-design.md §4, Tier-2 opt-in, `feature = "lockdep"`).
//!
//! A per-thread held-pushlock set (side table keyed by KTHREAD address — the
//! same open-addressing pattern proven by `ke/synctrace.rs`). Every pushlock
//! acquire pushes `(thread, lock, mode)`; every release pops it. When a thread
//! is ABOUT TO BLOCK on a pushlock (the `Exf*AcquirePushLock*` slow path, just
//! before `KeWaitForGate`), we scan its OWN held set: if it already holds this
//! exact lock, it is about to wait on a lock it owns — a self-deadlock (the
//! degenerate length-1 lock cycle). We record it.
//!
//! This is the purpose-built tool for the amd64 Mm WS-pushlock service-startup
//! wedge (field-manual §9.13): `MiDeletePte` teardown holds
//! `Vm.WorkingSetMutex` EXCLUSIVE, faults at `MiDecrementShareCount+0x71`, and
//! `MmAccessFault` re-acquires the SAME mutex SHARED → self-deadlock. Because
//! the WS lock *is* a pushlock, this fires deterministically at the shared
//! park, naming both the held (excl) and wanted (shared) modes — a strict
//! upgrade over the hand-derived `+0x71` we have today. Census proved the
//! stranding is deterministic, so this fires on every affected boot.
//!
//! Discipline (identical to flightrec/synctrace):
//! - Side table only; ABI-frozen kernel objects never grow.
//! - Lock-free, allocation-free. The RECORD path is print-free (an atomic
//!   store + one flightrec breadcrumb); the census (PASSIVE) prints it.
//! - `free-on-empty`: a thread's slot is released the moment it holds nothing,
//!   so the live working set (threads currently holding ≥1 pushlock) is small
//!   and the table doesn't fill over a boot's thread churn. No tombstones —
//!   a delete can break a probe chain for another key, which costs at worst a
//!   missed record (never corruption); acceptable for a diagnostic.
//! - The whole module is `#[cfg(feature = "lockdep")]` — zero image/runtime
//!   cost when the feature is off (the default; the smoke gate stays clean).

#![allow(non_snake_case)]

use core::sync::atomic::{AtomicU32, AtomicU64, AtomicUsize, Ordering::Relaxed, Ordering::Release};

use super::flightrec::{flightrec_log, OP_LOCKDEP_SELFDEADLOCK};

/// Concurrent threads tracked (power of two). Only threads *currently* holding
/// a pushlock occupy a slot (free-on-empty), so a few hundred is the live max;
/// 1024 is generous headroom.
pub const LOCKDEP_THREADS: usize = 1024;
/// Max simultaneously-held pushlocks per thread. NT lock nesting is shallow
/// (WS + AddressCreation + a resource or two); 8 covers it, overflow is
/// counted (fail loud) not silently wrong.
pub const HELD_DEPTH: usize = 8;
/// Recorded violations kept for the census / GDB (ring). The first
/// self-deadlock wedges its thread forever, so a small ring suffices.
pub const LOCKDEP_VIOLATIONS: usize = 16;
const PROBE_LIMIT: usize = 16;

/// Lock modes (match the pushlock API / debug_block.py MODE_NAMES).
pub const MODE_SHARED: u32 = 1;
pub const MODE_EXCL: u32 = 2;

#[repr(C)]
pub struct HeldLock {
    /// Pushlock address, 0 = empty held slot.
    pub lock: AtomicUsize,
    /// `MODE_SHARED` / `MODE_EXCL`.
    pub mode: AtomicU32,
    pub _pad: u32,
}

#[repr(C)]
pub struct ThreadHeld {
    /// KTHREAD address, 0 = free slot.
    pub thread: AtomicUsize,
    /// Pushes dropped because `held[]` was full (fail loud for the census).
    pub overflow: AtomicU32,
    pub _pad: u32,
    pub held: [HeldLock; HELD_DEPTH],
}

#[repr(C)]
pub struct Violation {
    /// Global violation number (1-based); 0 = empty. Stored LAST (Release).
    pub seq: AtomicU64,
    pub thread: AtomicUsize,
    pub lock: AtomicUsize,
    pub held_mode: AtomicU32,
    pub wanted_mode: AtomicU32,
}

#[repr(C)]
pub struct LockdepTable {
    /// 'LKD1' once initialized (all-zero at build time → .bss, no image cost).
    pub magic: AtomicU32,
    pub thread_slots: AtomicU32,
    pub held_depth: AtomicU32,
    /// Thread-slot inserts dropped (probe window full) — census reports it.
    pub dropped: AtomicU32,
    /// Total self-deadlock violations recorded (may exceed the ring size).
    pub violation_count: AtomicU64,
    pub threads: [ThreadHeld; LOCKDEP_THREADS],
    pub violations: [Violation; LOCKDEP_VIOLATIONS],
}

// SAFETY: all mutation is through atomics.
unsafe impl Sync for LockdepTable {}

#[allow(clippy::declare_interior_mutable_const)]
const EMPTY_HELD: HeldLock = HeldLock {
    lock: AtomicUsize::new(0),
    mode: AtomicU32::new(0),
    _pad: 0,
};

#[allow(clippy::declare_interior_mutable_const)]
const EMPTY_THREAD: ThreadHeld = ThreadHeld {
    thread: AtomicUsize::new(0),
    overflow: AtomicU32::new(0),
    _pad: 0,
    held: [EMPTY_HELD; HELD_DEPTH],
};

#[allow(clippy::declare_interior_mutable_const)]
const EMPTY_VIOLATION: Violation = Violation {
    seq: AtomicU64::new(0),
    thread: AtomicUsize::new(0),
    lock: AtomicUsize::new(0),
    held_mode: AtomicU32::new(0),
    wanted_mode: AtomicU32::new(0),
};

/// Exported (unmangled) so GDB / the debugger data block can find it without
/// DWARF — the Rust kernel has none.
#[no_mangle]
pub static VIBEOS_LOCKDEP: LockdepTable = LockdepTable {
    magic: AtomicU32::new(0),
    thread_slots: AtomicU32::new(0),
    held_depth: AtomicU32::new(0),
    dropped: AtomicU32::new(0),
    violation_count: AtomicU64::new(0),
    threads: [EMPTY_THREAD; LOCKDEP_THREADS],
    violations: [EMPTY_VIOLATION; LOCKDEP_VIOLATIONS],
};

/// Stamp the decoder header. Called once from `ExpInitSystemPhase1`; tracking
/// works before this (the table needs no init) — the stamp only lets external
/// decoders sanity-check they found the right symbol/layout.
pub fn lockdep_init() {
    VIBEOS_LOCKDEP.thread_slots.store(LOCKDEP_THREADS as u32, Relaxed);
    VIBEOS_LOCKDEP.held_depth.store(HELD_DEPTH as u32, Relaxed);
    VIBEOS_LOCKDEP.magic.store(0x4C4B_4431, Relaxed); // 'LKD1'
}

/// Fibonacci-hash the thread address (low bits are alignment zeros).
#[inline]
fn thread_slot_index(key: usize) -> usize {
    let h = (key >> 4).wrapping_mul(0x9E37_79B9_7F4A_7C15_u64 as usize);
    (h >> 16) & (LOCKDEP_THREADS - 1)
}

/// Find `thread`'s slot; when `claim`, insert on first empty. None if the
/// probe window is exhausted (counted in `dropped` on a claim).
#[inline]
fn thread_slot(thread: usize, claim: bool) -> Option<&'static ThreadHeld> {
    if thread == 0 {
        return None;
    }
    let start = thread_slot_index(thread);
    for i in 0..PROBE_LIMIT {
        let e = &VIBEOS_LOCKDEP.threads[(start + i) & (LOCKDEP_THREADS - 1)];
        let k = e.thread.load(Relaxed);
        if k == thread {
            return Some(e);
        }
        if k == 0 {
            if !claim {
                return None;
            }
            match e.thread.compare_exchange(0, thread, Relaxed, Relaxed) {
                Ok(_) => return Some(e),
                Err(actual) if actual == thread => return Some(e),
                Err(_) => continue, // raced a different key — keep probing
            }
        }
    }
    if claim {
        VIBEOS_LOCKDEP.dropped.fetch_add(1, Relaxed);
    }
    None
}

/// A pushlock was successfully acquired by `thread` in `mode` — record it in
/// the thread's held set. (The self-deadlock check is at PARK, not here: a
/// successful acquire never blocks, so re-appearing here would be legitimate.)
#[inline]
pub fn lockdep_acquire(thread: usize, lock: usize, mode: u32) {
    if thread == 0 || lock == 0 {
        return;
    }
    if let Some(t) = thread_slot(thread, true) {
        for h in t.held.iter() {
            if h.lock.compare_exchange(0, lock, Relaxed, Relaxed).is_ok() {
                h.mode.store(mode, Relaxed);
                return;
            }
        }
        t.overflow.fetch_add(1, Relaxed);
    }
}

/// A pushlock was released by `thread` — drop it from the held set. Frees the
/// thread's slot when it holds nothing (recycles slots across thread churn).
#[inline]
pub fn lockdep_release(thread: usize, lock: usize) {
    if thread == 0 || lock == 0 {
        return;
    }
    let t = match thread_slot(thread, false) {
        Some(t) => t,
        None => return,
    };
    let mut cleared = false;
    let mut remaining = 0usize;
    for h in t.held.iter() {
        let cur = h.lock.load(Relaxed);
        if !cleared && cur == lock {
            h.lock.store(0, Relaxed);
            cleared = true;
        } else if cur != 0 {
            remaining += 1;
        }
    }
    if remaining == 0 {
        // Best-effort free; if a concurrent (interrupt/DPC) push re-populated
        // the slot the CAS fails and we keep it — correct either way.
        let _ = t.thread.compare_exchange(thread, 0, Relaxed, Relaxed);
    }
}

/// Called from `Exf*AcquirePushLock*` immediately before the thread blocks on
/// the wake gate. If `thread` already holds `lock`, it is about to wait on a
/// lock it owns → self-deadlock. Records it (print-free).
#[inline]
pub fn lockdep_check_park(thread: usize, lock: usize, wanted_mode: u32) {
    if thread == 0 || lock == 0 {
        return;
    }
    let t = match thread_slot(thread, false) {
        Some(t) => t,
        None => return,
    };
    for h in t.held.iter() {
        if h.lock.load(Relaxed) == lock {
            record_violation(thread, lock, h.mode.load(Relaxed), wanted_mode);
            return;
        }
    }
}

fn record_violation(thread: usize, lock: usize, held_mode: u32, wanted_mode: u32) {
    let seq = VIBEOS_LOCKDEP.violation_count.fetch_add(1, Relaxed) + 1;
    let v = &VIBEOS_LOCKDEP.violations[((seq - 1) as usize) % LOCKDEP_VIOLATIONS];
    v.thread.store(thread, Relaxed);
    v.lock.store(lock, Relaxed);
    v.held_mode.store(held_mode, Relaxed);
    v.wanted_mode.store(wanted_mode, Relaxed);
    v.seq.store(seq, Release); // last: decoders treat mismatched seq as torn
    // Breadcrumb on the lock object so the flight recorder timeline shows the
    // self-deadlock inline with the WS lock/park events on the same address.
    flightrec_log(OP_LOCKDEP_SELFDEADLOCK, lock, thread, wanted_mode);
}

fn mode_name(m: u32) -> *const u8 {
    match m {
        MODE_SHARED => b"shared\0".as_ptr(),
        MODE_EXCL => b"excl\0".as_ptr(),
        _ => b"?\0".as_ptr(),
    }
}

/// Census helper (PASSIVE): print any recorded self-deadlock violations. A
/// nonzero count is the smoking gun for the Mm WS-pushlock wedge family.
pub fn lockdep_report() {
    let total = VIBEOS_LOCKDEP.violation_count.load(Relaxed);
    if total == 0 {
        return;
    }
    let dropped = VIBEOS_LOCKDEP.dropped.load(Relaxed);
    crate::dbg_print!(
        b"[LOCKDEP] %u self-deadlock violation(s) recorded (%u thread-slot drops):\n\0",
        total as u32,
        dropped,
    );
    let shown = core::cmp::min(total, LOCKDEP_VIOLATIONS as u64);
    for i in 0..shown {
        let v = &VIBEOS_LOCKDEP.violations[i as usize];
        let seq = v.seq.load(Relaxed);
        if seq == 0 {
            continue;
        }
        crate::dbg_print!(
            b"[LOCKDEP]   SELF-DEADLOCK #%u thread=%p WAIT(%s) on pushlock=%p already held(%s)\n\0",
            seq as u32,
            v.thread.load(Relaxed) as *const (),
            mode_name(v.wanted_mode.load(Relaxed)),
            v.lock.load(Relaxed) as *const (),
            mode_name(v.held_mode.load(Relaxed)),
        );
    }
}

/// Total recorded violations (for the census verdict / smoke triage).
pub fn lockdep_violation_count() -> u64 {
    VIBEOS_LOCKDEP.violation_count.load(Relaxed)
}

// ============================================================================
// Link-time taps for ntkrnl-sys/inline/ex.rs (the pushlock API inlines sit
// BELOW this crate in the dep graph, so they can't `use` it — the dependency
// points the other way, same shape as flightrec's VibeosFlightrecLog).
// ============================================================================

#[no_mangle]
pub extern "C" fn VibeosLockdepAcquire(thread: usize, lock: usize, mode: u32) {
    lockdep_acquire(thread, lock, mode);
}

#[no_mangle]
pub extern "C" fn VibeosLockdepRelease(thread: usize, lock: usize) {
    lockdep_release(thread, lock);
}
