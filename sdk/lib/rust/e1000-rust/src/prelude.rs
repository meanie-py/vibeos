//! e1000-rust prelude — the single import surface the translated modules use.
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

// e1000's own debug.h (DBG=1 arm): `NDIS_DbgPrint(level, ("fmt", args))` prints when the level
// passes DebugTraceLevel's mask, after a "(file:line)" prefix. The translations flatten the
// inner parentheses. A macro, never an out-of-line fn: C-variadic DbgPrint cannot sit behind a
// fixed-arity Rust fn. (ndis-rust carries the same macro for ndis's identical debug.h.)
#[macro_export] macro_rules! NDIS_DbgPrint {
    ($t:expr, ( $($inner:tt)* ) $(,)?) => { $crate::NDIS_DbgPrint!($t, $($inner)*) };
    ($t:expr, $fmt:literal $(, $a:expr)* $(,)?) => {
        $crate::NDIS_DbgPrint!($t, concat!($fmt, "\0").as_ptr() $(, $a)*)
    };
    ($t:expr, $fmt:expr $(, $a:expr)* $(,)?) => {{
        let t = ($t) as u32;
        let lvl = $crate::e1000::ndis::DebugTraceLevel as u32;
        let mask = $crate::types::NORMAL_MASK as u32;
        if if t > mask { (lvl & t) > mask } else { (lvl & mask) >= t } {
            ntkrnl_sys::driver_ddk::DbgPrint(
                b"(%s:%d) \0".as_ptr() as *const i8,
                concat!(file!(), "\0").as_ptr() as *const i8,
                line!(),
            );
            ntkrnl_sys::driver_ddk::DbgPrint(($fmt) as *const i8 $(, $a)*);
        }
    }};
}
pub use crate::NDIS_DbgPrint;

// Names the translations use that no island carries.
pub type VOID = c_void;
// ndis.h: `#define NdisInterruptLatched Latched` (KINTERRUPT_MODE's enumerators).
pub const NdisInterruptLatched: NDIS_INTERRUPT_MODE = _KINTERRUPT_MODE_Latched;
pub const NdisInterruptLevelSensitive: NDIS_INTERRUPT_MODE = _KINTERRUPT_MODE_LevelSensitive;
// c2rust's name for E1000_ADAPTER's anonymous MulticastList element struct.
pub type C2RustUnnamed_123 = _E1000_ADAPTER__bindgen_ty_1;

