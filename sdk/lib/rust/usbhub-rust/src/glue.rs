//! usbhub glue — c2rust phantom-extern bridges + cohab shims not in the bindgen types.rs.
//! Filled during integration/compile-fix (usbstor/usbccgp pattern). DbgPrint/RtlAssert resolve
//! from the ntoskrnl import lib (modules declare them in their own extern blocks).
#![allow(unused_imports)]

pub use crate::types::*;

// Basic kernel types the modules name bare but only import partially from ddk.
// PCHAR/PUCHAR/PCWSTR EXCLUDED (arrive via base/driver_ddk globs → would be ambiguous).
pub use ntkrnl_sys::ddk::{
    CCHAR, KPRIORITY, KPROCESSOR_MODE, PCSTR, PCWCH, PDRIVER_DISPATCH, PWCH,
    _UNICODE_STRING, LANGID, GUID,
};
pub type size_t = usize;
pub type wchar_t = u16;
pub type NTSTRSAFE_PWSTR = ntkrnl_sys::ddk::PWSTR;
pub type NTSTRSAFE_PCWSTR = ntkrnl_sys::ddk::PCWSTR;

// High-frequency consts the translator inlined as bare idents across many functions.
// Verified against the C headers (values, not guessed):
pub const USB_HUB_TAG: u32 = 0x42554855; // 'BUHU' (usbhub.h:26)
pub const EVENT_INCREMENT: i8 = 1; // ntddk.h:597
pub const STATUS_MORE_PROCESSING_REQUIRED: ntkrnl_sys::ddk::NTSTATUS = 0xC0000016u32 as i32; // ntstatus.h
pub const IOCTL_INTERNAL_USB_SUBMIT_URB: u32 = 0x00220003; // CTL_CODE(FILE_DEVICE_USB,0,METHOD_NEITHER,0)

// ---- inline-only DDK helpers + NDEBUG debug-macro no-ops (no exported symbol; undefined at
// cohab link). Decoration matches the undefined symbols (cdecl `_Name` except InterlockedExchange
// stdcall `_Name@8`). IRP-stack helpers wrap ntkrnl_sys::inline (same as usbstor/usbccgp). ----
use ntkrnl_sys::ddk::{
    PIRP, PIO_STACK_LOCATION, PDRIVER_CANCEL, PIO_COMPLETION_ROUTINE, PVOID, LONG, NTSTATUS,
    IRP, IO_STACK_LOCATION,
};

