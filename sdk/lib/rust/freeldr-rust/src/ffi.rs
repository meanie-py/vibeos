//! FreeLDR FFI keystone for the peloader translation — the C surface
//! rustconv/the LLM can't synthesize: freeldr-internal externs (exact
//! ABI from peloader.c.obj), the PE/address macros, the debug-channel
//! macros, and the two peloader.c-defined globals.
//!
//! ABI split is authoritative (nm of build-i386 peloader.c.obj):
//!  - cdecl  (`extern "C"`):  Arc*, DbgPrint2, FrLdrHeap*, Mm*, mem*,
//!                            str*, toupper
//!  - stdcall(`extern "system"`): RtlImageNtHeader@4,
//!    RtlImageDirectoryEntryToData@16, LdrRelocateImageWithBias@28,
//!    RtlAssert@16  (the AI imported these cdecl from ntkrnl_sys — the
//!    classic i386 ABI bug; see feedback_extern_c_abi).

#![allow(non_snake_case, non_upper_case_globals)]

use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::{LIST_ENTRY, PIMAGE_NT_HEADERS};
use core::ffi::c_void;

// ---------------------------------------------------------------------
// conversion.h FORCEINLINE VaToPa/PaToVa (non-_ZOOM2_ variant):
//   VaToPa(Va) = Va & ~KSEG0_BASE ;  PaToVa(Pa) = Pa | KSEG0_BASE
// KSEG0_BASE: i386 0x8000_0000 ; amd64/arm64 0xFFFF_F800_0000_0000.
// ---------------------------------------------------------------------
#[cfg(target_arch = "x86")]
pub const KSEG0_BASE: usize = 0x8000_0000;
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
pub const KSEG0_BASE: usize = 0xFFFF_F800_0000_0000;

// Generic over pointer type — C's `VaToPa(PVOID)` accepts/returns any
// pointer implicitly; the mask is pointer-width-agnostic. Generic keeps
// the caller's pointee type (no per-call-site cast churn).
#[inline]
pub unsafe fn VaToPa<T>(va: *const T) -> *mut T {
    ((va as usize) & !KSEG0_BASE) as *mut T
}
#[inline]
pub unsafe fn PaToVa<T>(pa: *const T) -> *mut T {
    ((pa as usize) | KSEG0_BASE) as *mut T
}

/// ntimage.h `IMAGE_FIRST_SECTION(NtHeader)` macro.
#[inline]
pub unsafe fn IMAGE_FIRST_SECTION(nt: PIMAGE_NT_HEADERS) -> ntkrnl_sys::ddk::PIMAGE_SECTION_HEADER {
    // (ULONG_PTR)nt + FIELD_OFFSET(IMAGE_NT_HEADERS, OptionalHeader)
    //            + FileHeader.SizeOfOptionalHeader
    let opt_off = core::mem::offset_of!(
        ntkrnl_sys::ddk::_IMAGE_NT_HEADERS,
        OptionalHeader
    );
    ((nt as usize)
        + opt_off
        + (*nt).FileHeader.SizeOfOptionalHeader as usize)
        as ntkrnl_sys::ddk::PIMAGE_SECTION_HEADER
}

/// freeldr.h `#define RVA(m, b) ((PVOID)((ULONG_PTR)(b)+(ULONG_PTR)(m)))`
/// — commutative add; callers pass (base, rva).
#[inline]
pub unsafe fn RVA(a: PVOID, b: u32) -> PVOID {
    ((a as usize).wrapping_add(b as usize)) as PVOID
}

// ntimage.h PE ordinal macros (arch-width).
#[cfg(target_arch = "x86")]
pub const IMAGE_ORDINAL_FLAG: usize = 0x8000_0000;
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
pub const IMAGE_ORDINAL_FLAG: usize = 0x8000_0000_0000_0000;

#[inline]
pub fn IMAGE_SNAP_BY_ORDINAL(ordinal: usize) -> u8 {
    ((ordinal & IMAGE_ORDINAL_FLAG) != 0) as u8
}
#[inline]
pub fn IMAGE_ORDINAL(ordinal: usize) -> usize {
    ordinal & 0xffff
}

