//! ndis-rust prelude — the single import surface the translated modules use.
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

// ndis's own debug.h (DBG=1 arm): `NDIS_DbgPrint(level, ("fmt", args))` prints when the level
// passes DebugTraceLevel's mask. The translations flatten the inner parentheses into
// `NDIS_DbgPrint(level, fmt, args...)`. A macro, never an out-of-line fn: a C-variadic
// DbgPrint cannot sit behind a fixed-arity Rust fn anyway.
#[macro_export] macro_rules! NDIS_DbgPrint {
    // C's own shape, `NDIS_DbgPrint(level, ("fmt", args))`, kept by some translations.
    ($t:expr, ( $($inner:tt)* ) $(,)?) => { $crate::NDIS_DbgPrint!($t, $($inner)*) };
    // A bare `"..."` (no `.as_ptr()`): the NUL the C literal carried.
    ($t:expr, $fmt:literal $(, $a:expr)* $(,)?) => {
        $crate::NDIS_DbgPrint!($t, concat!($fmt, "\0").as_ptr() $(, $a)*)
    };
    ($t:expr, $fmt:expr $(, $a:expr)* $(,)?) => {{
        let t = ($t) as u32;
        let lvl = $crate::ndis::main::DebugTraceLevel as u32;
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
#[macro_export] macro_rules! ASSERT_IRQL {
    ($x:expr $(,)?) => { ntkrnl_sys::ASSERT!(ntkrnl_sys::ke::KeGetCurrentIrql() <= ($x) as u8) };
}
pub use crate::{ASSERT_IRQL, NDIS_DbgPrint};

// ---- header macros `port macros` refuses (per-arch expansions, statement bodies) ----------
// Int32x32To64: `(LONGLONG)(LONG)a * (LONGLONG)(LONG)b` on amd64/arm64; i386's `__emul` is
// the same signed 32x32->64 product.
#[macro_export] macro_rules! Int32x32To64 {
    ($a:expr, $b:expr $(,)?) => { ((($a) as i32 as i64) * (($b) as i32 as i64)) };
}
// KeQueryTickCount: the 64-bit headers read KUSER_SHARED_DATA.TickCount
// (0xFFFFF78000000000 + 0x320); i386's calls the kernel export.
#[macro_export] macro_rules! KeQueryTickCount {
    ($p:expr $(,)?) => { $crate::prelude::ke_query_tick_count(($p) as *mut LARGE_INTEGER) };
}
#[inline(always)]
pub unsafe fn ke_query_tick_count(p: *mut LARGE_INTEGER) {
    #[cfg(not(target_arch = "x86"))] // nolint: cfg-vs-c (the header's own per-arch arms)
    {
        *(p as *mut u64) = core::ptr::read_volatile((0xFFFFF78000000000u64 + 0x320) as *const u64);
    }
    #[cfg(target_arch = "x86")] // nolint: cfg-vs-c (the header's own per-arch arms)
    {
        extern "stdcall" {
            fn KeQueryTickCount(TickCount: *mut LARGE_INTEGER);
        }
        KeQueryTickCount(p);
    }
}
// kefuncs.h: `(CallbackRecord)->State = BufferEmpty;`
#[macro_export] macro_rules! KeInitializeCallbackRecord {
    ($r:expr $(,)?) => {{ (*($r)).State = BufferEmpty as _; }};
}
// exfuncs.h: the SList pair ignores its Lock argument and names, per the header's own arms,
// the fastcall InterlockedPop/PushEntrySList exports on i386 and ExpInterlocked* elsewhere
// (ntoskrnl.spec: those are the only SList exports each arch has).
#[macro_export] macro_rules! ExInterlockedPopEntrySList {
    ($h:expr, $lock:expr $(,)?) => { $crate::prelude::ex_interlocked_pop_entry_slist(($h) as _) };
}
#[macro_export] macro_rules! ExInterlockedPushEntrySList {
    ($h:expr, $e:expr, $lock:expr $(,)?) => {
        $crate::prelude::ex_interlocked_push_entry_slist(($h) as _, ($e) as _)
    };
}
#[inline(always)]
pub unsafe fn ex_interlocked_pop_entry_slist(h: PSLIST_HEADER) -> PSLIST_ENTRY {
    #[cfg(target_arch = "x86")] // nolint: cfg-vs-c (the header's own per-arch arms)
    {
        crate::types::InterlockedPopEntrySList(h) as _
    }
    #[cfg(not(target_arch = "x86"))] // nolint: cfg-vs-c (the header's own per-arch arms)
    {
        crate::types::ExpInterlockedPopEntrySList(h) as _
    }
}
#[inline(always)]
pub unsafe fn ex_interlocked_push_entry_slist(h: PSLIST_HEADER, e: PSLIST_ENTRY) -> PSLIST_ENTRY {
    #[cfg(target_arch = "x86")] // nolint: cfg-vs-c (the header's own per-arch arms)
    {
        crate::types::InterlockedPushEntrySList(h, e as _) as _
    }
    #[cfg(not(target_arch = "x86"))] // nolint: cfg-vs-c (the header's own per-arch arms)
    {
        crate::types::ExpInterlockedPushEntrySList(h, e as _) as _
    }
}
// ndissys.h: `{{(q), (q) >> 32}}` is LowPart/HighPart of the same 64-bit value.
#[macro_export] macro_rules! RTL_CONSTANT_LARGE_INTEGER {
    ($q:expr $(,)?) => { LARGE_INTEGER { QuadPart: ($q) as i64 } };
}
// exfuncs.h: three plain stores. Hand-written because the lowered form converts the routine
// with `as _`, which cannot turn the `Some(f)` a caller passes into the `Option<fn>` field;
// a plain assignment coerces it.
#[macro_export] macro_rules! ExInitializeWorkItem {
    ($item:expr, $routine:expr, $param:expr $(,)?) => {{
        (*($item)).WorkerRoutine = $routine;
        (*($item)).Parameter = ($param) as _;
        (*($item)).List.Flink = core::ptr::null_mut();
    }};
}
pub use crate::{
    ExInitializeWorkItem, ExInterlockedPopEntrySList, ExInterlockedPushEntrySList, Int32x32To64,
    KeInitializeCallbackRecord, KeQueryTickCount, RTL_CONSTANT_LARGE_INTEGER,
};
pub use ntkrnl_sys::inline::common::KeNumberProcessorsValue;
// i386 spells these as macros over the fastcall Exf* exports; driver_ddk bridges both forms.
pub use ntkrnl_sys::driver_ddk::{
    ExInterlockedAddUlong, ExInterlockedInsertHeadList, ExInterlockedInsertTailList,
    ExInterlockedRemoveHeadList,
};

