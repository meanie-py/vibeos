//! legide-rust prelude — the single import surface the translated modules use.
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
pub const APC_LEVEL: u32 = 1; // atapi-rust/src/prelude.rs
pub const CM_RESOURCE_INTERRUPT_LATCHED: u32 = 1; // atapi-rust/src/types_amd64.rs
pub const CM_RESOURCE_PORT_16_BIT_DECODE: u32 = 16; // atapi-rust/src/types_amd64.rs
pub const CM_RESOURCE_PORT_IO: u32 = 1; // atapi-rust/src/types_amd64.rs
pub const CmResourceShareDeviceExclusive: u8 = 1; // pciidex-rust/src/prelude.rs
pub const IDE_COMMAND_ATAPI_IDENTIFY: u32 = 161; // atapi-legacy-rust/src/types_amd64.rs
pub const IDE_COMMAND_IDENTIFY: u32 = 236; // atapi-legacy-rust/src/types_amd64.rs
pub const IDE_DC_DISABLE_INTERRUPTS: u32 = 2; // atapi-legacy-rust/src/types_amd64.rs
pub const IDE_DC_RESET_CONTROLLER: u32 = 4; // atapi-legacy-rust/src/types_amd64.rs
pub const IDE_STATUS_BUSY: u32 = 128; // atapi-legacy-rust/src/types_amd64.rs
pub const IDE_STATUS_DEVICE_FAULT: u32 = 32; // atapi-rust/src/prelude.rs
pub const IDE_STATUS_DRQ: u32 = 8; // atapi-legacy-rust/src/types_amd64.rs
pub const IDE_STATUS_ERROR: u32 = 1; // atapi-legacy-rust/src/types_amd64.rs
pub const InterfaceTypeUndefined: _INTERFACE_TYPE = _INTERFACE_TYPE_InterfaceTypeUndefined; // island enum
pub const IsNEC_98: u32 = 0; // atapi-rust/src/prelude.rs
pub const Isa: _INTERFACE_TYPE = _INTERFACE_TYPE_Isa; // island enum
pub const KEY_ALL_ACCESS: u32 = 983103; // atapi-rust/src/prelude.rs
pub const MAX_IDE_DEVICE: u32 = 2; // atapi-rust/src/prelude.rs
pub const MmNonCached: MEMORY_CACHING_TYPE = 0; // atapi-rust/src/prelude.rs
pub const OBJ_CASE_INSENSITIVE: u32 = 64; // atapi-rust/src/prelude.rs
pub const OBJ_KERNEL_HANDLE: u32 = 512; // atapi-rust/src/prelude.rs
pub const PCIIDEX_GET_CONTROLLER_INTERFACE_SIGNATURE: u32 = 4294963199; // atapi-rust/src/types_amd64.rs
pub const PCIIDEX_INTERFACE_VERSION: u32 = 1; // atapi-rust/src/prelude.rs
pub const PagedPool: POOL_TYPE = 1; // atapi-rust/src/prelude.rs
pub const REG_DWORD: u32 = 4; // advapi32-sys/src/bindgen_amd64.rs
pub const RTL_QUERY_REGISTRY_DIRECT: u32 = 32; // atapi-rust/src/types_amd64.rs
pub const RTL_QUERY_REGISTRY_REQUIRED: u32 = 4; // atapi-rust/src/types_amd64.rs
pub const RTL_REGISTRY_ABSOLUTE: u32 = 0; // atapi-rust/src/types_amd64.rs
pub const RTL_REGISTRY_HANDLE: u32 = 1073741824; // atapi-rust/src/types_amd64.rs
pub const STATUS_CANCELLED: NTSTATUS = 0xC000_0120u32 as NTSTATUS; // atapi-rust/src/prelude.rs
pub const STATUS_INSUFFICIENT_RESOURCES: NTSTATUS = 0xC000_009Au32 as NTSTATUS; // atapi-rust/src/prelude.rs
pub const STATUS_NOT_FOUND: i32 = 0xC0000225_u32 as i32; // ntdll-sys/src/constants.rs
pub const STATUS_REVISION_MISMATCH: i32 = 0xC0000059_u32 as i32; // atapi-rust/src/prelude.rs
pub const STATUS_SUCCESS: i32 = 0; // atapi-rust/src/prelude.rs
pub const STATUS_UNSUCCESSFUL: NTSTATUS = 0xC000_0001u32 as NTSTATUS; // atapi-rust/src/prelude.rs
extern "system" {
    pub fn PciIdeXInitialize( DriverObject: PDRIVER_OBJECT, RegistryPath: PUNICODE_STRING, HwGetControllerProperties: PCONTROLLER_PROPERTIES, ExtensionSize: ULONG, ) -> NTSTATUS; // atapi-rust/src/types_amd64.rs
    pub fn RtlAssert( FailedAssertion: PVOID, FileName: PVOID, LineNumber: ULONG, Message: *mut CHAR, ); // mountmgr-rust/src/mountmgr.rs
}
// ==== end port resolve ====
pub use win32macros_rust::{ALIGN_UP, ARRAYSIZE, ASSERT, C_ASSERT, CopyMemory, DBG_UNREFERENCED_LOCAL_VARIABLE, DWLP_DLGPROC, DWLP_MSGRESULT, DWLP_USER, FIELD_OFFSET, GET_X_LPARAM, GET_Y_LPARAM, GetBValue, GetGValue, GetRValue, HIBYTE, HIWORD, IDE_SET_ADVANCE_PIO_MODE, IDE_SET_DEFAULT_PIO_MODE, IDE_SET_MWDMA_MODE, IDE_SET_SWDMA_MODE, IDE_SET_UDMA_MODE, INVALID_HANDLE_VALUE, InitializeObjectAttributes, IsContainerPartition, IsEqualGUID, IsFTPartition, IsRecognizedPartition, LANGIDFROMLCID, LOBYTE, LOWORD, MAKEINTRESOURCEA, MAKEINTRESOURCEW, MAKELANGID, MAKELCID, MAKELONG, MAKEWORD, MoveMemory, PAGE_ROUND_UP, PRIMARYLANGID, PtrToUlong, RGB, RTL_CONSTANT_STRING, RTL_CONTAINS_FIELD, RTL_FIELD_SIZE, RTL_NUMBER_OF, RTL_NUMBER_OF_FIELD, RTL_SIZEOF_THROUGH_FIELD, RtlCopyMemory, RtlFillMemory, RtlGetProcessHeap, RtlMoveMemory, RtlZeroMemory, SORTIDFROMLCID, SRB_STATUS, SUBLANGID, UNICODE_NULL, UNREFERENCED_PARAMETER, UlongToPtr, ZeroMemory, _countof, iswalnum, iswalpha, iswcntrl, iswdigit, iswgraph, iswlower, iswprint, iswpunct, iswspace, iswupper, iswxdigit, wide_bytes};
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, StringCbCatW, StringCbCopyNExW, StringCbCopyW, StringCbPrintfExW, StringCbPrintfW, StringCchCatW, StringCchCopyNExW, StringCchCopyW, StringCchLengthW, StringCchPrintfW, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
pub const CHANNEL_PC98_RESOURCE_COUNT: u32 = 0xC; // clang, legide.c
pub const CHANNEL_PCAT_RESOURCE_COUNT: u32 = 3; // clang, legide.c
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
// ==== end port externs ====

