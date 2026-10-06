//! atapi-rust prelude — the single import surface the translated modules use.
//! Bridges the bindgen `types` island and the translated C. Anything bindgen leaves out
//! (CRT, inlines, sibling-C externs, hand-transcribed consts) is declared here ONCE.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

pub use crate::types::*;
pub use core::ffi::{c_char, c_int, c_long, c_short, c_uchar, c_uint, c_ushort, c_void};
// C's `size_t`: bindgen spells it `usize` inline, so no island carries the name the C uses.
pub type size_t = usize;
pub use core::ptr::{addr_of, addr_of_mut};

// ================= debug ======================================================================
// reactos/debug.h DPRINT/DPRINT1/UNIMPLEMENTED — no-op MACROS, never out-of-line
// fns (field manual §9.65: an out-of-line no-op is an optimization barrier). debug_macros.py
// rewrites C's bare `UNIMPLEMENTED;` to `UNIMPLEMENTED!()`, so the prelude must have it.
#[macro_export] macro_rules! DPRINT  { ($($arg:tt)*) => {{}}; }
#[macro_export] macro_rules! DPRINT1 { ($($arg:tt)*) => {{}}; }
#[macro_export] macro_rules! UNIMPLEMENTED { ($($arg:tt)*) => {{}}; }
#[macro_export] macro_rules! UNIMPLEMENTED_ONCE { ($($arg:tt)*) => {{}}; }
pub use crate::{DPRINT, DPRINT1, UNIMPLEMENTED, UNIMPLEMENTED_ONCE};
// The module's own debug.h channels (ERR/WARN/INFO/TRACE, printf-shaped). Macros, never
// out-of-line fns, for the same reason as DPRINT above.
#[macro_export] macro_rules! ERR { ($($arg:tt)*) => {{}}; }
#[macro_export] macro_rules! WARN { ($($arg:tt)*) => {{}}; }
#[macro_export] macro_rules! INFO { ($($arg:tt)*) => {{}}; }
#[macro_export] macro_rules! TRACE { ($($arg:tt)*) => {{}}; }
pub use crate::{ERR, INFO, TRACE, WARN};

