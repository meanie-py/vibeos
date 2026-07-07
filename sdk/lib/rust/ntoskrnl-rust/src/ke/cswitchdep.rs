//! cswitchdep — context-switch hand-off verifier
//! (docs/engineering/cswitchdep-design.md, `feature = "cswitchdep"`).
//!
//! Phase 1: taps 1/2 (thread↔thread). A context switch is where every "who may
//! run here" invariant is handed off at once; cswitchdep snapshots the
//! execution context on both sides and asserts the transition was legal,
//! pinning the violation to the switch instead of the innocent-victim crash
//! minutes later.
//!
//! Checks live in Phase 1:
//! - **tap-1 entry** (`KiSwapThreadImpl`, before `KiSwapContext`): IRQL must be
//!   exactly DISPATCH_LEVEL (UP kernel: SYNCH==DISPATCH). A thread arriving at
//!   a voluntary wait at HIGH_LEVEL is the leaked-raise class (field manual
//!   §9.24); arriving below DISPATCH is a broken caller. → `HIGH_IRQL_AT_SWITCH`
//! - **tap-1 exit** (after the caller's `KeLowerIrql(WaitIrql)`): live IRQL ==
//!   WaitIrql. → `IRQL_UNBALANCED_SWITCH`
//! - **tap 2** (`KiSwapContextResume`, post-swap on the new stack, amd64):
//!   IRQL still DISPATCH (the swap machinery never changes IRQL — the caller
//!   lowers *after*); not in a DPC (`SWITCH_FROM_DPC`, recorded before the
//!   existing BugCheck so the table survives triage); `Prcb.CurrentThread ==
//!   NewThread` (`CURTHREAD_INCOHERENT` — the offset-drift class); SP inside
//!   the new thread's kernel stack (`STACK_OOB`).
//! - **ThreadShadow lifecycle**: branded at `KiInitializeContextThread`
//!   (amd64), evicted at the final switch away from a Terminated thread —
//!   KTHREAD pool-address reuse must never inherit a dead thread's shadow.
//!   A resume whose old side has no suspend record is NORMAL (idle loop /
//!   unbranded arch): counted, never a violation.
//!
//! Discipline (identical to lockdep — see that module's header):
//! - Side table only, `.bss` (all-zero at build → zero image cost off).
//! - Lock-free, allocation-free; record path print-capped (8 shots).
//! - The whole module is `#[cfg(feature = "cswitchdep")]`, and the feature
//!   implies `lockdep` (Cargo.toml) so the held-set mirror is always present
//!   for the Phase-2 checks — never a silent vacuous pass.

#![allow(non_snake_case)]

use core::sync::atomic::{
    AtomicI32, AtomicU32, AtomicU64, AtomicUsize, Ordering::Relaxed, Ordering::Release,
};

/// Concurrent threads tracked (power of two). Slots are evicted at thread
/// termination, so the live set tracks the real thread population.
pub const CSWITCH_THREADS: usize = 1024;
/// Recorded violations kept for the census / GDB (ring).
pub const CSWITCH_VIOLATIONS: usize = 16;
const PROBE_LIMIT: usize = 16;
/// Immediate-print cap (the SWAP-TRIPWIRE rule): a recurring false positive
/// must not flood serial; the census prints the totals at PASSIVE.
const PRINT_CAP: u32 = 8;