// ---------------------------------------------------------------------
// boot/freeldr/freeldr/include/debug.h: all four are `DbgPrint2(DbgDefaultChannel,
// <LEVEL>, __FILE__, __LINE__, fmt, ...)` under DBG. The level gate lives INSIDE
// DbgPrint2, and it has TWO terms: `DbgChannels[Mask] & Level` OR
// `Level & DBG_DEFAULT_LEVELS`. `DbgChannels` does start all-zero until the `/DEBUG=`
// boot option opens a channel, but the second term is channel-independent and covers
// ERR and FIXME — so those two print on every boot and only WARN/TRACE wait for
// `/DEBUG=`. (An earlier revision of this comment cited only the first term and
// concluded all four were silent-but-faithful; see debug.rs, where the port had
// dropped the same disjunct and really was silencing every ERR freeldr has.)
//
// `DbgDefaultChannel` is per-file, exactly as `DBG_DEFAULT_CHANNEL(ch)` makes it in the
// C, so each translated module names its own; the macro just reads it at the call site.
#[macro_export]
macro_rules! __freeldr_dbg {
    ($level:expr, $fmt:literal $(, $arg:expr)* $(,)?) => {{
        // Through the `ffi` extern, not `crate::debug`: the `debug` module is
        // `#[cfg(not(feature = "rosload"))]` and rosload.exe imports DbgPrint2 from
        // freeldr.sys, so a direct path stops resolving on that third archive.
        // (freeldr's own definition is not variadic — stable Rust cannot DEFINE a
        // C-variadic — but it now captures the argument slots as fixed parameters
        // and formats through `_vsnprintf`, so `%s`/`%lu` substitute properly. See
        // debug.rs `DbgFormatSlots`. cdecl is what lets the arities differ.)
        $crate::ffi::DbgPrint2(
            DbgDefaultChannel,
            $level,
            concat!(file!(), "\0").as_ptr() as *const core::ffi::c_char,
            line!(),
            concat!($fmt, "\0").as_ptr() as *const core::ffi::c_char
            $(, $arg)*,
        )
    }};
}
#[macro_export]
macro_rules! ERR   { ($($a:tt)*) => { $crate::__freeldr_dbg!($crate::frldr::ERR_LEVEL,   $($a)*) } }
#[macro_export]
macro_rules! FIXME { ($($a:tt)*) => { $crate::__freeldr_dbg!($crate::frldr::FIXME_LEVEL, $($a)*) } }

// TRACE / WARN print only for a channel `/DEBUG=` opens (see above), so they are live in
// the UEFI loader and rosload, which have room, and token-discarding (the C's non-DBG
// form: nothing evaluated, nothing emitted) in the BIOS image, which does not.
// freeldr.sys runs in real-mode memory below MEMORY_MARGIN and MmCheckFreeldrImageFile
// bug-checks when SizeOfImage passes MAX_FREELDR_PE_SIZE (0x77000): restoring ~960
// calls took it from 433,664 to 493,568 bytes and every amd64-bios boot hung at
// "OptionalHeader is invalid" (merge-queue gate 20261010-003633-155v3erm). ERR and FIXME,
// the levels that DO print on every boot, stay live everywhere -- ERR named that failure.
// tools/check_freeldr_size.py now fails the BUILD instead.
#[cfg(any(feature = "uefi", feature = "rosload"))]
#[macro_export]
macro_rules! TRACE { ($($a:tt)*) => { $crate::__freeldr_dbg!($crate::frldr::TRACE_LEVEL, $($a)*) } }
#[cfg(any(feature = "uefi", feature = "rosload"))]
#[macro_export]
macro_rules! WARN  { ($($a:tt)*) => { $crate::__freeldr_dbg!($crate::frldr::WARN_LEVEL,  $($a)*) } }
#[cfg(not(any(feature = "uefi", feature = "rosload")))]
#[macro_export]
macro_rules! TRACE { ($($a:tt)*) => { () } }
#[cfg(not(any(feature = "uefi", feature = "rosload")))]
#[macro_export]
macro_rules! WARN  { ($($a:tt)*) => { () } }

