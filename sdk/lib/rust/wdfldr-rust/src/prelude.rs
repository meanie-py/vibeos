//! wdfldr-rust prelude — the single import surface the translated modules use.
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
pub use win32macros_rust::{ALIGN_UP, ARRAYSIZE, ARRAY_SIZE, ASSERT, C_ASSERT, CopyMemory, DBG_UNREFERENCED_LOCAL_VARIABLE, DPRINT1, DWLP_DLGPROC, DWLP_MSGRESULT, DWLP_USER, ERR_ON, FAILED_UNEXPECTEDLY, FIELD_OFFSET, FIXME_ON, GET_WHEEL_DELTA_WPARAM, GET_X_LPARAM, GET_Y_LPARAM, GetAValue, GetBValue, GetGValue, GetRValue, HIBYTE, HIWORD, HRESULT_FROM_WIN32, HandleToUlong, IDE_SET_ADVANCE_PIO_MODE, IDE_SET_DEFAULT_PIO_MODE, IDE_SET_MWDMA_MODE, IDE_SET_SWDMA_MODE, IDE_SET_UDMA_MODE, INVALID_HANDLE_VALUE, IS_INTRESOURCE, InitializeObjectAttributes, IntToPtr, InterlockedExchange, IsContainerPartition, IsEqualGUID, IsFTPartition, IsRecognizedPartition, LANGIDFROMLCID, LOBYTE, LOWORD, LongToHandle, LongToPtr, MAKEINTATOM, MAKEINTRESOURCEA, MAKEINTRESOURCEW, MAKELANGID, MAKELCID, MAKELONG, MAKEWORD, MoveMemory, PAGE_ROUND_UP, PRIMARYLANGID, PtrToInt, PtrToLong, PtrToUint, PtrToUlong, RGB, RGBA, RTL_CONSTANT_STRING, RTL_CONTAINS_FIELD, RTL_FIELD_SIZE, RTL_NUMBER_OF, RTL_NUMBER_OF_FIELD, RTL_SIZEOF_THROUGH_FIELD, RtlCopyMemory, RtlFillMemory, RtlGetProcessHeap, RtlMoveMemory, RtlZeroMemory, SORTIDFROMLCID, SRB_STATUS, SUBLANGID, TRACE_ON, UIntToPtr, ULongToHandle, ULongToPtr, UNICODE_NULL, UNIMPLEMENTED_DBGBREAK, UNREFERENCED_PARAMETER, UintToPtr, UlongToHandle, UlongToPtr, WARN_ON, WINE_ERR, WINE_FIXME, WINE_TRACE, WINE_WARN, ZeroMemory, __HRESULT_FROM_WIN32, __p___argc, __p___argv, __p___wargv, __p__environ, __p__pgmptr, __p__wenviron, __p__wpgmptr, __wine_dprintf, __wine_is_debug_on, _countof, debugstr_a, debugstr_an, debugstr_guid, debugstr_w, debugstr_wn, heap_alloc, heap_alloc_zero, heap_calloc, heap_free, heap_realloc, iswalnum, iswalpha, iswcntrl, iswdigit, iswgraph, iswlower, iswprint, iswpunct, iswspace, iswupper, iswxdigit, wide_bytes, wine_dbgstr_a, wine_dbgstr_guid, wine_dbgstr_w};

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

// ==== port resolve: tree consensus (generated -- re-run `rustconv port resolve`) ====
pub const DPFLTR_DEFAULT_ID: _DPFLTR_TYPE = _DPFLTR_TYPE_DPFLTR_DEFAULT_ID; // island enum
pub const DPFLTR_ERROR_LEVEL: u32 = 0; // ntkrnl-sys/src/hal_bindgen_amd64.rs
pub const KeyValuePartialInformation: _KEY_VALUE_INFORMATION_CLASS = _KEY_VALUE_INFORMATION_CLASS_KeyValuePartialInformation; // island enum
pub const NonPagedPool: _POOL_TYPE = _POOL_TYPE_NonPagedPool; // island enum
pub const PagedPool: _POOL_TYPE = _POOL_TYPE_PagedPool; // island enum
// ==== end port resolve ====