/// Violation kinds (keep debug_block.py MODE_NAMES in sync when decoding).
pub const KIND_HIGH_IRQL_AT_SWITCH: u32 = 1;
pub const KIND_IRQL_UNBALANCED_SWITCH: u32 = 2;
pub const KIND_SWAP_IRQL: u32 = 3;
pub const KIND_SWITCH_FROM_DPC: u32 = 4;
pub const KIND_CURTHREAD_INCOHERENT: u32 = 5;
pub const KIND_STACK_OOB: u32 = 6;
/// P2: a voluntary switch with dispatch-level locks still held (per-CPU
/// held-counter fed by the Kx choke-point taps). Violation is amd64-only —
/// the only arch where the kernel Kx choke sees every funnel pair; on
/// i386/arm64 several funnels route through the HAL binary (untapped but
/// pair-symmetric), so the counter there is informational census data.
pub const KIND_SLEEP_WITH_SPINLOCK: u32 = 7;
/// P2: deferred-dispatch delivery (`KiDispatchInterrupt` /
/// `KiDispatchSoftwareIntArm64`) entered at the wrong IRQL.
pub const KIND_DPC_DISPATCH_IRQL: u32 = 8;
/// P2 (arm64): DAIF interrupt-mask state not balanced across the software-
/// interrupt dispatch (the §9.19 wake-from-exception-context family).
pub const KIND_INTR_STATE_UNBALANCED: u32 = 9;

const DISPATCH_LEVEL: u8 = 2;
/// _KTHREAD_STATE_Terminated (bindgen `_KTHREAD_STATE_Terminated == 4`).
pub const STATE_TERMINATED: u8 = 4;

#[repr(C)]
pub struct CpuShadow {
    /// IRQL observed at the last tap-1 suspend (the switching level).
    pub switch_irql: AtomicU32,
    /// P2: dispatch-level locks currently held on this CPU (Kx choke taps,
    /// +try-funnel). Signed and BASELINE-relative: `KxAcquire/ReleaseSpinLock`
    /// also exist as C FORCEINLINEs (`ntoskrnl/include/internal/spinlock.h`),
    /// so pure-C lock pairs are symmetric-invisible but MIXED pairs (C inline
    /// acquire → Rust-funnel release, the split acquire/release shape) skew
    /// the absolute count — measured ~-10.4K during an amd64 boot, stable
    /// once drivers settle. The census reports the raw value (a live gauge of
    /// the C↔Rust seam); the violation check compares against the post-boot
    /// `held_baseline` captured at `cswitchdep_init`.
    pub held_spinlocks: AtomicI32,
    /// Snapshot of `held_spinlocks` at `cswitchdep_init` (KiStartSyncCensus,
    /// dispatcher up) — the zero point for SLEEP_WITH_SPINLOCK.
    pub held_baseline: AtomicI32,
    /// Voluntary switches seen at tap 1 (census).
    pub switches: AtomicU64,
    /// P2: total spinlock acquires seen by the taps (census / coverage gauge).
    pub spin_acquires: AtomicU64,
    /// Resumes whose OLD thread had no suspend record (idle loop / unbranded
    /// arch) — normal, counted so the census shows coverage.
    pub resume_no_record: AtomicU32,
    /// Resumes of a birth-branded, never-suspended thread (first-run).
    pub first_runs: AtomicU32,
    /// ThreadShadow slots evicted at thread termination.
    pub evictions: AtomicU32,
    pub _pad: u32,
}

#[repr(C)]
pub struct ThreadShadow {
    /// KTHREAD address, 0 = free slot.
    pub kthread: AtomicUsize,
    /// 0 = birth-branded, never suspended; bumped at every suspend.
    /// Stored LAST at suspend (Release) — publish-last for external decoders.
    pub gen: AtomicU32,
    /// `KTHREAD.WaitIrql` captured at the last suspend (the IRQL the caller
    /// will restore after the switch).
    pub suspend_irql: AtomicU32,
}

#[repr(C)]
pub struct Violation {
    /// Global violation number (1-based); 0 = empty. Stored LAST (Release).
    pub seq: AtomicU64,
    pub expected: AtomicU64,
    pub actual: AtomicU64,
    pub kind: AtomicU32,
    pub _pad: u32,
    /// Tap call site (`llvm.returnaddress(0)` in the tap shim; symbolize with
    /// qemu/addr2sym.sh).
    pub pc: AtomicUsize,
    pub kthread_old: AtomicUsize,
    pub kthread_new: AtomicUsize,
}

