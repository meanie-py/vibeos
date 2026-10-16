//! storport-rust prelude — the single import surface the translated modules use.
//! Bridges the bindgen `types` island and the translated C. Anything bindgen leaves out
//! (CRT, inlines, sibling-C externs, hand-transcribed consts) is declared here ONCE.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

pub use crate::types::*;
pub use core::ffi::{c_char, c_int, c_long, c_short, c_uchar, c_uint, c_ushort, c_void};
// C's `size_t`: bindgen spells it `usize` inline, so no island carries the name the C uses.
pub type size_t = usize;
pub use core::ptr::{addr_of, addr_of_mut};

// ================= debug ======================================================================
// reactos/debug.h DPRINT/UNIMPLEMENTED — no-op MACROS, never out-of-line fns
// (field manual §9.65: an out-of-line no-op is an optimization barrier). debug_macros.py
// rewrites C's bare `UNIMPLEMENTED;` to `UNIMPLEMENTED!()`, so the prelude must have it.
// DPRINT1 PRINTS in this DBG=1 tree, kernel and user mode alike: it comes from
// win32macros-rust with the other macros.
#[macro_export] macro_rules! DPRINT  { ($($arg:tt)*) => {{}}; }
#[macro_export] macro_rules! UNIMPLEMENTED { ($($arg:tt)*) => {{}}; }
#[macro_export] macro_rules! UNIMPLEMENTED_ONCE { ($($arg:tt)*) => {{}}; }
pub use crate::{DPRINT, UNIMPLEMENTED, UNIMPLEMENTED_ONCE};

// ================= CRT ========================================================================
// NEVER hand-write memcpy/memset/memmove/memcmp in Rust: LLVM folds any byte-copy loop back into
// a call to the function itself → infinite self-recursion → silent hang. Import them (AGENTS.md).
extern "C" {
    pub fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    pub fn memset(dest: *mut c_void, c: c_int, n: usize) -> *mut c_void;
    pub fn memmove(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    pub fn memcmp(a: *const c_void, b: *const c_void, n: usize) -> c_int;
}

// bindgen does not emit NULL (a macro) nor NT_SUCCESS (an inline).
pub const NULL: *mut c_void = core::ptr::null_mut();
#[inline(always)]
pub fn NT_SUCCESS(s: NTSTATUS) -> bool {
    s >= 0
}
// Translations reach for the C MACRO forms too (`NT_SUCCESS!(st)`, `CONTAINING_RECORD!(p, T, f)`)
// and a fresh crate otherwise starts with `cannot find macro` in every module that walks a list.
// Address arithmetic, so neither needs an `unsafe` context of its own.
#[macro_export] macro_rules! NT_SUCCESS { ($s:expr) => { (($s) as i32) >= 0 }; }
#[macro_export] macro_rules! CONTAINING_RECORD {
    // An indexed member (`Tail.Overlay.DriverContext[0]`): offsetof the array plus the
    // element stride, as clang folds it -- offset_of! refuses an index.
    ($addr:expr, $ty:ty, $($field:ident).+ [ $idx:expr ] $(,)?) => {{
        const fn __elem<T, const N: usize>(_: *const [T; N]) -> usize {
            core::mem::size_of::<T>()
        }
        let slot = core::mem::MaybeUninit::<$ty>::uninit();
        let stride = __elem(unsafe { core::ptr::addr_of!((*slot.as_ptr()).$($field).+) });
        (($addr) as usize - (core::mem::offset_of!($ty, $($field).+) + ($idx) as usize * stride))
            as *mut $ty
    }};
    // C's field argument is a member PATH (`Tail.Overlay.ListEntry`), and the model
    // often leaves a trailing comma after a multi-line call.
    ($addr:expr, $ty:ty, $($field:ident).+ $(,)?) => {
        (($addr) as usize - core::mem::offset_of!($ty, $($field).+)) as *mut $ty
    };
}
pub use crate::{CONTAINING_RECORD, NT_SUCCESS};
// The Win32 header macros, ported once in win32macros-rust, not per module.
pub use win32macros_rust::{ALIGN_UP, ARRAYSIZE, ARRAY_SIZE, ASSERT, C_ASSERT, CopyMemory, DBG_UNREFERENCED_LOCAL_VARIABLE, DPRINT1, DWLP_DLGPROC, DWLP_MSGRESULT, DWLP_USER, ERR_ON, FAILED_UNEXPECTEDLY, FD_SET, FD_ZERO, FIELD_OFFSET, FIXME_ON, GET_WHEEL_DELTA_WPARAM, GET_X_LPARAM, GET_Y_LPARAM, GetAValue, GetBValue, GetGValue, GetRValue, HIBYTE, HIWORD, HRESULT_FROM_WIN32, HandleToUlong, IDE_SET_ADVANCE_PIO_MODE, IDE_SET_DEFAULT_PIO_MODE, IDE_SET_MWDMA_MODE, IDE_SET_SWDMA_MODE, IDE_SET_UDMA_MODE, INVALID_HANDLE_VALUE, IS_INTRESOURCE, InitializeObjectAttributes, IntToPtr, InterlockedExchange, IsContainerPartition, IsEqualGUID, IsFTPartition, IsRecognizedPartition, LANGIDFROMLCID, LOBYTE, LOWORD, LongToHandle, LongToPtr, MAKEINTATOM, MAKEINTRESOURCEA, MAKEINTRESOURCEW, MAKELANGID, MAKELCID, MAKELONG, MAKEWORD, MoveMemory, PAGE_ROUND_UP, PRIMARYLANGID, PtrToInt, PtrToLong, PtrToUint, PtrToUlong, RGB, RGBA, RTL_CONSTANT_STRING, RTL_CONTAINS_FIELD, RTL_FIELD_SIZE, RTL_NUMBER_OF, RTL_NUMBER_OF_FIELD, RTL_SIZEOF_THROUGH_FIELD, RtlCopyMemory, RtlFillMemory, RtlGetProcessHeap, RtlMoveMemory, RtlZeroMemory, SORTIDFROMLCID, SRB_STATUS, SUBLANGID, TRACE_ON, UIntToPtr, ULongToHandle, ULongToPtr, UNICODE_NULL, UNIMPLEMENTED_DBGBREAK, UNREFERENCED_PARAMETER, UintToPtr, UlongToHandle, UlongToPtr, WARN_ON, WINE_ERR, WINE_FIXME, WINE_TRACE, WINE_WARN, ZeroMemory, __HRESULT_FROM_WIN32, __debugbreak, __p___argc, __p___argv, __p___wargv, __p__environ, __p__pgmptr, __p__wenviron, __p__wpgmptr, __wine_dprintf, __wine_is_debug_on, _countof, debugstr_a, debugstr_an, debugstr_guid, debugstr_w, debugstr_wn, heap_alloc, heap_alloc_zero, heap_calloc, heap_free, heap_realloc, iswalnum, iswalpha, iswcntrl, iswdigit, iswgraph, iswlower, iswprint, iswpunct, iswspace, iswupper, iswxdigit, wide_bytes, wine_dbgstr_a, wine_dbgstr_guid, wine_dbgstr_w};

