//! fltmgr glue — c2rust phantom-extern macros + substrate import bridges +
//! driver-private shared structs not present in bindgen types.rs.
//! Filled during integration/compile-fix (npfs/serial/storport pattern).
#![allow(unused_imports)]

pub use crate::types::*;

use ntkrnl_sys::ddk::{UNICODE_STRING, USHORT, WCHAR, ULONG, PVOID};

// xdk/iotypes.h device types not present in ddk/constants bindgen
pub const FILE_DEVICE_WPD: ULONG = 0x00000040;

// ---- link-routing shims for translation externs (cohab .sys link) ----
// Each routes a phantom-extern decl in a module to an existing ntkrnl_sys
// inline / glue def so the symbol is real at link time.

// (1) ExInitializeFastMutex — route to the FORCEINLINE impl. Most call sites pass
// PFAST_MUTEX (the ddk-import sig); Interface.rs's lone c_void call casts `as _`.
#[inline]
pub unsafe fn ExInitializeFastMutex(m: ntkrnl_sys::ddk::PFAST_MUTEX) {
    ntkrnl_sys::inline::ex::ExInitializeFastMutex(m as _)
}

// (2) ExpChangePushlock — inline takes/returns usize; crate decl uses pointer types.
#[inline]
pub unsafe fn ExpChangePushlock(
    x: ntkrnl_sys::ddk::PEX_PUSH_LOCK,
    y: *mut core::ffi::c_void,
    z: *mut core::ffi::c_void,
) -> *mut core::ffi::c_void {
    ntkrnl_sys::inline::ex::ExpChangePushlock(x as _, y as _, z as _) as _
}

// (3) FlagOn — C macro ((_F)&(_SF))
#[inline]
pub unsafe fn FlagOn(f: ULONG, sf: ULONG) -> ULONG {
    f & sf
}

// (5) InterlockedOr — MSVC intrinsic; returns the OLD value.
#[inline]
pub unsafe fn InterlockedOr(
    d: *mut core::ffi::c_long,
    v: core::ffi::c_long,
) -> core::ffi::c_long {
    (*(d as *const core::sync::atomic::AtomicI32)).fetch_or(v, core::sync::atomic::Ordering::SeqCst)
}

// (6) IoGetCurrentIrpStackLocation / IoSkipCurrentIrpStackLocation — explicit
// re-export shadows the ddk::* bindgen extern (explicit beats glob).
pub use ntkrnl_sys::inline::tier1::{IoGetCurrentIrpStackLocation, IoSkipCurrentIrpStackLocation};

// (7) KeGetCurrentIrql — per-arch (amd64 inline CR8, i386/arm64 extern).
pub use ntkrnl_sys::ke::KeGetCurrentIrql;

// (8) RtlInitEmptyUnicodeString — explicit re-export shadows the ddk::* extern.
pub use ntkrnl_sys::inline::common::RtlInitEmptyUnicodeString;

// (9) _InterlockedExchange — MSVC intrinsic; returns the OLD value.
#[inline]
pub unsafe fn _InterlockedExchange(
    t: *mut core::ffi::c_long,
    v: core::ffi::c_long,
) -> core::ffi::c_long {
    (*(t as *const core::sync::atomic::AtomicI32)).swap(v, core::sync::atomic::Ordering::SeqCst)
}

// (link-watch) __debugbreak — software breakpoint intrinsic.
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline]
pub unsafe fn __debugbreak() {
    core::arch::asm!("int3")
}
#[cfg(target_arch = "aarch64")]
#[inline]
pub unsafe fn __debugbreak() {
    core::arch::asm!("brk #0")
}

// ---- driver-private constants (fltmgr.h / fltmgrint.h) ----
pub const FLT_MAJOR_VERSION: i32 = 0x0200; // fltmgr.h
pub const FM_TAG_FILTER: ULONG = u32::from_le_bytes(*b"FMfl"); // 'lfMF' (LE multichar)

// fltkernel.h IRP_MJ_* private extensions (UCHAR-valued)
pub const IRP_MJ_VOLUME_MOUNT: u8 = (-19i32) as u8; // ((UCHAR)-19)
pub const IRP_MJ_OPERATION_END: u8 = 0x80;