// ---------------------------------------------------------------------
// freeldr-internal C externs — cdecl. (ARC_STATUS/OPENMODE/SEEKMODE/
// TYPE_OF_MEMORY are C enums == i32-width → u32.)
// ---------------------------------------------------------------------
pub type ARC_STATUS = u32;

// --- constants the AI used but rustconv didn't import/define ---
// ARC_CODES (arc.h enum order): ESUCCESS=0 … ENOEXEC=15.
pub const ESUCCESS: ARC_STATUS = 0;
pub const ENOEXEC: ARC_STATUS = 15; // nolint: const-contradiction (ARC_CODES enumerator, arc.h:47 — not errno ENOEXEC=8)
// SEEKMODE / OPENMODE (enums start at 0).
pub const SeekAbsolute: u32 = 0;
pub const SeekRelative: u32 = 1;
pub const OpenReadOnly: u32 = 0;
// TYPE_OF_MEMORY: LoaderExceptionBlock=0 … LoaderLoadedProgram=4.
pub const LoaderLoadedProgram: u32 = 4;
// ldrtypes.h LDRP flags.
pub const LDRP_IMAGE_INTEGRITY_FORCED: ULONG = 0x0000_0020;
pub const LDRP_ENTRY_PROCESSED: ULONG = 0x0000_4000;
// ntimage.h.
pub const IMAGE_FILE_EXECUTABLE_IMAGE: USHORT = 0x0002;
pub const IMAGE_DLLCHARACTERISTICS_FORCE_INTEGRITY: USHORT = 0x0080;
pub const IMAGE_DIRECTORY_ENTRY_LOAD_CONFIG: USHORT = 10;
// fs.h.
pub const SECTOR_SIZE: ULONG = 512;
// peloader.c `#define DEFAULT_SECURITY_COOKIE` (arch-split; rustconv
// dropped the #define).
#[cfg(target_arch = "x86")]
pub const DEFAULT_SECURITY_COOKIE: usize = 0xBB40_E64E;
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
pub const DEFAULT_SECURITY_COOKIE: usize = 0x0000_2B99_2DDF_A232;
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
pub const COOKIE_MAX: usize = 0x0000_FFFF_FFFF_FFFF;

extern "C" {
    pub fn ArcOpen(path: *mut CHAR, open_mode: u32, file_id: *mut ULONG) -> ARC_STATUS;
    pub fn ArcClose(file_id: ULONG) -> ARC_STATUS;
    pub fn ArcRead(file_id: ULONG, buffer: PVOID, n: ULONG, count: *mut ULONG) -> ARC_STATUS;
    pub fn ArcSeek(file_id: ULONG, position: *mut i64, seek_mode: u32) -> ARC_STATUS;
    pub fn ArcGetRelativeTime() -> ULONG;

    pub fn DbgPrint2(mask: ULONG, level: ULONG, file: *const CHAR, line: ULONG, fmt: *const CHAR, ...) -> ULONG;

    pub fn FrLdrHeapAlloc(size: usize, tag: ULONG) -> PVOID;
    pub fn FrLdrHeapFree(p: PVOID, tag: ULONG);
    pub fn MmAllocateMemoryWithType(size: usize, mem_type: u32) -> PVOID;
    pub fn MmAllocateMemoryAtAddress(size: usize, desired: PVOID, mem_type: u32) -> PVOID;
    pub fn MmFreeMemory(p: PVOID);

    pub fn memmove(dst: PVOID, src: *const c_void, n: usize) -> PVOID;
    pub fn memset(dst: PVOID, c: i32, n: usize) -> PVOID;
    pub fn strchr(s: *const CHAR, c: i32) -> *mut CHAR;
    pub fn strrchr(s: *const CHAR, c: i32) -> *mut CHAR;
    pub fn strcmp(a: *const CHAR, b: *const CHAR) -> i32;
    pub fn strlen(s: *const CHAR) -> usize;
    pub fn toupper(c: i32) -> i32;
}