#[repr(C)]
pub struct CswitchTable {
    /// 'CSW1' once initialized (all-zero at build time → .bss, no image cost).
    pub magic: AtomicU32,
    pub thread_slots: AtomicU32,
    /// Thread-slot inserts dropped (probe window full) — census reports it.
    pub dropped: AtomicU32,
    pub _pad: u32,
    /// Total violations recorded (may exceed the ring size).
    pub violation_count: AtomicU64,
    pub cpu: CpuShadow,
    pub threads: [ThreadShadow; CSWITCH_THREADS],
    pub violations: [Violation; CSWITCH_VIOLATIONS],
}

// SAFETY: all mutation is through atomics.
unsafe impl Sync for CswitchTable {}

// The GDB decoder (qemu/gdb-helpers/debug_block.py, CSW1) hand-computes these
// offsets — pin the layout so drift breaks the BUILD, not the decoder.
const PSZ: usize = core::mem::size_of::<usize>();
const _: () = assert!(core::mem::size_of::<CpuShadow>() == 48);
const _: () = assert!(core::mem::size_of::<ThreadShadow>() == PSZ + 8);
// AtomicU64 is align-8 on all three arches, so the struct tail-pads to 8:
// amd64/arm64: 32 + 24 = 56; i386: 32 + 12 = 44 → 48.
const _: () = assert!(core::mem::size_of::<Violation>() == (32 + 3 * PSZ + 7) & !7);
const _: () = assert!(core::mem::offset_of!(CswitchTable, violation_count) == 16);
const _: () = assert!(core::mem::offset_of!(CswitchTable, cpu) == 24);
const _: () = assert!(core::mem::offset_of!(CswitchTable, threads) == 72);

#[allow(clippy::declare_interior_mutable_const)]
const EMPTY_THREAD: ThreadShadow = ThreadShadow {
    kthread: AtomicUsize::new(0),
    gen: AtomicU32::new(0),
    suspend_irql: AtomicU32::new(0),
};

#[allow(clippy::declare_interior_mutable_const)]
const EMPTY_VIOLATION: Violation = Violation {
    seq: AtomicU64::new(0),
    expected: AtomicU64::new(0),
    actual: AtomicU64::new(0),
    kind: AtomicU32::new(0),
    _pad: 0,
    pc: AtomicUsize::new(0),
    kthread_old: AtomicUsize::new(0),
    kthread_new: AtomicUsize::new(0),
};

/// Exported (unmangled) so GDB / the debugger data block can find it without
/// DWARF — the Rust kernel has none.
#[no_mangle]
pub static VIBEOS_CSWITCHDEP: CswitchTable = CswitchTable {
    magic: AtomicU32::new(0),
    thread_slots: AtomicU32::new(0),
    dropped: AtomicU32::new(0),
    _pad: 0,
    violation_count: AtomicU64::new(0),
    cpu: CpuShadow {
        switch_irql: AtomicU32::new(0),
        held_spinlocks: AtomicI32::new(0),
        held_baseline: AtomicI32::new(0),
        switches: AtomicU64::new(0),
        spin_acquires: AtomicU64::new(0),
        resume_no_record: AtomicU32::new(0),
        first_runs: AtomicU32::new(0),
        evictions: AtomicU32::new(0),
        _pad: 0,
    },
    threads: [EMPTY_THREAD; CSWITCH_THREADS],
    violations: [EMPTY_VIOLATION; CSWITCH_VIOLATIONS],
};

static PRINTS: AtomicU32 = AtomicU32::new(0);

/// Stamp the decoder header. Called once from `KiStartSyncCensus`; tracking
/// works before this (the table needs no init).
pub fn cswitchdep_init() {
    VIBEOS_CSWITCHDEP.thread_slots.store(CSWITCH_THREADS as u32, Relaxed);
    VIBEOS_CSWITCHDEP.magic.store(0x4353_5731, Release); // 'CSW1'
}

const FIB: usize = 0x9E37_79B9_7F4A_7C15_u64 as usize;

#[inline]
fn thread_slot_index(key: usize) -> usize {
    let h = (key >> 4).wrapping_mul(FIB);
    (h >> 16) & (CSWITCH_THREADS - 1)
}