// Hand ports of two items `port macros` bought, kept outside its block so a replay
// cannot undo them (nic.h / debug.h).
// E1000WriteIoUlong: IOADDR/IODATA, the indirect register pair at IoPort and IoPort+4.
// The bought body added 4 to a pointer as an integer.
#[inline]
pub unsafe fn E1000WriteIoUlong(Adapter: PE1000_ADAPTER, Address: ULONG, Value: ULONG) {
    let mut Dummy: ULONG = 0;
    NdisRawWritePortUlong!((*Adapter).IoPort as PULONG, Address);
    // The C reads STATUS between the two writes (a posted-write flush).
    E1000ReadUlong(Adapter, E1000_REG_STATUS, &mut Dummy);
    NdisRawWritePortUlong!((*Adapter).IoPort.add(4) as PULONG, Value);
}
// UNIMPLEMENTED_DBGBREAK(...): print, then break into the debugger.
#[allow(unused_macros)]
macro_rules! UNIMPLEMENTED_DBGBREAK {
    ($($arg:tt)*) => {{
        $crate::NDIS_DbgPrint!(MIN_TRACE, "UNIMPLEMENTED.\n");
        ntkrnl_sys::ke::DbgBreakPoint();
    }};
}
#[allow(unused_imports)]
pub(crate) use UNIMPLEMENTED_DBGBREAK;

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
pub use win32macros_rust::{ALIGN_UP, ARRAYSIZE, ARRAY_SIZE, ASSERT, C_ASSERT, CopyMemory, DBG_UNREFERENCED_LOCAL_VARIABLE, DPRINT1, DWLP_DLGPROC, DWLP_MSGRESULT, DWLP_USER, ERR_ON, FIELD_OFFSET, FIXME_ON, GET_WHEEL_DELTA_WPARAM, GET_X_LPARAM, GET_Y_LPARAM, GetAValue, GetBValue, GetGValue, GetRValue, HIBYTE, HIWORD, HRESULT_FROM_WIN32, HandleToUlong, IDE_SET_ADVANCE_PIO_MODE, IDE_SET_DEFAULT_PIO_MODE, IDE_SET_MWDMA_MODE, IDE_SET_SWDMA_MODE, IDE_SET_UDMA_MODE, INVALID_HANDLE_VALUE, IS_INTRESOURCE, InitializeObjectAttributes, IntToPtr, InterlockedExchange, IsContainerPartition, IsEqualGUID, IsFTPartition, IsRecognizedPartition, LANGIDFROMLCID, LOBYTE, LOWORD, LongToHandle, LongToPtr, MAKEINTATOM, MAKEINTRESOURCEA, MAKEINTRESOURCEW, MAKELANGID, MAKELCID, MAKELONG, MAKEWORD, MoveMemory, PAGE_ROUND_UP, PRIMARYLANGID, PtrToInt, PtrToLong, PtrToUint, PtrToUlong, RGB, RGBA, RTL_CONSTANT_STRING, RTL_CONTAINS_FIELD, RTL_FIELD_SIZE, RTL_NUMBER_OF, RTL_NUMBER_OF_FIELD, RTL_SIZEOF_THROUGH_FIELD, RtlCopyMemory, RtlFillMemory, RtlGetProcessHeap, RtlMoveMemory, RtlZeroMemory, SORTIDFROMLCID, SRB_STATUS, SUBLANGID, TRACE_ON, UIntToPtr, ULongToHandle, ULongToPtr, UNICODE_NULL, UNREFERENCED_PARAMETER, UintToPtr, UlongToHandle, UlongToPtr, WARN_ON, WINE_ERR, WINE_FIXME, WINE_TRACE, WINE_WARN, ZeroMemory, __HRESULT_FROM_WIN32, __p___argc, __p___argv, __p___wargv, __p__environ, __p__pgmptr, __p__wenviron, __p__wpgmptr, __wine_dprintf, __wine_is_debug_on, _countof, debugstr_a, debugstr_an, debugstr_w, debugstr_wn, heap_alloc, heap_alloc_zero, heap_calloc, heap_free, heap_realloc, iswalnum, iswalpha, iswcntrl, iswdigit, iswgraph, iswlower, iswprint, iswpunct, iswspace, iswupper, iswxdigit, wide_bytes, wine_dbgstr_a, wine_dbgstr_w};

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
pub const CmResourceShareShared: _CM_SHARE_DISPOSITION = _CM_SHARE_DISPOSITION_CmResourceShareShared; // island enum
pub const E1000_RCVBUF_INDEXMASK: _E1000_RCVBUF_SIZE = _E1000_RCVBUF_SIZE_E1000_RCVBUF_INDEXMASK; // island enum
pub const E1000_RCVBUF_RESERVED: _E1000_RCVBUF_SIZE = _E1000_RCVBUF_SIZE_E1000_RCVBUF_RESERVED; // island enum
pub const NDIS_STATUS_PENDING: NDIS_STATUS = 0x103; // ndis-rust/src/prelude.rs
pub const NDIS_STATUS_SUCCESS: NDIS_STATUS = 0; // ndis-rust/src/prelude.rs
pub const NdisHardwareStatusReady: _NDIS_HARDWARE_STATUS = _NDIS_HARDWARE_STATUS_NdisHardwareStatusReady; // island enum
pub const NdisInterfacePci: _NDIS_INTERFACE_TYPE = _NDIS_INTERFACE_TYPE_NdisInterfacePci; // island enum
pub const NdisMediaStateConnected: _NDIS_MEDIA_STATE = _NDIS_MEDIA_STATE_NdisMediaStateConnected; // island enum
pub const NdisMediaStateDisconnected: _NDIS_MEDIA_STATE = _NDIS_MEDIA_STATE_NdisMediaStateDisconnected; // island enum
pub const NdisMedium802_3: _NDIS_MEDIUM = _NDIS_MEDIUM_NdisMedium802_3; // island enum
pub const NdisPhysicalMedium802_3: _NDIS_PHYSICAL_MEDIUM = _NDIS_PHYSICAL_MEDIUM_NdisPhysicalMedium802_3; // island enum
pub const ScatterGatherListPacketInfo: _NDIS_PER_PACKET_INFO = _NDIS_PER_PACKET_INFO_ScatterGatherListPacketInfo; // island enum
// ==== end port resolve ====

