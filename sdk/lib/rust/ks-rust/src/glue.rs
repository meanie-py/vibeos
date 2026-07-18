//! ks-rust glue: bare-named enum-member aliases (bindgen names them `<EnumTag>_<member>`),
//! KS #define constants, and shared macro helpers the DeepSeek translation references bare.
//! Per-file logic/types stay in their modules; only genuinely SHARED symbols live here.
//! Constant values are header-sourced (sdk/include/psdk/ks.h, sdk/include/ddk/swenum.h);
//! verify_constants + Haiku audit are the correctness net.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code)]
use ntkrnl_sys::base::{ULONG, USHORT};

// ---- KS IOCTL codes (CTL_CODE-derived; FILE_DEVICE_KS=0x2f per sdk/include/psdk/ks.h) ----
pub const FILE_DEVICE_KS: ULONG = 0x2f;
const METHOD_NEITHER: u32 = 3;
const FILE_ANY_ACCESS: u32 = 0;
const FILE_READ_ACCESS: u32 = 1;
const FILE_WRITE_ACCESS: u32 = 2;
pub const fn CTL_CODE(DeviceType: u32, Function: u32, Method: u32, Access: u32) -> u32 {
    (DeviceType << 16) | (Access << 14) | (Function << 2) | Method
}
pub const IOCTL_KS_PROPERTY: ULONG = CTL_CODE(FILE_DEVICE_KS, 0x000, METHOD_NEITHER, FILE_ANY_ACCESS);
pub const IOCTL_KS_ENABLE_EVENT: ULONG = CTL_CODE(FILE_DEVICE_KS, 0x001, METHOD_NEITHER, FILE_ANY_ACCESS);
pub const IOCTL_KS_DISABLE_EVENT: ULONG = CTL_CODE(FILE_DEVICE_KS, 0x002, METHOD_NEITHER, FILE_ANY_ACCESS);
pub const IOCTL_KS_METHOD: ULONG = CTL_CODE(FILE_DEVICE_KS, 0x003, METHOD_NEITHER, FILE_ANY_ACCESS);
pub const IOCTL_KS_WRITE_STREAM: ULONG = CTL_CODE(FILE_DEVICE_KS, 0x004, METHOD_NEITHER, FILE_WRITE_ACCESS);
pub const IOCTL_KS_READ_STREAM: ULONG = CTL_CODE(FILE_DEVICE_KS, 0x005, METHOD_NEITHER, FILE_READ_ACCESS);
pub const IOCTL_KS_RESET_STATE: ULONG = CTL_CODE(FILE_DEVICE_KS, 0x006, METHOD_NEITHER, FILE_ANY_ACCESS);

// ---- misc KS #defines ----
pub const KSFILTER_NODE: ULONG = 0xFFFFFFFF;                 // (ULONG)-1, ks.h
pub const BUS_INTERFACE_SWENUM_VERSION: USHORT = 0x100;      // sdk/include/ddk/swenum.h