// A kernel island's `use ntkrnl_sys::ddk::*;` is private: glob it here.
pub use ntkrnl_sys::ddk::*;

// ================= PnP / resource constants the -sys glob does not carry ===================
// The C switches on these. ntkrnl-sys has them only in `driver_ddk` (hand-written, every
// arch) and, on x86, `hal_ddk` -- neither is reached by the `ddk` glob the island
// re-exports, so a bare name in a match arm would bind everything (§9.112). Values: wdm.h.
pub use ntkrnl_sys::driver_ddk::{
    IRP_MN_CANCEL_REMOVE_DEVICE, IRP_MN_CANCEL_STOP_DEVICE, IRP_MN_DEVICE_USAGE_NOTIFICATION,
    IRP_MN_QUERY_CAPABILITIES, IRP_MN_QUERY_DEVICE_RELATIONS, IRP_MN_QUERY_DEVICE_TEXT,
    IRP_MN_QUERY_ID, IRP_MN_QUERY_INTERFACE, IRP_MN_QUERY_PNP_DEVICE_STATE, IRP_MN_QUERY_POWER,
    IRP_MN_QUERY_REMOVE_DEVICE, IRP_MN_QUERY_RESOURCES, IRP_MN_QUERY_RESOURCE_REQUIREMENTS,
    IRP_MN_QUERY_STOP_DEVICE, IRP_MN_REMOVE_DEVICE, IRP_MN_SET_POWER, IRP_MN_START_DEVICE,
    IRP_MN_STOP_DEVICE, IRP_MN_SURPRISE_REMOVAL,
};
// CM_PARTIAL_RESOURCE_DESCRIPTOR.Type is a UCHAR.
pub const CmResourceTypePort: u8 = 1;
pub const CmResourceTypeInterrupt: u8 = 2;
pub const CmResourceTypeMemory: u8 = 3;

