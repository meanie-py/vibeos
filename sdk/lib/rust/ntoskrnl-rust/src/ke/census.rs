//! census — wedge self-census watchdog (sync-instrumentation §2).
//!
//! A kernel `khungtaskd` analog: a low-priority system thread wakes every 10s,
//! walks every process/thread, and reports any thread parked in **GateWait on
//! the same object across two consecutive scans** (i.e. stuck 10–20s+ — gates
//! are hand-off primitives; a healthy gate wait is microseconds). Each report
//! line carries the `synctrace` (§3) counters for the stuck object, so a wedge
//! self-diagnoses in the smoke serial log as lost-wakeup (`sig >= parked` yet
//! still parked) vs holder-stuck (`sig == 0`) — the discrimination that
//! previously cost a GDB session per occurrence (field-manual §9, the amd64
//! Mm-pushlock service-startup wedge).
//!
//! Safety stance: this walks live lists WITHOUT locks — on the wedged system
//! it exists for, the very locks that would make the walk safe may be the
//! things that are stuck. Every pointer is range/alignment-checked before
//! deref and every loop is capped; a torn read produces one garbage report
//! line, not a hang. During the target scenario (a wedge) nothing is exiting,
//! so the list-race window is moot.
//!
//! **Coverage was widened past `GateWait` (2026-07-28).** The arm64 blocker's
//! "face D" is a stalled test sweep — 180s timeouts throughout — during which
//! this watchdog printed exactly one line: its own startup banner. That was
//! read as *nothing is stuck*, when the only honest reading was *nothing is
//! stuck **in a gate wait***, which is all the scan ever looked at. So face C
//! (holder-stuck, reported) and face D (invisible) were being told apart by the
//! ABSENCE of a line — a discriminator resting entirely on this file's blind
//! spot. It now tracks every parked thread with its `State`/`WaitReason`,
//! names the ones on kernel-internal locks, and prints a wait-reason histogram
//! when the parked profile shifts.
//!
//! Silent when healthy: gate waits and kernel-lock waits report only when stuck
//! across two scans, and the histogram prints only when the parked-wait profile
//! *changes* — a healthy system parks a large but stable population of server
//! and worker threads forever, and that steady state is silence.

#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(static_mut_refs)] // PREV/PREV_COUNT: single-writer (census thread only)

use core::mem::offset_of;
// PsActiveProcessHead: ps-rust global read across the crate boundary as a link symbol
// (#[no_mangle] in ps-rust). crate-split Phase 1b.
extern "C" {
    static mut PsActiveProcessHead: ntkrnl_sys::ddk::_LIST_ENTRY;
}
use core::sync::atomic::{AtomicU32, Ordering::Relaxed};

use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::{
    HANDLE, KeDelayExecutionThread, PsCreateSystemThread, ZwClose, _EPROCESS, _ETHREAD, _KTHREAD,
};

// 0x1F03FF = STANDARD_RIGHTS_REQUIRED|SYNCHRONIZE|0x3FF. xdk/pstypes.h picks this
// under `#if (NTDDI_VERSION >= NTDDI_VISTA) ... #else`, and this tree builds
// -D_WIN32_WINNT=0x502 (Server 2003), so the PRE-Vista branch is the live one. The
// Vista value 0x1FFFFF asks for 0xFC00 of thread rights the object's GENERIC_MAPPING
// does not grant, which an open would refuse rather than silently narrow.
const THREAD_ALL_ACCESS: u32 = 0x001F_03FF;

use super::ke_defs::{kt_State, kt_WaitTime};
use super::synctrace::{synctrace_dropped, synctrace_lookup};

/// KTHREAD_STATE::GateWait (see ke/gate.rs).
const GateWait: u8 = 8;
/// KTHREAD_STATE::Waiting — every non-gate wait. Face D is somewhere in here.
const Waiting: u8 = 5;

