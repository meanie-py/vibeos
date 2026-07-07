//! waitdep — dispatcher wait-block integrity tripwire (ALWAYS-ON).
//!
//! Target: the arm64 KiUnwaitThread 0x1E flake (field-manual §9.24) — a
//! KWAIT_BLOCK / dispatcher-list corruption that surfaces as an
//! innocent-victim 0xC0000005 inside `KiUnwaitThread`/`KiUnlinkThread`
//! (`RemoveEntryList` on a corrupt `WaitListEntry`). Special pool cannot
//! catch it (wait blocks live on thread stacks, not pool), and a GDB
//! watchpoint needs a repro in hand. So: validate the structures at the two
//! chokepoints the crash path funnels through — `KiUnlinkThread` (every
//! unwait) and `KiWaitTest` (every object satisfaction) — and fault LOUDLY
//! at first violation, with the corrupt chain dumped to serial, instead of
//! letting `RemoveEntryList` silently corrupt neighbors and crash later.
//!
//! Discipline (same family as census/flightrec, NOT feature-gated):
//! - Checks are pure loads + compares over structures the caller is about to
//!   dereference anyway — a violation here means the existing code was about
//!   to AV or silently cross-link lists. No false-positive surface: every
//!   hard check is a doubly-linked-list / back-pointer invariant that MUST
//!   hold for the subsequent mutation to be correct.
//! - On hard violation: dump the block chain (`[WAITDEP]` on serial), then
//!   `KeBugCheckEx(0x109 CRITICAL_STRUCTURE_CORRUPTION)` with the culprit
//!   address in P1 — triage picks up both.
//! - Soft anomalies (thread State != Waiting at unwait) print-only,
//!   rate-limited — legit paths may race the state byte; structural
//!   invariants are the bugcheck-worthy ones.

#![allow(non_snake_case)]

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering::Relaxed};

use ntkrnl_sys::ddk::{KeBugCheckEx, LIST_ENTRY, PKTHREAD, PKWAIT_BLOCK, _KWAIT_BLOCK};

/// CRITICAL_STRUCTURE_CORRUPTION — the honest NT code for "a kernel
/// structure integrity check failed".
const BC_CRITICAL_STRUCTURE_CORRUPTION: u32 = 0x109;

/// Violation kinds (BUGCHECK P4 + report text).
const K_BAD_THREAD: u32 = 1; // thread pointer not a kernel VA
const K_BAD_BLOCK: u32 = 2; // wait block pointer not a kernel VA / unaligned
const K_BACKPTR: u32 = 3; // block.Thread != owning thread
const K_LIST: u32 = 4; // WaitListEntry Flink/Blink cross-check failed
const K_CHAIN: u32 = 5; // NextWaitBlock cycle never closed (overrun)
const K_OBJ_BACKPTR: u32 = 6; // block.Object != object whose list it sits on
const K_WAITLIST: u32 = 7; // object WaitListHead walk hit a bad entry

/// KTHREAD.State value for a waiting thread.
const STATE_WAITING: u8 = 5;

/// A thread chains at most MAXIMUM_WAIT_OBJECTS (64) + builtin blocks.
const MAX_CHAIN: usize = 68;
/// An object's waiter list is bounded by live threads; far past any real count.
const MAX_WAITERS: usize = 4096;

/// Total checks performed (cheap liveness signal for GDB).
#[no_mangle]
pub static VIBEOS_WAITDEP_CHECKS: AtomicU64 = AtomicU64::new(0);
/// Soft-anomaly prints emitted (rate-limit counter).
static SOFT_PRINTS: AtomicU32 = AtomicU32::new(0);
const SOFT_PRINT_CAP: u32 = 8;

/// Kernel-VA check that holds on all three arches: i386 kernel space is
/// >= 0x80000000 and amd64/arm64 kernel VAs are canonical high-half — in all
/// cases the top bit is set, i.e. the pointer is negative as isize.
#[inline(always)]
fn is_kva(p: usize) -> bool {
    (p as isize) < 0 && (p & (core::mem::align_of::<usize>() - 1)) == 0
}

fn kind_name(kind: u32) -> *const u8 {
    match kind {
        K_BAD_THREAD => b"bad-thread-ptr\0".as_ptr(),
        K_BAD_BLOCK => b"bad-block-ptr\0".as_ptr(),
        K_BACKPTR => b"thread-backptr\0".as_ptr(),
        K_LIST => b"list-integrity\0".as_ptr(),
        K_CHAIN => b"chain-overrun\0".as_ptr(),
        K_OBJ_BACKPTR => b"object-backptr\0".as_ptr(),
        K_WAITLIST => b"waitlist-walk\0".as_ptr(),
        _ => b"?\0".as_ptr(),
    }
}

/// Dump one wait block's raw fields (best-effort: only if the pointer itself
/// is dereferenceable-looking; a bad block pointer is reported by address only).
unsafe fn dump_block(idx: usize, wb: PKWAIT_BLOCK) {
    if !is_kva(wb as usize) {
        crate::dbg_print!(b"[WAITDEP]   blk[%u] = %p (NOT a kernel VA)\n\0", idx as u32, wb as *const ());
        return;
    }
    crate::dbg_print!(
        b"[WAITDEP]   blk[%u]=%p Thread=%p Object=%p Next=%p Flink=%p Blink=%p WaitKey=%u WaitType=%u\n\0",
        idx as u32,
        wb as *const (),
        (*wb).Thread as *const (),
        (*wb).Object as *const (),
        (*wb).NextWaitBlock as *const (),
        (*wb).WaitListEntry.Flink as *const (),
        (*wb).WaitListEntry.Blink as *const (),
        (*wb).WaitKey as u32,
        (*wb).WaitType as u32,
    );
}

