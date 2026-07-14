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
//! Silent when healthy: no output unless a stuck thread is found.

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

const THREAD_ALL_ACCESS: u32 = 0x001F_FFFF; // STANDARD_RIGHTS_REQUIRED|SYNCHRONIZE|0xFFFF

use super::ke_defs::kt_State;
use super::synctrace::{synctrace_dropped, synctrace_lookup};

/// KTHREAD_STATE::GateWait (see ke/gate.rs).
const GateWait: u8 = 8;

const SCAN_PERIOD_100NS: i64 = -10 * 10_000_000; // relative 10s
const MAX_PROCESSES: usize = 512;
const MAX_THREADS_PER_PROCESS: usize = 2048;
/// Sightings carried between scans. 128 covers the observed wedges (dozens of
/// parked threads); overflow is counted and reported, never silent.
const MAX_SIGHTINGS: usize = 128;

#[derive(Clone, Copy, PartialEq)]
struct Sighting {
    thread: usize,
    object: usize,
}

/// Single-writer state (only the census thread touches these).
static mut PREV: [Sighting; MAX_SIGHTINGS] = [Sighting { thread: 0, object: 0 }; MAX_SIGHTINGS];
static mut PREV_COUNT: usize = 0;
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

/// One full process/thread scan. Returns the new sighting set in `next`,
/// prints a report line for every second-consecutive sighting.
unsafe fn scan(next: &mut [Sighting; MAX_SIGHTINGS], next_count: &mut usize) {
    let head = core::ptr::addr_of_mut!(PsActiveProcessHead);
    let mut plink = (*head).Flink;
    let mut nprocs = 0usize;
    let mut reported = 0u32;

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

            if *kt_State(kthread) == GateWait {
                let object = (*kthread).__bindgen_anon_2.GateObject as usize;
                let s = Sighting { thread: kthread as usize, object };

                // Second consecutive sighting on the same object => stuck.
                let mut stuck = false;
                for i in 0..PREV_COUNT {
                    if PREV[i] == s {
                        stuck = true;
                        break;
                    }
                }
                if stuck {
                    report(eproc, kthread, object);
                    reported += 1;
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

    if reported != 0 {
        crate::dbg_print!(
            b"[CENSUS] %u thread(s) GateWait >10s (scan #%u, synctrace dropped=%u, sighting overflow=%u)\n\0",
            reported,
            VIBEOS_CENSUS_SCANS.load(Relaxed),
            synctrace_dropped(),
            SIGHTING_OVERFLOW.load(Relaxed),
        );
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
            #[cfg(feature = "smp")]
            {
                use ntkrnl_sys::ddk::_KGUARDED_MUTEX;
                let gate_off = core::mem::offset_of!(_KGUARDED_MUTEX, Gate);
                let gm = (object - gate_off) as *const _KGUARDED_MUTEX;
                let owner = core::ptr::read_volatile(core::ptr::addr_of!((*gm).Owner));
                if (owner as usize) >= 0xFFFF_8000_0000_0000 && ((owner as usize) & 7) == 0 {
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

unsafe extern "system" fn CensusThread(_ctx: PVOID) {
    let mut interval: LARGE_INTEGER = core::mem::zeroed();
    interval.QuadPart = SCAN_PERIOD_100NS;

    loop {
        KeDelayExecutionThread(0, 0, core::ptr::addr_of_mut!(interval) as *mut _);

        let mut next = [Sighting { thread: 0, object: 0 }; MAX_SIGHTINGS];
        let mut next_count = 0usize;
        scan(&mut next, &mut next_count);
        PREV = next;
        PREV_COUNT = next_count;
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
    crate::dbg_print!(b"[CENSUS] sync watchdog up (10s cadence, gate waits)\n\0");
}