const SCAN_PERIOD_100NS: i64 = -10 * 10_000_000; // relative 10s
const MAX_PROCESSES: usize = 512;
const MAX_THREADS_PER_PROCESS: usize = 2048;
/// Sightings carried between scans. Was 128 while only GateWait threads were
/// tracked; a booted desktop plus an 89-test sweep parks a few hundred threads
/// in ordinary waits, so the wide census needs the headroom. Overflow is
/// counted and reported, never silent.
const MAX_SIGHTINGS: usize = 512;
/// `MaximumWaitReason`. Anything at or above this is a torn read, not a reason.
const MAX_WAIT_REASON: usize = 37;
/// Per-thread lines emitted per scan for the wide (non-gate) class. The
/// histogram carries the totals, so this only has to name a few examples.
const MAX_WIDE_LINES: u32 = 8;
/// Lifetime cap on histogram prints — a stall lasting hours must not bury the
/// serial log the histogram exists to inform.
const MAX_HIST_PRINTS: u32 = 200;
/// Minimum scans between histogram prints (×10s cadence).
const HIST_MIN_SCAN_GAP: u32 = 3;

#[derive(Clone, Copy, PartialEq)]
struct Sighting {
    thread: usize,
    object: usize,
    state: u8,
    reason: u8,
    /// `KTHREAD.WaitTime` — the tick the wait began. This is what makes the
    /// key mean "still in the SAME wait" rather than "waiting again": a thread
    /// that wakes, works, and re-waits on the same object is indistinguishable
    /// from a wedged one by (thread, object) alone, and would otherwise be
    /// reported as stuck forever.
    wait_time: u32,
}

impl Sighting {
    const EMPTY: Sighting =
        Sighting { thread: 0, object: 0, state: 0, reason: 0, wait_time: 0 };
}

/// Single-writer state (only the census thread touches these).
///
/// Both buffers are `static`, not locals: at 512 sightings the pair is ~16 KB,
/// and the census runs on an ordinary system-thread stack.
static mut PREV: [Sighting; MAX_SIGHTINGS] = [Sighting::EMPTY; MAX_SIGHTINGS];
static mut PREV_COUNT: usize = 0;
static mut NEXT: [Sighting; MAX_SIGHTINGS] = [Sighting::EMPTY; MAX_SIGHTINGS];
static mut NEXT_COUNT: usize = 0;
/// Last printed wait-reason histogram, and the scan it was printed on. The
/// census stays SILENT while this profile holds steady and speaks when it
/// shifts — so a healthy boot prints nothing, and threads piling into one
/// reason (face D's signature) prints once as they pile in.
static mut LAST_HIST: [u16; MAX_WAIT_REASON] = [0; MAX_WAIT_REASON];
static mut LAST_HIST_SCAN: u32 = 0;
static HIST_PRINTS: AtomicU32 = AtomicU32::new(0);
static SIGHTING_OVERFLOW: AtomicU32 = AtomicU32::new(0);
/// Exported heartbeat for GDB/debug-block: scans completed since boot.
#[no_mangle]
pub static VIBEOS_CENSUS_SCANS: AtomicU32 = AtomicU32::new(0);

/// Plausible kernel pointer for this arch, aligned to `align`.
#[inline]
fn plausible(p: usize, align: usize) -> bool {
    if p == 0 || p & (align - 1) != 0 {
        return false;
    }
    #[cfg(target_pointer_width = "64")]
    {
        p >= 0xFFFF_0000_0000_0000
    }
    #[cfg(target_pointer_width = "32")]
    {
        p >= 0x8000_0000
    }
}

/// `KWAIT_REASON` names, indexed by the enum value. Printing `reason=28` and
/// making the reader go find the enum is how face D stayed unnamed this long.
static REASON_NAMES: [&[u8]; MAX_WAIT_REASON] = [
    b"Executive\0",
    b"FreePage\0",
    b"PageIn\0",
    b"PoolAllocation\0",
    b"DelayExecution\0",
    b"Suspended\0",
    b"UserRequest\0",
    b"WrExecutive\0",
    b"WrFreePage\0",
    b"WrPageIn\0",
    b"WrPoolAllocation\0",
    b"WrDelayExecution\0",
    b"WrSuspended\0",
    b"WrUserRequest\0",
    b"WrEventPair\0",
    b"WrQueue\0",
    b"WrLpcReceive\0",
    b"WrLpcReply\0",
    b"WrVirtualMemory\0",
    b"WrPageOut\0",
    b"WrRendezvous\0",
    b"WrKeyedEvent\0",
    b"WrTerminated\0",
    b"WrProcessInSwap\0",
    b"WrCpuRateControl\0",
    b"WrCalloutStack\0",
    b"WrKernel\0",
    b"WrResource\0",
    b"WrPushLock\0",
    b"WrMutex\0",
    b"WrQuantumEnd\0",
    b"WrDispatchInt\0",
    b"WrPreempted\0",
    b"WrYieldExecution\0",
    b"WrFastMutex\0",
    b"WrGuardedMutex\0",
    b"WrRundown\0",
];

