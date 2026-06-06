//! Glue the rustconv draft expects but that isn't in the kernel
//! bindgen/base globs: the C string/mem libc surface (provided at link
//! time by cnt-rust / libcntpr), the Wine TRACE debug helpers (no-ops in
//! the kernel build), and the SxS status/flag constants + FIELD_OFFSET
//! macro the C uses. Values transcribed from sdk/include (ntstatus.h,
//! winnt, ndk/rtltypes).

use ntkrnl_sys::base::{NTSTATUS, PVOID, WCHAR};
use ntkrnl_sys::ddk::{IMAGE_RESOURCE_DIRECTORY, LDR_DATA_TABLE_ENTRY};
use core::ffi::{c_char, c_int, c_void};

// ── NT heap / loader / conversion helpers (linked from ntoskrnl/ntdll) ─
extern "system" {
    pub fn RtlGetProcessHeap() -> PVOID;
    // SIZE_T == usize at the ABI level (pointer width); declaring the size
    // params/return as usize matches the draft's usize call sites without
    // the nominal ddk `u64` mismatch, and is ABI-correct on i386 too.
    pub fn RtlAllocateHeap(heap: PVOID, flags: u32, size: usize) -> PVOID;
    pub fn RtlSizeHeap(heap: PVOID, flags: u32, ptr: PVOID) -> usize;
    pub fn RtlReAllocateHeap(heap: PVOID, flags: u32, ptr: PVOID, size: usize) -> PVOID;
    pub fn RtlUTF8ToUnicodeN(
        dst: PVOID,
        dst_max: u32,
        written: *mut u32,
        src: *const c_void,
        src_len: u32,
    ) -> NTSTATUS;
    pub fn LdrFindEntryForAddress(
        addr: PVOID,
        entry: *mut *mut LDR_DATA_TABLE_ENTRY,
    ) -> NTSTATUS;
    // ReactOS ntdll process-compat init (actctx.c __REACTOS__ path).
    pub fn LdrpInitializeProcessCompat(process_actctx: PVOID, old_shim_data: *mut PVOID);
    // NtGetTickCount is a spec-forwarder to RtlGetTickCount; call the real
    // impl directly (the forwarder isn't a callable internal symbol).
    pub fn RtlGetTickCount() -> u32;
}

// NtCurrentTeb() — FORCEINLINE macro in C (reads the TEB self-pointer);
// no out-of-line symbol exists, so provide it via per-arch inline asm.
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

// ntstrsafe RtlStringCch{Copy,Cat}W — FORCEINLINE in ntstrsafe.h (no
// symbol); faithful bounded copy/append, null-terminating, returning
// STATUS_SUCCESS / STATUS_BUFFER_OVERFLOW / STATUS_INVALID_PARAMETER.
#[inline]
pub unsafe fn RtlStringCchCopyW(dst: *mut WCHAR, cch: usize, src: *const WCHAR) -> NTSTATUS {
    if cch == 0 {
        return 0xC000_000Du32 as i32; // STATUS_INVALID_PARAMETER
    }
    let mut i = 0usize;
    while i + 1 < cch && *src.add(i) != 0 {
        *dst.add(i) = *src.add(i);
        i += 1;
    }
    *dst.add(i) = 0;
    if *src.add(i) != 0 {
        0x8000_0005u32 as i32 // STATUS_BUFFER_OVERFLOW
    } else {
        0 // STATUS_SUCCESS
    }
}

#[inline]
pub unsafe fn RtlStringCchCatW(dst: *mut WCHAR, cch: usize, src: *const WCHAR) -> NTSTATUS {
    let mut d = 0usize;
    while d < cch && *dst.add(d) != 0 {
        d += 1;
    }
    if d >= cch {
        return 0xC000_000Du32 as i32; // STATUS_INVALID_PARAMETER (not terminated)
    }
    let mut i = 0usize;
    while d + i + 1 < cch && *src.add(i) != 0 {
        *dst.add(d + i) = *src.add(i);
        i += 1;
    }
    *dst.add(d + i) = 0;
    if *src.add(i) != 0 {
        0x8000_0005u32 as i32 // STATUS_BUFFER_OVERFLOW
    } else {
        0 // STATUS_SUCCESS
    }
}

// ── PE resource directory walkers (external rtl, stay C — image.c) ────
extern "C" {
    pub fn find_entry_by_name(
        dir: *mut IMAGE_RESOURCE_DIRECTORY,
        name: *const WCHAR,
        root: *mut c_void,
        want_dir: c_int,
    ) -> *mut IMAGE_RESOURCE_DIRECTORY;
    pub fn find_first_entry(
        dir: *mut IMAGE_RESOURCE_DIRECTORY,
        root: *mut c_void,
        want_dir: c_int,
    ) -> *mut IMAGE_RESOURCE_DIRECTORY;
}

