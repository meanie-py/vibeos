//! blue-rust prelude — the single import surface the translated modules use.
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
pub use win32macros_rust::{ALIGN_UP, ARRAYSIZE, ARRAY_SIZE, ASSERT, C_ASSERT, CopyMemory, DBG_UNREFERENCED_LOCAL_VARIABLE, DPRINT1, DWLP_DLGPROC, DWLP_MSGRESULT, DWLP_USER, ERR_ON, FAILED_UNEXPECTEDLY, FD_SET, FD_ZERO, FIELD_OFFSET, FIXME_ON, GET_WHEEL_DELTA_WPARAM, GET_X_LPARAM, GET_Y_LPARAM, GetAValue, GetBValue, GetGValue, GetRValue, HIBYTE, HIWORD, HRESULT_FROM_WIN32, HandleToUlong, IDE_SET_ADVANCE_PIO_MODE, IDE_SET_DEFAULT_PIO_MODE, IDE_SET_MWDMA_MODE, IDE_SET_SWDMA_MODE, IDE_SET_UDMA_MODE, INVALID_HANDLE_VALUE, IS_INTRESOURCE, InitializeObjectAttributes, IntToPtr, InterlockedExchange, IsContainerPartition, IsEqualGUID, IsFTPartition, IsRecognizedPartition, LANGIDFROMLCID, LOBYTE, LOWORD, LongToHandle, LongToPtr, MAKEINTATOM, MAKEINTRESOURCEA, MAKEINTRESOURCEW, MAKELANGID, MAKELCID, MAKELONG, MAKEWORD, MoveMemory, PAGE_ROUND_UP, PRIMARYLANGID, PtrToInt, PtrToLong, PtrToUint, PtrToUlong, RGB, RGBA, RTL_CONSTANT_STRING, RTL_CONTAINS_FIELD, RTL_FIELD_SIZE, RTL_NUMBER_OF, RTL_NUMBER_OF_FIELD, RTL_SIZEOF_THROUGH_FIELD, RtlCopyMemory, RtlFillMemory, RtlGetProcessHeap, RtlMoveMemory, RtlZeroMemory, SORTIDFROMLCID, SRB_STATUS, SUBLANGID, TRACE_ON, UIntToPtr, ULongToHandle, ULongToPtr, UNICODE_NULL, UNIMPLEMENTED_DBGBREAK, UNREFERENCED_PARAMETER, UintToPtr, UlongToHandle, UlongToPtr, WARN_ON, WINE_ERR, WINE_FIXME, WINE_TRACE, WINE_WARN, ZeroMemory, __HRESULT_FROM_WIN32, __debugbreak, __p___argc, __p___argv, __p___wargv, __p__environ, __p__pgmptr, __p__wenviron, __p__wpgmptr, __wine_dprintf, __wine_is_debug_on, _byteswap_uint64, _byteswap_ulong, _byteswap_ushort, _countof, debugstr_a, debugstr_an, debugstr_guid, debugstr_variant, debugstr_vt, debugstr_w, debugstr_wn, heap_alloc, heap_alloc_zero, heap_calloc, heap_free, heap_realloc, iswalnum, iswalpha, iswcntrl, iswdigit, iswgraph, iswlower, iswprint, iswpunct, iswspace, iswupper, iswxdigit, lrintf, wide_bytes, wine_dbgstr_a, wine_dbgstr_guid, wine_dbgstr_longlong, wine_dbgstr_variant, wine_dbgstr_vt, wine_dbgstr_w};

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

// intrin.h's _disable/_enable are instructions, not exports (ntkrnl-sys emits them inline).
pub use ntkrnl_sys::inline::ke::{_disable, _enable};

// ==== port resolve: tree consensus (generated -- re-run `rustconv port resolve`) ====
pub const NormalPagePriority: _MM_PAGE_PRIORITY = _MM_PAGE_PRIORITY_NormalPagePriority; // island enum
// ==== end port resolve ====

// ==== port resolve: cross-TU definitions (generated) ====
pub use crate::blue::font::ScrSetFont;
// ==== end cross-TU definitions ====