// ==== port resolve: substrate helpers (generated) ====
pub use ntkrnl_sys::ke::DbgBreakPoint;
pub use ntkrnl_sys::driver_ddk::DbgPrint;
pub use ntkrnl_sys::ddk::IoAcquireRemoveLockEx;
pub use ntkrnl_sys::ddk::IoReleaseRemoveLockAndWaitEx;
pub use ntkrnl_sys::ddk::IoReleaseRemoveLockEx;
pub use ntkrnl_sys::inline::ke::_InterlockedExchange;
// ==== end substrate helpers ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, StringCbCatW, StringCbCopyExW, StringCbCopyNExW, StringCbCopyNW, StringCbCopyW, StringCbPrintfExW, StringCbPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyExW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthA, StringCchLengthW, StringCchPrintfExW, StringCchPrintfW, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
pub const NDIS_STATUS_BUFFER_TOO_SHORT: NDIS_STATUS = -1073676266; // clang, info.c
pub const NDIS_STATUS_FAILURE: NDIS_STATUS = -1073741823; // clang, ndis.c
pub const NDIS_STATUS_INVALID_DATA: NDIS_STATUS = -1073676267; // clang, hardware.c
pub const NDIS_STATUS_INVALID_LENGTH: NDIS_STATUS = -1073676268; // clang, info.c
pub const NDIS_STATUS_MEDIA_CONNECT: NDIS_STATUS = 0x4001000B; // clang, interrupt.c
pub const NDIS_STATUS_MEDIA_DISCONNECT: NDIS_STATUS = 0x4001000C; // clang, interrupt.c
pub const NDIS_STATUS_MULTICAST_FULL: NDIS_STATUS = -1073676279; // clang, info.c
pub const NDIS_STATUS_NOT_SUPPORTED: NDIS_STATUS = -1073741637; // clang, info.c
pub const NDIS_STATUS_RESOURCES: NDIS_STATUS = -1073741670; // clang, ndis.c
pub const NDIS_STATUS_UNSUPPORTED_MEDIA: NDIS_STATUS = -1073676263; // clang, ndis.c
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// E1000_RCVBUF_SIZE: c2rust, hardware.c
pub type E1000_RCVBUF_SIZE = _E1000_RCVBUF_SIZE;
// NDIS_MEDIUM: c2rust, info.c
pub type NDIS_MEDIUM = _NDIS_MEDIUM;
// _GENERIC_INFORMATION: c2rust, info.c
#[derive(Copy, Clone)]
#[repr(C)]
pub union _GENERIC_INFORMATION {
    pub Ushort: USHORT,
    pub Ulong: ULONG,
    pub Ulong64: ULONG64,
    pub Medium: NDIS_MEDIUM,
    pub PmCapabilities: NDIS_PNP_CAPABILITIES,
}
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
extern "system" {
    pub fn KeStallExecutionProcessor(MicroSeconds: ULONG); // hardware.c
}
pub use crate::prelude::KeStallExecutionProcessor as NdisStallExecution; // sdk alias
// ==== end port externs ====

// ==== port macros: SDK header macros, lowered from the preprocessor's own expansion (generated -- re-run `rustconv port macros`) ====
#[macro_export] macro_rules! NDIS_PER_PACKET_INFO_FROM_PACKET { ($p0:expr, $p1:expr $(,)?) => { (*(((((($p0) as PUCHAR)).wrapping_add(((*$p0).Private.NdisPacketOobOffset) as usize)).wrapping_add((core::mem::size_of::<NDIS_PACKET_OOB_DATA>()) as usize)) as PNDIS_PACKET_EXTENSION)).NdisPacketInfo[($p1) as usize] }; }
#[macro_export] macro_rules! NdisMEthIndicateReceive { ($p0:expr, $p1:expr, $p2:expr, $p3:expr, $p4:expr, $p5:expr, $p6:expr $(,)?) => { { ((*(($p0) as PNDIS_MINIPORT_BLOCK)).EthRxIndicateHandler).unwrap_unchecked()((((*(($p0) as PNDIS_MINIPORT_BLOCK)).__bindgen_anon_1.EthDB) as _), (($p1) as _), (($p2) as _), (($p2) as _), (($p3) as _), (($p4) as _), (($p5) as _), (($p6) as _)); } }; }
#[macro_export] macro_rules! NdisMEthIndicateReceiveComplete { ($p0:expr $(,)?) => { { ((*(($p0) as PNDIS_MINIPORT_BLOCK)).EthRxCompleteHandler).unwrap_unchecked()((((*(($p0) as PNDIS_MINIPORT_BLOCK)).__bindgen_anon_1.EthDB) as _)); } }; }
#[macro_export] macro_rules! NdisMIndicateStatus { ($p0:expr, $p1:expr, $p2:expr, $p3:expr $(,)?) => { ((*(($p0) as PNDIS_MINIPORT_BLOCK)).StatusHandler).unwrap_unchecked()((($p0) as _), (($p1) as _), (($p2) as _), (($p3) as _)) }; }
#[macro_export] macro_rules! NdisMIndicateStatusComplete { ($p0:expr $(,)?) => { ((*(($p0) as PNDIS_MINIPORT_BLOCK)).StatusCompleteHandler).unwrap_unchecked()((($p0) as _)) }; }
#[macro_export] macro_rules! NdisMInitializeWrapper { ($p0:expr, $p1:expr, $p2:expr, $p3:expr $(,)?) => { NdisInitializeWrapper((($p0) as _), (($p1) as _), (($p2) as _), (($p3) as _)) }; }
#[macro_export] macro_rules! NdisMSendComplete { ($p0:expr, $p1:expr, $p2:expr $(,)?) => { { ((*(($p0) as PNDIS_MINIPORT_BLOCK)).SendCompleteHandler).unwrap_unchecked()((($p0) as _), (($p1) as _), (($p2) as _)); } }; }
#[macro_export] macro_rules! NdisMoveMemory { ($p0:expr, $p1:expr, $p2:expr $(,)?) => { memcpy((($p0) as _), (($p1) as _), (($p2) as _)) }; }
#[macro_export] macro_rules! NdisRawWritePortUlong { ($p0:expr, $p1:expr $(,)?) => { WRITE_PORT_ULONG((($p0) as PULONG), (($p1) as ULONG)) }; }
#[macro_export] macro_rules! NdisReadRegisterUlong { ($p0:expr, $p1:expr $(,)?) => { { (*$p1) = ((core::ptr::read_volatile((($p0) as *mut ULONG))) as _); } }; }
#[macro_export] macro_rules! NdisWriteRegisterUlong { ($p0:expr, $p1:expr $(,)?) => { WRITE_REGISTER_ULONG((($p0) as _), (($p1) as _)) }; }
pub use crate::{NDIS_PER_PACKET_INFO_FROM_PACKET, NdisMEthIndicateReceive, NdisMEthIndicateReceiveComplete, NdisMIndicateStatus, NdisMIndicateStatusComplete, NdisMInitializeWrapper, NdisMSendComplete, NdisMoveMemory, NdisRawWritePortUlong, NdisReadRegisterUlong, NdisWriteRegisterUlong};
// ==== end SDK header macros ====