// ================= kernel header helpers ======================================================
// wdm.h/ntddk.h MACROS and MSVC intrinsics the C calls as functions. ntkrnl-sys has portable
// wrappers for them outside the `ddk` glob (ke, inline::tier1, inline::ke, interlocked), so
// they are re-exported here BY NAME -- the C's `_`-prefixed intrinsic spellings included.
// ntkrnl_sys::ke's spin-lock helpers take ITS `PKSPIN_LOCK` (`*mut usize`); a driver's lock
// fields are the island's (`*mut c_ulonglong` on 64-bit) -- the same width, a distinct Rust
// type, so a bare re-export made every call site E0308 (16 in atapi). Thin wrappers over the
// island's type instead.
#[inline]
pub unsafe fn KeAcquireSpinLock(SpinLock: PKSPIN_LOCK, OldIrql: *mut KIRQL) {
    ntkrnl_sys::ke::KeAcquireSpinLock(SpinLock.cast(), OldIrql)
}
#[inline]
pub unsafe fn KeReleaseSpinLock(SpinLock: PKSPIN_LOCK, NewIrql: KIRQL) {
    ntkrnl_sys::ke::KeReleaseSpinLock(SpinLock.cast(), NewIrql)
}
#[inline]
pub unsafe fn KeAcquireSpinLockAtDpcLevel(SpinLock: PKSPIN_LOCK) {
    ntkrnl_sys::ke::KeAcquireSpinLockAtDpcLevel(SpinLock.cast())
}
#[inline]
pub unsafe fn KeReleaseSpinLockFromDpcLevel(SpinLock: PKSPIN_LOCK) {
    ntkrnl_sys::ke::KeReleaseSpinLockFromDpcLevel(SpinLock.cast())
}
#[inline]
pub unsafe fn KeInitializeSpinLock(SpinLock: PKSPIN_LOCK) {
    ntkrnl_sys::inline::ke::KeInitializeSpinLock(SpinLock.cast())
}
pub use ntkrnl_sys::inline::tier1::{
    IoCopyCurrentIrpStackLocationToNext, IoGetCurrentIrpStackLocation, IoGetNextIrpStackLocation,
    IoMarkIrpPending, IoSetCancelRoutine, IoSetCompletionRoutine, IoSetNextIrpStackLocation,
    IoSkipCurrentIrpStackLocation,
};
// KeRaiseIrql/KeLowerIrql are wdm.h MACROS over KfRaiseIrql/KfLowerIrql: x86 has wrappers in
// inline::tier1; amd64 writes CR8 inline (inline::mm); arm64 calls the HAL exports.
#[cfg(target_arch = "x86")]
pub use ntkrnl_sys::inline::tier1::{KeLowerIrql, KeRaiseIrql};
#[cfg(target_arch = "x86_64")]
#[inline]
pub unsafe fn KeRaiseIrql(NewIrql: KIRQL, OldIrql: *mut KIRQL) {
    *OldIrql = ntkrnl_sys::inline::mm::KfRaiseIrql(NewIrql);
}
#[cfg(target_arch = "x86_64")]
#[inline]
pub unsafe fn KeLowerIrql(NewIrql: KIRQL) {
    ntkrnl_sys::inline::mm::KfLowerIrql(NewIrql);
}
#[cfg(target_arch = "aarch64")]
extern "C" {
    pub fn KfRaiseIrql(NewIrql: KIRQL) -> KIRQL;
    pub fn KfLowerIrql(NewIrql: KIRQL);
}
#[cfg(target_arch = "aarch64")]
#[inline]
pub unsafe fn KeRaiseIrql(NewIrql: KIRQL, OldIrql: *mut KIRQL) {
    *OldIrql = KfRaiseIrql(NewIrql);
}
#[cfg(target_arch = "aarch64")]
#[inline]
pub unsafe fn KeLowerIrql(NewIrql: KIRQL) {
    KfLowerIrql(NewIrql);
}
// ntbasedef.h: `__assume(e)` is an optimiser promise; `RTL_BITS_OF(t)` is sizeof(t) * 8.
#[inline(always)]
pub unsafe fn __assume(cond: bool) {
    core::hint::assert_unchecked(cond)
}
#[macro_export] macro_rules! RTL_BITS_OF { ($t:ty $(,)?) => { (core::mem::size_of::<$t>() * 8) }; }
pub use crate::RTL_BITS_OF;
pub use ntkrnl_sys::inline::common::InterlockedCompareExchange;
pub use ntkrnl_sys::inline::common::InterlockedCompareExchange as _InterlockedCompareExchange;
pub use ntkrnl_sys::inline::ke::{BitScanForward, BitScanReverse};
pub use ntkrnl_sys::inline::ke::{
    BitScanForward as _BitScanForward, BitScanReverse as _BitScanReverse,
};
pub use ntkrnl_sys::interlocked::{
    InterlockedAnd, InterlockedCompareExchangePointer, InterlockedDecrement, InterlockedExchangeAdd,
    InterlockedExchangePointer, InterlockedIncrement, InterlockedOr, InterlockedXor,
};
pub use ntkrnl_sys::interlocked::{
    InterlockedAnd as _InterlockedAnd, InterlockedDecrement as _InterlockedDecrement,
    InterlockedExchangeAdd as _InterlockedExchangeAdd,
    InterlockedIncrement as _InterlockedIncrement,
    InterlockedOr as _InterlockedOr, InterlockedXor as _InterlockedXor,
};

