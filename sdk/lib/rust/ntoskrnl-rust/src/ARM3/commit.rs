//! Commit accounting: every page a process or a pagefile-backed section may touch is charged
//! against `MmTotalCommitLimit` when it is COMMITTED, so running out of memory fails the
//! allocation (`STATUS_COMMITMENT_LIMIT`) instead of the first touch of a page there is no
//! frame for. Upstream ReactOS never charged: `MmTotalCommittedPages` counted consumer pages in
//! use (balance.c), the limit was never checked, and an exhausted machine either bugchecked
//! 0x7A in a page-table fault or killed whichever process faulted next.
//!
//! The ledger, as Windows keeps it:
//!   * `MmTotalCommittedPages`: every charge in the system, in pages.
//!   * `EPROCESS::CommitCharge`: the charges for a process's private memory. Whatever is left
//!     when its address space is torn down is returned there in one go, so no VAD path can
//!     leak a charge.
//!   * `SEGMENT::NumberOfCommittedPages`: the charges for a pagefile-backed section, returned
//!     when the segment is deleted.
//!
//!   * PEB and TEB pages (procsup.rs `MiCreatePebOrTebCapped`); a TEB's charge comes back in
//!     `MmDeleteTeb`, a PEB's with the process's ledger.
//!
//! Not charged yet (owed): copy-on-write pages of image and file views, and kernel pool. The
//! limit leaves headroom for them (mminit.rs).

use core::ptr::addr_of_mut;
use core::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

use ntkrnl_sys::ddk::{NTSTATUS, PEPROCESS};

/// What a commit past the limit fails with (ntstatus.h); kernel32 maps it to
/// ERROR_COMMITMENT_LIMIT.
pub const STATUS_COMMITMENT_LIMIT: NTSTATUS = 0xC000_012D_u32 as NTSTATUS;

#[inline(always)]
fn Total() -> &'static AtomicUsize {
    unsafe { AtomicUsize::from_ptr(addr_of_mut!(crate::mm::freelist::MmTotalCommittedPages)) }
}

#[inline(always)]
fn Peak() -> &'static AtomicUsize {
    unsafe { AtomicUsize::from_ptr(addr_of_mut!(crate::mm::freelist::MmPeakCommitment)) }
}

#[inline(always)]
unsafe fn ProcessCharge(Process: PEPROCESS) -> &'static AtomicUsize {
    AtomicUsize::from_ptr(addr_of_mut!((*Process).CommitCharge) as *mut usize)
}

static Refusals: AtomicU32 = AtomicU32::new(0);
static Underflows: AtomicU32 = AtomicU32::new(0);

/// Said for the first few and then every 256th, so a starving process cannot flood serial.
fn Loud(Counter: &AtomicU32) -> bool {
    let N = Counter.fetch_add(1, Ordering::Relaxed);
    N < 8 || N % 256 == 0
}

/// Charges `Pages` against the commit limit, and to `Process`'s ledger when it is not null.
/// False, with nothing charged, when the charge would pass the limit.
pub unsafe fn MiChargeCommitment(Pages: usize, Process: PEPROCESS) -> bool {
    if Pages == 0 {
        return true;
    }
    let Limit = core::ptr::read_volatile(addr_of_mut!(super::mminit::MmTotalCommitLimit)) as usize;
    let mut Current = Total().load(Ordering::Relaxed);
    loop {
        let New = match Current.checked_add(Pages) {
            Some(N) if N <= Limit => N,
            _ => {
                if Loud(&Refusals) {
                    ntkrnl_sys::driver_ddk::DbgPrint(
                        b"[MM] commit limit: refused %Iu pages for process %p (%Iu of %Iu charged)\n\0".as_ptr() as _,
                        Pages,
                        Process,
                        Current,
                        Limit,
                    );
                }
                return false;
            }
        };
        match Total().compare_exchange_weak(Current, New, Ordering::Relaxed, Ordering::Relaxed) {
            Ok(_) => {
                Peak().fetch_max(New, Ordering::Relaxed);
                break;
            }
            Err(Seen) => Current = Seen,
        }
    }
    if !Process.is_null() {
        let Mine = ProcessCharge(Process).fetch_add(Pages, Ordering::Relaxed) + Pages;
        if Mine as u64 > (*Process).CommitChargePeak as u64 {
            (*Process).CommitChargePeak = Mine as _;
        }
    }
    true
}

/// Returns `Pages` charged earlier (to `Process`'s ledger too, when it is not null). A return
/// larger than what is charged is a ledger bug: it is said, and clamped rather than wrapped.
pub unsafe fn MiReturnCommitment(Pages: usize, Process: PEPROCESS) {
    if Pages == 0 {
        return;
    }
    let mut Pages = Pages;
    if !Process.is_null() {
        let Ledger = ProcessCharge(Process);
        let Mine = Ledger.load(Ordering::Relaxed);
        if Pages > Mine {
            if Loud(&Underflows) {
                ntkrnl_sys::driver_ddk::DbgPrint(
                    b"[MM] commit ledger: process %p returns %Iu pages, holds %Iu\n\0".as_ptr() as _,
                    Process,
                    Pages,
                    Mine,
                );
            }
            Pages = Mine;
        }
        Ledger.fetch_sub(Pages, Ordering::Relaxed);
    }
    let Previous = Total().fetch_sub(Pages, Ordering::Relaxed);
    if Previous < Pages {
        Total().fetch_add(Pages - Previous, Ordering::Relaxed);
        if Loud(&Underflows) {
            ntkrnl_sys::driver_ddk::DbgPrint(
                b"[MM] commit ledger: %Iu pages returned, %Iu charged\n\0".as_ptr() as _,
                Pages,
                Previous,
            );
        }
    }
}

/// Process teardown: whatever its private memory still holds goes back in one go.
pub unsafe fn MiReturnProcessCommitment(Process: PEPROCESS) {
    let Remaining = ProcessCharge(Process).load(Ordering::Relaxed);
    MiReturnCommitment(Remaining, Process);
}