/// ndis.h's `static __inline NdisQueryPacket`. Platform-header inlines are normally the -sys
/// crate's, but the NDIS types exist only in this crate's island; transcribed from the header.
#[inline]
pub unsafe fn NdisQueryPacket(
    Packet: PNDIS_PACKET,
    PhysicalBufferCount: PUINT,
    BufferCount: PUINT,
    FirstBuffer: *mut PNDIS_BUFFER,
    TotalPacketLength: PUINT,
) {
    if !FirstBuffer.is_null() {
        *FirstBuffer = (*Packet).Private.Head;
    }
    if !TotalPacketLength.is_null() || !BufferCount.is_null() || !PhysicalBufferCount.is_null() {
        if (*Packet).Private.ValidCounts == 0 {
            let mut offset: UINT = 0;
            let mut length: UINT = 0;
            let mut physical: UINT = 0;
            let mut total: UINT = 0;
            let mut count: UINT = 0;
            let mut buffer = (*Packet).Private.Head;
            while !buffer.is_null() {
                physical += crate::ndis::buffer::NDIS_BUFFER_TO_SPAN_PAGES(buffer);
                crate::ndis::buffer::NdisQueryBufferOffset(buffer, &mut offset, &mut length);
                total += length;
                count += 1;
                buffer = (*buffer).Next;
            }
            (*Packet).Private.PhysicalCount = physical;
            (*Packet).Private.TotalLength = total;
            (*Packet).Private.Count = count;
            (*Packet).Private.ValidCounts = TRUE as _;
        }
        if !PhysicalBufferCount.is_null() {
            *PhysicalBufferCount = (*Packet).Private.PhysicalCount;
        }
        if !BufferCount.is_null() {
            *BufferCount = (*Packet).Private.Count;
        }
        if !TotalPacketLength.is_null() {
            *TotalPacketLength = (*Packet).Private.TotalLength;
        }
    }
}

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
pub use win32macros_rust::{ALIGN_UP, ARRAYSIZE, ARRAY_SIZE, ASSERT, C_ASSERT, CopyMemory, DBG_UNREFERENCED_LOCAL_VARIABLE, DPRINT1, DWLP_DLGPROC, DWLP_MSGRESULT, DWLP_USER, FIELD_OFFSET, GET_WHEEL_DELTA_WPARAM, GET_X_LPARAM, GET_Y_LPARAM, GetAValue, GetBValue, GetGValue, GetRValue, HIBYTE, HIWORD, HandleToUlong, IDE_SET_ADVANCE_PIO_MODE, IDE_SET_DEFAULT_PIO_MODE, IDE_SET_MWDMA_MODE, IDE_SET_SWDMA_MODE, IDE_SET_UDMA_MODE, INVALID_HANDLE_VALUE, IS_INTRESOURCE, InitializeObjectAttributes, IntToPtr, IsContainerPartition, IsEqualGUID, IsFTPartition, IsRecognizedPartition, LANGIDFROMLCID, LOBYTE, LOWORD, LongToHandle, LongToPtr, MAKEINTATOM, MAKEINTRESOURCEA, MAKEINTRESOURCEW, MAKELANGID, MAKELCID, MAKELONG, MAKEWORD, MoveMemory, PAGE_ROUND_UP, PRIMARYLANGID, PtrToInt, PtrToLong, PtrToUint, PtrToUlong, RGB, RGBA, RTL_CONSTANT_STRING, RTL_CONTAINS_FIELD, RTL_FIELD_SIZE, RTL_NUMBER_OF, RTL_NUMBER_OF_FIELD, RTL_SIZEOF_THROUGH_FIELD, RtlCopyMemory, RtlFillMemory, RtlGetProcessHeap, RtlMoveMemory, RtlZeroMemory, SORTIDFROMLCID, SRB_STATUS, SUBLANGID, UIntToPtr, ULongToHandle, ULongToPtr, UNICODE_NULL, UNREFERENCED_PARAMETER, UintToPtr, UlongToHandle, UlongToPtr, ZeroMemory, _countof, debugstr_a, debugstr_an, debugstr_w, debugstr_wn, heap_alloc, heap_alloc_zero, heap_calloc, heap_free, heap_realloc, iswalnum, iswalpha, iswcntrl, iswdigit, iswgraph, iswlower, iswprint, iswpunct, iswspace, iswupper, iswxdigit, wide_bytes};

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