// ==== port macros: the module's own C macros (generated -- `rustconv port macros`) ====
// drivers/storage/ide/legide/legide.c:33: #define ATA_IO_WAIT() KeStallExecutionProcessor(1)
#[allow(unused_macros)]
macro_rules! ATA_IO_WAIT {
    () => {
        (KeStallExecutionProcessor(1))
    };
}
#[allow(unused_imports)]
pub(crate) use ATA_IO_WAIT;
// drivers/storage/ide/legide/legide.c:31: #define ATA_READ(Port) READ_PORT_UCHAR((Port))
#[allow(unused_macros)]
macro_rules! ATA_READ {
    ($Port:expr $(,)?) => {
        (READ_PORT_UCHAR($Port))
    };
}
#[allow(unused_imports)]
pub(crate) use ATA_READ;
// drivers/storage/ide/legide/legide.c:32: #define ATA_WRITE(Port, Value) WRITE_PORT_UCHAR((Port), (Value))
#[allow(unused_macros)]
macro_rules! ATA_WRITE {
    ($Port:expr, $Value:expr $(,)?) => {
        (WRITE_PORT_UCHAR($Port, $Value))
    };
}
#[allow(unused_imports)]
pub(crate) use ATA_WRITE;
// ==== end port macros ====

// ==== port resolve: substrate helpers (generated) ====
pub use ntkrnl_sys::driver_ddk::DbgPrint;
pub use ntkrnl_sys::inline::mm::ExAllocatePoolZero;
pub use ntkrnl_sys::ke::KeGetCurrentIrql;
pub use ntkrnl_sys::portio::READ_PORT_BUFFER_USHORT;
// ==== end substrate helpers ====