/// wdm.h: `IoAcquireRemoveLockEx(RemoveLock, Tag, __FILE__, __LINE__, sizeof(IO_REMOVE_LOCK))`.
#[inline]
pub unsafe fn IoAcquireRemoveLock(RemoveLock: PIO_REMOVE_LOCK, Tag: PVOID) -> NTSTATUS {
    IoAcquireRemoveLockEx(
        RemoveLock,
        Tag,
        concat!(file!(), "\0").as_ptr() as PCSTR,
        line!(),
        core::mem::size_of::<IO_REMOVE_LOCK>() as ULONG,
    )
}
/// wdm.h: `IoReleaseRemoveLockEx(RemoveLock, Tag, sizeof(IO_REMOVE_LOCK))`.
#[inline]
pub unsafe fn IoReleaseRemoveLock(RemoveLock: PIO_REMOVE_LOCK, Tag: PVOID) {
    IoReleaseRemoveLockEx(RemoveLock, Tag, core::mem::size_of::<IO_REMOVE_LOCK>() as ULONG)
}
/// wdm.h: `IoReleaseRemoveLockAndWaitEx(RemoveLock, Tag, sizeof(IO_REMOVE_LOCK))`.
#[inline]
pub unsafe fn IoReleaseRemoveLockAndWait(RemoveLock: PIO_REMOVE_LOCK, Tag: PVOID) {
    IoReleaseRemoveLockAndWaitEx(RemoveLock, Tag, core::mem::size_of::<IO_REMOVE_LOCK>() as ULONG)
}

// PAGED_CODE() asserts IRQL < DISPATCH_LEVEL in checked builds and is empty otherwise.
#[macro_export] macro_rules! PAGED_CODE { () => {{}}; }
// NT_VERIFY(exp) is `((exp) ? TRUE : FALSE)` in a free build: the expression is evaluated.
#[macro_export] macro_rules! NT_VERIFY { ($e:expr $(,)?) => { ($e) }; }
// ntbasedef.h: UNREACHABLE is `__builtin_unreachable()`, so reaching it is UB in C too.
#[macro_export] macro_rules! UNREACHABLE {
    () => { core::hint::unreachable_unchecked() }; // nolint: assert-to-ub (__builtin_unreachable)
}
pub use crate::{NT_VERIFY, PAGED_CODE, UNREACHABLE};

// HAL port/register I/O (hal.dll exports on i386/arm64, inline on amd64). By name, not by
// glob: an i386 island declares some of these as externs too, and two globs naming one item
// are ambiguous (E0659, e1000's WRITE_PORT_ULONG); an explicit import beats any glob.
pub use ntkrnl_sys::portio::{
    READ_PORT_UCHAR, READ_PORT_USHORT, READ_PORT_ULONG, READ_PORT_BUFFER_UCHAR,
    READ_PORT_BUFFER_USHORT, READ_PORT_BUFFER_ULONG, WRITE_PORT_UCHAR, WRITE_PORT_USHORT,
    WRITE_PORT_ULONG, WRITE_PORT_BUFFER_UCHAR, WRITE_PORT_BUFFER_USHORT,
    WRITE_PORT_BUFFER_ULONG, READ_REGISTER_UCHAR, READ_REGISTER_USHORT, READ_REGISTER_ULONG,
    READ_REGISTER_BUFFER_UCHAR, READ_REGISTER_BUFFER_USHORT, READ_REGISTER_BUFFER_ULONG,
    WRITE_REGISTER_UCHAR, WRITE_REGISTER_USHORT, WRITE_REGISTER_ULONG,
    WRITE_REGISTER_BUFFER_UCHAR, WRITE_REGISTER_BUFFER_USHORT, WRITE_REGISTER_BUFFER_ULONG,
};
// C's TRUE/FALSE are ints; in kernel code they land in BOOLEAN (UCHAR) far more than BOOL.
pub const TRUE: BOOLEAN = 1;
pub const FALSE: BOOLEAN = 0;

// C's min()/max() macros (`((a) < (b) ? (a) : (b))`): both operands already share a type in
// a translation that compiles, and the C evaluates the winner twice, which no caller here
// relies on.
#[inline]
pub fn min<T: PartialOrd>(a: T, b: T) -> T {
    if a < b { a } else { b }
}
#[inline]
pub fn max<T: PartialOrd>(a: T, b: T) -> T {
    if a > b { a } else { b }
}

// ================= header inlines/macros the substrate does not spell this way ===========
// ntdef.h `#define ANSI_NULL ((CHAR)0)`.
pub const ANSI_NULL: CHAR = 0;
pub use ntkrnl_sys::base::RemoveTailList;
// srb.c calls KfRaiseIrql directly: amd64 writes CR8 inline; i386 is a HAL fastcall export
// (arm64's C export is declared with KeRaiseIrql above).
#[cfg(target_arch = "x86_64")]
pub use ntkrnl_sys::inline::mm::KfRaiseIrql;
#[cfg(target_arch = "x86")]
extern "fastcall" {
    pub fn KfRaiseIrql(NewIrql: KIRQL) -> KIRQL;
}
// wdm.h InterlockedFlushSList: ExInterlockedFlushSList on i386 (a fastcall export, in the
// glob), ExpInterlockedFlushSList elsewhere (inline::ex wraps it).
#[cfg(not(target_arch = "x86"))]
pub use ntkrnl_sys::inline::ex::ExInterlockedFlushSList as InterlockedFlushSList;
#[cfg(target_arch = "x86")]
pub use ntkrnl_sys::ddk::ExInterlockedFlushSList as InterlockedFlushSList;
pub use ntkrnl_sys::inline::common::RtlULongLongMult;