// _FLT_OBJECT_FLAGS (fltmgrint.h)
pub const FLT_OBFL_TYPE_FILTER: u32 = 0x2000000;
// _FLT_FILTER_FLAGS (fltmgrint.h)
pub const FLTFL_FILTERING_INITIATED: u32 = 2;

// IRP_MJ_CREATE_MAILSLOT (xdk/iotypes.h) used to be duplicated here. It now lives in the
// authoritative bindgen island (ntkrnl_sys::driver_ddk, added 2026-07-25 by 6ae2ae1931d for
// the match-arm sweep), and BOTH are glob-imported by Interface.rs — two different items
// under one name is `error[E0659]: ambiguous`, which broke the i386 build of this crate on
// mainline. Deleting the local copy rather than disambiguating the use site: the DDK island
// is the compiler-facing authority and a hand-written duplicate of a header constant is
// exactly the drift `verify_constants` exists to prevent. The sole use site casts
// `as usize`, so the island's `u8` spelling is equivalent there.

// FLT_ASSERT(c) — debug-assert macro; no-op in retail (NDEBUG), like NT_ASSERT.
#[inline]
pub unsafe fn FLT_ASSERT(_cond: i32) {}

// RtlAssert is ntoskrnl's export: it prints the failed assertion and returns. An
// empty local definition here won the link and made every failed ASSERT in this
// driver vanish (scan_export_shadows).
extern "system" {
    pub fn RtlAssert(FailedAssertion: PVOID, FileName: PVOID, LineNumber: ULONG, Message: *mut ntkrnl_sys::ddk::CHAR);
}

// ---- driver-private IOCTL structs (sdk/include/reactos/drivers/fltmgr/fltmgr_shared.h) ----
// Not emitted by bindgen (driver-private header); defined faithfully here.

#[repr(C)]
pub struct _FILTER_NAME {
    pub Length: USHORT,
    pub FilterName: [WCHAR; 1],
}
pub type FILTER_NAME = _FILTER_NAME;
pub type PFILTER_NAME = *mut _FILTER_NAME;

#[repr(C)]
pub struct _FILTER_PORT_DATA {
    pub Size: ULONG,
    pub PortName: UNICODE_STRING,
    pub Options: ULONG,
    pub Context: PVOID,
    pub ContextSize: ULONG,
}
pub type FILTER_PORT_DATA = _FILTER_PORT_DATA;
pub type PFILTER_PORT_DATA = *mut _FILTER_PORT_DATA;

// ---- driver-private structs (drivers/filters/fltmgr/fltmgr.h) ----
// Not emitted by bindgen (driver-internal header).
use ntkrnl_sys::ddk::{PDRIVER_OBJECT, PDEVICE_OBJECT, PFAST_IO_DISPATCH, FAST_MUTEX};

#[repr(C)]
pub struct _DRIVER_DATA {
    pub DriverObject: PDRIVER_OBJECT,
    pub DeviceObject: PDEVICE_OBJECT,
    pub ServiceKey: UNICODE_STRING,
    pub CommsDeviceObject: PDEVICE_OBJECT,
    pub FastIoDispatch: PFAST_IO_DISPATCH,
    pub FilterAttachLock: FAST_MUTEX,
}
pub type DRIVER_DATA = _DRIVER_DATA;
pub type PDRIVER_DATA = *mut _DRIVER_DATA;

pub const MAX_DEVNAME_LENGTH: usize = 64;

#[repr(C)]
pub struct _FLTMGR_DEVICE_EXTENSION {
    pub AttachedToDeviceObject: PDEVICE_OBJECT,
    pub StorageStackDeviceObject: PDEVICE_OBJECT,
    pub DeviceName: UNICODE_STRING,
    pub DeviceNameBuffer: [WCHAR; MAX_DEVNAME_LENGTH],
}
pub type FLTMGR_DEVICE_EXTENSION = _FLTMGR_DEVICE_EXTENSION;
pub type PFLTMGR_DEVICE_EXTENSION = *mut _FLTMGR_DEVICE_EXTENSION;