// ==== port resolve: cross-TU definitions (generated) ====
pub use crate::wdfldr::registry::BuildServicePath;
pub use crate::wdfldr::class::ClassAddReference;
pub use crate::wdfldr::class::ClassClientCreate;
pub use crate::wdfldr::class::ClassClose;
pub use crate::wdfldr::class::ClassCreate;
pub use crate::wdfldr::class::ClassLinkInClient;
pub use crate::wdfldr::class::ClassOpen;
pub use crate::wdfldr::class::ClassRemoveFromLibraryList;
pub use crate::wdfldr::class::DereferenceClassVersion;
pub use crate::wdfldr::class::FindClassByServiceNameLocked;
pub use crate::wdfldr::library::FindLibraryByServicePathLocked;
pub use crate::wdfldr::library::FindModuleByClientService;
pub use crate::wdfldr::common::FxLdrAcquireLoadedModuleLock;
pub use crate::wdfldr::registry::FxLdrQueryData;
pub use crate::wdfldr::registry::FxLdrQueryUlong;
pub use crate::wdfldr::common::FxLdrReleaseLoadedModuleLock;
pub use crate::wdfldr::common::GetImageInfo;
pub use crate::wdfldr::common::GetImageName;
pub use crate::wdfldr::common::GetNameFromPath;
pub use crate::wdfldr::registry::GetVersionServicePath;
pub use crate::wdfldr::library::LibraryAcquireClientLock;
pub use crate::wdfldr::class::LibraryAddToClassListLocked;
pub use crate::wdfldr::library::LibraryClose;
pub use crate::wdfldr::library::LibraryCreate;
pub use crate::wdfldr::library::LibraryDereference;
pub use crate::wdfldr::library::LibraryFindOrLoad;
pub use crate::wdfldr::library::LibraryFree;
pub use crate::wdfldr::library::LibraryLinkInClient;
pub use crate::wdfldr::library::LibraryOpen;
pub use crate::wdfldr::library::LibraryReference;
pub use crate::wdfldr::library::LibraryReleaseClientLock;
pub use crate::wdfldr::library::LibraryReleaseReference;
pub use crate::wdfldr::library::LibraryUnlinkClient;
pub use crate::wdfldr::library::LibraryUnload;
pub use crate::wdfldr::class::LibraryUnloadClasses;
pub use crate::wdfldr::class::ReferenceClassVersion;
pub use crate::wdfldr::registry::ServiceCheckBootStart;
pub use crate::wdfldr::wdfldr::WdfLdrDiags;
pub use crate::wdfldr::wdfldr::WdfLdrGlobals;
// ==== end cross-TU definitions ====