// ---- auto-generated bare enum-member aliases (bindgen <EnumTag>_<member> -> bare member) ----
pub use crate::types::KSINTERFACE_STANDARD_KSINTERFACE_STANDARD_STREAMING as KSINTERFACE_STANDARD_STREAMING;
pub use crate::types::KSPIN_COMMUNICATION_KSPIN_COMMUNICATION_NONE as KSPIN_COMMUNICATION_NONE;
pub use crate::types::KSPIN_DATAFLOW_KSPIN_DATAFLOW_IN as KSPIN_DATAFLOW_IN;
pub use crate::types::KSPROPERTY_CLOCK_KSPROPERTY_CLOCK_CORRELATEDPHYSICALTIME as KSPROPERTY_CLOCK_CORRELATEDPHYSICALTIME;
pub use crate::types::KSPROPERTY_CLOCK_KSPROPERTY_CLOCK_CORRELATEDTIME as KSPROPERTY_CLOCK_CORRELATEDTIME;
pub use crate::types::KSPROPERTY_CLOCK_KSPROPERTY_CLOCK_FUNCTIONTABLE as KSPROPERTY_CLOCK_FUNCTIONTABLE;
pub use crate::types::KSPROPERTY_CLOCK_KSPROPERTY_CLOCK_PHYSICALTIME as KSPROPERTY_CLOCK_PHYSICALTIME;
pub use crate::types::KSPROPERTY_CLOCK_KSPROPERTY_CLOCK_RESOLUTION as KSPROPERTY_CLOCK_RESOLUTION;
pub use crate::types::KSPROPERTY_CLOCK_KSPROPERTY_CLOCK_STATE as KSPROPERTY_CLOCK_STATE;
pub use crate::types::KSPROPERTY_CLOCK_KSPROPERTY_CLOCK_TIME as KSPROPERTY_CLOCK_TIME;
pub use crate::types::KSPROPERTY_CONNECTION_KSPROPERTY_CONNECTION_ALLOCATORFRAMING_EX as KSPROPERTY_CONNECTION_ALLOCATORFRAMING_EX;
pub use crate::types::KSPROPERTY_CONNECTION_KSPROPERTY_CONNECTION_DATAFORMAT as KSPROPERTY_CONNECTION_DATAFORMAT;
pub use crate::types::KSPROPERTY_CONNECTION_KSPROPERTY_CONNECTION_STATE as KSPROPERTY_CONNECTION_STATE;
pub use crate::types::KSPROPERTY_GENERAL_KSPROPERTY_GENERAL_COMPONENTID as KSPROPERTY_GENERAL_COMPONENTID;
pub use crate::types::KSPROPERTY_PIN_KSPROPERTY_PIN_CTYPES as KSPROPERTY_PIN_CTYPES;
pub use crate::types::KSPROPERTY_STREAMALLOCATOR_KSPROPERTY_STREAMALLOCATOR_FUNCTIONTABLE as KSPROPERTY_STREAMALLOCATOR_FUNCTIONTABLE;
pub use crate::types::KSPROPERTY_STREAMALLOCATOR_KSPROPERTY_STREAMALLOCATOR_STATUS as KSPROPERTY_STREAMALLOCATOR_STATUS;
pub use crate::types::KSPROPERTY_STREAM_KSPROPERTY_STREAM_ALLOCATOR as KSPROPERTY_STREAM_ALLOCATOR;
pub use crate::types::KSPROPERTY_STREAM_KSPROPERTY_STREAM_MASTERCLOCK as KSPROPERTY_STREAM_MASTERCLOCK;
pub use crate::types::KSPROPERTY_STREAM_KSPROPERTY_STREAM_PIPE_ID as KSPROPERTY_STREAM_PIPE_ID;
pub use crate::types::KSPROPERTY_TOPOLOGY_KSPROPERTY_TOPOLOGY_CATEGORIES as KSPROPERTY_TOPOLOGY_CATEGORIES;
pub use crate::types::KSPROPERTY_TOPOLOGY_KSPROPERTY_TOPOLOGY_CONNECTIONS as KSPROPERTY_TOPOLOGY_CONNECTIONS;
pub use crate::types::KSPROPERTY_TOPOLOGY_KSPROPERTY_TOPOLOGY_NAME as KSPROPERTY_TOPOLOGY_NAME;
pub use crate::types::KSPROPERTY_TOPOLOGY_KSPROPERTY_TOPOLOGY_NODES as KSPROPERTY_TOPOLOGY_NODES;
pub use crate::types::KSRESET_KSRESET_END as KSRESET_END;
pub use crate::types::KSSTATE_KSSTATE_RUN as KSSTATE_RUN;
pub use crate::types::KSSTATE_KSSTATE_STOP as KSSTATE_STOP;
pub use crate::types::KSSTREAM_POINTER_STATE_KSSTREAM_POINTER_STATE_LOCKED as KSSTREAM_POINTER_STATE_LOCKED;
pub use crate::types::KSTARGET_STATE_KSTARGET_STATE_DISABLED as KSTARGET_STATE_DISABLED;
pub use crate::types::KSTARGET_STATE_KSTARGET_STATE_ENABLED as KSTARGET_STATE_ENABLED;
// KSEVENTS_LOCKTYPE enum members (bindgen: KSEVENTS_LOCKTYPE_KSEVENTS_*)
pub use crate::types::KSEVENTS_LOCKTYPE_KSEVENTS_NONE as KSEVENTS_NONE;
pub use crate::types::KSEVENTS_LOCKTYPE_KSEVENTS_SPINLOCK as KSEVENTS_SPINLOCK;
pub use crate::types::KSEVENTS_LOCKTYPE_KSEVENTS_MUTEX as KSEVENTS_MUTEX;
pub use crate::types::KSEVENTS_LOCKTYPE_KSEVENTS_FMUTEX as KSEVENTS_FMUTEX;
pub use crate::types::KSEVENTS_LOCKTYPE_KSEVENTS_FMUTEXUNSAFE as KSEVENTS_FMUTEXUNSAFE;
pub use crate::types::KSEVENTS_LOCKTYPE_KSEVENTS_INTERRUPT as KSEVENTS_INTERRUPT;
pub use crate::types::KSEVENTS_LOCKTYPE_KSEVENTS_ERESOURCE as KSEVENTS_ERESOURCE;
// KSOBJECTTYPE enum members
pub use crate::types::KSOBJECTTYPE_KsObjectTypeDevice as KsObjectTypeDevice;
pub use crate::types::KSOBJECTTYPE_KsObjectTypeFilterFactory as KsObjectTypeFilterFactory;
pub use crate::types::KSOBJECTTYPE_KsObjectTypeFilter as KsObjectTypeFilter;
pub use crate::types::KSOBJECTTYPE_KsObjectTypePin as KsObjectTypePin;
// DEVICE_STATE enum members
pub use crate::types::DEVICE_STATE_NotStarted as NotStarted;
pub use crate::types::DEVICE_STATE_Started as Started;
pub use crate::types::DEVICE_STATE_StopPending as StopPending;
pub use crate::types::DEVICE_STATE_NotStarted as _DEVICE_STATE_NotStarted;