// ==== port resolve: tree consensus (generated -- re-run `rustconv port resolve`) ====
pub const BufferEmpty: _KBUGCHECK_BUFFER_DUMP_STATE = _KBUGCHECK_BUFFER_DUMP_STATE_BufferEmpty; // island enum
pub const CriticalWorkQueue: _WORK_QUEUE_TYPE = _WORK_QUEUE_TYPE_CriticalWorkQueue; // island enum
pub const DeallocateObjectKeepRegisters: _IO_ALLOCATION_ACTION = _IO_ALLOCATION_ACTION_DeallocateObjectKeepRegisters; // island enum
pub const DelayedWorkQueue: _WORK_QUEUE_TYPE = _WORK_QUEUE_TYPE_DelayedWorkQueue; // island enum
pub const DevicePropertyAddress: DEVICE_REGISTRY_PROPERTY = DEVICE_REGISTRY_PROPERTY_DevicePropertyAddress; // island enum
pub const DevicePropertyBusNumber: DEVICE_REGISTRY_PROPERTY = DEVICE_REGISTRY_PROPERTY_DevicePropertyBusNumber; // island enum
pub const DevicePropertyDriverKeyName: DEVICE_REGISTRY_PROPERTY = DEVICE_REGISTRY_PROPERTY_DevicePropertyDriverKeyName; // island enum
pub const DevicePropertyLegacyBusType: DEVICE_REGISTRY_PROPERTY = DEVICE_REGISTRY_PROPERTY_DevicePropertyLegacyBusType; // island enum
pub const EisaConfiguration: _BUS_DATA_TYPE = _BUS_DATA_TYPE_EisaConfiguration; // island enum
pub const FileStandardInformation: _FILE_INFORMATION_CLASS = _FILE_INFORMATION_CLASS_FileStandardInformation; // island enum
pub const InterfaceTypeUndefined: _INTERFACE_TYPE = _INTERFACE_TYPE_InterfaceTypeUndefined; // island enum
pub const KeepObject: _IO_ALLOCATION_ACTION = _IO_ALLOCATION_ACTION_KeepObject; // island enum
pub const KernelMode: _MODE = _MODE_KernelMode; // island enum
pub const KeyBasicInformation: _KEY_INFORMATION_CLASS = _KEY_INFORMATION_CLASS_KeyBasicInformation; // island enum
pub const KeyValuePartialInformation: _KEY_VALUE_INFORMATION_CLASS = _KEY_VALUE_INFORMATION_CLASS_KeyValuePartialInformation; // island enum
pub const MmCached: _MEMORY_CACHING_TYPE = _MEMORY_CACHING_TYPE_MmCached; // island enum
pub const MmNonCached: _MEMORY_CACHING_TYPE = _MEMORY_CACHING_TYPE_MmNonCached; // island enum
pub const NdisEnvironmentWindowsNt: _NDIS_ENVIRONMENT_TYPE = _NDIS_ENVIRONMENT_TYPE_NdisEnvironmentWindowsNt; // island enum
pub const NdisInterfaceIsa: _NDIS_INTERFACE_TYPE = _NDIS_INTERFACE_TYPE_NdisInterfaceIsa; // island enum
pub const NdisInterfacePci: _NDIS_INTERFACE_TYPE = _NDIS_INTERFACE_TYPE_NdisInterfacePci; // island enum
pub const NdisMaxWorkItems: _NDIS_WORK_ITEM_TYPE = _NDIS_WORK_ITEM_TYPE_NdisMaxWorkItems; // island enum
pub const NdisMedium1394: _NDIS_MEDIUM = _NDIS_MEDIUM_NdisMedium1394; // island enum
pub const NdisMedium802_3: _NDIS_MEDIUM = _NDIS_MEDIUM_NdisMedium802_3; // island enum
pub const NdisMedium802_5: _NDIS_MEDIUM = _NDIS_MEDIUM_NdisMedium802_5; // island enum
pub const NdisMediumArcnet878_2: _NDIS_MEDIUM = _NDIS_MEDIUM_NdisMediumArcnet878_2; // island enum
pub const NdisMediumArcnetRaw: _NDIS_MEDIUM = _NDIS_MEDIUM_NdisMediumArcnetRaw; // island enum
pub const NdisMediumAtm: _NDIS_MEDIUM = _NDIS_MEDIUM_NdisMediumAtm; // island enum
pub const NdisMediumBpc: _NDIS_MEDIUM = _NDIS_MEDIUM_NdisMediumBpc; // island enum
pub const NdisMediumCoWan: _NDIS_MEDIUM = _NDIS_MEDIUM_NdisMediumCoWan; // island enum
pub const NdisMediumDix: _NDIS_MEDIUM = _NDIS_MEDIUM_NdisMediumDix; // island enum
pub const NdisMediumFddi: _NDIS_MEDIUM = _NDIS_MEDIUM_NdisMediumFddi; // island enum
pub const NdisMediumIrda: _NDIS_MEDIUM = _NDIS_MEDIUM_NdisMediumIrda; // island enum
pub const NdisMediumLocalTalk: _NDIS_MEDIUM = _NDIS_MEDIUM_NdisMediumLocalTalk; // island enum
pub const NdisMediumMax: _NDIS_MEDIUM = _NDIS_MEDIUM_NdisMediumMax; // island enum
pub const NdisMediumWan: _NDIS_MEDIUM = _NDIS_MEDIUM_NdisMediumWan; // island enum
pub const NdisMediumWirelessWan: _NDIS_MEDIUM = _NDIS_MEDIUM_NdisMediumWirelessWan; // island enum
pub const NdisParameterBinary: _NDIS_PARAMETER_TYPE = _NDIS_PARAMETER_TYPE_NdisParameterBinary; // island enum
pub const NdisParameterHexInteger: _NDIS_PARAMETER_TYPE = _NDIS_PARAMETER_TYPE_NdisParameterHexInteger; // island enum
pub const NdisParameterInteger: _NDIS_PARAMETER_TYPE = _NDIS_PARAMETER_TYPE_NdisParameterInteger; // island enum
pub const NdisParameterMultiString: _NDIS_PARAMETER_TYPE = _NDIS_PARAMETER_TYPE_NdisParameterMultiString; // island enum
pub const NdisParameterString: _NDIS_PARAMETER_TYPE = _NDIS_PARAMETER_TYPE_NdisParameterString; // island enum
pub const NdisPnPDeviceAdded: _NDIS_PNP_DEVICE_STATE = _NDIS_PNP_DEVICE_STATE_NdisPnPDeviceAdded; // island enum
pub const NdisPnPDeviceStarted: _NDIS_PNP_DEVICE_STATE = _NDIS_PNP_DEVICE_STATE_NdisPnPDeviceStarted; // island enum
pub const NdisPnPDeviceStopped: _NDIS_PNP_DEVICE_STATE = _NDIS_PNP_DEVICE_STATE_NdisPnPDeviceStopped; // island enum
pub const NdisProcessorX86: _NDIS_PROCESSOR_TYPE = _NDIS_PROCESSOR_TYPE_NdisProcessorX86; // island enum
pub const NdisRequestQueryInformation: _NDIS_REQUEST_TYPE = _NDIS_REQUEST_TYPE_NdisRequestQueryInformation; // island enum
pub const NdisRequestSetInformation: _NDIS_REQUEST_TYPE = _NDIS_REQUEST_TYPE_NdisRequestSetInformation; // island enum
pub const NdisWorkItemMiniportCallback: _NDIS_WORK_ITEM_TYPE = _NDIS_WORK_ITEM_TYPE_NdisWorkItemMiniportCallback; // island enum
pub const NdisWorkItemRequest: _NDIS_WORK_ITEM_TYPE = _NDIS_WORK_ITEM_TYPE_NdisWorkItemRequest; // island enum
pub const NdisWorkItemResetInProgress: _NDIS_WORK_ITEM_TYPE = _NDIS_WORK_ITEM_TYPE_NdisWorkItemResetInProgress; // island enum
pub const NdisWorkItemResetRequested: _NDIS_WORK_ITEM_TYPE = _NDIS_WORK_ITEM_TYPE_NdisWorkItemResetRequested; // island enum
pub const NdisWorkItemReturnPackets: _NDIS_WORK_ITEM_TYPE = _NDIS_WORK_ITEM_TYPE_NdisWorkItemReturnPackets; // island enum
pub const NdisWorkItemSend: _NDIS_WORK_ITEM_TYPE = _NDIS_WORK_ITEM_TYPE_NdisWorkItemSend; // island enum
pub const NdisWorkItemSendLoopback: _NDIS_WORK_ITEM_TYPE = _NDIS_WORK_ITEM_TYPE_NdisWorkItemSendLoopback; // island enum
pub const NetEventBindsComplete: _NET_PNP_EVENT_CODE = _NET_PNP_EVENT_CODE_NetEventBindsComplete; // island enum
pub const NetEventCancelRemoveDevice: _NET_PNP_EVENT_CODE = _NET_PNP_EVENT_CODE_NetEventCancelRemoveDevice; // island enum
pub const NetEventQueryPower: _NET_PNP_EVENT_CODE = _NET_PNP_EVENT_CODE_NetEventQueryPower; // island enum
pub const NetEventQueryRemoveDevice: _NET_PNP_EVENT_CODE = _NET_PNP_EVENT_CODE_NetEventQueryRemoveDevice; // island enum
pub const NetEventSetPower: _NET_PNP_EVENT_CODE = _NET_PNP_EVENT_CODE_NetEventSetPower; // island enum
pub const NonPagedPool: _POOL_TYPE = _POOL_TYPE_NonPagedPool; // island enum
pub const PCIConfiguration: _BUS_DATA_TYPE = _BUS_DATA_TYPE_PCIConfiguration; // island enum
pub const PCMCIAConfiguration: _BUS_DATA_TYPE = _BUS_DATA_TYPE_PCMCIAConfiguration; // island enum
pub const PacketCancelId: _NDIS_PER_PACKET_INFO = _NDIS_PER_PACKET_INFO_PacketCancelId; // island enum
pub const PagedPool: _POOL_TYPE = _POOL_TYPE_PagedPool; // island enum
pub const STATUS_BUFFER_OVERFLOW: i32 = 0x80000005_u32 as i32; // atapi-rust/src/prelude.rs
pub const STATUS_BUFFER_TOO_SMALL: i32 = 0xC000_0023u32 as i32; // atapi-rust/src/prelude.rs
pub const STATUS_INSUFFICIENT_RESOURCES: NTSTATUS = 0xC000_009Au32 as NTSTATUS; // atapi-rust/src/prelude.rs
pub const STATUS_INVALID_DEVICE_REQUEST: NTSTATUS = 0xC000_0010u32 as NTSTATUS; // atapi-rust/src/prelude.rs
pub const STATUS_NOT_IMPLEMENTED: NTSTATUS = 0xC0000002u32 as NTSTATUS; // atapi-rust/src/prelude.rs
pub const STATUS_NOT_SUPPORTED: i32 = 0xC00000BB_u32 as i32; // atapi-rust/src/prelude.rs
pub const STATUS_NO_MEMORY: NTSTATUS = 0xC0000017u32 as NTSTATUS; // diskpart-rust/src/prelude.rs
pub const STATUS_PENDING: NTSTATUS = 0x00000103; // atapi-rust/src/prelude.rs
pub const STATUS_SUCCESS: i32 = 0; // atapi-rust/src/prelude.rs
pub const STATUS_UNSUCCESSFUL: NTSTATUS = 0xC000_0001u32 as NTSTATUS; // atapi-rust/src/prelude.rs
pub const ScatterGatherListPacketInfo: _NDIS_PER_PACKET_INFO = _NDIS_PER_PACKET_INFO_ScatterGatherListPacketInfo; // island enum
extern "C" {
    pub fn wcscat(dst: *mut u16, src: *const u16) -> *mut u16; // advapi32-rust/src/wine/cred.rs
    pub fn wcscpy(dst: *mut u16, src: *const u16) -> *mut u16; // advapi32-rust/src/prelude.rs
    pub fn wcsncat(dst: *mut u16, src: *const u16, n: usize) -> *mut u16; // cnt-rust/src/cabi/generated.rs
    pub fn wcsncmp(a: *const u16, b: *const u16, n: usize) -> c_int; // desk-rust/src/prelude.rs
    pub fn wcsncpy(d: *mut u16, s: *const u16, n: usize) -> *mut u16; // kernel32-rust/src/prelude.rs
}
// ==== end port resolve ====

