//! inlines.rs — FORCEINLINE / header-only / intrinsic helpers the LLM rendered as calls.
//! IRP-stack walkers, the ntstrsafe wide-printf wrappers (header inlines over the
//! ntoskrnl-exported _vsnwprintf), RtlULongAdd (ntintsafe), the interlocked intrinsics.
//! None are ntoskrnl exports themselves (the bindgen `ddk` versions are phantom externs
//! that fail at link) except _vsnwprintf / vDbgPrintEx, which are — and those are now
//! reached through the C shim (see the va_list note further down).
#![allow(non_snake_case, non_upper_case_globals, dead_code, non_camel_case_types)]
use ntkrnl_sys::ddk::{NTSTATUS, ULONG, LONG, va_list};

// FORCEINLINE IRP-stack walkers (Tail.Overlay arithmetic) — re-export the tier1 inlines
// (the bindgen ddk versions are phantom externs). Strip these from the modules' explicit
// ddk imports so they resolve here, not to the undefined extern (the partmgr/fs_rec trap).
pub use ntkrnl_sys::inline::tier1::{
    IoGetCurrentIrpStackLocation, IoGetNextIrpStackLocation, IoSkipCurrentIrpStackLocation,
    IoSetCompletionRoutine,
};

// interlocked intrinsics (the LLM uses the _-prefixed MSVC intrinsic names).
pub use ntkrnl_sys::interlocked::InterlockedIncrement as _InterlockedIncrement;
pub use ntkrnl_sys::interlocked::InterlockedDecrement as _InterlockedDecrement;
pub use ntkrnl_sys::interlocked::InterlockedExchangeAdd;

// haltypes.h: #define HalGetInterruptTranslator HALDISPATCH->HalGetInterruptTranslator
// HALDISPATCH = &HalDispatchTable (an ntoskrnl data export, `@ extern HalDispatchTable`).
// Not a real exported function — it's a dispatch-table fn-ptr call. Mirror the macro:
// import the data symbol from ntoskrnl, then call the HalGetInterruptTranslator field.
// Cross-DLL data import from ntoskrnl — needs raw-dylib so rustc emits the __imp_
// indirection (same pattern hal_bindgen uses; a plain extern static produces a direct
// `_HalDispatchTable` ref that the i386 importlib can't satisfy).
#[link(name = "ntoskrnl.exe", kind = "raw-dylib", modifiers = "+verbatim")]
extern "C" {
    static HalDispatchTable: ntkrnl_sys::ddk::HAL_DISPATCH;
}
#[inline(always)]
pub unsafe fn HalGetInterruptTranslator(
    ParentInterfaceType: i32, ParentBusNumber: ULONG, BridgeInterfaceType: i32,
    Size: u16, Version: u16, Translator: *mut core::ffi::c_void,
    BridgeBusNumber: *mut ULONG,
) -> NTSTATUS {
    let f = HalDispatchTable.HalGetInterruptTranslator.expect("HalGetInterruptTranslator");
    f(ParentInterfaceType, ParentBusNumber, BridgeInterfaceType, Size, Version,
      Translator as ntkrnl_sys::ddk::PTRANSLATOR_INTERFACE, BridgeBusNumber)
}

// exfuncs.h FORCEINLINE ExAllocatePoolZero — ExAllocatePoolWithTag + zero-fill (ReactOS
// ntoskrnl doesn't export it; it's an inline wrapper). Faithful to the header body.
#[inline(always)]
pub unsafe fn ExAllocatePoolZero(
    PoolType: ntkrnl_sys::ddk::POOL_TYPE, NumberOfBytes: ntkrnl_sys::ddk::SIZE_T, Tag: ULONG,
) -> ntkrnl_sys::ddk::PVOID {
    let p = ntkrnl_sys::ddk::ExAllocatePoolWithTag(PoolType, NumberOfBytes, Tag);
    if !p.is_null() {
        core::ptr::write_bytes(p as *mut u8, 0, NumberOfBytes as usize);
    }
    p
}