#[inline]
fn reason_name(r: u8) -> *const u8 {
    let i = r as usize;
    if i < MAX_WAIT_REASON { REASON_NAMES[i].as_ptr() } else { b"?\0".as_ptr() }
}

/// Wait reasons that should never last 20 seconds.
///
/// These are kernel-internal lock/resource waits — microseconds to
/// milliseconds in a healthy system, so a thread still parked on one across
/// two scans is a wedge by definition. Everything NOT on this list (WrQueue
/// worker pools, WrLpcReceive server threads, UserRequest message loops,
/// DelayExecution sleeps) parks indefinitely by design and would be pure noise
/// as a per-thread report — those are counted in the histogram instead, where
/// a *shift* in the profile is the signal rather than the parking itself.
///
/// `Executive`(0) and `WrExecutive`(7) are deliberately NOT here, though they
/// look like they belong. They are NT's *generic* wait reason — the default for
/// `KeWaitForSingleObject` — so ordinary System and service threads park on
/// them forever by design. Including them was measured on the first soak and
/// produced 100% false positives: every one of 126 `[CENSUS] STUCK` lines in an
/// otherwise-clean amd64 round was `reason=Executive`, from `System` and
/// `lsass.exe`. They belong in the histogram, which is exactly where a large
/// stable population of generic waiters is informative rather than alarming.
#[inline]
fn is_kernel_lock_wait(r: u8) -> bool {
    matches!(
        r,
        2   // PageIn
        | 3 // PoolAllocation
        | 9 // WrPageIn
        | 10 // WrPoolAllocation
        | 18 // WrVirtualMemory
        | 20 // WrRendezvous
        | 21 // WrKeyedEvent
        | 25 // WrCalloutStack
        | 26 // WrKernel
        | 27 // WrResource
        | 28 // WrPushLock
        | 29 // WrMutex
        | 34 // WrFastMutex
        | 35 // WrGuardedMutex
    )
}