// GetProcessHeap() — actctx.c `#define GetProcessHeap() RtlGetProcessHeap()`.
#[inline(always)]
pub unsafe fn GetProcessHeap() -> PVOID {
    RtlGetProcessHeap()
}

// PtrToUlong / HIWORD (winnt macros).
#[inline(always)]
pub fn PtrToUlong(p: PVOID) -> u32 {
    p as usize as u32
}
// HIWORD(l) — winnt macro; polymorphic (accepts pointers or integers).
#[macro_export]
macro_rules! HIWORD {
    ($l:expr) => {
        ((($l) as usize as u32) >> 16) & 0xffff
    };
}

// windows_dir = SharedUserData->NtSystemRoot. KUSER_SHARED_DATA is at the
// fixed user VA 0x7FFE0000 (NtSystemRoot at +0x30); actctx runs in ntdll
// (user mode) — the kernel librtl copy never reaches the SxS winsxs path.
#[inline(always)]
pub unsafe fn windows_dir() -> *const WCHAR {
    (0x7FFE_0000usize + 0x30) as *const WCHAR
}

// ── Wine TRACE/FIXME/WARN/ERR — no-op (args discarded) ───────────────
#[macro_export]
macro_rules! TRACE { ($($t:tt)*) => {{}}; }
#[macro_export]
macro_rules! FIXME { ($($t:tt)*) => {{}}; }
#[macro_export]
macro_rules! WARN { ($($t:tt)*) => {{}}; }
#[macro_export]
macro_rules! ERR { ($($t:tt)*) => {{}}; }