// Hand-transcribed case label (sdk_facts cannot evaluate it): cmtypes.h
// `#define CmResourceTypeDma 4`. i386/amd64 get it from the HAL island; arm64 has none,
// and an unresolved name in a match arm is a catch-all binding.
pub const CmResourceTypeDma: u32 = 4;

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
pub const IOCTL_SCSI_GET_ADDRESS: u32 = 0x41018; // clang, ioctl.c
pub const IOCTL_STORAGE_QUERY_PROPERTY: u32 = 0x2D1400; // clang, ioctl.c
pub const TAG_ACCRESS_RANGE: u32 = 0x52417453; // clang, miniport.c
pub const TAG_ADDRESS_MAPPING: u32 = 0x4D417453; // clang, misc.c
pub const TAG_DEVICE_ID: u32 = 0x44497453; // clang, pdo.c
pub const TAG_DEVICE_RELATION: u32 = 0x52447453; // clang, fdo.c
pub const TAG_DEVICE_TEXT: u32 = 0x58547453; // clang, pdo.c
pub const TAG_INIT_DATA: u32 = 0x44497453; // clang, storport.c
pub const TAG_INQUIRY_DATA: u32 = 0x51497453; // clang, fdo.c
pub const TAG_MINIPORT_DATA: u32 = 0x444D7453; // clang, miniport.c
pub const TAG_QUEUED_REQUEST: u32 = 0x52517453; // clang, fdo.c
pub const TAG_REPORT_LUN_DATA: u32 = 0x4C527453; // clang, fdo.c
pub const TAG_RESOURCE_LIST: u32 = 0x4C527453; // clang, misc.c
pub const TAG_SENSE_DATA: u32 = 0x4E537453; // clang, fdo.c
pub const TAG_SRB_EXTENSION: u32 = 0x58537453; // clang, fdo.c
// ==== end port facts ====

// ==== port variadic: C-variadic entry points, trampolines in the module's rust_varargs_shim.c (generated -- `rustconv port integrate`) ====
extern "C" {
    pub fn StorPortDebugPrint(DebugPrintLevel: ULONG, DebugMessage: PCHAR, ...);
    pub fn StorPortExtendedFunction(FunctionCode: STORPORT_FUNCTION_CODE, HwDeviceExtension: PVOID, ...) -> ULONG;
    pub fn StorPortNotification(NotificationType: SCSI_NOTIFICATION_TYPE, HwDeviceExtension: PVOID, ...);
}
// ==== end port variadic ====