// ==== port resolve: substrate helpers (generated) ====
pub use ntkrnl_sys::ddk::ExAllocatePoolWithTag;
pub use ntkrnl_sys::inline::mm::ExAllocatePoolZero;
pub use ntkrnl_sys::ddk::ExFreePoolWithTag;
pub use ntkrnl_sys::inline::common::InitializeListHead;
pub use ntkrnl_sys::inline::common::InsertHeadList;
pub use ntkrnl_sys::inline::common::InsertTailList;
pub use ntkrnl_sys::ddk::IoAcquireRemoveLockEx;
pub use ntkrnl_sys::ddk::IoReleaseRemoveLockAndWaitEx;
pub use ntkrnl_sys::ddk::IoReleaseRemoveLockEx;
pub use ntkrnl_sys::inline::common::IsListEmpty;
#[cfg(not(target_arch = "x86_64"))]
pub use ntkrnl_sys::ddk::KeGetCurrentIrql;
#[cfg(target_arch = "x86_64")]
pub use ntkrnl_sys::ke::KeGetCurrentIrql;
#[cfg(target_arch = "x86_64")]
pub use ntkrnl_sys::inline::tier1::KeGetCurrentThread;
#[cfg(not(target_arch = "x86_64"))]
pub use ntkrnl_sys::mm::KeGetCurrentThread;
pub use ntkrnl_sys::ob::ObDereferenceObject;
pub use ntkrnl_sys::inline::common::RemoveEntryList;
pub use ntkrnl_sys::inline::common::RemoveHeadList;
pub use ntkrnl_sys::inline::common::RtlULongAdd;
pub use ntkrnl_sys::inline::common::RtlUShortAdd;
pub use ntkrnl_sys::constants::STATUS_BUFFER_TOO_SMALL;
pub use ntkrnl_sys::constants::STATUS_IMAGE_ALREADY_LOADED;
pub use ntkrnl_sys::base::STATUS_INSUFFICIENT_RESOURCES;
pub use ntkrnl_sys::constants::STATUS_INVALID_PARAMETER;
pub use ntkrnl_sys::constants::STATUS_NOT_FOUND;
pub use ntkrnl_sys::constants::STATUS_OBJECT_NAME_NOT_FOUND;
pub use ntkrnl_sys::constants::STATUS_OBJECT_PATH_NOT_FOUND;
pub use ntkrnl_sys::constants::STATUS_OBJECT_TYPE_MISMATCH;
pub use ntkrnl_sys::constants::STATUS_REVISION_MISMATCH;
pub use ntkrnl_sys::base::STATUS_SUCCESS;
pub use ntkrnl_sys::driver_ddk::wcslen;
// ==== end substrate helpers ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, RtlUnicodeStringPrintf, StringCbCatNW, StringCbCatW, StringCbCopyExW, StringCbCopyNExW, StringCbCopyNW, StringCbCopyW, StringCbPrintfExW, StringCbPrintfW, StringCbVPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyA, StringCchCopyExW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthA, StringCchLengthW, StringCchPrintfA, StringCchPrintfExA, StringCchPrintfExW, StringCchPrintfW, StringCchVPrintfA, StringCchVPrintfExA, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};
pub use conutils_h::{AnsiText, Binary, CON_STREAM, CON_STREAM_MODE, CON_WRITE_FUNC, ConInitStdStreams, ConInitStdStreamsAndMode, ConMsgPrintf, ConMsgPuts, ConPrintf, ConPrintfV, ConPuts, ConResMsgPrintf, ConResMsgPrintfEx, ConResPrintf, ConResPrintfEx, ConResPrintfV, ConResPuts, ConResPutsEx, ConSetThreadUILanguage, ConStdStreamsSetCacheCodePage, ConStreamGetOSHandle, ConStreamInit, ConStreamInitEx, ConStreamSetCacheCodePage, ConStreamSetMode, ConStreamSetOSHandle, ConStreamWrite, ConWrite, INVALID_CP, K32LoadStringW, PCON_STREAM, StdErr, StdIn, StdOut, UTF16Text, UTF8Text, WideText, csStdErr, csStdIn, csStdOut};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
pub const WDFLDR_TAG: u32 = 0x4C666457; // clang, class.c
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// PWDF_INTERFACE_HEADER: c2rust, wdfldr.c
pub type PWDF_INTERFACE_HEADER = *mut _WDF_INTERFACE_HEADER;
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
extern "C" {
    pub fn _strnicmp(
        _Str1: *const ::core::ffi::c_char,
        _Str2: *const ::core::ffi::c_char,
        _MaxCount: size_t,
    ) -> ::core::ffi::c_int; // common.c
}
// ==== end port externs ====

// ==== port macros: SDK header macros, lowered from the preprocessor's own expansion (generated -- re-run `rustconv port macros`) ====
pub const STATUS_DRIVER_INTERNAL_ERROR: NTSTATUS = 0xc0000183usize as NTSTATUS;
pub const STATUS_INTEGER_OVERFLOW: NTSTATUS = 0xc0000095usize as NTSTATUS;
pub const STATUS_INVALID_BUFFER_SIZE: NTSTATUS = 0xc0000206usize as NTSTATUS;
pub const STATUS_INVALID_DEVICE_STATE: NTSTATUS = 0xc0000184usize as NTSTATUS;
pub const STATUS_NOINTERFACE: NTSTATUS = 0xc00002b9usize as NTSTATUS;
pub const STATUS_NOT_SAFE_MODE_DRIVER: NTSTATUS = 0xc000035fusize as NTSTATUS;
// ==== end SDK header macros ====