// ==== port resolve: substrate helpers (generated) ====
pub use ntkrnl_sys::ddk::ExAllocatePoolWithTag;
pub use ntkrnl_sys::ddk::ExFreePoolWithTag;
pub use ntkrnl_sys::ddk::IoAcquireRemoveLockEx;
pub use ntkrnl_sys::driver_ddk::IoCompleteRequest;
pub use ntkrnl_sys::ddk::IoReleaseRemoveLockAndWaitEx;
pub use ntkrnl_sys::ddk::IoReleaseRemoveLockEx;
#[cfg(target_arch = "x86_64")]
pub use ntkrnl_sys::inline::tier1::KeGetCurrentThread;
#[cfg(not(target_arch = "x86_64"))]
pub use ntkrnl_sys::mm::KeGetCurrentThread;
pub use ntkrnl_sys::mm::MmNonCached;
pub use ntkrnl_sys::mm::NonPagedPool;
pub use ntkrnl_sys::constants::OBJ_KERNEL_HANDLE;
pub use ntkrnl_sys::ddk::PsTerminateSystemThread;
pub use ntkrnl_sys::base::STATUS_INSUFFICIENT_RESOURCES;
pub use ntkrnl_sys::constants::STATUS_INVALID_PARAMETER;
pub use ntkrnl_sys::base::STATUS_NOT_IMPLEMENTED;
pub use ntkrnl_sys::base::STATUS_NO_MEMORY;
pub use ntkrnl_sys::base::STATUS_SUCCESS;
pub use ntkrnl_sys::constants::STATUS_UNSUCCESSFUL;
// ==== end substrate helpers ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, RtlUnicodeStringPrintf, StringCbCatNW, StringCbCatW, StringCbCopyExW, StringCbCopyNExW, StringCbCopyNW, StringCbCopyW, StringCbPrintfExW, StringCbPrintfW, StringCbVPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyA, StringCchCopyExW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthA, StringCchLengthW, StringCchPrintfA, StringCchPrintfExA, StringCchPrintfExW, StringCchPrintfW, StringCchVPrintfA, StringCchVPrintfExA, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
pub const ATTRIB: PUCHAR = 0x3C0usize as PUCHAR; // clang, blue.c
pub const CRTC: PUCHAR = 0x3D4usize as PUCHAR; // clang, blue.c
pub const CRTCDATA: PUCHAR = 0x3D5usize as PUCHAR; // clang, blue.c
pub const FOREGROUND_LIGHTGRAY: u32 = 7; // clang, blue.c
pub const GRAPHICS: PUCHAR = 0x3CEusize as PUCHAR; // clang, blue.c
pub const GRAPHICSDATA: PUCHAR = 0x3CFusize as PUCHAR; // clang, blue.c
pub const IOCTL_CONSOLE_DRAW: u32 = 0x1CA0C1; // clang, blue.c
pub const IOCTL_CONSOLE_FILL_OUTPUT_ATTRIBUTE: u32 = 0x1CA040; // clang, blue.c
pub const IOCTL_CONSOLE_FILL_OUTPUT_CHARACTER: u32 = 0x1CA080; // clang, blue.c
pub const IOCTL_CONSOLE_GET_CURSOR_INFO: u32 = 0x1C600C; // clang, blue.c
pub const IOCTL_CONSOLE_GET_MODE: u32 = 0x1C6014; // clang, blue.c
pub const IOCTL_CONSOLE_GET_SCREEN_BUFFER_INFO: u32 = 0x1C6004; // clang, blue.c
pub const IOCTL_CONSOLE_LOADFONT: u32 = 0x1CA101; // clang, blue.c
pub const IOCTL_CONSOLE_READ_OUTPUT_ATTRIBUTE: u32 = 0x1C6046; // clang, blue.c
pub const IOCTL_CONSOLE_READ_OUTPUT_CHARACTER: u32 = 0x1C6086; // clang, blue.c
pub const IOCTL_CONSOLE_RESET_SCREEN: u32 = 0x1CA000; // clang, blue.c
pub const IOCTL_CONSOLE_SET_CURSOR_INFO: u32 = 0x1CA010; // clang, blue.c
pub const IOCTL_CONSOLE_SET_MODE: u32 = 0x1CA018; // clang, blue.c
pub const IOCTL_CONSOLE_SET_SCREEN_BUFFER_INFO: u32 = 0x1CA008; // clang, blue.c
pub const IOCTL_CONSOLE_SET_TEXT_ATTRIBUTE: u32 = 0x1CA04C; // clang, blue.c
pub const IOCTL_CONSOLE_WRITE_OUTPUT_ATTRIBUTE: u32 = 0x1CA049; // clang, blue.c
pub const IOCTL_CONSOLE_WRITE_OUTPUT_CHARACTER: u32 = 0x1CA089; // clang, blue.c
pub const MISC: PUCHAR = 0x3C2usize as PUCHAR; // clang, blue.c
pub const PELDATA: PUCHAR = 0x3C9usize as PUCHAR; // clang, blue.c
pub const PELINDEX: PUCHAR = 0x3C8usize as PUCHAR; // clang, blue.c
pub const PELMASK: PUCHAR = 0x3C6usize as PUCHAR; // clang, blue.c
pub const SEQ: PUCHAR = 0x3C4usize as PUCHAR; // clang, blue.c
pub const SEQDATA: PUCHAR = 0x3C5usize as PUCHAR; // clang, blue.c
pub const STATUS: PUCHAR = 0x3DAusize as PUCHAR; // clang, blue.c
pub const TAG_BLUE: u32 = 0x45554C42; // clang, blue.c
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// DEVICE_EXTENSION: c2rust, blue.c
pub type DEVICE_EXTENSION = _DEVICE_EXTENSION;
// VGA_REGISTERS: c2rust, blue.c
pub type VGA_REGISTERS = _VGA_REGISTERS;
// _DEVICE_EXTENSION: c2rust, blue.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct _DEVICE_EXTENSION {
    pub VideoMemory: PUCHAR,
    pub VideoMemorySize: SIZE_T,
    pub Enabled: BOOLEAN,
    pub ScreenBuffer: PUCHAR,
    pub ScreenBufferSize: SIZE_T,
    pub CursorSize: ULONG,
    pub CursorVisible: INT,
    pub CharAttribute: USHORT,
    pub Mode: ULONG,
    pub ScanLines: UCHAR,
    pub Rows: USHORT,
    pub Columns: USHORT,
    pub CursorX: USHORT,
    pub CursorY: USHORT,
    pub FontBitfield: PUCHAR,
}
// _VGA_REGISTERS: c2rust, blue.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct _VGA_REGISTERS {
    pub CRT: [UCHAR; 24],
    pub Attribute: [UCHAR; 21],
    pub Graphics: [UCHAR; 9],
    pub Sequencer: [UCHAR; 5],
    pub Misc: UCHAR,
}
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
// ==== end port externs ====