// ==== port resolve: tree consensus (generated -- re-run `rustconv port resolve`) ====
pub const AcquireSpinLock: _SCSI_NOTIFICATION_TYPE = _SCSI_NOTIFICATION_TYPE_AcquireSpinLock; // island enum
pub const BusQueryCompatibleIDs: _BUS_QUERY_ID_TYPE = _BUS_QUERY_ID_TYPE_BusQueryCompatibleIDs; // island enum
pub const BusQueryDeviceID: _BUS_QUERY_ID_TYPE = _BUS_QUERY_ID_TYPE_BusQueryDeviceID; // island enum
pub const BusQueryHardwareIDs: _BUS_QUERY_ID_TYPE = _BUS_QUERY_ID_TYPE_BusQueryHardwareIDs; // island enum
pub const BusQueryInstanceID: _BUS_QUERY_ID_TYPE = _BUS_QUERY_ID_TYPE_BusQueryInstanceID; // island enum
pub const BusRelations: _DEVICE_RELATION_TYPE = _DEVICE_RELATION_TYPE_BusRelations; // island enum
pub const BusTypeSata: _STORAGE_BUS_TYPE = _STORAGE_BUS_TYPE_BusTypeSata; // island enum
pub const CmResourceShareShared: _CM_SHARE_DISPOSITION = _CM_SHARE_DISPOSITION_CmResourceShareShared; // island enum
pub const DPFLTR_ERROR_LEVEL: u32 = 0; // ntkrnl-sys/src/hal_bindgen_amd64.rs
pub const DevicePropertyBusTypeGuid: DEVICE_REGISTRY_PROPERTY = DEVICE_REGISTRY_PROPERTY_DevicePropertyBusTypeGuid; // island enum
pub const DeviceTextDescription: _DEVICE_TEXT_TYPE = _DEVICE_TEXT_TYPE_DeviceTextDescription; // island enum
pub const DeviceTextLocationInformation: _DEVICE_TEXT_TYPE = _DEVICE_TEXT_TYPE_DeviceTextLocationInformation; // island enum
pub const DpcLock: _STOR_SPINLOCK = _STOR_SPINLOCK_DpcLock; // island enum
pub const EnablePassiveInitialization: _SCSI_NOTIFICATION_TYPE = _SCSI_NOTIFICATION_TYPE_EnablePassiveInitialization; // island enum
pub const ExtFunctionAllocatePool: _STORPORT_FUNCTION_CODE = _STORPORT_FUNCTION_CODE_ExtFunctionAllocatePool; // island enum
pub const ExtFunctionFreePool: _STORPORT_FUNCTION_CODE = _STORPORT_FUNCTION_CODE_ExtFunctionFreePool; // island enum
pub const ExtFunctionInitializeTimer: _STORPORT_FUNCTION_CODE = _STORPORT_FUNCTION_CODE_ExtFunctionInitializeTimer; // island enum
pub const ExtFunctionRequestTimer: _STORPORT_FUNCTION_CODE = _STORPORT_FUNCTION_CODE_ExtFunctionRequestTimer; // island enum
pub const GetExtendedFunctionTable: _SCSI_NOTIFICATION_TYPE = _SCSI_NOTIFICATION_TYPE_GetExtendedFunctionTable; // island enum
pub const InitializeDpc: _SCSI_NOTIFICATION_TYPE = _SCSI_NOTIFICATION_TYPE_InitializeDpc; // island enum
pub const InterfaceTypeUndefined: _INTERFACE_TYPE = _INTERFACE_TYPE_InterfaceTypeUndefined; // island enum
pub const InterruptLock: _STOR_SPINLOCK = _STOR_SPINLOCK_InterruptLock; // island enum
pub const IoModifyAccess: _LOCK_OPERATION = _LOCK_OPERATION_IoModifyAccess; // island enum
pub const IssueDpc: _SCSI_NOTIFICATION_TYPE = _SCSI_NOTIFICATION_TYPE_IssueDpc; // island enum
pub const Latched: _KINTERRUPT_MODE = _KINTERRUPT_MODE_Latched; // island enum
pub const LevelSensitive: _KINTERRUPT_MODE = _KINTERRUPT_MODE_LevelSensitive; // island enum
pub const MmCached: _MEMORY_CACHING_TYPE = _MEMORY_CACHING_TYPE_MmCached; // island enum
pub const PCIBus: _INTERFACE_TYPE = _INTERFACE_TYPE_PCIBus; // island enum
pub const PCMCIABus: _INTERFACE_TYPE = _INTERFACE_TYPE_PCMCIABus; // island enum
pub const PNPISABus: _INTERFACE_TYPE = _INTERFACE_TYPE_PNPISABus; // island enum
pub const PropertyExistsQuery: _STORAGE_QUERY_TYPE = _STORAGE_QUERY_TYPE_PropertyExistsQuery; // island enum
pub const PropertyStandardQuery: _STORAGE_QUERY_TYPE = _STORAGE_QUERY_TYPE_PropertyStandardQuery; // island enum
pub const ReleaseSpinLock: _SCSI_NOTIFICATION_TYPE = _SCSI_NOTIFICATION_TYPE_ReleaseSpinLock; // island enum
pub const RemovalRelations: _DEVICE_RELATION_TYPE = _DEVICE_RELATION_TYPE_RemovalRelations; // island enum
pub const RequestComplete: _SCSI_NOTIFICATION_TYPE = _SCSI_NOTIFICATION_TYPE_RequestComplete; // island enum
pub const RequestTimerCall: _SCSI_NOTIFICATION_TYPE = _SCSI_NOTIFICATION_TYPE_RequestTimerCall; // island enum
pub const StartIoLock: _STOR_SPINLOCK = _STOR_SPINLOCK_StartIoLock; // island enum
pub const StorageAdapterProperty: _STORAGE_PROPERTY_ID = _STORAGE_PROPERTY_ID_StorageAdapterProperty; // island enum
pub const StorageDeviceIdProperty: _STORAGE_PROPERTY_ID = _STORAGE_PROPERTY_ID_StorageDeviceIdProperty; // island enum
pub const StorageDevicePowerProperty: _STORAGE_PROPERTY_ID = _STORAGE_PROPERTY_ID_StorageDevicePowerProperty; // island enum
pub const StorageDeviceProperty: _STORAGE_PROPERTY_ID = _STORAGE_PROPERTY_ID_StorageDeviceProperty; // island enum
pub const TargetDeviceRelation: _DEVICE_RELATION_TYPE = _DEVICE_RELATION_TYPE_TargetDeviceRelation; // island enum
pub const Width16Bits: _DMA_WIDTH = _DMA_WIDTH_Width16Bits; // island enum
pub const Width32Bits: _DMA_WIDTH = _DMA_WIDTH_Width32Bits; // island enum
pub const Width8Bits: _DMA_WIDTH = _DMA_WIDTH_Width8Bits; // island enum
pub const dsRemoved: DEVICE_STATE = DEVICE_STATE_dsRemoved; // island enum
pub const dsStarted: DEVICE_STATE = DEVICE_STATE_dsStarted; // island enum
pub const dsStopped: DEVICE_STATE = DEVICE_STATE_dsStopped; // island enum
pub const dsSurpriseRemoved: DEVICE_STATE = DEVICE_STATE_dsSurpriseRemoved; // island enum
// ==== end port resolve ====