// ==== port resolve: substrate helpers (generated) ====
pub use ntkrnl_sys::mm::ADDRESS_AND_SIZE_TO_SPAN_PAGES;
pub use ntkrnl_sys::ke::DbgBreakPoint;
pub use ntkrnl_sys::driver_ddk::DbgPrint;
pub use ntkrnl_sys::ddk::ExAllocatePoolWithTag;
pub use ntkrnl_sys::ddk::ExFreePoolWithTag;
#[cfg(not(target_arch = "x86"))]
pub use ntkrnl_sys::ddk::ExpInterlockedPopEntrySList;
#[cfg(not(target_arch = "x86"))]
pub use ntkrnl_sys::ddk::ExpInterlockedPushEntrySList;
pub use ntkrnl_sys::inline::common::InitializeListHead;
pub use ntkrnl_sys::ddk::IoAcquireRemoveLockEx;
pub use ntkrnl_sys::driver_ddk::IoCallDriver;
pub use ntkrnl_sys::driver_ddk::IoCompleteRequest;
pub use ntkrnl_sys::ddk::IoReleaseRemoveLockAndWaitEx;
pub use ntkrnl_sys::ddk::IoReleaseRemoveLockEx;
pub use ntkrnl_sys::inline::common::IsListEmpty;
pub use ntkrnl_sys::ddk::KeBugCheckEx;
pub use ntkrnl_sys::ddk::KeClearEvent;
pub use ntkrnl_sys::inline::common::KeFlushIoBuffers;
#[cfg(not(target_arch = "x86_64"))]
pub use ntkrnl_sys::ddk::KeGetCurrentIrql;
#[cfg(target_arch = "x86_64")]
pub use ntkrnl_sys::ke::KeGetCurrentIrql;
pub use ntkrnl_sys::inline::ke::KeGetCurrentProcessorNumber;
#[cfg(target_arch = "x86")]
pub use ntkrnl_sys::ddk::KeQuerySystemTime;
#[cfg(not(target_arch = "x86"))]
pub use ntkrnl_sys::inline::ke::KeQuerySystemTime;
pub use ntkrnl_sys::ddk::KeSetEvent;
pub use ntkrnl_sys::inline::mm::MmGetSystemAddressForMdlSafe;
pub use ntkrnl_sys::ddk::MmMapLockedPages;
pub use ntkrnl_sys::driver_ddk::PoCallDriver;
pub use ntkrnl_sys::driver_ddk::PoStartNextPowerIrp;
pub use ntkrnl_sys::portio::READ_PORT_UCHAR;
pub use ntkrnl_sys::portio::READ_PORT_ULONG;
pub use ntkrnl_sys::portio::READ_PORT_USHORT;
pub use ntkrnl_sys::inline::common::RemoveEntryList;
pub use ntkrnl_sys::ddk::RtlAssert;
pub use ntkrnl_sys::portio::WRITE_PORT_UCHAR;
pub use ntkrnl_sys::portio::WRITE_PORT_ULONG;
pub use ntkrnl_sys::portio::WRITE_PORT_USHORT;
pub use ntkrnl_sys::driver_ddk::wcslen;
// ==== end substrate helpers ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, StringCbCatW, StringCbCopyNExW, StringCbCopyW, StringCbPrintfExW, StringCbPrintfW, StringCchCatW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthW, StringCchPrintfW, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
pub const ETH_LENGTH_OF_ADDRESS: u32 = 6; // clang, efilter.c
pub const IOCTL_NDIS_QUERY_GLOBAL_STATS: u32 = 0x170002; // clang, miniport.c
pub const IOCTL_NDIS_RESERVED7: u32 = 0x17003E; // clang, miniport.c
pub const NDIS_DMA_32BITS: NDIS_DMA_SIZE = 1; // clang, io.c
pub const NDIS_DMA_64BITS: NDIS_DMA_SIZE = 2; // clang, io.c
pub const NDIS_STATUS_ADAPTER_NOT_FOUND: NDIS_STATUS = -1073676282; // clang, protocol.c
pub const NDIS_STATUS_ALREADY_MAPPED: NDIS_STATUS = -1073676259; // clang, misc.c
pub const NDIS_STATUS_BAD_CHARACTERISTICS: NDIS_STATUS = -1073676283; // clang, miniport.c
pub const NDIS_STATUS_BAD_VERSION: NDIS_STATUS = -1073676284; // clang, miniport.c
pub const NDIS_STATUS_ERROR_READING_FILE: NDIS_STATUS = -1073676260; // clang, misc.c
pub const NDIS_STATUS_FAILURE: NDIS_STATUS = -1073741823; // clang, 40stubs.c
pub const NDIS_STATUS_FILE_NOT_FOUND: NDIS_STATUS = -1073676261; // clang, misc.c
pub const NDIS_STATUS_NOT_SUPPORTED: NDIS_STATUS = -1073741637; // clang, config.c
pub const NDIS_STATUS_PENDING: NDIS_STATUS = 0x103; // clang, memory.c
pub const NDIS_STATUS_RESET_END: NDIS_STATUS = 0x40010005; // clang, miniport.c
pub const NDIS_STATUS_RESET_START: NDIS_STATUS = 0x40010004; // clang, miniport.c
pub const NDIS_STATUS_RESOURCES: NDIS_STATUS = -1073741670; // clang, buffer.c
pub const NDIS_STATUS_RESOURCE_CONFLICT: NDIS_STATUS = -1073676258; // clang, io.c
pub const NDIS_STATUS_SUCCESS: NDIS_STATUS = 0; // clang, buffer.c
pub const NDIS_STATUS_UNSUPPORTED_MEDIA: NDIS_STATUS = -1073676263; // clang, miniport.c
pub const NDIS_TAG: u32 = 0x5349444E; // clang, protocol.c
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// INTERFACE_TYPE: c2rust, io.c
pub type INTERFACE_TYPE = _INTERFACE_TYPE;
// IO_ALLOCATION_ACTION: c2rust, io.c
pub type IO_ALLOCATION_ACTION = _IO_ALLOCATION_ACTION;
// MM_PAGE_PRIORITY: c2rust, buffer.c
pub type MM_PAGE_PRIORITY = _MM_PAGE_PRIORITY;
// NDIS_DEVICE_POWER_STATE: c2rust, protocol.c
pub type NDIS_DEVICE_POWER_STATE = _NDIS_DEVICE_POWER_STATE;
// NDIS_INTERFACE_TYPE: c2rust, hardware.c
pub type NDIS_INTERFACE_TYPE = _NDIS_INTERFACE_TYPE;
// NDIS_MEDIUM: c2rust, 50stubs.c
pub type NDIS_MEDIUM = _NDIS_MEDIUM;
// NDIS_PARAMETER_TYPE: c2rust, config.c
pub type NDIS_PARAMETER_TYPE = _NDIS_PARAMETER_TYPE;
// NDIS_WORK_ITEM_TYPE: c2rust, miniport.c
pub type NDIS_WORK_ITEM_TYPE = _NDIS_WORK_ITEM_TYPE;
// NET_PNP_EVENT: c2rust, protocol.c
pub type NET_PNP_EVENT = _NET_PNP_EVENT;
// NET_PNP_EVENT_CODE: c2rust, protocol.c
pub type NET_PNP_EVENT_CODE = _NET_PNP_EVENT_CODE;
// PETH_FILTER: c2rust, efilter.c
pub type PETH_FILTER = *mut _X_FILTER;
// PFDDI_FILTER: c2rust, 30stubs.c
pub type PFDDI_FILTER = *mut _X_FILTER;
// PTR_FILTER: c2rust, 30stubs.c
pub type PTR_FILTER = *mut _X_FILTER;
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
// ==== end port externs ====

