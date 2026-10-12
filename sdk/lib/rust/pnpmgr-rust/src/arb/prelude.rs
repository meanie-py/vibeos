//! Import surface for the root arbiters (ntoskrnl/io/pnpmgr/arb/*.c). The instances themselves
//! are pnpinit.c's globals (the C declares them `extern` here), so they come from crate::pnpinit.
#![allow(unused_imports)]

pub use core::ffi::c_void;
pub use crate::pnpinit::{
    IopRootBusNumberArbiter, IopRootDmaArbiter, IopRootIrqArbiter, IopRootMemArbiter,
    IopRootPortArbiter,
};
pub use ntkrnl_sys::ddk::*;
pub use ntkrnl_sys::base::{FALSE, NT_SUCCESS, STATUS_SUCCESS, TRUE};
pub use ntkrnl_sys::constants::{STATUS_INVALID_PARAMETER, STATUS_UNSUCCESSFUL};
pub use win32macros_rust::DPRINT1;

// wdm.h's PAGED_CODE: NOP_FUNCTION in this build's headers. A macro, never an out-of-line fn.
macro_rules! PAGED_CODE {
    () => {{}};
}
pub(crate) use PAGED_CODE;

// Enum members the C spells bare; bindgen emits them only under their enum's prefix.
pub use ntkrnl_sys::ddk::_INTERFACE_TYPE_Isa as Isa;

// sdk/lib/drivers/arbiter/arbiter.h — the library is arbiter-rust, a separate staticlib linked
// into the kernel, so its entry points are declared here exactly as the header does.
extern "system" {
    pub fn ArbiterLibInitializeInstance(
        Arbiter: PARBITER_INSTANCE,
        BusDeviceObject: PDEVICE_OBJECT,
        ResourceType: CM_RESOURCE_TYPE,
        ArbiterName: PCWSTR,
        OrderName: PCWSTR,
        TranslateOrderingFunction: PARB_TRANSLATE_ORDERING,
    ) -> NTSTATUS;
    pub fn ArbiterLibFindSuitableRange(
        Arbiter: PARBITER_INSTANCE,
        ArbState: PARBITER_ALLOCATION_STATE,
    ) -> BOOLEAN;
}

// xdk/cmtypes.h's resource constants, shared from ntkrnl-sys on every arch.
pub use ntkrnl_sys::cmres::{
    CmResourceTypeBusNumber, CmResourceTypeDma, CmResourceTypeInterrupt, CmResourceTypeMemory,
    CmResourceTypeMemoryLarge, CmResourceTypeNull, CmResourceTypePort, CM_RESOURCE_MEMORY_24,
    CM_RESOURCE_PORT_10_BIT_DECODE, CM_RESOURCE_PORT_12_BIT_DECODE,
};
// Header constants with no shared binding yet; scan_const_vs_island checks each against the
// islands that do carry it.
/// ntddk.h
pub const ARBITER_FLAG_BOOT_CONFIG: ULONG = 0x00000001;
pub const RTL_RANGE_LIST_ADD_IF_CONFLICT: ULONG = 0x00000001;
pub const RTL_RANGE_LIST_ADD_SHARED: ULONG = 0x00000002;
/// sdk/lib/drivers/arbiter/arbiter.h
pub const ARBITER_ALTERNATIVE_FLAG_SHARED: ULONG = 0x00000001;
pub const ARBITER_RANGE_BOOT_ALLOCATED: UCHAR = 0x01;
pub const ARBITER_RANGE_PORT_ALIAS: UCHAR = 0x10;
/// ntdef.h: `#define MAXLONG 0x7fffffff`, LONG's maximum
pub const MAXLONG: LONG = LONG::MAX;

/// windef.h's `min` macro (side-effect-free arguments at every call site here).
#[inline(always)]
pub fn min<T: PartialOrd>(a: T, b: T) -> T {
    if a < b { a } else { b }
}