// ---- standard kernel types re-exported from ntkrnl_sys::ddk (bindgen layer) ----
// These are bare-referenced by the translation but not glob-imported per-file. Safe to
// re-export (they are NOT in crate::types, so no glob ambiguity vs `use crate::types::*`).
pub use ntkrnl_sys::ddk::{
    _UNICODE_STRING, CCHAR, IID, PCSTR, PIO_STATUS_BLOCK, PLARGE_INTEGER, UINT,
    PPAGED_LOOKASIDE_LIST, PNPAGED_LOOKASIDE_LIST, PDRIVER_DISPATCH, PWCH, PFILE_OBJECT,
    LPWSTR, KPROCESSOR_MODE, KMUTEX, PKMUTEX, PRKMUTEX, PUNICODE_STRING, KEVENT, KTIMER,
    ACCESS_MASK, KPRIORITY, PCUNICODE_STRING, PDMA_ADAPTER, NPAGED_LOOKASIDE_LIST,
    PAGED_LOOKASIDE_LIST, WORK_QUEUE_ITEM, CLSID, GUID, PWORKER_THREAD_ROUTINE,
    NTSTRSAFE_PCWSTR, NTSTRSAFE_PWSTR,
};
use ntkrnl_sys::base::NTSTATUS;
pub type LPCVOID = *const core::ffi::c_void;
pub type REFCLSID = *const GUID;

// IUnknown IID = {00000000-0000-0000-C000-000000000046}
pub static IID_IUnknown: GUID = GUID {
    Data1: 0x00000000,
    Data2: 0x0000,
    Data3: 0x0000,
    Data4: [0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46],
};

// ---- NTSTATUS codes (header-sourced: sdk/include ntstatus) ----
pub const STATUS_NO_MATCH: NTSTATUS = 0xC0000272u32 as NTSTATUS;
pub const STATUS_DEVICE_REMOVED: NTSTATUS = 0xC00002B6u32 as NTSTATUS;
pub const STATUS_DEVICE_NOT_READY: NTSTATUS = 0xC00000A3u32 as NTSTATUS;
pub const STATUS_INVALID_BUFFER_SIZE: NTSTATUS = 0xC0000206u32 as NTSTATUS;
pub const STATUS_INVALID_DEVICE_STATE: NTSTATUS = 0xC0000184u32 as NTSTATUS;
pub const STATUS_MORE_PROCESSING_REQUIRED: NTSTATUS = 0xC0000016u32 as NTSTATUS;
pub const STATUS_NOINTERFACE: NTSTATUS = 0xC00002B9u32 as NTSTATUS;
pub const STATUS_PROPSET_NOT_FOUND: NTSTATUS = 0xC0000230u32 as NTSTATUS;

