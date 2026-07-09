//! lockdep — pushlock/guarded-mutex deadlock detection
//! (sync-instrumentation-design.md §4, Tier-2 opt-in, `feature = "lockdep"`).
//!
//! **Phase A — self-deadlock (length-1 cycle).** A per-thread held-lock set
//! (side table keyed by KTHREAD address — the open-addressing pattern proven
//! by `ke/synctrace.rs`). Every acquire pushes `(thread, lock, mode, site)`;
//! every release pops it. When a thread is ABOUT TO BLOCK on a lock (the
//! `Exf*AcquirePushLock*` / `KiAcquireGuardedMutex` slow paths, just before
//! `KeWaitForGate`), we scan its OWN held set: if it already holds this exact
//! lock, it is about to wait on a lock it owns — self-deadlock. Proven live
//! 2026-07-04: fires 2–4×/boot on the Mm WS-pushlock teardown fault
//! (field-manual §9.13 addendum 2) and explained the census's
//! deterministically-stranded HOLDER-STUCK threads.
//!
//! **Phase B — lock-order graph (ABBA inversions).** A global edge table of
//! *instance-address* pairs: acquiring B while holding A records the directed
//! edge A→B (first-wins). If the reverse edge B→A was ever recorded (any
//! thread), the pair has been taken in both orders → a latent ABBA deadlock —
//! recorded with both locks, both threads, and both acquire SITES (return
//! address captured in the export shims; symbolize with `addr2sym.sh`).
//! Instance-level (not class-level) on purpose:
//! - it is exact — no aliasing, no annotation burden;
//! - the address-ordered double-lock idiom (lock two WS mutexes in address
//!   order) yields only ONE direction per concrete pair, so it does NOT
//!   false-positive here (a class-level graph would need nesting annotations);
//! - try-acquires never park, so they record held entries but NO edges
//!   (`lockdep_acquire_try`) — a try-reversed order cannot deadlock.
//!
//! Discipline (identical to flightrec/synctrace):
//! - Side table only; ABI-frozen kernel objects never grow.
//! - Lock-free, allocation-free. The RECORD path is print-free (atomic
//!   stores + one flightrec breadcrumb); the census (PASSIVE) prints it.
//! - `free-on-empty`: a thread's slot is released the moment it holds nothing,
//!   so the live working set stays small across a boot's thread churn. No
//!   tombstones — a delete can break a probe chain for another key, which
//!   costs at worst a missed record (never corruption).
//! - The whole module is `#[cfg(feature = "lockdep")]` — zero image/runtime
//!   cost when the feature is off (the default; the smoke gate stays clean).

#![allow(non_snake_case)]

use core::sync::atomic::{AtomicU32, AtomicU64, AtomicUsize, Ordering::Relaxed, Ordering::Release};

use super::flightrec::{flightrec_log, OP_LOCKDEP_ABBA, OP_LOCKDEP_SELFDEADLOCK};

/// Concurrent threads tracked (power of two). Only threads *currently* holding
/// a lock occupy a slot (free-on-empty), so a few hundred is the live max;
/// 1024 is generous headroom.
pub const LOCKDEP_THREADS: usize = 1024;
/// Max simultaneously-held locks per thread. NT lock nesting is shallow
/// (WS + AddressCreation + a resource or two); 8 covers it, overflow is
/// counted (fail loud) not silently wrong.
pub const HELD_DEPTH: usize = 8;
/// Recorded violations kept for the census / GDB (ring).
pub const LOCKDEP_VIOLATIONS: usize = 16;
/// Lock-order edges (power of two). Instance-level pairs are per-lock-OBJECT,
/// so per-process locks multiply pairs (global-lock → each process's WS mutex
/// is a distinct pair per process): a real amd64 boot recorded 6K+ distinct
/// pairs and saturated an 8K table (49K dropped inserts). 32K + a deeper
/// probe window keeps the load factor sane for a boot's population; overflow
/// is counted, never silent. 32K × 48B ≈ 1.5 MB of .bss (zero image cost).
pub const LOCKDEP_EDGES: usize = 32768;
const PROBE_LIMIT: usize = 16;
/// Pair-keyed probing needs a deeper window than the thread table — the edge
/// table runs at a much higher load factor.
const EDGE_PROBE_LIMIT: usize = 32;

/// Lock modes (match the tap call sites / debug_block.py MODE_NAMES).
pub const MODE_SHARED: u32 = 1;
pub const MODE_EXCL: u32 = 2;
/// Guarded mutex (always exclusive; distinct so reports name the lock type).
pub const MODE_GM: u32 = 3;