// stdcall (NTAPI, i386 `_Name@N`). MUST be `extern "system"`.
extern "system" {
    pub fn RtlImageNtHeader(base: PVOID) -> PIMAGE_NT_HEADERS;
    pub fn RtlImageDirectoryEntryToData(
        base: PVOID,
        mapped_as_image: u8,
        directory_entry: USHORT,
        size: *mut ULONG,
    ) -> PVOID;
    pub fn LdrRelocateImageWithBias(
        new_base: PVOID,
        additional_bias: i64,
        loader_name: *mut CHAR,
        success: ULONG,
        conflict: ULONG,
        invalid: ULONG,
    ) -> ULONG;
    pub fn RtlAssert(failed: PVOID, file: PVOID, line: ULONG, message: *mut CHAR);
}

/// FreeLDR PE image base (linker symbol; i386 GNU `___ImageBase`).
extern "C" {
    pub static __ImageBase: u8;
}

// ---------------------------------------------------------------------
// peloader.c-defined globals (the AI mis-declared these `extern`).
// ---------------------------------------------------------------------
/// `LIST_ENTRY FrLdrModuleList;`
#[no_mangle]
pub static mut FrLdrModuleList: LIST_ENTRY = LIST_ENTRY {
    Flink: core::ptr::null_mut(),
    Blink: core::ptr::null_mut(),
};

/// `PELDR_IMPORTDLL_LOAD_CALLBACK PeLdrImportDllLoadCallback = NULL;`
/// C typedef is `VOID (NTAPI *)(PCSTR)` — NTAPI = `stdcall` on i386,
/// `cdecl` elsewhere → `extern "system"`. Was `extern "C"` (cdecl), which
/// leaked 8 bytes of ESP per i386 invocation: caller (cdecl) does
/// `add esp,4` after CALL, but stdcall callee already did `ret 4`. The next
/// `PeLdrLoadImage(...)` call's pushed args then wrote OVER the caller's
/// `FullDllName` buffer (the pushed `1u8` KernelMapping landed at offset 0),
/// turning the bootvid.dll path into `\x01\0` → ArcOpen EINVAL → loader
/// hangs after kdcom load on i386.
pub type PELDR_IMPORTDLL_LOAD_CALLBACK = Option<unsafe extern "system" fn(*const CHAR)>;
#[no_mangle]
pub static mut PeLdrImportDllLoadCallback: PELDR_IMPORTDLL_LOAD_CALLBACK = None;

// ---------------------------------------------------------------------
// ntstrsafe RtlStringCbCopyA/CatA — peloader.c emits these locally
// (static inline from ntstrsafe.h; rustconv dropped them). Faithful
// bounded ANSI copy/cat; `cb` is a BYTE count. NTSTATUS return.
// ---------------------------------------------------------------------
const STATUS_SUCCESS: NTSTATUS = 0;
const STATUS_BUFFER_OVERFLOW: NTSTATUS = 0x8000_0005u32 as NTSTATUS;
const STATUS_INVALID_PARAMETER: NTSTATUS = 0xC000_000Du32 as NTSTATUS;

#[inline]
#[no_mangle]
pub unsafe extern "C" fn RtlStringCbCopyA(dest: *mut CHAR, cb_dest: usize, src: *const CHAR) -> NTSTATUS {
    if dest.is_null() || src.is_null() || cb_dest == 0 {
        return STATUS_INVALID_PARAMETER;
    }
    let mut i = 0usize;
    while i + 1 < cb_dest {
        let ch = *src.add(i);
        *dest.add(i) = ch;
        if ch == 0 {
            return STATUS_SUCCESS;
        }
        i += 1;
    }
    *dest.add(i) = 0;
    if *src.add(i) == 0 { STATUS_SUCCESS } else { STATUS_BUFFER_OVERFLOW }
}