// ==== port macros: SDK header macros, lowered from the preprocessor's own expansion (generated -- re-run `rustconv port macros`) ====
#[macro_export] macro_rules! MmGetMdlByteCount { ($p0:expr $(,)?) => { (*$p0).ByteCount }; }
#[macro_export] macro_rules! MmGetMdlByteOffset { ($p0:expr $(,)?) => { (*$p0).ByteOffset }; }
#[macro_export] macro_rules! MmGetMdlVirtualAddress { ($p0:expr $(,)?) => { ((((((*$p0).StartVa) as PCHAR)).wrapping_add(((*$p0).ByteOffset) as usize)) as PVOID) }; }
#[macro_export] macro_rules! MmGetSystemAddressForMdl { ($p0:expr $(,)?) => { (if (((*$p0).MdlFlags & (0x0001 | 0x0004))) as isize != 0 { (*$p0).MappedSystemVa } else { MmMapLockedPages((($p0) as _), ((KernelMode) as _)) }) }; }
#[macro_export] macro_rules! NDIS_GET_PACKET_CANCEL_ID { ($p0:expr $(,)?) => { (*(((((($p0) as PUCHAR)).wrapping_add(((*$p0).Private.NdisPacketOobOffset) as usize)).wrapping_add((core::mem::size_of::<NDIS_PACKET_OOB_DATA>()) as usize)) as PNDIS_PACKET_EXTENSION)).NdisPacketInfo[(PacketCancelId) as usize] }; }
#[macro_export] macro_rules! NDIS_GET_PACKET_HEADER_SIZE { ($p0:expr $(,)?) => { (*((((($p0) as PUCHAR)).wrapping_add(((*$p0).Private.NdisPacketOobOffset) as usize)) as PNDIS_PACKET_OOB_DATA)).HeaderSize }; }
#[macro_export] macro_rules! NDIS_GET_PACKET_STATUS { ($p0:expr $(,)?) => { (*((((($p0) as PUCHAR)).wrapping_add(((*$p0).Private.NdisPacketOobOffset) as usize)) as PNDIS_PACKET_OOB_DATA)).Status }; }
#[macro_export] macro_rules! NDIS_PACKET_EXTENSION_FROM_PACKET { ($p0:expr $(,)?) => { (((((($p0) as PUCHAR)).wrapping_add(((*$p0).Private.NdisPacketOobOffset) as usize)).wrapping_add((core::mem::size_of::<NDIS_PACKET_OOB_DATA>()) as usize)) as PNDIS_PACKET_EXTENSION) }; }
#[macro_export] macro_rules! NDIS_PER_PACKET_INFO_FROM_PACKET { ($p0:expr, $p1:expr $(,)?) => { (*(((((($p0) as PUCHAR)).wrapping_add(((*$p0).Private.NdisPacketOobOffset) as usize)).wrapping_add((core::mem::size_of::<NDIS_PACKET_OOB_DATA>()) as usize)) as PNDIS_PACKET_EXTENSION)).NdisPacketInfo[($p1) as usize] }; }
#[macro_export] macro_rules! NDIS_SET_PACKET_CANCEL_ID { ($p0:expr, $p1:expr $(,)?) => { { (*(((((($p0) as PUCHAR)).wrapping_add(((*$p0).Private.NdisPacketOobOffset) as usize)).wrapping_add((core::mem::size_of::<NDIS_PACKET_OOB_DATA>()) as usize)) as PNDIS_PACKET_EXTENSION)).NdisPacketInfo[(PacketCancelId) as usize] = (($p1) as _); } }; }
#[macro_export] macro_rules! NDIS_SET_PACKET_STATUS { ($p0:expr, $p1:expr $(,)?) => { { (*((((($p0) as PUCHAR)).wrapping_add(((*$p0).Private.NdisPacketOobOffset) as usize)) as PNDIS_PACKET_OOB_DATA)).Status = (($p1) as _); } }; }
#[macro_export] macro_rules! NdisGetNextBuffer { ($p0:expr, $p1:expr $(,)?) => { { { (*$p1) = (((*$p0).Next) as _); }; } }; }
pub const BUGCODE_ID_DRIVER: ULONG = ((0x80000080u32) as ULONG);
pub type NDIS_ERROR_CODE = ULONG;
pub const STATUS_MORE_PROCESSING_REQUIRED: NTSTATUS = 0xc0000016usize as NTSTATUS;
pub use crate::{MmGetMdlByteCount, MmGetMdlByteOffset, MmGetMdlVirtualAddress, MmGetSystemAddressForMdl, NDIS_GET_PACKET_CANCEL_ID, NDIS_GET_PACKET_HEADER_SIZE, NDIS_GET_PACKET_STATUS, NDIS_PACKET_EXTENSION_FROM_PACKET, NDIS_PER_PACKET_INFO_FROM_PACKET, NDIS_SET_PACKET_CANCEL_ID, NDIS_SET_PACKET_STATUS, NdisGetNextBuffer};
// ==== end SDK header macros ====