// ── libc string/mem (linked from cnt-rust) ──────────────────────────
extern "C" {
    pub fn wcslen(s: *const WCHAR) -> usize;
    pub fn wcscpy(dst: *mut WCHAR, src: *const WCHAR) -> *mut WCHAR;
    pub fn wcscat(dst: *mut WCHAR, src: *const WCHAR) -> *mut WCHAR;
    pub fn wcscmp(a: *const WCHAR, b: *const WCHAR) -> c_int;
    pub fn wcsncmp(a: *const WCHAR, b: *const WCHAR, n: usize) -> c_int;
    pub fn wcschr(s: *const WCHAR, c: c_int) -> *mut WCHAR;
    pub fn wcsrchr(s: *const WCHAR, c: c_int) -> *mut WCHAR;
    pub fn wcstoul(s: *const WCHAR, end: *mut *mut WCHAR, base: c_int) -> core::ffi::c_ulong;
    pub fn memcpy(dst: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    pub fn memset(dst: *mut c_void, c: c_int, n: usize) -> *mut c_void;
    pub fn memcmp(a: *const c_void, b: *const c_void, n: usize) -> c_int;

    // macro-renamed in C (`#define wcsnicmp _wcsnicmp`, etc.)
    #[link_name = "_wcsnicmp"]
    pub fn wcsnicmp(a: *const WCHAR, b: *const WCHAR, n: usize) -> c_int;
    #[link_name = "_wcsicmp"]
    pub fn wcsicmp(a: *const WCHAR, b: *const WCHAR) -> c_int;

    // `#define swprintf _snwprintf` — count form, variadic.
    #[link_name = "_snwprintf"]
    pub fn swprintf(buf: *mut WCHAR, count: usize, fmt: *const WCHAR, ...) -> c_int;
    pub fn _swprintf(buf: *mut WCHAR, fmt: *const WCHAR, ...) -> c_int;
}

// ── Wine TRACE/debug helpers — no-ops in the kernel build ────────────
#[inline(always)]
pub unsafe fn debugstr_w(_s: *const WCHAR) -> *const c_char {
    core::ptr::null()
}
#[inline(always)]
pub unsafe fn debugstr_wn(_s: *const WCHAR, _n: u32) -> *const c_char {
    core::ptr::null()
}
#[inline(always)]
pub unsafe fn debugstr_a(_s: *const c_char) -> *const c_char {
    core::ptr::null()
}
// wine_dbg_sprintf is variadic in C; the only consumers are debug strings
// (dead in the kernel), so a no-op stub that ignores its args is faithful.
#[macro_export]
macro_rules! wine_dbg_sprintf {
    ($($arg:tt)*) => {
        core::ptr::null::<core::ffi::c_char>()
    };
}

// FIELD_OFFSET(type, field) -> byte offset (the C macro).
#[macro_export]
macro_rules! FIELD_OFFSET {
    ($t:ty, $f:ident) => {
        core::mem::offset_of!($t, $f)
    };
}

// ── SxS / resource NTSTATUS (ntstatus.h) ────────────────────────────
pub const STATUS_SXS_SECTION_NOT_FOUND: i32 = 0xC0150001u32 as i32;
pub const STATUS_SXS_ASSEMBLY_NOT_FOUND: i32 = 0xC0150004u32 as i32;
pub const STATUS_SXS_CANT_GEN_ACTCTX: i32 = 0xC0150002u32 as i32;
pub const STATUS_SXS_INVALID_ACTCTXDATA_FORMAT: i32 = 0xC0150003u32 as i32;
pub const STATUS_SXS_KEY_NOT_FOUND: i32 = 0xC0150008u32 as i32;
pub const STATUS_SXS_EARLY_DEACTIVATION: i32 = 0xC015000Fu32 as i32;
pub const STATUS_SXS_INVALID_DEACTIVATION: i32 = 0xC0150010u32 as i32;
pub const STATUS_RESOURCE_TYPE_NOT_FOUND: i32 = 0xC000008Au32 as i32;
pub const STATUS_RESOURCE_NAME_NOT_FOUND: i32 = 0xC000008Bu32 as i32;
pub const STATUS_RESOURCE_DATA_NOT_FOUND: i32 = 0xC0000089u32 as i32;
pub const STATUS_DLL_NOT_FOUND: i32 = 0xC0000135u32 as i32;
pub const STATUS_INVALID_IMAGE_FORMAT: i32 = 0xC000007Bu32 as i32;

// ── flags / misc constants ──────────────────────────────────────────
pub const HEAP_ZERO_MEMORY: u32 = 0x0000_0008;
// RTL_PATH_TYPE::RtlPathTypeRelative (actctx.c `#define RELATIVE_PATH`).
pub const RtlPathTypeRelative: u32 = 5;
pub const HASH_STRING_ALGORITHM_X65599: u32 = 1;

pub const FIND_ACTCTX_SECTION_KEY_RETURN_HACTCTX: u32 = 0x0000_0001;
pub const FIND_ACTCTX_SECTION_KEY_RETURN_FLAGS: u32 = 0x0000_0002;
pub const FIND_ACTCTX_RETURN_FLAGS: u32 = 0x0000_0002;
pub const FIND_ACTCTX_RETURN_ASSEMBLY_METADATA: u32 = 0x0000_0004;

pub const RTL_QUERY_ACTIVATION_CONTEXT_FLAG_USE_ACTIVE_ACTIVATION_CONTEXT: u32 = 0x01;
pub const RTL_QUERY_ACTIVATION_CONTEXT_FLAG_IS_HMODULE: u32 = 0x02;
pub const RTL_QUERY_ACTIVATION_CONTEXT_FLAG_IS_ADDRESS: u32 = 0x04;
pub const RTL_QUERY_ACTIVATION_CONTEXT_FLAG_NO_ADDREF: u32 = 0x8000_0000;
pub const RTL_DEACTIVATE_ACTIVATION_CONTEXT_FLAG_FORCE_EARLY_DEACTIVATION: u32 = 0x01;

pub const RTL_ACTIVATION_CONTEXT_STACK_FRAME_FLAG_HEAP_ALLOCATED: u32 = 0x08;
pub const RTL_ACTIVATION_CONTEXT_STACK_FRAME_FLAG_NOT_REALLY_ACTIVATED: u32 = 0x10;
pub const RTL_ACTIVATION_CONTEXT_STACK_FRAME_FLAG_ACTIVATED: u32 = 0x20;
pub const RTL_ACTIVATION_CONTEXT_STACK_FRAME_FLAG_DEACTIVATED: u32 = 0x40;

pub const IS_TEXT_UNICODE_SIGNATURE: u32 = 8;
pub const IS_TEXT_UNICODE_REVERSE_SIGNATURE: u32 = 128;
pub const IMAGE_DIRECTORY_ENTRY_RESOURCE: u32 = 2;
pub const FILE_DIRECTORY_FILE: u32 = 0x0000_0001;
pub const FILE_SYNCHRONOUS_IO_ALERT: u32 = 0x0000_0010;
pub const FILE_SYNCHRONOUS_IO_NONALERT: u32 = 0x0000_0020;

// FILE_INFORMATION_CLASS values (ddk bindgen doesn't export the bare
// variant names). FileEndOfFileInformation is `#define`d to
// FileStandardInformation (=5) in actctx.c.
pub const FileBothDirectoryInformation: u32 = 3;
pub const FileStandardInformation: u32 = 5;
pub const FileEndOfFileInformation: u32 = 5;

// ── comclass_threadingmodel — bindgen-style enum-variant names the
//    draft uses. ───────────────────────────────────────────────────
pub const comclass_threadingmodel_ThreadingModel_Apartment: u32 = 1;
pub const comclass_threadingmodel_ThreadingModel_Free: u32 = 2;
pub const comclass_threadingmodel_ThreadingModel_No: u32 = 3;
pub const comclass_threadingmodel_ThreadingModel_Both: u32 = 4;
pub const comclass_threadingmodel_ThreadingModel_Neutral: u32 = 5;
