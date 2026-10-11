//! arbiter-rust prelude — the single import surface the translated modules use. The bindgen
//! island (`types`, from arbiter.h) carries the library's own constants; ARBITER_INSTANCE and
//! the rest of the kernel types keep their single ntkrnl-sys identity.
#![allow(unused_imports)]

pub use crate::types::*;
pub use core::ffi::{c_char, c_void};
pub use ntkrnl_sys::ddk::*;
pub use ntkrnl_sys::CONTAINING_RECORD;
// A FORCEINLINE in rtlfuncs.h: no export to link, so the Rust inline, never the bindgen extern.
pub use ntkrnl_sys::inline::common::RtlInitEmptyUnicodeString;
pub use ntkrnl_sys::base::{
    InitializeListHead, InsertTailList, IsListEmpty, RemoveHeadList, APC_LEVEL, FALSE, NT_SUCCESS,
    PAGE_SIZE, STATUS_INSUFFICIENT_RESOURCES, STATUS_NOT_IMPLEMENTED, STATUS_SUCCESS, TRUE,
};
pub use ntkrnl_sys::constants::{
    IO_NO_INCREMENT, KEY_READ, OBJ_CASE_INSENSITIVE, OBJ_KERNEL_HANDLE, REG_SZ,
    STATUS_BUFFER_OVERFLOW, STATUS_BUFFER_TOO_SMALL, STATUS_INVALID_PARAMETER,
    STATUS_UNSUCCESSFUL,
};
pub use ntkrnl_sys::driver_ddk::{DbgPrint, UNICODE_NULL};
pub use ntkrnl_sys::ke::KeQueryInterruptTime;
pub use ntkrnl_sys::mm::{NonPagedPool, PagedPool};
pub use ntkrnl_sys::rtl::RtlCopyMemory;
pub use win32macros_rust::{DPRINT1, RTL_CONSTANT_STRING, UNREFERENCED_PARAMETER};

// wdm.h's PAGED_CODE: NOP_FUNCTION in this build's headers. A macro, never an out-of-line fn:
// an out-of-line no-op is an optimization barrier.
macro_rules! PAGED_CODE {
    () => {{}};
}
pub(crate) use PAGED_CODE;

// arbiter.h defines bindgen cannot express: a multi-character tag and a cast expression.
/// `#define TAG_ARBITER 'ibrA'` — a C multi-character constant, first character highest.
pub const TAG_ARBITER: ULONG = u32::from_be_bytes(*b"ibrA");
/// `#define ARBITER_MAXIMUM_ADDRESS ((ULONGLONG)~0ULL)`
pub const ARBITER_MAXIMUM_ADDRESS: ULONGLONG = !0;
/// ntstatus.h: `#define STATUS_BAD_MCFG_TABLE ((NTSTATUS)0xC0000908)` (a cast, so bindgen
/// emits nothing for it).
pub const STATUS_BAD_MCFG_TABLE: NTSTATUS = 0xC0000908u32 as NTSTATUS;

/// ntstatus.h: `#define STATUS_DEVICE_CONFIGURATION_ERROR ((NTSTATUS)0xC0000182)`
pub const STATUS_DEVICE_CONFIGURATION_ERROR: NTSTATUS = 0xC0000182u32 as NTSTATUS;

// Enum members the C spells bare; bindgen emits them only under their enum's prefix.
pub use ntkrnl_sys::ddk::{
    _ARBITER_ACTION_ArbiterActionAddReserved as ArbiterActionAddReserved,
    _ARBITER_ACTION_ArbiterActionBootAllocation as ArbiterActionBootAllocation,
    _ARBITER_ACTION_ArbiterActionCommitAllocation as ArbiterActionCommitAllocation,
    _ARBITER_ACTION_ArbiterActionQueryAllocatedResources as ArbiterActionQueryAllocatedResources,
    _ARBITER_ACTION_ArbiterActionQueryArbitrate as ArbiterActionQueryArbitrate,
    _ARBITER_ACTION_ArbiterActionQueryConflict as ArbiterActionQueryConflict,
    _ARBITER_ACTION_ArbiterActionRetestAllocation as ArbiterActionRetestAllocation,
    _ARBITER_ACTION_ArbiterActionRollbackAllocation as ArbiterActionRollbackAllocation,
    _ARBITER_ACTION_ArbiterActionTestAllocation as ArbiterActionTestAllocation,
    _ARBITER_ACTION_ArbiterActionWriteReservedResources as ArbiterActionWriteReservedResources,
    _ARBITER_REQUEST_SOURCE_ArbiterRequestLegacyAssigned as ArbiterRequestLegacyAssigned,
    _ARBITER_REQUEST_SOURCE_ArbiterRequestLegacyReported as ArbiterRequestLegacyReported,
    _ARBITER_REQUEST_SOURCE_ArbiterRequestPnpEnumerated as ArbiterRequestPnpEnumerated,
    _ARBITER_RESULT_ArbiterResultExternalConflict as ArbiterResultExternalConflict,
    _ARBITER_RESULT_ArbiterResultNullRequest as ArbiterResultNullRequest,
    _ARBITER_RESULT_ArbiterResultSuccess as ArbiterResultSuccess,
    _CM_SHARE_DISPOSITION_CmResourceShareDriverExclusive as CmResourceShareDriverExclusive,
    _CM_SHARE_DISPOSITION_CmResourceShareShared as CmResourceShareShared,
    DEVICE_REGISTRY_PROPERTY_DevicePropertyBusNumber as DevicePropertyBusNumber,
    DEVICE_REGISTRY_PROPERTY_DevicePropertyEnumeratorName as DevicePropertyEnumeratorName,
    DEVICE_REGISTRY_PROPERTY_DevicePropertyLegacyBusType as DevicePropertyLegacyBusType,
    _KEY_VALUE_INFORMATION_CLASS_KeyValueFullInformationAlign64 as KeyValueFullInformationAlign64,
    _INTERFACE_TYPE_Isa as Isa,
};

/// windef.h's `min`/`max` macros (each argument evaluated once here; the C call sites pass
/// side-effect-free expressions, so the single evaluation changes nothing).
#[inline(always)]
pub fn min<T: PartialOrd>(a: T, b: T) -> T {
    if a < b { a } else { b }
}
#[inline(always)]
pub fn max<T: PartialOrd>(a: T, b: T) -> T {
    if a > b { a } else { b }
}