// ==== port macros: the module's own C macros (generated -- `rustconv port macros`) ====
// drivers/network/ndis/include/ndissys.h:39: #define ExInterlockedRemoveEntryList(_List, _Lock) { KIRQL OldIrql; KeAcquireSpinLock(_Lock, &OldIrql); RemoveEntryList(_List); KeReleaseSpinLock(_Lock, OldIrql); }
#[allow(unused_macros)]
macro_rules! ExInterlockedRemoveEntryList {
    ($List:expr, $Lock:expr $(,)?) => {{
        let mut OldIrql: KIRQL = unsafe { core::mem::zeroed() };
        KeAcquireSpinLock($Lock, &mut OldIrql);
        RemoveEntryList($List);
        KeReleaseSpinLock($Lock, OldIrql);
    }};
}
#[allow(unused_imports)]
pub(crate) use ExInterlockedRemoveEntryList;
// drivers/network/ndis/include/protocol.h:39: #define GET_ADAPTER_BINDING(Handle) ((PADAPTER_BINDING)Handle)
#[allow(unused_macros)]
macro_rules! GET_ADAPTER_BINDING {
    ($Handle:expr $(,)?) => {
        (($Handle) as PADAPTER_BINDING)
    };
}
#[allow(unused_imports)]
pub(crate) use GET_ADAPTER_BINDING;
// drivers/network/ndis/include/miniport.h:103: #define GET_LOGICAL_ADAPTER(Handle) ((PLOGICAL_ADAPTER)Handle)
#[allow(unused_macros)]
macro_rules! GET_LOGICAL_ADAPTER {
    ($Handle:expr $(,)?) => {
        (($Handle) as PLOGICAL_ADAPTER)
    };
}
#[allow(unused_imports)]
pub(crate) use GET_LOGICAL_ADAPTER;
// drivers/network/ndis/include/miniport.h:84: #define GET_MINIPORT_DRIVER(Handle) ((PNDIS_M_DRIVER_BLOCK)Handle)
#[allow(unused_macros)]
macro_rules! GET_MINIPORT_DRIVER {
    ($Handle:expr $(,)?) => {
        (($Handle) as PNDIS_M_DRIVER_BLOCK)
    };
}
#[allow(unused_imports)]
pub(crate) use GET_MINIPORT_DRIVER;
// drivers/network/ndis/include/protocol.h:18: #define GET_PROTOCOL_BINDING(Handle) ((PPROTOCOL_BINDING)Handle)
#[allow(unused_macros)]
macro_rules! GET_PROTOCOL_BINDING {
    ($Handle:expr $(,)?) => {
        (($Handle) as PPROTOCOL_BINDING)
    };
}
#[allow(unused_imports)]
pub(crate) use GET_PROTOCOL_BINDING;
// drivers/network/ndis/include/ndissys.h:31: #define MIN(value1, value2) ((value1 < value2)? value1 : value2)
#[allow(unused_macros)]
macro_rules! MIN {
    ($value1:expr, $value2:expr $(,)?) => {
        (if ($value1) < ($value2) { ($value1) } else { ($value2) })
    };
}
#[allow(unused_imports)]
pub(crate) use MIN;
// ==== end port macros ====