/// Report + halt. `owner` is the thread (unwait path) or object (waittest
/// path) whose structure failed; `culprit` is the corrupt pointer/entry.
unsafe fn violation(kind: u32, owner: usize, culprit: usize, chain: PKWAIT_BLOCK) -> ! {
    crate::dbg_print!(
        b"[WAITDEP] VIOLATION %s: owner=%p culprit=%p - dispatcher wait-block corruption caught BEFORE the mutating walk (field-manual 9.24)\n\0",
        kind_name(kind),
        owner as *const (),
        culprit as *const (),
    );
    // Best-effort chain dump for forensics (bounded; tolerate the corruption).
    if !chain.is_null() {
        let mut wb = chain;
        for i in 0..MAX_CHAIN {
            dump_block(i, wb);
            if !is_kva(wb as usize) {
                break;
            }
            let next = (*wb).NextWaitBlock;
            if next == chain || next.is_null() {
                break;
            }
            wb = next;
        }
    }
    KeBugCheckEx(
        BC_CRITICAL_STRUCTURE_CORRUPTION,
        culprit as _,
        owner as _,
        VIBEOS_WAITDEP_CHECKS.load(Relaxed) as _,
        kind as _,
    );
}

/// Validate one LIST_ENTRY's doubly-linked invariants: both links are kernel
/// VAs and both neighbors point back at us. This is exactly what
/// `RemoveEntryList(entry)` is about to rely on — a failure here means the
/// removal would have cross-linked foreign memory.
#[inline]
unsafe fn entry_ok(entry: *mut LIST_ENTRY) -> bool {
    let f = (*entry).Flink;
    let b = (*entry).Blink;
    if !is_kva(f as usize) || !is_kva(b as usize) {
        return false;
    }
    (*f).Blink == entry && (*b).Flink == entry
}

/// Chokepoint 1 — called at `KiUnlinkThread` entry (which `KiUnwaitThread`
/// funnels through), BEFORE any RemoveEntryList: walk the thread's wait-block
/// chain and prove every block it is about to unlink is intact.
pub unsafe fn check_unwait(Thread: PKTHREAD) {
    VIBEOS_WAITDEP_CHECKS.fetch_add(1, Relaxed);

    if !is_kva(Thread as usize) {
        violation(K_BAD_THREAD, Thread as usize, Thread as usize, core::ptr::null_mut());
    }

    // Soft anomaly: unwaiting a thread that isn't Waiting. Print-only —
    // structurally survivable, but a breadcrumb if the flake mutates state.
    let state = *super::ke_defs::kt_State(Thread);
    if state != STATE_WAITING && SOFT_PRINTS.fetch_add(1, Relaxed) < SOFT_PRINT_CAP {
        crate::dbg_print!(
            b"[WAITDEP] soft: unwait of thread=%p with State=%u (expected Waiting=5)\n\0",
            Thread as *const (),
            state as u32,
        );
    }

    let head: PKWAIT_BLOCK = (*Thread).__bindgen_anon_2.WaitBlockList;
    let mut wb = head;
    let mut i = 0usize;
    loop {
        if !is_kva(wb as usize) {
            violation(K_BAD_BLOCK, Thread as usize, wb as usize, head);
        }
        if (*wb).Thread != Thread {
            violation(K_BACKPTR, Thread as usize, wb as usize, head);
        }
        let entry = core::ptr::addr_of_mut!((*wb).WaitListEntry) as *mut LIST_ENTRY;
        if !entry_ok(entry) {
            violation(K_LIST, Thread as usize, wb as usize, head);
        }
        wb = (*wb).NextWaitBlock;
        if wb == head {
            break;
        }
        i += 1;
        if i > MAX_CHAIN {
            violation(K_CHAIN, Thread as usize, wb as usize, head);
        }
    }

    // The thread's own WaitListEntry (KiUnlinkThread removes it when linked).
    let twe = core::ptr::addr_of_mut!((*Thread).__bindgen_anon_3.WaitListEntry) as *mut LIST_ENTRY;
    if !(*twe).Flink.is_null() && !entry_ok(twe) {
        violation(K_LIST, Thread as usize, twe as usize, head);
    }
}

/// Chokepoint 2 — called at `KiWaitTest` entry with the object address and
/// its `Header.WaitListHead`: walk the waiter list and prove every wait block
/// on it belongs here (Object back-pointer) and is structurally sound before
/// the satisfy/unwait loop starts consuming it.
pub unsafe fn check_waittest(object: usize, wait_list: *mut LIST_ENTRY) {
    VIBEOS_WAITDEP_CHECKS.fetch_add(1, Relaxed);

    let mut entry = (*wait_list).Flink;
    let mut i = 0usize;
    while entry != wait_list {
        if !is_kva(entry as usize) || !entry_ok(entry) {
            violation(K_WAITLIST, object, entry as usize, core::ptr::null_mut());
        }
        let wb = ((entry as usize) - core::mem::offset_of!(_KWAIT_BLOCK, WaitListEntry)) as PKWAIT_BLOCK;
        if (*wb).Object as usize != object {
            violation(K_OBJ_BACKPTR, object, wb as usize, wb);
        }
        if !is_kva((*wb).Thread as usize) {
            violation(K_BACKPTR, object, wb as usize, wb);
        }
        entry = (*entry).Flink;
        i += 1;
        if i > MAX_WAITERS {
            violation(K_WAITLIST, object, entry as usize, core::ptr::null_mut());
        }
    }
}
