//! helpers.rs — the handful of header inlines msfs.h / the .c files use that are not plain
//! ntoskrnl exports: CONTAINING_RECORD, the KeLockMutex / KeUnlockMutex macros (msfs.h),
//! the wide-string builder for the device name, and two IRP overlay-field accessors that
//! spell out the bindgen anonymous-union paths for Tail.Overlay.{DriverContext,ListEntry}.
#![allow(non_snake_case, dead_code)]

use core::ptr::{addr_of_mut, null_mut};
use ntkrnl_sys::base::{BOOLEAN, PVOID};
use ntkrnl_sys::ddk::{
    KeReleaseMutex, KeWaitForSingleObject, KMUTEX, LIST_ENTRY, PIRP, PRKMUTEX, _IRP,
};

use crate::glue::{Executive, KernelMode, FALSE};

/// CONTAINING_RECORD(addr, Type, field): recover the base of the containing struct from a
/// pointer to one of its fields. Self-consistent via core::mem::offset_of.
#[macro_export]
macro_rules! CONTAINING_RECORD {
    ($addr:expr, $T:ty, $field:ident) => {
        (($addr as *const u8).sub(core::mem::offset_of!($T, $field)) as *mut $T)
    };
}

/// wsz — const ASCII &[u8] → NUL-terminated UTF-16 `[u16; N]`. N must be (char count + 1).
pub const fn wsz<const N: usize>(s: &[u8]) -> [u16; N] {
    let mut a = [0u16; N];
    let mut i = 0;
    while i < s.len() {
        a[i] = s[i] as u16;
        i += 1;
    }
    a
}

/// msfs.h: KeLockMutex(x) == KeWaitForSingleObject(x, Executive, KernelMode, FALSE, NULL).
#[inline(always)]
pub unsafe fn KeLockMutex(Mutex: *mut KMUTEX) {
    KeWaitForSingleObject(Mutex as PVOID, Executive, KernelMode, FALSE, null_mut());
}

/// msfs.h: KeUnlockMutex(x) == KeReleaseMutex(x, FALSE).
#[inline(always)]
pub unsafe fn KeUnlockMutex(Mutex: *mut KMUTEX) {
    KeReleaseMutex(Mutex as PRKMUTEX, FALSE);
}

/// &Irp->Tail.Overlay.DriverContext[0] — the driver-private scratch slot msfs stores its
/// MSFS_DPC_CTX pointer in while a read IRP is queued.
#[inline(always)]
pub unsafe fn irp_driver_context0(Irp: PIRP) -> *mut PVOID {
    addr_of_mut!((*Irp).Tail.Overlay.__bindgen_anon_1.__bindgen_anon_1.DriverContext[0])
}

/// &Irp->Tail.Overlay.ListEntry — the link msfs threads pending read IRPs on the CSQ
/// PendingIrpQueue with.
#[inline(always)]
pub unsafe fn irp_list_entry(Irp: PIRP) -> *mut LIST_ENTRY {
    addr_of_mut!((*Irp).Tail.Overlay.__bindgen_anon_2.ListEntry)
}

/// Inverse of irp_list_entry: CONTAINING_RECORD(entry, IRP, Tail.Overlay.ListEntry). The
/// dotted offset_of path traverses the bindgen anon unions to the same field.
#[inline(always)]
pub unsafe fn irp_from_list_entry(entry: *mut LIST_ENTRY) -> PIRP {
    (entry as *const u8)
        .sub(core::mem::offset_of!(_IRP, Tail.Overlay.__bindgen_anon_2.ListEntry)) as PIRP
}
