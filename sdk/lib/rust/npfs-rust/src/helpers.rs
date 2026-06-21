//! helpers.rs — the npfs.h FORCEINLINE helpers (VCB lock wrappers + NpCompleteDeferredIrps)
//! and the CONTAINING_RECORD machinery the modules need. The VCB lock helpers operate on the
//! global NpVcb (globals.rs). NpBugCheck is a macro per source file (it folds in the file ID +
//! __LINE__) — modules emit it inline via KeBugCheckEx; not provided here.
#![allow(non_snake_case, dead_code)]

use core::ptr::addr_of_mut;

use ntkrnl_sys::ddk::{
    ExAcquireResourceExclusiveLite, ExAcquireResourceSharedLite, ExReleaseResourceLite,
    LIST_ENTRY, PIRP, _IRP,
};
use ntkrnl_sys::driver_ddk::IoCompleteRequest;

use crate::glue::{IO_NAMED_PIPE_INCREMENT, TRUE};
use crate::globals::NpVcb;

/// CONTAINING_RECORD(addr, Type, field): recover the containing struct base from a pointer to
/// one of its fields. Self-consistent via core::mem::offset_of (single-ident field).
#[macro_export]
macro_rules! CONTAINING_RECORD {
    ($addr:expr, $T:ty, $field:ident) => {
        (($addr as *const u8).sub(core::mem::offset_of!($T, $field)) as *mut $T)
    };
}

/// &Irp->Tail.Overlay.ListEntry (bindgen anon-union path).
#[inline(always)]
pub unsafe fn irp_list_entry(Irp: PIRP) -> *mut LIST_ENTRY {
    addr_of_mut!((*Irp).Tail.Overlay.__bindgen_anon_2.ListEntry)
}

/// Inverse: CONTAINING_RECORD(entry, IRP, Tail.Overlay.ListEntry) via the dotted offset_of.
#[inline(always)]
pub unsafe fn irp_from_list_entry(entry: *mut LIST_ENTRY) -> PIRP {
    (entry as *const u8)
        .sub(core::mem::offset_of!(_IRP, Tail.Overlay.__bindgen_anon_2.ListEntry)) as PIRP
}

#[inline(always)]
pub unsafe fn NpAcquireSharedVcb() {
    ExAcquireResourceSharedLite(addr_of_mut!((*NpVcb).Lock), TRUE);
}

#[inline(always)]
pub unsafe fn NpAcquireExclusiveVcb() {
    ExAcquireResourceExclusiveLite(addr_of_mut!((*NpVcb).Lock), TRUE);
}

#[inline(always)]
pub unsafe fn NpReleaseVcb() {
    ExReleaseResourceLite(addr_of_mut!((*NpVcb).Lock));
}

/// Process deferred IRPs outside the VCB lock but still within the critical region.
#[inline(always)]
pub unsafe fn NpCompleteDeferredIrps(DeferredList: *mut LIST_ENTRY) {
    let mut ThisEntry = (*DeferredList).Flink;
    while ThisEntry != DeferredList {
        let NextEntry = (*ThisEntry).Flink;
        let Irp = irp_from_list_entry(ThisEntry);
        IoCompleteRequest(Irp, IO_NAMED_PIPE_INCREMENT);
        ThisEntry = NextEntry;
    }
}