/// Find `thread`'s slot; when `claim`, insert on first empty. None if the
/// probe window is exhausted (counted in `dropped` on a claim).
#[inline]
fn thread_slot(thread: usize, claim: bool) -> Option<&'static ThreadShadow> {
    if thread == 0 {
        return None;
    }
    let start = thread_slot_index(thread);
    for i in 0..PROBE_LIMIT {
        let e = &VIBEOS_CSWITCHDEP.threads[(start + i) & (CSWITCH_THREADS - 1)];
        let k = e.kthread.load(Relaxed);
        if k == thread {
            return Some(e);
        }
        if k == 0 {
            if !claim {
                return None;
            }
            match e.kthread.compare_exchange(0, thread, Relaxed, Relaxed) {
                Ok(_) => {
                    // Fresh claim: make sure no stale content survives from a
                    // previous occupant (free-then-reclaim of the same slot).
                    e.suspend_irql.store(0, Relaxed);
                    e.gen.store(0, Relaxed);
                    return Some(e);
                }
                Err(actual) if actual == thread => return Some(e),
                Err(_) => continue, // raced a different key — keep probing
            }
        }
    }
    if claim {
        VIBEOS_CSWITCHDEP.dropped.fetch_add(1, Relaxed);
    }
    None
}

/// Current IRQL, portably. amd64: CR8 is FORCEINLINE-only in C, read it
/// directly; i386/arm64: the HAL exports the real symbol.
#[inline]
unsafe fn current_irql() -> u8 {
    #[cfg(target_arch = "x86_64")]
    {
        ntkrnl_sys::inline::ke::KeGetCurrentIrql_amd64() as u8
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        ntkrnl_sys::ddk::KeGetCurrentIrql() as u8
    }
}

fn record_violation(kind: u32, pc: usize, old: usize, new: usize, expected: u64, actual: u64) {
    let seq = VIBEOS_CSWITCHDEP.violation_count.fetch_add(1, Relaxed) + 1;
    let v = &VIBEOS_CSWITCHDEP.violations[((seq - 1) as usize) % CSWITCH_VIOLATIONS];
    v.kind.store(kind, Relaxed);
    v.pc.store(pc, Relaxed);
    v.kthread_old.store(old, Relaxed);
    v.kthread_new.store(new, Relaxed);
    v.expected.store(expected, Relaxed);
    v.actual.store(actual, Relaxed);
    v.seq.store(seq, Release); // last: decoders treat mismatched seq as torn
}

fn kind_name(k: u32) -> *const u8 {
    match k {
        KIND_HIGH_IRQL_AT_SWITCH => b"HIGH_IRQL_AT_SWITCH\0".as_ptr(),
        KIND_IRQL_UNBALANCED_SWITCH => b"IRQL_UNBALANCED_SWITCH\0".as_ptr(),
        KIND_SWAP_IRQL => b"SWAP_IRQL\0".as_ptr(),
        KIND_SWITCH_FROM_DPC => b"SWITCH_FROM_DPC\0".as_ptr(),
        KIND_CURTHREAD_INCOHERENT => b"CURTHREAD_INCOHERENT\0".as_ptr(),
        KIND_STACK_OOB => b"STACK_OOB\0".as_ptr(),
        KIND_SLEEP_WITH_SPINLOCK => b"SLEEP_WITH_SPINLOCK\0".as_ptr(),
        KIND_DPC_DISPATCH_IRQL => b"DPC_DISPATCH_IRQL\0".as_ptr(),
        KIND_INTR_STATE_UNBALANCED => b"INTR_STATE_UNBALANCED\0".as_ptr(),
        _ => b"?\0".as_ptr(),
    }
}