/// One full process/thread scan. Returns the new sighting set in `next`,
/// prints a report line for every second-consecutive sighting.
unsafe fn scan(next: &mut [Sighting; MAX_SIGHTINGS], next_count: &mut usize) {
    let head = core::ptr::addr_of_mut!(PsActiveProcessHead);
    let mut plink = (*head).Flink;
    let mut nprocs = 0usize;
    let mut reported = 0u32;
    // Wide (non-gate) census state for this scan.
    let mut hist = [0u16; MAX_WAIT_REASON];
    let mut wide_reported = 0u32;
    let mut wide_lines = 0u32;

    while plausible(plink as usize, core::mem::size_of::<usize>())
        && plink != head
        && nprocs < MAX_PROCESSES
    {
        nprocs += 1;
        let eproc = ((plink as usize) - offset_of!(_EPROCESS, ActiveProcessLinks)) as *mut _EPROCESS;

        // Thread list: KPROCESS.ThreadListHead links KTHREAD.ThreadListEntry.
        let thead = core::ptr::addr_of_mut!((*eproc).Pcb.ThreadListHead);
        let mut tlink = (*thead).Flink;
        let mut nthreads = 0usize;

        while plausible(tlink as usize, core::mem::size_of::<usize>())
            && tlink != thead
            && nthreads < MAX_THREADS_PER_PROCESS
        {
            nthreads += 1;
            let kthread =
                ((tlink as usize) - offset_of!(_KTHREAD, ThreadListEntry)) as *mut _KTHREAD;

            let state = *kt_State(kthread);
            if state == GateWait || state == Waiting {
                // A gate wait keys on its GateObject, as it always has. A plain
                // wait has no single object to key on, so WaitTime carries the
                // identity of the wait instead.
                let object = if state == GateWait {
                    (*kthread).__bindgen_anon_2.GateObject as usize
                } else {
                    0
                };
                let reason = (*kthread).WaitReason as u8;
                let s = Sighting {
                    thread: kthread as usize,
                    object,
                    state,
                    reason,
                    wait_time: *kt_WaitTime(kthread),
                };

                // Second consecutive sighting => same wait, still parked.
                let mut stuck = false;
                for i in 0..PREV_COUNT {
                    if PREV[i] == s {
                        stuck = true;
                        break;
                    }
                }
                if stuck {
                    if state == GateWait {
                        report(eproc, kthread, object);
                        reported += 1;
                    } else {
                        // Histogram every repeat; name only the kernel-lock
                        // ones, and only a few per scan.
                        if (reason as usize) < MAX_WAIT_REASON {
                            hist[reason as usize] = hist[reason as usize].saturating_add(1);
                        }
                        if is_kernel_lock_wait(reason) {
                            wide_reported += 1;
                            if wide_lines < MAX_WIDE_LINES {
                                wide_lines += 1;
                                report_wide(eproc, kthread, state, reason, s.wait_time);
                            }
                        }
                    }
                }
                if *next_count < MAX_SIGHTINGS {
                    next[*next_count] = s;
                    *next_count += 1;
                } else {
                    SIGHTING_OVERFLOW.fetch_add(1, Relaxed);
                }
            }

            tlink = (*tlink).Flink;
        }

        plink = (*plink).Flink;
    }

    let scan_no = VIBEOS_CENSUS_SCANS.load(Relaxed);

    if reported != 0 || wide_reported != 0 {
        crate::dbg_print!(
            b"[CENSUS] %u GateWait + %u kernel-lock thread(s) stuck >10s (scan #%u, synctrace dropped=%u, sighting overflow=%u)\n\0",
            reported,
            wide_reported,
            scan_no,
            synctrace_dropped(),
            SIGHTING_OVERFLOW.load(Relaxed),
        );
    }

    // The face-D net. Face D is a stalled sweep with ZERO census output, which
    // told us only that nothing was in a GATE wait — the watchdog's own banner
    // said "gate waits". So the discriminator against face C was the ABSENCE of
    // a line, i.e. not-looking dressed up as looked-and-clean (§9.122).
    //
    // Print the whole parked-wait profile, but only when it SHIFTS: a healthy
    // system parks a stable population of server/worker threads forever, so a
    // steady profile is silence and threads piling into one reason is a line.
    let mut total = 0u32;
    let mut shifted = false;
    for i in 0..MAX_WAIT_REASON {
        total += hist[i] as u32;
        let (a, b) = (hist[i], LAST_HIST[i]);
        let delta = if a > b { a - b } else { b - a };
        // A new reason appearing at all is a shift; otherwise it takes a real
        // pile-up, so ordinary churn does not chatter.
        if delta >= 4 || (a != 0 && b == 0) {
            shifted = true;
        }
    }
    if (reported != 0 || wide_reported != 0 || (shifted && total >= 4))
        && scan_no.wrapping_sub(LAST_HIST_SCAN) >= HIST_MIN_SCAN_GAP
        && HIST_PRINTS.load(Relaxed) < MAX_HIST_PRINTS
    {
        crate::dbg_print!(
            b"[CENSUS] wait profile: %u thread(s) parked >20s (scan #%u)\n\0",
            total,
            scan_no,
        );
        for i in 0..MAX_WAIT_REASON {
            if hist[i] != 0 {
                crate::dbg_print!(
                    b"[CENSUS]   %s=%u\n\0",
                    reason_name(i as u8),
                    hist[i] as u32,
                );
            }
        }
        LAST_HIST = hist;
        LAST_HIST_SCAN = scan_no;
        HIST_PRINTS.fetch_add(1, Relaxed);
    }

    // lockdep §4: surface any recorded self-deadlock, whether or not the census
    // caught a >10s GateWait this scan — the violation IS the wedge's smoking
    // gun and fires at the park, deterministically, before the >10s window.
    #[cfg(feature = "lockdep")]
    super::lockdep::lockdep_report();

    // cswitchdep: switch census + any hand-off violations (same rationale —
    // the violation fires at the switch, before any downstream wedge/crash).
    #[cfg(feature = "cswitchdep")]
    super::cswitchdep::cswitchdep_report();
}