// ==== port macros: SDK header macros, lowered from the preprocessor's own expansion (generated -- re-run `rustconv port macros`) ====
pub const ATTRC_INPST1: PUCHAR = 0x3dausize as PUCHAR;
pub const ATTRC_READREG: PUCHAR = 0x3c1usize as PUCHAR;
pub const ATTRC_WRITEREG: PUCHAR = 0x3c0usize as PUCHAR;
pub const CRTC_COMMAND: PUCHAR = 0x3d4usize as PUCHAR;
pub const CRTC_DATA: PUCHAR = 0x3d5usize as PUCHAR;
pub const GCT_COMMAND: PUCHAR = 0x3ceusize as PUCHAR;
pub const GCT_DATA: PUCHAR = 0x3cfusize as PUCHAR;
pub const SEQ_COMMAND: PUCHAR = 0x3c4usize as PUCHAR;
pub const SEQ_DATA: PUCHAR = 0x3c5usize as PUCHAR;
pub const STATUS_INVALID_BUFFER_SIZE: NTSTATUS = 0xc0000206usize as NTSTATUS;
// ==== end SDK header macros ====
pub use ddk_macros_rust::{Int32x32To64, KeQueryTickCount, MmGetSystemAddressForMdl, MmGetSystemAddressForMdlSafe, NdisAcquireSpinLock, NdisChainBufferAtBack, NdisChainBufferAtFront, NdisCopyLookaheadData, NdisDprAcquireSpinLock, NdisDprReleaseSpinLock, NdisGetFirstBufferFromPacketSafe, NdisInterruptLatched, NdisInterruptLevelSensitive, NdisQueryBuffer, NdisReleaseSpinLock, PROTOCOL_RESERVED_SIZE_IN_PACKET, RTL_CONSTANT_OBJECT_ATTRIBUTES};