// ================= kernel header helpers ======================================================
// wdm.h/ntddk.h MACROS and MSVC intrinsics the C calls as functions. ntkrnl-sys has portable
// wrappers for them outside the `ddk` glob (ke, inline::tier1, inline::ke, interlocked), so
// they are re-exported here BY NAME -- the C's `_`-prefixed intrinsic spellings included.
pub use ntkrnl_sys::ke::{
    KeAcquireSpinLock, KeAcquireSpinLockAtDpcLevel, KeReleaseSpinLock, KeReleaseSpinLockFromDpcLevel,
};
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
    fn KfRaiseIrql(NewIrql: KIRQL) -> KIRQL;
    fn KfLowerIrql(NewIrql: KIRQL);
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
pub use ntkrnl_sys::inline::ke::{BitScanForward as _BitScanForward, BitScanReverse as _BitScanReverse};
pub use ntkrnl_sys::interlocked::{
    InterlockedAnd, InterlockedCompareExchangePointer, InterlockedDecrement, InterlockedExchangeAdd,
    InterlockedExchangePointer, InterlockedIncrement, InterlockedOr, InterlockedXor,
};
pub use ntkrnl_sys::interlocked::{
    InterlockedAnd as _InterlockedAnd, InterlockedDecrement as _InterlockedDecrement,
    InterlockedExchangeAdd as _InterlockedExchangeAdd, InterlockedIncrement as _InterlockedIncrement,
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
#[macro_export] macro_rules! UNREACHABLE { () => { core::hint::unreachable_unchecked() }; }
pub use crate::{NT_VERIFY, PAGED_CODE, UNREACHABLE};

// HAL port/register I/O (hal.dll exports on i386/arm64, inline on amd64).
pub use ntkrnl_sys::portio::*;
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

// ================= ata_user.h transfer-mode index macros (hand-ported) =====================
// Function-like SDK macros, so `port macros` (the module's own) does not reach them.
#[macro_export] macro_rules! PIO_MODE { ($n:expr $(,)?) => { ($n) }; }
#[macro_export] macro_rules! SWDMA_MODE { ($n:expr $(,)?) => { (5 + ($n)) }; }
#[macro_export] macro_rules! MWDMA_MODE { ($n:expr $(,)?) => { (8 + ($n)) }; }
#[macro_export] macro_rules! UDMA_MODE { ($n:expr $(,)?) => { (11 + ($n)) }; }
pub use crate::{MWDMA_MODE, PIO_MODE, SWDMA_MODE, UDMA_MODE};


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
        const fn __elem<T, const N: usize>(_: *const [T; N]) -> usize { core::mem::size_of::<T>() }
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

// A kernel island's `use ntkrnl_sys::ddk::*;` is private: glob it here.
pub use ntkrnl_sys::ddk::*;
// ntkrnl-sys has _SCSI_REQUEST_BLOCK opaque; the island (UNBLOCK) has the real layout,
// and an explicit re-export beats the glob above.
pub use crate::types::{PSCSI_REQUEST_BLOCK, SCSI_REQUEST_BLOCK, _SCSI_REQUEST_BLOCK};

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

// ==== port resolve: tree consensus (generated -- re-run `rustconv port resolve`) ====
pub const ACTION_DEVICE_CONFIG: _ATA_PORT_ACTION = _ATA_PORT_ACTION_ACTION_DEVICE_CONFIG; // island enum
pub const ACTION_DEVICE_ERROR: _ATA_PORT_ACTION = _ATA_PORT_ACTION_ACTION_DEVICE_ERROR; // island enum
pub const ACTION_DEVICE_POWER: _ATA_PORT_ACTION = _ATA_PORT_ACTION_ACTION_DEVICE_POWER; // island enum
pub const ACTION_ENUM_DEVICE: _ATA_PORT_ACTION = _ATA_PORT_ACTION_ACTION_ENUM_DEVICE; // island enum
pub const ACTION_ENUM_DEVICE_NEW: _ATA_PORT_ACTION = _ATA_PORT_ACTION_ACTION_ENUM_DEVICE_NEW; // island enum
pub const ACTION_ENUM_PORT: _ATA_PORT_ACTION = _ATA_PORT_ACTION_ACTION_ENUM_PORT; // island enum
pub const ACTION_PORT_RESET: _ATA_PORT_ACTION = _ATA_PORT_ACTION_ACTION_PORT_RESET; // island enum
pub const ACTION_PORT_TIMING: _ATA_PORT_ACTION = _ATA_PORT_ACTION_ACTION_PORT_TIMING; // island enum
pub const ANSI_NULL: CHAR = 0; // freeldr-rust/src/ui_keystone.rs
pub const APC_LEVEL: u32 = 1; // ntkrnl-sys/src/hal_bindgen_amd64.rs
pub const ATA_MAX_LBA_28: ULONG64 = 0x0FFFFFFF; // freeldr-rust/src/hwide.rs
pub const CAP_ATAPI_ID_CMD: u32 = 2; // cdrom-rust/src/types_amd64.rs
pub const CAP_ATA_ID_CMD: u32 = 1; // cdrom-rust/src/types_amd64.rs
pub const CAP_SMART_CMD: u32 = 4; // cdrom-rust/src/types_amd64.rs
pub const COMPLETE_IRP: ATA_COMPLETION_ACTION = ATA_COMPLETION_ACTION_COMPLETE_IRP; // island enum
pub const COMPLETE_NO_IRP: ATA_COMPLETION_ACTION = ATA_COMPLETION_ACTION_COMPLETE_NO_IRP; // island enum
pub const CONN_STATUS_DEV_ATAPI: _ATA_CONNECTION_STATUS = _ATA_CONNECTION_STATUS_CONN_STATUS_DEV_ATAPI; // island enum
pub const CONN_STATUS_FAILURE: _ATA_CONNECTION_STATUS = _ATA_CONNECTION_STATUS_CONN_STATUS_FAILURE; // island enum
pub const CONN_STATUS_NO_DEVICE: _ATA_CONNECTION_STATUS = _ATA_CONNECTION_STATUS_CONN_STATUS_NO_DEVICE; // island enum
pub const DEV_ATA: _ATA_DEVICE_TYPE = _ATA_DEVICE_TYPE_DEV_ATA; // island enum
pub const DEV_ATAPI: _ATA_DEVICE_TYPE = _ATA_DEVICE_TYPE_DEV_ATAPI; // island enum
pub const DEV_NONE: _ATA_DEVICE_TYPE = _ATA_DEVICE_TYPE_DEV_NONE; // island enum
pub const DEV_STATUS_FAILED: _ATA_DEVICE_STATUS = _ATA_DEVICE_STATUS_DEV_STATUS_FAILED; // island enum
pub const DEV_STATUS_NEW_DEVICE: _ATA_DEVICE_STATUS = _ATA_DEVICE_STATUS_DEV_STATUS_NEW_DEVICE; // island enum
pub const DEV_STATUS_NO_DEVICE: _ATA_DEVICE_STATUS = _ATA_DEVICE_STATUS_DEV_STATUS_NO_DEVICE; // island enum
pub const DEV_STATUS_SAME_DEVICE: _ATA_DEVICE_STATUS = _ATA_DEVICE_STATUS_DEV_STATUS_SAME_DEVICE; // island enum
pub const DIRECTORY_ALL_ACCESS: u32 = 983055; // ntdll-sys/src/bindgen_amd64.rs
pub const DIRECT_ACCESS_DEVICE: u32 = 0; // atapi-legacy-rust/src/types_amd64.rs
pub const DISPATCH_LEVEL: u32 = 2; // ntkrnl-sys/src/hal_bindgen_amd64.rs
pub const DO_DEVICE_INITIALIZING: u32 = 128; // ntkrnl-sys/src/hal_bindgen_amd64.rs
pub const DO_DIRECT_IO: u32 = 16; // ntkrnl-sys/src/hal_bindgen_amd64.rs
pub const DO_POWER_INRUSH: u32 = 16384; // ntkrnl-sys/src/hal_bindgen_amd64.rs
pub const DevicePowerState: u32 = 1; // kbdhid-rust/src/glue.rs
pub const EVENT_CODE_BAD_STATE: _ATA_ERROR_LOG_VALUE = _ATA_ERROR_LOG_VALUE_EVENT_CODE_BAD_STATE; // island enum
pub const EVENT_CODE_CRC_ERROR: _ATA_ERROR_LOG_VALUE = _ATA_ERROR_LOG_VALUE_EVENT_CODE_CRC_ERROR; // island enum
pub const EVENT_CODE_DMA_DISABLE: _ATA_ERROR_LOG_VALUE = _ATA_ERROR_LOG_VALUE_EVENT_CODE_DMA_DISABLE; // island enum
pub const EVENT_CODE_DOWNSHIFT: _ATA_ERROR_LOG_VALUE = _ATA_ERROR_LOG_VALUE_EVENT_CODE_DOWNSHIFT; // island enum
pub const EVENT_CODE_NCQ_DISABLE: _ATA_ERROR_LOG_VALUE = _ATA_ERROR_LOG_VALUE_EVENT_CODE_NCQ_DISABLE; // island enum
pub const EVENT_CODE_TIMEOUT: _ATA_ERROR_LOG_VALUE = _ATA_ERROR_LOG_VALUE_EVENT_CODE_TIMEOUT; // island enum
pub const FILE_DEVICE_CONTROLLER: u32 = 4; // ntdll-sys/src/bindgen_amd64.rs
pub const FILE_DEVICE_MASS_STORAGE: u32 = 45; // ntdll-sys/src/bindgen_amd64.rs
pub const FILE_DEVICE_SECURE_OPEN: u32 = 256; // ntdll-sys/src/bindgen_amd64.rs
pub const FILE_REMOVABLE_MEDIA: u32 = 1; // ntdll-sys/src/bindgen_amd64.rs
pub const GetDeviceType: DEVICE_TYPE_NAME = DEVICE_TYPE_NAME_GetDeviceType; // island enum
pub const GetGenericType: DEVICE_TYPE_NAME = DEVICE_TYPE_NAME_GetGenericType; // island enum
pub const HighPagePriority: ULONG = 2; // ntoskrnl-rust/src/ARM3/helpers.rs
pub const IDENTIFY_BUFFER_SIZE: u32 = 512; // cdrom-rust/src/types_amd64.rs
pub const IDE_COMMAND_ATAPI_IDENTIFY: u32 = 161; // atapi-legacy-rust/src/types_amd64.rs
pub const IDE_COMMAND_DOOR_LOCK: u32 = 222; // storahci-rust/src/types_amd64.rs
pub const IDE_COMMAND_DOOR_UNLOCK: u32 = 223; // storahci-rust/src/types_amd64.rs
pub const IDE_COMMAND_FLUSH_CACHE: u32 = 231; // storahci-rust/src/types_amd64.rs
pub const IDE_COMMAND_FLUSH_CACHE_EXT: u32 = 234; // storahci-rust/src/types_amd64.rs
pub const IDE_COMMAND_IDENTIFY: u32 = 236; // atapi-legacy-rust/src/types_amd64.rs
pub const IDE_COMMAND_IDLE_IMMEDIATE: u32 = 225; // storahci-rust/src/types_amd64.rs
pub const IDE_COMMAND_MEDIA_EJECT: u32 = 237; // atapi-legacy-rust/src/types_amd64.rs
pub const IDE_COMMAND_READ: u32 = 32; // atapi-legacy-rust/src/types_amd64.rs
pub const IDE_COMMAND_READ_DMA: u32 = 200; // atapi-legacy-rust/src/types_amd64.rs
pub const IDE_COMMAND_READ_DMA_EXT: u32 = 37; // storahci-rust/src/types_amd64.rs
pub const IDE_COMMAND_READ_EXT: u32 = 36; // storahci-rust/src/types_amd64.rs
pub const IDE_COMMAND_READ_FPDMA_QUEUED: u32 = 96; // storahci-rust/src/types_amd64.rs
pub const IDE_COMMAND_READ_LOG_EXT: u32 = 47; // storahci-rust/src/types_amd64.rs
pub const IDE_COMMAND_READ_MULTIPLE: u32 = 196; // atapi-legacy-rust/src/types_amd64.rs
pub const IDE_COMMAND_READ_MULTIPLE_EXT: u32 = 41; // storahci-rust/src/types_amd64.rs
pub const IDE_COMMAND_SECURITY_FREEZE_LOCK: u32 = 245; // storahci-rust/src/types_amd64.rs
pub const IDE_COMMAND_SET_DRIVE_PARAMETERS: u32 = 145; // atapi-legacy-rust/src/types_amd64.rs
pub const IDE_COMMAND_SET_FEATURE: u32 = 239; // storahci-rust/src/types_amd64.rs
pub const IDE_COMMAND_SET_MULTIPLE: u32 = 198; // atapi-legacy-rust/src/types_amd64.rs
pub const IDE_COMMAND_SMART: u32 = 176; // storahci-rust/src/types_amd64.rs
pub const IDE_COMMAND_STANDBY_IMMEDIATE: u32 = 224; // storahci-rust/src/types_amd64.rs
pub const IDE_COMMAND_VERIFY: u32 = 64; // atapi-legacy-rust/src/types_amd64.rs
pub const IDE_COMMAND_VERIFY_EXT: u32 = 66; // storahci-rust/src/types_amd64.rs
pub const IDE_COMMAND_WRITE: u32 = 48; // atapi-legacy-rust/src/types_amd64.rs
pub const IDE_COMMAND_WRITE_DMA: u32 = 202; // atapi-legacy-rust/src/types_amd64.rs
pub const IDE_COMMAND_WRITE_DMA_EXT: u32 = 53; // storahci-rust/src/types_amd64.rs
pub const IDE_COMMAND_WRITE_DMA_FUA_EXT: u32 = 61; // storahci-rust/src/types_amd64.rs
pub const IDE_COMMAND_WRITE_EXT: u32 = 52; // storahci-rust/src/types_amd64.rs
pub const IDE_COMMAND_WRITE_FPDMA_QUEUED: u32 = 97; // storahci-rust/src/types_amd64.rs
pub const IDE_COMMAND_WRITE_MULTIPLE: u32 = 197; // atapi-legacy-rust/src/types_amd64.rs
pub const IDE_COMMAND_WRITE_MULTIPLE_EXT: u32 = 57; // storahci-rust/src/types_amd64.rs
pub const IDE_COMMAND_WRITE_MULTIPLE_FUA_EXT: u32 = 206; // storahci-rust/src/types_amd64.rs
pub const IDE_DRIVE_SELECT: u32 = 160; // uniata-rust/src/types_amd64.rs
pub const IDE_ERROR_ADDRESS_NOT_FOUND: u32 = 1; // storahci-rust/src/types_amd64.rs
pub const IDE_ERROR_COMMAND_ABORTED: u32 = 4; // atapi-legacy-rust/src/types_amd64.rs
pub const IDE_ERROR_CRC_ERROR: u32 = 128; // storahci-rust/src/types_amd64.rs
pub const IDE_ERROR_DATA_ERROR: u32 = 64; // atapi-legacy-rust/src/types_amd64.rs
pub const IDE_ERROR_END_OF_MEDIA: u32 = 2; // atapi-legacy-rust/src/types_amd64.rs
pub const IDE_ERROR_ID_NOT_FOUND: u32 = 16; // atapi-legacy-rust/src/types_amd64.rs
pub const IDE_ERROR_MEDIA_CHANGE: u32 = 32; // atapi-legacy-rust/src/types_amd64.rs
pub const IDE_ERROR_MEDIA_CHANGE_REQ: u32 = 8; // atapi-legacy-rust/src/types_amd64.rs
pub const IDE_FEATURE_DISABLE_REVERT_TO_POWER_ON: u32 = 102; // storahci-rust/src/types_amd64.rs
pub const IDE_FEATURE_ENABLE_MSN: u32 = 149; // storahci-rust/src/types_amd64.rs
pub const IDE_FEATURE_PUIS_SPIN_UP: u32 = 7; // storahci-rust/src/types_amd64.rs
pub const IDE_FEATURE_SET_TRANSFER_MODE: u32 = 3; // storahci-rust/src/types_amd64.rs
pub const IDE_LBA_MODE: u32 = 64; // storahci-rust/src/types_amd64.rs
pub const IDE_STATUS_DEVICE_FAULT: u32 = 32; // storahci-rust/src/types_amd64.rs
pub const IDE_STATUS_ERROR: u32 = 1; // atapi-legacy-rust/src/types_amd64.rs
pub const IDE_STATUS_INDEX: u32 = 2; // atapi-legacy-rust/src/types_amd64.rs
pub const ID_CMD: u32 = 236; // cdrom-rust/src/types_amd64.rs
pub const INQUIRYDATABUFFERSIZE: u32 = 36; // atapi-legacy-rust/src/types_amd64.rs
pub const IOCTL_STORAGE_EVENT_NOTIFICATION: u32 = 0x002D1C90; // classpnp-rust/src/glue.rs
pub const IO_DISK_INCREMENT: u32 = 1; // ntkrnl-sys/src/hal_bindgen_amd64.rs
pub const IO_NO_INCREMENT: u32 = 0; // ntkrnl-sys/src/hal_bindgen_amd64.rs
pub const IRP_MJ_CLOSE: u32 = 2; // ntkrnl-sys/src/hal_bindgen_amd64.rs
pub const IRP_MJ_CREATE: u32 = 0; // ntkrnl-sys/src/hal_bindgen_amd64.rs
pub const IRP_MJ_DEVICE_CONTROL: u32 = 14; // ntkrnl-sys/src/hal_bindgen_amd64.rs
pub const IRP_MJ_PNP: u32 = 27; // ntkrnl-sys/src/hal_bindgen_amd64.rs
pub const IRP_MJ_POWER: u32 = 22; // ntkrnl-sys/src/hal_bindgen_amd64.rs
pub const IRP_MJ_SCSI: u32 = 15; // ntkrnl-sys/src/hal_bindgen_amd64.rs
pub const IRP_MJ_SYSTEM_CONTROL: u32 = 23; // ntkrnl-sys/src/hal_bindgen_amd64.rs
pub const IRP_PAGING_IO: u32 = 2; // ntkrnl-sys/src/hal_bindgen_amd64.rs
pub const IRP_SYNCHRONOUS_PAGING_IO: u32 = 64; // ntkrnl-sys/src/hal_bindgen_amd64.rs
pub const IsNEC_98: u32 = 0; // ntdll-sys/src/bindgen_amd64.rs
pub const KEY_ALL_ACCESS: u32 = 983103; // ntdll-sys/src/bindgen_amd64.rs
pub const KEY_READ: DWORD = 0x20019; // shell32-rust/src/cfs_folder.rs
pub const KeyValuePartialInformation: u32 = 2; // config-rust/src/cmsysini.rs
pub const MAXULONG: u32 = 4294967295; // ntdll-sys/src/bindgen_amd64.rs
pub const MAX_IDE_DEVICE: u32 = 2; // pciidex-legacy-rust/src/types_amd64.rs
pub const MAX_SLOTS: usize = 16; // usbxhci-rust/src/controller.rs
pub const MODE_DSP_FUA_SUPPORTED: u32 = 16; // atapi-legacy-rust/src/types_amd64.rs
pub const MODE_DSP_WRITE_PROTECT: u32 = 128; // atapi-legacy-rust/src/types_amd64.rs
pub const MODE_PAGE_CACHING: u32 = 8; // atapi-legacy-rust/src/types_amd64.rs
pub const MODE_PAGE_CONTROL: u32 = 10; // atapi-legacy-rust/src/types_amd64.rs
pub const MODE_PAGE_ERROR_RECOVERY: u32 = 1; // atapi-legacy-rust/src/types_amd64.rs
pub const MODE_PAGE_FAULT_REPORTING: u32 = 28; // atapi-legacy-rust/src/types_amd64.rs
pub const MODE_SENSE_RETURN_ALL: u32 = 63; // atapi-legacy-rust/src/types_amd64.rs
pub const MODE_SENSE_SAVED_VALUES: u32 = 192; // atapi-legacy-rust/src/types_amd64.rs
pub const MmNonCached: MEMORY_CACHING_TYPE = 0; // ntkrnl-sys/src/mm.rs
pub const NonPagedPool: POOL_TYPE = 0; // msfs-rust/src/glue.rs
pub const OBJ_CASE_INSENSITIVE: u32 = 64; // ntdll-sys/src/bindgen_amd64.rs
pub const OBJ_KERNEL_HANDLE: u32 = 512; // ntdll-sys/src/bindgen_amd64.rs
pub const OBJ_PERMANENT: u32 = 16; // ntdll-sys/src/bindgen_amd64.rs
pub const PAGE_SIZE: u32 = 4096; // ntdll-sys/src/bindgen_amd64.rs
pub const PIO_MODE0: u32 = 1; // pciidex-legacy-rust/src/types_amd64.rs
pub const PIO_MODE1: u32 = 2; // pciidex-legacy-rust/src/types_amd64.rs
pub const PIO_MODE2: u32 = 4; // pciidex-legacy-rust/src/types_amd64.rs
pub const PLUGPLAY_REGKEY_DEVICE: u32 = 1; // classpnp-rust/src/glue.rs
pub const PNP_DEVICE_NOT_DISABLEABLE: u32 = 32; // ntkrnl-sys/src/hal_bindgen_amd64.rs
pub const PagedPool: POOL_TYPE = 1; // ks-rust/src/image.rs
pub const PowerDeviceD0: usize = 1; // kbdhid-rust/src/glue.rs
pub const PropertyExistsQuery: u32 = 1; // scsiport-rust/src/glue.rs
pub const PropertyStandardQuery: u32 = 0; // scsiport-rust/src/glue.rs
pub const READ_ATTRIBUTE_BUFFER_SIZE: u32 = 512; // cdrom-rust/src/types_amd64.rs
pub const READ_ONLY_DIRECT_ACCESS_DEVICE: u32 = 5; // atapi-legacy-rust/src/types_amd64.rs
pub const READ_THRESHOLD_BUFFER_SIZE: u32 = 512; // cdrom-rust/src/types_amd64.rs
pub const REG_DWORD: u32 = 4; // advapi32-sys/src/bindgen_amd64.rs
pub const REG_OPTION_VOLATILE: u32 = 1; // advapi32-sys/src/bindgen_amd64.rs
pub const REQUEST_FLAG_LBA48: ULONG = 0x00000001; // freeldr-rust/src/hwide.rs
pub const REQUEST_FLAG_PACKET_COMMAND: ULONG = 0x00000004; // freeldr-rust/src/hwide.rs
pub const REQUEST_FLAG_READ_WRITE_MULTIPLE: ULONG = 0x00000002; // freeldr-rust/src/hwide.rs
pub const REQUEST_FLAG_SET_DEVICE_REGISTER: ULONG = 0x00000008; // freeldr-rust/src/hwide.rs
pub const SCSIOP_ATA_PASSTHROUGH12: u32 = 161; // atapi-legacy-rust/src/types_amd64.rs
pub const SCSIOP_INQUIRY: u32 = 18; // atapi-legacy-rust/src/types_amd64.rs
pub const SCSIOP_MODE_SENSE: u32 = 26; // atapi-legacy-rust/src/types_amd64.rs
pub const SCSIOP_READ_CAPACITY: u32 = 37; // atapi-legacy-rust/src/types_amd64.rs
pub const SCSIOP_REQUEST_SENSE: u32 = 3; // atapi-legacy-rust/src/types_amd64.rs
pub const SCSISTAT_CHECK_CONDITION: u32 = 2; // atapi-legacy-rust/src/types_amd64.rs
pub const SCSISTAT_GOOD: u32 = 0; // atapi-legacy-rust/src/types_amd64.rs
pub const SCSI_ADSENSE_ILLEGAL_BLOCK: u32 = 33; // atapi-legacy-rust/src/types_amd64.rs
pub const SCSI_ADSENSE_ILLEGAL_COMMAND: u32 = 32; // atapi-legacy-rust/src/types_amd64.rs
pub const SCSI_ADSENSE_INTERNAL_TARGET_FAILURE: u32 = 68; // atapi-legacy-rust/src/types_amd64.rs
pub const SCSI_ADSENSE_INVALID_CDB: u32 = 36; // atapi-legacy-rust/src/types_amd64.rs
pub const SCSI_ADSENSE_INVALID_FIELD_PARAMETER_LIST: u32 = 38; // atapi-legacy-rust/src/types_amd64.rs
pub const SCSI_ADSENSE_LUN_COMMUNICATION: u32 = 8; // atapi-legacy-rust/src/types_amd64.rs
pub const SCSI_ADSENSE_MEDIUM_CHANGED: u32 = 40; // atapi-legacy-rust/src/types_amd64.rs
pub const SCSI_ADSENSE_NO_MEDIA_IN_DEVICE: u32 = 58; // atapi-legacy-rust/src/types_amd64.rs
pub const SCSI_ADSENSE_NO_SENSE: u32 = 0; // atapi-legacy-rust/src/types_amd64.rs
pub const SCSI_ADSENSE_OPERATOR_REQUEST: u32 = 90; // atapi-legacy-rust/src/types_amd64.rs
pub const SCSI_ADSENSE_UNRECOVERED_ERROR: u32 = 17; // atapi-legacy-rust/src/types_amd64.rs
pub const SCSI_ADSENSE_WRITE_PROTECT: u32 = 39; // atapi-legacy-rust/src/types_amd64.rs
pub const SCSI_SENSEQ_INTERNAL_TARGET_FAILURE: u32 = 0; // atapi-legacy-rust/src/types_amd64.rs
pub const SCSI_SENSEQ_LOGICAL_ADDRESS_OUT_OF_RANGE: u32 = 0; // atapi-legacy-rust/src/types_amd64.rs
pub const SCSI_SENSEQ_MEDIUM_REMOVAL: u32 = 1; // atapi-legacy-rust/src/types_amd64.rs
pub const SCSI_SENSEQ_UNRECOVERED_READ_ERROR: u32 = 0; // atapi-legacy-rust/src/types_amd64.rs
pub const SCSI_SENSE_ABORTED_COMMAND: u32 = 11; // atapi-legacy-rust/src/types_amd64.rs
pub const SCSI_SENSE_DATA_PROTECT: u32 = 7; // atapi-legacy-rust/src/types_amd64.rs
pub const SCSI_SENSE_DESCRIPTOR_TYPE_ATA_STATUS_RETURN: u32 = 9; // atapi-legacy-rust/src/types_amd64.rs
pub const SCSI_SENSE_ERRORCODE_DESCRIPTOR_CURRENT: u32 = 114; // atapi-legacy-rust/src/types_amd64.rs
pub const SCSI_SENSE_ERRORCODE_FIXED_CURRENT: u32 = 112; // atapi-legacy-rust/src/types_amd64.rs
pub const SCSI_SENSE_HARDWARE_ERROR: u32 = 4; // atapi-legacy-rust/src/types_amd64.rs
pub const SCSI_SENSE_ILLEGAL_REQUEST: u32 = 5; // atapi-legacy-rust/src/types_amd64.rs
pub const SCSI_SENSE_MEDIUM_ERROR: u32 = 3; // atapi-legacy-rust/src/types_amd64.rs
pub const SCSI_SENSE_NOT_READY: u32 = 2; // atapi-legacy-rust/src/types_amd64.rs
pub const SCSI_SENSE_NO_SENSE: u32 = 0; // atapi-legacy-rust/src/types_amd64.rs
pub const SCSI_SENSE_RECOVERED_ERROR: u32 = 1; // atapi-legacy-rust/src/types_amd64.rs
pub const SCSI_SENSE_UNIT_ATTENTION: u32 = 6; // atapi-legacy-rust/src/types_amd64.rs
pub const SCSI_SESNEQ_COMM_CRC_ERROR: u32 = 3; // atapi-legacy-rust/src/types_amd64.rs
pub const SERVICE_ACTION_READ_CAPACITY16: u32 = 16; // atapi-legacy-rust/src/types_amd64.rs
pub const SMART_CMD: u32 = 176; // cdrom-rust/src/types_amd64.rs
pub const SMART_EXTENDED_SELFTEST_CAPTIVE: u32 = 130; // cdrom-rust/src/types_amd64.rs
pub const SMART_IDE_ERROR: u32 = 1; // cdrom-rust/src/types_amd64.rs
pub const SMART_LOG_SECTOR_SIZE: u32 = 512; // cdrom-rust/src/types_amd64.rs
pub const SMART_NO_ERROR: u32 = 0; // cdrom-rust/src/types_amd64.rs
pub const SMART_SHORT_SELFTEST_CAPTIVE: u32 = 129; // cdrom-rust/src/types_amd64.rs
pub const SRB_FLAGS_BYPASS_FROZEN_QUEUE: u32 = 16; // atapi-legacy-rust/src/types_amd64.rs
pub const SRB_FLAGS_BYPASS_LOCKED_QUEUE: u32 = 524288; // atapi-legacy-rust/src/types_amd64.rs
pub const SRB_FLAGS_DATA_IN: u32 = 64; // atapi-legacy-rust/src/types_amd64.rs
pub const SRB_FLAGS_DISABLE_AUTOSENSE: u32 = 32; // atapi-legacy-rust/src/types_amd64.rs
pub const SRB_FLAGS_NO_QUEUE_FREEZE: u32 = 256; // atapi-legacy-rust/src/types_amd64.rs
pub const SRB_FLAGS_UNSPECIFIED_DIRECTION: u32 = 192; // atapi-legacy-rust/src/types_amd64.rs
pub const SRB_FUNCTION_IO_CONTROL: u32 = 2; // atapi-legacy-rust/src/types_amd64.rs
pub const SRB_FUNCTION_RELEASE_DEVICE: u32 = 6; // atapi-legacy-rust/src/types_amd64.rs
pub const SRB_FUNCTION_UNLOCK_QUEUE: u32 = 25; // atapi-legacy-rust/src/types_amd64.rs
pub const SRB_STATUS_ABORTED: u32 = 2; // atapi-legacy-rust/src/types_amd64.rs
pub const SRB_STATUS_AUTOSENSE_VALID: u32 = 128; // atapi-legacy-rust/src/types_amd64.rs
pub const SRB_STATUS_BUSY: u32 = 5; // atapi-legacy-rust/src/types_amd64.rs
pub const SRB_STATUS_BUS_RESET: u32 = 14; // atapi-legacy-rust/src/types_amd64.rs
pub const SRB_STATUS_DATA_OVERRUN: u32 = 18; // atapi-legacy-rust/src/types_amd64.rs
pub const SRB_STATUS_ERROR: u32 = 4; // atapi-legacy-rust/src/types_amd64.rs
pub const SRB_STATUS_INSUFFICIENT_RESOURCES: u32 = 38; // atapi-legacy-rust/src/types_amd64.rs
pub const SRB_STATUS_INTERNAL_ERROR: u32 = 48; // atapi-legacy-rust/src/types_amd64.rs
pub const SRB_STATUS_INVALID_REQUEST: u32 = 6; // atapi-legacy-rust/src/types_amd64.rs
pub const SRB_STATUS_NO_DEVICE: u32 = 8; // atapi-legacy-rust/src/types_amd64.rs
pub const SRB_STATUS_PENDING: u32 = 0; // atapi-legacy-rust/src/types_amd64.rs
pub const SRB_STATUS_QUEUE_FROZEN: u32 = 64; // atapi-legacy-rust/src/types_amd64.rs
pub const SRB_STATUS_REQUEST_SENSE_FAILED: u32 = 16; // atapi-legacy-rust/src/types_amd64.rs
pub const SRB_STATUS_SUCCESS: u32 = 1; // atapi-legacy-rust/src/types_amd64.rs
pub const SRB_STATUS_TIMEOUT: u32 = 9; // atapi-legacy-rust/src/types_amd64.rs
pub const SRB_TYPE_SCSI_REQUEST_BLOCK: u32 = 0; // cdrom-rust/src/types_amd64.rs
pub const STATUS_ADAPTER_HARDWARE_ERROR: i32 = 0xC00000C2u32 as i32; // scsiport-rust/src/glue.rs
pub const STATUS_BUFFER_OVERFLOW: i32 = 0x80000005_u32 as i32; // fastfat-rust/src/glue.rs
pub const STATUS_BUFFER_TOO_SMALL: NTSTATUS = 0xC0000023u32 as NTSTATUS; // kernel32-rust/src/console/history.rs
pub const STATUS_CANCELLED: NTSTATUS = 0xC000_0120u32 as NTSTATUS; // msfs-rust/src/glue.rs
pub const STATUS_CONTINUE_COMPLETION: NTSTATUS = 0x00000000_u32 as i32; // hidusb-rust/src/glue.rs
pub const STATUS_DEVICE_BUSY: i32 = 0x80000011u32 as i32; // classpnp-rust/src/glue.rs
pub const STATUS_DEVICE_DOES_NOT_EXIST: i32 = 0xC00000C0u32 as i32; // classpnp-rust/src/glue.rs
pub const STATUS_DEVICE_NOT_CONNECTED: i32 = 0xC000009D_u32 as i32; // ntdll-sys/src/constants.rs
pub const STATUS_DEVICE_REMOVED: i32 = 0xC00002B6u32 as i32; // partmgr-rust/src/glue.rs
pub const STATUS_INFO_LENGTH_MISMATCH: i32 = 0xC0000004u32 as i32; // kernel32-rust/src/client/sysinfo.rs
pub const STATUS_INSUFFICIENT_RESOURCES: NTSTATUS = 0xC000_009Au32 as NTSTATUS; // msfs-rust/src/glue.rs
pub const STATUS_INVALID_DEVICE_REQUEST: NTSTATUS = 0xC000_0010u32 as NTSTATUS; // mup-rust/src/glue.rs
pub const STATUS_INVALID_PARAMETER: i32 = 0xC000_000Du32 as i32; // network/src/lib.rs
pub const STATUS_IO_DEVICE_ERROR: i32 = 0xC0000185u32 as i32; // classpnp-rust/src/glue.rs
pub const STATUS_IO_TIMEOUT: i32 = 0xC00000B5u32 as i32; // classpnp-rust/src/glue.rs
pub const STATUS_MORE_PROCESSING_REQUIRED: i32 = 0xC0000016u32 as i32; // classpnp-rust/src/glue.rs
pub const STATUS_NOT_IMPLEMENTED: NTSTATUS = 0xC0000002u32 as NTSTATUS; // kernel32-rust/src/console/history.rs
pub const STATUS_NOT_SUPPORTED: i32 = 0xC00000BB_u32 as i32; // ntdll-sys/src/constants.rs
pub const STATUS_NO_SUCH_DEVICE: i32 = 0xC000000Eu32 as i32; // afd-rust/src/glue.rs
pub const STATUS_PENDING: NTSTATUS = 0x00000103; // kernel32-rust/src/client/synch.rs
pub const STATUS_REVISION_MISMATCH: i32 = 0xC0000059_u32 as i32; // ntdll-sys/src/constants.rs
pub const STATUS_SUCCESS: i32 = 0; // network/src/lib.rs
pub const STATUS_UNSUCCESSFUL: NTSTATUS = 0xC000_0001u32 as NTSTATUS; // msfs-rust/src/glue.rs
pub const STATUS_WMI_GUID_NOT_FOUND: i32 = 0xC0000295u32 as i32; // classpnp-rust/src/glue.rs
pub const STATUS_WMI_INSTANCE_NOT_FOUND: i32 = 0xC0000296u32 as i32; // classpnp-rust/src/glue.rs
pub const STORAGE_ADDRESS_TYPE_BTL8: u32 = 0; // cdrom-rust/src/types_amd64.rs
pub const STORAGE_EVENT_ALL: u32 = 7; // cdrom-rust/src/types_amd64.rs
pub const STORAGE_EVENT_NOTIFICATION_VERSION_V1: u32 = 1; // cdrom-rust/src/types_amd64.rs
pub const THREAD_ALL_ACCESS: u32 = 2032639; // ntkrnl-sys/src/hal_bindgen_amd64.rs
pub const VPD_BLOCK_DEVICE_CHARACTERISTICS: u32 = 177; // atapi-legacy-rust/src/types_amd64.rs
pub const VPD_BLOCK_LIMITS: u32 = 176; // atapi-legacy-rust/src/types_amd64.rs
pub const VPD_DEVICE_IDENTIFIERS: u32 = 131; // atapi-legacy-rust/src/types_amd64.rs
pub const VPD_LOGICAL_BLOCK_PROVISIONING: u32 = 178; // atapi-legacy-rust/src/types_amd64.rs
pub const VPD_SERIAL_NUMBER: u32 = 128; // atapi-legacy-rust/src/types_amd64.rs
pub const VPD_SUPPORTED_PAGES: u32 = 0; // atapi-legacy-rust/src/types_amd64.rs
pub const VpdCodeSetAscii: _VPD_CODE_SET = _VPD_CODE_SET_VpdCodeSetAscii; // island enum
pub const VpdCodeSetBinary: _VPD_CODE_SET = _VPD_CODE_SET_VpdCodeSetBinary; // island enum
pub const VpdIdentifierTypeFCPHName: _VPD_IDENTIFIER_TYPE = _VPD_IDENTIFIER_TYPE_VpdIdentifierTypeFCPHName; // island enum
pub const VpdIdentifierTypeVendorId: _VPD_IDENTIFIER_TYPE = _VPD_IDENTIFIER_TYPE_VpdIdentifierTypeVendorId; // island enum
pub const WMIREG_ACTION_DEREGISTER: u32 = 2; // ntkrnl-sys/src/hal_bindgen_amd64.rs
pub const WMIREG_ACTION_REGISTER: u32 = 1; // classpnp-rust/src/glue.rs
pub const WMIREG_FLAG_INSTANCE_PDO: u32 = 32; // classpnp-rust/src/types_amd64.rs
extern "C" {
    pub fn DbgPrint(fmt: *const core::ffi::c_char, ...) -> u32; // arm64input-cargo/src/main.rs
    pub fn GetIRPMinorFunctionString(minor: u8) -> *const i8; // scsiport-rust/src/pdo.rs
    pub fn InitializeListHead(ListHead: PLIST_ENTRY); // winlogon-sys/src/bindgen_amd64.rs
    pub fn IsListEmpty(ListHead: *const LIST_ENTRY) -> BOOLEAN; // winlogon-sys/src/bindgen_amd64.rs
    pub fn PushEntryList(ListHead: *mut SINGLE_LIST_ENTRY, Entry: *mut SINGLE_LIST_ENTRY); // cdfs-rust/src/strucsup.rs
    pub fn RemoveEntryList(Entry: PLIST_ENTRY) -> BOOLEAN; // winlogon-sys/src/bindgen_amd64.rs
    pub fn RemoveHeadList(ListHead: PLIST_ENTRY) -> PLIST_ENTRY; // winlogon-sys/src/bindgen_amd64.rs
    pub fn RtlULongAdd(a: u32, b: u32, result: *mut u32) -> i32; // cdrom-rust/src/ioctl.rs
    pub fn SptiHandleScsiPassthru( DeviceObject: PDEVICE_OBJECT, Irp: PIRP, MaximumTransferLength: ULONG, MaximumPhysicalPages: ULONG, ) -> NTSTATUS; // usbstor-rust/src/disk.rs
    pub fn strcmp(a: *const c_char, b: *const c_char) -> c_int; // browser-rust/src/prelude.rs
    pub fn strlen(s: *const i8) -> usize; // advapi32-rust/src/prelude.rs
    pub fn strstr(haystack: *const i8, needle: *const i8) -> *mut i8; // fstub-rust/src/disksup.rs
}
extern "system" {
    pub fn ExFreePoolWithTag(P: PVOID, Tag: ULONG); // iomgr-rust/src/file.rs
    pub fn InitializeSListHead(ListHead: PSLIST_HEADER); // aclui-rust/src/types_amd64.rs
    pub fn IoCallDriver(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS; // fastfat-rust/src/flush.rs
    pub fn IoCompleteRequest(Irp: PIRP, PriorityBoost: CCHAR); // hal-rust/src/x86/halpnpdd.rs
    pub fn KeClearEvent(Event: PRKEVENT); // ntkrnl-sys/src/bindgen_amd64.rs
    pub fn KeGetCurrentIrql() -> u8; fn ClassModeSense( DeviceObject: PDEVICE_OBJECT, buffer: *mut i8, bufferSize: u32, pageCode: u8, ) -> u32; // disk-rust/src/disk.rs
    pub fn KeSetEvent(Event: PRKEVENT, Increment: KPRIORITY, Wait: BOOLEAN) -> LONG; // ntkrnl-sys/src/bindgen_amd64.rs
    pub fn ObDereferenceObject(Object: PVOID); // fastfat-rust/src/create.rs
    pub fn PoCallDriver(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS; // hal-rust/src/arm64/pnpdd.rs
    pub fn PoRequestPowerIrp( DeviceObject: PDEVICE_OBJECT, MinorFunction: UCHAR, PowerState: POWER_STATE, CompletionFunction: PREQUEST_POWER_COMPLETE, Context: PVOID, Irp: *mut PIRP, ) -> NTSTATUS; // usbhub-rust/src/power.rs
    pub fn PoSetPowerState( DeviceObject: PDEVICE_OBJECT, Type: ULONG, State: POWER_STATE, ) -> POWER_STATE; // acpi-rust/src/buspdo.rs
    pub fn PoStartNextPowerIrp(Irp: PIRP); // hal-rust/src/arm64/pnpdd.rs
    pub fn RtlAssert(FailedAssertion: PVOID, FileName: PVOID, LineNumber: ULONG, Message: PCHAR); // ntdll-sys/src/bindgen_amd64.rs
    pub fn _InterlockedExchange(Target: *mut i32, Value: i32) -> i32; // disk-rust/src/disk.rs
}
// ==== end port resolve ====
pub use win32macros_rust::{ARRAYSIZE, ASSERT, C_ASSERT, DWLP_DLGPROC, DWLP_MSGRESULT, DWLP_USER, FIELD_OFFSET, GET_X_LPARAM, GET_Y_LPARAM, GetBValue, GetGValue, GetRValue, HIBYTE, HIWORD, INVALID_HANDLE_VALUE, InitializeObjectAttributes, IsContainerPartition, IsFTPartition, IsRecognizedPartition, LANGIDFROMLCID, LOBYTE, LOWORD, MAKEINTRESOURCEA, MAKEINTRESOURCEW, MAKELANGID, MAKELCID, MAKELONG, MAKEWORD, PAGE_ROUND_UP, PRIMARYLANGID, RGB, RTL_CONSTANT_STRING, RTL_FIELD_SIZE, RTL_NUMBER_OF, RTL_NUMBER_OF_FIELD, RTL_SIZEOF_THROUGH_FIELD, RtlCopyMemory, RtlFillMemory, RtlGetProcessHeap, RtlMoveMemory, RtlZeroMemory, SORTIDFROMLCID, SUBLANGID, UNICODE_NULL, UNREFERENCED_PARAMETER, UlongToPtr, ZeroMemory, _countof, wide_bytes};
pub use strsafe_rust::{RtlStringCbPrintfW, RtlStringCchPrintfExW, RtlStringCchPrintfW, StringCbCatW, StringCbCopyNExW, StringCbCopyW, StringCbPrintfExW, StringCbPrintfW, StringCchCatW, StringCchCopyNExW, StringCchCopyW, StringCchLengthW, StringCchPrintfW, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerW, RtlStringLengthWorkerW};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
pub const ATAPORT_FN_FIELD: u32 = 0x28; // clang, enum.c
pub const ATAPORT_RN_FIELD: u32 = 8; // clang, enum.c
pub const ATAPORT_TAG: u32 = 0x50656449; // clang, atapi.c
pub const ATA_CHANNEL_FLAG_IS_AHCI: u32 = 8; // clang, fdo.c
pub const ATA_CHANNEL_FLAG_IS_EXTERNAL: u32 = 2; // clang, fdo.c
pub const ATA_CHANNEL_FLAG_NCQ: u32 = 4; // clang, fdo.c
pub const ATA_CHANNEL_FLAG_PIO_FOR_LBA48_XFER: u32 = 0x10; // clang, fdo.c
pub const ATA_CHANNEL_FLAG_PIO_VIA_DMA: u32 = 1; // clang, fdo.c
pub const ATA_DEVICE_REQUEST_SIGNATURE: u32 = 0x72415441; // clang, fdo.c
pub const ATA_LOCAL_BUFFER_SIZE: u32 = 0x23C; // clang, dev_error.c
pub const ATA_MAX_DEVICE: u32 = 0xF; // clang, atapi.c
pub const ATA_MAX_SECTORS_PER_IO: u32 = 0x100; // clang, satl.c
pub const ATA_MIN_BUFFER_ALIGNMENT: u32 = 1; // clang, atapi.c
pub const ATA_MIN_SECTOR_SIZE: u32 = 0x200; // clang, enum.c
pub const ATA_RESERVED_PAGES: u32 = 4; // clang, fdo.c
pub const DEVICE_CACHE_INQUIRY: u32 = 0x4000; // clang, dev_identify.c
pub const DEVICE_DESCRIPTOR_SENSE: u32 = 0x800; // clang, satl.c
pub const DEVICE_HAS_CDB_INTERRUPT: u32 = 0x20; // clang, dev_identify.c
pub const DEVICE_HAS_FUA: u32 = 8; // clang, enum.c
pub const DEVICE_HAS_MEDIA_STATUS: u32 = 0x20; // clang, dev_config.c
pub const DEVICE_IS_ATAPI: u32 = 0x10; // clang, dev_identify.c
pub const DEVICE_IS_NEC_CDR260: u32 = 0x80; // clang, dev_identify.c
pub const DEVICE_IS_PDO_REMOVABLE: u32 = 0x100; // clang, pdo.c
pub const DEVICE_IS_SUPER_FLOPPY: u32 = 0x80; // clang, atapi.c
pub const DEVICE_LBA48: u32 = 4; // clang, dev_power.c
pub const DEVICE_LBA_MODE: u32 = 2; // clang, dev_config.c
pub const DEVICE_NCQ: u32 = 0x10; // clang, dev_config.c
pub const DEVICE_NEED_DMA_DIRECTION: u32 = 0x40; // clang, dev_identify.c
pub const DEVICE_PIO_FOR_LBA48_XFER: u32 = 0x2000; // clang, pdo.c
pub const DEVICE_PIO_ONLY: u32 = 1; // clang, dev_config.c
pub const DEVICE_PIO_VIA_DMA: u32 = 0x1000; // clang, pdo.c
pub const DEVICE_PNP_STARTED: u32 = 0x400; // clang, dev_timings.c
pub const DEVICE_QUEUE_DEPTH_MASK: u32 = 0xFF00; // clang, scsi.c
pub const DEVICE_QUEUE_DEPTH_SHIFT: u32 = 8; // clang, enum.c
pub const DEVICE_SENSE_DATA_REPORTING: u32 = 0x40; // clang, dev_config.c
pub const DEVICE_UNINITIALIZED: u32 = 0x200; // clang, dev_identify.c
pub const DEV_WORKER_FLAG_HOLD_REFERENCE: u32 = 1; // clang, portstate.c
pub const DEV_WORKER_FLAG_REMOVED: u32 = 2; // clang, portstate.c
pub const DO_IS_FDO: u32 = 0x80000000; // clang, atapi.c
pub const IDE_ACPI_TIMING_MODE_NOT_SUPPORTED: u32 = 0xFFFFFFFF; // clang, dev_timings.c
pub const IDE_COMMAND_REQUEST_SENSE_DATA_EXT: u32 = 0xB; // clang, dev_error.c
pub const IDE_DEVICE_FUA_NCQ: u32 = 0x80; // clang, satl.c
pub const IDE_DRIVE_SELECT_SLAVE: u32 = 0x10; // clang, satl.c
pub const IDE_ERROR_WRITE_PROTECT: u32 = 0x40; // clang, satl.c
pub const IDE_GP_LOG_NCQ_COMMAND_ERROR_ADDRESS: u32 = 0x10; // clang, dev_error.c
pub const IDE_GP_LOG_SECTOR_SIZE: u32 = 0x200; // clang, dev_error.c
pub const IO_ERR_CONTROLLER_ERROR: NTSTATUS = 0xC004000B_u32 as NTSTATUS; // clang, dev_error.c
pub const IO_ERR_PARITY: NTSTATUS = 0xC0040005_u32 as NTSTATUS; // clang, dev_error.c
pub const IO_ERR_TIMEOUT: NTSTATUS = 0xC0040009_u32 as NTSTATUS; // clang, dev_error.c
pub const MWDMA_ALL: u32 = 0x700; // clang, dev_error.c
pub const PCIIDEX_INTERFACE_VERSION: u32 = 1; // clang, fdo.c
pub const PIO_ALL: u32 = 0x1F; // clang, dev_config.c
pub const PORT_FLAG_CHANNEL_ATTACHED: u32 = 0x80; // clang, fdo.c
pub const PORT_FLAG_EXIT_THREAD: u32 = 0x80000000; // clang, fdo.c
pub const PORT_FLAG_IO_TIMER_ACTIVE: u32 = 0x40; // clang, fdo.c
pub const PORT_FLAG_IS_AHCI: u32 = 0x10; // clang, fdo.c
pub const PORT_FLAG_IS_EXTERNAL: u32 = 4; // clang, fdo.c
pub const PORT_FLAG_IS_SIMPLEX: u32 = 1; // clang, fdo.c
pub const PORT_FLAG_NCQ: u32 = 8; // clang, enum.c
pub const PORT_FLAG_PIO_FOR_LBA48_XFER: u32 = 0x200; // clang, fdo.c
pub const PORT_FLAG_PIO_ONLY: u32 = 0x100; // clang, fdo.c
pub const PORT_FLAG_PIO_VIA_DMA: u32 = 2; // clang, dev_error.c
pub const PORT_FLAG_SYMLINK_CREATED: u32 = 0x20; // clang, fdo.c
pub const PORT_INT_FLAG_IGNORE_LINK_IRQ: u32 = 2; // clang, dev_identify.c
pub const PORT_INT_FLAG_IS_IO_ACTIVE: u32 = 1; // clang, fdo.c
pub const PORT_QUEUE_FLAG_EXCLUSIVE_MODE: u32 = 1; // clang, scsi.c
pub const PORT_QUEUE_FLAG_SIGNAL_STOP: u32 = 2; // clang, portstate.c
pub const QUEUE_FLAGS_FROZEN: u32 = 0x80077; // clang, scsi.c
pub const QUEUE_FLAGS_FROZEN_NOT_BYPASS: u32 = 0x67; // clang, scsi.c
pub const QUEUE_FLAG_FROZEN_PNP: u32 = 4; // clang, enum.c
pub const QUEUE_FLAG_FROZEN_PORT_BUSY: u32 = 1; // clang, portstate.c
pub const QUEUE_FLAG_FROZEN_POWER: u32 = 0x20; // clang, dev_power.c
pub const QUEUE_FLAG_FROZEN_QUEUE_FREEZE: u32 = 0x10; // clang, scsi.c
pub const QUEUE_FLAG_FROZEN_QUEUE_LOCK: u32 = 0x80000; // clang, scsi.c
pub const QUEUE_FLAG_FROZEN_REMOVED: u32 = 0x40; // clang, pdo.c
pub const QUEUE_FLAG_FROZEN_SLOT: u32 = 2; // clang, scsi.c
pub const QUEUE_FLAG_SIGNAL_STOP: u32 = 8; // clang, scsi.c
pub const REQUEST_DMA_FLAGS: u32 = 0x200001; // clang, satl.c
pub const REQUEST_EXCLUSIVE_ACCESS_FLAGS: u32 = 0x4200; // clang, scsi.c
pub const REQUEST_FLAG_DATA_IN: u32 = 0x40; // clang, dev_error.c
pub const REQUEST_FLAG_DATA_OUT: u32 = 0x80; // clang, satl.c
pub const REQUEST_FLAG_DMA: u32 = 1; // clang, dev_error.c
pub const REQUEST_FLAG_EXCLUSIVE: u32 = 0x4000; // clang, satl.c
pub const REQUEST_FLAG_FUA: u32 = 0x100; // clang, satl.c
pub const REQUEST_FLAG_HAS_LOCAL_BUFFER: u32 = 0x8000; // clang, dev_error.c
pub const REQUEST_FLAG_HAS_MDL: u32 = 0x20000; // clang, scsi.c
pub const REQUEST_FLAG_HAS_RESERVED_MAPPING: u32 = 0x80000; // clang, scsi.c
pub const REQUEST_FLAG_HAS_SG_LIST: u32 = 0x10000; // clang, scsi.c
pub const REQUEST_FLAG_HAS_TASK_FILE: u32 = 0x400; // clang, dev_error.c
pub const REQUEST_FLAG_INTERNAL: u32 = 0x400000; // clang, portstate.c
pub const REQUEST_FLAG_NCQ: u32 = 4; // clang, dev_error.c
pub const REQUEST_FLAG_NO_KEEP_AWAKE: u32 = 0x100000; // clang, scsi.c
pub const REQUEST_FLAG_PASSTHROUGH: u32 = 0x800000; // clang, dev_error.c
pub const REQUEST_FLAG_POLL: u32 = 0x80000000; // clang, dev_identify.c
pub const REQUEST_FLAG_PROGRAM_DMA: u32 = 0x200000; // clang, scsi.c
pub const REQUEST_FLAG_READ_WRITE: u32 = 0x20; // clang, satl.c
pub const REQUEST_FLAG_SAVE_TASK_FILE: u32 = 0x200; // clang, dev_error.c
pub const REQUEST_STATE_FREEZE_QUEUE: u32 = 3; // clang, dev_error.c
pub const REQUEST_STATE_NOT_STARTED: u32 = 1; // clang, scsi.c
pub const REQUEST_STATE_REQUEUE: u32 = 2; // clang, portstate.c
pub const SCSI_ADSENSE_ADDRESS_MARK_NOT_FOUND_FOR_DATA_FIELD: u32 = 0x13; // clang, satl.c
pub const SCSI_SENSEQ_ATA_PASS_THROUGH_INFORMATION_AVAILABLE: u32 = 0x1D; // clang, satl.c
pub const SRB_FLAG_LOW_MEM_RETRY: u32 = 0x100; // clang, scsi.c
pub const SRB_FLAG_PIO_RETRY: u32 = 0x200; // clang, dev_error.c
pub const SRB_FLAG_RETRY_COUNT_MASK: u32 = 0xFF; // clang, dev_error.c
pub const SWDMA_ALL: u32 = 0xE0; // clang, dev_error.c
pub const UDMA_ALL: u32 = 0x3F800; // clang, dev_error.c
pub const WORKER_FLAG_COMPLETE_PORT_ENUM_EVENT: u32 = 2; // clang, portstate.c
pub const WORKER_FLAG_NEED_RESCAN: u32 = 1; // clang, dev_error.c
pub const _WMIGUIDREGINFO: u32 = 8; // clang, atapi.c
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// ATA_CONNECTION_STATUS: c2rust, dev_identify.c
pub type ATA_CONNECTION_STATUS = _ATA_CONNECTION_STATUS;
// ATA_DEVICE_STATUS: c2rust, dev_identify.c
pub type ATA_DEVICE_STATUS = _ATA_DEVICE_STATUS;
// ATA_DEVICE_TYPE: c2rust, dev_identify.c
pub type ATA_DEVICE_TYPE = _ATA_DEVICE_TYPE;
// ATA_ERROR_LOG_VALUE: c2rust, dev_error.c
pub type ATA_ERROR_LOG_VALUE = _ATA_ERROR_LOG_VALUE;
// ATA_PORT_ACTION: c2rust, enum.c
pub type ATA_PORT_ACTION = _ATA_PORT_ACTION;
// IO_ALLOCATION_ACTION: c2rust, scsi.c
pub type IO_ALLOCATION_ACTION = _IO_ALLOCATION_ACTION;
// PORT_NOTIFICATION_TYPE: c2rust, portstate.c
pub type PORT_NOTIFICATION_TYPE = _PORT_NOTIFICATION_TYPE;
// SYSCTL_IRP_DISPOSITION: c2rust, wmi.c
pub type SYSCTL_IRP_DISPOSITION = _SYSCTL_IRP_DISPOSITION;
// _HEADER: c2rust, satl.c
#[derive(Copy, Clone)]
#[repr(C)]
pub union _HEADER {
    pub Header6: PMODE_PARAMETER_HEADER,
    pub Header10: PMODE_PARAMETER_HEADER10,
}
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
extern "C" {
    pub fn SptiHandleAtaPassthru(
        DeviceObject: PDEVICE_OBJECT,
        Irp: PIRP,
        MaximumTransferLength: ULONG,
        MaximumPhysicalPages: ULONG,
    ) -> NTSTATUS; // ioctl.c
}
extern "system" {
    pub fn PoRegisterDeviceForIdleDetection(
        DeviceObject: *mut _DEVICE_OBJECT,
        ConservationIdleTime: ULONG,
        PerformanceIdleTime: ULONG,
        State: DEVICE_POWER_STATE,
    ) -> PULONG; // pdo.c
    pub fn WmiCompleteRequest(
        DeviceObject: PDEVICE_OBJECT,
        Irp: PIRP,
        Status: NTSTATUS,
        BufferUsed: ULONG,
        PriorityBoost: CCHAR,
    ) -> NTSTATUS; // wmi.c
    pub fn WmiSystemControl(
        WmiLibInfo: PWMILIB_CONTEXT,
        DeviceObject: PDEVICE_OBJECT,
        Irp: PIRP,
        IrpDisposition: PSYSCTL_IRP_DISPOSITION,
    ) -> NTSTATUS; // wmi.c
}
// ==== end port externs ====

// ==== port resolve: cross-TU definitions (generated) ====
pub use crate::atapi::r#enum::AtaCopyIdStringSafe;
pub use crate::atapi::r#enum::AtaCopyIdStringUnsafe;
pub use crate::atapi::satl::AtaCreateStandardInquiryData;
pub use crate::atapi::dev_power::AtaDeviceFlushPowerIrpQueue;
pub use crate::atapi::dev_power::AtaDeviceGetFlushCacheCommand;
pub use crate::atapi::portstate::AtaDeviceQueueEvent;
pub use crate::atapi::dev_identify::AtaDeviceSendIdentify;
pub use crate::atapi::r#enum::AtaDeviceSetAddressingMode;
pub use crate::atapi::ioctl::AtaDispatchDeviceControl;
pub use crate::atapi::pdo::AtaDispatchPnp;
pub use crate::atapi::dev_power::AtaDispatchPower;
pub use crate::atapi::scsi::AtaDispatchScsi;
pub use crate::atapi::wmi::AtaDispatchWmi;
pub use crate::atapi::fdo::AtaFdoDeviceListInsert;
pub use crate::atapi::fdo::AtaFdoFindDeviceByPath;
pub use crate::atapi::fdo::AtaFdoFindNextDeviceByPath;
pub use crate::atapi::fdo::AtaFdoPnp;
pub use crate::atapi::r#enum::AtaFdoQueryBusRelations;
pub use crate::atapi::atapi::AtaGetRegistryKey;
pub use crate::atapi::pdo::AtaPdoCompletionRoutine;
pub use crate::atapi::pdo::AtaPdoCreateDevice;
pub use crate::atapi::pdo::AtaPdoFreeDevice;
pub use crate::atapi::smart::AtaPdoHandleMiniportIdentify;
pub use crate::atapi::smart::AtaPdoHandleMiniportSmartVersion;
pub use crate::atapi::wmi::AtaPdoWmiRegistration;
pub use crate::atapi::atapi::AtaPnpInitializeCommonExtension;
pub use crate::atapi::atapi::AtaPnpQueryDeviceUsageNotification;
pub use crate::atapi::atapi::AtaPnpQueryInterface;
pub use crate::atapi::atapi::AtaPnpQueryPnpDeviceState;
pub use crate::atapi::atapi::AtaPnpRepeatRequest;
pub use crate::atapi::dev_power::AtaPortCheckDevicePowerState;
pub use crate::atapi::portstate::AtaPortCompleteInternalRequest;
pub use crate::atapi::dev_config::AtaPortDeviceProcessConfig;
pub use crate::atapi::dev_error::AtaPortDeviceProcessError;
pub use crate::atapi::dev_power::AtaPortDeviceProcessPowerChange;
pub use crate::atapi::dev_identify::AtaPortIdentifyDevice;
pub use crate::atapi::scsi::AtaPortIoTimer;
pub use crate::atapi::portstate::AtaPortNotification;
pub use crate::atapi::dev_timings::AtaPortSelectTimings;
pub use crate::atapi::portstate::AtaPortSendRequest;
pub use crate::atapi::portstate::AtaPortSignalWorkerThread;
pub use crate::atapi::portstate::AtaPortTimeout;
pub use crate::atapi::portstate::AtaPortWorkerSignalDpc;
pub use crate::atapi::portstate::AtaPortWorkerThread;
pub use crate::atapi::scsi::AtaReqAllocateMdl;
pub use crate::atapi::satl::AtaReqBuildReadLogTaskFile;
pub use crate::atapi::scsi::AtaReqCompletionDpc;
pub use crate::atapi::satl::AtaReqDmaTransferToPioTransfer;
pub use crate::atapi::satl::AtaReqExecuteScsi;
pub use crate::atapi::scsi::AtaReqFlushDeviceQueue;
pub use crate::atapi::scsi::AtaReqFreezeQueue;
pub use crate::atapi::scsi::AtaReqSendRequest;
pub use crate::atapi::satl::AtaReqSetFixedAtaSenseData;
pub use crate::atapi::scsi::AtaReqSetFixedSenseData;
pub use crate::atapi::scsi::AtaReqSetLbaInformation;
pub use crate::atapi::smart::AtaReqSmartIoControl;
pub use crate::atapi::scsi::AtaReqStartCompletionDpc;
pub use crate::atapi::scsi::AtaReqThawQueue;
pub use crate::atapi::scsi::AtaReqWaitForOutstandingIoToComplete;
pub use crate::atapi::atapi::AtaSetPortRegistryKey;
pub use crate::atapi::atapi::AtaSetRegistryKey;
pub use crate::atapi::atapi::AtaStorageNotificationlDpc;
pub use crate::atapi::r#enum::AtaSwapIdString;
pub use crate::atapi::atapi::AtaTypeCodeToName;
pub use crate::atapi::scsi::AtapCompletionDpc;
pub use crate::atapi::scsi::AtapCompletionQueueList;
pub use crate::atapi::atapi::AtapDriverRegistryPath;
pub use crate::atapi::atapi::AtapInPEMode;
// ==== end cross-TU definitions ====

// ==== port macros: the module's own C macros (generated -- `rustconv port macros`) ====
// drivers/storage/ide/atapi/atapi.h:326: #define ASSERT_REQUEST(Request) ASSERT((Request) && (Request)->Signature == ATA_DEVICE_REQUEST_SIGNATURE)
#[allow(unused_macros)]
macro_rules! ASSERT_REQUEST {
    ($Request:expr $(,)?) => {
        ASSERT!((($Request) && (*($Request)).Signature == ATA_DEVICE_REQUEST_SIGNATURE))
    };
}
#[allow(unused_imports)]
pub(crate) use ASSERT_REQUEST;
// drivers/storage/ide/atapi/atapi.h:165: #define ASSUME(cond) do { ASSERT(cond); __assume(cond); } while (0)
#[allow(unused_macros)]
macro_rules! ASSUME {
    ($cond:expr $(,)?) => {
        {
            ASSERT!($cond);
            __assume($cond);
        }
    };
}
#[allow(unused_imports)]
pub(crate) use ASSUME;
// drivers/storage/ide/atapi/atapi.h:624: header-inline `AtaCommandUseLba48`
#[allow(dead_code)]
#[inline]
pub unsafe fn AtaCommandUseLba48(SectorNumber: ULONG64, SectorCount: ULONG) -> BOOLEAN {
    ((((SectorNumber + SectorCount as ULONG64) >= (ATA_MAX_LBA_28 as ULONG64)) || (SectorCount > 0x100))) as u8
}
// drivers/storage/ide/atapi/atapi.h:579: header-inline `AtaMarshallScsiAddress`
#[allow(dead_code)]
#[inline]
pub unsafe fn AtaMarshallScsiAddress(PathId: ULONG, TargetId: ULONG, Lun: ULONG) -> ATA_SCSI_ADDRESS {
    let mut AtaScsiAddress: ATA_SCSI_ADDRESS = core::mem::zeroed();
    AtaScsiAddress.Lun = (Lun) as u8;
    AtaScsiAddress.TargetId = (TargetId) as u8;
    AtaScsiAddress.PathId = (PathId) as u8;
    AtaScsiAddress.IsValid = 0xAA;
    AtaScsiAddress
}
// drivers/storage/ide/atapi/atapi.h:607: header-inline `AtaPacketCommandUseDma`
#[allow(dead_code)]
#[inline]
pub unsafe fn AtaPacketCommandUseDma(OpCode: UCHAR) -> BOOLEAN {
    (((OpCode) as u32 == SCSIOP_READ6 ||
     (OpCode) as u32 == SCSIOP_WRITE6 ||
     (OpCode) as u32 == SCSIOP_READ ||
     (OpCode) as u32 == SCSIOP_WRITE ||
     (OpCode) as u32 == SCSIOP_READ12 ||
     (OpCode) as u32 == SCSIOP_WRITE12 ||
     (OpCode) as u32 == SCSIOP_READ16 ||
     (OpCode) as u32 == SCSIOP_WRITE16 ||
     (OpCode) as u32 == SCSIOP_READ_CD ||
     (OpCode) as u32 == SCSIOP_READ_CD_MSF)) as u8
}
// drivers/storage/ide/atapi/atapi.h:567: header-inline `AtaPortQueueEmpty`
#[allow(dead_code)]
#[inline]
pub unsafe fn AtaPortQueueEmpty(PortData: PATAPORT_PORT_DATA) -> BOOLEAN {
    let SlotsBitmap: ULONG = (*PortData).Worker.PausedSlotsBitmap | (*PortData).FreeSlotsBitmap;
    (((*PortData).MaxSlotsBitmap == SlotsBitmap)) as u8
}
// drivers/storage/ide/atapi/ioctl.c:14: #define AtaVerifyInBuffer(IoStack, Length) ((IoStack)->Parameters.DeviceIoControl.InputBufferLength >= (Length))
#[allow(unused_macros)]
macro_rules! AtaVerifyInBuffer {
    ($IoStack:expr, $Length:expr $(,)?) => {
        ((*($IoStack)).Parameters.DeviceIoControl.InputBufferLength >= ($Length))
    };
}
#[allow(unused_imports)]
pub(crate) use AtaVerifyInBuffer;
// drivers/storage/ide/atapi/ioctl.c:17: #define AtaVerifyOutBuffer(IoStack, Length) ((IoStack)->Parameters.DeviceIoControl.OutputBufferLength >= (Length))
#[allow(unused_macros)]
macro_rules! AtaVerifyOutBuffer {
    ($IoStack:expr, $Length:expr $(,)?) => {
        ((*($IoStack)).Parameters.DeviceIoControl.OutputBufferLength >= ($Length))
    };
}
#[allow(unused_imports)]
pub(crate) use AtaVerifyOutBuffer;
// drivers/storage/ide/atapi/scsiex.h:117: header-inline `CdbGetAllocationLength16`
#[allow(dead_code)]
#[inline]
pub unsafe fn CdbGetAllocationLength16(Cdb: PCDB) -> ULONG {
    (((*Cdb).CDB16.TransferLength[0] as ULONG) << 24) |
    (((*Cdb).CDB16.TransferLength[1] as ULONG) << 16) |
    (((*Cdb).CDB16.TransferLength[2] as ULONG) << 8) |
    (((*Cdb).CDB16.TransferLength[3] as ULONG) << 0)
}
// drivers/storage/ide/atapi/scsiex.h:172: header-inline `CdbGetLogicalBlockAddress10`
#[allow(dead_code)]
#[inline]
pub unsafe fn CdbGetLogicalBlockAddress10(Cdb: PCDB) -> ULONG {
    (((*Cdb).CDB10.LogicalBlockByte0 as ULONG) << 24) |
    (((*Cdb).CDB10.LogicalBlockByte1 as ULONG) << 16) |
    (((*Cdb).CDB10.LogicalBlockByte2 as ULONG) << 8) |
    (((*Cdb).CDB10.LogicalBlockByte3 as ULONG) << 0)
}
// drivers/storage/ide/atapi/scsiex.h:184: header-inline `CdbGetLogicalBlockAddress12`
#[allow(dead_code)]
#[inline]
pub unsafe fn CdbGetLogicalBlockAddress12(Cdb: PCDB) -> ULONG {
    (((*Cdb).CDB12.LogicalBlock[0] as ULONG) << 24) |
    (((*Cdb).CDB12.LogicalBlock[1] as ULONG) << 16) |
    (((*Cdb).CDB12.LogicalBlock[2] as ULONG) << 8) |
    (((*Cdb).CDB12.LogicalBlock[3] as ULONG) << 0)
}
// drivers/storage/ide/atapi/scsiex.h:196: header-inline `CdbGetLogicalBlockAddress16`
#[allow(dead_code)]
#[inline]
pub unsafe fn CdbGetLogicalBlockAddress16(Cdb: PCDB) -> ULONG64 {
    (((*Cdb).CDB16.LogicalBlock[0] as ULONG64) << 56) |
    (((*Cdb).CDB16.LogicalBlock[1] as ULONG64) << 48) |
    (((*Cdb).CDB16.LogicalBlock[2] as ULONG64) << 40) |
    (((*Cdb).CDB16.LogicalBlock[3] as ULONG64) << 32) |
    (((*Cdb).CDB16.LogicalBlock[4] as ULONG64) << 24) |
    (((*Cdb).CDB16.LogicalBlock[5] as ULONG64) << 16) |
    (((*Cdb).CDB16.LogicalBlock[6] as ULONG64) << 8) |
    (((*Cdb).CDB16.LogicalBlock[7] as ULONG64) << 0)
}
// drivers/storage/ide/atapi/scsiex.h:162: header-inline `CdbGetLogicalBlockAddress6`
#[allow(dead_code)]
#[inline]
pub unsafe fn CdbGetLogicalBlockAddress6(Cdb: PCDB) -> ULONG {
    (((*Cdb).CDB6READWRITE.LogicalBlockMsb0 as ULONG) << 8) |
    (((*Cdb).CDB6READWRITE.LogicalBlockLsb as ULONG) << 0)
}
// drivers/storage/ide/atapi/scsiex.h:128: header-inline `CdbGetTransferLength10`
#[allow(dead_code)]
#[inline]
pub unsafe fn CdbGetTransferLength10(Cdb: PCDB) -> USHORT {
    (((*Cdb).CDB10.TransferBlocksMsb as USHORT) << 8) |
    (((*Cdb).CDB10.TransferBlocksLsb as USHORT) << 0)
}
// drivers/storage/ide/atapi/scsiex.h:138: header-inline `CdbGetTransferLength12`
#[allow(dead_code)]
#[inline]
pub unsafe fn CdbGetTransferLength12(Cdb: PCDB) -> ULONG {
    (((*Cdb).CDB12.TransferLength[0] as ULONG) << 24) |
    (((*Cdb).CDB12.TransferLength[1] as ULONG) << 16) |
    (((*Cdb).CDB12.TransferLength[2] as ULONG) << 8) |
    (((*Cdb).CDB12.TransferLength[3] as ULONG) << 0)
}
// drivers/storage/ide/atapi/scsiex.h:150: header-inline `CdbGetTransferLength16`
#[allow(dead_code)]
#[inline]
pub unsafe fn CdbGetTransferLength16(Cdb: PCDB) -> ULONG {
    (((*Cdb).CDB16.TransferLength[0] as ULONG) << 24) |
    (((*Cdb).CDB16.TransferLength[1] as ULONG) << 16) |
    (((*Cdb).CDB16.TransferLength[2] as ULONG) << 8) |
    (((*Cdb).CDB16.TransferLength[3] as ULONG) << 0)
}
// drivers/storage/ide/atapi/atapi.h:155: #define DECLARE_PAGED_UNICODE_STRING(Variable, Str) static const ATAPORT_PAGED_DATA WCHAR Variable##_buffer[] = Str; UNICODE_STRING Variable = { sizeof(Str) - sizeof(WCHAR), sizeof(Str), (PWCH)Variable##_buffer }
#[allow(unused_macros)]
macro_rules! DECLARE_PAGED_UNICODE_STRING {
    // atapi.h declares a `static const` UNICODE_STRING in PAGECONS. A Rust `static` of it
    // cannot compile (its Buffer is a raw pointer, so it is not Sync), and the macro is used
    // in function scope to declare a local, so it binds one: the string data itself stays a
    // static, which is all the C's section placement was for.
    ($Variable:ident, $Str:expr $(,)?) => {
        let $Variable: UNICODE_STRING = UNICODE_STRING {
            Length: (core::mem::size_of_val(&$Str) - core::mem::size_of::<WCHAR>()) as USHORT,
            MaximumLength: core::mem::size_of_val(&$Str) as USHORT,
            Buffer: $Str.as_ptr() as PWCH,
        };
    };
}
#[allow(unused_imports)]
pub(crate) use DECLARE_PAGED_UNICODE_STRING;
// drivers/storage/ide/atapi/atapi.h:162: #define DECLARE_PAGED_WSTRING(v, n) static const ATAPORT_PAGED_DATA WCHAR (v)[] = (n)
#[allow(unused_macros)]
macro_rules! DECLARE_PAGED_WSTRING {
    ($v:ident, $n:expr $(,)?) => {
        static $v: [WCHAR; core::mem::size_of_val(&$n) / core::mem::size_of::<WCHAR>()] = $n;
    };
}
#[allow(unused_imports)]
pub(crate) use DECLARE_PAGED_WSTRING;
// drivers/storage/ide/atapi/atapi.h:313: #define IRP_FROM_QUEUE_ENTRY(QueueEntry) (PIRP)CONTAINING_RECORD(QueueEntry, IRP, Tail.Overlay.DriverContext[0])
#[allow(unused_macros)]
macro_rules! IRP_FROM_QUEUE_ENTRY {
    ($QueueEntry:expr $(,)?) => {
        (CONTAINING_RECORD!($QueueEntry, IRP, Tail.Overlay.__bindgen_anon_1.__bindgen_anon_1.DriverContext[0]) as PIRP)
    };
}
#[allow(unused_imports)]
pub(crate) use IRP_FROM_QUEUE_ENTRY;
// drivers/storage/ide/atapi/atapi.h:176: #define IS_ATAPI(Device) (((Device)->TransportFlags & DEVICE_IS_ATAPI) != 0)
#[allow(unused_macros)]
macro_rules! IS_ATAPI {
    ($Device:expr $(,)?) => {
        (((*($Device)).TransportFlags & DEVICE_IS_ATAPI) != 0)
    };
}
#[allow(unused_imports)]
pub(crate) use IS_ATAPI;
// drivers/storage/ide/atapi/atapi.h:173: #define IS_FDO(p) ((((PATAPORT_COMMON_EXTENSION)(p))->Flags & DO_IS_FDO) != 0)
#[allow(unused_macros)]
macro_rules! IS_FDO {
    ($p:expr $(,)?) => {
        (((*($p as PATAPORT_COMMON_EXTENSION)).Flags & DO_IS_FDO) != 0)
    };
}
#[allow(unused_imports)]
pub(crate) use IS_FDO;
// drivers/storage/ide/atapi/atapi.h:179: #define NUM_TO_BITMAP(num) (0xFFFFFFFF >> (RTL_BITS_OF(ULONG) - (num)))
#[allow(unused_macros)]
macro_rules! NUM_TO_BITMAP {
    ($num:expr $(,)?) => {
        (0xFFFFFFFFu32 >> (RTL_BITS_OF!(ULONG) - ($num)))
    };
}
#[allow(unused_imports)]
pub(crate) use NUM_TO_BITMAP;
// drivers/storage/ide/atapi/atapi.h:310: #define QUEUE_ENTRY_FROM_IRP(Irp) ((PREQUEST_QUEUE_ENTRY)&(((PIRP)(Irp))->Tail.Overlay.DriverContext[0]))
#[allow(unused_macros)]
macro_rules! QUEUE_ENTRY_FROM_IRP {
    ($Irp:expr $(,)?) => {
        (core::ptr::addr_of!((*($Irp as PIRP)).Tail.Overlay.__bindgen_anon_1.__bindgen_anon_1.DriverContext[0]) as PREQUEST_QUEUE_ENTRY)
    };
}
#[allow(unused_imports)]
pub(crate) use QUEUE_ENTRY_FROM_IRP;
// drivers/storage/ide/atapi/atapi.h:235: #define SRB_CLEAR_FLAGS(Srb, Flags) ((Srb)->SrbExtension = (PVOID)((ULONG_PTR)(Srb)->SrbExtension & ~(Flags)))
#[allow(unused_macros)]
macro_rules! SRB_CLEAR_FLAGS {
    ($Srb:expr, $Flags:expr $(,)?) => {
        {
            (*($Srb)).SrbExtension = ((((*($Srb)).SrbExtension as ULONG_PTR) & !($Flags)) as PVOID);
            (*($Srb)).SrbExtension
        }
    };
}
#[allow(unused_imports)]
pub(crate) use SRB_CLEAR_FLAGS;
// drivers/storage/ide/atapi/atapi.h:238: #define SRB_GET_FLAGS(Srb) ((ULONG_PTR)(Srb)->SrbExtension)
#[allow(unused_macros)]
macro_rules! SRB_GET_FLAGS {
    ($Srb:expr $(,)?) => {
        ((*($Srb)).SrbExtension as ULONG_PTR)
    };
}
#[allow(unused_imports)]
pub(crate) use SRB_GET_FLAGS;
// drivers/storage/ide/atapi/atapi.h:232: #define SRB_SET_FLAGS(Srb, Flags) ((Srb)->SrbExtension = (PVOID)((ULONG_PTR)(Srb)->SrbExtension | (Flags)))
#[allow(unused_macros)]
macro_rules! SRB_SET_FLAGS {
    ($Srb:expr, $Flags:expr $(,)?) => {
        {
            (*($Srb)).SrbExtension = ((((*($Srb)).SrbExtension as ULONG_PTR) | ($Flags)) as PVOID);
            (*($Srb)).SrbExtension
        }
    };
}
#[allow(unused_imports)]
pub(crate) use SRB_SET_FLAGS;
// ==== end port macros ====
