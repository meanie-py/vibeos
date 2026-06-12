//! rtl-diag shared glue: consts, byte-order/word macros, FORCEINLINE gaps,
//! and libc externs the diagnostics / image / message cluster needs.
//!
//! The byte-swap macros (SWAPD/SWAPW/SWAPQ) are identity on little-endian
//! (rtlp.h's `#else` arm); all three target arches are LE, so they pass
//! values through unchanged — matching the C build.

use ntkrnl_sys::base::*;
use core::ffi::{c_char, c_int};

// ---- types the bindgen `ddk` has but the staging referenced bare ----
pub use ntkrnl_sys::ddk::{HANDLE, ULONG64, UINT, _UNICODE_STRING};
pub use ntkrnl_sys::ddk::{IMAGE_NT_HEADERS, PIMAGE_NT_HEADERS, PIMAGE_SECTION_HEADER};

// ---- NTSTATUS / IMAGE / RTL consts (exact values from sdk/include) ----
pub const STATUS_INVALID_IMAGE_FORMAT: NTSTATUS = 0xC000007Bu32 as NTSTATUS;
pub const STATUS_MESSAGE_NOT_FOUND: NTSTATUS = 0xC0000109u32 as NTSTATUS;

// PE base-relocation fixup types. CRITICAL: these MUST be defined consts —
// if absent, the `match Type` in LdrProcessRelocationBlockLongLong treats the
// bare names as catch-all *bindings*, so the first arm swallows every fixup and
// NO relocations are applied (driver loads unrelocated → jumps to garbage).
pub const IMAGE_REL_BASED_ABSOLUTE: u32 = 0;
pub const IMAGE_REL_BASED_HIGH: u32 = 1;
pub const IMAGE_REL_BASED_LOW: u32 = 2;
pub const IMAGE_REL_BASED_HIGHLOW: u32 = 3;
pub const IMAGE_REL_BASED_HIGHADJ: u32 = 4;
pub const IMAGE_REL_BASED_DIR64: u32 = 10;

pub const IMAGE_DOS_SIGNATURE: USHORT = 0x5A4D;
pub const IMAGE_NT_SIGNATURE: ULONG = 0x0000_4550;
pub const IMAGE_DIRECTORY_ENTRY_BASERELOC: USHORT = 5;
pub const IMAGE_FILE_RELOCS_STRIPPED: USHORT = 0x0001;
pub const RTL_IMAGE_NT_HEADER_EX_FLAG_NO_RANGE_CHECK: ULONG = 0x0000_0001;
pub const RESOURCE_DATA_LEVEL: ULONG = 3;

pub const RTL_DEBUG_QUERY_MODULES: ULONG = 0x01;
pub const RTL_DEBUG_QUERY_HEAPS: ULONG = 0x04;
pub const RTL_DEBUG_QUERY_LOCKS: ULONG = 0x20;
pub const RTL_UNLOAD_EVENT_TRACE_NUMBER: usize = 16;

// ---- exception / VEH consts ----
pub const EXCEPTION_CONTINUE_EXECUTION: LONG = -1;
pub const EXCEPTION_CONTINUE_SEARCH: LONG = 0;
pub const EXCEPTION_NONCONTINUABLE: ULONG = 0x0001;
pub const STATUS_POSSIBLE_DEADLOCK: NTSTATUS = 0xC0000194u32 as NTSTATUS;

// CONTEXT_FULL = CONTEXT_<arch> | CONTROL | INTEGER | (FLOATING_POINT|SEGMENTS)
#[cfg(target_arch = "x86")]
pub const CONTEXT_FULL: ULONG = 0x0001_0007; // i386: CONTROL|INTEGER|SEGMENTS
#[cfg(target_arch = "x86_64")]
pub const CONTEXT_FULL: ULONG = 0x0010_000B; // amd64: CONTROL|INTEGER|FLOATING_POINT
#[cfg(target_arch = "aarch64")]
pub const CONTEXT_FULL: ULONG = 0x0040_0007; // arm64: CONTROL|INTEGER|FLOATING_POINT

// RtlFillMemoryUlong: NTSYSAPI export on i386 but FORCEINLINE on amd64/arm64
// (no out-of-line symbol), so provide a local faithful impl for all arches.
#[inline]
pub unsafe fn RtlFillMemoryUlong(Destination: PVOID, Length: SIZE_T, Pattern: ULONG) {
    let count = Length / core::mem::size_of::<ULONG>();
    let p = Destination as *mut ULONG;
    let mut i = 0usize;
    while i < count {
        core::ptr::write_volatile(p.add(i), Pattern);
        i += 1;
    }
}

// ---- FORCEINLINE word macros ----
#[inline(always)]
pub fn PtrToUlong(p: PVOID) -> ULONG {
    (p as ULONG_PTR) as ULONG
}
#[inline(always)]
pub fn UlongToPtr(u: ULONG) -> PVOID {
    (u as ULONG_PTR) as PVOID
}
#[inline(always)]
pub fn HandleToULong(h: HANDLE) -> ULONG {
    (h as ULONG_PTR) as ULONG
}