// ==== port resolve: cross-TU definitions (generated) ====
pub use crate::storport::misc::AllocateAddressMapping;
pub use crate::storport::misc::CopyResourceList;
pub use crate::storport::fdo::FdoFindLun;
pub use crate::storport::misc::ForwardIrpAndForget;
pub use crate::storport::misc::GetBusInterface;
pub use crate::storport::misc::GetDeviceType;
pub use crate::storport::misc::GetGenericType;
pub use crate::storport::misc::GetResourceListInterrupt;
pub use crate::storport::miniport::MiniportFindAdapter;
pub use crate::storport::miniport::MiniportHwInitialize;
pub use crate::storport::miniport::MiniportHwInterrupt;
pub use crate::storport::miniport::MiniportInitialize;
pub use crate::storport::srb::PortCompletionDpc;
pub use crate::storport::pdo::PortCreatePdo;
pub use crate::storport::pdo::PortDeletePdo;
pub use crate::storport::ioctl::PortFdoDeviceControl;
pub use crate::storport::fdo::PortFdoPnp;
pub use crate::storport::fdo::PortFdoScsi;
pub use crate::storport::storport::PortGetDriverInitData;
pub use crate::storport::fdo::PortInitializeStarvedRequestQueue;
pub use crate::storport::ioctl::PortPdoDeviceControl;
pub use crate::storport::pdo::PortPdoPnp;
pub use crate::storport::srb::PortPdoScsi;
pub use crate::storport::fdo::PortQueueStarvedRequest;
pub use crate::storport::fdo::PortRetryStarvedRequest;
pub use crate::storport::misc::QueryBusInterface;
pub use crate::storport::misc::SimpleTimerCallbackDpcRoutine;
pub use crate::storport::inline_h::StorpCheckFlowControl;
pub use crate::storport::inline_h::StorpCompleteRequest;
pub use crate::storport::inline_h::StorpGetMiniportFdo;
pub use crate::storport::inline_h::StorpInterlockedInsertHeadListCounted;
pub use crate::storport::inline_h::StorpInterlockedRemoveEntryListCounted;
pub use crate::storport::misc::TimerCallbackDpcRoutine;
pub use crate::storport::misc::TranslateResourceListAddress;
// ==== end cross-TU definitions ====

// ==== port resolve: substrate helpers (generated) ====
pub use ntkrnl_sys::ke::DbgBreakPoint;
pub use ntkrnl_sys::inline::ex::ExAllocateFromNPagedLookasideList;
pub use ntkrnl_sys::inline::mm::ExAllocatePoolUninitialized;
pub use ntkrnl_sys::ddk::ExAllocatePoolWithTag;
pub use ntkrnl_sys::ddk::ExFreePoolWithTag;
pub use ntkrnl_sys::inline::ex::ExFreeToNPagedLookasideList;
pub use ntkrnl_sys::inline::common::InitializeListHead;
pub use ntkrnl_sys::inline::common::InitializeSListHead;
pub use ntkrnl_sys::inline::common::InsertHeadList;
pub use ntkrnl_sys::inline::common::InsertTailList;
pub use ntkrnl_sys::inline::ex::InterlockedPushEntrySList;
pub use ntkrnl_sys::ddk::IoAcquireRemoveLockEx;
pub use ntkrnl_sys::driver_ddk::IoCallDriver;
pub use ntkrnl_sys::driver_ddk::IoCompleteRequest;
pub use ntkrnl_sys::ddk::IoReleaseRemoveLockAndWaitEx;
pub use ntkrnl_sys::ddk::IoReleaseRemoveLockEx;
pub use ntkrnl_sys::inline::common::IsListEmpty;
#[cfg(target_arch = "x86")]
pub use ntkrnl_sys::ddk::KeQuerySystemTime;
#[cfg(not(target_arch = "x86"))]
pub use ntkrnl_sys::inline::ke::KeQuerySystemTime;
pub use ntkrnl_sys::NT_ASSERT;
pub use ntkrnl_sys::mm::NonPagedPool;
pub use ntkrnl_sys::inline::tier1::ObReferenceObject;
pub use ntkrnl_sys::mm::PagedPool;
pub use ntkrnl_sys::inline::common::RemoveEntryList;
pub use ntkrnl_sys::inline::common::RemoveHeadList;
pub use ntkrnl_sys::inline::common::RtlULongAdd;
pub use ntkrnl_sys::constants::STATUS_BUFFER_TOO_SMALL;
pub use ntkrnl_sys::constants::STATUS_CANCELLED;
pub use ntkrnl_sys::base::STATUS_INSUFFICIENT_RESOURCES;
pub use ntkrnl_sys::constants::STATUS_INVALID_DEVICE_REQUEST;
pub use ntkrnl_sys::constants::STATUS_INVALID_PARAMETER;
pub use ntkrnl_sys::constants::STATUS_INVALID_PARAMETER_1;
pub use ntkrnl_sys::constants::STATUS_INVALID_PARAMETER_2;
pub use ntkrnl_sys::constants::STATUS_NOT_FOUND;
pub use ntkrnl_sys::base::STATUS_NOT_IMPLEMENTED;
pub use ntkrnl_sys::constants::STATUS_NOT_SUPPORTED;
pub use ntkrnl_sys::base::STATUS_NO_MEMORY;
pub use ntkrnl_sys::constants::STATUS_PENDING;
pub use ntkrnl_sys::constants::STATUS_REVISION_MISMATCH;
pub use ntkrnl_sys::base::STATUS_SUCCESS;
pub use ntkrnl_sys::constants::STATUS_UNSUCCESSFUL;
// ==== end substrate helpers ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, RtlUnicodeStringPrintf, StringCbCatNW, StringCbCatW, StringCbCopyExW, StringCbCopyNExW, StringCbCopyNW, StringCbCopyW, StringCbPrintfExW, StringCbPrintfW, StringCbVPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyA, StringCchCopyExW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthA, StringCchLengthW, StringCchPrintfA, StringCchPrintfExA, StringCchPrintfExW, StringCchPrintfW, StringCchVPrintfA, StringCchVPrintfExA, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// INTERFACE_TYPE: c2rust, fdo.c
pub type INTERFACE_TYPE = _INTERFACE_TYPE;
// KINTERRUPT_MODE: c2rust, fdo.c
pub type KINTERRUPT_MODE = _KINTERRUPT_MODE;
// SCSI_NOTIFICATION_TYPE: c2rust, storport.c
pub type SCSI_NOTIFICATION_TYPE = _SCSI_NOTIFICATION_TYPE;
// STORPORT_FUNCTION_CODE: c2rust, storport.c
pub type STORPORT_FUNCTION_CODE = _STORPORT_FUNCTION_CODE;
// STOR_SPINLOCK: c2rust, storport.c
pub type STOR_SPINLOCK = _STOR_SPINLOCK;
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
extern "C" {
    pub fn strlen(_Str: *const ::core::ffi::c_char) -> size_t; // pdo.c
}
// ==== end port externs ====