/// Loud path, print-capped: the census prints totals; this names the first
/// PRINT_CAP culprits live (a §9.24-class leak may never reach the census).
fn loud(kind: u32, pc: usize, old: usize, new: usize, expected: u64, actual: u64) {
    record_violation(kind, pc, old, new, expected, actual);
    if PRINTS.fetch_add(1, Relaxed) < PRINT_CAP {
        crate::dbg_print!(
            b"[CSWITCH] %s old=%p new=%p expected=%u actual=%u pc=%p\n\0",
            kind_name(kind),
            old as *const (),
            new as *const (),
            expected as u32,
            actual as u32,
            pc as *const (),
        );
    }
}

// ============================================================================
// Taps (design doc "Instrumentation points"). All #[inline(never)] so
// llvm.returnaddress(0) inside them is the tap CALL SITE.
// ============================================================================

/// Birth brand: `KiInitializeContextThread`. Claims (or re-claims) the slot so
/// a pool-reused KTHREAD address never inherits a dead thread's shadow, and so
/// tap 2 can tell a first-run from a lost record.
#[inline(never)]
pub fn cswitch_thread_init(thread: usize) {
    if let Some(e) = thread_slot(thread, true) {
        e.suspend_irql.store(0, Relaxed);
        e.gen.store(0, Relaxed);
    }
}

/// Tap-1 entry: `KiSwapThreadImpl`, immediately before `KiSwapContext`.
/// `wait_irql` = the KTHREAD.WaitIrql the caller will restore after the swap;
/// `terminating` = the old thread's State is Terminated (final switch away —
/// evict its shadow).
#[inline(never)]
pub unsafe fn cswitch_suspend(old: usize, wait_irql: u8, terminating: bool) {
    let pc = super::lockdep::return_address0();
    VIBEOS_CSWITCHDEP.cpu.switches.fetch_add(1, Relaxed);

    let irql = current_irql();
    VIBEOS_CSWITCHDEP.cpu.switch_irql.store(irql as u32, Relaxed);
    // UP kernel: SYNCH_LEVEL == DISPATCH_LEVEL, so the switching level is
    // exactly DISPATCH. >DISPATCH is the leaked-raise class (§9.24 lead:
    // HIGH_LEVEL never lowered); <DISPATCH is a broken caller.
    if irql != DISPATCH_LEVEL {
        loud(KIND_HIGH_IRQL_AT_SWITCH, pc, old, 0, DISPATCH_LEVEL as u64, irql as u64);
    }

    // P2: no dispatch-level lock may be held across a voluntary wait,
    // measured as a DELTA above a SELF-CALIBRATING floor (mixed C-inline/
    // Rust-funnel pairs make the absolute count unusable — see CpuShadow).
    // Every clean voluntary switch proves the true floor: ratchet the
    // baseline down to the minimum ever observed here; a count above that
    // floor at a wait means a Rust-funnel-acquired lock is still held. The
    // measured drift is purely negative (C-inline acquire → Rust release), so
    // the floor converges and stays; if a positive mixed pair ever appears
    // (Rust acquire → C-inline release) this will fire persistently — which
    // is itself the signal to go find that pair. amd64-only (fullest funnel
    // coverage); on i386/arm64 the counter is census data, never a violation.
    #[cfg(target_arch = "x86_64")]
    {
        let held = VIBEOS_CSWITCHDEP.cpu.held_spinlocks.load(Relaxed);
        let base = VIBEOS_CSWITCHDEP.cpu.held_baseline.load(Relaxed);
        if held < base {
            VIBEOS_CSWITCHDEP.cpu.held_baseline.store(held, Relaxed);
        } else if held > base {
            loud(KIND_SLEEP_WITH_SPINLOCK, pc, old, 0, base as u64, held as u64);
        }
    }

    if terminating {
        // Final switch away from a dying thread: free the slot so the pool
        // address can be reused by a future KTHREAD without inheritance.
        if let Some(e) = thread_slot(old, false) {
            e.suspend_irql.store(0, Relaxed);
            e.gen.store(0, Relaxed);
            e.kthread.store(0, Relaxed);
            VIBEOS_CSWITCHDEP.cpu.evictions.fetch_add(1, Relaxed);
        }
        return;
    }

    if let Some(e) = thread_slot(old, true) {
        e.suspend_irql.store(wait_irql as u32, Relaxed);
        let g = e.gen.load(Relaxed);
        e.gen.store(g.wrapping_add(1).max(1), Release); // publish-last; never back to 0
    }
}