/// One stuck-thread report line, with the synctrace verdict inputs inline.
unsafe fn report(eproc: *mut _EPROCESS, kthread: *mut _KTHREAD, object: usize) {
    // ImageFileName is CHAR[16], usually but not guaranteedly NUL-terminated.
    let mut img = [0u8; 17];
    let src = core::ptr::addr_of!((*eproc).ImageFileName) as *const u8;
    core::ptr::copy_nonoverlapping(src, img.as_mut_ptr(), 16);
    let tid = (*(kthread as *mut _ETHREAD)).Cid.UniqueThread as usize;
    let pid = (*eproc).UniqueProcessId as usize;

    match synctrace_lookup(object) {
        Some((parked, satisfied, signals, _lw, last_signaller)) => {
            crate::dbg_print!(
                b"[CENSUS] STUCK %s pid=%u tid=%u thr=%p GateWait obj=%p parked=%u sat=%u sig=%u last_sig_thr=%p verdict=%s\n\0",
                img.as_ptr(),
                pid as u32,
                tid as u32,
                kthread,
                object as *const (),
                parked,
                satisfied,
                signals,
                last_signaller as *const (),
                // parked > satisfied is implied by the thread being parked;
                // signals distinguish the failure family.
                if signals == 0 {
                    b"HOLDER-STUCK(no signal ever)\0".as_ptr()
                } else if signals >= parked {
                    b"LOST-WAKEUP(signals>=parked)\0".as_ptr()
                } else {
                    b"UNDER-SIGNALLED\0".as_ptr()
                },
            );
            super::flightrec::flightrec_dump_object(object, 8);

            // HOLDER attribution (heuristic, 2026-07-14): >10s GateWaits here
            // are almost always guarded-mutex gates — if so the KGATE is
            // embedded at KGUARDED_MUTEX.Gate and the wedged HOLDER is .Owner.
            // Print the candidate owner + its state so a wedge names the
            // holder, not just its victims. `gm?` marks the heuristic: a bare
            // KeWaitForGate user makes these fields garbage — sanity-gated on
            // the owner looking like an aligned kernel pointer.
            // PUSHLOCK attribution — the one the arm64 §9.97 soak actually needed.
            //
            // The only KeWaitForGate callers in the tree are ke/wait.rs and
            // ex-rust/pushlock.rs, so a >10s GateWait is in practice a pushlock
            // wait, and `HOLDER-STUCK(no signal ever)` means a pushlock holder
            // never released. WHICH lock was the missing fact: pushlock.rs
            // deliberately keys its synctrace on the PUSHLOCK address ("the wake
            // gate is a per-wait stack object — useless as a key"), while this
            // report prints only that stack-allocated gate. The identity was
            // recorded and never surfaced.
            //
            // Recoverable in one read: EX_PUSH_LOCK_WAIT_BLOCK's first field is
            // the union holding WakeGate, so the gate address IS the wait-block
            // address, and .PushLock is a field of it. Same sanity gate as the
            // gm? heuristic below — a bare KeWaitForGate user makes this garbage.
            // Deliberately NOT x86_64-gated: the gm? block below is, which is why
            // arm64 — the arch with the bug — has had no holder attribution at all.
            if (*kthread).WaitReason as u32 == 28
            /* _KWAIT_REASON_WrPushLock */
            {
                use ntkrnl_sys::ddk::_EX_PUSH_LOCK_WAIT_BLOCK;
                let wb = (object - offset_of!(_EX_PUSH_LOCK_WAIT_BLOCK, __bindgen_anon_1))
                    as *const _EX_PUSH_LOCK_WAIT_BLOCK;
                let pl = core::ptr::read_volatile(core::ptr::addr_of!((*wb).PushLock)) as usize;
                // Use plausible(), not a hand-rolled range: a 64-bit literal does
                // not fit i386's usize (CARGO_CHECK_FAIL), and a pushlock is
                // pointer-aligned — 4 bytes there, not 8.
                if plausible(pl, core::mem::size_of::<usize>()) {
                    // dbg_print! evaluates to u32, so each arm must be a
                    // statement block or the match takes its value.
                    match synctrace_lookup(pl) {
                        Some((p_parked, p_sat, p_sig, _, p_last)) => {
                            crate::dbg_print!(
                                b"[CENSUS]   pushlock=%p parked=%u sat=%u sig=%u last_sig_thr=%p\n\0",
                                pl as *const (),
                                p_parked,
                                p_sat,
                                p_sig,
                                p_last as *const (),
                            );
                        }
                        None => {
                            crate::dbg_print!(
                                b"[CENSUS]   pushlock=%p (no synctrace entry)\n\0",
                                pl as *const (),
                            );
                        }
                    }
                }
            }

            #[cfg(all(feature = "smp", target_arch = "x86_64"))]
            {
                use ntkrnl_sys::ddk::_KGUARDED_MUTEX;
                let gate_off = core::mem::offset_of!(_KGUARDED_MUTEX, Gate);
                let gm = (object - gate_off) as *const _KGUARDED_MUTEX;
                let owner = core::ptr::read_volatile(core::ptr::addr_of!((*gm).Owner));
                if (owner as u64) >= 0xFFFF_8000_0000_0000 && ((owner as usize) & 7) == 0 {
                    crate::dbg_print!(
                        b"[CENSUS]   gm? owner=%p state=%u waitreason=%u count=%d contention=%u\n\0",
                        owner,
                        ntkrnl_sys::inline::ke::kt_state_read(owner) as u32,
                        (*owner).WaitReason as u32,
                        core::ptr::read_volatile(core::ptr::addr_of!((*gm).Count)),
                        core::ptr::read_volatile(core::ptr::addr_of!((*gm).Contention)),
                    );
                }
            }
        }
        None => {
            crate::dbg_print!(
                b"[CENSUS] STUCK %s pid=%u tid=%u thr=%p GateWait obj=%p (no synctrace entry)\n\0",
                img.as_ptr(),
                pid as u32,
                tid as u32,
                kthread,
                object as *const (),
            );
        }
    }
}

