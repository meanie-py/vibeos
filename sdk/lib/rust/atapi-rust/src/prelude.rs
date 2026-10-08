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
// reactos/debug.h DPRINT/UNIMPLEMENTED — no-op MACROS, never out-of-line fns (field manual
// §9.65: an out-of-line no-op is an optimization barrier). debug_macros.py rewrites C's bare
// `UNIMPLEMENTED;` to `UNIMPLEMENTED!()`, so the prelude must have it. DPRINT1 PRINTS in this
// DBG=1 tree: it is win32macros-rust's.
#[macro_export] macro_rules! DPRINT  { ($($arg:tt)*) => {{}}; }
#[macro_export] macro_rules! UNIMPLEMENTED { ($($arg:tt)*) => {{}}; }
#[macro_export] macro_rules! UNIMPLEMENTED_ONCE { ($($arg:tt)*) => {{}}; }
pub use crate::{DPRINT, UNIMPLEMENTED, UNIMPLEMENTED_ONCE};
pub use win32macros_rust::DPRINT1;
// The module's own debug.h channels (ERR/WARN/INFO/TRACE, printf-shaped). Macros, never
// out-of-line fns, for the same reason as DPRINT above. debug.h defines DEBUG_WARN and
// DEBUG_ERR under DBG, so in this DBG=1 tree ERR and WARN print, as DPRINT1's "(file:line) "
// format; INFO and TRACE are compiled out upstream too.
#[macro_export] macro_rules! ERR { ($($arg:tt)*) => { ::win32macros_rust::DPRINT1!($($arg)*) }; }
#[macro_export] macro_rules! WARN { ($($arg:tt)*) => { ::win32macros_rust::DPRINT1!($($arg)*) }; }
#[macro_export] macro_rules! INFO { ($($arg:tt)*) => {{}}; }
#[macro_export] macro_rules! TRACE { ($($arg:tt)*) => {{}}; }
pub use crate::{ERR, INFO, TRACE, WARN};

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
    crate::types::IoAcquireRemoveLockEx(
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
    crate::types::IoReleaseRemoveLockEx(RemoveLock, Tag, core::mem::size_of::<IO_REMOVE_LOCK>() as ULONG)
}
/// wdm.h: `IoReleaseRemoveLockAndWaitEx(RemoveLock, Tag, sizeof(IO_REMOVE_LOCK))`.
#[inline]
pub unsafe fn IoReleaseRemoveLockAndWait(RemoveLock: PIO_REMOVE_LOCK, Tag: PVOID) {
    crate::types::IoReleaseRemoveLockAndWaitEx(RemoveLock, Tag, core::mem::size_of::<IO_REMOVE_LOCK>() as ULONG)
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
pub const APC_LEVEL: u32 = 1; // legide-rust/src/prelude.rs
pub const ATA_CHANNEL_FLAG_IS_AHCI: u32 = 8; // legide-rust/src/types_amd64.rs
pub const ATA_CHANNEL_FLAG_IS_EXTERNAL: u32 = 2; // legide-rust/src/types_amd64.rs
pub const ATA_CHANNEL_FLAG_NCQ: u32 = 4; // legide-rust/src/types_amd64.rs
pub const ATA_CHANNEL_FLAG_PIO_FOR_LBA48_XFER: u32 = 16; // legide-rust/src/types_amd64.rs
pub const ATA_CHANNEL_FLAG_PIO_VIA_DMA: u32 = 1; // legide-rust/src/types_amd64.rs
pub const ATA_MAX_DEVICE: u32 = 15; // legide-rust/src/types_amd64.rs
pub const ATA_MAX_LBA_28: ULONG64 = 0x0FFFFFFF; // freeldr-rust/src/hwide.rs
pub const ATA_MAX_SECTORS_PER_IO: u32 = 256; // legide-rust/src/types_amd64.rs
pub const ATA_MIN_BUFFER_ALIGNMENT: u32 = 1; // legide-rust/src/types_amd64.rs
pub const ATA_MIN_SECTOR_SIZE: u32 = 512; // legide-rust/src/types_amd64.rs
pub const BusQueryCompatibleIDs: _BUS_QUERY_ID_TYPE = _BUS_QUERY_ID_TYPE_BusQueryCompatibleIDs; // island enum
pub const BusQueryDeviceID: _BUS_QUERY_ID_TYPE = _BUS_QUERY_ID_TYPE_BusQueryDeviceID; // island enum
pub const BusQueryHardwareIDs: _BUS_QUERY_ID_TYPE = _BUS_QUERY_ID_TYPE_BusQueryHardwareIDs; // island enum
pub const BusQueryInstanceID: _BUS_QUERY_ID_TYPE = _BUS_QUERY_ID_TYPE_BusQueryInstanceID; // island enum
pub const BusRelations: _DEVICE_RELATION_TYPE = _DEVICE_RELATION_TYPE_BusRelations; // island enum
pub const BusTypeAta: _STORAGE_BUS_TYPE = _STORAGE_BUS_TYPE_BusTypeAta; // island enum
pub const BusTypeSata: _STORAGE_BUS_TYPE = _STORAGE_BUS_TYPE_BusTypeSata; // island enum
pub const CAP_ATAPI_ID_CMD: u32 = 2; // cdrom-rust/src/types_amd64.rs
pub const CAP_ATA_ID_CMD: u32 = 1; // cdrom-rust/src/types_amd64.rs
pub const CAP_SMART_CMD: u32 = 4; // cdrom-rust/src/types_amd64.rs
pub const COMPLETE_IRP: ATA_COMPLETION_ACTION = ATA_COMPLETION_ACTION_COMPLETE_IRP; // island enum
pub const COMPLETE_NO_IRP: ATA_COMPLETION_ACTION = ATA_COMPLETION_ACTION_COMPLETE_NO_IRP; // island enum
pub const CONN_STATUS_DEV_ATAPI: _ATA_CONNECTION_STATUS = _ATA_CONNECTION_STATUS_CONN_STATUS_DEV_ATAPI; // island enum
pub const CONN_STATUS_FAILURE: _ATA_CONNECTION_STATUS = _ATA_CONNECTION_STATUS_CONN_STATUS_FAILURE; // island enum
pub const CONN_STATUS_NO_DEVICE: _ATA_CONNECTION_STATUS = _ATA_CONNECTION_STATUS_CONN_STATUS_NO_DEVICE; // island enum
pub const CriticalWorkQueue: _WORK_QUEUE_TYPE = _WORK_QUEUE_TYPE_CriticalWorkQueue; // island enum
pub const DEVICE_HAS_CDB_INTERRUPT: u32 = 32; // legide-rust/src/types_amd64.rs
pub const DEVICE_IS_ATAPI: u32 = 16; // legide-rust/src/types_amd64.rs
pub const DEVICE_IS_NEC_CDR260: u32 = 128; // legide-rust/src/types_amd64.rs
pub const DEVICE_NEED_DMA_DIRECTION: u32 = 64; // legide-rust/src/types_amd64.rs
pub const DEVICE_QUEUE_DEPTH_MASK: u32 = 65280; // legide-rust/src/types_amd64.rs
pub const DEVICE_QUEUE_DEPTH_SHIFT: u32 = 8; // legide-rust/src/types_amd64.rs
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
pub const DO_IS_FDO: u32 = 2147483648; // pciidex-rust/src/prelude.rs
pub const DO_POWER_INRUSH: u32 = 16384; // ntkrnl-sys/src/hal_bindgen_amd64.rs
pub const DevicePowerState: _POWER_STATE_TYPE = _POWER_STATE_TYPE_DevicePowerState; // island enum
pub const DeviceTextDescription: _DEVICE_TEXT_TYPE = _DEVICE_TEXT_TYPE_DeviceTextDescription; // island enum
pub const DeviceTextLocationInformation: _DEVICE_TEXT_TYPE = _DEVICE_TEXT_TYPE_DeviceTextLocationInformation; // island enum
pub const DeviceUsageTypeDumpFile: _DEVICE_USAGE_NOTIFICATION_TYPE = _DEVICE_USAGE_NOTIFICATION_TYPE_DeviceUsageTypeDumpFile; // island enum
pub const DeviceUsageTypeHibernation: _DEVICE_USAGE_NOTIFICATION_TYPE = _DEVICE_USAGE_NOTIFICATION_TYPE_DeviceUsageTypeHibernation; // island enum
pub const DeviceUsageTypePaging: _DEVICE_USAGE_NOTIFICATION_TYPE = _DEVICE_USAGE_NOTIFICATION_TYPE_DeviceUsageTypePaging; // island enum
pub const EVENT_CODE_BAD_STATE: _ATA_ERROR_LOG_VALUE = _ATA_ERROR_LOG_VALUE_EVENT_CODE_BAD_STATE; // island enum
pub const EVENT_CODE_CRC_ERROR: _ATA_ERROR_LOG_VALUE = _ATA_ERROR_LOG_VALUE_EVENT_CODE_CRC_ERROR; // island enum
pub const EVENT_CODE_DMA_DISABLE: _ATA_ERROR_LOG_VALUE = _ATA_ERROR_LOG_VALUE_EVENT_CODE_DMA_DISABLE; // island enum
pub const EVENT_CODE_DOWNSHIFT: _ATA_ERROR_LOG_VALUE = _ATA_ERROR_LOG_VALUE_EVENT_CODE_DOWNSHIFT; // island enum
pub const EVENT_CODE_NCQ_DISABLE: _ATA_ERROR_LOG_VALUE = _ATA_ERROR_LOG_VALUE_EVENT_CODE_NCQ_DISABLE; // island enum
pub const EVENT_CODE_TIMEOUT: _ATA_ERROR_LOG_VALUE = _ATA_ERROR_LOG_VALUE_EVENT_CODE_TIMEOUT; // island enum
pub const FILE_DEVICE_CONTROLLER: u32 = 4; // diskpart-rust/src/types_amd64.rs
pub const FILE_DEVICE_MASS_STORAGE: u32 = 45; // diskpart-rust/src/types_amd64.rs
pub const FILE_DEVICE_SECURE_OPEN: u32 = 256; // ntdll-sys/src/bindgen_amd64.rs
pub const FILE_REMOVABLE_MEDIA: u32 = 1; // diskpart-rust/src/prelude.rs
pub const GetDeviceType: DEVICE_TYPE_NAME = DEVICE_TYPE_NAME_GetDeviceType; // island enum
pub const GetGenericType: DEVICE_TYPE_NAME = DEVICE_TYPE_NAME_GetGenericType; // island enum
pub const GetPeripheralId: DEVICE_TYPE_NAME = DEVICE_TYPE_NAME_GetPeripheralId; // island enum
pub const HighPagePriority: ULONG = 2; // ntoskrnl-rust/src/ARM3/helpers.rs
pub const IDENTIFY_BUFFER_SIZE: u32 = 512; // cdrom-rust/src/types_amd64.rs
pub const IDE_ACPI_TIMING_MODE_NOT_SUPPORTED: u32 = 4294967295; // legide-rust/src/types_amd64.rs
pub const IDE_COMMAND_ATAPI_IDENTIFY: u32 = 161; // atapi-legacy-rust/src/types_amd64.rs
pub const IDE_COMMAND_DOOR_LOCK: u32 = 222; // legide-rust/src/types_amd64.rs
pub const IDE_COMMAND_DOOR_UNLOCK: u32 = 223; // legide-rust/src/types_amd64.rs
pub const IDE_COMMAND_FLUSH_CACHE: u32 = 231; // legide-rust/src/types_amd64.rs
pub const IDE_COMMAND_FLUSH_CACHE_EXT: u32 = 234; // legide-rust/src/types_amd64.rs
pub const IDE_COMMAND_IDENTIFY: u32 = 236; // atapi-legacy-rust/src/types_amd64.rs
pub const IDE_COMMAND_IDLE_IMMEDIATE: u32 = 225; // legide-rust/src/types_amd64.rs
pub const IDE_COMMAND_MEDIA_EJECT: u32 = 237; // atapi-legacy-rust/src/types_amd64.rs
pub const IDE_COMMAND_READ: u32 = 32; // atapi-legacy-rust/src/types_amd64.rs
pub const IDE_COMMAND_READ_DMA: u32 = 200; // atapi-legacy-rust/src/types_amd64.rs
pub const IDE_COMMAND_READ_DMA_EXT: u32 = 37; // legide-rust/src/types_amd64.rs
pub const IDE_COMMAND_READ_EXT: u32 = 36; // legide-rust/src/types_amd64.rs
pub const IDE_COMMAND_READ_FPDMA_QUEUED: u32 = 96; // legide-rust/src/types_amd64.rs
pub const IDE_COMMAND_READ_LOG_EXT: u32 = 47; // legide-rust/src/types_amd64.rs
pub const IDE_COMMAND_READ_MULTIPLE: u32 = 196; // atapi-legacy-rust/src/types_amd64.rs
pub const IDE_COMMAND_READ_MULTIPLE_EXT: u32 = 41; // legide-rust/src/types_amd64.rs
pub const IDE_COMMAND_SECURITY_FREEZE_LOCK: u32 = 245; // legide-rust/src/types_amd64.rs
pub const IDE_COMMAND_SET_DRIVE_PARAMETERS: u32 = 145; // atapi-legacy-rust/src/types_amd64.rs
pub const IDE_COMMAND_SET_FEATURE: u32 = 239; // legide-rust/src/types_amd64.rs
pub const IDE_COMMAND_SET_MULTIPLE: u32 = 198; // atapi-legacy-rust/src/types_amd64.rs
pub const IDE_COMMAND_SMART: u32 = 176; // legide-rust/src/types_amd64.rs
pub const IDE_COMMAND_STANDBY_IMMEDIATE: u32 = 224; // legide-rust/src/types_amd64.rs
pub const IDE_COMMAND_VERIFY: u32 = 64; // atapi-legacy-rust/src/types_amd64.rs
pub const IDE_COMMAND_VERIFY_EXT: u32 = 66; // legide-rust/src/types_amd64.rs
pub const IDE_COMMAND_WRITE: u32 = 48; // atapi-legacy-rust/src/types_amd64.rs
pub const IDE_COMMAND_WRITE_DMA: u32 = 202; // atapi-legacy-rust/src/types_amd64.rs
pub const IDE_COMMAND_WRITE_DMA_EXT: u32 = 53; // legide-rust/src/types_amd64.rs
pub const IDE_COMMAND_WRITE_DMA_FUA_EXT: u32 = 61; // legide-rust/src/types_amd64.rs
pub const IDE_COMMAND_WRITE_EXT: u32 = 52; // legide-rust/src/types_amd64.rs
pub const IDE_COMMAND_WRITE_FPDMA_QUEUED: u32 = 97; // legide-rust/src/types_amd64.rs
pub const IDE_COMMAND_WRITE_MULTIPLE: u32 = 197; // atapi-legacy-rust/src/types_amd64.rs
pub const IDE_COMMAND_WRITE_MULTIPLE_EXT: u32 = 57; // legide-rust/src/types_amd64.rs
pub const IDE_COMMAND_WRITE_MULTIPLE_FUA_EXT: u32 = 206; // legide-rust/src/types_amd64.rs
pub const IDE_DRIVE_SELECT: u32 = 160; // pciidex-rust/src/prelude.rs
pub const IDE_ERROR_ADDRESS_NOT_FOUND: u32 = 1; // legide-rust/src/types_amd64.rs
pub const IDE_ERROR_COMMAND_ABORTED: u32 = 4; // atapi-legacy-rust/src/types_amd64.rs
pub const IDE_ERROR_CRC_ERROR: u32 = 128; // legide-rust/src/types_amd64.rs
pub const IDE_ERROR_DATA_ERROR: u32 = 64; // atapi-legacy-rust/src/types_amd64.rs
pub const IDE_ERROR_END_OF_MEDIA: u32 = 2; // atapi-legacy-rust/src/types_amd64.rs
pub const IDE_ERROR_ID_NOT_FOUND: u32 = 16; // atapi-legacy-rust/src/types_amd64.rs
pub const IDE_ERROR_MEDIA_CHANGE: u32 = 32; // atapi-legacy-rust/src/types_amd64.rs
pub const IDE_ERROR_MEDIA_CHANGE_REQ: u32 = 8; // atapi-legacy-rust/src/types_amd64.rs
pub const IDE_FEATURE_DISABLE_REVERT_TO_POWER_ON: u32 = 102; // legide-rust/src/types_amd64.rs
pub const IDE_FEATURE_ENABLE_MSN: u32 = 149; // legide-rust/src/types_amd64.rs
pub const IDE_FEATURE_PUIS_SPIN_UP: u32 = 7; // legide-rust/src/types_amd64.rs
pub const IDE_FEATURE_SET_TRANSFER_MODE: u32 = 3; // legide-rust/src/types_amd64.rs
pub const IDE_GP_LOG_NCQ_COMMAND_ERROR_ADDRESS: u32 = 16; // legide-rust/src/types_amd64.rs
pub const IDE_GP_LOG_SECTOR_SIZE: u32 = 512; // legide-rust/src/types_amd64.rs
pub const IDE_LBA_MODE: u32 = 64; // legide-rust/src/types_amd64.rs
pub const IDE_STATUS_DEVICE_FAULT: u32 = 32; // legide-rust/src/prelude.rs
pub const IDE_STATUS_ERROR: u32 = 1; // atapi-legacy-rust/src/types_amd64.rs
pub const IDE_STATUS_INDEX: u32 = 2; // atapi-legacy-rust/src/types_amd64.rs
pub const ID_CMD: u32 = 236; // cdrom-rust/src/types_amd64.rs
pub const INQUIRYDATABUFFERSIZE: u32 = 36; // atapi-legacy-rust/src/types_amd64.rs
pub const IOCTL_SCSI_GET_ADDRESS: u32 = 0x00041018; // classpnp-rust/src/glue.rs
pub const IOCTL_SCSI_GET_CAPABILITIES: u32 = 0x0004_1010; // scsidisk-rust/src/lib.rs
pub const IOCTL_STORAGE_EVENT_NOTIFICATION: u32 = 0x002D1C90; // classpnp-rust/src/glue.rs
pub const IOCTL_STORAGE_MANAGE_DATA_SET_ATTRIBUTES: u32 = 0x002D9404; // classpnp-rust/src/glue.rs
pub const IOCTL_STORAGE_QUERY_PROPERTY: u32 = 0x002D_1400; // diskpart-rust/src/prelude.rs
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
pub const IrpForward: _SYSCTL_IRP_DISPOSITION = _SYSCTL_IRP_DISPOSITION_IrpForward; // island enum
pub const IrpNotCompleted: _SYSCTL_IRP_DISPOSITION = _SYSCTL_IRP_DISPOSITION_IrpNotCompleted; // island enum
pub const IrpNotWmi: _SYSCTL_IRP_DISPOSITION = _SYSCTL_IRP_DISPOSITION_IrpNotWmi; // island enum
pub const IrpProcessed: _SYSCTL_IRP_DISPOSITION = _SYSCTL_IRP_DISPOSITION_IrpProcessed; // island enum
pub const IsNEC_98: u32 = 0; // legide-rust/src/prelude.rs
pub const KEY_ALL_ACCESS: u32 = 983103; // legide-rust/src/prelude.rs
pub const KEY_READ: u32 = 131097; // ntdll-sys/src/bindgen_amd64.rs
pub const KeepObject: _IO_ALLOCATION_ACTION = _IO_ALLOCATION_ACTION_KeepObject; // island enum
pub const KeyValuePartialInformation: u32 = 2; // config-rust/src/cmsysini.rs
pub const MAXULONG: u32 = 4294967295; // legide-rust/src/types_amd64.rs
pub const MAX_IDE_DEVICE: u32 = 2; // legide-rust/src/prelude.rs
pub const MAX_SLOTS: usize = 16; // usbxhci-rust/src/controller.rs
pub const MODE_DSP_FUA_SUPPORTED: u32 = 16; // atapi-legacy-rust/src/types_amd64.rs
pub const MODE_DSP_WRITE_PROTECT: u32 = 128; // atapi-legacy-rust/src/types_amd64.rs
pub const MODE_PAGE_CACHING: u32 = 8; // atapi-legacy-rust/src/types_amd64.rs
pub const MODE_PAGE_CONTROL: u32 = 10; // atapi-legacy-rust/src/types_amd64.rs
pub const MODE_PAGE_ERROR_RECOVERY: u32 = 1; // atapi-legacy-rust/src/types_amd64.rs
pub const MODE_PAGE_FAULT_REPORTING: u32 = 28; // atapi-legacy-rust/src/types_amd64.rs
pub const MODE_SENSE_RETURN_ALL: u32 = 63; // atapi-legacy-rust/src/types_amd64.rs
pub const MODE_SENSE_SAVED_VALUES: u32 = 192; // atapi-legacy-rust/src/types_amd64.rs
pub const MWDMA_ALL: u32 = 1792; // pciidex-rust/src/prelude.rs
pub const MmCached: _MEMORY_CACHING_TYPE = _MEMORY_CACHING_TYPE_MmCached; // island enum
pub const MmNonCached: MEMORY_CACHING_TYPE = 0; // legide-rust/src/prelude.rs
pub const NonPagedPool: POOL_TYPE = 0; // msfs-rust/src/glue.rs
pub const OBJ_CASE_INSENSITIVE: u32 = 64; // diskpart-rust/src/prelude.rs
pub const OBJ_KERNEL_HANDLE: u32 = 512; // legide-rust/src/prelude.rs
pub const OBJ_PERMANENT: u32 = 16; // legide-rust/src/types_amd64.rs
pub const PAGE_SIZE: u32 = 4096; // ntdll-sys/src/bindgen_amd64.rs
pub const PCIIDEX_INTERFACE_VERSION: u32 = 1; // legide-rust/src/prelude.rs
pub const PIO_ALL: u32 = 31; // pciidex-rust/src/prelude.rs
pub const PIO_MODE0: u32 = 1; // legide-rust/src/types_amd64.rs
pub const PIO_MODE1: u32 = 2; // legide-rust/src/types_amd64.rs
pub const PIO_MODE2: u32 = 4; // legide-rust/src/types_amd64.rs
pub const PLUGPLAY_REGKEY_DEVICE: u32 = 1; // classpnp-rust/src/glue.rs
pub const PNP_DEVICE_NOT_DISABLEABLE: u32 = 32; // ntkrnl-sys/src/hal_bindgen_amd64.rs
pub const PagedPool: POOL_TYPE = 1; // ks-rust/src/image.rs
pub const PowerDeviceD0: usize = 1; // kbdhid-rust/src/glue.rs
pub const PowerDeviceD3: _DEVICE_POWER_STATE = _DEVICE_POWER_STATE_PowerDeviceD3; // island enum
pub const PowerSystemUnspecified: _SYSTEM_POWER_STATE = _SYSTEM_POWER_STATE_PowerSystemUnspecified; // island enum
pub const PowerSystemWorking: _SYSTEM_POWER_STATE = _SYSTEM_POWER_STATE_PowerSystemWorking; // island enum
pub const PropertyExistsQuery: u32 = 1; // scsiport-rust/src/glue.rs
pub const PropertyStandardQuery: u32 = 0; // scsiport-rust/src/glue.rs
pub const READ_ATTRIBUTE_BUFFER_SIZE: u32 = 512; // cdrom-rust/src/types_amd64.rs
pub const READ_ONLY_DIRECT_ACCESS_DEVICE: u32 = 5; // atapi-legacy-rust/src/types_amd64.rs
pub const READ_THRESHOLD_BUFFER_SIZE: u32 = 512; // cdrom-rust/src/types_amd64.rs
pub const REG_DWORD: u32 = 4; // advapi32-sys/src/bindgen_amd64.rs
pub const REG_OPTION_VOLATILE: u32 = 1; // advapi32-sys/src/bindgen_amd64.rs
pub const REQUEST_DMA_FLAGS: u32 = 2097153; // legide-rust/src/types_amd64.rs
pub const REQUEST_EXCLUSIVE_ACCESS_FLAGS: u32 = 16896; // legide-rust/src/types_amd64.rs
pub const REQUEST_FLAG_DATA_IN: u32 = 64; // legide-rust/src/types_amd64.rs
pub const REQUEST_FLAG_DATA_OUT: u32 = 128; // legide-rust/src/types_amd64.rs
pub const REQUEST_FLAG_DMA: u32 = 1; // legide-rust/src/types_amd64.rs
pub const REQUEST_FLAG_EXCLUSIVE: u32 = 16384; // legide-rust/src/types_amd64.rs
pub const REQUEST_FLAG_FUA: u32 = 256; // legide-rust/src/types_amd64.rs
pub const REQUEST_FLAG_HAS_LOCAL_BUFFER: u32 = 32768; // legide-rust/src/types_amd64.rs
pub const REQUEST_FLAG_HAS_MDL: u32 = 131072; // legide-rust/src/types_amd64.rs
pub const REQUEST_FLAG_HAS_RESERVED_MAPPING: u32 = 524288; // legide-rust/src/types_amd64.rs
pub const REQUEST_FLAG_HAS_SG_LIST: u32 = 65536; // legide-rust/src/types_amd64.rs
pub const REQUEST_FLAG_HAS_TASK_FILE: u32 = 1024; // legide-rust/src/types_amd64.rs
pub const REQUEST_FLAG_INTERNAL: u32 = 4194304; // legide-rust/src/types_amd64.rs
pub const REQUEST_FLAG_NCQ: u32 = 4; // legide-rust/src/types_amd64.rs
pub const REQUEST_FLAG_NO_KEEP_AWAKE: u32 = 1048576; // legide-rust/src/types_amd64.rs
pub const REQUEST_FLAG_PASSTHROUGH: u32 = 8388608; // legide-rust/src/types_amd64.rs
pub const REQUEST_FLAG_POLL: u32 = 2147483648; // legide-rust/src/types_amd64.rs
pub const REQUEST_FLAG_PROGRAM_DMA: u32 = 2097152; // legide-rust/src/types_amd64.rs
pub const REQUEST_FLAG_READ_WRITE: u32 = 32; // legide-rust/src/types_amd64.rs
pub const REQUEST_FLAG_SAVE_TASK_FILE: u32 = 512; // legide-rust/src/types_amd64.rs
pub const REQUEST_STATE_FREEZE_QUEUE: u32 = 3; // legide-rust/src/types_amd64.rs
pub const REQUEST_STATE_NOT_STARTED: u32 = 1; // legide-rust/src/types_amd64.rs
pub const REQUEST_STATE_REQUEUE: u32 = 2; // legide-rust/src/types_amd64.rs
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
pub const STATUS_BUFFER_TOO_SMALL: i32 = 0xC000_0023u32 as i32; // diskpart-rust/src/prelude.rs
pub const STATUS_CANCELLED: NTSTATUS = 0xC000_0120u32 as NTSTATUS; // legide-rust/src/prelude.rs
pub const STATUS_CONTINUE_COMPLETION: NTSTATUS = 0x00000000_u32 as i32; // hidusb-rust/src/glue.rs
pub const STATUS_DEVICE_BUSY: i32 = 0x80000011u32 as i32; // classpnp-rust/src/glue.rs
pub const STATUS_DEVICE_DOES_NOT_EXIST: i32 = 0xC00000C0u32 as i32; // classpnp-rust/src/glue.rs
pub const STATUS_DEVICE_NOT_CONNECTED: i32 = 0xC000009D_u32 as i32; // ntdll-sys/src/constants.rs
pub const STATUS_DEVICE_REMOVED: i32 = 0xC00002B6u32 as i32; // partmgr-rust/src/glue.rs
pub const STATUS_INFO_LENGTH_MISMATCH: i32 = 0xC0000004u32 as i32; // kernel32-rust/src/client/sysinfo.rs
pub const STATUS_INSUFFICIENT_RESOURCES: NTSTATUS = 0xC000_009Au32 as NTSTATUS; // legide-rust/src/prelude.rs
pub const STATUS_INVALID_DEVICE_REQUEST: NTSTATUS = 0xC000_0010u32 as NTSTATUS; // mup-rust/src/glue.rs
pub const STATUS_INVALID_PARAMETER: i32 = 0xC000_000Du32 as i32; // network/src/lib.rs
pub const STATUS_IO_DEVICE_ERROR: i32 = 0xC0000185u32 as i32; // classpnp-rust/src/glue.rs
pub const STATUS_IO_TIMEOUT: i32 = 0xC00000B5u32 as i32; // classpnp-rust/src/glue.rs
pub const STATUS_MORE_PROCESSING_REQUIRED: i32 = 0xC0000016u32 as i32; // classpnp-rust/src/glue.rs
pub const STATUS_NOT_IMPLEMENTED: NTSTATUS = 0xC0000002u32 as NTSTATUS; // kernel32-rust/src/console/history.rs
pub const STATUS_NOT_SUPPORTED: i32 = 0xC00000BB_u32 as i32; // ntdll-sys/src/constants.rs
pub const STATUS_NO_SUCH_DEVICE: i32 = 0xC000000Eu32 as i32; // afd-rust/src/glue.rs
pub const STATUS_PENDING: NTSTATUS = 0x00000103; // kernel32-rust/src/client/synch.rs
pub const STATUS_REVISION_MISMATCH: i32 = 0xC0000059_u32 as i32; // legide-rust/src/prelude.rs
pub const STATUS_SUCCESS: i32 = 0; // diskpart-rust/src/prelude.rs
pub const STATUS_UNSUCCESSFUL: NTSTATUS = 0xC000_0001u32 as NTSTATUS; // diskpart-rust/src/prelude.rs
pub const STATUS_WMI_GUID_NOT_FOUND: i32 = 0xC0000295u32 as i32; // classpnp-rust/src/glue.rs
pub const STATUS_WMI_INSTANCE_NOT_FOUND: i32 = 0xC0000296u32 as i32; // classpnp-rust/src/glue.rs
pub const STORAGE_ADDRESS_TYPE_BTL8: u32 = 0; // cdrom-rust/src/types_amd64.rs
pub const STORAGE_EVENT_ALL: u32 = 7; // cdrom-rust/src/types_amd64.rs
pub const STORAGE_EVENT_NOTIFICATION_VERSION_V1: u32 = 1; // cdrom-rust/src/types_amd64.rs
pub const SWDMA_ALL: u32 = 224; // pciidex-rust/src/prelude.rs
pub const StorageAccessAlignmentProperty: _STORAGE_PROPERTY_ID = _STORAGE_PROPERTY_ID_StorageAccessAlignmentProperty; // island enum
pub const StorageAdapterProperty: _STORAGE_PROPERTY_ID = _STORAGE_PROPERTY_ID_StorageAdapterProperty; // island enum
pub const StorageDeviceProperty: _STORAGE_PROPERTY_ID = _STORAGE_PROPERTY_ID_StorageDeviceProperty; // island enum
pub const StorageDeviceSeekPenaltyProperty: _STORAGE_PROPERTY_ID = _STORAGE_PROPERTY_ID_StorageDeviceSeekPenaltyProperty; // island enum
pub const StorageDeviceTrimProperty: _STORAGE_PROPERTY_ID = _STORAGE_PROPERTY_ID_StorageDeviceTrimProperty; // island enum
pub const SystemPowerState: _POWER_STATE_TYPE = _POWER_STATE_TYPE_SystemPowerState; // island enum
pub const THREAD_ALL_ACCESS: u32 = 2032639; // ntkrnl-sys/src/hal_bindgen_amd64.rs
pub const TargetDeviceRelation: _DEVICE_RELATION_TYPE = _DEVICE_RELATION_TYPE_TargetDeviceRelation; // island enum
pub const UDMA_ALL: u32 = 260096; // pciidex-rust/src/prelude.rs
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
    pub fn SptiHandleScsiPassthru( DeviceObject: PDEVICE_OBJECT, Irp: PIRP, MaximumTransferLength: ULONG, MaximumPhysicalPages: ULONG, ) -> NTSTATUS; // usbstor-rust/src/disk.rs
    pub fn strcmp(a: *const c_char, b: *const c_char) -> c_int; // browser-rust/src/prelude.rs
    pub fn strlen(s: *const i8) -> usize; // advapi32-rust/src/prelude.rs
    pub fn strstr(haystack: *const i8, needle: *const i8) -> *mut i8; // fstub-rust/src/disksup.rs
}
extern "system" {
    pub fn ExFreePoolWithTag(P: PVOID, Tag: ULONG); // freeldr-rust/src/heap.rs
    pub fn KeClearEvent(Event: PRKEVENT); // ntkrnl-sys/src/bindgen_amd64.rs
    pub fn KeSetEvent(Event: PRKEVENT, Increment: KPRIORITY, Wait: BOOLEAN) -> LONG; // ntkrnl-sys/src/bindgen_amd64.rs
    pub fn PoCallDriver(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS; // hal-rust/src/arm64/pnpdd.rs
    pub fn PoRequestPowerIrp( DeviceObject: *mut _DEVICE_OBJECT, MinorFunction: UCHAR, PowerState: POWER_STATE, CompletionFunction: PREQUEST_POWER_COMPLETE, Context: PVOID, Irp: *mut *mut _IRP, ) -> NTSTATUS; // pciidex-rust/src/types_amd64.rs
    pub fn PoSetPowerState( DeviceObject: *mut _DEVICE_OBJECT, Type: POWER_STATE_TYPE, State: POWER_STATE, ) -> POWER_STATE; // pciidex-rust/src/types_amd64.rs
    pub fn PoStartNextPowerIrp(Irp: PIRP); // hal-rust/src/arm64/pnpdd.rs
    pub fn RtlAssert( FailedAssertion: PVOID, FileName: PVOID, LineNumber: ULONG, Message: *mut CHAR, ); // mountmgr-rust/src/mountmgr.rs
}
// ==== end port resolve ====
pub use win32macros_rust::{ALIGN_UP, ARRAYSIZE, ASSERT, C_ASSERT, CopyMemory, DBG_UNREFERENCED_LOCAL_VARIABLE, DWLP_DLGPROC, DWLP_MSGRESULT, DWLP_USER, FIELD_OFFSET, GET_X_LPARAM, GET_Y_LPARAM, GetBValue, GetGValue, GetRValue, HIBYTE, HIWORD, IDE_SET_ADVANCE_PIO_MODE, IDE_SET_DEFAULT_PIO_MODE, IDE_SET_MWDMA_MODE, IDE_SET_SWDMA_MODE, IDE_SET_UDMA_MODE, INVALID_HANDLE_VALUE, InitializeObjectAttributes, IsContainerPartition, IsEqualGUID, IsFTPartition, IsRecognizedPartition, LANGIDFROMLCID, LOBYTE, LOWORD, MAKEINTRESOURCEA, MAKEINTRESOURCEW, MAKELANGID, MAKELCID, MAKELONG, MAKEWORD, MoveMemory, PAGE_ROUND_UP, PRIMARYLANGID, PtrToUlong, RGB, RTL_CONSTANT_STRING, RTL_CONTAINS_FIELD, RTL_FIELD_SIZE, RTL_NUMBER_OF, RTL_NUMBER_OF_FIELD, RTL_SIZEOF_THROUGH_FIELD, RtlCopyMemory, RtlFillMemory, RtlGetProcessHeap, RtlMoveMemory, RtlZeroMemory, SORTIDFROMLCID, SRB_STATUS, SUBLANGID, UNICODE_NULL, UNREFERENCED_PARAMETER, UlongToPtr, ZeroMemory, _countof, iswalnum, iswalpha, iswcntrl, iswdigit, iswgraph, iswlower, iswprint, iswpunct, iswspace, iswupper, iswxdigit, wide_bytes};
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, StringCbCatW, StringCbCopyNExW, StringCbCopyW, StringCbPrintfExW, StringCbPrintfW, StringCchCatW, StringCchCopyNExW, StringCchCopyW, StringCchLengthW, StringCchPrintfW, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
pub const ATAPORT_TAG: u32 = 0x50656449; // clang, atapi.c
pub const ATA_DEVICE_REQUEST_SIGNATURE: u32 = 0x72415441; // clang, fdo.c
pub const IOCTL_ATA_PASS_THROUGH: u32 = 0x4D02C; // clang, ioctl.c
pub const IOCTL_ATA_PASS_THROUGH_DIRECT: u32 = 0x4D030; // clang, ioctl.c
pub const IOCTL_SCSI_GET_INQUIRY_DATA: u32 = 0x4100C; // clang, ioctl.c
pub const IOCTL_SCSI_MINIPORT: u32 = 0x4D008; // clang, ioctl.c
pub const IOCTL_SCSI_PASS_THROUGH: u32 = 0x4D004; // clang, ioctl.c
pub const IOCTL_SCSI_PASS_THROUGH_DIRECT: u32 = 0x4D014; // clang, ioctl.c
pub const IOCTL_SCSI_RESCAN_BUS: u32 = 0x4101C; // clang, ioctl.c
pub const IO_ERR_CONTROLLER_ERROR: NTSTATUS = 0xC004000B_u32 as NTSTATUS; // clang, dev_error.c
pub const IO_ERR_PARITY: NTSTATUS = 0xC0040005_u32 as NTSTATUS; // clang, dev_error.c
pub const IO_ERR_TIMEOUT: NTSTATUS = 0xC0040009_u32 as NTSTATUS; // clang, dev_error.c
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
pub use crate::atapi::identify_funcs_h::AtaDevBytesPerLogicalSector;
pub use crate::atapi::identify_funcs_h::AtaDevCdbSizeInWords;
pub use crate::atapi::identify_funcs_h::AtaDevCurrentChsTranslation;
pub use crate::atapi::identify_funcs_h::AtaDevDefaultChsTranslation;
pub use crate::atapi::identify_funcs_h::AtaDevHas48BitAddressFeature;
pub use crate::atapi::identify_funcs_h::AtaDevHasCdbInterrupt;
pub use crate::atapi::identify_funcs_h::AtaDevHasDratFunction;
pub use crate::atapi::identify_funcs_h::AtaDevHasFlushCache;
pub use crate::atapi::identify_funcs_h::AtaDevHasFlushCacheExt;
pub use crate::atapi::identify_funcs_h::AtaDevHasForceUnitAccessCommands;
pub use crate::atapi::identify_funcs_h::AtaDevHasIeee1667;
pub use crate::atapi::identify_funcs_h::AtaDevHasLbaTranslation;
pub use crate::atapi::identify_funcs_h::AtaDevHasNcqAutosense;
pub use crate::atapi::identify_funcs_h::AtaDevHasRemovableMediaFeature;
pub use crate::atapi::identify_funcs_h::AtaDevHasRemovableMediaStatusNotification;
pub use crate::atapi::identify_funcs_h::AtaDevHasRzatFunction;
pub use crate::atapi::identify_funcs_h::AtaDevHasSecurityModeFeature;
pub use crate::atapi::identify_funcs_h::AtaDevHasSmartFeature;
pub use crate::atapi::identify_funcs_h::AtaDevHasTrimFunction;
pub use crate::atapi::identify_funcs_h::AtaDevHasWorldWideName;
pub use crate::atapi::identify_funcs_h::AtaDevInPuisState;
pub use crate::atapi::identify_funcs_h::AtaDevIsCurrentGeometryValid;
pub use crate::atapi::identify_funcs_h::AtaDevIsDmaDirectionRequired;
pub use crate::atapi::identify_funcs_h::AtaDevIsIdentifyDataIncomplete;
pub use crate::atapi::identify_funcs_h::AtaDevIsIdentifyDataValid;
pub use crate::atapi::identify_funcs_h::AtaDevIsReadLookAHeadEnabled;
pub use crate::atapi::identify_funcs_h::AtaDevIsRemovable;
pub use crate::atapi::identify_funcs_h::AtaDevIsRotatingDevice;
pub use crate::atapi::identify_funcs_h::AtaDevIsSsd;
pub use crate::atapi::identify_funcs_h::AtaDevIsTape;
pub use crate::atapi::identify_funcs_h::AtaDevIsVolatileWriteCacheEnabled;
pub use crate::atapi::identify_funcs_h::AtaDevLogicalSectorAlignment;
pub use crate::atapi::identify_funcs_h::AtaDevLogicalSectorsPerPhysicalSector;
pub use crate::atapi::identify_funcs_h::AtaDevMaximumSectorsPerDrq;
pub use crate::atapi::identify_funcs_h::AtaDevMediumRotationRate;
pub use crate::atapi::identify_funcs_h::AtaDevNominalFormFactor;
pub use crate::atapi::identify_funcs_h::AtaDevQueueDepth;
pub use crate::atapi::identify_funcs_h::AtaDevUserAddressableSectors28Bit;
pub use crate::atapi::identify_funcs_h::AtaDevUserAddressableSectors48Bit;
pub use crate::atapi::identify_funcs_h::AtaDevZonedCapabilities;
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

// ==== hand-ported macros: `port macros` leaves an item defined outside its block alone ====
// atapi.h:326 `ASSERT((Request) && (Request)->Signature == ATA_DEVICE_REQUEST_SIGNATURE)` --
// a pointer's truth is "not NULL".
#[allow(unused_macros)]
macro_rules! ASSERT_REQUEST {
    ($Request:expr $(,)?) => {
        ASSERT!((!($Request).is_null() && (*($Request)).Signature == ATA_DEVICE_REQUEST_SIGNATURE))
    };
}
#[allow(unused_imports)]
pub(crate) use ASSERT_REQUEST;
// atapi.h:179 `(0xFFFFFFFF >> (RTL_BITS_OF(ULONG) - (num)))` -- unsigned int arithmetic in C,
// whatever width `num` arrives in.
#[allow(unused_macros)]
macro_rules! NUM_TO_BITMAP {
    ($num:expr $(,)?) => {
        (0xFFFFFFFFu32 >> ((RTL_BITS_OF!(ULONG) as u32) - (($num) as u32)))
    };
}
#[allow(unused_imports)]
pub(crate) use NUM_TO_BITMAP;
// atapi.h:232/235 -- the flags are or'ed into / masked out of a pointer-wide value: C widens
// `Flags` to ULONG_PTR before `~`, so the complement covers the high half too.
#[allow(unused_macros)]
macro_rules! SRB_SET_FLAGS {
    ($Srb:expr, $Flags:expr $(,)?) => {{
        (*($Srb)).SrbExtension =
            (((*($Srb)).SrbExtension as ULONG_PTR) | (($Flags) as ULONG_PTR)) as PVOID;
        (*($Srb)).SrbExtension
    }};
}
#[allow(unused_imports)]
pub(crate) use SRB_SET_FLAGS;
#[allow(unused_macros)]
macro_rules! SRB_CLEAR_FLAGS {
    ($Srb:expr, $Flags:expr $(,)?) => {{
        (*($Srb)).SrbExtension =
            (((*($Srb)).SrbExtension as ULONG_PTR) & !(($Flags) as ULONG_PTR)) as PVOID;
        (*($Srb)).SrbExtension
    }};
}
#[allow(unused_imports)]
pub(crate) use SRB_CLEAR_FLAGS;

// ==== port macros: the module's own C macros (generated -- `rustconv port macros`) ====
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
        (((*($Device)).__bindgen_anon_1.TransportFlags & DEVICE_IS_ATAPI) != 0)
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
// drivers/storage/ide/atapi/atapi.h:310: #define QUEUE_ENTRY_FROM_IRP(Irp) ((PREQUEST_QUEUE_ENTRY)&(((PIRP)(Irp))->Tail.Overlay.DriverContext[0]))
#[allow(unused_macros)]
macro_rules! QUEUE_ENTRY_FROM_IRP {
    ($Irp:expr $(,)?) => {
        (core::ptr::addr_of!((*($Irp as PIRP)).Tail.Overlay.__bindgen_anon_1.__bindgen_anon_1.DriverContext[0]) as PREQUEST_QUEUE_ENTRY)
    };
}
#[allow(unused_imports)]
pub(crate) use QUEUE_ENTRY_FROM_IRP;
// drivers/storage/ide/atapi/atapi.h:238: #define SRB_GET_FLAGS(Srb) ((ULONG_PTR)(Srb)->SrbExtension)
#[allow(unused_macros)]
macro_rules! SRB_GET_FLAGS {
    ($Srb:expr $(,)?) => {
        ((*($Srb)).SrbExtension as ULONG_PTR)
    };
}
#[allow(unused_imports)]
pub(crate) use SRB_GET_FLAGS;
// ==== end port macros ====

// ==== port resolve: substrate helpers (generated) ====
pub use ntkrnl_sys::mm::ADDRESS_AND_SIZE_TO_SPAN_PAGES;
pub use ntkrnl_sys::inline::mm::ExAllocatePoolUninitialized;
pub use ntkrnl_sys::inline::mm::ExAllocatePoolZero;
#[cfg(target_arch = "x86")]
pub use ntkrnl_sys::ddk::ExInterlockedFlushSList;
#[cfg(not(target_arch = "x86"))]
pub use ntkrnl_sys::inline::ex::ExInterlockedFlushSList;
pub use ntkrnl_sys::inline::common::GetIRPMinorFunctionString;
pub use ntkrnl_sys::inline::common::InitializeListHead;
pub use ntkrnl_sys::inline::common::InitializeSListHead;
pub use ntkrnl_sys::inline::common::InsertTailList;
pub use ntkrnl_sys::inline::ex::InterlockedPushEntrySList;
pub use ntkrnl_sys::ddk::IoAcquireRemoveLockEx;
pub use ntkrnl_sys::inline::common::IoAdjustPagingPathCount;
pub use ntkrnl_sys::driver_ddk::IoCallDriver;
pub use ntkrnl_sys::driver_ddk::IoCompleteRequest;
pub use ntkrnl_sys::inline::common::IoInitializeRemoveLock;
pub use ntkrnl_sys::ddk::IoReleaseRemoveLockAndWaitEx;
pub use ntkrnl_sys::ddk::IoReleaseRemoveLockEx;
pub use ntkrnl_sys::inline::common::IsListEmpty;
pub use ntkrnl_sys::ke::KeGetCurrentIrql;
#[cfg(target_arch = "x86")]
pub use ntkrnl_sys::ddk::KeQuerySystemTime;
#[cfg(not(target_arch = "x86"))]
pub use ntkrnl_sys::inline::ke::KeQuerySystemTime;
pub use ntkrnl_sys::inline::mm::MmGetMdlByteCount;
pub use ntkrnl_sys::inline::mm::MmGetMdlVirtualAddress;
pub use ntkrnl_sys::inline::mm::MmGetSystemAddressForMdlSafe;
pub use ntkrnl_sys::ob::ObDereferenceObject;
pub use ntkrnl_sys::inline::tier1::ObReferenceObject;
pub use ntkrnl_sys::inline::common::PoSetDeviceBusy;
pub use ntkrnl_sys::inline::common::PopEntryList;
pub use ntkrnl_sys::inline::common::PushEntryList;
pub use ntkrnl_sys::inline::common::RTL_CONSTANT_OBJECT_ATTRIBUTES;
pub use ntkrnl_sys::inline::common::RemoveEntryList;
pub use ntkrnl_sys::inline::common::RemoveHeadList;
pub use ntkrnl_sys::inline::common::RtlEqualMemory;
pub use ntkrnl_sys::inline::common::RtlULongAdd;
#[cfg(target_arch = "x86")]
pub use ntkrnl_sys::ddk::RtlUlongByteSwap;
#[cfg(not(target_arch = "x86"))]
pub use ntkrnl_sys::inline::common::RtlUlongByteSwap;
#[cfg(target_arch = "x86")]
pub use ntkrnl_sys::ddk::RtlUlonglongByteSwap;
#[cfg(not(target_arch = "x86"))]
pub use ntkrnl_sys::inline::common::RtlUlonglongByteSwap;
#[cfg(target_arch = "x86")]
pub use ntkrnl_sys::ddk::RtlUshortByteSwap;
#[cfg(not(target_arch = "x86"))]
pub use ntkrnl_sys::inline::common::RtlUshortByteSwap;
pub use ntkrnl_sys::inline::ke::_InterlockedExchange;
// ==== end substrate helpers ====

// ==== port resolve: INITGUID storage (generated) ====
#[no_mangle]
pub static GUID_DEVINTERFACE_STORAGEPORT: GUID = GUID { Data1: 0x2accfe60, Data2: 0xc130, Data3: 0x11d2, Data4: [0xb0, 0x82, 0x00, 0xa0, 0xc9, 0x1e, 0xfb, 0x8b] };
#[no_mangle]
pub static GUID_PCIIDE_INTERFACE_ROS: GUID = GUID { Data1: 0xD677FBCF, Data2: 0xABED, Data3: 0x47C8, Data4: [0x80, 0xA3, 0xE4, 0x34, 0x7E, 0xA4, 0x96, 0x47] };
// ==== end INITGUID storage ====