/// Tap-1 exit: `KiSwapThreadImpl`, after the caller's `KeLowerIrql(WaitIrql)`.
/// The live IRQL must now be exactly the thread's saved WaitIrql — anything
/// else means the raise/lower pairing across the wait is broken.
#[inline(never)]
pub unsafe fn cswitch_after_switch(thread: usize, wait_irql: u8) {
    let irql = current_irql();
    if irql != wait_irql {
        let pc = super::lockdep::return_address0();
        loud(KIND_IRQL_UNBALANCED_SWITCH, pc, thread, 0, wait_irql as u64, irql as u64);
    }
}

/// Tap 2: `KiSwapContextResume` (post-swap, on the NEW thread's stack).
/// `prcb_current` = `Prcb.CurrentThread` as the resume code sees it;
/// `sp` = an address on the current stack (a local's address in the caller);
/// `stack_limit`/`stack_base` = the new thread's kernel stack bounds
/// (`KTHREAD.StackLimit` / `KTHREAD.InitialStack`);
/// `dpc_active` = `Prcb.DpcRoutineActive != 0` (recorded here BEFORE the
/// existing ATTEMPTED_SWITCH_FROM_DPC BugCheck fires, so the table + serial
/// carry the record into triage).
#[inline(never)]
pub unsafe fn cswitch_swap_resume(
    old: usize,
    new: usize,
    prcb_current: usize,
    sp: usize,
    stack_limit: usize,
    stack_base: usize,
    dpc_active: bool,
) {
    let pc = super::lockdep::return_address0();

    // The swap machinery never changes IRQL: post-swap we must still be at the
    // switching level (DISPATCH). The caller lowers to WaitIrql AFTER.
    let irql = current_irql();
    if irql != DISPATCH_LEVEL {
        loud(KIND_SWAP_IRQL, pc, old, new, DISPATCH_LEVEL as u64, irql as u64);
    }

    if dpc_active {
        loud(KIND_SWITCH_FROM_DPC, pc, old, new, 0, 1);
    }

    // Post-switch coherence: the per-CPU chain must already name the new
    // thread (the offset-drift / stale-CurrentThread class).
    if prcb_current != new {
        loud(KIND_CURTHREAD_INCOHERENT, pc, old, new, new as u64, prcb_current as u64);
    }

    // We are on the new thread's kernel stack: (StackLimit, InitialStack].
    if stack_limit != 0 && stack_base != 0 && !(sp > stack_limit && sp <= stack_base) {
        loud(KIND_STACK_OOB, pc, old, new, stack_base as u64, sp as u64);
    }

    // First-run / missing-record accounting (normal, never a violation):
    // idle threads switch away via KiSwapContext directly (no tap 1), and
    // unbranded arches have no birth record.
    match thread_slot(new, false) {
        None => {
            VIBEOS_CSWITCHDEP.cpu.resume_no_record.fetch_add(1, Relaxed);
        }
        Some(e) => {
            if e.gen.load(Relaxed) == 0 {
                VIBEOS_CSWITCHDEP.cpu.first_runs.fetch_add(1, Relaxed);
            }
        }
    }
}

// ============================================================================
// P2 spinlock taps. The acquire/release pair rides the kernel Kx choke point
// (ntkrnl-sys inline/ke.rs KxAcquireSpinLock/KxReleaseSpinLock — every
// ntoskrnl spinlock funnel delegates there; link-name externs mirror the
// lockdep pushlock taps). The try funnel (KeTryToAcquireSpinLockAtDpcLevel)
// succeeds without touching Kx on UP, so it calls spin_acquired() directly;
// its release DOES route through Kx, keeping the pair balanced.
// ============================================================================