/// One stuck-thread line for the wide (non-gate) class.
///
/// Deliberately thinner than `report()`: there is no synctrace entry to look up
/// for an arbitrary wait, so this names WHO and WHAT-KIND. That is the fact
/// face D was missing — "77 tests timed out and the census saw nothing" becomes
/// "these threads are parked on WrFastMutex".
unsafe fn report_wide(
    eproc: *mut _EPROCESS,
    kthread: *mut _KTHREAD,
    state: u8,
    reason: u8,
    wait_time: u32,
) {
    let mut img = [0u8; 17];
    let src = core::ptr::addr_of!((*eproc).ImageFileName) as *const u8;
    core::ptr::copy_nonoverlapping(src, img.as_mut_ptr(), 16);
    let tid = (*(kthread as *mut _ETHREAD)).Cid.UniqueThread as usize;
    let pid = (*eproc).UniqueProcessId as usize;

    crate::dbg_print!(
        b"[CENSUS] STUCK %s pid=%u tid=%u thr=%p state=%u reason=%s(%u) waittick=%u\n\0",
        img.as_ptr(),
        pid as u32,
        tid as u32,
        kthread,
        state as u32,
        reason_name(reason),
        reason as u32,
        wait_time,
    );
}

unsafe extern "system" fn CensusThread(_ctx: PVOID) {
    let mut interval: LARGE_INTEGER = core::mem::zeroed();
    interval.QuadPart = SCAN_PERIOD_100NS;

    loop {
        KeDelayExecutionThread(0, 0, core::ptr::addr_of_mut!(interval) as *mut _);

        // NEXT/PREV are statics, not locals: 512 sightings is ~16 KB per
        // buffer and this runs on an ordinary system-thread stack.
        NEXT_COUNT = 0;
        scan(&mut *core::ptr::addr_of_mut!(NEXT), &mut *core::ptr::addr_of_mut!(NEXT_COUNT));
        PREV = NEXT;
        PREV_COUNT = NEXT_COUNT;
        VIBEOS_CENSUS_SCANS.fetch_add(1, Relaxed);
    }
}