// ==== port resolve: cross-TU definitions (generated) ====
pub use crate::ndis::miniport::AdapterListHead;
pub use crate::ndis::miniport::AdapterListLock;
pub use crate::ndis::buffer::CopyBufferChainToBuffer;
pub use crate::ndis::buffer::CopyPacketToBuffer;
pub use crate::ndis::main::DebugTraceLevel;
pub use crate::ndis::efilter::EthCreateFilter;
pub use crate::ndis::efilter::EthDeleteFilter;
pub use crate::ndis::efilter::EthFilterDprIndicateReceive;
pub use crate::ndis::efilter::EthFilterDprIndicateReceiveComplete;
pub use crate::ndis::miniport::MiniAdapterHasAddress;
pub use crate::ndis::miniport::MiniDisplayPacket;
pub use crate::ndis::miniport::MiniDoRequest;
pub use crate::ndis::miniport::MiniIndicateData;
pub use crate::ndis::miniport::MiniIsBusy;
pub use crate::ndis::miniport::MiniLocateDevice;
pub use crate::ndis::miniport::MiniQueueWorkItem;
pub use crate::ndis::miniport::MiniReset;
pub use crate::ndis::miniport::MiniSendComplete;
pub use crate::ndis::miniport::MiniWorkItemComplete;
pub use crate::ndis::miniport::MiniportListHead;
pub use crate::ndis::miniport::MiniportListLock;
pub use crate::ndis::config::NdisCloseConfiguration;
pub use crate::ndis::buffer::NdisCopyFromPacketToPacket;
pub use crate::ndis::buffer::NdisGetFirstBufferFromPacket;
pub use crate::ndis::protocol::NdisIPnPCancelStopDevice;
pub use crate::ndis::protocol::NdisIPnPQueryStopDevice;
pub use crate::ndis::string::NdisInitUnicodeString;
pub use crate::ndis::config::NdisOpenConfiguration;
pub use crate::ndis::hardware::NdisQueryPciBusInterface;
pub use crate::ndis::config::NdisReadConfiguration;
pub use crate::ndis::protocol::ProIndicatePacket;
pub use crate::ndis::protocol::ProSendPackets;
pub use crate::ndis::protocol::ProtocolListHead;
pub use crate::ndis::protocol::ProtocolListLock;
pub use crate::ndis::protocol::ndisBindMiniportsToProtocol;
// ==== end cross-TU definitions ====