// NtCurrentTeb — FORCEINLINE in winnt.h (reads the TEB self-pointer); no
// out-of-line symbol exists, so provide it via per-arch inline asm (matching
// rtl-actctx's shim). Modules use this instead of the bindgen extern.
#[inline(always)]
pub unsafe fn NtCurrentTeb() -> *mut ntkrnl_sys::ddk::TEB {
    let teb: *mut ntkrnl_sys::ddk::TEB;
    #[cfg(target_arch = "x86")]
    core::arch::asm!("mov {}, fs:[0x18]", out(reg) teb, options(nostack, nomem, pure));
    #[cfg(target_arch = "x86_64")]
    core::arch::asm!("mov {}, gs:[0x30]", out(reg) teb, options(nostack, nomem, pure));
    #[cfg(target_arch = "aarch64")]
    core::arch::asm!("mov {}, x18", out(reg) teb, options(nostack, nomem, pure));
    teb
}

// ---- DbgPrint BREAKPOINT_* service codes ----
pub const BREAKPOINT_PRINT: ULONG = 1;
pub const BREAKPOINT_PROMPT: ULONG = 2;
pub const BREAKPOINT_LOAD_SYMBOLS: ULONG = 3;
pub const BREAKPOINT_UNLOAD_SYMBOLS: ULONG = 4;
pub const BREAKPOINT_COMMAND_STRING: ULONG = 5;

// ---- wine debug-channel types (sdk/include/wine/debug.h) ----
pub type __wine_debug_class = core::ffi::c_uint;
#[repr(C)]
pub struct __wine_debug_channel {
    pub flags: core::ffi::c_uchar,
    pub name: [core::ffi::c_char; 15],
}

// ---- byte-order macros (identity on little-endian) ----
#[inline(always)]
pub fn SWAPD<T>(x: T) -> T { x }
#[inline(always)]
pub fn SWAPW<T>(x: T) -> T { x }
#[inline(always)]
pub fn SWAPQ<T>(x: T) -> T { x }

// ---- WinDef word macros ----
#[inline(always)]
pub fn LOWORD(l: ULONG) -> USHORT { (l & 0xffff) as USHORT }
#[inline(always)]
pub fn HIWORD(l: ULONG) -> USHORT { ((l >> 16) & 0xffff) as USHORT }
#[inline(always)]
pub fn MAKELONG(a: USHORT, b: USHORT) -> LONG {
    ((a as u32) | ((b as u32) << 16)) as LONG
}

/// `IMAGE_FIRST_SECTION(h)` — section table follows the optional header.
#[inline]
pub unsafe fn IMAGE_FIRST_SECTION(h: PIMAGE_NT_HEADERS) -> PIMAGE_SECTION_HEADER {
    ((h as ULONG_PTR)
        + core::mem::offset_of!(IMAGE_NT_HEADERS, OptionalHeader) as ULONG_PTR
        + (*h).FileHeader.SizeOfOptionalHeader as ULONG_PTR) as PIMAGE_SECTION_HEADER
}

/// `RTL_SIZEOF_THROUGH_FIELD(type, field)` = offset-of(field) + sizeof(field).
#[macro_export]
macro_rules! RTL_SIZEOF_THROUGH_FIELD {
    ($t:ty, $f:ident) => {
        (core::mem::offset_of!($t, $f) + {
            fn sz_of<T>(_: *const T) -> usize { core::mem::size_of::<T>() }
            // Infer the field's type for size_of via a pointer projection.
            // Use a non-null *dangling* base (not core::ptr::null()): nightly
            // -Zbuild-std builds `#[deny(deref_nullptr)]`, and a null base trips
            // it. `addr_of!` only forms the field address (no read), so an
            // aligned dangling base is well-defined here.
            sz_of(unsafe {
                core::ptr::addr_of!((*(core::ptr::NonNull::<$t>::dangling().as_ptr())).$f)
            })
        })
    };
}

// ---- libc + ldr externs ----
extern "C" {
    pub fn strrchr(s: *const c_char, c: c_int) -> *mut c_char;
}

extern "system" {
    pub fn LdrQueryProcessModuleInformation(
        ModuleInformation: ntkrnl_sys::ddk::PRTL_PROCESS_MODULES,
        Size: ULONG,
        ReturnLength: PULONG,
    ) -> NTSTATUS;

    // FORCEINLINE in C (NtCurrentPeb()->ProcessHeap); a real exported symbol
    // exists in rtl-strings cabi (ntdll) and ntoskrnl libsupp.
    pub fn RtlGetProcessHeap() -> PVOID;
    // Defined per-consumer in rtl/libsupp.c (kernel + ntdll).
    pub fn RtlpCheckForActiveDebugger() -> BOOLEAN;
    // Pointer obfuscation (exported by ntdll; ntoskrnl provides the kernel side).
    pub fn RtlEncodePointer(Ptr: PVOID) -> PVOID;
    pub fn RtlDecodePointer(Ptr: PVOID) -> PVOID;

    // libsupp.c (per-consumer): DbgPrint reentrancy guard.
    pub fn RtlpSetInDbgPrint() -> BOOLEAN;
    pub fn RtlpClearInDbgPrint();
    // debug_asm.S: the kernel-debugger service trap.
    pub fn DebugService(Service: ULONG, Arg1: PVOID, Arg2: PVOID, Arg3: PVOID, Arg4: PVOID) -> ULONG;
    pub fn DbgBreakPointWithStatus(Status: ULONG);
}

// wine debug channel flags (wine/debug.h) — cdecl. Provided by the C shim
// dbgprint_variadic.c (no real provider exists in the kernel/ntdll link;
// the C original's ros_dbg_log was always NDEBUG-dead-stripped).
extern "C" {
    pub fn __wine_dbg_get_channel_flags(channel: *mut __wine_debug_channel) -> core::ffi::c_uchar;
}