#[no_mangle]
pub extern "C" fn VibeosCswitchdepSpinAcquire(_lock: usize) {
    VIBEOS_CSWITCHDEP.cpu.held_spinlocks.fetch_add(1, Relaxed);
    VIBEOS_CSWITCHDEP.cpu.spin_acquires.fetch_add(1, Relaxed);
}

#[no_mangle]
pub extern "C" fn VibeosCswitchdepSpinRelease(_lock: usize) {
    VIBEOS_CSWITCHDEP.cpu.held_spinlocks.fetch_sub(1, Relaxed);
}

/// Direct-tap form for the try-acquire funnel (no Kx on the UP success path).
#[inline]
pub fn spin_acquired(lock: usize) {
    VibeosCswitchdepSpinAcquire(lock);
}

/// P2 tap: deferred-dispatch delivery (`KiDispatchInterrupt` /
/// `KiDispatchSoftwareIntArm64`) must run at exactly DISPATCH_LEVEL.
#[inline(never)]
pub unsafe fn cswitch_dpc_dispatch() {
    let irql = current_irql();
    if irql != DISPATCH_LEVEL {
        let pc = super::lockdep::return_address0();
        loud(KIND_DPC_DISPATCH_IRQL, pc, 0, 0, DISPATCH_LEVEL as u64, irql as u64);
    }
}

/// P2 tap (arm64): interrupt-mask state must be balanced across the software
/// interrupt dispatch — `entry`/`exit` are the DAIF values around the body.
#[inline(never)]
pub unsafe fn cswitch_intr_state(entry: u64, exit: u64) {
    if entry != exit {
        let pc = super::lockdep::return_address0();
        loud(KIND_INTR_STATE_UNBALANCED, pc, 0, 0, entry, exit);
    }
}

/// Census helper (PASSIVE): print the switch census + any recorded violations.
pub fn cswitchdep_report() {
    let total = VIBEOS_CSWITCHDEP.violation_count.load(Relaxed);
    crate::dbg_print!(
        b"[CSWITCH] census: switches=%u violations=%u first_runs=%u no_record=%u evictions=%u drops=%u spin_acq=%u held_now=%d base=%d\n\0",
        VIBEOS_CSWITCHDEP.cpu.switches.load(Relaxed) as u32,
        total as u32,
        VIBEOS_CSWITCHDEP.cpu.first_runs.load(Relaxed),
        VIBEOS_CSWITCHDEP.cpu.resume_no_record.load(Relaxed),
        VIBEOS_CSWITCHDEP.cpu.evictions.load(Relaxed),
        VIBEOS_CSWITCHDEP.dropped.load(Relaxed),
        VIBEOS_CSWITCHDEP.cpu.spin_acquires.load(Relaxed) as u32,
        VIBEOS_CSWITCHDEP.cpu.held_spinlocks.load(Relaxed),
        VIBEOS_CSWITCHDEP.cpu.held_baseline.load(Relaxed),
    );
    if total == 0 {
        return;
    }
    let shown = core::cmp::min(total, CSWITCH_VIOLATIONS as u64);
    for i in 0..shown {
        let v = &VIBEOS_CSWITCHDEP.violations[i as usize];
        let seq = v.seq.load(Relaxed);
        if seq == 0 {
            continue;
        }
        crate::dbg_print!(
            b"[CSWITCH]   #%u %s old=%p new=%p expected=%u actual=%u pc=%p\n\0",
            seq as u32,
            kind_name(v.kind.load(Relaxed)),
            v.kthread_old.load(Relaxed) as *const (),
            v.kthread_new.load(Relaxed) as *const (),
            v.expected.load(Relaxed) as u32,
            v.actual.load(Relaxed) as u32,
            v.pc.load(Relaxed) as *const (),
        );
    }
}

/// Total recorded violations (for the census verdict / smoke triage).
pub fn cswitchdep_violation_count() -> u64 {
    VIBEOS_CSWITCHDEP.violation_count.load(Relaxed)
}