// ==== wdfloader.h's debug macros, hand-ported ====================================================
// C: `__DBGPRINT(("fmt", args))`, a do/while that prints a "Wdfldr: <function> - " prefix and
// then hands the inner argument list to DbgPrintEx. The generated port took one token tree and
// passed DbgPrint a Rust &str; integrate had meanwhile flattened some call sites to
// `__DBGPRINT!(b"..\0".as_ptr(), ..)`. These accept both shapes, and a string-literal format is
// NUL-terminated here. __FUNCTION__ has no stable Rust spelling: module_path!() names the file.
#[allow(unused_macros)]
macro_rules! __PrintUnfiltered {
    (($($inner:tt)*)) => { $crate::prelude::__PrintUnfiltered!($($inner)*) };
    ($fmt:literal $(, $a:expr)* $(,)?) => {
        DbgPrintEx(DPFLTR_DEFAULT_ID as _, DPFLTR_ERROR_LEVEL as _,
            concat!($fmt, "\0").as_ptr() as _ $(, $a)*)
    };
    ($fmt:expr $(, $a:expr)* $(,)?) => {
        DbgPrintEx(DPFLTR_DEFAULT_ID as _, DPFLTR_ERROR_LEVEL as _, ($fmt) as _ $(, $a)*)
    };
}
#[allow(unused_imports)]
pub(crate) use __PrintUnfiltered;
macro_rules! wdfldr_diag {
    ($flag:expr, $prefix:literal $(, $($x:tt)*)?) => {{
        if (WdfLdrDiags.__bindgen_anon_1.DiagFlags & ($flag)) != 0 {
            DbgPrint(concat!($prefix, "\0").as_ptr() as _,
                concat!(module_path!(), "\0").as_ptr() as *const core::ffi::c_char);
            $( $crate::prelude::__PrintUnfiltered!($($x)*); )?
        }
    }};
}
pub(crate) use wdfldr_diag;
#[allow(unused_macros)]
macro_rules! __DBGPRINT {
    ($($x:tt)*) => { $crate::prelude::wdfldr_diag!(DIAGFLAG_ENABLED, "Wdfldr: %s - ", $($x)*) };
}
#[allow(unused_imports)]
pub(crate) use __DBGPRINT;
#[allow(unused_macros)]
macro_rules! DPRINT_ERROR {
    ($($x:tt)*) => { $crate::prelude::wdfldr_diag!(DIAGFLAG_LOG_ERRORS, "WdfLdr: ERROR: %s - ", $($x)*) };
}
#[allow(unused_imports)]
pub(crate) use DPRINT_ERROR;
#[allow(unused_macros)]
macro_rules! DPRINT_VERBOSE {
    ($($x:tt)*) => { $crate::prelude::wdfldr_diag!(DIAGFLAG_VERBOSE_LOGGING, "WdfLdr: %s - ", $($x)*) };
}
#[allow(unused_imports)]
pub(crate) use DPRINT_VERBOSE;
#[allow(unused_macros)]
macro_rules! DPRINT_TRACE_ENTRY {
    () => { $crate::prelude::wdfldr_diag!(DIAGFLAG_TRACE_FUNCTION_ENTRY, "WdfLdr: ENTER: %s\n") };
}
#[allow(unused_imports)]
pub(crate) use DPRINT_TRACE_ENTRY;
#[allow(unused_macros)]
macro_rules! DPRINT_TRACE_EXIT {
    () => { $crate::prelude::wdfldr_diag!(DIAGFLAG_TRACE_FUNCTION_EXIT, "WdfLdr: EXIT: %s\n") };
}
#[allow(unused_imports)]
pub(crate) use DPRINT_TRACE_EXIT;