// wdm.h FORCEINLINE KeInitializeSpinLock — zero the lock word.
#[inline(always)]
pub unsafe fn KeInitializeSpinLock(SpinLock: ntkrnl_sys::ddk::PKSPIN_LOCK) {
    *SpinLock = 0;
}

// exfuncs.h FORCEINLINE ExInitializeFastMutex — FM_LOCK_BIT(=1)/Owner=NULL/Contention=0 +
// KeInitializeEvent(SynchronizationEvent, FALSE).
#[inline(always)]
pub unsafe fn ExInitializeFastMutex(FastMutex: ntkrnl_sys::ddk::PFAST_MUTEX) {
    (*FastMutex).Count = 1; // FM_LOCK_BIT
    (*FastMutex).Owner = core::ptr::null_mut();
    (*FastMutex).Contention = 0;
    ntkrnl_sys::ddk::KeInitializeEvent(
        &mut (*FastMutex).Event,
        ntkrnl_sys::ddk::_EVENT_TYPE_SynchronizationEvent,
        0,
    );
}

// wdm.h macro: IoAdjustPagingPathCount(Count, Inc) = InterlockedExchangeAdd(Count, Inc).
#[inline(always)]
pub unsafe fn IoAdjustPagingPathCount(Count: *mut LONG, Increment: LONG) {
    InterlockedExchangeAdd(Count, Increment);
}

// DbgBreakPoint — stable replacement for core::intrinsics::breakpoint() the LLM emitted.
#[inline(always)]
pub unsafe fn rust_breakpoint() {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    core::arch::asm!("int3", options(nomem, nostack));
    #[cfg(target_arch = "aarch64")]
    core::arch::asm!("brk #0xf000", options(nomem, nostack));
}

// __int2c — the MSVC intrinsic for `int 0x2c` (RaiseAssertion/break). Used only on the
// assertion-failure path; a debug breakpoint is faithful.
#[inline(always)]
pub unsafe fn __int2c() {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    core::arch::asm!("int 0x2c", options(nomem, nostack));
    #[cfg(target_arch = "aarch64")]
    core::arch::asm!("brk #0xf000", options(nomem, nostack));
}

// NTSTRSAFE pointer aliases the modules cast through.
pub type NTSTRSAFE_PWSTR = *mut u16;
pub type NTSTRSAFE_PCWSTR = *const u16;

// NTSTATUS values used by the ntstrsafe/ntintsafe wrappers.
const STATUS_SUCCESS_: NTSTATUS = 0;
const STATUS_BUFFER_OVERFLOW_: NTSTATUS = 0x80000005u32 as i32;
const STATUS_INVALID_PARAMETER_: NTSTATUS = 0xC000000Du32 as i32;
const STATUS_INTEGER_OVERFLOW_: NTSTATUS = 0xC0000095u32 as i32;

// ---- the C-variadic boundary (drivers/storage/ide/pciidex/rust_varargs_shim.c) -------
//
// Stable Rust cannot DEFINE a C-variadic fn (`va_start`) nor `va_arg` — that is the
// nightly `c_variadic` feature (rust-lang#44930). It CAN declare and CALL one. So the
// variadic ENTRY of each ntstrsafe wrapper lives in the shim, which immediately calls the
// fixed-arity `_v` twin below holding ALL the logic. Call sites are unchanged.
//
// Neither wrapper pulls args itself: both hand the whole va_list to ntoskrnl's exported
// `_vsnwprintf`. Rust only ever holds an opaque POINTER to the C frame's va_list, so it
// cannot pass one BY VALUE — the shim's `vibeos_vsnwprintf` dereferences the pointer and
// makes that call. It does nothing else; the buffer sizing / NUL-termination / NTSTATUS
// mapping below is untouched. (Same shape for vDbgPrintEx, used by miniport.rs.)
extern "C" {
    fn vibeos_vsnwprintf(
        buffer: *mut u16, count: usize, format: *const u16, ap: *mut core::ffi::c_void,
    ) -> i32;
    pub fn vibeos_vDbgPrintEx(
        ComponentId: ULONG, Level: ULONG, Format: *const i8, ap: *mut core::ffi::c_void,
    ) -> ULONG;
}

