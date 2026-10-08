//! usbccgp glue — c2rust phantom-extern bridges + cohab shims not in the bindgen types.rs.
//! Filled during integration/compile-fix (usbstor pattern, minus SCSI). DbgPrint/RtlAssert
//! resolve from the ntoskrnl import lib (modules declare them in their own extern blocks).
#![allow(unused_imports)]

pub use crate::types::*;

// Basic kernel types the modules name bare but only import partially from ddk (PCSTR etc.).
// PCHAR/PUCHAR/PCWSTR EXCLUDED (arrive via base/driver_ddk globs → would be ambiguous).
pub use ntkrnl_sys::ddk::{
    CCHAR, KPRIORITY, KPROCESSOR_MODE, PCSTR, PCWCH, PDRIVER_DISPATCH, PWCH, PWSTR,
    _UNICODE_STRING, LANGID, GUID,
};
pub type size_t = usize;
pub type NTSTRSAFE_PWSTR = ntkrnl_sys::ddk::PWSTR;
pub type NTSTRSAFE_PCWSTR = ntkrnl_sys::ddk::PCWSTR;

// ---- inline-only DDK helpers (no exported symbol; C inlines them per-TU; undefined at cohab
// link). Faithful out-of-line shims, decoration matching the undefined symbols (cdecl `_Name`
// except ExAllocatePoolZero stdcall `_Name@12`). IRP-stack helpers wrap ntkrnl_sys::inline. ----
use ntkrnl_sys::ddk::{
    PIRP, PIO_STACK_LOCATION, PIO_COMPLETION_ROUTINE, POOL_TYPE,
    PVOID, ULONG, USHORT, UCHAR, PMDL,
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
// IoSetCompletionRoutine — set NEXT stack location's completion routine + invoke flags
// (SL_INVOKE_ON_{SUCCESS,ERROR,CANCEL} = 0x40/0x80/0x20).
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
// CC = ntkrnl-sys bindgen_i386 decl class (callers import THAT decl; a mismatched
// shim def in the same crate folds every such call to ud2 — the 2026-07-18 i386
// driver-fleet bomb class).
#[no_mangle]
pub unsafe extern "C" fn KeInitializeSpinLock(SpinLock: *mut usize) {
    *SpinLock = 0;
}
// ExAllocatePoolZero — allocate-and-zero (wdm.h inline, not exported here). stdcall `@12`.
#[no_mangle]
pub unsafe extern "system" fn ExAllocatePoolZero(
    PoolType: POOL_TYPE, NumberOfBytes: ntkrnl_sys::ddk::SIZE_T, Tag: ULONG,
) -> PVOID {
    ntkrnl_sys::inline::mm::ExAllocatePoolZero(PoolType, NumberOfBytes as _, Tag)
}
// RtlAssert is ntoskrnl's export: it prints the failed assertion and returns. An
// empty local definition here won the link and made every failed ASSERT in this
// driver vanish (scan_export_shadows).
extern "system" {
    pub fn RtlAssert(FailedAssertion: PVOID, FileName: PVOID, LineNumber: ULONG, Message: *mut ntkrnl_sys::ddk::CHAR);
}
// UsbBuildGetDescriptorRequest — usbdlib.h macro; URB_FUNCTION_GET_DESCRIPTOR_FROM_DEVICE=0x000B.
#[no_mangle]
pub unsafe extern "C" fn UsbBuildGetDescriptorRequest(
    Urb: PURB, Length: USHORT, DescriptorType: UCHAR, Index: UCHAR, LanguageId: USHORT,
    TransferBuffer: PVOID, TransferBufferMDL: PVOID, TransferBufferLength: ULONG, UrbLink: PURB,
) {
    let hdr = (*Urb).__bindgen_anon_1.UrbHeader.as_mut();
    (*hdr).Function = 0x000B;
    (*hdr).Length = Length;
    let r = (*Urb).__bindgen_anon_1.UrbControlDescriptorRequest.as_mut();
    (*r).TransferBufferLength = TransferBufferLength;
    (*r).TransferBufferMDL = TransferBufferMDL as PMDL;
    (*r).TransferBuffer = TransferBuffer;
    (*r).DescriptorType = DescriptorType;
    (*r).Index = Index;
    (*r).LanguageId = LanguageId;
    (*r).UrbLink = UrbLink;
}
// UsbBuildSelectInterfaceRequest — usbdlib.h macro; URB_FUNCTION_SELECT_INTERFACE=0x0001.
// Interface.Length = TotalLength - sizeof(_URB_HEADER) - sizeof(USBD_CONFIGURATION_HANDLE).
#[no_mangle]
pub unsafe extern "C" fn UsbBuildSelectInterfaceRequest(
    Urb: PURB, TotalLength: USHORT, ConfigurationHandle: USBD_CONFIGURATION_HANDLE,
    InterfaceNumber: UCHAR, AlternateSetting: UCHAR,
) {
    let hdr = (*Urb).__bindgen_anon_1.UrbHeader.as_mut();
    (*hdr).Function = 0x0001;
    (*hdr).Length = TotalLength;
    let r = (*Urb).__bindgen_anon_1.UrbSelectInterface.as_mut();
    (*r).Interface.AlternateSetting = AlternateSetting;
    (*r).Interface.InterfaceNumber = InterfaceNumber;
    (*r).Interface.Length = (TotalLength as usize
        - core::mem::size_of::<_URB_HEADER>()
        - core::mem::size_of::<USBD_CONFIGURATION_HANDLE>()) as USHORT;
    (*r).ConfigurationHandle = ConfigurationHandle;
}

// USB_BUS_INTERFACE_USBC_CONFIGURATION_GUID — usbbusif.h DEFINE_GUID. bindgen already emitted
// an `extern static` decl for it into types.rs (usbbusif.h is allowlisted), so this provides
// the BACKING definition only. Kept PRIVATE (no `pub`) so it does NOT enter glue's glob and
// clash with the types.rs decl (E0659); #[no_mangle] still emits the symbol the decl binds to.
// No arch provides it externally (nm over build-i386: nothing defines it — the old
// "i386 import floor" note was wrong) -> define everywhere.
#[no_mangle]
static USB_BUS_INTERFACE_USBC_CONFIGURATION_GUID: GUID = GUID {
    Data1: 0x893b6a96,
    Data2: 0x0b7f,
    Data3: 0x4d4d,
    Data4: [0xbd, 0xb4, 0xbb, 0xd4, 0xce, 0xeb, 0xb3, 0x1c],
};