#[no_mangle]
pub unsafe extern "C" fn IoGetCurrentIrpStackLocation(Irp: PIRP) -> PIO_STACK_LOCATION {
    ntkrnl_sys::inline::tier1::IoGetCurrentIrpStackLocation(Irp)
}
#[no_mangle]
pub unsafe extern "C" fn IoGetNextIrpStackLocation(Irp: PIRP) -> PIO_STACK_LOCATION {
    ntkrnl_sys::inline::tier1::IoGetNextIrpStackLocation(Irp)
}
#[no_mangle]
pub unsafe extern "C" fn IoSkipCurrentIrpStackLocation(Irp: PIRP) {
    ntkrnl_sys::inline::tier1::IoSkipCurrentIrpStackLocation(Irp)
}
#[no_mangle]
pub unsafe extern "C" fn IoMarkIrpPending(Irp: PIRP) {
    ntkrnl_sys::inline::tier1::IoMarkIrpPending(Irp)
}
#[no_mangle]
pub unsafe extern "C" fn IoSetCancelRoutine(Irp: PIRP, CancelRoutine: PDRIVER_CANCEL) -> PDRIVER_CANCEL {
    ntkrnl_sys::inline::tier1::IoSetCancelRoutine(Irp, CancelRoutine)
}
#[no_mangle]
pub unsafe extern "C" fn IoSetCompletionRoutine(
    Irp: PIRP, CompletionRoutine: PIO_COMPLETION_ROUTINE, Context: PVOID,
    InvokeOnSuccess: u8, InvokeOnError: u8, InvokeOnCancel: u8,
) {
    let stk = IoGetNextIrpStackLocation(Irp);
    (*stk).CompletionRoutine = CompletionRoutine;
    (*stk).Context = Context;
    let mut ctl: u8 = 0;
    if InvokeOnSuccess != 0 { ctl |= 0x40; }
    if InvokeOnError != 0 { ctl |= 0x80; }
    if InvokeOnCancel != 0 { ctl |= 0x20; }
    (*stk).Control = ctl;
}
// IoCopyCurrentIrpStackLocationToNext — copy current→next stack up to (excl) CompletionRoutine,
// clear next Control (wdm.h FORCEINLINE).
#[no_mangle]
pub unsafe extern "C" fn IoCopyCurrentIrpStackLocationToNext(Irp: PIRP) {
    let cur = IoGetCurrentIrpStackLocation(Irp);
    let next = IoGetNextIrpStackLocation(Irp);
    let n = core::mem::offset_of!(IO_STACK_LOCATION, CompletionRoutine);
    core::ptr::copy_nonoverlapping(cur as *const u8, next as *mut u8, n);
    (*next).Control = 0;
}
// IoSizeOfIrp — wdm.h FORCEINLINE: sizeof(IRP) + StackSize * sizeof(IO_STACK_LOCATION).
#[no_mangle]
pub unsafe extern "C" fn IoSizeOfIrp(StackSize: i8) -> u16 {
    (core::mem::size_of::<IRP>() + (StackSize as usize) * core::mem::size_of::<IO_STACK_LOCATION>()) as u16
}
#[no_mangle]
pub unsafe extern "C" fn KeInitializeSpinLock(SpinLock: *mut usize) {
    *SpinLock = 0;
}
// InterlockedExchange — atomic swap (intrinsic on i386, no ntoskrnl export). stdcall `@8`.
#[no_mangle]
pub unsafe extern "system" fn InterlockedExchange(Target: *mut LONG, Value: LONG) -> LONG {
    (*(Target as *const core::sync::atomic::AtomicI32)).swap(Value, core::sync::atomic::Ordering::SeqCst)
}
// RtlStringCbPrintfW — ntstrsafe.h FORCEINLINE wide formatted print; variadic formatting is not
// reconstructible from the fixed extern decl. cdecl (caller cleans the stack, so this fixed-arity
// def safely satisfies the variadic call site). Null-terminates the dest. NOTE: known fidelity
// gap — owed a faithful ntstrsafe port (mirrors cdrom-rust's RtlStringCchPrintfW shim).
#[no_mangle]
pub unsafe extern "C" fn RtlStringCbPrintfW(pszDest: *mut u16, cbDest: usize, _pszFormat: *const u16) -> NTSTATUS {
    if !pszDest.is_null() && cbDest >= 2 { *pszDest = 0; }
    0
}
// DPRINT_PNP / DPRINT_SCE — NDEBUG-disabled DbgPrint wrappers (dbg_uhub.h); no symbol in retail.
// cdecl no-ops (fixed arity is fine: cdecl caller cleans the varargs).
#[no_mangle]
pub unsafe extern "C" fn DPRINT_PNP(_fmt: *const i8) {}
#[no_mangle]
pub unsafe extern "C" fn DPRINT_SCE(_fmt: *const i8) {}
// InterlockedIncrement/Decrement — atomic +1/-1 returning the NEW value (intrinsics, no export).
// stdcall `@4`.
#[no_mangle]
unsafe extern "system" fn InterlockedIncrement(Addend: *mut core::ffi::c_long) -> core::ffi::c_long {
    (*(Addend as *const core::sync::atomic::AtomicI32)).fetch_add(1, core::sync::atomic::Ordering::SeqCst) + 1
}
#[no_mangle]
unsafe extern "system" fn InterlockedDecrement(Addend: *mut core::ffi::c_long) -> core::ffi::c_long {
    (*(Addend as *const core::sync::atomic::AtomicI32)).fetch_sub(1, core::sync::atomic::Ordering::SeqCst) - 1
}
// URB_FROM_IRP — usbdlib.h macro: (PURB)IoGetCurrentIrpStackLocation(Irp)->Parameters.Others.Argument1.
// stdcall `@4`.
#[no_mangle]
pub unsafe extern "system" fn URB_FROM_IRP(Irp: PIRP) -> PURB {
    let stk = IoGetCurrentIrpStackLocation(Irp);
    (*stk).Parameters.Others.Argument1 as PURB
}
// RtlStringCbPrintfExW — ntstrsafe.h FORCEINLINE; variadic formatting not reconstructible from
// the fixed extern decl. cdecl no-op (null-terminates dest). NOTE: fidelity gap, like
// RtlStringCbPrintfW above — owed a faithful ntstrsafe port.
#[no_mangle]
pub unsafe extern "C" fn RtlStringCbPrintfExW(
    pszDest: *mut u16, cbDest: usize, _ppszDestEnd: *mut *mut u16, _pcbRemaining: *mut usize,
    _dwFlags: u32, _pszFormat: *const u16,
) -> NTSTATUS {
    if !pszDest.is_null() && cbDest >= 2 { *pszDest = 0; }
    0
}
// USB_BUS_INTERFACE_{HUB,USBDI}_GUID — usbbusif.h/hubbusif.h DEFINE_GUID data symbols. bindgen
// emitted extern decls into types.rs (both headers allowlisted) → these provide the BACKING
// definitions only. PRIVATE (no `pub`) so they don't enter the glob and clash (E0659).
#[no_mangle]
static USB_BUS_INTERFACE_HUB_GUID: GUID = GUID {
    Data1: 0xb2bb8c0a, Data2: 0x5ab4, Data3: 0x11d3,
    Data4: [0xa8, 0xcd, 0x00, 0xc0, 0x4f, 0x68, 0x74, 0x7a],
};
#[no_mangle]
static USB_BUS_INTERFACE_USBDI_GUID: GUID = GUID {
    Data1: 0xb1a96a13, Data2: 0x3de0, Data3: 0x4574,
    Data4: [0x9b, 0x01, 0xc0, 0x8f, 0xea, 0xb3, 0x18, 0xd6],
};