#[inline]
#[no_mangle]
pub unsafe extern "C" fn RtlStringCbCatA(dest: *mut CHAR, cb_dest: usize, src: *const CHAR) -> NTSTATUS {
    if dest.is_null() || src.is_null() || cb_dest == 0 {
        return STATUS_INVALID_PARAMETER;
    }
    let mut d = 0usize;
    while d < cb_dest && *dest.add(d) != 0 {
        d += 1;
    }
    if d >= cb_dest {
        return STATUS_INVALID_PARAMETER;
    }
    let mut s = 0usize;
    while d + 1 < cb_dest {
        let ch = *src.add(s);
        *dest.add(d) = ch;
        if ch == 0 {
            return STATUS_SUCCESS;
        }
        d += 1;
        s += 1;
    }
    *dest.add(d) = 0;
    if *src.add(s) == 0 { STATUS_SUCCESS } else { STATUS_BUFFER_OVERFLOW }
}

// ntstrsafe RtlStringCbPrintfA — REAL out-of-line def lives in
// boot/freeldr/freeldr/lib/strsafe_shim.c (forwards to
// RtlStringCbVPrintfA via va_list — Rust can't host the variadic def on
// stable). The previous degraded `RtlStringCbCopyA` shim here silently
// left %u/%s/%x unformatted in boot paths (rosload.exe failed to load
// because `multi(0)disk(0)cdrom(%u)\rosload.exe` reached ArcOpen
// verbatim).

// ntstrsafe RtlStringCbPrintfW — the real out-of-line def is in
// boot/freeldr/freeldr/lib/strsafe_shim.c, beside the A variant and for the same
// reason. The degraded copy that stood here returned its FORMAT verbatim, so
// setupldr registered every boot driver as "%S\%S" and then panicked stripping
// ".sys" from the two-character name "%S" -- the first thing an install from the
// bootcd did.

// rtlfuncs.h RtlInitEmptyUnicodeString — FORCEINLINE (3-line body, no
// symbol anywhere). bindgen emits it as a cdecl `extern` decl, and
// wlregistry.rs / setupldr.rs call it via ntkrnl_sys::ddk, so the i386
// link needs a real `_RtlInitEmptyUnicodeString`. Faithful translation
// of the rtlfuncs.h body. cdecl (bindgen declared it `extern "C"`).
// UNIQUE: no other #[no_mangle] defines this symbol.
#[no_mangle]
pub unsafe extern "C" fn RtlInitEmptyUnicodeString(
    UnicodeString: *mut ntkrnl_sys::ddk::_UNICODE_STRING,
    Buffer: *mut u16,
    BufferSize: u16,
) {
    if UnicodeString.is_null() {
        return;
    }
    (*UnicodeString).Length = 0;
    (*UnicodeString).MaximumLength = BufferSize;
    (*UnicodeString).Buffer = Buffer;
}

// ntstrsafe.h:381 RtlStringCbCopyNA — bounded copy of at most
// `cb_to_copy` bytes from src into a `cb_dest`-byte dest, stopping at
// src NUL, always NUL-terminating. Inline in ntstrsafe.h (no symbol in
// non-MSVC freeldr); rustconv emitted `extern { fn ... }`. cdecl,
// UNIQUE.
#[no_mangle]
pub unsafe extern "C" fn RtlStringCbCopyNA(
    dest: *mut CHAR,
    cb_dest: usize,
    src: *const CHAR,
    cb_to_copy: usize,
) -> NTSTATUS {
    if dest.is_null() || src.is_null() || cb_dest == 0 {
        return STATUS_INVALID_PARAMETER;
    }
    let mut i = 0usize;
    while i + 1 < cb_dest && i < cb_to_copy {
        let ch = *src.add(i);
        if ch == 0 {
            break;
        }
        *dest.add(i) = ch;
        i += 1;
    }
    *dest.add(i) = 0;
    // Overflow iff the source still had bytes to copy within the
    // requested count but the dest buffer ran out.
    if i < cb_to_copy && *src.add(i) != 0 {
        STATUS_BUFFER_OVERFLOW
    } else {
        STATUS_SUCCESS
    }
}