// ==== port macros: the module's own C macros (generated -- `rustconv port macros`) ====
// drivers/network/dd/e1000/nic.h:207: header-inline `E1000ReadUlong`
#[allow(dead_code)]
#[inline]
pub unsafe fn E1000ReadUlong(Adapter: PE1000_ADAPTER, Address: ULONG, Value: PULONG) {
    NdisReadRegisterUlong!(
        (*Adapter).IoBase.cast::<u8>().add(Address as usize).cast::<ULONG>(),
        Value
    )
}
// drivers/network/dd/e1000/nic.h:217: header-inline `E1000WriteUlong`
#[allow(dead_code)]
#[inline]
pub unsafe fn E1000WriteUlong(Adapter: PE1000_ADAPTER, Address: ULONG, Value: ULONG) {
    NdisWriteRegisterUlong!(
        (*Adapter).IoBase.cast::<u8>().add(Address as usize).cast::<ULONG>(),
        Value
    )
}
// drivers/network/dd/e1000/nic.h:241: header-inline `NICApplyInterruptMask`
#[allow(dead_code)]
#[inline]
pub unsafe fn NICApplyInterruptMask(Adapter: PE1000_ADAPTER) {
    E1000WriteUlong(Adapter, E1000_REG_IMS, ((*Adapter).InterruptMask) as ULONG)
}
// drivers/network/dd/e1000/nic.h:249: header-inline `NICDisableInterrupts`
#[allow(dead_code)]
#[inline]
pub unsafe fn NICDisableInterrupts(Adapter: PE1000_ADAPTER) {
    E1000WriteUlong(Adapter, E1000_REG_IMC, !0)
}
// ==== end port macros ====

// ==== port resolve: cross-TU definitions (generated) ====
pub use crate::e1000::nic_h::E1000_TAG;
pub use crate::e1000::interrupt::MiniportHandleInterrupt;
pub use crate::e1000::interrupt::MiniportISR;
pub use crate::e1000::info::MiniportQueryInformation;
pub use crate::e1000::send::MiniportSend;
pub use crate::e1000::info::MiniportSetInformation;
pub use crate::e1000::hardware::NICAllocateIoResources;
pub use crate::e1000::hardware::NICApplyPacketFilter;
pub use crate::e1000::hardware::NICDisableTxRx;
pub use crate::e1000::hardware::NICEnableTxRx;
pub use crate::e1000::hardware::NICGetPermanentMacAddress;
pub use crate::e1000::hardware::NICInitializeAdapterResources;
pub use crate::e1000::hardware::NICPowerOn;
pub use crate::e1000::hardware::NICRecognizeHardware;
pub use crate::e1000::hardware::NICRegisterInterrupts;
pub use crate::e1000::hardware::NICReleaseIoResources;
pub use crate::e1000::hardware::NICSoftReset;
pub use crate::e1000::hardware::NICUnregisterInterrupts;
pub use crate::e1000::hardware::NICUpdateLinkStatus;
pub use crate::e1000::hardware::NICUpdateMulticastList;
pub use crate::e1000::debug::Oid2Str;
// ==== end cross-TU definitions ====