// The variadic entry points themselves — DEFINED in the shim, declared here so the module
// call sites (pciidex.rs / fdo.rs / pdo.rs) keep calling `RtlStringCbPrintfW(...)` /
// `RtlStringCchPrintfExW(...)` exactly as before. Declaring + calling a C-variadic fn is
// stable; only defining one is not.
extern "C" {
    pub fn RtlStringCbPrintfW(
        dest: *mut u16, cbDest: usize, format: *const u16, ...
    ) -> NTSTATUS;
    pub fn RtlStringCchPrintfExW(
        dest: *mut u16, cchDest: usize, ppszDestEnd: *mut *mut u16,
        pcchRemaining: *mut usize, flags: ULONG, format: *const u16, ...
    ) -> NTSTATUS;
}

// ntstrsafe.h FORCEINLINE RtlStringCbPrintfW (cb = byte count) — forward to _vsnwprintf,
// guarantee null-termination, map overflow to STATUS_BUFFER_OVERFLOW. Faithful to the
// ntstrsafe contract for the formats pciidex uses (%u / %ls).
// Fixed-arity twin of the shim's `RtlStringCbPrintfW`; `ap` points at its va_list.
#[no_mangle]
pub unsafe extern "C" fn RtlStringCbPrintfW_v(
    dest: *mut u16,
    cbDest: usize,
    format: *const u16,
    ap: *mut core::ffi::c_void,
) -> NTSTATUS {
    let cch = cbDest / core::mem::size_of::<u16>();
    if cch == 0 {
        return STATUS_INVALID_PARAMETER_;
    }
    let n = vibeos_vsnwprintf(dest, cch - 1, format, ap);
    let overflow = n < 0 || (n as usize) >= cch;
    let idx = if overflow { cch - 1 } else { n as usize };
    *dest.add(idx) = 0;
    if overflow { STATUS_BUFFER_OVERFLOW_ } else { STATUS_SUCCESS_ }
}

// ntstrsafe.h FORCEINLINE RtlStringCchPrintfExW (cch = char count, with end-ptr +
// remaining out-params + flags). pciidex passes flags=0.
// Fixed-arity twin of the shim's `RtlStringCchPrintfExW`; `ap` points at its va_list.
#[no_mangle]
pub unsafe extern "C" fn RtlStringCchPrintfExW_v(
    dest: *mut u16,
    cchDest: usize,
    ppszDestEnd: *mut *mut u16,
    pcchRemaining: *mut usize,
    _flags: ULONG,
    format: *const u16,
    ap: *mut core::ffi::c_void,
) -> NTSTATUS {
    if cchDest == 0 {
        return STATUS_INVALID_PARAMETER_;
    }
    let n = vibeos_vsnwprintf(dest, cchDest - 1, format, ap);
    let overflow = n < 0 || (n as usize) >= cchDest;
    let written = if overflow { cchDest - 1 } else { n as usize };
    *dest.add(written) = 0;
    if !ppszDestEnd.is_null() {
        *ppszDestEnd = dest.add(written);
    }
    if !pcchRemaining.is_null() {
        *pcchRemaining = cchDest - written;
    }
    if overflow { STATUS_BUFFER_OVERFLOW_ } else { STATUS_SUCCESS_ }
}

// ntintsafe.h FORCEINLINE RtlULongAdd — checked 32-bit add.
#[inline(always)]
pub unsafe fn RtlULongAdd(Augend: ULONG, Addend: ULONG, Result: *mut ULONG) -> NTSTATUS {
    match Augend.checked_add(Addend) {
        Some(v) => {
            *Result = v;
            STATUS_SUCCESS_
        }
        None => {
            *Result = ULONG::MAX;
            STATUS_INTEGER_OVERFLOW_
        }
    }
}