/// Start the watchdog. Called once from `ExpInitSystemPhase1` (system-thread
/// context, dispatcher fully up). Failure is non-fatal but LOUD.
#[no_mangle]
pub unsafe extern "system" fn KiStartSyncCensus() {
    super::synctrace::synctrace_init();
    super::flightrec::flightrec_init();
    #[cfg(feature = "lockdep")]
    super::lockdep::lockdep_init();
    #[cfg(feature = "cswitchdep")]
    super::cswitchdep::cswitchdep_init();
    // fw_cfg-armed detector selftest (CSWITCH_SELFTEST=1): runs here at
    // PASSIVE, after init, so the deliberate violation lands in the census.
    #[cfg(feature = "cswitchdep")]
    if crate::tripwire::CSWITCH_SELFTEST_ARMED {
        super::cswitchdep::cswitch_selftest();
    }
    crate::debugblock::debug_block_stamp();
    if crate::tripwire::FREEZE_SELFTEST_ARMED {
        freeze_selftest();
    }

    let mut handle: HANDLE = core::ptr::null_mut();
    let status = PsCreateSystemThread(
        &mut handle,
        THREAD_ALL_ACCESS,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        // Same transmute-by-address pattern psmgr.rs uses for Phase1Initialization:
        // bindgen's PKSTART_ROUTINE spelling differs per arch; the symbol's real
        // convention is extern "system" (NTAPI) on all three.
        core::mem::transmute(CensusThread as usize),
        core::ptr::null_mut(),
    );
    if status < 0 {
        crate::dbg_print!(b"[CENSUS] FAILED to start watchdog: status=%x\n\0", status as u32);
        return;
    }
    ZwClose(handle);
    // The banner states the COVERAGE, and it is load-bearing: face D was read
    // as "nothing is stuck" when the honest reading of the old banner was
    // "nothing is stuck IN A GATE WAIT". Keep it truthful as coverage changes.
    crate::dbg_print!(
        b"[CENSUS] sync watchdog up (10s cadence, gate waits + all parked waits w/ reason)\n\0"
    );
}


// ---------------------------------------------------------------------------
// FREEZE_SELFTEST=1 (tripwire): does a bugcheck stop the other processors?
//
// A ticker thread pinned to the LAST processor prints a numbered tick in a tight
// loop; this one waits until ticks are flowing, then bugchecks. With a working
// freeze no tick appears after the *** BUGCHECK line. arm64's KxFreezeExecution was
// an empty stub and the bugcheck froze nothing, so other CPUs ran on through the
// dump; nothing could show that without a deliberate crash on a busy MP machine.
// ---------------------------------------------------------------------------

static FREEZE_TICKS: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0);

unsafe extern "system" fn FreezeSelftestTicker(_ctx: PVOID) {
    let last = (ntkrnl_sys::globals::KeNumberProcessors as u32).saturating_sub(1);
    ntkrnl_sys::ddk::KeSetSystemAffinityThread(1 << last);
    loop {
        let n = FREEZE_TICKS.fetch_add(1, core::sync::atomic::Ordering::SeqCst);
        crate::dbg_print!(
            b"[FREEZE-SELFTEST] tick %u cpu=%u\n\0",
            n,
            ntkrnl_sys::ddk::KeGetCurrentProcessorNumber() as u32,
        );
        ntkrnl_sys::ddk::KeStallExecutionProcessor(2000);
    }
}

unsafe fn freeze_selftest() {
    let cpus = ntkrnl_sys::globals::KeNumberProcessors as u32;
    if cpus < 2 {
        crate::dbg_print!(
            b"[FREEZE-SELFTEST] ERROR: needs >= 2 CPUs (have %u) -- boot with --smp 2+\n\0",
            cpus,
        );
        return;
    }
    let mut handle: HANDLE = core::ptr::null_mut();
    let status = PsCreateSystemThread(
        &mut handle,
        THREAD_ALL_ACCESS,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::mem::transmute(FreezeSelftestTicker as usize),
        core::ptr::null_mut(),
    );
    if status < 0 {
        crate::dbg_print!(b"[FREEZE-SELFTEST] ERROR: ticker thread failed: %x\n\0", status as u32);
        return;
    }
    ZwClose(handle);
    // Stall, not KeDelayExecutionThread: this runs inside Phase-1 init, where a timed
    // wait did not return (the ticker ran 249k ticks and the bugcheck never came).
    while FREEZE_TICKS.load(core::sync::atomic::Ordering::SeqCst) < 20 {
        ntkrnl_sys::ddk::KeStallExecutionProcessor(1000);
    }
    crate::dbg_print!(
        b"[FREEZE-SELFTEST] ticks flowing on cpu %u; bugchecking from cpu %u -- PASS iff ticks stop within one line of the BUGCHECK banner (it prints before the freeze, on purpose)\n\0",
        cpus - 1,
        ntkrnl_sys::ddk::KeGetCurrentProcessorNumber() as u32,
    );
    ntkrnl_sys::ddk::KeBugCheckEx(0xE2 /* MANUALLY_INITIATED_CRASH */, 0xF2EE2E, 0, 0, 0);
}