// ---- IRP minor function codes (UCHAR; sdk/include) ----
pub const IRP_MN_QUERY_INTERFACE: u8 = 0x08;
pub const IRP_MN_QUERY_CAPABILITIES: u8 = 0x09;
pub const IRP_MN_QUERY_RESOURCES: u8 = 0x0A;
pub const IRP_MN_QUERY_RESOURCE_REQUIREMENTS: u8 = 0x0B;
pub const IRP_MN_READ_CONFIG: u8 = 0x0F;
pub const IRP_MN_WRITE_CONFIG: u8 = 0x10;
pub const IRP_MN_QUERY_ID: u8 = 0x13;
pub const IRP_MN_QUERY_PNP_DEVICE_STATE: u8 = 0x14;
pub const IRP_MN_QUERY_BUS_INFORMATION: u8 = 0x15;

// ---- misc flags / privileges (ULONG; header-sourced) ----
pub const FO_SYNCHRONOUS_IO: ULONG = 0x00000002;
pub const SE_LOAD_DRIVER_PRIVILEGE: ULONG = 10;
pub const KEY_WRITE: ULONG = 0x20006;
pub const MDL_MAPPED_TO_SYSTEM_VA: u16 = 0x0001;
pub const MDL_SOURCE_IS_NONPAGED_POOL: u16 = 0x0004;
pub const MDL_MAPPING_CAN_FAIL: u16 = 0x2000;
pub const IO_FORCE_ACCESS_CHECK: ULONG = 0x001;
pub const IO_NO_PARAMETER_CHECKING: ULONG = 0x100;
pub const IO_REPARSE: ULONG = 0x0;
pub const IRP_BUFFERED_IO: ULONG = 0x00000010;
pub const IRP_DEALLOCATE_BUFFER: ULONG = 0x00000020;
pub const IRP_INPUT_OPERATION: ULONG = 0x00000040;
pub const PLUGPLAY_REGKEY_DEVICE: ULONG = 1;
pub const PNP_DEVICE_DONT_DISPLAY_IN_UI: ULONG = 0x00000002;
pub const PNP_DEVICE_NOT_DISABLEABLE: ULONG = 0x00000020;

// ---- FORCEINLINE DDK helpers (no real export) -> #[no_mangle] wrappers over ntkrnl-sys
// inlines, so the cohab ks.sys link resolves them (mirrors usbport-rust / usbaudio-rust glue).
use ntkrnl_sys::base::{BOOLEAN, PVOID};
use ntkrnl_sys::ddk::{PIRP, PIO_STACK_LOCATION, PDRIVER_CANCEL, PIO_COMPLETION_ROUTINE};
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
    Irp: PIRP,
    CompletionRoutine: PIO_COMPLETION_ROUTINE,
    Context: PVOID,
    InvokeOnSuccess: BOOLEAN,
    InvokeOnError: BOOLEAN,
    InvokeOnCancel: BOOLEAN,
) {
    ntkrnl_sys::inline::tier1::IoSetCompletionRoutine(
        Irp, CompletionRoutine, Context, InvokeOnSuccess, InvokeOnError, InvokeOnCancel,
    )
}
#[no_mangle]
pub unsafe extern "system" fn KeInitializeSpinLock(SpinLock: *mut usize) {
    *SpinLock = 0;
}

// RtlInterlockedPush/PopEntrySList: provided by the `slist` crate (canonical lock-free SList
// impl with correct per-arch decoration), pulled in via `extern crate slist;` in lib.rs —
// ntoskrnl only exports them under fastcall/Exp* aliases, so ks.sys carries its own copy.

// KeQueryTickCount is exported only on i386/arm; on amd64/arm64 it is an inline reading
// KUSER_SHARED_DATA.TickCount. Provide the non-x86 definition (cdrom-rust precedent).
#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn KeQueryTickCount(Count: *mut ntkrnl_sys::ddk::LARGE_INTEGER) {
    const TICK: usize = 0xFFFFF78000000000 + 0x320; // KUSER_SHARED_DATA.TickCount
    loop {
        let high1 = core::ptr::read_volatile((TICK + 4) as *const i32);
        let low = core::ptr::read_volatile(TICK as *const u32);
        let high2 = core::ptr::read_volatile((TICK + 8) as *const i32);
        if high1 == high2 {
            (*Count).QuadPart = ((high1 as i64) << 32) | (low as i64 & 0xFFFF_FFFF);
            return;
        }
    }
}

// RtlRaiseStatus is not exported by ntoskrnl (only ExRaiseStatus exists) -> forward to it.
extern "system" {
    fn ExRaiseStatus(Status: NTSTATUS);
}
#[no_mangle]
pub unsafe extern "system" fn RtlRaiseStatus(Status: NTSTATUS) {
    ExRaiseStatus(Status);
}
