//! Per-process client window procs, for a process whose user32 is not the native one.
//!
//! win32k gives a system-class window (ScrollBar, Button, Edit, ...) its proc from
//! `gpsi->apfnClientA/W`, which the FIRST user32 to call
//! `NtUserInitializeClientPfnArrays` fills and every later call leaves alone. That
//! holds while every process's user32 is the same build at the same base. An
//! x86-64 guest on arm64 (wowx64) runs the AMD64 user32, which shares the native
//! one's preferred base but not its layout: regedit's ScrollBar got the native
//! `ScrollBarWndProcW` address, which in the guest is the middle of an amd64
//! instruction, and FEX raised c000001d there. (A 32-bit WoW64 guest never gets
//! this far: wow64.dll intercepts the call and maps procs at its own boundary,
//! wow64_pfnmap.c.)
//!
//! So a process whose tables DIFFER from the global ones keeps a private copy, and
//! the `GETPFNCLIENT*`/`GETPFNSERVER` lookups (class.rs, window.rs) answer from it
//! for that process. Native processes pass identical tables and never get an
//! entry. Owed: the guest's user32 still reads `gpsi->apfnClient*` itself (it is a
//! shared read-only mapping), so a user-mode comparison against those slots in an
//! x86-64 guest sees the native values.
//!
//! Every access runs under the user lock: written under exclusive, read by
//! window/class code that holds it.

use win32k_sys::win32k::*;
use ntkrnl_sys::base::{STATUS_INSUFFICIENT_RESOURCES, STATUS_SUCCESS};
use ntkrnl_sys::ddk::PsGetCurrentProcessWin32Process;

extern "C" {
    fn DbgPrint(format: *const u8, ...) -> u32;
}

const SLOTS: usize = core::mem::size_of::<PFNCLIENT>() / core::mem::size_of::<usize>();
const PRIVATE_TABLES: usize = 32;

#[derive(Clone, Copy)]
struct Private {
    ppi: usize,
    a: [usize; SLOTS],
    w: [usize; SLOTS],
}

static mut TABLE: [Private; PRIVATE_TABLES] = [Private { ppi: 0, a: [0; SLOTS], w: [0; SLOTS] }; PRIVATE_TABLES];

unsafe fn global(ansi: bool) -> &'static [usize; SLOTS] {
    let p = if ansi {
        core::ptr::addr_of!((*gpsi).apfnClientA)
    } else {
        core::ptr::addr_of!((*gpsi).apfnClientW)
    };
    &*(p as *const [usize; SLOTS])
}

unsafe fn entry(ppi: usize) -> Option<&'static mut Private> {
    if ppi == 0 {
        return None;
    }
    (*core::ptr::addr_of_mut!(TABLE)).iter_mut().find(|e| e.ppi == ppi)
}

/// `NtUserInitializeClientPfnArrays` after the global tables are set: keep this
/// process's own tables when they differ. Called with the user lock held
/// exclusively; the tables were already captured from user memory.
pub unsafe fn remember(a: &[usize; SLOTS], w: &[usize; SLOTS]) -> NTSTATUS {
    if a == global(true) && w == global(false) {
        return STATUS_SUCCESS;
    }
    let ppi = PsGetCurrentProcessWin32Process() as usize;
    let slot = match entry(ppi) {
        Some(e) => e,
        None => match (*core::ptr::addr_of_mut!(TABLE)).iter_mut().find(|e| e.ppi == 0) {
            Some(e) => e,
            None => {
                // Loud, not silent: falling back to the global procs is the crash
                // this table exists to prevent.
                DbgPrint(
                    b"[WIN32K] NtUserInitializeClientPfnArrays: no free per-process slot (%u in use); refusing\n\0".as_ptr(),
                    PRIVATE_TABLES as u32,
                );
                return STATUS_INSUFFICIENT_RESOURCES;
            }
        },
    };
    slot.ppi = ppi;
    slot.a = *a;
    slot.w = *w;
    STATUS_SUCCESS
}

/// The client proc for slot `index` of the A or W table, as the current process
/// knows it.
pub unsafe fn client_proc(ansi: bool, index: usize) -> usize {
    match entry(PsGetCurrentProcessWin32Process() as usize) {
        Some(e) => if ansi { e.a[index] } else { e.w[index] },
        None => global(ansi)[index],
    }
}

/// The current process's private W table, if it has one: `GETPFNSERVER` reads
/// `aStoCidPfn`, which `NtUserInitializeClientPfnArrays` fills from the W table.
pub unsafe fn private_w(index: usize) -> Option<usize> {
    entry(PsGetCurrentProcessWin32Process() as usize).map(|e| e.w[index])
}

/// `UserProcessDestroy`: the PROCESSINFO is about to be freed and its address
/// reused.
pub unsafe fn forget(ppi: PPROCESSINFO) {
    if let Some(e) = entry(ppi as usize) {
        e.ppi = 0;
    }
}