// ==== port resolve: INITGUID storage (generated) ====
#[no_mangle]
pub static GUID_BUS_INTERFACE_STANDARD: GUID = GUID { Data1: 0x496b8280, Data2: 0x6f25, Data3: 0x11d0, Data4: [0xbe, 0xaf, 0x08, 0x00, 0x2b, 0xe2, 0x09, 0x2f] };
#[no_mangle]
pub static GUID_BUS_TYPE_ISAPNP: GUID = GUID { Data1: 0xe676f854, Data2: 0xd87d, Data3: 0x11d0, Data4: [0x92, 0xb2, 0x00, 0xa0, 0xc9, 0x05, 0x5f, 0xc5] };
#[no_mangle]
pub static GUID_BUS_TYPE_PCI: GUID = GUID { Data1: 0xc8ebdfb0, Data2: 0xb510, Data3: 0x11d0, Data4: [0x80, 0xe5, 0x00, 0xa0, 0xc9, 0x25, 0x42, 0xe3] };
#[no_mangle]
pub static GUID_BUS_TYPE_PCMCIA: GUID = GUID { Data1: 0x09343630, Data2: 0xaf9f, Data3: 0x11d0, Data4: [0x92, 0xE9, 0x00, 0x00, 0xf8, 0x1e, 0x1b, 0x30] };
// ==== end INITGUID storage ====

// ==== port macros: SDK header macros, lowered from the preprocessor's own expansion (generated -- re-run `rustconv port macros`) ====
#[macro_export] macro_rules! MmGetMdlVirtualAddress { ($p0:expr $(,)?) => { ((((((*$p0).StartVa) as PCHAR)).wrapping_add(((*$p0).ByteOffset) as usize)) as PVOID) }; }
#[macro_export] macro_rules! va_end { ($p0:expr $(,)?) => { __builtin_va_end((($p0) as _)) }; }
#[macro_export] macro_rules! va_start { ($p0:expr, $p1:expr $(,)?) => { __builtin_va_start((($p0) as _), (($p1) as _)) }; }
pub const SP_UNINITIALIZED_VALUE: ULONG = (((!0)) as ULONG);
pub const STATUS_ADAPTER_HARDWARE_ERROR: NTSTATUS = 0xc00000c2usize as NTSTATUS;
pub const STATUS_DEVICE_BUSY: NTSTATUS = 0x80000011usize as NTSTATUS;
pub const STATUS_DEVICE_CONFIGURATION_ERROR: NTSTATUS = 0xc0000182usize as NTSTATUS;
pub const STATUS_INTERNAL_ERROR: NTSTATUS = 0xc00000e5usize as NTSTATUS;
pub const STATUS_IO_DEVICE_ERROR: NTSTATUS = 0xc0000185usize as NTSTATUS;
pub const STATUS_NO_SUCH_DEVICE: NTSTATUS = 0xc000000eusize as NTSTATUS;
pub use crate::{MmGetMdlVirtualAddress, va_end, va_start};
// ==== end SDK header macros ====

// ==== port macros: the module's own C macros (generated -- `rustconv port macros`) ====
// drivers/storage/port/storport/precomp.h:45: #define TEST_FLAG(Flags, Bit) (((Flags) & (Bit)) != 0)
#[allow(unused_macros)]
macro_rules! TEST_FLAG {
    ($flags:expr, $bit:expr $(,)?) => {
        ((($flags) & ($bit)) != 0)
    };
}
#[allow(unused_imports)]
pub(crate) use TEST_FLAG;
// ==== end port macros ====