/// Violation kinds.
pub const KIND_SELF_DEADLOCK: u32 = 1;
pub const KIND_ABBA: u32 = 2;

#[repr(C)]
pub struct HeldLock {
    /// Lock address (pushlock or guarded mutex), 0 = empty held slot.
    pub lock: AtomicUsize,
    /// Acquire site (return address in the export shim's caller; 0 = untraced
    /// direct-inline acquire).
    pub site: AtomicUsize,
    /// `MODE_SHARED` / `MODE_EXCL` / `MODE_GM`.
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

/// One directed lock-order edge: some thread held `from` while acquiring `to`.
/// First-wins: the first thread to take the pair in this order records it.
/// `modes` doubles as the publish flag (stored LAST with Release; nonzero when
/// the entry is fully written — readers skip modes==0 entries).
#[repr(C)]
pub struct Edge {
    /// Held lock address (claim word: CAS 0→from claims the slot).
    pub from: AtomicUsize,
    /// Acquired lock address.
    pub to: AtomicUsize,
    /// Acquire site of `from` (from the held entry; 0 = untraced).
    pub from_site: AtomicUsize,
    /// Acquire site of `to` (the current acquire; 0 = untraced).
    pub to_site: AtomicUsize,
    /// First thread to record this edge.
    pub thread: AtomicUsize,
    /// `(from_mode << 8) | to_mode`; 0 = entry not yet published.
    pub modes: AtomicU32,
    pub _pad: u32,
}

#[repr(C)]
pub struct Violation {
    /// Global violation number (1-based); 0 = empty. Stored LAST (Release).
    pub seq: AtomicU64,
    /// The violating thread (self-deadlock: the parker; ABBA: the thread that
    /// completed the second direction).
    pub thread: AtomicUsize,
    /// The lock being acquired.
    pub lock: AtomicUsize,
    /// ABBA only: the lock held while acquiring `lock` (0 for self-deadlock).
    pub lock2: AtomicUsize,
    /// ABBA only: the thread that recorded the REVERSE edge earlier.
    pub thread2: AtomicUsize,
    /// ABBA only: our acquire site of `lock` (0 = untraced).
    pub site: AtomicUsize,
    /// ABBA only: the reverse edge's acquire site (thread2 acquiring `lock2`).
    pub site2: AtomicUsize,
    /// Self-deadlock: mode we already hold the lock in. ABBA: mode of `lock2`.
    pub held_mode: AtomicU32,
    /// Mode of the `lock` acquisition.
    pub wanted_mode: AtomicU32,
    /// `KIND_SELF_DEADLOCK` / `KIND_ABBA`.
    pub kind: AtomicU32,
    pub _pad: u32,
}

#[repr(C)]
pub struct LockdepTable {
    /// 'LKD2' once initialized (all-zero at build time → .bss, no image cost).
    pub magic: AtomicU32,
    pub thread_slots: AtomicU32,
    pub held_depth: AtomicU32,
    pub edge_slots: AtomicU32,
    /// Thread-slot inserts dropped (probe window full) — census reports it.
    pub dropped: AtomicU32,
    /// Edge inserts dropped (probe window full) — census reports it.
    pub edges_dropped: AtomicU32,
    /// Distinct edges recorded (informational; ≈ graph size).
    pub edge_count: AtomicU32,
    pub _pad: u32,
    /// Total violations recorded (may exceed the ring size).
    pub violation_count: AtomicU64,
    pub threads: [ThreadHeld; LOCKDEP_THREADS],
    pub violations: [Violation; LOCKDEP_VIOLATIONS],
    pub edges: [Edge; LOCKDEP_EDGES],
}

// SAFETY: all mutation is through atomics.
unsafe impl Sync for LockdepTable {}

// The GDB decoder (qemu/gdb-helpers/debug_block.py, LKD2) hand-computes these
// offsets — pin the layout so drift breaks the BUILD, not the decoder.
const PSZ: usize = core::mem::size_of::<usize>();
const _: () = assert!(core::mem::size_of::<HeldLock>() == 2 * PSZ + 8);
const _: () = assert!(core::mem::size_of::<ThreadHeld>() == PSZ + 8 + HELD_DEPTH * (2 * PSZ + 8));
const _: () = assert!(core::mem::size_of::<Violation>() == 8 + 6 * PSZ + 16);
const _: () = assert!(core::mem::size_of::<Edge>() == 5 * PSZ + 8);
const _: () = assert!(core::mem::offset_of!(LockdepTable, violation_count) == 32);
const _: () = assert!(core::mem::offset_of!(LockdepTable, threads) == 40);

#[allow(clippy::declare_interior_mutable_const)]
const EMPTY_HELD: HeldLock = HeldLock {
    lock: AtomicUsize::new(0),
    site: AtomicUsize::new(0),
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
    lock2: AtomicUsize::new(0),
    thread2: AtomicUsize::new(0),
    site: AtomicUsize::new(0),
    site2: AtomicUsize::new(0),
    held_mode: AtomicU32::new(0),
    wanted_mode: AtomicU32::new(0),
    kind: AtomicU32::new(0),
    _pad: 0,
};

#[allow(clippy::declare_interior_mutable_const)]
const EMPTY_EDGE: Edge = Edge {
    from: AtomicUsize::new(0),
    to: AtomicUsize::new(0),
    from_site: AtomicUsize::new(0),
    to_site: AtomicUsize::new(0),
    thread: AtomicUsize::new(0),
    modes: AtomicU32::new(0),
    _pad: 0,
};

/// Exported (unmangled) so GDB / the debugger data block can find it without
/// DWARF — the Rust kernel has none.
#[no_mangle]
pub static VIBEOS_LOCKDEP: LockdepTable = LockdepTable {
    magic: AtomicU32::new(0),
    thread_slots: AtomicU32::new(0),
    held_depth: AtomicU32::new(0),
    edge_slots: AtomicU32::new(0),
    dropped: AtomicU32::new(0),
    edges_dropped: AtomicU32::new(0),
    edge_count: AtomicU32::new(0),
    _pad: 0,
    violation_count: AtomicU64::new(0),
    threads: [EMPTY_THREAD; LOCKDEP_THREADS],
    violations: [EMPTY_VIOLATION; LOCKDEP_VIOLATIONS],
    edges: [EMPTY_EDGE; LOCKDEP_EDGES],
};

/// Stamp the decoder header. Called once from `ExpInitSystemPhase1`; tracking
/// works before this (the table needs no init) — the stamp only lets external
/// decoders sanity-check they found the right symbol/layout.
pub fn lockdep_init() {
    VIBEOS_LOCKDEP.thread_slots.store(LOCKDEP_THREADS as u32, Relaxed);
    VIBEOS_LOCKDEP.held_depth.store(HELD_DEPTH as u32, Relaxed);
    VIBEOS_LOCKDEP.edge_slots.store(LOCKDEP_EDGES as u32, Relaxed);
    VIBEOS_LOCKDEP.magic.store(0x4C4B_4432, Relaxed); // 'LKD2'
}

const FIB: usize = 0x9E37_79B9_7F4A_7C15_u64 as usize;

/// Fibonacci-hash the thread address (low bits are alignment zeros).
#[inline]
fn thread_slot_index(key: usize) -> usize {
    let h = (key >> 4).wrapping_mul(FIB);
    (h >> 16) & (LOCKDEP_THREADS - 1)
}

/// Hash a directed lock pair to an edge slot.
#[inline]
fn edge_index(from: usize, to: usize) -> usize {
    let k = (from >> 4).wrapping_add((to >> 4).rotate_left(21));
    (k.wrapping_mul(FIB) >> 16) & (LOCKDEP_EDGES - 1)
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

/// Look up the published edge (from→to). Skips half-written entries
/// (modes==0) — a racing reader may transiently miss a brand-new edge, which
/// costs a duplicate insert or a missed first-discovery, never corruption.
#[inline]
fn edge_lookup(from: usize, to: usize) -> Option<&'static Edge> {
    let start = edge_index(from, to);
    for i in 0..EDGE_PROBE_LIMIT {
        let e = &VIBEOS_LOCKDEP.edges[(start + i) & (LOCKDEP_EDGES - 1)];
        let f = e.from.load(Relaxed);
        if f == 0 {
            return None; // end of chain
        }
        if f == from && e.to.load(Relaxed) == to && e.modes.load(Relaxed) != 0 {
            return Some(e);
        }
    }
    None
}

/// Record edge (from→to) if not already present. Returns true if this call
/// inserted it (first sighting of this ordered pair).
#[inline]
fn edge_record(
    from: usize,
    from_mode: u32,
    from_site: usize,
    to: usize,
    to_mode: u32,
    to_site: usize,
    thread: usize,
) -> bool {
    let start = edge_index(from, to);
    for i in 0..EDGE_PROBE_LIMIT {
        let e = &VIBEOS_LOCKDEP.edges[(start + i) & (LOCKDEP_EDGES - 1)];
        let f = e.from.load(Relaxed);
        if f == from && e.to.load(Relaxed) == to {
            return false; // already recorded (or being recorded right now)
        }
        if f == 0 {
            match e.from.compare_exchange(0, from, Relaxed, Relaxed) {
                Ok(_) => {
                    e.to.store(to, Relaxed);
                    e.from_site.store(from_site, Relaxed);
                    e.to_site.store(to_site, Relaxed);
                    e.thread.store(thread, Relaxed);
                    // Publish LAST: modes nonzero = entry complete.
                    e.modes.store((from_mode << 8) | to_mode.max(1), Release);
                    VIBEOS_LOCKDEP.edge_count.fetch_add(1, Relaxed);
                    return true;
                }
                Err(_) => {
                    // Raced another insert into this slot; if it was the same
                    // pair we're done, else keep probing.
                    if e.from.load(Relaxed) == from && e.to.load(Relaxed) == to {
                        return false;
                    }
                    continue;
                }
            }
        }
    }
    VIBEOS_LOCKDEP.edges_dropped.fetch_add(1, Relaxed);
    false
}

/// Is an ABBA violation for the unordered pair {a, b} already in the ring?
/// (Report dedup — the underlying edges stay recorded either way.)
fn abba_already_reported(a: usize, b: usize) -> bool {
    for v in VIBEOS_LOCKDEP.violations.iter() {
        if v.kind.load(Relaxed) == KIND_ABBA {
            let l1 = v.lock.load(Relaxed);
            let l2 = v.lock2.load(Relaxed);
            if (l1 == a && l2 == b) || (l1 == b && l2 == a) {
                return true;
            }
        }
    }
    false
}

/// A lock was successfully acquired by `thread` in `mode` — record lock-order
/// edges against everything already held, check each NEW ordered pair for a
/// previously-recorded REVERSE edge (ABBA), then push into the held set.
/// (The SELF-deadlock check is at PARK, not here: a successful acquire never
/// blocked, so re-appearing here is legitimate e.g. recursive shared.)
#[inline]
pub fn lockdep_acquire(thread: usize, lock: usize, mode: u32, site: usize) {
    if thread == 0 || lock == 0 {
        return;
    }
    if let Some(t) = thread_slot(thread, true) {
        // Phase B: order edges vs the PRE-acquire held set.
        for h in t.held.iter() {
            let held = h.lock.load(Relaxed);
            if held == 0 || held == lock {
                continue; // empty, or same-lock re-acquire (kind-1 territory)
            }
            let held_mode = h.mode.load(Relaxed);
            let held_site = h.site.load(Relaxed);
            edge_record(held, held_mode, held_site, lock, mode, site, thread);
            // Reverse direction ever taken (by anyone)? → ABBA. Checked on
            // every sighting (not just first insert) so a racy concurrent
            // first-insert of both directions still gets caught eventually;
            // the report ring is deduped per unordered pair instead.
            if let Some(rev) = edge_lookup(lock, held) {
                if !abba_already_reported(lock, held) {
                    record_violation(
                        KIND_ABBA,
                        thread,
                        lock,
                        held,
                        rev.thread.load(Relaxed),
                        site,
                        rev.from_site.load(Relaxed),
                        held_mode,
                        mode,
                    );
                }
            }
        }
        // Phase A: push into the held set.
        for h in t.held.iter() {
            if h.lock.compare_exchange(0, lock, Relaxed, Relaxed).is_ok() {
                h.site.store(site, Relaxed);
                h.mode.store(mode, Relaxed);
                return;
            }
        }
        t.overflow.fetch_add(1, Relaxed);
    }
}

/// Try-acquire success: push into the held set but record NO edges — a try
/// never parks, so a try-reversed order cannot deadlock (and would otherwise
/// be a class of ABBA false positives).
#[inline]
pub fn lockdep_acquire_try(thread: usize, lock: usize, mode: u32, site: usize) {
    if thread == 0 || lock == 0 {
        return;
    }
    if let Some(t) = thread_slot(thread, true) {
        for h in t.held.iter() {
            if h.lock.compare_exchange(0, lock, Relaxed, Relaxed).is_ok() {
                h.site.store(site, Relaxed);
                h.mode.store(mode, Relaxed);
                return;
            }
        }
        t.overflow.fetch_add(1, Relaxed);
    }
}

/// A lock was released by `thread` — drop it from the held set. Frees the
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

/// Called from the contended slow paths (`Exf*AcquirePushLock*`,
/// `KiAcquireGuardedMutex`) immediately before the thread blocks on the wake
/// gate. If `thread` already holds `lock`, it is about to wait on a lock it
/// owns → self-deadlock. Records it (print-free).
#[inline]
#[no_mangle]
pub extern "C" fn lockdep_check_park(thread: usize, lock: usize, wanted_mode: u32) {
    if thread == 0 || lock == 0 {
        return;
    }
    let t = match thread_slot(thread, false) {
        Some(t) => t,
        None => return,
    };
    for h in t.held.iter() {
        if h.lock.load(Relaxed) == lock {
            record_violation(
                KIND_SELF_DEADLOCK,
                thread,
                lock,
                0,
                0,
                h.site.load(Relaxed),
                0,
                h.mode.load(Relaxed),
                wanted_mode,
            );
            return;
        }
    }
}

/// FAULT-UNDER-WS tripwire (field-manual §9.13 family): MmAccessFault passes
/// the WS lock(s) its handling will acquire; if the faulting thread ALREADY
/// holds one of them, report AT THE FAULT — with the faulting address — rather
/// than (maybe) at park time. This adds coverage the park check cannot give:
/// a shared re-acquire of a shared-held pushlock SUCCEEDS without parking
/// (silent recursion, no self-deadlock record), and the park-time report has
/// no fault context (which address, that a #PF triggered the re-acquire).
/// Prints immediately because in the excl case the census may never run.
pub fn lockdep_fault_check(thread: usize, fault_addr: usize, ws_locks: &[usize]) {
    if thread == 0 {
        return;
    }
    let t = match thread_slot(thread, false) {
        Some(t) => t,
        None => return,
    };
    for h in t.held.iter() {
        let lk = h.lock.load(Relaxed);
        if lk == 0 {
            continue;
        }
        if ws_locks.iter().any(|&w| w != 0 && w == lk) {
            let mode = h.mode.load(Relaxed);
            let site = h.site.load(Relaxed);
            crate::dbg_print!(
                b"[LOCKDEP] FAULT-UNDER-WS: thread=%p faults on %p while holding WS lock=%p(%s) site=%p - the fault path re-acquires it (self-deadlock / silent recursion)\n\0",
                thread as *const (),
                fault_addr as *const (),
                lk as *const (),
                mode_name(mode),
                site as *const (),
            );
            // Violation record for the census/GDB decoders: site2 = fault addr.
            record_violation(KIND_SELF_DEADLOCK, thread, lk, 0, 0, site, fault_addr, mode, 0);
            return;
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn record_violation(
    kind: u32,
    thread: usize,
    lock: usize,
    lock2: usize,
    thread2: usize,
    site: usize,
    site2: usize,
    held_mode: u32,
    wanted_mode: u32,
) {
    let seq = VIBEOS_LOCKDEP.violation_count.fetch_add(1, Relaxed) + 1;
    let v = &VIBEOS_LOCKDEP.violations[((seq - 1) as usize) % LOCKDEP_VIOLATIONS];
    v.kind.store(kind, Relaxed);
    v.thread.store(thread, Relaxed);
    v.lock.store(lock, Relaxed);
    v.lock2.store(lock2, Relaxed);
    v.thread2.store(thread2, Relaxed);
    v.site.store(site, Relaxed);
    v.site2.store(site2, Relaxed);
    v.held_mode.store(held_mode, Relaxed);
    v.wanted_mode.store(wanted_mode, Relaxed);
    v.seq.store(seq, Release); // last: decoders treat mismatched seq as torn
    // Breadcrumb on the lock object so the flight recorder timeline shows the
    // violation inline with the lock/park events on the same address.
    let op = if kind == KIND_ABBA { OP_LOCKDEP_ABBA } else { OP_LOCKDEP_SELFDEADLOCK };
    flightrec_log(op, lock, thread, wanted_mode);
}

fn mode_name(m: u32) -> *const u8 {
    match m {
        MODE_SHARED => b"shared\0".as_ptr(),
        MODE_EXCL => b"excl\0".as_ptr(),
        MODE_GM => b"gmutex\0".as_ptr(),
        _ => b"?\0".as_ptr(),
    }
}

/// Census helper (PASSIVE): print any recorded violations. A nonzero count is
/// the smoking gun for the Mm WS-pushlock wedge family (kind 1) or a latent
/// lock-order inversion (kind 2).
pub fn lockdep_report() {
    let total = VIBEOS_LOCKDEP.violation_count.load(Relaxed);
    if total == 0 {
        return;
    }
    crate::dbg_print!(
        b"[LOCKDEP] %u violation(s) recorded (thread-slot drops=%u, edge drops=%u, edges=%u):\n\0",
        total as u32,
        VIBEOS_LOCKDEP.dropped.load(Relaxed),
        VIBEOS_LOCKDEP.edges_dropped.load(Relaxed),
        VIBEOS_LOCKDEP.edge_count.load(Relaxed),
    );
    let shown = core::cmp::min(total, LOCKDEP_VIOLATIONS as u64);
    for i in 0..shown {
        let v = &VIBEOS_LOCKDEP.violations[i as usize];
        let seq = v.seq.load(Relaxed);
        if seq == 0 {
            continue;
        }
        match v.kind.load(Relaxed) {
            KIND_ABBA => {
                crate::dbg_print!(
                    b"[LOCKDEP]   ABBA #%u thr=%p holds %p(%s) acquiring %p(%s) site=%p\n\0",
                    seq as u32,
                    v.thread.load(Relaxed) as *const (),
                    v.lock2.load(Relaxed) as *const (),
                    mode_name(v.held_mode.load(Relaxed)),
                    v.lock.load(Relaxed) as *const (),
                    mode_name(v.wanted_mode.load(Relaxed)),
                    v.site.load(Relaxed) as *const (),
                );
                crate::dbg_print!(
                    b"[LOCKDEP]        reverse order seen: thr=%p took %p before %p site=%p\n\0",
                    v.thread2.load(Relaxed) as *const (),
                    v.lock.load(Relaxed) as *const (),
                    v.lock2.load(Relaxed) as *const (),
                    v.site2.load(Relaxed) as *const (),
                );
            }
            _ => {
                crate::dbg_print!(
                    b"[LOCKDEP]   SELF-DEADLOCK #%u thread=%p WAIT(%s) on lock=%p already held(%s) site=%p\n\0",
                    seq as u32,
                    v.thread.load(Relaxed) as *const (),
                    mode_name(v.wanted_mode.load(Relaxed)),
                    v.lock.load(Relaxed) as *const (),
                    mode_name(v.held_mode.load(Relaxed)),
                    v.site.load(Relaxed) as *const (),
                );
            }
        }
    }
}

/// Total recorded violations (for the census verdict / smoke triage).
pub fn lockdep_violation_count() -> u64 {
    VIBEOS_LOCKDEP.violation_count.load(Relaxed)
}

// ============================================================================
// Acquire-site capture: llvm.returnaddress(0), inlined into the export shims
// (which are #[inline(never)] extern "C"/"fastcall" — their return address IS
// the true caller). The ddk→shim funnel collapses #[track_caller], so this is
// the only cheap true-caller signal. Symbolize with qemu/addr2sym.sh.
// ============================================================================

extern "C" {
    #[link_name = "llvm.returnaddress"]
    fn llvm_returnaddress(level: i32) -> *const u8;
}

/// Return address of the CALLING function's frame — must be `inline(always)`
/// so the intrinsic evaluates in the shim's frame, not a helper's.
#[inline(always)]
pub unsafe fn return_address0() -> usize {
    llvm_returnaddress(0) as usize
}

// ============================================================================
// Link-time taps for ntkrnl-sys inline funnels (the pushlock/gmutex API
// inlines sit BELOW this crate in the dep graph, so they can't `use` it — the
// dependency points the other way, same shape as flightrec's
// VibeosFlightrecLog).
// ============================================================================

#[no_mangle]
pub extern "C" fn VibeosLockdepAcquire(thread: usize, lock: usize, mode: u32, site: usize) {
    lockdep_acquire(thread, lock, mode, site);
}

#[no_mangle]
pub extern "C" fn VibeosLockdepAcquireTry(thread: usize, lock: usize, mode: u32, site: usize) {
    lockdep_acquire_try(thread, lock, mode, site);
}

#[no_mangle]
pub extern "C" fn VibeosLockdepRelease(thread: usize, lock: usize) {
    lockdep_release(thread, lock);
}
