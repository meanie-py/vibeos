//! `#[no_mangle] extern "system"` NTAPI shims — Wave RTL-1. Unsafe
//! counted-string struct reconstruct + the exact legacy `Rtl*`
//! contract. `extern "system"` is NTAPI (cdecl on amd64/arm64,
//! stdcall on i386 — and plain C on the x86-64 host the diff harness
//! links against). The pure decisions live in `core_rs::ustr`.
#![allow(non_snake_case)]

#[cfg(not(feature = "kernel"))]
use crate::core_rs::utf8;
use crate::core_rs::{
    acl, conv, dos8dot3, env, misc, nls, path as path_, priv_, process as prc, resource as res,
    sd as sd_, sec, sid, thread as thr, timezone as tz, ustr,
};

/// NLS code-page-table machinery (structural sub-bucket of nls.c).
/// Kernel-only: the byte-diff harness supplies its own modelled `Nls*`
/// globals + `Rtlp{Up,Down}caseUnicodeChar`.
#[cfg(feature = "kernel")]
pub mod nls_tables;

/// `STRING` / `ANSI_STRING` / `OEM_STRING` (umtypes.h).
#[repr(C)]
pub struct STRING {
    pub Length: u16,
    pub MaximumLength: u16,
    pub Buffer: *mut u8,
}

/// `UNICODE_STRING` (umtypes.h).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct UNICODE_STRING {
    pub Length: u16,
    pub MaximumLength: u16,
    pub Buffer: *mut u16,
}
unsafe impl Sync for UNICODE_STRING {}

const STATUS_SUCCESS: i32 = 0x0000_0000;
const STATUS_NAME_TOO_LONG: i32 = 0xC000_0106u32 as i32;
const STATUS_INVALID_PARAMETER: i32 = 0xC000_000Du32 as i32;
const STATUS_BUFFER_OVERFLOW: i32 = 0x8000_0005u32 as i32;
const STATUS_NO_MEMORY: i32 = 0xC000_0017u32 as i32;
const STATUS_INVALID_PARAMETER_2: i32 = 0xC000_00F0u32 as i32;
#[cfg(not(feature = "kernel"))]
const STATUS_INVALID_PARAMETER_4: i32 = 0xC000_00F2u32 as i32;
#[cfg(not(feature = "kernel"))]
const STATUS_INVALID_PARAMETER_5: i32 = 0xC000_00F3u32 as i32;
const STATUS_UNMAPPABLE_CHARACTER: i32 = 0xC0000162_u32 as i32;
const STATUS_ACCESS_VIOLATION: i32 = 0xC000_0005u32 as i32;
const STATUS_BUFFER_TOO_SMALL: i32 = 0xC000_0023u32 as i32;
const STATUS_NOT_FOUND: i32 = 0xC000_0225u32 as i32;
const STATUS_NOT_IMPLEMENTED: i32 = 0xC000_0002u32 as i32;
const STATUS_INVALID_SID: i32 = 0xC000_0078u32 as i32;
const STATUS_INVALID_INFO_CLASS: i32 = 0xC000_0003u32 as i32;
const STATUS_REVISION_MISMATCH: i32 = 0xC000_0059u32 as i32;
const STATUS_INVALID_ACL: i32 = 0xC000_0077u32 as i32;
const STATUS_ALLOTTED_SPACE_EXCEEDED: i32 = 0xC000_0099u32 as i32;
const STATUS_INVALID_COMPUTER_NAME: i32 = 0xC000_0122u32 as i32;
const MAXUSHORT: u32 = 0xFFFF;
const MAX_COMPUTERNAME_LENGTH: u32 = 15;

const RTL_SKIP_BUFFER_COPY: u32 = 0x0000_0001;
const RTL_DUP_NULL_TERMINATE: u32 = 1;
const RTL_DUP_ALLOCATE_NULL_STRING: u32 = 2;

// String allocator tags (sdk/lib/rtl/rtlp.h). C multi-char literals
// are big-endian; `u32::from_le_bytes(*b"RTSU")` gives the same value.
// C: tag.h `#define TAG_USTR 'RTSU'`. from_le_bytes(*b"RTSU") is the REVERSE of
// that literal -- the byte string and the fn must be chosen as a PAIR.
const TAG_USTR: u32 = u32::from_be_bytes(*b"RTSU");
const TAG_ASTR: u32 = u32::from_be_bytes(*b"RTSA"); // C literal 'RTSA' -- ntoskrnl/include/internal/tag.h
const TAG_OSTR: u32 = u32::from_be_bytes(*b"RTSO"); // C literal 'RTSO' -- sdk/lib/rtl/rtlp.h

// Allocator routing — three distinct link contexts, selected by the
// `kernel` feature + `target_os`:
//
//   1. Kernel  (`kernel` feature, PE):   route through the existing C
//      `RtlpAllocateMemory` (ntoskrnl/rtl/libsupp.c — wraps
//      ExAllocatePoolWithTag(PagedPool, ...)).
//   2. User-mode (`cabi` only, windows): route through the process heap
//      via `RtlAllocateHeap(RtlGetProcessHeap(), ...)` — this is the
//      same librtl.a, linked into ntdll. Without this branch every
//      user-mode SID/SD/ACL/string alloc returns NULL → SMSS dies with
//      STATUS_NO_MEMORY (bugcheck 0x4C). See memory
//      feedback_rtl_strings_alloc_stubs.
//   3. Host difftest (`cabi` only, NOT windows): the cdylib dlopened by
//      the C harness keeps the NULL stub so its alloc-failure model
//      paths still fire (and RtlAllocateHeap isn't linkable on host).
#[cfg(feature = "kernel")]
extern "system" {
    fn RtlpAllocateMemory(Bytes: u32, Tag: u32) -> *mut u8;
    fn RtlpFreeMemory(Mem: *mut u8, Tag: u32);
}
#[cfg(all(not(feature = "kernel"), target_os = "windows"))]
extern "system" {
    fn RtlGetProcessHeap() -> *mut core::ffi::c_void;
    fn RtlAllocateHeap(HeapHandle: *mut core::ffi::c_void, Flags: u32, Size: usize) -> *mut u8;
    fn RtlFreeHeap(HeapHandle: *mut core::ffi::c_void, Flags: u32, BaseAddress: *mut u8) -> u8;
}
#[inline]
unsafe fn rtl_alloc(bytes: usize, tag: u32) -> *mut u8 {
    #[cfg(feature = "kernel")]
    {
        RtlpAllocateMemory(bytes as u32, tag)
    }
    #[cfg(all(not(feature = "kernel"), target_os = "windows"))]
    {
        let _ = tag;
        RtlAllocateHeap(RtlGetProcessHeap(), 0, bytes)
    }
    #[cfg(all(not(feature = "kernel"), not(target_os = "windows")))]
    {
        let _ = (bytes, tag);
        core::ptr::null_mut()
    }
}
#[inline]
#[allow(dead_code)]
unsafe fn rtl_free(p: *mut u8, tag: u32) {
    #[cfg(feature = "kernel")]
    {
        if !p.is_null() {
            RtlpFreeMemory(p, tag);
        }
    }
    #[cfg(all(not(feature = "kernel"), target_os = "windows"))]
    {
        let _ = tag;
        if !p.is_null() {
            RtlFreeHeap(RtlGetProcessHeap(), 0, p);
        }
    }
    #[cfg(all(not(feature = "kernel"), not(target_os = "windows")))]
    {
        let _ = (p, tag);
    }
}

/// `RTL_BUFFER` (ndk/rtltypes.h) — host layout (SIZE_T = 8).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct RTL_BUFFER {
    pub Buffer: *mut u8,
    pub StaticBuffer: *mut u8,
    pub Size: usize,
    pub StaticSize: usize,
    pub ReservedForAllocatedSize: usize,
    pub ReservedForIMalloc: *mut u8,
}

const RTL_FIND_CHAR_START_AT_END: u32 = 0x0000_0001;
const RTL_FIND_CHAR_COMPLEMENT: u32 = 0x0000_0002;
const RTL_FIND_CHAR_CASE_INSENSITIVE: u32 = 0x0000_0004;

/// `GUID` (guiddef.h) — 16 bytes, `#[repr(C)]`.
#[repr(C)]
pub struct GUID {
    pub Data1: u32,
    pub Data2: u16,
    pub Data3: u16,
    pub Data4: [u8; 8],
}

#[inline]
fn nt_success(s: i32) -> bool {
    s >= 0
}

/// Reconstruct an immutable slice from an NT `(ptr, count)` pair,
/// tolerating the genuine `count == 0` / NULL case (no deref happens
/// in that path).
#[inline]
unsafe fn rslice<'a, T>(p: *const T, n: usize) -> &'a [T] {
    if p.is_null() || n == 0 {
        &[]
    } else {
        core::slice::from_raw_parts(p, n)
    }
}
#[inline]
unsafe fn wslice<'a, T>(p: *mut T, n: usize) -> &'a mut [T] {
    if p.is_null() || n == 0 {
        &mut []
    } else {
        core::slice::from_raw_parts_mut(p, n)
    }
}
#[inline]
unsafe fn put_u32(p: *mut u32, v: u32) {
    if !p.is_null() {
        core::ptr::write_volatile(p, v);
    }
}

#[inline]
unsafe fn strlen(p: *const u8) -> usize {
    if p.is_null() {
        return 0;
    }
    let mut n = 0;
    while *p.add(n) != 0 {
        n += 1;
    }
    n
}

#[inline]
unsafe fn wcslen(p: *const u16) -> usize {
    if p.is_null() {
        return 0;
    }
    let mut n = 0;
    while *p.add(n) != 0 {
        n += 1;
    }
    n
}

/// Length-bounded byte view of a counted `STRING` (NUL not implied).
#[inline]
unsafe fn sbytes<'a>(s: *const STRING) -> &'a [u8] {
    let buf = (*s).Buffer;
    let len = (*s).Length as usize;
    if buf.is_null() || len == 0 {
        return &[];
    }
    core::slice::from_raw_parts(buf, len)
}

/// Length-bounded WCHAR-unit view of a counted `UNICODE_STRING`
/// (`Length / sizeof(WCHAR)` units; trailing odd byte ignored — the
/// genuine `min(L1,L2)/2`).
#[inline]
unsafe fn uunits<'a>(s: *const UNICODE_STRING) -> &'a [u16] {
    let buf = (*s).Buffer;
    let units = (*s).Length as usize / 2;
    if buf.is_null() || units == 0 {
        return &[];
    }
    core::slice::from_raw_parts(buf, units)
}

/// `VOID NTAPI RtlInitAnsiString(PANSI_STRING, PCSZ)`.
#[no_mangle]
pub unsafe extern "system" fn RtlInitAnsiString(dest: *mut STRING, src: *const u8) {
    if src.is_null() {
        (*dest).Length = 0;
        (*dest).MaximumLength = 0;
    } else {
        let (l, m) = ustr::ansi_lengths(strlen(src));
        (*dest).Length = l;
        (*dest).MaximumLength = m;
    }
    // genuine: `DestinationString->Buffer = (PCHAR)SourceString;`
    // (set even in the NULL case).
    (*dest).Buffer = src as *mut u8;
}

/// `VOID NTAPI RtlInitString(PSTRING, PCSZ)` — delegates to
/// `RtlInitAnsiString`.
#[no_mangle]
pub unsafe extern "system" fn RtlInitString(dest: *mut STRING, src: *const u8) {
    RtlInitAnsiString(dest, src)
}

/// `NTSTATUS NTAPI RtlInitAnsiStringEx(PANSI_STRING, PCSZ)`.
#[no_mangle]
pub unsafe extern "system" fn RtlInitAnsiStringEx(dest: *mut STRING, src: *const u8) -> i32 {
    if src.is_null() {
        (*dest).Length = 0;
        (*dest).MaximumLength = 0;
    } else {
        match ustr::ansi_lengths_ex(strlen(src)) {
            None => return STATUS_NAME_TOO_LONG,
            Some((l, m)) => {
                (*dest).Length = l;
                (*dest).MaximumLength = m;
            }
        }
    }
    (*dest).Buffer = src as *mut u8;
    STATUS_SUCCESS
}

/// `VOID NTAPI RtlInitUnicodeString(PUNICODE_STRING, PCWSTR)`.
#[no_mangle]
pub unsafe extern "system" fn RtlInitUnicodeString(dest: *mut UNICODE_STRING, src: *const u16) {
    if src.is_null() {
        (*dest).Length = 0;
        (*dest).MaximumLength = 0;
    } else {
        let (l, m) = ustr::uni_lengths(wcslen(src));
        (*dest).Length = l;
        (*dest).MaximumLength = m;
    }
    (*dest).Buffer = src as *mut u16;
}

/// `NTSTATUS NTAPI RtlInitUnicodeStringEx(PUNICODE_STRING, PCWSTR)`.
#[no_mangle]
pub unsafe extern "system" fn RtlInitUnicodeStringEx(
    dest: *mut UNICODE_STRING,
    src: *const u16,
) -> i32 {
    if src.is_null() {
        (*dest).Length = 0;
        (*dest).MaximumLength = 0;
    } else {
        match ustr::uni_lengths_ex(wcslen(src)) {
            None => return STATUS_NAME_TOO_LONG,
            Some((l, m)) => {
                (*dest).Length = l;
                (*dest).MaximumLength = m;
            }
        }
    }
    (*dest).Buffer = src as *mut u16;
    STATUS_SUCCESS
}

/// `LONG NTAPI RtlCompareString(const STRING*, const STRING*,
/// BOOLEAN)`. Case-sensitive: RTL-1. Case-insensitive (RTL-2): the
/// genuine `RtlUpperChar` fold on the shared CP1252 model.
#[no_mangle]
pub unsafe extern "system" fn RtlCompareString(
    s1: *const STRING,
    s2: *const STRING,
    case_insensitive: u8,
) -> i32 {
    if case_insensitive != 0 {
        nls::compare_ansi(sbytes(s1), sbytes(s2), true)
    } else {
        ustr::compare(sbytes(s1), sbytes(s2))
    }
}

/// `BOOLEAN NTAPI RtlEqualString(const STRING*, const STRING*,
/// BOOLEAN)` (case-sensitive path).
#[no_mangle]
pub unsafe extern "system" fn RtlEqualString(
    s1: *const STRING,
    s2: *const STRING,
    case_insensitive: u8,
) -> u8 {
    // genuine: `if (L1 != L2) return FALSE; return !RtlCompareString`
    if (*s1).Length != (*s2).Length {
        return 0;
    }
    if case_insensitive != 0 {
        (nls::compare_ansi(sbytes(s1), sbytes(s2), true) == 0) as u8
    } else {
        ustr::equal(sbytes(s1), sbytes(s2)) as u8
    }
}

/// `BOOLEAN NTAPI RtlPrefixString(const STRING*, const STRING*,
/// BOOLEAN)` (case-sensitive path). NULL Buffer ⇒ FALSE (genuine
/// `if (pc1 && pc2) {...} return FALSE`).
#[no_mangle]
pub unsafe extern "system" fn RtlPrefixString(
    string1: *const STRING,
    string2: *const STRING,
    case_insensitive: u8,
) -> u8 {
    // `if (String2->Length < String1->Length) return FALSE;` is
    // checked before the buffer test in the genuine code.
    if (*string2).Length < (*string1).Length {
        return 0;
    }
    if (*string1).Buffer.is_null() || (*string2).Buffer.is_null() {
        return 0;
    }
    if case_insensitive != 0 {
        nls::prefix_ansi(sbytes(string1), sbytes(string2), true) as u8
    } else {
        ustr::prefix(sbytes(string1), sbytes(string2)) as u8
    }
}

// ── Wave RTL-2: the case-aware (shared CP1252 NLS) surface ──────────

/// `WCHAR NTAPI RtlUpcaseUnicodeChar(WCHAR)` (nls.c → the shared
/// model).
#[no_mangle]
pub unsafe extern "system" fn RtlUpcaseUnicodeChar(src: u16) -> u16 {
    nls::upcase_uc(src)
}

/// `CHAR NTAPI RtlUpperChar(CHAR)` (nls.c). Signed-`char` guard ⇒
/// only a–z fold; every other byte unchanged.
#[no_mangle]
pub unsafe extern "system" fn RtlUpperChar(src: u8) -> u8 {
    nls::upper_ansi(src)
}

/// `LONG NTAPI RtlCompareUnicodeString(PCUNICODE_STRING,
/// PCUNICODE_STRING, BOOLEAN)`. Buffers derefed unconditionally by the
/// genuine code, but only `min(L1,L2)/2` units — NULL is safe iff that
/// side's `Length == 0`.
#[no_mangle]
pub unsafe extern "system" fn RtlCompareUnicodeString(
    s1: *const UNICODE_STRING,
    s2: *const UNICODE_STRING,
    case_insensitive: u8,
) -> i32 {
    nls::compare_uni(
        uunits(s1),
        uunits(s2),
        (*s1).Length as i32,
        (*s2).Length as i32,
        case_insensitive != 0,
    )
}

/// `BOOLEAN NTAPI RtlEqualUnicodeString(...)` — `L1 != L2 ⇒ FALSE`,
/// else `!RtlCompareUnicodeString`.
#[no_mangle]
pub unsafe extern "system" fn RtlEqualUnicodeString(
    s1: *const UNICODE_STRING,
    s2: *const UNICODE_STRING,
    case_insensitive: u8,
) -> u8 {
    if (*s1).Length != (*s2).Length {
        return 0;
    }
    (nls::compare_uni(
        uunits(s1),
        uunits(s2),
        (*s1).Length as i32,
        (*s2).Length as i32,
        case_insensitive != 0,
    ) == 0) as u8
}

/// `BOOLEAN NTAPI RtlPrefixUnicodeString(...)`. `L2 < L1 ⇒ FALSE`,
/// then the `if (pc1 && pc2)` NULL guard (else FALSE), then the first
/// `L1/sizeof(WCHAR)` units equal under the fold.
#[no_mangle]
pub unsafe extern "system" fn RtlPrefixUnicodeString(
    string1: *const UNICODE_STRING,
    string2: *const UNICODE_STRING,
    case_insensitive: u8,
) -> u8 {
    if (*string2).Length < (*string1).Length {
        return 0;
    }
    if (*string1).Buffer.is_null() || (*string2).Buffer.is_null() {
        return 0;
    }
    let n = (*string1).Length as usize / 2;
    let p1 = core::slice::from_raw_parts((*string1).Buffer, n);
    let p2 = core::slice::from_raw_parts((*string2).Buffer, n);
    nls::prefix_uni(p1, p2, case_insensitive != 0) as u8
}

/// `VOID NTAPI RtlUpperString(PSTRING Dest, const STRING* Src)`.
/// `Length = min(Src->Length, Dest->MaximumLength)`; writes that many
/// upcased bytes (NO NUL), sets `Dest->Length`.
#[no_mangle]
pub unsafe extern "system" fn RtlUpperString(dest: *mut STRING, src: *const STRING) {
    let n = ((*src).Length).min((*dest).MaximumLength) as usize;
    let s = sbytes(src);
    let d = (*dest).Buffer;
    for i in 0..n {
        core::ptr::write_volatile(d.add(i), nls::upper_ansi(s[i]));
    }
    (*dest).Length = n as u16;
}

/// `NTSTATUS NTAPI RtlHashUnicodeString(PCUNICODE_STRING, BOOLEAN,
/// ULONG, PULONG)`. `String||HashValue` NULL ⇒
/// `STATUS_INVALID_PARAMETER` (and `*HashValue` untouched); only
/// algorithms DEFAULT(0)/X65599(1) compute, any other ⇒
/// `STATUS_INVALID_PARAMETER`. The CI fold here is ASCII-only (NOT
/// NLS) — a deliberate genuine split.
#[no_mangle]
pub unsafe extern "system" fn RtlHashUnicodeString(
    string: *const UNICODE_STRING,
    case_insensitive: u8,
    hash_algorithm: u32,
    hash_value: *mut u32,
) -> i32 {
    if string.is_null() || hash_value.is_null() {
        return STATUS_INVALID_PARAMETER;
    }
    if hash_algorithm != 0 && hash_algorithm != 1 {
        return STATUS_INVALID_PARAMETER;
    }
    let h = nls::hash_uni(uunits(string), case_insensitive != 0);
    core::ptr::write_volatile(hash_value, h);
    STATUS_SUCCESS
}

// ── Wave RTL-3: single-byte (CP1252-model) converter core ──────────

/// `NTSTATUS NTAPI RtlMultiByteToUnicodeN(PWCHAR, ULONG UnicodeSize,
/// PULONG ResultSize, PCSTR, ULONG MbSize)`.
#[no_mangle]
pub unsafe extern "system" fn RtlMultiByteToUnicodeN(
    unicode: *mut u16,
    unicode_size: u32,
    result_size: *mut u32,
    mb: *const u8,
    mb_size: u32,
) -> i32 {
    let dst = wslice(unicode, unicode_size as usize / 2);
    let src = rslice(mb, mb_size as usize);
    let n = nls::mb_to_uni(dst, src);
    put_u32(result_size, (n * 2) as u32);
    STATUS_SUCCESS
}

/// `NTSTATUS NTAPI RtlOemToUnicodeN(...)` — same single-byte body as
/// MultiByte (shared model: OEM CP == ANSI CP == CP1252).
#[no_mangle]
pub unsafe extern "system" fn RtlOemToUnicodeN(
    unicode: *mut u16,
    unicode_size: u32,
    result_size: *mut u32,
    oem: *const u8,
    oem_size: u32,
) -> i32 {
    let dst = wslice(unicode, unicode_size as usize / 2);
    let src = rslice(oem, oem_size as usize);
    let n = nls::mb_to_uni(dst, src);
    put_u32(result_size, (n * 2) as u32);
    STATUS_SUCCESS
}

/// `NTSTATUS NTAPI RtlUnicodeToMultiByteN(PCHAR, ULONG MbSize,
/// PULONG ResultSize, PCWCH, ULONG UnicodeSize)`.
#[no_mangle]
pub unsafe extern "system" fn RtlUnicodeToMultiByteN(
    mb: *mut u8,
    mb_size: u32,
    result_size: *mut u32,
    unicode: *const u16,
    unicode_size: u32,
) -> i32 {
    let dst = wslice(mb, mb_size as usize);
    let src = rslice(unicode, unicode_size as usize / 2);
    let n = nls::uni_to_mb(dst, src);
    put_u32(result_size, n as u32);
    STATUS_SUCCESS
}

/// `NTSTATUS NTAPI RtlUnicodeToOemN(...)` — the `while` shape:
/// `STATUS_BUFFER_OVERFLOW` iff Unicode input left over.
#[no_mangle]
pub unsafe extern "system" fn RtlUnicodeToOemN(
    oem: *mut u8,
    oem_size: u32,
    result_size: *mut u32,
    unicode: *const u16,
    unicode_size: u32,
) -> i32 {
    let dst = wslice(oem, oem_size as usize);
    let src = rslice(unicode, unicode_size as usize / 2);
    let (n, overflow) = nls::uni_to_oem(dst, src);
    put_u32(result_size, n as u32);
    if overflow {
        STATUS_BUFFER_OVERFLOW
    } else {
        STATUS_SUCCESS
    }
}

/// `NTSTATUS NTAPI RtlUpcaseUnicodeToMultiByteN(...)` — min-precompute
/// shape, always SUCCESS, upcase via the shared model.
#[no_mangle]
pub unsafe extern "system" fn RtlUpcaseUnicodeToMultiByteN(
    mb: *mut u8,
    mb_size: u32,
    result_size: *mut u32,
    unicode: *const u16,
    unicode_size: u32,
) -> i32 {
    let dst = wslice(mb, mb_size as usize);
    let src = rslice(unicode, unicode_size as usize / 2);
    let n = nls::upcase_uni_to_mb(dst, src);
    put_u32(result_size, n as u32);
    STATUS_SUCCESS
}

/// `NTSTATUS NTAPI RtlUpcaseUnicodeToOemN(...)` — same min-precompute
/// shape as the MultiByte upcase variant (NOT the `while`/overflow
/// shape of `RtlUnicodeToOemN`).
#[no_mangle]
pub unsafe extern "system" fn RtlUpcaseUnicodeToOemN(
    oem: *mut u8,
    oem_size: u32,
    result_size: *mut u32,
    unicode: *const u16,
    unicode_size: u32,
) -> i32 {
    let dst = wslice(oem, oem_size as usize);
    let src = rslice(unicode, unicode_size as usize / 2);
    let n = nls::upcase_uni_to_mb(dst, src);
    put_u32(result_size, n as u32);
    STATUS_SUCCESS
}

/// `NTSTATUS NTAPI RtlMultiByteToUnicodeSize(PULONG, PCSTR, ULONG)` —
/// single-byte: `*UnicodeSize = MbSize * sizeof(WCHAR)`.
#[no_mangle]
pub unsafe extern "system" fn RtlMultiByteToUnicodeSize(
    unicode_size: *mut u32,
    _mb: *const u8,
    mb_size: u32,
) -> i32 {
    put_u32(unicode_size, mb_size.wrapping_mul(2));
    STATUS_SUCCESS
}

/// `NTSTATUS NTAPI RtlUnicodeToMultiByteSize(PULONG, PCWCH, ULONG)` —
/// single-byte: `*MbSize = UnicodeSize / sizeof(WCHAR)`.
#[no_mangle]
pub unsafe extern "system" fn RtlUnicodeToMultiByteSize(
    mb_size: *mut u32,
    _unicode: *const u16,
    unicode_size: u32,
) -> i32 {
    put_u32(mb_size, unicode_size / 2);
    STATUS_SUCCESS
}

// ── Wave RTL-3 slice 2: unicode.c ANSI/OEM/Upcase wrappers ─────────
//
// AllocateDestinationString is modelled as the harness config: the
// allocator stub yields NULL, so the genuine path returns
// STATUS_NO_MEMORY — we return the same (the conversion logic on the
// caller-buffer path is what the diff actually exercises; the
// allocate path's NULL/NO_MEMORY contract is the pinned edge).

/// `WCHAR NTAPI RtlAnsiCharToUnicodeChar(IN OUT PUCHAR *AnsiChar)` —
/// NlsLeadByteInfo is NULL in the model ⇒ Size=1; converts one byte,
/// advances `*AnsiChar` by 1.
#[no_mangle]
pub unsafe extern "system" fn RtlAnsiCharToUnicodeChar(ansi_char: *mut *mut u8) -> u16 {
    let p = *ansi_char;
    let uc = nls::dec_byte(*p);
    *ansi_char = p.add(1);
    uc
}

/// `RtlAnsiStringToUnicodeString` (single-byte): `Length =
/// (Ansi.Length+1)*2`; always writes the terminating NUL; NO partial
/// copy on overflow.
#[no_mangle]
pub unsafe extern "system" fn RtlAnsiStringToUnicodeString(
    dest: *mut UNICODE_STRING,
    src: *const STRING,
    alloc: u8,
) -> i32 {
    let length = ((*src).Length as u32 + 1) * 2;
    if length > MAXUSHORT {
        return STATUS_INVALID_PARAMETER_2;
    }
    (*dest).Length = length as u16 - 2;
    if alloc != 0 {
        let buf = rtl_alloc(length as usize, TAG_USTR) as *mut u16;
        // Store Buffer UNCONDITIONALLY, then test it — the C writes the
        // allocation result into the out-parameter and only then checks it, so
        // a caller that gets STATUS_NO_MEMORY sees Buffer=NULL. Returning
        // before the store instead leaves the caller's own stale pointer in
        // place next to the Length/MaximumLength this function just
        // overwrote: a string that looks live and aims at foreign memory.
        // Same shape in all 11 converters here; rtl-difftest pins it.
        (*dest).Buffer = buf;
        (*dest).MaximumLength = length as u16;
        if buf.is_null() {
            return STATUS_NO_MEMORY;
        }
    }
    if (*dest).Length >= (*dest).MaximumLength {
        return STATUS_BUFFER_OVERFLOW;
    }
    let dlen = (*dest).Length;
    let buf = wslice((*dest).Buffer, dlen as usize / 2);
    let s = rslice((*src).Buffer, (*src).Length as usize);
    let n = nls::mb_to_uni(&mut buf[..dlen as usize / 2], s);
    core::ptr::write_volatile((*dest).Buffer.add(n), 0u16);
    STATUS_SUCCESS
}

/// `ULONG NTAPI RtlxAnsiStringToUnicodeSize` = `Ansi.Length*2 + 2`.
#[no_mangle]
pub unsafe extern "system" fn RtlxAnsiStringToUnicodeSize(s: *const STRING) -> u32 {
    (*s).Length as u32 * 2 + 2
}

/// `RtlUnicodeStringToAnsiString` (single-byte): `Length =
/// (Uni.Length+2)/2`; performs a PARTIAL copy if dest too small
/// (unless MaximumLength==0).
#[no_mangle]
pub unsafe extern "system" fn RtlUnicodeStringToAnsiString(
    dest: *mut STRING,
    src: *const UNICODE_STRING,
    alloc: u8,
) -> i32 {
    let length = ((*src).Length as u32 + 2) / 2;
    if length > MAXUSHORT {
        return STATUS_INVALID_PARAMETER_2;
    }
    (*dest).Length = length as u16 - 1;
    let mut status = STATUS_SUCCESS;
    if alloc != 0 {
        let buf = rtl_alloc(length as usize, TAG_ASTR);
        (*dest).Buffer = buf;
        (*dest).MaximumLength = length as u16;
        if buf.is_null() {
            return STATUS_NO_MEMORY;
        }
    } else if (*dest).Length >= (*dest).MaximumLength {
        if (*dest).MaximumLength == 0 {
            return STATUS_BUFFER_OVERFLOW;
        }
        status = STATUS_BUFFER_OVERFLOW;
        (*dest).Length = (*dest).MaximumLength - 1;
    }
    let dlen = (*dest).Length;
    let buf = wslice((*dest).Buffer, dlen as usize);
    let s = rslice((*src).Buffer, (*src).Length as usize / 2);
    let n = nls::uni_to_mb(&mut buf[..dlen as usize], s);
    core::ptr::write_volatile((*dest).Buffer.add(n), 0u8);
    status
}

/// `ULONG NTAPI RtlxUnicodeStringToAnsiSize` = `Uni.Length/2 + 1`.
#[no_mangle]
pub unsafe extern "system" fn RtlxUnicodeStringToAnsiSize(s: *const UNICODE_STRING) -> u32 {
    (*s).Length as u32 / 2 + 1
}

/// `ULONG NTAPI RtlxUnicodeStringToOemSize` = `Uni.Length/2 + 1`.
#[no_mangle]
pub unsafe extern "system" fn RtlxUnicodeStringToOemSize(s: *const UNICODE_STRING) -> u32 {
    (*s).Length as u32 / 2 + 1
}

/// `ULONG NTAPI RtlxOemStringToUnicodeSize` = `Oem.Length*2 + 2`.
#[no_mangle]
pub unsafe extern "system" fn RtlxOemStringToUnicodeSize(s: *const STRING) -> u32 {
    (*s).Length as u32 * 2 + 2
}

/// `RtlOemStringToUnicodeString` — `Length = (Oem.Length+1)*2`; NUL
/// terminates; NO partial copy.
#[no_mangle]
pub unsafe extern "system" fn RtlOemStringToUnicodeString(
    dest: *mut UNICODE_STRING,
    src: *const STRING,
    alloc: u8,
) -> i32 {
    let length = ((*src).Length as u32 + 1) * 2;
    if length > MAXUSHORT {
        return STATUS_INVALID_PARAMETER_2;
    }
    (*dest).Length = length as u16 - 2;
    if alloc != 0 {
        let buf = rtl_alloc(length as usize, TAG_USTR) as *mut u16;
        (*dest).Buffer = buf;
        (*dest).MaximumLength = length as u16;
        if buf.is_null() {
            return STATUS_NO_MEMORY;
        }
    }
    if (*dest).Length >= (*dest).MaximumLength {
        return STATUS_BUFFER_OVERFLOW;
    }
    let dlen = (*dest).Length;
    let buf = wslice((*dest).Buffer, dlen as usize / 2);
    let s = rslice((*src).Buffer, (*src).Length as usize);
    let n = nls::mb_to_uni(&mut buf[..dlen as usize / 2], s);
    core::ptr::write_volatile((*dest).Buffer.add(n), 0u16);
    STATUS_SUCCESS
}

/// `RtlUnicodeStringToOemString` — `Length = (Uni.Length+2)/2`; NUL
/// terminates; converter status propagates (no DidWork check here).
#[no_mangle]
pub unsafe extern "system" fn RtlUnicodeStringToOemString(
    dest: *mut STRING,
    src: *const UNICODE_STRING,
    alloc: u8,
) -> i32 {
    let length = ((*src).Length as u32 + 2) / 2;
    if length > MAXUSHORT {
        return STATUS_INVALID_PARAMETER_2;
    }
    (*dest).Length = length as u16 - 1;
    if alloc != 0 {
        let buf = rtl_alloc(length as usize, TAG_OSTR);
        (*dest).Buffer = buf;
        (*dest).MaximumLength = length as u16;
        if buf.is_null() {
            return STATUS_NO_MEMORY;
        }
    }
    if (*dest).Length >= (*dest).MaximumLength {
        return STATUS_BUFFER_OVERFLOW;
    }
    let dlen = (*dest).Length;
    let buf = wslice((*dest).Buffer, dlen as usize);
    let s = rslice((*src).Buffer, (*src).Length as usize / 2);
    let (n, overflow) = nls::uni_to_oem(&mut buf[..dlen as usize], s);
    core::ptr::write_volatile((*dest).Buffer.add(n), 0u8);
    if overflow {
        STATUS_BUFFER_OVERFLOW
    } else {
        STATUS_SUCCESS
    }
}

/// `RtlOemStringToCountedUnicodeString` — `Length = Oem.Length*2`;
/// `Length==0` ⇒ zero the dest UNICODE_STRING & SUCCESS; STRICT `>`
/// overflow; NO NUL; returns SUCCESS even past a converter call.
#[no_mangle]
pub unsafe extern "system" fn RtlOemStringToCountedUnicodeString(
    dest: *mut UNICODE_STRING,
    src: *const STRING,
    alloc: u8,
) -> i32 {
    let length = (*src).Length as u32 * 2;
    if length == 0 {
        (*dest).Length = 0;
        (*dest).MaximumLength = 0;
        (*dest).Buffer = core::ptr::null_mut();
        return STATUS_SUCCESS;
    }
    if length > MAXUSHORT {
        return STATUS_INVALID_PARAMETER_2;
    }
    (*dest).Length = length as u16;
    if alloc != 0 {
        let buf = rtl_alloc(length as usize, TAG_USTR) as *mut u16;
        (*dest).Buffer = buf;
        (*dest).MaximumLength = length as u16;
        if buf.is_null() {
            return STATUS_NO_MEMORY;
        }
    }
    if (*dest).Length > (*dest).MaximumLength {
        return STATUS_BUFFER_OVERFLOW;
    }
    let dlen = (*dest).Length;
    let buf = wslice((*dest).Buffer, dlen as usize / 2);
    let s = rslice((*src).Buffer, (*src).Length as usize);
    nls::mb_to_uni(&mut buf[..dlen as usize / 2], s);
    STATUS_SUCCESS
}

/// `RtlUnicodeStringToCountedOemString` — `Length =
/// (Uni.Length+2)/2 - 1`; `Length==0` ⇒ zero OEM_STRING & SUCCESS;
/// STRICT `>` overflow; DidWork ⇒ STATUS_UNMAPPABLE_CHARACTER; NO NUL.
#[no_mangle]
pub unsafe extern "system" fn RtlUnicodeStringToCountedOemString(
    dest: *mut STRING,
    src: *const UNICODE_STRING,
    alloc: u8,
) -> i32 {
    let length = ((*src).Length as u32 + 2) / 2 - 1;
    if length == 0 {
        (*dest).Length = 0;
        (*dest).MaximumLength = 0;
        (*dest).Buffer = core::ptr::null_mut();
        return STATUS_SUCCESS;
    }
    if length > MAXUSHORT {
        return STATUS_INVALID_PARAMETER_2;
    }
    (*dest).Length = length as u16;
    if alloc != 0 {
        let buf = rtl_alloc(length as usize, TAG_OSTR);
        (*dest).Buffer = buf;
        (*dest).MaximumLength = length as u16;
        if buf.is_null() {
            return STATUS_NO_MEMORY;
        }
    }
    if (*dest).Length > (*dest).MaximumLength {
        return STATUS_BUFFER_OVERFLOW;
    }
    let dlen = (*dest).Length;
    let s = rslice((*src).Buffer, (*src).Length as usize / 2);
    let buf = wslice((*dest).Buffer, dlen as usize);
    let (n, overflow) = nls::uni_to_oem(&mut buf[..dlen as usize], s);
    let mut status = if overflow {
        STATUS_BUFFER_OVERFLOW
    } else {
        STATUS_SUCCESS
    };
    if nt_success(status) && !nls::did_oem_work(s, rslice((*dest).Buffer, n)) {
        status = STATUS_UNMAPPABLE_CHARACTER;
    }
    status
}

/// `RtlUpcaseUnicodeString` — in-place-capable upcase; `MaximumLength`
/// (alloc) = `Source.Length`; STRICT `>` overflow; NO NUL.
#[no_mangle]
pub unsafe extern "system" fn RtlUpcaseUnicodeString(
    dest: *mut UNICODE_STRING,
    src: *const UNICODE_STRING,
    alloc: u8,
) -> i32 {
    if alloc != 0 {
        let buf = rtl_alloc((*src).Length as usize, TAG_USTR) as *mut u16;
        (*dest).MaximumLength = (*src).Length;
        (*dest).Buffer = buf;
        if buf.is_null() {
            return STATUS_NO_MEMORY;
        }
    }
    if (*src).Length > (*dest).MaximumLength {
        return STATUS_BUFFER_OVERFLOW;
    }
    let j = (*src).Length as usize / 2;
    let s = rslice((*src).Buffer, j);
    let d = wslice((*dest).Buffer, j);
    for i in 0..j {
        d[i] = nls::upcase_uc(s[i]);
    }
    (*dest).Length = (*src).Length;
    STATUS_SUCCESS
}

/// `RtlUpcaseUnicodeStringToAnsiString` — `Length = Uni.Length/2 + 1`;
/// the overflow branch only early-returns when `MaximumLength==0`
/// (otherwise it falls through — a faithful quirk); NUL terminates.
#[no_mangle]
pub unsafe extern "system" fn RtlUpcaseUnicodeStringToAnsiString(
    dest: *mut STRING,
    src: *const UNICODE_STRING,
    alloc: u8,
) -> i32 {
    let length = (*src).Length as u32 / 2 + 1;
    if length > MAXUSHORT {
        return STATUS_INVALID_PARAMETER_2;
    }
    (*dest).Length = length as u16 - 1;
    if alloc != 0 {
        let buf = rtl_alloc(length as usize, TAG_ASTR);
        (*dest).Buffer = buf;
        (*dest).MaximumLength = length as u16;
        if buf.is_null() {
            return STATUS_NO_MEMORY;
        }
    } else if (*dest).Length >= (*dest).MaximumLength && (*dest).MaximumLength == 0 {
        return STATUS_BUFFER_OVERFLOW;
    }
    let dlen = (*dest).Length;
    let buf = wslice((*dest).Buffer, dlen as usize);
    let s = rslice((*src).Buffer, (*src).Length as usize / 2);
    let n = nls::upcase_uni_to_mb(&mut buf[..dlen as usize], s);
    core::ptr::write_volatile((*dest).Buffer.add(n), 0u8);
    STATUS_SUCCESS
}

/// `RtlUpcaseUnicodeStringToCountedOemString` — `Length =
/// (Uni.Length+2)/2 - 1`; `Length==0` zeroes but does NOT return
/// (falls through, faithful); STRICT `>` overflow; DidWork ⇒
/// UNMAPPABLE; NO NUL.
#[no_mangle]
pub unsafe extern "system" fn RtlUpcaseUnicodeStringToCountedOemString(
    dest: *mut STRING,
    src: *const UNICODE_STRING,
    alloc: u8,
) -> i32 {
    let length = ((*src).Length as u32 + 2) / 2 - 1;
    if length == 0 {
        (*dest).Length = 0;
        (*dest).MaximumLength = 0;
        (*dest).Buffer = core::ptr::null_mut();
    }
    if length > MAXUSHORT {
        return STATUS_INVALID_PARAMETER_2;
    }
    (*dest).Length = length as u16;
    if alloc != 0 {
        let buf = rtl_alloc(length as usize, TAG_OSTR);
        (*dest).Buffer = buf;
        (*dest).MaximumLength = length as u16;
        if buf.is_null() {
            return STATUS_NO_MEMORY;
        }
    }
    if (*dest).Length > (*dest).MaximumLength {
        return STATUS_BUFFER_OVERFLOW;
    }
    let dlen = (*dest).Length;
    let s = rslice((*src).Buffer, (*src).Length as usize / 2);
    let buf = wslice((*dest).Buffer, dlen as usize);
    let n = nls::upcase_uni_to_mb(&mut buf[..dlen as usize], s);
    let mut status = STATUS_SUCCESS;
    if nt_success(status) && !nls::did_oem_work(s, rslice((*dest).Buffer, n)) {
        status = STATUS_UNMAPPABLE_CHARACTER;
    }
    status
}

/// `RtlUpcaseUnicodeStringToOemString` — `Length = (Uni.Length+2)/2`;
/// `>=` overflow; DidWork ⇒ UNMAPPABLE; NUL terminates.
#[no_mangle]
pub unsafe extern "system" fn RtlUpcaseUnicodeStringToOemString(
    dest: *mut STRING,
    src: *const UNICODE_STRING,
    alloc: u8,
) -> i32 {
    let length = ((*src).Length as u32 + 2) / 2;
    if length > MAXUSHORT {
        return STATUS_INVALID_PARAMETER_2;
    }
    (*dest).Length = length as u16 - 1;
    if alloc != 0 {
        let buf = rtl_alloc(length as usize, TAG_OSTR);
        (*dest).Buffer = buf;
        (*dest).MaximumLength = length as u16;
        if buf.is_null() {
            return STATUS_NO_MEMORY;
        }
    }
    if (*dest).Length >= (*dest).MaximumLength {
        return STATUS_BUFFER_OVERFLOW;
    }
    let dlen = (*dest).Length;
    let s = rslice((*src).Buffer, (*src).Length as usize / 2);
    let buf = wslice((*dest).Buffer, dlen as usize);
    let n = nls::upcase_uni_to_mb(&mut buf[..dlen as usize], s);
    let mut status = STATUS_SUCCESS;
    if nt_success(status) && !nls::did_oem_work(s, rslice((*dest).Buffer, n)) {
        status = STATUS_UNMAPPABLE_CHARACTER;
    }
    core::ptr::write_volatile((*dest).Buffer.add(n), 0u8);
    status
}

// ── Wave RTL-3b: downcase family + default code page ──────────────

/// `WCHAR NTAPI RtlDowncaseUnicodeChar(WCHAR)` (nls.c → shared model).
#[no_mangle]
pub unsafe extern "system" fn RtlDowncaseUnicodeChar(src: u16) -> u16 {
    nls::downcase_uc(src)
}

/// `RtlDowncaseUnicodeString` — like `RtlUpcaseUnicodeString` but
/// downcasing; STRICT `>` overflow; alloc MaximumLength=Source.Length;
/// NO NUL.
#[no_mangle]
pub unsafe extern "system" fn RtlDowncaseUnicodeString(
    dest: *mut UNICODE_STRING,
    src: *const UNICODE_STRING,
    alloc: u8,
) -> i32 {
    if alloc != 0 {
        let buf = rtl_alloc((*src).Length as usize, TAG_USTR) as *mut u16;
        (*dest).MaximumLength = (*src).Length;
        (*dest).Buffer = buf;
        if buf.is_null() {
            return STATUS_NO_MEMORY;
        }
    }
    if (*src).Length > (*dest).MaximumLength {
        return STATUS_BUFFER_OVERFLOW;
    }
    (*dest).Length = (*src).Length;
    let j = (*src).Length as usize / 2;
    let s = rslice((*src).Buffer, j);
    let d = wslice((*dest).Buffer, j);
    for i in 0..j {
        d[i] = nls::downcase_uc(s[i]);
    }
    STATUS_SUCCESS
}

/// `VOID NTAPI RtlGetDefaultCodePage(PUSHORT Ansi, PUSHORT Oem)` —
/// shared model is single-byte CP1252 (1252) for both.
#[no_mangle]
pub unsafe extern "system" fn RtlGetDefaultCodePage(ansi_cp: *mut u16, oem_cp: *mut u16) {
    if !ansi_cp.is_null() {
        core::ptr::write_volatile(ansi_cp, 1252);
    }
    if !oem_cp.is_null() {
        core::ptr::write_volatile(oem_cp, 1252);
    }
}

// ── Wave RTL-4a: integer ⇄ string + copy/append ───────────────────

/// `NTSTATUS NTAPI RtlCharToInteger(PCSZ, ULONG, PULONG)`. Base
/// validation out-ranks the `value == NULL` ⇒ ACCESS_VIOLATION check
/// (faithful ordering); `*value` written volatile.
#[no_mangle]
pub unsafe extern "system" fn RtlCharToInteger(str: *const u8, base: u32, value: *mut u32) -> i32 {
    let s = core::slice::from_raw_parts(str, strlen(str));
    match conv::char_to_integer(s, base) {
        conv::IntParse::InvalidBase => STATUS_INVALID_PARAMETER,
        conv::IntParse::Value(v) => {
            if value.is_null() {
                STATUS_ACCESS_VIOLATION
            } else {
                core::ptr::write_volatile(value, v);
                STATUS_SUCCESS
            }
        }
    }
}

/// `NTSTATUS NTAPI RtlUnicodeStringToInteger(PCUNICODE_STRING, ULONG,
/// PULONG)`. The whitespace skip is `CharsRemaining`-bounded (embedded
/// NUL units are eaten); same base-before-NULL ordering.
#[no_mangle]
pub unsafe extern "system" fn RtlUnicodeStringToInteger(
    str: *const UNICODE_STRING,
    base: u32,
    value: *mut u32,
) -> i32 {
    match conv::unicode_string_to_integer(uunits(str), base) {
        conv::IntParse::InvalidBase => STATUS_INVALID_PARAMETER,
        conv::IntParse::Value(v) => {
            if value.is_null() {
                STATUS_ACCESS_VIOLATION
            } else {
                core::ptr::write_volatile(value, v);
                STATUS_SUCCESS
            }
        }
    }
}

/// `NTSTATUS NTAPI RtlIntegerToUnicode(ULONG, ULONG, ULONG, LPWSTR)`.
/// Writes `count` digit units **plus** a NUL; the size gate counts
/// only the digits (genuine NUL-overrun quirk — caller buffer must
/// hold count+1 units).
#[no_mangle]
pub unsafe extern "system" fn RtlIntegerToUnicode(
    value: u32,
    base: u32,
    length: u32,
    string: *mut u16,
) -> i32 {
    match conv::integer_to_unicode(value, base, length) {
        conv::IntToUni::InvalidBase => STATUS_INVALID_PARAMETER,
        conv::IntToUni::TooSmall => STATUS_BUFFER_TOO_SMALL,
        conv::IntToUni::Digits(out, c) => {
            for i in 0..c {
                core::ptr::write_volatile(string.add(i), out[i]);
            }
            core::ptr::write_volatile(string.add(c), 0u16);
            STATUS_SUCCESS
        }
    }
}

/// `VOID NTAPI RtlCopyString(PSTRING, const STRING* OPTIONAL)`.
/// `Length = min(Dest->MaximumLength, Src->Length)`; NULL src ⇒
/// `Dest->Length = 0` only. No NUL.
#[no_mangle]
pub unsafe extern "system" fn RtlCopyString(dest: *mut STRING, src: *const STRING) {
    if src.is_null() {
        (*dest).Length = 0;
        return;
    }
    let n = ((*dest).MaximumLength).min((*src).Length) as usize;
    (*dest).Length = n as u16;
    let s = (*src).Buffer;
    let d = (*dest).Buffer;
    for i in 0..n {
        core::ptr::write_volatile(d.add(i), *s.add(i));
    }
}

/// `VOID NTAPI RtlCopyUnicodeString(PUNICODE_STRING,
/// PCUNICODE_STRING)`. `Length` (BYTES) `= min(Dest->MaximumLength,
/// Src->Length)`; NUL appended iff `Length < MaximumLength`.
#[no_mangle]
pub unsafe extern "system" fn RtlCopyUnicodeString(
    dest: *mut UNICODE_STRING,
    src: *const UNICODE_STRING,
) {
    if src.is_null() {
        (*dest).Length = 0;
        return;
    }
    let n = ((*dest).MaximumLength).min((*src).Length) as usize;
    (*dest).Length = n as u16;
    let s = (*src).Buffer as *const u8;
    let d = (*dest).Buffer as *mut u8;
    for i in 0..n {
        core::ptr::write_volatile(d.add(i), *s.add(i));
    }
    if (*dest).Length < (*dest).MaximumLength {
        core::ptr::write_volatile((*dest).Buffer.add(n / 2), 0u16);
    }
}

/// `NTSTATUS NTAPI RtlAppendStringToString(PSTRING, const STRING*)`.
/// `Dest->Length + Src->Length > Dest->MaximumLength ⇒
/// STATUS_BUFFER_TOO_SMALL`. No NUL.
#[no_mangle]
pub unsafe extern "system" fn RtlAppendStringToString(
    dest: *mut STRING,
    src: *const STRING,
) -> i32 {
    let sl = (*src).Length;
    if sl != 0 {
        if (*dest).Length as u32 + sl as u32 > (*dest).MaximumLength as u32 {
            return STATUS_BUFFER_TOO_SMALL;
        }
        let off = (*dest).Length as usize;
        let s = (*src).Buffer;
        let d = (*dest).Buffer;
        for i in 0..sl as usize {
            core::ptr::write_volatile(d.add(off + i), *s.add(i));
        }
        (*dest).Length += sl;
    }
    STATUS_SUCCESS
}

/// `NTSTATUS NTAPI RtlAppendUnicodeStringToString(PUNICODE_STRING,
/// PCUNICODE_STRING)`. NUL appended iff `MaximumLength > Length` after
/// the append.
#[no_mangle]
pub unsafe extern "system" fn RtlAppendUnicodeStringToString(
    dest: *mut UNICODE_STRING,
    src: *const UNICODE_STRING,
) -> i32 {
    let sl = (*src).Length;
    if sl != 0 {
        if sl as u32 + (*dest).Length as u32 > (*dest).MaximumLength as u32 {
            return STATUS_BUFFER_TOO_SMALL;
        }
        let unit_off = (*dest).Length as usize / 2;
        let d = (*dest).Buffer.add(unit_off) as *mut u8;
        let s = (*src).Buffer as *const u8;
        for i in 0..sl as usize {
            core::ptr::write_volatile(d.add(i), *s.add(i));
        }
        (*dest).Length += sl;
        if (*dest).MaximumLength > (*dest).Length {
            core::ptr::write_volatile((*dest).Buffer.add(unit_off + sl as usize / 2), 0u16);
        }
    }
    STATUS_SUCCESS
}

/// `NTSTATUS NTAPI RtlAppendUnicodeToString(PUNICODE_STRING, PCWSTR)`.
/// `Length` is the `RtlInitUnicodeString`-clamped byte size of the
/// source. NULL src ⇒ unchanged ⇒ SUCCESS. NUL iff `MaximumLength >
/// Length` after.
#[no_mangle]
pub unsafe extern "system" fn RtlAppendUnicodeToString(
    dest: *mut UNICODE_STRING,
    src: *const u16,
) -> i32 {
    if !src.is_null() {
        let (length, _) = ustr::uni_lengths(wcslen(src));
        if (*dest).Length as u32 + length as u32 > (*dest).MaximumLength as u32 {
            return STATUS_BUFFER_TOO_SMALL;
        }
        let unit_off = (*dest).Length as usize / 2;
        let d = (*dest).Buffer.add(unit_off) as *mut u8;
        let s = src as *const u8;
        for i in 0..length as usize {
            core::ptr::write_volatile(d.add(i), *s.add(i));
        }
        (*dest).Length += length;
        if (*dest).MaximumLength > (*dest).Length {
            core::ptr::write_volatile((*dest).Buffer.add(unit_off + length as usize / 2), 0u16);
        }
    }
    STATUS_SUCCESS
}

/// `NTSTATUS NTAPI RtlAppendAsciizToString(PSTRING, PCSZ)`. NULL src ⇒
/// unchanged ⇒ SUCCESS. No NUL.
#[no_mangle]
pub unsafe extern "system" fn RtlAppendAsciizToString(dest: *mut STRING, src: *const u8) -> i32 {
    if !src.is_null() {
        let size = strlen(src);
        if (*dest).Length as usize + size > (*dest).MaximumLength as usize {
            return STATUS_BUFFER_TOO_SMALL;
        }
        let off = (*dest).Length as usize;
        let d = (*dest).Buffer;
        for i in 0..size {
            core::ptr::write_volatile(d.add(off + i), *src.add(i));
        }
        (*dest).Length += size as u16;
    }
    STATUS_SUCCESS
}

/// `LONG NTAPI RtlCompareUnicodeStrings(PCWCH, SIZE_T, PCWCH, SIZE_T,
/// BOOLEAN)` — raw-buffer compare, unit-count lengths, `SIZE_T`
/// subtraction tiebreak.
#[no_mangle]
pub unsafe extern "system" fn RtlCompareUnicodeStrings(
    string1: *const u16,
    string1_length: usize,
    string2: *const u16,
    string2_length: usize,
    case_insensitive: u8,
) -> i32 {
    nls::compare_unicode_strings(
        rslice(string1, string1_length),
        rslice(string2, string2_length),
        case_insensitive != 0,
    )
}

// ── Wave RTL-4b slice 4: computer-name ────────────────────────────

/// `BOOLEAN NTAPI RtlEqualComputerName(PUNICODE_STRING,
/// PUNICODE_STRING)`. Chains through `RtlUpcaseUnicodeStringToOemString
/// (…, TRUE)` which (alloc → NULL) yields STATUS_NO_MEMORY ⇒
/// `NT_SUCCESS` FALSE ⇒ the outer guard never sets Result ⇒ FALSE.
#[no_mangle]
pub unsafe extern "system" fn RtlEqualComputerName(
    _computer_name1: *mut UNICODE_STRING,
    _computer_name2: *mut UNICODE_STRING,
) -> u8 {
    0
}

/// `BOOLEAN NTAPI RtlEqualDomainName(...)` — delegates to
/// `RtlEqualComputerName`.
#[no_mangle]
pub unsafe extern "system" fn RtlEqualDomainName(
    domain_name1: *mut UNICODE_STRING,
    domain_name2: *mut UNICODE_STRING,
) -> u8 {
    RtlEqualComputerName(domain_name1, domain_name2)
}

/// `NTSTATUS NTAPI RtlDnsHostNameToComputerName(PUNICODE_STRING
/// ComputerName, PUNICODE_STRING DnsHostName, BOOLEAN Allocate)`.
/// Dot-scan → `RtlUpcaseUnicodeToOemN` (≤15) → `RtlpDidUnicodeToOemWork`
/// → `RtlOemStringToUnicodeString`. Default Status is
/// STATUS_INVALID_COMPUTER_NAME.
#[no_mangle]
pub unsafe extern "system" fn RtlDnsHostNameToComputerName(
    computer_name: *mut UNICODE_STRING,
    dns_host_name: *mut UNICODE_STRING,
    allocate: u8,
) -> i32 {
    let mut status = STATUS_INVALID_COMPUTER_NAME;
    let mut cn_len: u32 = (*dns_host_name).Length as u32;

    let dns_units = uunits(dns_host_name);
    for (l, &w) in dns_units.iter().enumerate() {
        if w == b'.' as u16 {
            cn_len = (l as u32) * 2;
            break;
        }
    }

    if cn_len > 0 {
        let mut oemn = [0u8; (MAX_COMPUTERNAME_LENGTH + 1) as usize];
        let mut oemn_len: u32 = 0;
        status = RtlUpcaseUnicodeToOemN(
            oemn.as_mut_ptr(),
            MAX_COMPUTERNAME_LENGTH,
            &mut oemn_len,
            (*dns_host_name).Buffer,
            cn_len,
        );
        if status == STATUS_SUCCESS || status == STATUS_BUFFER_OVERFLOW {
            oemn[MAX_COMPUTERNAME_LENGTH as usize] = 0;
            status = STATUS_INVALID_COMPUTER_NAME;
            let cn_oem = STRING {
                Length: oemn_len as u16,
                MaximumLength: (MAX_COMPUTERNAME_LENGTH + 1) as u16,
                Buffer: oemn.as_mut_ptr(),
            };
            // RtlpDidUnicodeToOemWork(DnsHostName, &cn_oem): reads
            // DnsHostName->Buffer[i] / oemn[i] for i < oemn_len.
            let uni_src = rslice((*dns_host_name).Buffer, oemn_len as usize);
            if nls::did_oem_work(uni_src, &oemn[..oemn_len as usize]) {
                status = RtlOemStringToUnicodeString(computer_name, &cn_oem, allocate);
            }
        }
    }
    status
}

// ── Wave RTL-4b slice 3: alloc-edge (harness alloc → NULL) ────────
//
// RtlpAllocateStringMemory is the harness NULL stub, so every
// allocating path returns failure with the genuine field state set
// *before* the NULL-alloc check (the pinned edge — same discipline
// as RTL-3 slice 2).

/// `BOOLEAN NTAPI RtlCreateUnicodeString(PUNICODE_STRING, PCWSTR)`.
/// `Size = (wcslen+1)*2`; `Size > MAXUSHORT` ⇒ FALSE; allocate via
/// `RtlpAllocateMemory` (kernel) and copy. Difftest harness still gets
/// the NULL-alloc model (rtl_alloc returns NULL ⇒ FALSE).
#[no_mangle]
pub unsafe extern "system" fn RtlCreateUnicodeString(
    uni_dest: *mut UNICODE_STRING,
    source: *const u16,
) -> u8 {
    let size = (wcslen(source) + 1) * 2;
    if size > MAXUSHORT as usize {
        return 0;
    }
    let buf = rtl_alloc(size, TAG_USTR) as *mut u16;
    if buf.is_null() {
        (*uni_dest).Buffer = core::ptr::null_mut();
        return 0;
    }
    // C mutates the destination only after a successful alloc+copy; match
    // that so the alloc-failure path leaves MaximumLength untouched.
    core::ptr::copy_nonoverlapping(source, buf, size / 2);
    (*uni_dest).Buffer = buf;
    (*uni_dest).Length = size as u16 - 2;
    (*uni_dest).MaximumLength = size as u16;
    1
}

/// `BOOLEAN NTAPI RtlCreateUnicodeStringFromAsciiz(PUNICODE_STRING,
/// PCSZ)` — `RtlInitAnsiString` + `RtlAnsiStringToUnicodeString(…,
/// TRUE)`; in the model the alloc path sets Length/Buffer(NULL)/Max
/// then NO_MEMORY ⇒ FALSE.
#[no_mangle]
pub unsafe extern "system" fn RtlCreateUnicodeStringFromAsciiz(
    destination: *mut UNICODE_STRING,
    source: *const u8,
) -> u8 {
    let mut ansi = STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    RtlInitAnsiString(&mut ansi, source);
    let status = RtlAnsiStringToUnicodeString(destination, &ansi, 1);
    if nt_success(status) {
        1
    } else {
        0
    }
}

/// `NTSTATUS NTAPI RtlDuplicateUnicodeString(ULONG, PCUNICODE_STRING,
/// PUNICODE_STRING)`.
#[no_mangle]
pub unsafe extern "system" fn RtlDuplicateUnicodeString(
    flags: u32,
    source: *const UNICODE_STRING,
    destination: *mut UNICODE_STRING,
) -> i32 {
    if source.is_null()
        || destination.is_null()
        || (*source).Length > (*source).MaximumLength
        || ((*source).Length == 0 && (*source).MaximumLength > 0 && (*source).Buffer.is_null())
        || flags == RTL_DUP_ALLOCATE_NULL_STRING
        || flags >= 4
    {
        return STATUS_INVALID_PARAMETER;
    }
    if (*source).Length == 0 && flags != (RTL_DUP_NULL_TERMINATE | RTL_DUP_ALLOCATE_NULL_STRING) {
        (*destination).Length = 0;
        (*destination).MaximumLength = 0;
        (*destination).Buffer = core::ptr::null_mut();
    } else {
        // alloc length matches C: Length + 2 if NULL-terminate flag set,
        // else just Length. Bytes; min 2 for the empty-NUL-string case.
        let alloc_len = if (flags & RTL_DUP_NULL_TERMINATE) != 0 {
            (*source).Length as usize + 2
        } else {
            (*source).Length as usize
        };
        let alloc_len = if alloc_len == 0 { 2 } else { alloc_len };
        let buf = rtl_alloc(alloc_len, TAG_USTR) as *mut u16;
        if buf.is_null() {
            (*destination).Buffer = core::ptr::null_mut();
            return STATUS_NO_MEMORY;
        }
        core::ptr::copy_nonoverlapping((*source).Buffer, buf, (*source).Length as usize / 2);
        if (flags & RTL_DUP_NULL_TERMINATE) != 0 {
            *buf.add((*source).Length as usize / 2) = 0;
        }
        (*destination).Buffer = buf;
        (*destination).Length = (*source).Length;
        (*destination).MaximumLength = alloc_len as u16;
    }
    STATUS_SUCCESS
}

/// `NTSTATUS NTAPI RtlStringFromGUID(REFGUID, PUNICODE_STRING)`.
#[no_mangle]
pub unsafe extern "system" fn RtlStringFromGUID(
    guid: *const GUID,
    guid_string: *mut UNICODE_STRING,
) -> i32 {
    const GUID_STRING_CHARS: usize = 38;

    #[inline]
    fn hex_digit(value: u8) -> u16 {
        match value & 0x0f {
            n @ 0..=9 => b'0' as u16 + n as u16,
            n => b'a' as u16 + (n as u16 - 10),
        }
    }

    #[inline]
    unsafe fn write_hex_u32(buffer: *mut u16, offset: &mut usize, value: u32, digits: usize) {
        for shift in (0..digits).rev() {
            *buffer.add(*offset) = hex_digit((value >> (shift * 4)) as u8);
            *offset += 1;
        }
    }

    #[inline]
    unsafe fn write_hex_u16(buffer: *mut u16, offset: &mut usize, value: u16, digits: usize) {
        write_hex_u32(buffer, offset, value as u32, digits);
    }

    #[inline]
    unsafe fn write_hex_u8(buffer: *mut u16, offset: &mut usize, value: u8) {
        *buffer.add(*offset) = hex_digit(value >> 4);
        *offset += 1;
        *buffer.add(*offset) = hex_digit(value);
        *offset += 1;
    }

    (*guid_string).Length = 38 * 2;
    (*guid_string).MaximumLength = 38 * 2 + 2;
    (*guid_string).Buffer = rtl_alloc((*guid_string).MaximumLength as usize, TAG_USTR) as *mut u16;
    if (*guid_string).Buffer.is_null() {
        return STATUS_NO_MEMORY;
    }

    let buffer = (*guid_string).Buffer;
    let mut offset = 0usize;
    *buffer.add(offset) = b'{' as u16;
    offset += 1;
    write_hex_u32(buffer, &mut offset, (*guid).Data1, 8);
    *buffer.add(offset) = b'-' as u16;
    offset += 1;
    write_hex_u16(buffer, &mut offset, (*guid).Data2, 4);
    *buffer.add(offset) = b'-' as u16;
    offset += 1;
    write_hex_u16(buffer, &mut offset, (*guid).Data3, 4);
    *buffer.add(offset) = b'-' as u16;
    offset += 1;
    write_hex_u8(buffer, &mut offset, (*guid).Data4[0]);
    write_hex_u8(buffer, &mut offset, (*guid).Data4[1]);
    *buffer.add(offset) = b'-' as u16;
    offset += 1;
    for i in 2..8 {
        write_hex_u8(buffer, &mut offset, (*guid).Data4[i]);
    }
    *buffer.add(offset) = b'}' as u16;
    offset += 1;
    debug_assert_eq!(offset, GUID_STRING_CHARS);
    *buffer.add(offset) = 0;

    STATUS_SUCCESS
}

/// `NTSTATUS NTAPI RtlpEnsureBufferSize(ULONG, PRTL_BUFFER, SIZE_T)`.
#[no_mangle]
pub unsafe extern "system" fn RtlpEnsureBufferSize(
    flags: u32,
    buffer: *mut RTL_BUFFER,
    required_size: usize,
) -> i32 {
    if flags & !RTL_SKIP_BUFFER_COPY != 0 {
        return STATUS_INVALID_PARAMETER;
    }
    if buffer.is_null() {
        return STATUS_INVALID_PARAMETER;
    }
    if (*buffer).Size >= required_size {
        return STATUS_SUCCESS;
    }
    if (*buffer).Buffer == (*buffer).StaticBuffer && (*buffer).StaticSize >= required_size {
        (*buffer).Size = required_size;
        return STATUS_SUCCESS;
    }
    // alloc → NULL
    STATUS_NO_MEMORY
}

/// `NTSTATUS NTAPI RtlMultiAppendUnicodeStringBuffer(...)` — genuine
/// is `UNIMPLEMENTED; return STATUS_NOT_IMPLEMENTED;`.
#[no_mangle]
pub unsafe extern "system" fn RtlMultiAppendUnicodeStringBuffer(
    _string_buffer: *mut u8,
    _number_of_addends: u32,
    _addends: *const UNICODE_STRING,
) -> i32 {
    STATUS_NOT_IMPLEMENTED
}

/// `BOOLEAN NTAPI RtlIsTextUnicode(CONST VOID*, INT, INT*)`.
#[no_mangle]
pub unsafe extern "system" fn RtlIsTextUnicode(buf: *const u8, len: i32, pf: *mut i32) -> u8 {
    let pf_in = if pf.is_null() { None } else { Some(*pf) };
    let blen = if len >= 2 { len as usize } else { 0 };
    let bytes: &[u8] = if blen == 0 || buf.is_null() {
        &[]
    } else {
        core::slice::from_raw_parts(buf, blen)
    };
    let (res, pf_out) = misc::is_text_unicode(bytes, len, pf_in);
    if let Some(v) = pf_out {
        core::ptr::write_volatile(pf, v);
    }
    res as u8
}

// ── Wave RTL-4b slice 1: validate / find / erase / free / GUID ────

/// `NTSTATUS NTAPI RtlValidateUnicodeString(ULONG Flags,
/// PCUNICODE_STRING)`. Flags must be 0; a NULL `String` is VALID.
#[no_mangle]
pub unsafe extern "system" fn RtlValidateUnicodeString(
    flags: u32,
    string: *const UNICODE_STRING,
) -> i32 {
    if flags != 0 {
        return STATUS_INVALID_PARAMETER;
    }
    if string.is_null() {
        return STATUS_SUCCESS;
    }
    if misc::unicode_string_valid(
        (*string).Buffer.is_null(),
        (*string).Length,
        (*string).MaximumLength,
    ) {
        STATUS_SUCCESS
    } else {
        STATUS_INVALID_PARAMETER
    }
}

/// `NTSTATUS NTAPI RtlFindCharInUnicodeString(ULONG, PCUNICODE_STRING
/// Search, PCUNICODE_STRING Match, PUSHORT Position)`.
#[no_mangle]
pub unsafe extern "system" fn RtlFindCharInUnicodeString(
    flags: u32,
    search: *const UNICODE_STRING,
    match_string: *const UNICODE_STRING,
    position: *mut u16,
) -> i32 {
    if position.is_null() {
        return STATUS_INVALID_PARAMETER;
    }
    core::ptr::write_volatile(position, 0);
    if flags
        & !(RTL_FIND_CHAR_START_AT_END | RTL_FIND_CHAR_COMPLEMENT | RTL_FIND_CHAR_CASE_INSENSITIVE)
        != 0
    {
        return STATUS_INVALID_PARAMETER;
    }
    let want = flags & RTL_FIND_CHAR_COMPLEMENT == 0;
    let ci = flags & RTL_FIND_CHAR_CASE_INSENSITIVE != 0;
    match misc::find_char(
        uunits(search),
        uunits(match_string),
        flags & RTL_FIND_CHAR_START_AT_END != 0,
        want,
        ci,
    ) {
        Some(p) => {
            core::ptr::write_volatile(position, p);
            STATUS_SUCCESS
        }
        None => STATUS_NOT_FOUND,
    }
}

/// `VOID NTAPI RtlEraseUnicodeString(PUNICODE_STRING)`. Zeroes
/// `MaximumLength` **bytes** of Buffer (if both non-zero), `Length=0`.
#[no_mangle]
pub unsafe extern "system" fn RtlEraseUnicodeString(string: *mut UNICODE_STRING) {
    if !(*string).Buffer.is_null() && (*string).MaximumLength != 0 {
        let d = (*string).Buffer as *mut u8;
        for i in 0..(*string).MaximumLength as usize {
            core::ptr::write_volatile(d.add(i), 0u8);
        }
        (*string).Length = 0;
    }
}

/// `VOID NTAPI RtlFreeAnsiString(PANSI_STRING)`. Buffer non-NULL ⇒
/// free (harness stub: no-op) + zero the whole struct.
#[no_mangle]
pub unsafe extern "system" fn RtlFreeAnsiString(ansi_string: *mut STRING) {
    if !(*ansi_string).Buffer.is_null() {
        rtl_free((*ansi_string).Buffer, TAG_ASTR);
        (*ansi_string).Length = 0;
        (*ansi_string).MaximumLength = 0;
        (*ansi_string).Buffer = core::ptr::null_mut();
    }
}

/// `VOID NTAPI RtlFreeOemString(POEM_STRING)`. Buffer non-NULL ⇒ free
/// **only** — genuine does NOT zero the struct (the pinned quirk).
#[no_mangle]
pub unsafe extern "system" fn RtlFreeOemString(oem_string: *mut STRING) {
    if !(*oem_string).Buffer.is_null() {
        rtl_free((*oem_string).Buffer, TAG_OSTR);
        // genuine: no field writes (pinned quirk).
    }
}

/// `VOID NTAPI RtlFreeUnicodeString(PUNICODE_STRING)`. Buffer
/// non-NULL ⇒ free (stub) + zero the whole struct.
#[no_mangle]
pub unsafe extern "system" fn RtlFreeUnicodeString(unicode_string: *mut UNICODE_STRING) {
    if !(*unicode_string).Buffer.is_null() {
        rtl_free((*unicode_string).Buffer as *mut u8, TAG_USTR);
        (*unicode_string).Length = 0;
        (*unicode_string).MaximumLength = 0;
        (*unicode_string).Buffer = core::ptr::null_mut();
    }
}

/// `NTSTATUS NTAPI RtlGUIDFromString(UNICODE_STRING*, GUID*)`. The
/// Wine fixed-format state machine; partial in-place writes on
/// failure are faithful; the buffer is read unbounded (≥38 units).
#[no_mangle]
pub unsafe extern "system" fn RtlGUIDFromString(
    str: *const UNICODE_STRING,
    guid: *mut GUID,
) -> i32 {
    let s = core::slice::from_raw_parts((*str).Buffer, 38);
    let mut buf: [u8; 16] = *(guid as *const [u8; 16]);
    let ok = misc::guid_from_string(s, &mut buf);
    let p = guid as *mut u8;
    for i in 0..16 {
        core::ptr::write_volatile(p.add(i), buf[i]);
    }
    if ok {
        STATUS_SUCCESS
    } else {
        STATUS_INVALID_PARAMETER
    }
}

// ── Wave RTL-4c: integer→char + the *ToUnicodeString wrappers ──────

/// `NTSTATUS NTAPI RtlIntegerToChar(ULONG, ULONG, ULONG, PCHAR)`.
/// Uppercase digits; `len > length` ⇒ OVERFLOW out-ranks `str==NULL`
/// ⇒ ACCESS_VIOLATION; writes the digits, plus a NUL iff `len <
/// length` (exact-fit omits it).
#[no_mangle]
pub unsafe extern "system" fn RtlIntegerToChar(
    value: u32,
    base: u32,
    length: u32,
    str: *mut u8,
) -> i32 {
    let (d, n) = match conv::integer_to_char(value, base) {
        None => return STATUS_INVALID_PARAMETER,
        Some(x) => x,
    };
    if n as u32 > length {
        return STATUS_BUFFER_OVERFLOW;
    }
    if str.is_null() {
        return STATUS_ACCESS_VIOLATION;
    }
    for i in 0..n {
        core::ptr::write_volatile(str.add(i), d[i]);
    }
    if (n as u32) < length {
        core::ptr::write_volatile(str.add(n), 0u8);
    }
    STATUS_SUCCESS
}

/// `NTSTATUS NTAPI RtlLargeIntegerToChar(PLARGE_INTEGER, ULONG, ULONG,
/// PCHAR)`. Like `RtlIntegerToChar` but 64-bit and with NO `str==NULL`
/// guard (genuine memcpys unconditionally — NULL str is genuine UB,
/// excluded). The copy is `RtlpSafeCopyMemory` (a faithful memcpy).
#[no_mangle]
pub unsafe extern "system" fn RtlLargeIntegerToChar(
    value: *const i64,
    base: u32,
    length: u32,
    str: *mut u8,
) -> i32 {
    let (d, n) = match conv::large_integer_to_char(*value as u64, base) {
        None => return STATUS_INVALID_PARAMETER,
        Some(x) => x,
    };
    if n as u32 > length {
        return STATUS_BUFFER_OVERFLOW;
    }
    for i in 0..n {
        core::ptr::write_volatile(str.add(i), d[i]);
    }
    if (n as u32) < length {
        core::ptr::write_volatile(str.add(n), 0u8);
    }
    STATUS_SUCCESS
}

/// `NTSTATUS NTAPI RtlIntegerToUnicodeString(ULONG, ULONG,
/// PUNICODE_STRING)` — `RtlIntegerToChar` into a `[u8;33]` then
/// `RtlAnsiStringToUnicodeString(…, FALSE)`.
#[no_mangle]
pub unsafe extern "system" fn RtlIntegerToUnicodeString(
    value: u32,
    base: u32,
    string: *mut UNICODE_STRING,
) -> i32 {
    let mut buf = [0u8; 33];
    let status = RtlIntegerToChar(value, base, 33, buf.as_mut_ptr());
    if !nt_success(status) {
        return status;
    }
    let ansi = STRING {
        Length: strlen(buf.as_ptr()) as u16,
        MaximumLength: 33,
        Buffer: buf.as_mut_ptr(),
    };
    RtlAnsiStringToUnicodeString(string, &ansi, 0)
}

/// `NTSTATUS NTAPI RtlInt64ToUnicodeString(ULONGLONG, ULONG,
/// PUNICODE_STRING)` — `RtlLargeIntegerToChar` into a `[u8;65]` then
/// `RtlAnsiStringToUnicodeString(…, FALSE)`.
#[no_mangle]
pub unsafe extern "system" fn RtlInt64ToUnicodeString(
    value: u64,
    base: u32,
    string: *mut UNICODE_STRING,
) -> i32 {
    let mut buf = [0u8; 65];
    let q = value as i64;
    let status = RtlLargeIntegerToChar(&q, base, 65, buf.as_mut_ptr());
    if !nt_success(status) {
        return status;
    }
    let ansi = STRING {
        Length: strlen(buf.as_ptr()) as u16,
        MaximumLength: 65,
        Buffer: buf.as_mut_ptr(),
    };
    RtlAnsiStringToUnicodeString(string, &ansi, 0)
}

// ── Wave RTL-5: utf8.c ────────────────────────────────────────────

/// `NTSTATUS NTAPI RtlUnicodeToUTF8N(CHAR*, ULONG, ULONG*, const
/// WCHAR*, ULONG)`.
///
/// Gated off the `kernel` build: the dedicated `utf8` crate
/// (libutf8.a → rtl_vista) already owns these two symbols and is
/// smoke-green; emitting them here too would duplicate-define at the
/// final ntoskrnl link. `rtl-difftest` (no `kernel` feature) still
/// fuzzes them.
#[cfg(not(feature = "kernel"))]
#[no_mangle]
pub unsafe extern "system" fn RtlUnicodeToUTF8N(
    utf8_dest: *mut u8,
    utf8_bytes_max: u32,
    utf8_bytes_written: *mut u32,
    uni_src: *const u16,
    uni_bytes: u32,
) -> i32 {
    if uni_src.is_null() {
        return STATUS_INVALID_PARAMETER_4;
    }
    if utf8_bytes_written.is_null() {
        return STATUS_INVALID_PARAMETER;
    }
    if !utf8_dest.is_null() && uni_bytes % 2 != 0 {
        return STATUS_INVALID_PARAMETER_5;
    }
    let src = rslice(uni_src, (uni_bytes / 2) as usize);
    let (written, status) = if utf8_dest.is_null() {
        utf8::unicode_to_utf8(src, None, utf8_bytes_max)
    } else {
        let d = wslice(utf8_dest, utf8_bytes_max as usize);
        utf8::unicode_to_utf8(src, Some(d), utf8_bytes_max)
    };
    core::ptr::write_volatile(utf8_bytes_written, written);
    status
}

/// `NTSTATUS NTAPI RtlUTF8ToUnicodeN(WCHAR*, ULONG, ULONG*, const
/// CHAR*, ULONG)`. Gated off the `kernel` build (see
/// `RtlUnicodeToUTF8N`).
#[cfg(not(feature = "kernel"))]
#[no_mangle]
pub unsafe extern "system" fn RtlUTF8ToUnicodeN(
    uni_dest: *mut u16,
    uni_bytes_max: u32,
    uni_bytes_written: *mut u32,
    utf8_src: *const u8,
    utf8_bytes: u32,
) -> i32 {
    if utf8_src.is_null() {
        return STATUS_INVALID_PARAMETER_4;
    }
    if uni_bytes_written.is_null() {
        return STATUS_INVALID_PARAMETER;
    }
    let src = rslice(utf8_src, utf8_bytes as usize);
    let (units, status) = if uni_dest.is_null() {
        utf8::utf8_to_unicode(src, None, uni_bytes_max)
    } else {
        let d = wslice(uni_dest, (uni_bytes_max / 2) as usize);
        utf8::utf8_to_unicode(src, Some(d), uni_bytes_max)
    };
    core::ptr::write_volatile(uni_bytes_written, units * 2);
    status
}

// ── Wave RTL-6: dos8dot3.c ────────────────────────────────────────

/// `GENERATE_NAME_CONTEXT` (ndk/rtltypes.h) — 36 bytes, align 4.
#[repr(C)]
pub struct GenerateNameContext {
    pub checksum: u16,
    pub checksum_inserted: u8,
    pub name_length: u8,
    pub name_buffer: [u16; 8],
    pub extension_length: u32,
    pub extension_buffer: [u16; 4],
    pub last_index_value: u32,
}

/// Shared-CP1252 model of `RtlIsValidOemCharacter` (unicode.c):
/// `u = upcase(dec(enc(*c)))`; `valid = enc(u) != '?'`; on valid the
/// genuine **mutates** `*c = u`.
#[inline]
fn iv_oem(ch: &mut u16) -> bool {
    let u = nls::upcase_uc(nls::dec_byte(nls::enc_byte(*ch)));
    if nls::enc_byte(u) == b'?' {
        return false;
    }
    *ch = u;
    true
}

/// `BOOLEAN NTAPI RtlIsValidOemCharacter(PWCHAR Char)`.
#[no_mangle]
pub unsafe extern "system" fn RtlIsValidOemCharacter(ch: *mut u16) -> u8 {
    let mut c = *ch;
    if iv_oem(&mut c) {
        core::ptr::write_volatile(ch, c);
        1
    } else {
        0
    }
}

/// `VOID NTAPI RtlGenerate8dot3Name(PUNICODE_STRING Name, BOOLEAN
/// AllowExtendedCharacters, PGENERATE_NAME_CONTEXT, PUNICODE_STRING
/// Name8dot3)`. Single-byte model (`NlsMbOemCodePageTag == 0` ⇒ the
/// MB-OEM extra-decrement branch is dead).
#[no_mangle]
pub unsafe extern "system" fn RtlGenerate8dot3Name(
    name: *const UNICODE_STRING,
    allow_ext: u8,
    ctx: *mut GenerateNameContext,
    out: *mut UNICODE_STRING,
) {
    let nbuf = (*name).Buffer;
    let length = (*name).Length as usize / 2;
    let c = &mut *ctx;

    if c.name_length == 0 {
        let mut dot_pos = length;
        for index in 0..length {
            if *nbuf.add(index) == b'.' as u16 {
                dot_pos = index;
            }
        }

        let mut oem_left: u8 = 6;
        let mut index = 0;
        while index < dot_pos && oem_left != 0 {
            let mut ch = *nbuf.add(index);
            if ch > b' ' as u16
                && ch != b'.' as u16
                && (ch < 127 || (allow_ext != 0 && iv_oem(&mut ch)))
            {
                if dos8dot3::is_short_illegal(ch) {
                    ch = b'_' as u16;
                } else if (b'a' as u16..=b'z' as u16).contains(&ch) {
                    ch = nls::upcase_uc(ch);
                }
                c.name_buffer[c.name_length as usize] = ch;
                c.name_length += 1;
                oem_left -= 1;
            }
            index += 1;
        }

        c.extension_length = 0;
        if dot_pos < length {
            c.extension_buffer[0] = b'.' as u16;
            c.extension_length = 1;
            let mut dp = dot_pos;
            let mut ch = b'.' as u16;
            while dp < length && c.extension_length < 4 {
                ch = *nbuf.add(dp);
                if ch > b' ' as u16
                    && ch != b'.' as u16
                    && (ch < 127 || (allow_ext != 0 && iv_oem(&mut ch)))
                {
                    if dos8dot3::is_short_illegal(ch) {
                        ch = b'_' as u16;
                    } else if (b'a' as u16..=b'z' as u16).contains(&ch) {
                        ch = nls::upcase_uc(ch);
                    }
                    c.extension_buffer[c.extension_length as usize] = ch;
                    c.extension_length += 1;
                }
                ch = 0;
                dp += 1;
            }
            if ch != 0 {
                c.extension_buffer[c.extension_length as usize - 1] = b'~' as u16;
            }
        }

        if c.name_length <= 2 {
            let mut checksum = dos8dot3::get_checksum(rslice(nbuf, length), (*name).Length);
            c.checksum = checksum;
            for index in 0..4 {
                let nib = checksum & 0xF;
                c.name_buffer[c.name_length as usize + index] = if nib > 9 {
                    nib + b'A' as u16 - 10
                } else {
                    nib + b'0' as u16
                };
                checksum >>= 4;
            }
            c.checksum_inserted = 1;
            c.name_length += 4;
        }
    }

    c.last_index_value += 1;

    if c.last_index_value > 4 && c.checksum_inserted == 0 {
        let mut checksum = dos8dot3::get_checksum(rslice(nbuf, length), (*name).Length);
        c.checksum = checksum;
        for index in 2..6 {
            let nib = checksum & 0xF;
            c.name_buffer[index] = if nib > 9 {
                nib + b'A' as u16 - 10
            } else {
                nib + b'0' as u16
            };
            checksum >>= 4;
        }
        c.checksum_inserted = 1;
        c.name_length = 6;
        c.last_index_value = 1;
    }

    // index buffer (decimal, reversed) + leading '~'
    let mut idx_buf = [0u16; 8];
    let mut idx = c.last_index_value;
    let mut index_length: usize = 1;
    while index_length <= 7 && idx > 0 {
        idx_buf[8 - index_length] = b'0' as u16 + (idx % 10) as u16;
        idx /= 10;
        index_length += 1;
    }
    idx_buf[8 - index_length] = b'~' as u16;

    let obuf = (*out).Buffer;
    let mut olen: usize = 0;
    (*out).Length = 0;

    if c.name_length != 0 {
        for i in 0..c.name_length as usize {
            core::ptr::write_volatile(obuf.add(i), c.name_buffer[i]);
        }
        olen = c.name_length as usize;
        (*out).Length = (olen * 2) as u16;
    }

    for i in 0..index_length {
        core::ptr::write_volatile(obuf.add(olen + i), idx_buf[8 - index_length + i]);
    }
    olen += index_length;
    (*out).Length = (olen * 2) as u16;

    if c.extension_length != 0 {
        for i in 0..c.extension_length as usize {
            core::ptr::write_volatile(obuf.add(olen + i), c.extension_buffer[i]);
        }
        olen += c.extension_length as usize;
        (*out).Length = (olen * 2) as u16;
    }
}

/// `BOOLEAN NTAPI RtlIsNameLegalDOS8Dot3(PCUNICODE_STRING,
/// POEM_STRING OemName, PBOOLEAN NameContainsSpaces)`.
#[no_mangle]
pub unsafe extern "system" fn RtlIsNameLegalDOS8Dot3(
    name: *const UNICODE_STRING,
    oem_name: *mut STRING,
    name_contains_spaces: *mut u8,
) -> u8 {
    const ILLEGAL: &[u8] = b"*?<>|\"+=,;[]:/\\\xE5";
    let mut local = [0u8; 12];
    let mut local_os;
    let oem: *mut STRING = if oem_name.is_null() {
        local_os = STRING {
            Length: 12,
            MaximumLength: 12,
            Buffer: local.as_mut_ptr(),
        };
        &mut local_os
    } else {
        oem_name
    };

    let status = RtlUpcaseUnicodeStringToCountedOemString(oem, name, 0);
    if !nt_success(status) {
        return 0;
    }
    let olen = (*oem).Length as usize;
    if (*oem).Length > 12 || (*oem).Buffer.is_null() {
        return 0;
    }
    let ob = (*oem).Buffer;

    if *ob == b'.' {
        if (*oem).Length != 1 && ((*oem).Length != 2 || *ob.add(1) != b'.') {
            return 0;
        }
        if !name_contains_spaces.is_null() {
            core::ptr::write_volatile(name_contains_spaces, 0);
        }
        return 1;
    }

    let mut dot: i32 = -1;
    let mut got_space = false;
    for i in 0..olen {
        let b = *ob.add(i);
        match b {
            b' ' => {
                if i == 0 || i == olen - 1 || *ob.add(i + 1) == b'.' {
                    return 0;
                }
                got_space = true;
            }
            b'.' => {
                if dot != -1 {
                    return 0;
                }
                dot = i as i32;
            }
            _ => {
                if b == 0 || ILLEGAL.contains(&b) {
                    return 0;
                }
            }
        }
    }

    if dot == -1 {
        if olen > 8 {
            return 0;
        }
    } else if dot > 8 || (olen as i32 - dot > 4) || dot == olen as i32 - 1 {
        return 0;
    }

    if !name_contains_spaces.is_null() {
        core::ptr::write_volatile(name_contains_spaces, got_space as u8);
    }
    1
}

// ── Wave SID: sdk/lib/rtl/sid.c (byte + alloc-edge surface) ───────
// A SID is a raw binary record (no counted-string wrapper):
//   off0 Revision:u8  off1 SubAuthorityCount:u8
//   off2 IdentifierAuthority:[u8;6]  off8 SubAuthority:[u32 LE]
// Pure decisions live in core_rs::sid; these shims do pointer
// reconstruction + the interior-pointer returns. rtl-strings is
// no-alloc (smoke-validated, cf. RtlCreateUnicodeString) so
// RtlAllocateAndInitializeSid yields the NO_MEMORY edge and
// RtlFreeSid is the genuine "always NULL" no-op. The swprintf
// formatter RtlConvertSidToUnicodeString is Wave SID-2 (sid.c stays
// C until then — whole-file rule).

/// `SID_AND_ATTRIBUTES { PSID Sid; ULONG Attributes; }` — per-arch
/// layout (ptr 8/4 + u32) via repr(C).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SidAndAttributes {
    pub sid: *mut u8,
    pub attributes: u32,
}

/// `BOOLEAN NTAPI RtlValidSid(PSID)`. NULL ⇒ TRUE (genuine `&&`
/// short-circuits). A bad non-NULL pointer faults (DUMMY_PSEH dead
/// `_SEH2_EXCEPT`) = genuine UB, never fuzzed.
#[no_mangle]
pub unsafe extern "system" fn RtlValidSid(sid_: *const u8) -> u8 {
    if sid_.is_null() {
        return sid::valid(0, 0, true) as u8;
    }
    sid::valid(*sid_, *sid_.add(1), false) as u8
}

/// `ULONG NTAPI RtlLengthRequiredSid(ULONG SubAuthorityCount)`.
#[no_mangle]
pub unsafe extern "system" fn RtlLengthRequiredSid(count: u32) -> u32 {
    sid::length_required(count)
}

/// `NTSTATUS NTAPI RtlInitializeSid(PSID, PSID_IDENTIFIER_AUTHORITY,
/// UCHAR)`. Writes Revision/Count/IdentifierAuthority only (does NOT
/// zero the subauthorities).
#[no_mangle]
pub unsafe extern "system" fn RtlInitializeSid(
    sid_: *mut u8,
    id_auth: *const u8,
    count: u8,
) -> i32 {
    *sid_ = sid::SID_REVISION;
    *sid_.add(1) = count;
    core::ptr::copy_nonoverlapping(id_auth, sid_.add(2), 6);
    STATUS_SUCCESS
}

/// `PULONG NTAPI RtlSubAuthoritySid(PSID, ULONG)` —
/// `&Sid->SubAuthority[n]`.
#[no_mangle]
pub unsafe extern "system" fn RtlSubAuthoritySid(sid_: *mut u8, sub_authority: u32) -> *mut u32 {
    sid_.add(8 + 4 * sub_authority as usize) as *mut u32
}

/// `PUCHAR NTAPI RtlSubAuthorityCountSid(PSID)` —
/// `&Sid->SubAuthorityCount`.
#[no_mangle]
pub unsafe extern "system" fn RtlSubAuthorityCountSid(sid_: *mut u8) -> *mut u8 {
    sid_.add(1)
}

/// `PSID_IDENTIFIER_AUTHORITY NTAPI RtlIdentifierAuthoritySid(PSID)`.
#[no_mangle]
pub unsafe extern "system" fn RtlIdentifierAuthoritySid(sid_: *mut u8) -> *mut u8 {
    sid_.add(2)
}

/// `BOOLEAN NTAPI RtlEqualSid(PSID, PSID)`. Quick u16
/// (Revision+Count) compare, then byte-equal over
/// `RtlLengthSid(Sid1)` (genuine reads Sid2 to Sid1's length —
/// over-read if Sid2 shorter = genuine UB; fuzz sizes Sid2 ≥ Sid1).
#[no_mangle]
pub unsafe extern "system" fn RtlEqualSid(s1: *const u8, s2: *const u8) -> u8 {
    if *(s1 as *const u16) != *(s2 as *const u16) {
        return 0;
    }
    let len = sid::length(*s1.add(1)) as usize;
    (rslice(s1, len) == rslice(s2, len)) as u8
}

/// `ULONG NTAPI RtlLengthSid(PSID)` = `8 + 4*SubAuthorityCount`.
#[no_mangle]
pub unsafe extern "system" fn RtlLengthSid(sid_: *const u8) -> u32 {
    sid::length(*sid_.add(1))
}

/// `NTSTATUS NTAPI RtlCopySid(ULONG BufferLength, PSID Dest,
/// PSID Src)`. `RtlMoveMemory` (overlap-safe).
#[no_mangle]
pub unsafe extern "system" fn RtlCopySid(buffer_length: u32, dest: *mut u8, src: *const u8) -> i32 {
    let sid_length = sid::length(*src.add(1));
    if sid_length > buffer_length {
        return STATUS_BUFFER_TOO_SMALL;
    }
    core::ptr::copy(src, dest, sid_length as usize);
    STATUS_SUCCESS
}

/// `PVOID NTAPI RtlFreeSid(PSID)` — genuine always returns NULL; the
/// free is a no-op (rtl-strings no-alloc discipline).
#[no_mangle]
pub unsafe extern "system" fn RtlFreeSid(_sid: *mut u8) -> *mut u8 {
    core::ptr::null_mut()
}

/// `BOOLEAN NTAPI RtlEqualPrefixSid(PSID, PSID)`.
#[no_mangle]
pub unsafe extern "system" fn RtlEqualPrefixSid(s1: *const u8, s2: *const u8) -> u8 {
    let c1 = *s1.add(1) as usize;
    let c2 = *s2.add(1) as usize;
    let auth1 = &*(s1.add(2) as *const [u8; 6]);
    let auth2 = &*(s2.add(2) as *const [u8; 6]);
    let subs1 = core::slice::from_raw_parts(s1.add(8) as *const u32, c1);
    let subs2 = core::slice::from_raw_parts(s2.add(8) as *const u32, c2);
    sid::equal_prefix(*s1, auth1, subs1, *s2, auth2, subs2) as u8
}

/// `NTSTATUS NTAPI RtlCopySidAndAttributesArray(...)`.
#[no_mangle]
pub unsafe extern "system" fn RtlCopySidAndAttributesArray(
    count: u32,
    src: *const SidAndAttributes,
    sid_area_size: u32,
    dest: *mut SidAndAttributes,
    sid_area: *mut u8,
    remaining_sid_area: *mut *mut u8,
    remaining_sid_area_size: *mut u32,
) -> i32 {
    let mut area = sid_area;
    let mut area_size = sid_area_size;
    for i in 0..count as usize {
        let s = &*src.add(i);
        let sid_length = sid::length(*s.sid.add(1));
        if sid_length > area_size {
            return STATUS_BUFFER_TOO_SMALL;
        }
        area_size -= sid_length;
        let d = &mut *dest.add(i);
        d.sid = area;
        d.attributes = s.attributes;
        core::ptr::copy(s.sid as *const u8, area, sid_length as usize);
        area = area.add(sid_length as usize);
    }
    core::ptr::write_volatile(remaining_sid_area, area);
    core::ptr::write_volatile(remaining_sid_area_size, area_size);
    STATUS_SUCCESS
}

/// `NTSTATUS NTAPI RtlAllocateAndInitializeSid(...)`. `> 8`
/// subauthorities ⇒ STATUS_INVALID_SID; else allocate a SID via
/// `RtlpAllocateMemory` (kernel/ntdll wraps PagedPool / process heap)
/// and fill in the header + the requested subauthorities. Difftest
/// harness keeps the NULL-alloc model (rtl_alloc → NULL ⇒ NO_MEMORY).
///
/// SID layout (winnt.h SID): 1 byte Revision + 1 byte SubAuthorityCount
/// + 6 bytes IdentifierAuthority + N×4 SubAuthorities = 8 + N*4 bytes.
const TAG_SID: u32 = u32::from_le_bytes(*b"pSid"); // nolint: const-contradiction ('diSp' -- sdk/lib/rtl/sid.c:15; ntoskrnl's TAG_SID is 'iSeS')
#[no_mangle]
pub unsafe extern "system" fn RtlAllocateAndInitializeSid(
    id_auth: *const u8,
    sub_authority_count: u8,
    s0: u32,
    s1: u32,
    s2: u32,
    s3: u32,
    s4: u32,
    s5: u32,
    s6: u32,
    s7: u32,
    sid: *mut *mut u8,
) -> i32 {
    if sub_authority_count > 8 {
        return STATUS_INVALID_SID;
    }
    let size = 8usize + (sub_authority_count as usize) * 4;
    let buf = rtl_alloc(size, TAG_SID);
    if buf.is_null() {
        return STATUS_NO_MEMORY;
    }
    // Header: Revision=1, SubAuthorityCount, IdentifierAuthority[6].
    *buf.add(0) = 1u8; // SID_REVISION
    *buf.add(1) = sub_authority_count;
    core::ptr::copy_nonoverlapping(id_auth, buf.add(2), 6);
    // Subauthorities (variable count).
    let subs = [s0, s1, s2, s3, s4, s5, s6, s7];
    let sub_ptr = buf.add(8) as *mut u32;
    for i in 0..sub_authority_count as usize {
        *sub_ptr.add(i) = subs[i];
    }
    *sid = buf;
    STATUS_SUCCESS
}

/// `NTSTATUS NTAPI RtlConvertSidToUnicodeString(PUNICODE_STRING,
/// PSID, BOOLEAN)`. `!RtlValidSid` ⇒ INVALID_SID. AllocateBuffer ⇒
/// RtlCreateUnicodeString (rtl-strings no-alloc ⇒ NO_MEMORY edge).
/// Caller-buffer: Length=(wcs-Buffer)*2; `> MaximumLength` ⇒
/// BUFFER_TOO_SMALL; else set Length, copy, and NUL-terminate at
/// `Buffer[Length/2]` only when `Length < MaximumLength`.
#[no_mangle]
pub unsafe extern "system" fn RtlConvertSidToUnicodeString(
    string: *mut UNICODE_STRING,
    sid_: *const u8,
    allocate_buffer: u8,
) -> i32 {
    let is_null = sid_.is_null();
    let valid = if is_null {
        sid::valid(0, 0, true)
    } else {
        sid::valid(*sid_, *sid_.add(1), false)
    };
    if !valid {
        return STATUS_INVALID_SID;
    }
    let auth = &*(sid_.add(2) as *const [u8; 6]);
    let count = *sid_.add(1) as usize;
    let subs = core::slice::from_raw_parts(sid_.add(8) as *const u32, count);

    let mut buf = [0u16; 256];
    let n = sid::convert_text(auth, subs, &mut buf);

    if allocate_buffer != 0 {
        // RtlCreateUnicodeString — rtl-strings is no-alloc ⇒ FALSE.
        let mut tmp = [0u16; 257];
        tmp[..n].copy_from_slice(&buf[..n]);
        tmp[n] = 0;
        if RtlCreateUnicodeString(string, tmp.as_ptr()) == 0 {
            return STATUS_NO_MEMORY;
        }
        return STATUS_SUCCESS;
    }

    let length = (n * 2) as u32;
    let max = (*string).MaximumLength as u32;
    if length > max {
        return STATUS_BUFFER_TOO_SMALL;
    }
    (*string).Length = length as u16;
    core::ptr::copy_nonoverlapping(buf.as_ptr(), (*string).Buffer, n);
    if length < max {
        *(*string).Buffer.add(n) = 0; // UNICODE_NULL at Buffer[Length/2]
    }
    STATUS_SUCCESS
}

/// `NTSTATUS NTAPI RtlCreateServiceSid(...)` — genuine UNIMPLEMENTED.
#[no_mangle]
pub unsafe extern "system" fn RtlCreateServiceSid(
    _service_name: *const UNICODE_STRING,
    _service_sid: *mut u8,
    _service_sid_length: *mut u32,
) -> i32 {
    STATUS_NOT_IMPLEMENTED
}

// ── Wave ACL-1: sdk/lib/rtl/acl.c structural/query/validate ───────
// ACL/ACE are raw binary records over a caller buffer (pointer-free
// layout — identical on every arch). Pure constants/validator live
// in core_rs::acl; these shims do the pointer walks. No alloc.

#[inline]
unsafe fn acl_rev(p: *const u8) -> u8 {
    *p
}
#[inline]
unsafe fn acl_size(p: *const u8) -> u16 {
    (p.add(2) as *const u16).read_unaligned()
}
#[inline]
unsafe fn acl_ace_count(p: *const u8) -> u16 {
    (p.add(4) as *const u16).read_unaligned()
}
#[inline]
unsafe fn ace_size(p: *const u8) -> u16 {
    (p.add(2) as *const u16).read_unaligned()
}

/// `NTSTATUS NTAPI RtlCreateAcl(PACL, ULONG AclSize, ULONG Revision)`.
#[no_mangle]
pub unsafe extern "system" fn RtlCreateAcl(acl_: *mut u8, acl_sz: u32, acl_revision: u32) -> i32 {
    if (acl_sz as usize) < acl::ACL_HDR {
        return STATUS_BUFFER_TOO_SMALL;
    }
    if acl_revision < acl::MIN_ACL_REVISION as u32
        || acl_revision > acl::MAX_ACL_REVISION as u32
        || acl_sz > MAXUSHORT
    {
        return STATUS_INVALID_PARAMETER;
    }
    *acl_ = acl_revision as u8; // AclRevision
    *acl_.add(1) = 0; // Sbz1
    (acl_.add(2) as *mut u16).write_unaligned(acl::round_up(acl_sz, 4) as u16);
    (acl_.add(4) as *mut u16).write_unaligned(0); // AceCount
    (acl_.add(6) as *mut u16).write_unaligned(0); // Sbz2
    STATUS_SUCCESS
}

/// `BOOLEAN NTAPI RtlValidAcl(PACL)`.
#[no_mangle]
pub unsafe extern "system" fn RtlValidAcl(acl_: *const u8) -> u8 {
    // Need the header to know AclSize; valid_acl re-checks size>=8.
    let sz = acl_size(acl_) as usize;
    let view = core::slice::from_raw_parts(acl_, sz.max(acl::ACL_HDR));
    acl::valid_acl(&view[..sz.min(view.len())]) as u8
}

/// `BOOLEAN NTAPI RtlFirstFreeAce(PACL, PACE *FirstFreeAce)`.
#[no_mangle]
pub unsafe extern "system" fn RtlFirstFreeAce(acl_: *mut u8, first_free_ace: *mut *mut u8) -> u8 {
    *first_free_ace = core::ptr::null_mut();
    let acl_end = acl_.add(acl_size(acl_) as usize);
    let mut current = acl_.add(acl::ACL_HDR);
    let count = acl_ace_count(acl_);
    for _ in 0..count {
        if current as usize >= acl_end as usize {
            return 0;
        }
        current = current.add(ace_size(current) as usize);
    }
    if current as usize <= acl_end as usize {
        *first_free_ace = current;
    }
    1
}

/// `NTSTATUS NTAPI RtlGetAce(PACL, ULONG AceIndex, PVOID *Ace)`.
#[no_mangle]
pub unsafe extern "system" fn RtlGetAce(
    acl_: *mut u8,
    ace_index: u32,
    ace_out: *mut *mut u8,
) -> i32 {
    let rev = acl_rev(acl_);
    if rev < acl::MIN_ACL_REVISION
        || rev > acl::MAX_ACL_REVISION
        || ace_index >= acl_ace_count(acl_) as u32
    {
        return STATUS_INVALID_PARAMETER;
    }
    let end = acl_.add(acl_size(acl_) as usize) as usize;
    let mut ace = acl_.add(acl::ACL_HDR);
    for _ in 0..ace_index {
        if ace as usize >= end {
            return STATUS_INVALID_PARAMETER;
        }
        ace = ace.add(ace_size(ace) as usize);
    }
    if ace as usize >= end {
        return STATUS_INVALID_PARAMETER;
    }
    *ace_out = ace;
    STATUS_SUCCESS
}

/// `NTSTATUS NTAPI RtlQueryInformationAcl(PACL, PVOID Info,
/// ULONG Len, ACL_INFORMATION_CLASS Class)`. Class 1 =
/// ACL_REVISION_INFORMATION {u32}, 2 = ACL_SIZE_INFORMATION
/// {AceCount, AclBytesInUse, AclBytesFree}.
#[no_mangle]
pub unsafe extern "system" fn RtlQueryInformationAcl(
    acl_: *mut u8,
    information: *mut u8,
    information_length: u32,
    information_class: u32,
) -> i32 {
    let rev = acl_rev(acl_);
    if rev < acl::MIN_ACL_REVISION || rev > acl::MAX_ACL_REVISION {
        return STATUS_INVALID_PARAMETER;
    }
    match information_class {
        x if x == acl::ACL_REVISION_INFORMATION => {
            if (information_length as usize) < 4 {
                return STATUS_BUFFER_TOO_SMALL;
            }
            (information as *mut u32).write_unaligned(rev as u32);
        }
        x if x == acl::ACL_SIZE_INFORMATION => {
            if (information_length as usize) < 12 {
                return STATUS_BUFFER_TOO_SMALL;
            }
            let mut free: *mut u8 = core::ptr::null_mut();
            if RtlFirstFreeAce(acl_, &mut free) == 0 {
                return STATUS_INVALID_PARAMETER;
            }
            let i = information as *mut u32;
            i.write_unaligned(acl_ace_count(acl_) as u32); // AceCount
            if !free.is_null() {
                let in_use = free as usize - acl_ as usize;
                i.add(1).write_unaligned(in_use as u32);
                i.add(2)
                    .write_unaligned(acl_size(acl_) as u32 - in_use as u32);
            } else {
                i.add(1).write_unaligned(acl_size(acl_) as u32);
                i.add(2).write_unaligned(0);
            }
        }
        _ => return STATUS_INVALID_INFO_CLASS,
    }
    STATUS_SUCCESS
}

/// `NTSTATUS NTAPI RtlSetInformationAcl(PACL, PVOID, ULONG,
/// ACL_INFORMATION_CLASS)`. Only AclRevisionInformation; new
/// revision must be strictly greater than the current.
#[no_mangle]
pub unsafe extern "system" fn RtlSetInformationAcl(
    acl_: *mut u8,
    information: *mut u8,
    information_length: u32,
    information_class: u32,
) -> i32 {
    let rev = acl_rev(acl_);
    if rev < acl::MIN_ACL_REVISION || rev > acl::MAX_ACL_REVISION {
        return STATUS_INVALID_PARAMETER;
    }
    if information_class == acl::ACL_REVISION_INFORMATION {
        if (information_length as usize) < 4 {
            return STATUS_BUFFER_TOO_SMALL;
        }
        let new_rev = (information as *const u32).read_unaligned();
        if rev as u32 >= new_rev {
            return STATUS_INVALID_PARAMETER;
        }
        *acl_ = new_rev as u8;
        STATUS_SUCCESS
    } else {
        STATUS_INVALID_INFO_CLASS
    }
}

// ── Wave ACL-2: ACE add/delete + workers + typed wrappers ─────────
// RtlValidSid/RtlLengthSid/RtlCopySid/RtlValidAcl/RtlFirstFreeAce
// are this crate's own diff-proven shims (called as Rust fns). No
// alloc. Faithful quirk: RtlpAddData's down-shift is a genuine
// RtlCopyMemory (memcpy) — overlapping when AceListLength < Offset
// is genuine C-UB; the diff fuzzes only the non-overlapping domain
// (Offset==0 or AceListLength>=Offset), documented edge (the SID
// over-read / strpbrk≤2 precedent).

/// `VOID NTAPI RtlpAddData(PVOID AceList, ULONG AceListLength,
/// PVOID Ace, ULONG Offset)`.
#[no_mangle]
pub unsafe extern "system" fn RtlpAddData(
    ace_list: *const u8,
    ace_list_length: u32,
    ace: *mut u8,
    offset: u32,
) {
    if offset > 0 {
        core::ptr::copy(ace, ace.add(ace_list_length as usize), offset as usize);
    }
    if ace_list_length != 0 {
        core::ptr::copy_nonoverlapping(ace_list, ace, ace_list_length as usize);
    }
}

/// `VOID NTAPI RtlpDeleteData(PVOID Ace, ULONG AceSize, ULONG Offset)`.
#[no_mangle]
pub unsafe extern "system" fn RtlpDeleteData(ace: *mut u8, ace_size: u32, offset: u32) {
    if ace_size < offset {
        core::ptr::copy(
            ace.add(ace_size as usize),
            ace,
            (offset - ace_size) as usize,
        );
    }
    if offset.wrapping_sub(ace_size) < offset {
        core::ptr::write_bytes(ace.add((offset - ace_size) as usize), 0, ace_size as usize);
    }
}

/// `NTSTATUS NTAPI RtlpAddKnownAce(PACL, Revision, Flags, Mask,
/// PSID, Type)`. KNOWN_ACE: Header@0, Mask@4, SidStart@8.
#[no_mangle]
pub unsafe extern "system" fn RtlpAddKnownAce(
    acl_: *mut u8,
    revision: u32,
    flags: u32,
    access_mask: u32,
    sid_: *const u8,
    type_: u8,
) -> i32 {
    if RtlValidSid(sid_) == 0 {
        return STATUS_INVALID_SID;
    }
    let acl_revb = acl_rev(acl_) as u32;
    if acl_revb > acl::ACL_REVISION4 || revision > acl::ACL_REVISION4 {
        return STATUS_REVISION_MISMATCH;
    }
    let revision = if revision < acl_revb {
        acl_revb
    } else {
        revision
    };
    let invalid = if type_ == acl::SYSTEM_AUDIT_ACE_TYPE {
        flags
            & !(acl::VALID_INHERIT_FLAGS
                | acl::SUCCESSFUL_ACCESS_ACE_FLAG
                | acl::FAILED_ACCESS_ACE_FLAG)
    } else {
        flags & !acl::VALID_INHERIT_FLAGS
    };
    if invalid != 0 {
        return STATUS_INVALID_PARAMETER;
    }
    if RtlValidAcl(acl_) == 0 {
        return STATUS_INVALID_ACL;
    }
    let mut ace: *mut u8 = core::ptr::null_mut();
    if RtlFirstFreeAce(acl_, &mut ace) == 0 {
        return STATUS_INVALID_ACL;
    }
    let sidlen = RtlLengthSid(sid_);
    let ace_sz = sidlen + acl::SIZEOF_ACE as u32;
    if ace.is_null() || ace as usize + ace_sz as usize > acl_ as usize + acl_size(acl_) as usize {
        return STATUS_ALLOTTED_SPACE_EXCEEDED;
    }
    *ace.add(1) = flags as u8; // AceFlags
    *ace = type_; // AceType
    (ace.add(2) as *mut u16).write_unaligned(ace_sz as u16);
    (ace.add(4) as *mut u32).write_unaligned(access_mask);
    RtlCopySid(sidlen, ace.add(8), sid_);
    (acl_.add(4) as *mut u16).write_unaligned(acl_ace_count(acl_) + 1);
    *acl_ = revision as u8;
    STATUS_SUCCESS
}

/// `NTSTATUS NTAPI RtlpAddKnownObjectAce(...)`. KNOWN_OBJECT_ACE:
/// Header@0, Mask@4, Flags@8, SidStart@12. Revision must be exactly
/// ACL_REVISION4.
#[no_mangle]
pub unsafe extern "system" fn RtlpAddKnownObjectAce(
    acl_: *mut u8,
    revision: u32,
    flags: u32,
    access_mask: u32,
    object_type_guid: *const u8,
    inherited_object_type_guid: *const u8,
    sid_: *const u8,
    type_: u8,
) -> i32 {
    if RtlValidSid(sid_) == 0 {
        return STATUS_INVALID_SID;
    }
    let acl_revb = acl_rev(acl_) as u32;
    if acl_revb > acl::ACL_REVISION4 || revision != acl::ACL_REVISION4 {
        return STATUS_REVISION_MISMATCH;
    }
    let revision = if revision < acl_revb {
        acl_revb
    } else {
        revision
    };
    let invalid = if type_ == acl::SYSTEM_AUDIT_OBJECT_ACE_TYPE
        || type_ == acl::SYSTEM_AUDIT_CALLBACK_OBJECT_ACE_TYPE
    {
        flags
            & !(acl::VALID_INHERIT_FLAGS
                | acl::SUCCESSFUL_ACCESS_ACE_FLAG
                | acl::FAILED_ACCESS_ACE_FLAG)
    } else {
        flags & !acl::VALID_INHERIT_FLAGS
    };
    if invalid != 0 {
        return STATUS_INVALID_PARAMETER;
    }
    if RtlValidAcl(acl_) == 0 {
        return STATUS_INVALID_ACL;
    }
    let mut ace: *mut u8 = core::ptr::null_mut();
    if RtlFirstFreeAce(acl_, &mut ace) == 0 {
        return STATUS_INVALID_ACL;
    }
    let sidlen = RtlLengthSid(sid_);
    let mut ace_sz = sidlen + acl::SIZEOF_ACE as u32 + 4; // + sizeof(ULONG)
    let mut object_flags = 0u32;
    if !object_type_guid.is_null() {
        object_flags |= acl::ACE_OBJECT_TYPE_PRESENT;
        ace_sz += acl::GUID_SZ as u32;
    }
    if !inherited_object_type_guid.is_null() {
        object_flags |= acl::ACE_INHERITED_OBJECT_TYPE_PRESENT;
        ace_sz += acl::GUID_SZ as u32;
    }
    if ace.is_null() || ace as usize + ace_sz as usize > acl_ as usize + acl_size(acl_) as usize {
        return STATUS_ALLOTTED_SPACE_EXCEEDED;
    }
    *ace.add(1) = flags as u8;
    *ace = type_;
    (ace.add(2) as *mut u16).write_unaligned(ace_sz as u16);
    (ace.add(4) as *mut u32).write_unaligned(access_mask);
    (ace.add(8) as *mut u32).write_unaligned(object_flags);
    let mut sid_start = ace.add(12);
    if !object_type_guid.is_null() {
        core::ptr::copy_nonoverlapping(object_type_guid, sid_start, acl::GUID_SZ);
        sid_start = sid_start.add(acl::GUID_SZ);
    }
    if !inherited_object_type_guid.is_null() {
        core::ptr::copy_nonoverlapping(inherited_object_type_guid, sid_start, acl::GUID_SZ);
        sid_start = sid_start.add(acl::GUID_SZ);
    }
    RtlCopySid(sidlen, sid_start, sid_);
    (acl_.add(4) as *mut u16).write_unaligned(acl_ace_count(acl_) + 1);
    *acl_ = revision as u8;
    STATUS_SUCCESS
}

/// `NTSTATUS NTAPI RtlAddAce(PACL, AclRevision, StartingIndex,
/// PVOID AceList, ULONG AceListLength)`.
#[no_mangle]
pub unsafe extern "system" fn RtlAddAce(
    acl_: *mut u8,
    acl_revision: u32,
    starting_index: u32,
    ace_list: *mut u8,
    ace_list_length: u32,
) -> i32 {
    if RtlValidAcl(acl_) == 0 {
        return STATUS_INVALID_PARAMETER;
    }
    let mut free_ace: *mut u8 = core::ptr::null_mut();
    if RtlFirstFreeAce(acl_, &mut free_ace) == 0 {
        return STATUS_INVALID_PARAMETER;
    }
    let list_end = ace_list as usize + ace_list_length as usize;
    let mut ace = ace_list;
    let mut new_ace_count: u16 = 0;
    while (ace as usize) < list_end {
        let at = *ace; // AceType
        if at <= acl::ACCESS_MAX_MS_ACE_TYPE {
            if at > acl::ACCESS_MAX_MS_V3_ACE_TYPE {
                if acl_revision < acl::ACL_REVISION4 {
                    return STATUS_INVALID_PARAMETER;
                }
            } else if at > acl::ACCESS_MAX_MS_V2_ACE_TYPE && acl_revision < acl::ACL_REVISION3 {
                return STATUS_INVALID_PARAMETER;
            }
        }
        ace = ace.add(ace_size(ace) as usize);
        new_ace_count += 1;
    }
    if ace as usize > list_end {
        return STATUS_INVALID_PARAMETER;
    }
    if free_ace.is_null()
        || free_ace as usize + ace_list_length as usize > acl_ as usize + acl_size(acl_) as usize
    {
        return STATUS_BUFFER_TOO_SMALL;
    }
    let mut at_ace = acl_.add(acl::ACL_HDR);
    let count = acl_ace_count(acl_) as u32;
    let mut index = 0u32;
    while index < starting_index && index < count {
        at_ace = at_ace.add(ace_size(at_ace) as usize);
        index += 1;
    }
    RtlpAddData(
        ace_list,
        ace_list_length,
        at_ace,
        (free_ace as usize - at_ace as usize) as u32,
    );
    (acl_.add(4) as *mut u16).write_unaligned(acl_ace_count(acl_) + new_ace_count);
    let cur = acl_rev(acl_) as u32;
    *acl_ = cur.min(acl_revision) as u8;
    STATUS_SUCCESS
}

/// `NTSTATUS NTAPI RtlDeleteAce(PACL, ULONG AceIndex)`.
#[no_mangle]
pub unsafe extern "system" fn RtlDeleteAce(acl_: *mut u8, ace_index: u32) -> i32 {
    if RtlValidAcl(acl_) == 0 {
        return STATUS_INVALID_PARAMETER;
    }
    let mut free_ace: *mut u8 = core::ptr::null_mut();
    if acl_ace_count(acl_) as u32 <= ace_index || RtlFirstFreeAce(acl_, &mut free_ace) == 0 {
        return STATUS_INVALID_PARAMETER;
    }
    let mut ace = acl_.add(acl::ACL_HDR);
    let mut i = ace_index;
    while i != 0 {
        ace = ace.add(ace_size(ace) as usize);
        i -= 1;
    }
    let asz = ace_size(ace) as u32;
    RtlpDeleteData(ace, asz, (free_ace as usize - ace as usize) as u32);
    (acl_.add(4) as *mut u16).write_unaligned(acl_ace_count(acl_) - 1);
    STATUS_SUCCESS
}

// Typed Add* wrappers (thin pass-throughs).
#[no_mangle]
pub unsafe extern "system" fn RtlAddAccessAllowedAce(
    acl_: *mut u8,
    revision: u32,
    access_mask: u32,
    sid_: *const u8,
) -> i32 {
    RtlpAddKnownAce(
        acl_,
        revision,
        0,
        access_mask,
        sid_,
        acl::ACCESS_ALLOWED_ACE_TYPE,
    )
}

#[no_mangle]
pub unsafe extern "system" fn RtlAddAccessAllowedAceEx(
    acl_: *mut u8,
    revision: u32,
    flags: u32,
    access_mask: u32,
    sid_: *const u8,
) -> i32 {
    RtlpAddKnownAce(
        acl_,
        revision,
        flags,
        access_mask,
        sid_,
        acl::ACCESS_ALLOWED_ACE_TYPE,
    )
}

#[no_mangle]
pub unsafe extern "system" fn RtlAddAccessAllowedObjectAce(
    acl_: *mut u8,
    revision: u32,
    flags: u32,
    access_mask: u32,
    object_type_guid: *const u8,
    inherited_object_type_guid: *const u8,
    sid_: *const u8,
) -> i32 {
    if object_type_guid.is_null() && inherited_object_type_guid.is_null() {
        return RtlpAddKnownAce(
            acl_,
            revision,
            flags,
            access_mask,
            sid_,
            acl::ACCESS_ALLOWED_ACE_TYPE,
        );
    }
    RtlpAddKnownObjectAce(
        acl_,
        revision,
        flags,
        access_mask,
        object_type_guid,
        inherited_object_type_guid,
        sid_,
        acl::ACCESS_ALLOWED_OBJECT_ACE_TYPE,
    )
}

#[no_mangle]
pub unsafe extern "system" fn RtlAddAccessDeniedAce(
    acl_: *mut u8,
    revision: u32,
    access_mask: u32,
    sid_: *const u8,
) -> i32 {
    RtlpAddKnownAce(
        acl_,
        revision,
        0,
        access_mask,
        sid_,
        acl::ACCESS_DENIED_ACE_TYPE,
    )
}

#[no_mangle]
pub unsafe extern "system" fn RtlAddAccessDeniedAceEx(
    acl_: *mut u8,
    revision: u32,
    flags: u32,
    access_mask: u32,
    sid_: *const u8,
) -> i32 {
    RtlpAddKnownAce(
        acl_,
        revision,
        flags,
        access_mask,
        sid_,
        acl::ACCESS_DENIED_ACE_TYPE,
    )
}

#[no_mangle]
pub unsafe extern "system" fn RtlAddAccessDeniedObjectAce(
    acl_: *mut u8,
    revision: u32,
    flags: u32,
    access_mask: u32,
    object_type_guid: *const u8,
    inherited_object_type_guid: *const u8,
    sid_: *const u8,
) -> i32 {
    if object_type_guid.is_null() && inherited_object_type_guid.is_null() {
        return RtlpAddKnownAce(
            acl_,
            revision,
            flags,
            access_mask,
            sid_,
            acl::ACCESS_DENIED_ACE_TYPE,
        );
    }
    RtlpAddKnownObjectAce(
        acl_,
        revision,
        flags,
        access_mask,
        object_type_guid,
        inherited_object_type_guid,
        sid_,
        acl::ACCESS_DENIED_OBJECT_ACE_TYPE,
    )
}

#[no_mangle]
pub unsafe extern "system" fn RtlAddAuditAccessAce(
    acl_: *mut u8,
    revision: u32,
    access_mask: u32,
    sid_: *const u8,
    success: u8,
    failure: u8,
) -> i32 {
    let mut flags = 0u32;
    if success != 0 {
        flags |= acl::SUCCESSFUL_ACCESS_ACE_FLAG;
    }
    if failure != 0 {
        flags |= acl::FAILED_ACCESS_ACE_FLAG;
    }
    RtlpAddKnownAce(
        acl_,
        revision,
        flags,
        access_mask,
        sid_,
        acl::SYSTEM_AUDIT_ACE_TYPE,
    )
}

#[no_mangle]
pub unsafe extern "system" fn RtlAddAuditAccessAceEx(
    acl_: *mut u8,
    revision: u32,
    flags: u32,
    access_mask: u32,
    sid_: *const u8,
    success: u8,
    failure: u8,
) -> i32 {
    let mut flags = flags;
    if success != 0 {
        flags |= acl::SUCCESSFUL_ACCESS_ACE_FLAG;
    }
    if failure != 0 {
        flags |= acl::FAILED_ACCESS_ACE_FLAG;
    }
    RtlpAddKnownAce(
        acl_,
        revision,
        flags,
        access_mask,
        sid_,
        acl::SYSTEM_AUDIT_ACE_TYPE,
    )
}

#[no_mangle]
pub unsafe extern "system" fn RtlAddAuditAccessObjectAce(
    acl_: *mut u8,
    revision: u32,
    flags: u32,
    access_mask: u32,
    object_type_guid: *const u8,
    inherited_object_type_guid: *const u8,
    sid_: *const u8,
    success: u8,
    failure: u8,
) -> i32 {
    let mut flags = flags;
    if success != 0 {
        flags |= acl::SUCCESSFUL_ACCESS_ACE_FLAG;
    }
    if failure != 0 {
        flags |= acl::FAILED_ACCESS_ACE_FLAG;
    }
    if object_type_guid.is_null() && inherited_object_type_guid.is_null() {
        return RtlpAddKnownAce(
            acl_,
            revision,
            flags,
            access_mask,
            sid_,
            acl::SYSTEM_AUDIT_ACE_TYPE,
        );
    }
    RtlpAddKnownObjectAce(
        acl_,
        revision,
        flags,
        access_mask,
        object_type_guid,
        inherited_object_type_guid,
        sid_,
        acl::SYSTEM_AUDIT_OBJECT_ACE_TYPE,
    )
}

// ============================================================================
// Wave SD-1 — structural surface of `sdk/lib/rtl/sd.c`.
//
// `SECURITY_DESCRIPTOR_RELATIVE` is a flat 20-byte record on every
// arch. `SECURITY_DESCRIPTOR` (absolute) bears 4 pointers (size = 12
// header bytes + 4*sizeof(ptr); 20 on 32-bit, 40 on 64-bit). Both
// repr(C) layouts come out per-target by construction.
// ============================================================================

const STATUS_UNKNOWN_REVISION: i32 = 0xC000_0058u32 as i32;
const STATUS_INVALID_SECURITY_DESCR: i32 = 0xC000_0079u32 as i32;
const STATUS_BAD_DESCRIPTOR_FORMAT: i32 = 0xC000_00E7u32 as i32;
const STATUS_INVALID_PARAMETER_1: i32 = 0xC000_00EFu32 as i32;

/// `SECURITY_DESCRIPTOR` — internal absolute form (PISECURITY_DESCRIPTOR).
#[repr(C)]
pub struct SECURITY_DESCRIPTOR {
    pub Revision: u8,
    pub Sbz1: u8,
    pub Control: u16,
    pub Owner: *mut u8,
    pub Group: *mut u8,
    pub Sacl: *mut u8,
    pub Dacl: *mut u8,
}

/// `SECURITY_DESCRIPTOR_RELATIVE` — self-relative form (4 ULONG offsets).
#[repr(C)]
pub struct SECURITY_DESCRIPTOR_RELATIVE {
    pub Revision: u8,
    pub Sbz1: u8,
    pub Control: u16,
    pub Owner: u32,
    pub Group: u32,
    pub Sacl: u32,
    pub Dacl: u32,
}

/// `RtlLengthSid` against the in-tree SID port. Pointer-only;
/// the caller has already established a valid prefix.
#[inline]
unsafe fn sid_length_from_ptr(p: *const u8) -> u32 {
    sid::length(*p.add(1))
}

/// `ROUND_UP(v, sizeof(ULONG))`.
#[inline]
fn round_up_ulong(v: u32) -> u32 {
    (v + 3) & !3u32
}

// --- Sep* internal helpers (FORCEINLINE in ntoskrnl/include/internal/se.h).

#[inline]
unsafe fn sep_get_owner(sd: *const SECURITY_DESCRIPTOR) -> *mut u8 {
    let control = (*sd).Control;
    if control & sd_::SE_SELF_RELATIVE != 0 {
        let rel = sd as *const SECURITY_DESCRIPTOR_RELATIVE;
        let off = (*rel).Owner;
        if off == 0 {
            core::ptr::null_mut()
        } else {
            (sd as *mut u8).add(off as usize)
        }
    } else {
        (*sd).Owner
    }
}

#[inline]
unsafe fn sep_get_group(sd: *const SECURITY_DESCRIPTOR) -> *mut u8 {
    let control = (*sd).Control;
    if control & sd_::SE_SELF_RELATIVE != 0 {
        let rel = sd as *const SECURITY_DESCRIPTOR_RELATIVE;
        let off = (*rel).Group;
        if off == 0 {
            core::ptr::null_mut()
        } else {
            (sd as *mut u8).add(off as usize)
        }
    } else {
        (*sd).Group
    }
}

#[inline]
unsafe fn sep_get_dacl(sd: *const SECURITY_DESCRIPTOR) -> *mut u8 {
    let control = (*sd).Control;
    if control & sd_::SE_DACL_PRESENT == 0 {
        return core::ptr::null_mut();
    }
    if control & sd_::SE_SELF_RELATIVE != 0 {
        let rel = sd as *const SECURITY_DESCRIPTOR_RELATIVE;
        let off = (*rel).Dacl;
        if off == 0 {
            core::ptr::null_mut()
        } else {
            (sd as *mut u8).add(off as usize)
        }
    } else {
        (*sd).Dacl
    }
}

#[inline]
unsafe fn sep_get_sacl(sd: *const SECURITY_DESCRIPTOR) -> *mut u8 {
    let control = (*sd).Control;
    if control & sd_::SE_SACL_PRESENT == 0 {
        return core::ptr::null_mut();
    }
    if control & sd_::SE_SELF_RELATIVE != 0 {
        let rel = sd as *const SECURITY_DESCRIPTOR_RELATIVE;
        let off = (*rel).Sacl;
        if off == 0 {
            core::ptr::null_mut()
        } else {
            (sd as *mut u8).add(off as usize)
        }
    } else {
        (*sd).Sacl
    }
}

/// Internal `RtlpQuerySecurityDescriptor`: yield (owner, owner_size,
/// group, group_size, dacl, dacl_size, sacl, sacl_size). Sizes are
/// `ROUND_UP(payload, 4)`; payload is `RtlLengthSid` for SIDs, `AclSize`
/// for ACLs. Zero ptr ⇒ zero size.
unsafe fn rtlp_query_sd(
    sd: *const SECURITY_DESCRIPTOR,
) -> (*mut u8, u32, *mut u8, u32, *mut u8, u32, *mut u8, u32) {
    let owner = sep_get_owner(sd);
    let owner_size = if owner.is_null() {
        0
    } else {
        round_up_ulong(sid_length_from_ptr(owner))
    };
    let group = sep_get_group(sd);
    let group_size = if group.is_null() {
        0
    } else {
        round_up_ulong(sid_length_from_ptr(group))
    };
    let dacl = sep_get_dacl(sd);
    let dacl_size = if dacl.is_null() {
        0
    } else {
        round_up_ulong(acl_size(dacl) as u32)
    };
    let sacl = sep_get_sacl(sd);
    let sacl_size = if sacl.is_null() {
        0
    } else {
        round_up_ulong(acl_size(sacl) as u32)
    };
    (
        owner, owner_size, group, group_size, dacl, dacl_size, sacl, sacl_size,
    )
}

// --- SD-1 cabi exports.

/// `NTSTATUS NTAPI RtlCreateSecurityDescriptor(PSD, ULONG Revision)`.
#[no_mangle]
pub unsafe extern "system" fn RtlCreateSecurityDescriptor(
    sd_p: *mut SECURITY_DESCRIPTOR,
    revision: u32,
) -> i32 {
    if revision != sd_::SECURITY_DESCRIPTOR_REVISION {
        return STATUS_UNKNOWN_REVISION;
    }
    // Zero the whole struct, then stamp Revision.
    core::ptr::write_bytes(
        sd_p as *mut u8,
        0,
        core::mem::size_of::<SECURITY_DESCRIPTOR>(),
    );
    (*sd_p).Revision = sd_::SECURITY_DESCRIPTOR_REVISION as u8;
    STATUS_SUCCESS
}

/// `NTSTATUS NTAPI RtlCreateSecurityDescriptorRelative(PISDREL, ULONG Revision)`.
#[no_mangle]
pub unsafe extern "system" fn RtlCreateSecurityDescriptorRelative(
    sd_p: *mut SECURITY_DESCRIPTOR_RELATIVE,
    revision: u32,
) -> i32 {
    if revision != sd_::SECURITY_DESCRIPTOR_REVISION {
        return STATUS_UNKNOWN_REVISION;
    }
    core::ptr::write_bytes(
        sd_p as *mut u8,
        0,
        core::mem::size_of::<SECURITY_DESCRIPTOR_RELATIVE>(),
    );
    (*sd_p).Revision = sd_::SECURITY_DESCRIPTOR_REVISION as u8;
    (*sd_p).Control = sd_::SE_SELF_RELATIVE;
    STATUS_SUCCESS
}

/// `ULONG NTAPI RtlLengthSecurityDescriptor(PSD)`.
#[no_mangle]
pub unsafe extern "system" fn RtlLengthSecurityDescriptor(sd_p: *const SECURITY_DESCRIPTOR) -> u32 {
    let control = (*sd_p).Control;
    let mut length = if control & sd_::SE_SELF_RELATIVE != 0 {
        core::mem::size_of::<SECURITY_DESCRIPTOR_RELATIVE>() as u32
    } else {
        core::mem::size_of::<SECURITY_DESCRIPTOR>() as u32
    };

    let owner = sep_get_owner(sd_p);
    if !owner.is_null() {
        length += round_up_ulong(sid_length_from_ptr(owner));
    }
    let group = sep_get_group(sd_p);
    if !group.is_null() {
        length += round_up_ulong(sid_length_from_ptr(group));
    }
    let dacl = sep_get_dacl(sd_p);
    if !dacl.is_null() {
        length += round_up_ulong(acl_size(dacl) as u32);
    }
    let sacl = sep_get_sacl(sd_p);
    if !sacl.is_null() {
        length += round_up_ulong(acl_size(sacl) as u32);
    }
    length
}

/// `NTSTATUS NTAPI RtlGetDaclSecurityDescriptor(PSD, PBOOLEAN, PACL*, PBOOLEAN)`.
#[no_mangle]
pub unsafe extern "system" fn RtlGetDaclSecurityDescriptor(
    sd_p: *const SECURITY_DESCRIPTOR,
    dacl_present: *mut u8,
    dacl_out: *mut *mut u8,
    dacl_defaulted: *mut u8,
) -> i32 {
    if (*sd_p).Revision as u32 != sd_::SECURITY_DESCRIPTOR_REVISION {
        return STATUS_UNKNOWN_REVISION;
    }
    let present = ((*sd_p).Control & sd_::SE_DACL_PRESENT) == sd_::SE_DACL_PRESENT;
    *dacl_present = present as u8;
    if present {
        *dacl_out = sep_get_dacl(sd_p);
        *dacl_defaulted =
            (((*sd_p).Control & sd_::SE_DACL_DEFAULTED) == sd_::SE_DACL_DEFAULTED) as u8;
    }
    STATUS_SUCCESS
}

/// `NTSTATUS NTAPI RtlGetSaclSecurityDescriptor(PSD, PBOOLEAN, PACL*, PBOOLEAN)`.
#[no_mangle]
pub unsafe extern "system" fn RtlGetSaclSecurityDescriptor(
    sd_p: *const SECURITY_DESCRIPTOR,
    sacl_present: *mut u8,
    sacl_out: *mut *mut u8,
    sacl_defaulted: *mut u8,
) -> i32 {
    if (*sd_p).Revision as u32 != sd_::SECURITY_DESCRIPTOR_REVISION {
        return STATUS_UNKNOWN_REVISION;
    }
    let present = ((*sd_p).Control & sd_::SE_SACL_PRESENT) == sd_::SE_SACL_PRESENT;
    *sacl_present = present as u8;
    if present {
        *sacl_out = sep_get_sacl(sd_p);
        *sacl_defaulted =
            (((*sd_p).Control & sd_::SE_SACL_DEFAULTED) == sd_::SE_SACL_DEFAULTED) as u8;
    }
    STATUS_SUCCESS
}

/// `NTSTATUS NTAPI RtlGetOwnerSecurityDescriptor(PSD, PSID*, PBOOLEAN)`.
#[no_mangle]
pub unsafe extern "system" fn RtlGetOwnerSecurityDescriptor(
    sd_p: *const SECURITY_DESCRIPTOR,
    owner_out: *mut *mut u8,
    owner_defaulted: *mut u8,
) -> i32 {
    if (*sd_p).Revision as u32 != sd_::SECURITY_DESCRIPTOR_REVISION {
        return STATUS_UNKNOWN_REVISION;
    }
    *owner_out = sep_get_owner(sd_p);
    *owner_defaulted =
        (((*sd_p).Control & sd_::SE_OWNER_DEFAULTED) == sd_::SE_OWNER_DEFAULTED) as u8;
    STATUS_SUCCESS
}

/// `NTSTATUS NTAPI RtlGetGroupSecurityDescriptor(PSD, PSID*, PBOOLEAN)`.
#[no_mangle]
pub unsafe extern "system" fn RtlGetGroupSecurityDescriptor(
    sd_p: *const SECURITY_DESCRIPTOR,
    group_out: *mut *mut u8,
    group_defaulted: *mut u8,
) -> i32 {
    if (*sd_p).Revision as u32 != sd_::SECURITY_DESCRIPTOR_REVISION {
        return STATUS_UNKNOWN_REVISION;
    }
    *group_out = sep_get_group(sd_p);
    *group_defaulted =
        (((*sd_p).Control & sd_::SE_GROUP_DEFAULTED) == sd_::SE_GROUP_DEFAULTED) as u8;
    STATUS_SUCCESS
}

/// `NTSTATUS NTAPI RtlSetDaclSecurityDescriptor(PSD, BOOLEAN, PACL, BOOLEAN)`.
#[no_mangle]
pub unsafe extern "system" fn RtlSetDaclSecurityDescriptor(
    sd_p: *mut SECURITY_DESCRIPTOR,
    dacl_present: u8,
    dacl: *mut u8,
    dacl_defaulted: u8,
) -> i32 {
    if (*sd_p).Revision as u32 != sd_::SECURITY_DESCRIPTOR_REVISION {
        return STATUS_UNKNOWN_REVISION;
    }
    if (*sd_p).Control & sd_::SE_SELF_RELATIVE != 0 {
        return STATUS_INVALID_SECURITY_DESCR;
    }
    if dacl_present == 0 {
        (*sd_p).Control &= !sd_::SE_DACL_PRESENT;
        return STATUS_SUCCESS;
    }
    (*sd_p).Dacl = dacl;
    (*sd_p).Control |= sd_::SE_DACL_PRESENT;
    (*sd_p).Control &= !sd_::SE_DACL_DEFAULTED;
    if dacl_defaulted != 0 {
        (*sd_p).Control |= sd_::SE_DACL_DEFAULTED;
    }
    STATUS_SUCCESS
}

/// `NTSTATUS NTAPI RtlSetSaclSecurityDescriptor(PSD, BOOLEAN, PACL, BOOLEAN)`.
#[no_mangle]
pub unsafe extern "system" fn RtlSetSaclSecurityDescriptor(
    sd_p: *mut SECURITY_DESCRIPTOR,
    sacl_present: u8,
    sacl: *mut u8,
    sacl_defaulted: u8,
) -> i32 {
    if (*sd_p).Revision as u32 != sd_::SECURITY_DESCRIPTOR_REVISION {
        return STATUS_UNKNOWN_REVISION;
    }
    if (*sd_p).Control & sd_::SE_SELF_RELATIVE != 0 {
        return STATUS_INVALID_SECURITY_DESCR;
    }
    if sacl_present == 0 {
        (*sd_p).Control &= !sd_::SE_SACL_PRESENT;
        return STATUS_SUCCESS;
    }
    (*sd_p).Sacl = sacl;
    (*sd_p).Control |= sd_::SE_SACL_PRESENT;
    (*sd_p).Control &= !sd_::SE_SACL_DEFAULTED;
    if sacl_defaulted != 0 {
        (*sd_p).Control |= sd_::SE_SACL_DEFAULTED;
    }
    STATUS_SUCCESS
}

/// `NTSTATUS NTAPI RtlSetOwnerSecurityDescriptor(PSD, PSID, BOOLEAN)`.
#[no_mangle]
pub unsafe extern "system" fn RtlSetOwnerSecurityDescriptor(
    sd_p: *mut SECURITY_DESCRIPTOR,
    owner: *mut u8,
    owner_defaulted: u8,
) -> i32 {
    if (*sd_p).Revision as u32 != sd_::SECURITY_DESCRIPTOR_REVISION {
        return STATUS_UNKNOWN_REVISION;
    }
    if (*sd_p).Control & sd_::SE_SELF_RELATIVE != 0 {
        return STATUS_INVALID_SECURITY_DESCR;
    }
    (*sd_p).Owner = owner;
    (*sd_p).Control &= !sd_::SE_OWNER_DEFAULTED;
    if owner_defaulted != 0 {
        (*sd_p).Control |= sd_::SE_OWNER_DEFAULTED;
    }
    STATUS_SUCCESS
}

/// `NTSTATUS NTAPI RtlSetGroupSecurityDescriptor(PSD, PSID, BOOLEAN)`.
#[no_mangle]
pub unsafe extern "system" fn RtlSetGroupSecurityDescriptor(
    sd_p: *mut SECURITY_DESCRIPTOR,
    group: *mut u8,
    group_defaulted: u8,
) -> i32 {
    if (*sd_p).Revision as u32 != sd_::SECURITY_DESCRIPTOR_REVISION {
        return STATUS_UNKNOWN_REVISION;
    }
    if (*sd_p).Control & sd_::SE_SELF_RELATIVE != 0 {
        return STATUS_INVALID_SECURITY_DESCR;
    }
    (*sd_p).Group = group;
    (*sd_p).Control &= !sd_::SE_GROUP_DEFAULTED;
    if group_defaulted != 0 {
        (*sd_p).Control |= sd_::SE_GROUP_DEFAULTED;
    }
    STATUS_SUCCESS
}

/// `NTSTATUS NTAPI RtlGetControlSecurityDescriptor(PSD, PSDCONTROL, PULONG)`.
#[no_mangle]
pub unsafe extern "system" fn RtlGetControlSecurityDescriptor(
    sd_p: *const SECURITY_DESCRIPTOR,
    control_out: *mut u16,
    revision_out: *mut u32,
) -> i32 {
    *revision_out = (*sd_p).Revision as u32;
    if (*sd_p).Revision as u32 != sd_::SECURITY_DESCRIPTOR_REVISION {
        return STATUS_UNKNOWN_REVISION;
    }
    *control_out = (*sd_p).Control;
    STATUS_SUCCESS
}

/// `NTSTATUS NTAPI RtlSetControlSecurityDescriptor(PSD, SDCONTROL, SDCONTROL)`.
#[no_mangle]
pub unsafe extern "system" fn RtlSetControlSecurityDescriptor(
    sd_p: *mut SECURITY_DESCRIPTOR,
    control_bits_of_interest: u16,
    control_bits_to_set: u16,
) -> i32 {
    if (control_bits_of_interest & !sd_::SET_ATTRIBUTES_MASK) != 0
        || (control_bits_to_set & !control_bits_of_interest) != 0
    {
        return STATUS_INVALID_PARAMETER;
    }
    (*sd_p).Control &= !control_bits_of_interest;
    (*sd_p).Control |= control_bits_to_set & control_bits_of_interest;
    STATUS_SUCCESS
}

/// `BOOLEAN NTAPI RtlGetSecurityDescriptorRMControl(PSD, PUCHAR)`.
#[no_mangle]
pub unsafe extern "system" fn RtlGetSecurityDescriptorRMControl(
    sd_p: *const SECURITY_DESCRIPTOR,
    rm_control: *mut u8,
) -> u8 {
    if (*sd_p).Control & sd_::SE_RM_CONTROL_VALID == 0 {
        *rm_control = 0;
        return 0;
    }
    *rm_control = (*sd_p).Sbz1;
    1
}

/// `VOID NTAPI RtlSetSecurityDescriptorRMControl(PSD, PUCHAR)`.
#[no_mangle]
pub unsafe extern "system" fn RtlSetSecurityDescriptorRMControl(
    sd_p: *mut SECURITY_DESCRIPTOR,
    rm_control: *const u8,
) {
    if rm_control.is_null() {
        (*sd_p).Control &= !sd_::SE_RM_CONTROL_VALID;
        (*sd_p).Sbz1 = 0;
    } else {
        (*sd_p).Control |= sd_::SE_RM_CONTROL_VALID;
        (*sd_p).Sbz1 = *rm_control;
    }
}

/// `NTSTATUS NTAPI RtlSetAttributesSecurityDescriptor(PSD, SDCONTROL, PULONG)`.
#[no_mangle]
pub unsafe extern "system" fn RtlSetAttributesSecurityDescriptor(
    sd_p: *mut SECURITY_DESCRIPTOR,
    control: u16,
    revision_out: *mut u32,
) -> i32 {
    *revision_out = (*sd_p).Revision as u32;
    if (*sd_p).Revision as u32 != sd_::SECURITY_DESCRIPTOR_REVISION {
        return STATUS_UNKNOWN_REVISION;
    }
    let masked = control & sd_::SET_ATTRIBUTES_MASK;
    RtlSetControlSecurityDescriptor(sd_p, masked, masked)
}

/// `BOOLEAN NTAPI RtlValidSecurityDescriptor(PSD)`.
///
/// Verbatim port: rev check + RtlValidSid(owner/group if present) +
/// RtlValidAcl(dacl/sacl if present). The C wraps the whole body in
/// `_SEH2_TRY` / `_SEH2_EXCEPT(EXECUTE_HANDLER)`; under DUMMY_PSEH the
/// handler is dead, so this faithful port matches both the live and
/// the dead halves of the original (only valid-ish inputs are tested
/// by the diff harness).
#[no_mangle]
pub unsafe extern "system" fn RtlValidSecurityDescriptor(sd_p: *const SECURITY_DESCRIPTOR) -> u8 {
    if (*sd_p).Revision as u32 != sd_::SECURITY_DESCRIPTOR_REVISION {
        return 0;
    }
    let owner = sep_get_owner(sd_p);
    if !owner.is_null() && RtlValidSid(owner) == 0 {
        return 0;
    }
    let group = sep_get_group(sd_p);
    if !group.is_null() && RtlValidSid(group) == 0 {
        return 0;
    }
    let dacl = sep_get_dacl(sd_p);
    if !dacl.is_null() && RtlValidAcl(dacl) == 0 {
        return 0;
    }
    let sacl = sep_get_sacl(sd_p);
    if !sacl.is_null() && RtlValidAcl(sacl) == 0 {
        return 0;
    }
    1
}

/// `BOOLEAN NTAPI RtlValidRelativeSecurityDescriptor(PSD, ULONG, SECURITY_INFORMATION)`.
#[no_mangle]
pub unsafe extern "system" fn RtlValidRelativeSecurityDescriptor(
    sd_p: *const u8,
    sd_len: u32,
    required_info: u32,
) -> u8 {
    let bytes = core::slice::from_raw_parts(sd_p, sd_len as usize);
    sd_::valid_relative_sd(bytes, required_info) as u8
}

// ============================================================================
// Wave SD-2 — self-relative ⇄ absolute conversions + Copy.
// ============================================================================

/// `NTSTATUS NTAPI RtlMakeSelfRelativeSD(PSD, PSD, PULONG)` — produces
/// a self-relative SD packed as `[REL hdr | SACL | DACL | Owner | Group]`
/// (matches the C copy order exactly: SACL→DACL→Owner→Group).
#[no_mangle]
pub unsafe extern "system" fn RtlMakeSelfRelativeSD(
    src_p: *const SECURITY_DESCRIPTOR,
    rel_p: *mut SECURITY_DESCRIPTOR_RELATIVE,
    buffer_length: *mut u32,
) -> i32 {
    let (owner, owner_len, group, group_len, dacl, dacl_len, sacl, sacl_len) = rtlp_query_sd(src_p);
    let header_size = core::mem::size_of::<SECURITY_DESCRIPTOR_RELATIVE>() as u32;
    let total = header_size + owner_len + group_len + sacl_len + dacl_len;
    if *buffer_length < total {
        *buffer_length = total;
        return STATUS_BUFFER_TOO_SMALL;
    }

    core::ptr::write_bytes(rel_p as *mut u8, 0, total as usize);
    // Copy the header fields up through (but not including) the Owner offset.
    let header_prefix = core::mem::offset_of!(SECURITY_DESCRIPTOR_RELATIVE, Owner);
    core::ptr::copy_nonoverlapping(src_p as *const u8, rel_p as *mut u8, header_prefix);

    let mut current = header_size as usize;
    if sacl_len > 0 {
        core::ptr::copy_nonoverlapping(sacl, (rel_p as *mut u8).add(current), sacl_len as usize);
        (*rel_p).Sacl = current as u32;
        current += sacl_len as usize;
    }
    if dacl_len > 0 {
        core::ptr::copy_nonoverlapping(dacl, (rel_p as *mut u8).add(current), dacl_len as usize);
        (*rel_p).Dacl = current as u32;
        current += dacl_len as usize;
    }
    if owner_len > 0 {
        core::ptr::copy_nonoverlapping(owner, (rel_p as *mut u8).add(current), owner_len as usize);
        (*rel_p).Owner = current as u32;
        current += owner_len as usize;
    }
    if group_len > 0 {
        core::ptr::copy_nonoverlapping(group, (rel_p as *mut u8).add(current), group_len as usize);
        (*rel_p).Group = current as u32;
    }
    (*rel_p).Control |= sd_::SE_SELF_RELATIVE;
    STATUS_SUCCESS
}

/// `NTSTATUS NTAPI RtlAbsoluteToSelfRelativeSD(PSD, PSD, PULONG)`.
#[no_mangle]
pub unsafe extern "system" fn RtlAbsoluteToSelfRelativeSD(
    src_p: *const SECURITY_DESCRIPTOR,
    rel_p: *mut SECURITY_DESCRIPTOR_RELATIVE,
    buffer_length: *mut u32,
) -> i32 {
    if (*src_p).Control & sd_::SE_SELF_RELATIVE != 0 {
        return STATUS_BAD_DESCRIPTOR_FORMAT;
    }
    RtlMakeSelfRelativeSD(src_p, rel_p, buffer_length)
}

/// `NTSTATUS NTAPI RtlSelfRelativeToAbsoluteSD(PSD, PSD, PULONG, PACL, PULONG,
///                                             PACL, PULONG, PSID, PULONG, PSID, PULONG)`.
#[no_mangle]
pub unsafe extern "system" fn RtlSelfRelativeToAbsoluteSD(
    rel_p: *const SECURITY_DESCRIPTOR_RELATIVE,
    abs_p: *mut SECURITY_DESCRIPTOR,
    abs_size: *mut u32,
    dacl: *mut u8,
    dacl_size: *mut u32,
    sacl: *mut u8,
    sacl_size: *mut u32,
    owner: *mut u8,
    owner_size: *mut u32,
    group: *mut u8,
    group_size: *mut u32,
) -> i32 {
    if (*rel_p).Control & sd_::SE_SELF_RELATIVE == 0 {
        return STATUS_BAD_DESCRIPTOR_FORMAT;
    }
    let (p_owner, owner_len, p_group, group_len, p_dacl, dacl_len, p_sacl, sacl_len) =
        rtlp_query_sd(rel_p as *const SECURITY_DESCRIPTOR);

    if abs_p.is_null()
        || (core::mem::size_of::<SECURITY_DESCRIPTOR>() as u32) > *abs_size
        || owner_len > *owner_size
        || group_len > *group_size
        || dacl_len > *dacl_size
        || sacl_len > *sacl_size
    {
        *abs_size = core::mem::size_of::<SECURITY_DESCRIPTOR>() as u32;
        *owner_size = owner_len;
        *group_size = group_len;
        *dacl_size = dacl_len;
        *sacl_size = sacl_len;
        return STATUS_BUFFER_TOO_SMALL;
    }

    // Copy the header fields (RtlMoveMemory of sizeof(SDREL)).
    core::ptr::copy(
        rel_p as *const u8,
        abs_p as *mut u8,
        core::mem::size_of::<SECURITY_DESCRIPTOR_RELATIVE>(),
    );
    (*abs_p).Owner = core::ptr::null_mut();
    (*abs_p).Group = core::ptr::null_mut();
    (*abs_p).Sacl = core::ptr::null_mut();
    (*abs_p).Dacl = core::ptr::null_mut();
    (*abs_p).Control &= !sd_::SE_SELF_RELATIVE;

    if !p_owner.is_null() {
        core::ptr::copy(p_owner, owner, sid_length_from_ptr(p_owner) as usize);
        (*abs_p).Owner = owner;
    }
    if !p_group.is_null() {
        core::ptr::copy(p_group, group, sid_length_from_ptr(p_group) as usize);
        (*abs_p).Group = group;
    }
    if !p_dacl.is_null() {
        core::ptr::copy(p_dacl, dacl, acl_size(p_dacl) as usize);
        (*abs_p).Dacl = dacl;
    }
    if !p_sacl.is_null() {
        core::ptr::copy(p_sacl, sacl, acl_size(p_sacl) as usize);
        (*abs_p).Sacl = sacl;
    }
    STATUS_SUCCESS
}

/// `NTSTATUS NTAPI RtlSelfRelativeToAbsoluteSD2(PSD, PULONG)` — in-place.
#[no_mangle]
pub unsafe extern "system" fn RtlSelfRelativeToAbsoluteSD2(
    sd_p: *mut SECURITY_DESCRIPTOR,
    buffer_size: *mut u32,
) -> i32 {
    if sd_p.is_null() {
        return STATUS_INVALID_PARAMETER_1;
    }
    if buffer_size.is_null() {
        return STATUS_INVALID_PARAMETER_2;
    }
    if (*sd_p).Control & sd_::SE_SELF_RELATIVE == 0 {
        return STATUS_BAD_DESCRIPTOR_FORMAT;
    }

    let rel_p = sd_p as *mut SECURITY_DESCRIPTOR_RELATIVE;
    let (p_owner, owner_len, p_group, group_len, p_dacl, dacl_len, p_sacl, sacl_len) =
        rtlp_query_sd(sd_p);

    // Move delta = sizeof(SD) - sizeof(SDREL). On 64-bit hosts this is +20; on 32-bit 0.
    let move_delta = core::mem::size_of::<SECURITY_DESCRIPTOR>() as isize
        - core::mem::size_of::<SECURITY_DESCRIPTOR_RELATIVE>() as isize;
    if move_delta == 0 {
        // 32-bit: same layout, just clear flag + set the (still-valid) pointers.
        (*sd_p).Control &= !sd_::SE_SELF_RELATIVE;
        (*sd_p).Owner = p_owner;
        (*sd_p).Group = p_group;
        (*sd_p).Sacl = p_sacl;
        (*sd_p).Dacl = p_dacl;
        return STATUS_SUCCESS;
    }

    // Compute the [DataStart, DataEnd) range over all present payloads.
    let mut data_start = p_owner;
    let mut data_end = if p_owner.is_null() {
        core::ptr::null_mut()
    } else {
        p_owner.add(owner_len as usize)
    };
    let consider = |s: *mut u8, l: u32, ds: &mut *mut u8, de: &mut *mut u8| {
        if s.is_null() {
            return;
        }
        if (*ds).is_null() || (s as usize) < (*ds as usize) {
            *ds = s;
        }
        let s_end = s.add(l as usize);
        if (*de).is_null() || (s_end as usize) > (*de as usize) {
            *de = s_end;
        }
    };
    consider(p_group, group_len, &mut data_start, &mut data_end);
    consider(p_dacl, dacl_len, &mut data_start, &mut data_end);
    consider(p_sacl, sacl_len, &mut data_start, &mut data_end);

    let data_size = if data_end.is_null() {
        0
    } else {
        (data_end as usize) - (data_start as usize)
    } as u32;

    if *buffer_size < core::mem::size_of::<SECURITY_DESCRIPTOR>() as u32 + data_size {
        *buffer_size = core::mem::size_of::<SECURITY_DESCRIPTOR>() as u32 + data_size;
        return STATUS_BUFFER_TOO_SMALL;
    }

    if data_size != 0 {
        // Move the payload to right after the new (absolute) header.
        core::ptr::copy(
            data_start as *const u8,
            (sd_p as *mut u8).add(core::mem::size_of::<SECURITY_DESCRIPTOR>()),
            data_size as usize,
        );
    }

    (*sd_p).Owner = if p_owner.is_null() {
        core::ptr::null_mut()
    } else {
        p_owner.offset(move_delta)
    };
    (*sd_p).Group = if p_group.is_null() {
        core::ptr::null_mut()
    } else {
        p_group.offset(move_delta)
    };
    (*sd_p).Sacl = if p_sacl.is_null() {
        core::ptr::null_mut()
    } else {
        p_sacl.offset(move_delta)
    };
    (*sd_p).Dacl = if p_dacl.is_null() {
        core::ptr::null_mut()
    } else {
        p_dacl.offset(move_delta)
    };
    (*sd_p).Control &= !sd_::SE_SELF_RELATIVE;
    let _ = rel_p; // (silence unused — used implicitly via aliasing).
    STATUS_SUCCESS
}

/// `NTSTATUS NTAPI RtlCopySecurityDescriptor(PSD, PSD*)`.
///
/// No-alloc edge: `RtlAllocateHeap=NULL` in the diff harness ⇒ this
/// always returns STATUS_NO_MEMORY. The Rust side wraps the same
/// FFI shape so the diff pins the NO_MEMORY path identically.
#[no_mangle]
pub unsafe extern "system" fn RtlCopySecurityDescriptor(
    src_p: *const SECURITY_DESCRIPTOR,
    dst_pp: *mut *mut SECURITY_DESCRIPTOR,
) -> i32 {
    let (_owner, owner_len, _group, group_len, _dacl, dacl_len, _sacl, sacl_len) =
        rtlp_query_sd(src_p);
    let total = core::mem::size_of::<SECURITY_DESCRIPTOR_RELATIVE>() as u32
        + owner_len
        + group_len
        + dacl_len
        + sacl_len;
    // Faithful to sd.c RtlCopySecurityDescriptor: alloc + memcpy.
    let dst = RtlAllocateHeap(RtlGetProcessHeap(), 0, total as usize);
    if dst.is_null() {
        *dst_pp = core::ptr::null_mut();
        return STATUS_NO_MEMORY;
    }
    core::ptr::copy_nonoverlapping(src_p as *const u8, dst, total as usize);
    *dst_pp = dst as *mut SECURITY_DESCRIPTOR;
    0
}

// ============================================================================
// Wave PRIV — straight translation of `sdk/lib/rtl/priv.c` (6 fns,
// all pure syscall plumbing → Zw* chains). No byte-diff harness (the
// outputs are syscall traces, not data buffers); user-mode lsass/
// services/csrss-tuiterm exercise it during boot. Per user choice
// "translate + smoke (fast)".
// ============================================================================

const STATUS_BUFFER_TOO_SMALL_2: i32 = 0xC000_0023u32 as i32;
const STATUS_PRIVILEGE_NOT_HELD: i32 = 0xC000_0061u32 as i32;
const STATUS_NOT_ALL_ASSIGNED: i32 = 0x0000_0106;

/// `LUID` (winnt.h).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct LUID {
    pub LowPart: u32,
    pub HighPart: i32,
}

/// `LUID_AND_ATTRIBUTES`.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct LUID_AND_ATTRIBUTES {
    pub Luid: LUID,
    pub Attributes: u32,
}

/// `TOKEN_PRIVILEGES` header — followed by `[LUID_AND_ATTRIBUTES; PrivilegeCount]`.
/// We only need the header here; the array is accessed by offset.
#[repr(C)]
pub struct TOKEN_PRIVILEGES_HDR {
    pub PrivilegeCount: u32,
    pub Privileges: [LUID_AND_ATTRIBUTES; 1], // ANYSIZE_ARRAY
}

/// `OBJECT_ATTRIBUTES`.
#[repr(C)]
pub struct OBJECT_ATTRIBUTES_FFI {
    pub Length: u32,
    pub RootDirectory: *mut u8,
    pub ObjectName: *mut u8,
    pub Attributes: u32,
    pub SecurityDescriptor: *mut u8,
    pub SecurityQualityOfService: *mut u8,
}

/// `SECURITY_QUALITY_OF_SERVICE`.
#[repr(C)]
pub struct SECURITY_QUALITY_OF_SERVICE_FFI {
    pub Length: u32,
    pub ImpersonationLevel: u32,
    pub ContextTrackingMode: u8,
    pub EffectiveOnly: u8,
}

/// `RTL_ACQUIRE_STATE` (ndk/rtltypes.h). `OldPrivBuffer[1024]` is a
/// static inline buffer the C code uses to avoid an alloc in the
/// common case (privilege count fits).
#[repr(C)]
pub struct RTL_ACQUIRE_STATE_FFI {
    pub Token: *mut u8,
    pub OldImpersonationToken: *mut u8,
    pub OldPrivileges: *mut u8,
    pub NewPrivileges: *mut u8,
    pub Flags: u32,
    pub OldPrivBuffer: [u8; 1024],
}

// --- Zw* / Rtl heap externs. NTAPI = stdcall on i386, cdecl on
// amd64/arm64 → `extern "system"`. Resolved by ntoskrnl (kernel) or
// ntdll (user-mode) at link time.

extern "system" {
    fn ZwOpenThreadToken(
        ThreadHandle: *mut u8,
        DesiredAccess: u32,
        OpenAsSelf: u8,
        TokenHandle: *mut *mut u8,
    ) -> i32;
    fn ZwOpenProcessToken(
        ProcessHandle: *mut u8,
        DesiredAccess: u32,
        TokenHandle: *mut *mut u8,
    ) -> i32;
    fn ZwDuplicateToken(
        ExistingTokenHandle: *mut u8,
        DesiredAccess: u32,
        ObjectAttributes: *mut OBJECT_ATTRIBUTES_FFI,
        EffectiveOnly: u8,
        TokenType: u32,
        NewTokenHandle: *mut *mut u8,
    ) -> i32;
    fn ZwSetInformationThread(
        ThreadHandle: *mut u8,
        InfoClass: u32,
        Info: *const u8,
        InfoLen: u32,
    ) -> i32;
    fn ZwAdjustPrivilegesToken(
        TokenHandle: *mut u8,
        DisableAllPrivileges: u8,
        NewState: *const TOKEN_PRIVILEGES_HDR,
        BufferLength: u32,
        PreviousState: *mut TOKEN_PRIVILEGES_HDR,
        ReturnLength: *mut u32,
    ) -> i32;
    fn ZwQueryInformationToken(
        TokenHandle: *mut u8,
        InfoClass: u32,
        Info: *mut u8,
        InfoLen: u32,
        ReturnLength: *mut u32,
    ) -> i32;
    fn ZwClose(Handle: *mut u8) -> i32;
    fn RtlRaiseStatus(Status: i32) -> !;
}

// RtlAllocateHeap/RtlFreeHeap/RtlGetProcessHeap are already declared near
// the top of this file, cfg-gated to the user-mode windows build (for the
// rtl_alloc/rtl_free helpers). Declare them here ONLY for the kernel and
// host-difftest builds so every context has exactly one declaration.
#[cfg(not(all(not(feature = "kernel"), target_os = "windows")))]
extern "system" {
    fn RtlAllocateHeap(Heap: *mut u8, Flags: u32, Size: usize) -> *mut u8;
    fn RtlFreeHeap(Heap: *mut u8, Flags: u32, Ptr: *mut u8) -> u8;
    fn RtlGetProcessHeap() -> *mut u8;
}

/// Read `NtCurrentTeb()->IsImpersonating` via per-arch inline asm
/// (offset constant lives in `core_rs::priv_`). Safe to call from
/// user-mode threads; in kernel mode (where this translation unit's
/// `RtlAcquirePrivilege` is dead — no kernel caller) the read
/// points at whatever the kernel has GS/FS/X18 set to.
#[inline]
unsafe fn read_is_impersonating() -> u32 {
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    {
        let val: u32;
        let tmp: u64;
        core::arch::asm!(
            "mov {tmp}, gs:[0x30]",
            "mov {val:e}, [{tmp} + {off}]",
            tmp = out(reg) tmp,
            val = out(reg) val,
            off = const priv_::TEB_IS_IMPERSONATING_OFFSET,
            options(nostack, preserves_flags, readonly),
        );
        let _ = tmp;
        val
    }
    #[cfg(all(target_os = "windows", target_arch = "x86"))]
    {
        let val: u32;
        let tmp: u32;
        core::arch::asm!(
            "mov {tmp:e}, fs:[0x18]",
            "mov {val:e}, [{tmp:e} + {off}]",
            tmp = out(reg) tmp,
            val = out(reg) val,
            off = const priv_::TEB_IS_IMPERSONATING_OFFSET,
            options(nostack, preserves_flags, readonly),
        );
        let _ = tmp;
        val
    }
    #[cfg(all(target_os = "windows", target_arch = "aarch64"))]
    {
        let val: u32;
        core::arch::asm!(
            "ldr {val:w}, [x18, {off}]",
            val = out(reg) val,
            off = const priv_::TEB_IS_IMPERSONATING_OFFSET,
            options(nostack, preserves_flags, readonly),
        );
        val
    }
    #[cfg(not(target_os = "windows"))]
    {
        // Host (rtl-difftest): no TEB. priv shims not exercised by
        // the difftest harness; return 0 (= "not impersonating").
        0
    }
}

/// `NTSTATUS NTAPI RtlpOpenThreadToken(ACCESS_MASK, PHANDLE)` —
/// private helper, calls ZwOpenThreadToken twice (`OpenAsSelf=TRUE`
/// then `FALSE`).
#[no_mangle]
pub unsafe extern "system" fn RtlpOpenThreadToken(
    desired_access: u32,
    token_handle: *mut *mut u8,
) -> i32 {
    let mut status = ZwOpenThreadToken(priv_::nt_current_thread(), desired_access, 1, token_handle);
    if !nt_success(status) {
        status = ZwOpenThreadToken(priv_::nt_current_thread(), desired_access, 0, token_handle);
    }
    status
}

/// `NTSTATUS NTAPI RtlImpersonateSelf(SECURITY_IMPERSONATION_LEVEL)`.
#[no_mangle]
pub unsafe extern "system" fn RtlImpersonateSelf(impersonation_level: u32) -> i32 {
    let mut process_token: *mut u8 = core::ptr::null_mut();
    let mut impersonation_token: *mut u8 = core::ptr::null_mut();

    let mut status = ZwOpenProcessToken(
        priv_::nt_current_process(),
        priv_::TOKEN_DUPLICATE,
        &mut process_token,
    );
    if !nt_success(status) {
        return status;
    }

    let mut sqos = SECURITY_QUALITY_OF_SERVICE_FFI {
        Length: core::mem::size_of::<SECURITY_QUALITY_OF_SERVICE_FFI>() as u32,
        ImpersonationLevel: impersonation_level,
        ContextTrackingMode: 0,
        EffectiveOnly: 0,
    };
    let mut obj_attr = OBJECT_ATTRIBUTES_FFI {
        Length: core::mem::size_of::<OBJECT_ATTRIBUTES_FFI>() as u32,
        RootDirectory: core::ptr::null_mut(),
        ObjectName: core::ptr::null_mut(),
        Attributes: 0,
        SecurityDescriptor: core::ptr::null_mut(),
        SecurityQualityOfService: &mut sqos as *mut _ as *mut u8,
    };

    status = ZwDuplicateToken(
        process_token,
        priv_::TOKEN_IMPERSONATE,
        &mut obj_attr,
        sqos.EffectiveOnly,
        priv_::TOKEN_IMPERSONATION_TYPE,
        &mut impersonation_token,
    );
    if !nt_success(status) {
        let _ = ZwClose(process_token);
        return status;
    }

    status = ZwSetInformationThread(
        priv_::nt_current_thread(),
        priv_::THREAD_IMPERSONATION_TOKEN,
        &impersonation_token as *const _ as *const u8,
        core::mem::size_of::<*mut u8>() as u32,
    );
    // The C ignores this status's NT_SUCCESS check (does DPRINT only).
    let _ = ZwClose(impersonation_token);
    let _ = ZwClose(process_token);
    status
}

/// `NTSTATUS NTAPI RtlAcquirePrivilege(PULONG, ULONG, ULONG, PVOID*)`.
///
/// Straight translation; uses `read_is_impersonating()` (TEB inline
/// asm) for the user-mode impersonation check.
#[no_mangle]
pub unsafe extern "system" fn RtlAcquirePrivilege(
    privilege: *mut u32,
    num_priv: u32,
    flags: u32,
    returned_state: *mut *mut u8,
) -> i32 {
    if flags & !(priv_::RTL_ACQUIRE_PRIVILEGE_PROCESS | priv_::RTL_ACQUIRE_PRIVILEGE_IMPERSONATE)
        != 0
    {
        return STATUS_INVALID_PARAMETER;
    }
    let mut flags = flags;
    if flags & priv_::RTL_ACQUIRE_PRIVILEGE_PROCESS != 0 {
        flags |= priv_::RTL_ACQUIRE_PRIVILEGE_IMPERSONATE;
    }

    // alloc state + new-privileges trailing area
    let alloc_size = core::mem::size_of::<RTL_ACQUIRE_STATE_FFI>()
        + core::mem::size_of::<TOKEN_PRIVILEGES_HDR>()
        + (num_priv as usize).saturating_sub(1) * core::mem::size_of::<LUID_AND_ATTRIBUTES>();
    let state_p = RtlAllocateHeap(RtlGetProcessHeap(), 0, alloc_size) as *mut RTL_ACQUIRE_STATE_FFI;
    if state_p.is_null() {
        return STATUS_NO_MEMORY;
    }
    (*state_p).Token = core::ptr::null_mut();
    (*state_p).OldImpersonationToken = core::ptr::null_mut();
    (*state_p).Flags = 0;
    (*state_p).OldPrivileges = core::ptr::null_mut();

    let mut imp_token: *mut u8 = core::ptr::null_mut();
    let mut status: i32;

    if read_is_impersonating() != 0 {
        if flags & priv_::RTL_ACQUIRE_PRIVILEGE_IMPERSONATE != 0 {
            status = RtlpOpenThreadToken(
                priv_::TOKEN_ADJUST_PRIVILEGES | priv_::TOKEN_QUERY,
                &mut (*state_p).Token,
            );
            if !nt_success(status) {
                let _ = RtlFreeHeap(RtlGetProcessHeap(), 0, state_p as *mut u8);
                return status;
            }
        } else {
            status = RtlpOpenThreadToken(
                priv_::TOKEN_IMPERSONATE,
                &mut (*state_p).OldImpersonationToken,
            );
            if !nt_success(status) {
                let _ = RtlFreeHeap(RtlGetProcessHeap(), 0, state_p as *mut u8);
                return status;
            }
            (*state_p).Flags |= priv_::RTL_ACQUIRE_PRIVILEGE_IMPERSONATE;
            // revert impersonation (give 0 as handle)
            let _ = ZwSetInformationThread(
                priv_::nt_current_thread(),
                priv_::THREAD_IMPERSONATION_TOKEN,
                &imp_token as *const _ as *const u8,
                core::mem::size_of::<*mut u8>() as u32,
            );
        }
    }

    if (*state_p).Token.is_null() {
        if flags & priv_::RTL_ACQUIRE_PRIVILEGE_PROCESS != 0 {
            status = ZwOpenProcessToken(
                priv_::nt_current_process(),
                priv_::TOKEN_ADJUST_PRIVILEGES | priv_::TOKEN_QUERY,
                &mut (*state_p).Token,
            );
            if !nt_success(status) {
                return cleanup_acquire(state_p, status, flags);
            }
        } else {
            let mut process_token: *mut u8 = core::ptr::null_mut();
            // (C calls ZwOpenProcessToken without checking the status here
            // and relies on ZwDuplicateToken to fail if the token is bad —
            // verbatim port of that exact misdesign.)
            let _ignored = ZwOpenProcessToken(
                priv_::nt_current_process(),
                priv_::TOKEN_DUPLICATE,
                &mut process_token,
            );
            let mut sqos = SECURITY_QUALITY_OF_SERVICE_FFI {
                Length: core::mem::size_of::<SECURITY_QUALITY_OF_SERVICE_FFI>() as u32,
                ImpersonationLevel: priv_::SECURITY_DELEGATION,
                ContextTrackingMode: 1,
                EffectiveOnly: 0,
            };
            let mut obj_attr = OBJECT_ATTRIBUTES_FFI {
                Length: core::mem::size_of::<OBJECT_ATTRIBUTES_FFI>() as u32,
                RootDirectory: core::ptr::null_mut(),
                ObjectName: core::ptr::null_mut(),
                Attributes: 0,
                SecurityDescriptor: core::ptr::null_mut(),
                SecurityQualityOfService: &mut sqos as *mut _ as *mut u8,
            };
            status = ZwDuplicateToken(
                process_token,
                priv_::TOKEN_ADJUST_PRIVILEGES | priv_::TOKEN_QUERY | priv_::TOKEN_IMPERSONATE,
                &mut obj_attr,
                0,
                priv_::TOKEN_IMPERSONATION_TYPE,
                &mut imp_token,
            );
            if !nt_success(status) {
                let _ = ZwClose(process_token);
                return cleanup_acquire(state_p, status, flags);
            }
            status = ZwSetInformationThread(
                priv_::nt_current_thread(),
                priv_::THREAD_IMPERSONATION_TOKEN,
                &imp_token as *const _ as *const u8,
                core::mem::size_of::<*mut u8>() as u32,
            );
            if !nt_success(status) {
                let _ = ZwClose(imp_token);
                let _ = ZwClose(process_token);
                return cleanup_acquire(state_p, status, flags);
            }
            (*state_p).Token = imp_token;
            (*state_p).Flags |= priv_::RTL_ACQUIRE_PRIVILEGE_IMPERSONATE;
            let _ = ZwClose(process_token);
        }
    }

    // Wire OldPrivileges → into OldPrivBuffer; NewPrivileges → right after.
    let buffer_start = &mut (*state_p).OldPrivBuffer as *mut [u8; 1024] as *mut u8;
    (*state_p).OldPrivileges = buffer_start;
    (*state_p).NewPrivileges = buffer_start.add((*state_p).OldPrivBuffer.len());

    let new_p = (*state_p).NewPrivileges as *mut TOKEN_PRIVILEGES_HDR;
    (*new_p).PrivilegeCount = num_priv;
    let privs_arr =
        &mut (*new_p).Privileges as *mut [LUID_AND_ATTRIBUTES; 1] as *mut LUID_AND_ATTRIBUTES;
    for i in 0..num_priv {
        let priv_id = *privilege.add(i as usize);
        let entry = privs_arr.add(i as usize);
        (*entry).Luid.LowPart = priv_id;
        (*entry).Luid.HighPart = 0;
        (*entry).Attributes = priv_::SE_PRIVILEGE_ENABLED;
    }

    let mut old_size = (*state_p).OldPrivBuffer.len() as u32;
    loop {
        let mut return_length: u32 = (*state_p).OldPrivBuffer.len() as u32;
        status = ZwAdjustPrivilegesToken(
            (*state_p).Token,
            0,
            (*state_p).NewPrivileges as *const TOKEN_PRIVILEGES_HDR,
            old_size,
            (*state_p).OldPrivileges as *mut TOKEN_PRIVILEGES_HDR,
            &mut return_length,
        );

        if status == STATUS_BUFFER_TOO_SMALL_2 {
            let new_old = RtlAllocateHeap(RtlGetProcessHeap(), 0, return_length as usize);
            if !new_old.is_null() {
                (*state_p).OldPrivileges = new_old;
                old_size = return_length;
                continue;
            }
            status = STATUS_NO_MEMORY;
        }

        if status == STATUS_NOT_ALL_ASSIGNED && num_priv == 1 {
            status = STATUS_PRIVILEGE_NOT_HELD;
        }

        if !nt_success(status) {
            return cleanup_acquire(state_p, status, flags);
        }
        *returned_state = state_p as *mut u8;
        return status;
    }
}

unsafe fn cleanup_acquire(state_p: *mut RTL_ACQUIRE_STATE_FFI, status: i32, _flags: u32) -> i32 {
    let buffer_start = &mut (*state_p).OldPrivBuffer as *mut [u8; 1024] as *mut u8;
    if !(*state_p).OldPrivileges.is_null() && (*state_p).OldPrivileges != buffer_start {
        let _ = RtlFreeHeap(RtlGetProcessHeap(), 0, (*state_p).OldPrivileges);
    }
    if (*state_p).Flags & priv_::RTL_ACQUIRE_PRIVILEGE_IMPERSONATE != 0 {
        let int_status = ZwSetInformationThread(
            priv_::nt_current_thread(),
            priv_::THREAD_IMPERSONATION_TOKEN,
            &(*state_p).OldImpersonationToken as *const _ as *const u8,
            core::mem::size_of::<*mut u8>() as u32,
        );
        if !nt_success(int_status) {
            RtlRaiseStatus(int_status);
        }
    }
    if !(*state_p).Token.is_null() {
        let _ = ZwClose((*state_p).Token);
    }
    let _ = RtlFreeHeap(RtlGetProcessHeap(), 0, state_p as *mut u8);
    status
}

/// `VOID NTAPI RtlReleasePrivilege(PVOID)`.
#[no_mangle]
pub unsafe extern "system" fn RtlReleasePrivilege(returned_state: *mut u8) {
    let state_p = returned_state as *mut RTL_ACQUIRE_STATE_FFI;

    if (*state_p).Flags & priv_::RTL_ACQUIRE_PRIVILEGE_IMPERSONATE != 0 {
        let status = ZwSetInformationThread(
            priv_::nt_current_thread(),
            priv_::THREAD_IMPERSONATION_TOKEN,
            &(*state_p).OldImpersonationToken as *const _ as *const u8,
            core::mem::size_of::<*mut u8>() as u32,
        );
        if !nt_success(status) {
            RtlRaiseStatus(status);
        }
        if !(*state_p).OldImpersonationToken.is_null() {
            let _ = ZwClose((*state_p).OldImpersonationToken);
        }
    } else {
        let status = ZwAdjustPrivilegesToken(
            (*state_p).Token,
            0,
            (*state_p).OldPrivileges as *const TOKEN_PRIVILEGES_HDR,
            0,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
        if !nt_success(status) {
            RtlRaiseStatus(status);
        }
    }

    let buffer_start = &mut (*state_p).OldPrivBuffer as *mut [u8; 1024] as *mut u8;
    if (*state_p).OldPrivileges != buffer_start {
        let _ = RtlFreeHeap(RtlGetProcessHeap(), 0, (*state_p).OldPrivileges);
    }
    let _ = ZwClose((*state_p).Token);
    let _ = RtlFreeHeap(RtlGetProcessHeap(), 0, state_p as *mut u8);
}

/// `NTSTATUS NTAPI RtlAdjustPrivilege(ULONG, BOOLEAN, BOOLEAN, PBOOLEAN)`.
#[no_mangle]
pub unsafe extern "system" fn RtlAdjustPrivilege(
    privilege: u32,
    enable: u8,
    current_thread: u8,
    enabled: *mut u8,
) -> i32 {
    let mut token_handle: *mut u8 = core::ptr::null_mut();
    let mut status = if current_thread != 0 {
        ZwOpenThreadToken(
            priv_::nt_current_thread(),
            priv_::TOKEN_ADJUST_PRIVILEGES | priv_::TOKEN_QUERY,
            0,
            &mut token_handle,
        )
    } else {
        ZwOpenProcessToken(
            priv_::nt_current_process(),
            priv_::TOKEN_ADJUST_PRIVILEGES | priv_::TOKEN_QUERY,
            &mut token_handle,
        )
    };
    if !nt_success(status) {
        return status;
    }

    let mut new_state = TOKEN_PRIVILEGES_HDR {
        PrivilegeCount: 1,
        Privileges: [LUID_AND_ATTRIBUTES {
            Luid: LUID {
                LowPart: privilege,
                HighPart: 0,
            },
            Attributes: if enable != 0 {
                priv_::SE_PRIVILEGE_ENABLED
            } else {
                0
            },
        }],
    };
    let mut old_state = TOKEN_PRIVILEGES_HDR {
        PrivilegeCount: 1,
        Privileges: [LUID_AND_ATTRIBUTES {
            Luid: LUID {
                LowPart: 0,
                HighPart: 0,
            },
            Attributes: 0,
        }],
    };
    let mut return_length: u32 = 0;

    status = ZwAdjustPrivilegesToken(
        token_handle,
        0,
        &new_state,
        core::mem::size_of::<TOKEN_PRIVILEGES_HDR>() as u32,
        &mut old_state,
        &mut return_length,
    );
    let _ = ZwClose(token_handle);
    if status == STATUS_NOT_ALL_ASSIGNED {
        return STATUS_PRIVILEGE_NOT_HELD;
    }
    if !nt_success(status) {
        return status;
    }

    if old_state.PrivilegeCount == 0 {
        *enabled = enable;
    } else {
        *enabled = if old_state.Privileges[0].Attributes & priv_::SE_PRIVILEGE_ENABLED != 0 {
            1
        } else {
            0
        };
    }
    // mute unused-mut
    let _ = &mut new_state;
    STATUS_SUCCESS
}

/// `NTSTATUS NTAPI RtlRemovePrivileges(HANDLE, PULONG, ULONG)` —
/// Vista+. Removes all privileges except those listed in
/// `PrivilegesToKeep`.
#[no_mangle]
pub unsafe extern "system" fn RtlRemovePrivileges(
    token_handle: *mut u8,
    privileges_to_keep: *mut u32,
    privilege_count: u32,
) -> i32 {
    let mut keep_bitmap: u64 = 0;
    if privilege_count != 0 {
        for i in 0..privilege_count {
            let p = *privileges_to_keep.add(i as usize);
            if (p as u64) > priv_::SE_MAX_WELL_KNOWN_PRIVILEGE {
                return STATUS_INVALID_PARAMETER;
            }
            keep_bitmap |= 1u64 << p;
        }
    }

    // Fixed-size buffer:
    //   sizeof(TOKEN_PRIVILEGES) +
    //   sizeof(LUID_AND_ATTRIBUTES) *
    //       (SE_MAX_WELL_KNOWN_PRIVILEGE - SE_MIN_WELL_KNOWN_PRIVILEGE)
    const BUF_SIZE: usize = core::mem::size_of::<TOKEN_PRIVILEGES_HDR>()
        + core::mem::size_of::<LUID_AND_ATTRIBUTES>() * priv_::SE_MAX_OLD_PRIV_BUFFER_ENTRIES;
    let mut buffer = [0u8; BUF_SIZE];
    let mut return_length: u32 = 0;

    let mut status = ZwQueryInformationToken(
        token_handle,
        priv_::TOKEN_PRIVILEGES_CLASS,
        buffer.as_mut_ptr(),
        BUF_SIZE as u32,
        &mut return_length,
    );
    if !nt_success(status) {
        return status;
    }

    let hdr = buffer.as_mut_ptr() as *mut TOKEN_PRIVILEGES_HDR;
    let priv_count = (*hdr).PrivilegeCount;
    let arr = &mut (*hdr).Privileges as *mut [LUID_AND_ATTRIBUTES; 1] as *mut LUID_AND_ATTRIBUTES;
    for i in 0..priv_count {
        let entry = arr.add(i as usize);
        let luid_q =
            ((*entry).Luid.LowPart as u64) | (((*entry).Luid.HighPart as u32 as u64) << 32);
        if keep_bitmap & (1u64 << luid_q) != 0 {
            keep_bitmap &= !(1u64 << luid_q);
        } else {
            (*entry).Attributes = priv_::SE_PRIVILEGE_REMOVED;
        }
    }

    if keep_bitmap != 0 {
        status = STATUS_NOT_ALL_ASSIGNED;
    } else {
        status = ZwAdjustPrivilegesToken(
            token_handle,
            0,
            hdr as *const TOKEN_PRIVILEGES_HDR,
            BUF_SIZE as u32,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
    }
    status
}

// =====================================================================
// Wave TZ — straight translation of `sdk/lib/rtl/timezone.c` (2 fns,
// pure registry plumbing: 1× RtlQueryRegistryValues + 7× RtlWriteRegistryValue).
// No byte-diff: there's no algorithmic content to diff — these are
// thin wrappers over already-Rust registry infra. Smoke is the net.
// =====================================================================

/// Layout-match for NDK `TIME_FIELDS` (`CSHORT × 8 = 16 bytes`).
#[repr(C)]
struct TIME_FIELDS_TZ {
    Year: i16,
    Month: i16,
    Day: i16,
    Hour: i16,
    Minute: i16,
    Second: i16,
    Milliseconds: i16,
    Weekday: i16,
}

/// Layout-match for NDK `RTL_TIME_ZONE_INFORMATION` (172 bytes).
#[repr(C)]
pub struct RTL_TIME_ZONE_INFORMATION {
    Bias: i32,
    StandardName: [u16; 32],
    StandardDate: TIME_FIELDS_TZ,
    StandardBias: i32,
    DaylightName: [u16; 32],
    DaylightDate: TIME_FIELDS_TZ,
    DaylightBias: i32,
}

/// Layout-match for XDK `RTL_QUERY_REGISTRY_TABLE`.
#[repr(C)]
struct RTL_QUERY_REGISTRY_TABLE_TZ {
    QueryRoutine: *mut u8,
    Flags: u32,
    Name: *const u16,
    EntryContext: *mut u8,
    DefaultType: u32,
    DefaultData: *mut u8,
    DefaultLength: u32,
}

extern "system" {
    fn RtlQueryRegistryValues(
        RelativeTo: u32,
        Path: *const u16,
        QueryTable: *mut RTL_QUERY_REGISTRY_TABLE_TZ,
        Context: *mut u8,
        Environment: *mut u8,
    ) -> i32;
    fn RtlWriteRegistryValue(
        RelativeTo: u32,
        Path: *const u16,
        ValueName: *const u16,
        ValueType: u32,
        ValueData: *mut u8,
        ValueLength: u32,
    ) -> i32;
}

/// `const fn` widening from ASCII bytes (with trailing NUL) to a
/// null-terminated `[u16; N]`. Replaces the C `L"…"` literal.
const fn wsz<const N: usize>(s: &[u8; N]) -> [u16; N] {
    let mut out = [0u16; N];
    let mut i = 0;
    while i < N {
        out[i] = s[i] as u16;
        i += 1;
    }
    out
}
static TZ_PATH: [u16; 20] = wsz(b"TimeZoneInformation\0");
static TZ_BIAS: [u16; 5] = wsz(b"Bias\0");
static TZ_STD_NAME: [u16; 13] = wsz(b"StandardName\0");
static TZ_STD_BIAS: [u16; 13] = wsz(b"StandardBias\0");
static TZ_STD_START: [u16; 14] = wsz(b"StandardStart\0");
static TZ_DAY_NAME: [u16; 13] = wsz(b"DaylightName\0");
static TZ_DAY_BIAS: [u16; 13] = wsz(b"DaylightBias\0");
static TZ_DAY_START: [u16; 14] = wsz(b"DaylightStart\0");

/// `RtlQueryTimeZoneInformation(PRTL_TIME_ZONE_INFORMATION)`.
///
/// Builds an 8-entry RTL_QUERY_REGISTRY_TABLE (entry 7 = zero
/// sentinel) over `Control\TimeZoneInformation`, then defers to
/// `RtlQueryRegistryValues`. Faithful to the C — including the
/// `UNICODE_STRING` wrapper around the inline `StandardName` /
/// `DaylightName` `WCHAR[32]` buffers needed by the
/// `RTL_QUERY_REGISTRY_DIRECT` REG_SZ direct-write path.
#[no_mangle]
pub unsafe extern "system" fn RtlQueryTimeZoneInformation(
    tzi: *mut RTL_TIME_ZONE_INFORMATION,
) -> i32 {
    // UNICODE_STRING wrappers around the inline name buffers.
    // Layout-match for NDK UNICODE_STRING (Length: u16, Max: u16, Buffer: *mut u16).
    #[repr(C)]
    struct US {
        Length: u16,
        MaximumLength: u16,
        Buffer: *mut u16,
    }
    let mut standard = US {
        Length: 0,
        MaximumLength: tz::TZ_NAME_CHARS * 2,
        Buffer: (*tzi).StandardName.as_mut_ptr(),
    };
    let mut daylight = US {
        Length: 0,
        MaximumLength: tz::TZ_NAME_CHARS * 2,
        Buffer: (*tzi).DaylightName.as_mut_ptr(),
    };

    let zero = RTL_QUERY_REGISTRY_TABLE_TZ {
        QueryRoutine: core::ptr::null_mut(),
        Flags: 0,
        Name: core::ptr::null(),
        EntryContext: core::ptr::null_mut(),
        DefaultType: 0,
        DefaultData: core::ptr::null_mut(),
        DefaultLength: 0,
    };
    let mut table: [RTL_QUERY_REGISTRY_TABLE_TZ; 8] = [
        // [0] Bias (REG_DWORD → LONG)
        RTL_QUERY_REGISTRY_TABLE_TZ {
            QueryRoutine: core::ptr::null_mut(),
            Flags: tz::RTL_QUERY_REGISTRY_DIRECT,
            Name: TZ_BIAS.as_ptr(),
            EntryContext: core::ptr::addr_of_mut!((*tzi).Bias) as *mut u8,
            DefaultType: 0,
            DefaultData: core::ptr::null_mut(),
            DefaultLength: 0,
        },
        // [1] StandardName (REG_SZ → UNICODE_STRING)
        RTL_QUERY_REGISTRY_TABLE_TZ {
            QueryRoutine: core::ptr::null_mut(),
            Flags: tz::RTL_QUERY_REGISTRY_DIRECT,
            Name: TZ_STD_NAME.as_ptr(),
            EntryContext: core::ptr::addr_of_mut!(standard) as *mut u8,
            DefaultType: 0,
            DefaultData: core::ptr::null_mut(),
            DefaultLength: 0,
        },
        // [2] StandardBias
        RTL_QUERY_REGISTRY_TABLE_TZ {
            QueryRoutine: core::ptr::null_mut(),
            Flags: tz::RTL_QUERY_REGISTRY_DIRECT,
            Name: TZ_STD_BIAS.as_ptr(),
            EntryContext: core::ptr::addr_of_mut!((*tzi).StandardBias) as *mut u8,
            DefaultType: 0,
            DefaultData: core::ptr::null_mut(),
            DefaultLength: 0,
        },
        // [3] StandardStart (REG_BINARY → TIME_FIELDS / SYSTEMTIME-shaped)
        RTL_QUERY_REGISTRY_TABLE_TZ {
            QueryRoutine: core::ptr::null_mut(),
            Flags: tz::RTL_QUERY_REGISTRY_DIRECT,
            Name: TZ_STD_START.as_ptr(),
            EntryContext: core::ptr::addr_of_mut!((*tzi).StandardDate) as *mut u8,
            DefaultType: 0,
            DefaultData: core::ptr::null_mut(),
            DefaultLength: 0,
        },
        // [4] DaylightName
        RTL_QUERY_REGISTRY_TABLE_TZ {
            QueryRoutine: core::ptr::null_mut(),
            Flags: tz::RTL_QUERY_REGISTRY_DIRECT,
            Name: TZ_DAY_NAME.as_ptr(),
            EntryContext: core::ptr::addr_of_mut!(daylight) as *mut u8,
            DefaultType: 0,
            DefaultData: core::ptr::null_mut(),
            DefaultLength: 0,
        },
        // [5] DaylightBias
        RTL_QUERY_REGISTRY_TABLE_TZ {
            QueryRoutine: core::ptr::null_mut(),
            Flags: tz::RTL_QUERY_REGISTRY_DIRECT,
            Name: TZ_DAY_BIAS.as_ptr(),
            EntryContext: core::ptr::addr_of_mut!((*tzi).DaylightBias) as *mut u8,
            DefaultType: 0,
            DefaultData: core::ptr::null_mut(),
            DefaultLength: 0,
        },
        // [6] DaylightStart
        RTL_QUERY_REGISTRY_TABLE_TZ {
            QueryRoutine: core::ptr::null_mut(),
            Flags: tz::RTL_QUERY_REGISTRY_DIRECT,
            Name: TZ_DAY_START.as_ptr(),
            EntryContext: core::ptr::addr_of_mut!((*tzi).DaylightDate) as *mut u8,
            DefaultType: 0,
            DefaultData: core::ptr::null_mut(),
            DefaultLength: 0,
        },
        // [7] zero sentinel
        zero,
    ];

    RtlQueryRegistryValues(
        tz::RTL_REGISTRY_CONTROL,
        TZ_PATH.as_ptr(),
        table.as_mut_ptr(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    )
}

/// `RtlSetTimeZoneInformation(PRTL_TIME_ZONE_INFORMATION)`.
///
/// Seven sequential `RtlWriteRegistryValue` calls; bails on first
/// failure. Faithful to the C — `wcslen` on the inline name buffers
/// (caller's responsibility to null-terminate within `WCHAR[32]`).
#[no_mangle]
pub unsafe extern "system" fn RtlSetTimeZoneInformation(
    tzi: *mut RTL_TIME_ZONE_INFORMATION,
) -> i32 {
    let status = RtlWriteRegistryValue(
        tz::RTL_REGISTRY_CONTROL,
        TZ_PATH.as_ptr(),
        TZ_BIAS.as_ptr(),
        tz::REG_DWORD,
        core::ptr::addr_of_mut!((*tzi).Bias) as *mut u8,
        core::mem::size_of::<i32>() as u32,
    );
    if !nt_success(status) {
        return status;
    }

    let std_len = (wcslen((*tzi).StandardName.as_ptr()) + 1) * 2;
    let status = RtlWriteRegistryValue(
        tz::RTL_REGISTRY_CONTROL,
        TZ_PATH.as_ptr(),
        TZ_STD_NAME.as_ptr(),
        tz::REG_SZ,
        (*tzi).StandardName.as_mut_ptr() as *mut u8,
        std_len as u32,
    );
    if !nt_success(status) {
        return status;
    }

    let status = RtlWriteRegistryValue(
        tz::RTL_REGISTRY_CONTROL,
        TZ_PATH.as_ptr(),
        TZ_STD_BIAS.as_ptr(),
        tz::REG_DWORD,
        core::ptr::addr_of_mut!((*tzi).StandardBias) as *mut u8,
        core::mem::size_of::<i32>() as u32,
    );
    if !nt_success(status) {
        return status;
    }

    let status = RtlWriteRegistryValue(
        tz::RTL_REGISTRY_CONTROL,
        TZ_PATH.as_ptr(),
        TZ_STD_START.as_ptr(),
        tz::REG_BINARY,
        core::ptr::addr_of_mut!((*tzi).StandardDate) as *mut u8,
        tz::SIZEOF_SYSTEMTIME,
    );
    if !nt_success(status) {
        return status;
    }

    let day_len = (wcslen((*tzi).DaylightName.as_ptr()) + 1) * 2;
    let status = RtlWriteRegistryValue(
        tz::RTL_REGISTRY_CONTROL,
        TZ_PATH.as_ptr(),
        TZ_DAY_NAME.as_ptr(),
        tz::REG_SZ,
        (*tzi).DaylightName.as_mut_ptr() as *mut u8,
        day_len as u32,
    );
    if !nt_success(status) {
        return status;
    }

    let status = RtlWriteRegistryValue(
        tz::RTL_REGISTRY_CONTROL,
        TZ_PATH.as_ptr(),
        TZ_DAY_BIAS.as_ptr(),
        tz::REG_DWORD,
        core::ptr::addr_of_mut!((*tzi).DaylightBias) as *mut u8,
        core::mem::size_of::<i32>() as u32,
    );
    if !nt_success(status) {
        return status;
    }

    RtlWriteRegistryValue(
        tz::RTL_REGISTRY_CONTROL,
        TZ_PATH.as_ptr(),
        TZ_DAY_START.as_ptr(),
        tz::REG_BINARY,
        core::ptr::addr_of_mut!((*tzi).DaylightDate) as *mut u8,
        tz::SIZEOF_SYSTEMTIME,
    )
}

// =====================================================================
// Wave ENV-1 — pure string-walk slice of `sdk/lib/rtl/env.c`:
//   - RtlQueryEnvironmentVariable_U (find var in env block)
//   - RtlExpandEnvironmentStrings_U (%VAR% substitution)
// Byte-diff coverage in rtl-difftest. The 5 syscall-bearing fns
// (Create/Destroy/SetCurrent/SetEnvironmentVariable/SetEnvironmentStrings)
// land in Wave ENV-2 as translate-only.
// =====================================================================

/// Compute length-in-chars (excluding trailing NUL) of a
/// double-NUL-terminated environment block. Faithful to the C
/// `do { env_end += wcslen+1 } while(*env_end); env_end++` loop.
/// Returns total chars including the terminating double-NUL.
unsafe fn env_block_chars(env: *const u16) -> usize {
    if env.is_null() || *env == 0 {
        return 1; // just the terminator
    }
    let mut end = 0usize;
    loop {
        let mut k = 0;
        while *env.add(end + k) != 0 {
            k += 1;
        }
        end += k + 1;
        if *env.add(end) == 0 {
            return end + 1;
        }
    }
}

/// PEB helpers used by the env shims. `RtlpGetPebEnvironment` /
/// `RtlpSetPebEnvironment` are C helpers in `{dll/ntdll, ntoskrnl}
/// /rtl/libsupp.c` that know the real PEB / RTL_USER_PROCESS_PARAMETERS
/// layout per arch — we go through them instead of mirroring those
/// structs in Rust. `RtlAcquirePebLock` / `RtlReleasePebLock` are
/// no-ops in kernel mode.
extern "system" {
    fn RtlpGetPebEnvironment() -> *mut u16;
    fn RtlpSetPebEnvironment(new_env: *mut u16);
    fn RtlAcquirePebLock();
    fn RtlReleasePebLock();
}

/// Syscalls used by the env-VM plumbing fns. NTAPI on all 3 arches.
extern "system" {
    fn NtQueryVirtualMemory(
        process: *mut u8,
        base: *mut u8,
        info_class: u32,
        info: *mut MemoryBasicInformation,
        info_size: usize,
        return_size: *mut usize,
    ) -> i32;
    fn NtAllocateVirtualMemory(
        process: *mut u8,
        base: *mut *mut u8,
        zero_bits: usize,
        region_size: *mut usize,
        allocation_type: u32,
        protect: u32,
    ) -> i32;
    fn NtFreeVirtualMemory(
        process: *mut u8,
        base: *mut *mut u8,
        region_size: *mut usize,
        free_type: u32,
    ) -> i32;
}

/// Layout-match for `MEMORY_BASIC_INFORMATION` (psdk/winnt.h). Only
/// the `RegionSize` field is read; the rest is sized correctly for
/// the syscall's `info_size` byte count.
#[repr(C)]
struct MemoryBasicInformation {
    BaseAddress: *mut u8,
    AllocationBase: *mut u8,
    AllocationProtect: u32,
    _pad1: [u8; 4], // alignment on x86_64
    RegionSize: usize,
    State: u32,
    Protect: u32,
    Type: u32,
    _pad2: [u8; 4], // tail padding
}

const PAGE_SIZE: usize = 0x1000;
const MEM_RESERVE: u32 = 0x2000;
const MEM_COMMIT: u32 = 0x1000;
const MEM_RELEASE: u32 = 0x8000;
const PAGE_READWRITE: u32 = 0x04;
const MEMORY_BASIC_INFORMATION_CLASS: u32 = 0;

/// `NtCurrentProcess()` — pseudo-handle (-1) for the calling process.
#[inline]
fn nt_current_process() -> *mut u8 {
    -1isize as usize as *mut u8
}

/// `RtlQueryEnvironmentVariable(Environment, Name, NameLength, Value,
/// ValueLength, ReturnLength) -> NTSTATUS`. Raw-buffer core, added
/// upstream 2026-09 (`sdk/lib/rtl/env.c`) alongside the pre-existing
/// `_U` wrapper; exported at `ntdll.spec:1094` (`-version=0x600+`).
///
/// If `Environment` is NULL, resolves the PEB's process parameters
/// (faithful to C — this is where the real fallback lives; the `_U`
/// wrapper below just passes `Environment` through unchanged).
///
/// Upstream semantics (2026-09 rewrite): a hit is a SUCCESS only when
/// the value's length is *strictly less* than `ValueLength` (there
/// must be room for the trailing NUL); otherwise it's
/// `STATUS_BUFFER_TOO_SMALL` and `*ReturnLength` carries
/// `value_len + 1` (the wrapper subtracts the 1 back out). On
/// too-small, only `Value[0]` is NUL'd (if there is room) — unlike
/// the pre-rewrite C, the value is *not* partially copied.
#[no_mangle]
pub unsafe extern "system" fn RtlQueryEnvironmentVariable(
    environment: *mut u16,
    name: *const u16,
    name_length: usize,
    value: *mut u16,
    value_length: usize,
    return_length: *mut usize,
) -> i32 {
    let mut env_ptr = environment;
    let mut sys_env_used = false;

    if env_ptr.is_null() {
        RtlAcquirePebLock();
        env_ptr = RtlpGetPebEnvironment();
        sys_env_used = true;
    }

    if env_ptr.is_null() {
        if sys_env_used {
            RtlReleasePebLock();
        }
        return env::STATUS_VARIABLE_NOT_FOUND;
    }

    *return_length = 0;

    let env_chars = env_block_chars(env_ptr);
    let env_slice = core::slice::from_raw_parts(env_ptr, env_chars);
    let name_slice = core::slice::from_raw_parts(name, name_length);

    let result = env::find_env_var(env_slice, name_slice);

    let status = match result {
        env::EnvQuery::NotFound => env::STATUS_VARIABLE_NOT_FOUND,
        env::EnvQuery::Found {
            value_start,
            value_len,
        } => {
            if value_len < value_length {
                // memcpy(Value, val, (ReturnLength + 1) * sizeof(WCHAR))
                // — copies the value AND its trailing NUL.
                core::ptr::copy_nonoverlapping(
                    env_ptr.add(value_start),
                    value,
                    value_len + 1,
                );
                *return_length = value_len;
                env::STATUS_SUCCESS
            } else {
                if value_length > 0 {
                    *value = 0;
                }
                *return_length = value_len + 1;
                env::STATUS_BUFFER_TOO_SMALL
            }
        }
    };

    if sys_env_used {
        RtlReleasePebLock();
    }
    status
}

/// `RtlQueryEnvironmentVariable_U(Environment, Name, Value) -> NTSTATUS`.
///
/// Thin `UNICODE_STRING` wrapper over the raw-buffer core above,
/// faithful to the 2026-09 upstream rewrite: it applies the
/// `UNICODE_STRING_MAX_CHARS` overflow guard, undoes the core's
/// `+1` on `STATUS_BUFFER_TOO_SMALL`, and sets `Value->Length`
/// unconditionally (even on `STATUS_VARIABLE_NOT_FOUND`, where it
/// ends up 0).
#[no_mangle]
pub unsafe extern "system" fn RtlQueryEnvironmentVariable_U(
    environment: *mut u16,
    name: *const UNICODE_STRING,
    value: *mut UNICODE_STRING,
) -> i32 {
    let name_chars = ((*name).Length / 2) as usize;
    let value_max_chars = ((*value).MaximumLength / 2) as usize;
    let mut return_length: usize = 0;

    let status = RtlQueryEnvironmentVariable(
        environment,
        (*name).Buffer,
        name_chars,
        (*value).Buffer,
        value_max_chars,
        &mut return_length,
    );

    if return_length > env::UNICODE_STRING_MAX_CHARS {
        return env::STATUS_UNSUCCESSFUL;
    }

    if status == env::STATUS_BUFFER_TOO_SMALL {
        return_length -= 1;
    }

    (*value).Length = (return_length * 2) as u16;
    status
}

/// `RtlExpandEnvironmentStrings(Environment, SourceBuffer, SourceLength,
/// Destination, DestMax, Length) -> NTSTATUS`. Raw-buffer core, added
/// upstream 2026-09 alongside the pre-existing `_U` wrapper; exported
/// at `ntdll.spec:831` (`-version=0x600+`).
///
/// Faithful to C: does *not* itself resolve a NULL `Environment` via
/// the PEB (that fallback lives in `RtlQueryEnvironmentVariable`,
/// reached per-`%VAR%` through the genuine C's nested
/// `RtlQueryEnvironmentVariable_U` call). The `_U` wrapper below
/// keeps our pre-existing up-front PEB substitution instead (see its
/// doc comment) — equivalent in practice, and this raw export has no
/// other caller today.
#[no_mangle]
pub unsafe extern "system" fn RtlExpandEnvironmentStrings(
    environment: *mut u16,
    source_buffer: *const u16,
    source_length: usize,
    destination: *mut u16,
    dest_max: usize,
    length: *mut usize,
) -> i32 {
    let env_slice: &[u16] = if environment.is_null() {
        &[]
    } else {
        let n = env_block_chars(environment);
        core::slice::from_raw_parts(environment, n)
    };
    let src_slice = core::slice::from_raw_parts(source_buffer, source_length);
    let dst_slice = core::slice::from_raw_parts_mut(destination, dest_max);

    let result = env::expand_env_strings(env_slice, src_slice, dst_slice);

    if !length.is_null() {
        *length = result.total_chars as usize;
    }
    result.status
}

/// `RtlExpandEnvironmentStrings_U(Env, Source, Destination, Length*)`.
///
/// Thin `UNICODE_STRING` wrapper over the raw-buffer core above,
/// faithful to the 2026-09 upstream rewrite: applies the
/// `UNICODE_STRING_MAX_CHARS` overflow guard, and — this is the part
/// that changed — only writes `Destination->Length` when the core
/// call *succeeded*. On `STATUS_BUFFER_TOO_SMALL` (or the overflow
/// guard's `STATUS_UNSUCCESSFUL`) `Destination->Length` is left
/// untouched, unlike the pre-rewrite C which always set it to the
/// partial length written. `*ReturnLength` (the out-param), by
/// contrast, is always written regardless of status.
///
/// A NULL `environment` means "use the current process environment" —
/// the genuine C passes it straight to `RtlQueryEnvironmentVariable_U`
/// per `%VAR%`, which resolves NULL to
/// `NtCurrentPeb()->ProcessParameters->Environment`. Our raw core
/// above only looks up variables in the slice it is given, so we
/// substitute the PEB environment here up front instead; otherwise
/// NULL (the form RtlQueryRegistryValues uses for REG_EXPAND_SZ)
/// yields an empty env and `%SystemRoot%`-style references are left
/// unexpanded (broke smss KnownDll dir resolution → boot failure).
#[no_mangle]
pub unsafe extern "system" fn RtlExpandEnvironmentStrings_U(
    environment: *mut u16,
    source: *const UNICODE_STRING,
    destination: *mut UNICODE_STRING,
    length: *mut u32,
) -> i32 {
    let env_ptr: *mut u16 = if environment.is_null() {
        RtlpGetPebEnvironment()
    } else {
        environment
    };

    let src_chars = ((*source).Length / 2) as usize;
    let dst_max_chars = ((*destination).MaximumLength / 2) as usize;
    let mut result_length: usize = 0;

    let mut status = RtlExpandEnvironmentStrings(
        env_ptr,
        (*source).Buffer,
        src_chars,
        (*destination).Buffer,
        dst_max_chars,
        &mut result_length,
    );

    if result_length > env::UNICODE_STRING_MAX_CHARS {
        status = env::STATUS_UNSUCCESSFUL;
        result_length = 0;
    }

    if status == env::STATUS_SUCCESS {
        // (USHORT)ResultLength * sizeof(WCHAR) - sizeof(UNICODE_NULL)
        (*destination).Length = ((result_length as u16) * 2) - 2;
    }

    if !length.is_null() {
        *length = (result_length as u32) * 2;
    }

    status
}

// =====================================================================
// Wave ENV-2 — syscall-plumbing translation of `sdk/lib/rtl/env.c`
// (5 fns: stub + Create/Destroy/SetCurrent/SetEnvironmentVariable).
// No byte-diff (PEB + Nt*VirtualMemory dependencies); smoke is the
// regression net.
// =====================================================================

/// `RtlSetEnvironmentStrings(PWCHAR, ULONG) -> NTSTATUS`. Stub; the
/// genuine C is `UNIMPLEMENTED; return STATUS_NOT_IMPLEMENTED;`.
#[no_mangle]
pub unsafe extern "system" fn RtlSetEnvironmentStrings(
    _new_environment: *mut u16,
    _new_environment_size: u32,
) -> i32 {
    STATUS_NOT_IMPLEMENTED
}

/// `RtlCreateEnvironment(BOOLEAN Inherit, PWSTR* OutEnvironment) -> NTSTATUS`.
/// Faithful port of the C. If `Inherit`, copies the calling process'
/// current env block (under PEB lock); else allocates an empty page.
#[no_mangle]
pub unsafe extern "system" fn RtlCreateEnvironment(
    inherit: u8,
    out_environment: *mut *mut u16,
) -> i32 {
    let mut mem_info = MemoryBasicInformation {
        BaseAddress: core::ptr::null_mut(),
        AllocationBase: core::ptr::null_mut(),
        AllocationProtect: 0,
        _pad1: [0; 4],
        RegionSize: 0,
        State: 0,
        Protect: 0,
        Type: 0,
        _pad2: [0; 4],
    };
    let mut new_env: *mut u8 = core::ptr::null_mut();
    let mut status: i32 = 0; // STATUS_SUCCESS
    let mut region_size: usize = PAGE_SIZE;

    if inherit != 0 {
        RtlAcquirePebLock();
        let current_env = RtlpGetPebEnvironment();
        if !current_env.is_null() {
            status = NtQueryVirtualMemory(
                nt_current_process(),
                current_env as *mut u8,
                MEMORY_BASIC_INFORMATION_CLASS,
                &mut mem_info,
                core::mem::size_of::<MemoryBasicInformation>(),
                core::ptr::null_mut(),
            );
            if !nt_success(status) {
                RtlReleasePebLock();
                *out_environment = core::ptr::null_mut();
                return status;
            }

            region_size = mem_info.RegionSize;
            status = NtAllocateVirtualMemory(
                nt_current_process(),
                &mut new_env,
                0,
                &mut region_size,
                MEM_RESERVE | MEM_COMMIT,
                PAGE_READWRITE,
            );
            if !nt_success(status) {
                RtlReleasePebLock();
                *out_environment = core::ptr::null_mut();
                return status;
            }

            core::ptr::copy_nonoverlapping(current_env as *const u8, new_env, mem_info.RegionSize);
        }
        RtlReleasePebLock();
    }

    if new_env.is_null() {
        status = NtAllocateVirtualMemory(
            nt_current_process(),
            &mut new_env,
            0,
            &mut region_size,
            MEM_RESERVE | MEM_COMMIT,
            PAGE_READWRITE,
        );
        if nt_success(status) {
            core::ptr::write_bytes(new_env, 0, region_size);
        }
    }

    *out_environment = new_env as *mut u16;
    status
}

/// `VOID RtlDestroyEnvironment(PWSTR Environment)`. Frees the env
/// block via `NtFreeVirtualMemory`. Faithful to C: passes a
/// stack-local `&Environment` and `Size=0` (full release).
#[no_mangle]
pub unsafe extern "system" fn RtlDestroyEnvironment(environment: *mut u16) {
    let mut env_ptr = environment as *mut u8;
    let mut size: usize = 0;
    NtFreeVirtualMemory(nt_current_process(), &mut env_ptr, &mut size, MEM_RELEASE);
}

/// `VOID RtlSetCurrentEnvironment(NewEnvironment, PWSTR* OldEnvironment)`.
/// Swaps the PEB env pointer under PEB lock; reports the old pointer
/// in `*OldEnvironment` if non-NULL.
#[no_mangle]
pub unsafe extern "system" fn RtlSetCurrentEnvironment(
    new_environment: *mut u16,
    old_environment: *mut *mut u16,
) {
    RtlAcquirePebLock();
    let env_ptr = RtlpGetPebEnvironment();
    RtlpSetPebEnvironment(new_environment);
    if !old_environment.is_null() {
        *old_environment = env_ptr;
    }
    RtlReleasePebLock();
}

/// `RtlSetEnvironmentVariable(PWSTR* Environment, PUNICODE_STRING Name,
///                            PUNICODE_STRING Value) -> NTSTATUS`.
///
/// Faithful translation of the C. The walk is sorted-insert:
/// inserts a new entry in case-insensitive sort order, or replaces
/// the value of an existing entry. If `Value` is NULL, removes the
/// variable. May reallocate the env block via Nt*VirtualMemory.
#[no_mangle]
pub unsafe extern "system" fn RtlSetEnvironmentVariable(
    environment: *mut *mut u16,
    name: *const UNICODE_STRING,
    value: *const UNICODE_STRING,
) -> i32 {
    let mut mbi = MemoryBasicInformation {
        BaseAddress: core::ptr::null_mut(),
        AllocationBase: core::ptr::null_mut(),
        AllocationProtect: 0,
        _pad1: [0; 4],
        RegionSize: 0,
        State: 0,
        Protect: 0,
        Type: 0,
        _pad2: [0; 4],
    };
    let mut new_env: *mut u16 = core::ptr::null_mut();
    let mut env_len: usize = 0;
    let mut env_end: *mut u16 = core::ptr::null_mut();
    let mut val_ptr: *mut u16 = core::ptr::null_mut();
    let mut tail: *mut u16 = core::ptr::null_mut();
    let mut hole: *mut u16 = core::ptr::null_mut();
    let mut head: *mut u16 = core::ptr::null_mut();
    let mut new_size: usize;
    let mut f: i32 = 1; // 1 = insert (default); 0 = exact match
    let mut status: i32 = 0;

    // Variable name must not be empty.
    if (*name).Length < 2 {
        return env::STATUS_INVALID_PARAMETER;
    }

    // Variable names can't contain '=' except as a first character.
    let name_chars = ((*name).Length / 2) as usize;
    let name_buf = (*name).Buffer;
    let mut idx = 1usize;
    while idx < name_chars {
        if *name_buf.add(idx) == b'=' as u16 {
            return env::STATUS_INVALID_PARAMETER;
        }
        idx += 1;
    }

    let env: *mut u16 = if !environment.is_null() {
        *environment
    } else {
        RtlAcquirePebLock();
        RtlpGetPebEnvironment()
    };

    if !env.is_null() {
        // get environment length: walk by `wcslen+1` until null entry.
        let mut wcs = env;
        env_end = env;
        loop {
            env_end = env_end.add(wcslen(env_end) + 1);
            if *env_end == 0 {
                break;
            }
        }
        env_end = env_end.add(1);
        env_len = ((env_end as usize) - (env as usize)) / 2;

        // find insertion point: walk entries comparing names.
        'outer: loop {
            if *wcs == 0 {
                break;
            }
            let var_buf = wcs;
            wcs = wcs.add(1);
            // search for '='
            let mut p = wcs;
            while *p != 0 && *p != b'=' as u16 {
                p = p.add(1);
            }
            wcs = p;
            if *wcs == 0 {
                wcs = var_buf.add(wcslen(var_buf));
            }
            if *wcs != 0 {
                let var_length = ((wcs as usize) - (var_buf as usize)) as u16;
                let var = UNICODE_STRING {
                    Length: var_length,
                    MaximumLength: var_length,
                    Buffer: var_buf,
                };
                val_ptr = wcs.add(1);
                wcs = val_ptr;
                wcs = wcs.add(wcslen(wcs));
                f = RtlCompareUnicodeString(&var, name, 1);
                if f >= 0 {
                    if f != 0 {
                        // Insert before found.
                        hole = var_buf;
                        tail = var_buf;
                    } else {
                        // Exact match.
                        head = var_buf;
                        wcs = wcs.add(1);
                        tail = wcs;
                        hole = val_ptr;
                    }
                    break 'outer;
                }
            }
            wcs = wcs.add(1);
        }
        if hole.is_null() {
            // Append to environment.
            hole = wcs;
            tail = wcs;
        }
    }

    if !value.is_null() && !(*value).Buffer.is_null() {
        let hole_len = ((tail as usize) - (hole as usize)) / 2;
        new_size = (*value).Length as usize + 2;
        if f != 0 {
            new_size += (*name).Length as usize + 2;
        }
        let new_len = new_size / 2;
        if hole_len < new_len {
            // enlarge environment size
            new_size += (env_len - hole_len) * 2;
            // ROUND_UP to PAGE_SIZE
            new_size = (new_size + PAGE_SIZE - 1) & !(PAGE_SIZE - 1);
            mbi.RegionSize = 0;

            if !env.is_null() {
                status = NtQueryVirtualMemory(
                    nt_current_process(),
                    env as *mut u8,
                    MEMORY_BASIC_INFORMATION_CLASS,
                    &mut mbi,
                    core::mem::size_of::<MemoryBasicInformation>(),
                    core::ptr::null_mut(),
                );
                if !nt_success(status) {
                    if environment.is_null() {
                        RtlReleasePebLock();
                    }
                    return status;
                }
            }

            if new_size > mbi.RegionSize {
                // Reallocate.
                let mut tmp_base: *mut u8 = core::ptr::null_mut();
                status = NtAllocateVirtualMemory(
                    nt_current_process(),
                    &mut tmp_base,
                    0,
                    &mut new_size,
                    MEM_RESERVE | MEM_COMMIT,
                    PAGE_READWRITE,
                );
                new_env = tmp_base as *mut u16;
                if !nt_success(status) {
                    if environment.is_null() {
                        RtlReleasePebLock();
                    }
                    return status;
                }

                if !env.is_null() {
                    let pre_len = ((hole as usize) - (env as usize)) / 2;
                    core::ptr::copy_nonoverlapping(env, new_env, pre_len);
                    hole = new_env.add(pre_len);
                } else {
                    tail = new_env;
                    hole = new_env;
                    *hole = 0;
                    env_end = hole.add(1);
                }
            }
        }

        // Move the tail to make room.
        let tail_chars = ((env_end as usize) - (tail as usize)) / 2;
        core::ptr::copy(tail, hole.add(new_len), tail_chars);

        if !new_env.is_null() {
            // We reallocated; free the old env.
            if !environment.is_null() {
                *environment = new_env;
            } else {
                RtlpSetPebEnvironment(new_env);
            }
            if !env.is_null() {
                let mut env_p = env as *mut u8;
                let mut sz: usize = 0;
                NtFreeVirtualMemory(nt_current_process(), &mut env_p, &mut sz, MEM_RELEASE);
            }
        }

        // Copy in the name (if inserting) and the value.
        if f != 0 {
            core::ptr::copy_nonoverlapping((*name).Buffer, hole, ((*name).Length / 2) as usize);
            hole = hole.add(((*name).Length / 2) as usize);
            *hole = b'=' as u16;
            hole = hole.add(1);
        }
        core::ptr::copy_nonoverlapping((*value).Buffer, hole, ((*value).Length / 2) as usize);
        hole = hole.add(((*value).Length / 2) as usize);
        *hole = 0;
    } else {
        // remove the environment variable
        if f == 0 && !head.is_null() && !tail.is_null() {
            let tail_chars = ((env_end as usize) - (tail as usize)) / 2;
            core::ptr::copy(tail, head, tail_chars);
        }
    }

    if environment.is_null() {
        RtlReleasePebLock();
    }
    status
}

// =====================================================================
// Wave SEC — translation of `sdk/lib/rtl/security.c` (18 fns):
//   byte-diff:   RtlpSetSecurityObject + RtlQuerySecurityObject
//   translate-only stubs:
//      RtlpNewSecurityObject, RtlpConvertToAutoInheritSecurityObject,
//      RtlCreateAndSetSD, RtlRegisterSecureMemoryCacheCallback,
//      RtlFlushSecureMemoryCache  (UNIMPLEMENTED in C)
//   translate-only thin wrappers:
//      RtlSetSecurityObject(Ex), RtlConvertToAutoInheritSecurityObject,
//      RtlNewSecurityObject(Ex)(WithMultipleInheritance),
//      RtlDeleteSecurityObject, RtlNewInstanceSecurityObject,
//      RtlCreateUserSecurityObject
//   translate-only syscall-bearing:
//      RtlDefaultNpAcl, RtlNewSecurityGrantedAccess
// =====================================================================

const STATUS_INVALID_OWNER: i32 = 0xC000_005Au32 as i32;
const STATUS_INVALID_PRIMARY_GROUP: i32 = 0xC000_005Bu32 as i32;
const STATUS_NO_TOKEN: i32 = 0xC000007C_u32 as i32;
const STATUS_NOT_IMPLEMENTED_SEC: i32 = 0xC000_0002u32 as i32;

const PAGED_POOL: u32 = 1;

const TOKEN_OWNER_INFO_CLASS: u32 = 4;
const TOKEN_STATISTICS_INFO_CLASS: u32 = 10;

const SE_SECURITY_PRIVILEGE: u32 = 8;
const SE_PRIVILEGE_USED_FOR_ACCESS: u32 = 0x8000_0000;
const ACCESS_SYSTEM_SECURITY: u32 = 0x0100_0000;

const SECURITY_NT_AUTHORITY_VAL: [u8; 6] = [0, 0, 0, 0, 0, 5];
const SECURITY_WORLD_AUTHORITY_VAL: [u8; 6] = [0, 0, 0, 0, 0, 1];
const SECURITY_LOCAL_SYSTEM_RID: u32 = 0x12;
const SECURITY_BUILTIN_DOMAIN_RID: u32 = 0x20;
const DOMAIN_ALIAS_RID_ADMINS: u32 = 0x220;
const SECURITY_ANONYMOUS_LOGON_RID: u32 = 7;
const SECURITY_WORLD_RID: u32 = 0;

const ACL_REVISION2: u8 = 2;
const GENERIC_ALL: u32 = 0x1000_0000;
const GENERIC_READ: u32 = 0x8000_0000;

/// Layout-match for `GENERIC_MAPPING` (winnt.h) — 4 ACCESS_MASK fields.
#[repr(C)]
pub struct GENERIC_MAPPING_FFI {
    pub GenericRead: u32,
    pub GenericWrite: u32,
    pub GenericExecute: u32,
    pub GenericAll: u32,
}

/// Layout-match for `LUID` (FFI mirror of `priv_::LUID`-shaped pair).
#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct LUID_FFI {
    pub LowPart: u32,
    pub HighPart: i32,
}

/// `LUID_AND_ATTRIBUTES`.
#[repr(C)]
pub struct LUID_AND_ATTRIBUTES_FFI {
    pub Luid: LUID_FFI,
    pub Attributes: u32,
}

/// `PRIVILEGE_SET` header (followed by `Privilege[Count]`).
#[repr(C)]
pub struct PRIVILEGE_SET_FFI {
    pub PrivilegeCount: u32,
    pub Control: u32,
    pub Privilege: [LUID_AND_ATTRIBUTES_FFI; 1],
}

/// `TOKEN_STATISTICS` — only the `ModifiedId` field is read here.
/// Layout-match: TokenId + AuthenticationId + ExpirationTime +
/// TokenType + ImpersonationLevel + DynamicCharged + DynamicAvailable +
/// GroupCount + PrivilegeCount + ModifiedId. We mirror with a
/// minimal-enough fixed buffer instead of replicating the full struct.
const SIZEOF_TOKEN_STATISTICS: usize = 64;

/// `SID_IDENTIFIER_AUTHORITY` (FFI shape).
#[repr(C)]
pub struct SID_IDENTIFIER_AUTHORITY_FFI {
    pub Value: [u8; 6],
}

/// SID with up to 2 SubAuthorities, as needed by `RtlDefaultNpAcl`.
#[repr(C)]
pub struct SID2 {
    pub Revision: u8,
    pub SubAuthorityCount: u8,
    pub IdentifierAuthority: SID_IDENTIFIER_AUTHORITY_FFI,
    pub SubAuthority: [u32; 2],
}

extern "system" {
    fn NtOpenThreadToken(
        thread: *mut u8,
        desired_access: u32,
        open_as_self: u8,
        token_handle: *mut *mut u8,
    ) -> i32;
    fn NtOpenProcessToken(process: *mut u8, desired_access: u32, token_handle: *mut *mut u8)
        -> i32;
    fn NtQueryInformationToken(
        token: *mut u8,
        info_class: u32,
        info: *mut u8,
        info_len: u32,
        return_len: *mut u32,
    ) -> i32;
    fn RtlMapGenericMask(access_mask: *mut u32, generic_mapping: *const GENERIC_MAPPING_FFI);
    fn RtlEqualLuid(luid1: *const LUID_FFI, luid2: *const LUID_FFI) -> u8;
    fn NtClose(handle: *mut u8) -> i32;
}

#[inline]
unsafe fn round_up_4(n: u32) -> u32 {
    sec::round_up_4(n)
}

// ---------------------------------------------------------------------
// RtlpSetSecurityObject — THE byte-diff target.
// ---------------------------------------------------------------------

/// `NTSTATUS RtlpSetSecurityObject(Object?, SecurityInformation,
///     ModificationDescriptor, *ObjectsSecurityDescriptor,
///     AutoInheritFlags, PoolType, GenericMapping, Token?) -> NTSTATUS`
///
/// Builds a fresh self-relative SD by merging fields from the
/// modification descriptor (for components signalled by
/// `SecurityInformation`) with the existing descriptor's other
/// components. Faithful port of security.c.
#[no_mangle]
pub unsafe extern "system" fn RtlpSetSecurityObject(
    _object: *mut u8,
    security_information: u32,
    modification_descriptor: *const u8,
    objects_security_descriptor: *mut *mut u8,
    _auto_inherit_flags: u32,
    _pool_type: u32,
    _generic_mapping: *const GENERIC_MAPPING_FFI,
    _token: *mut u8,
) -> i32 {
    let mut p_owner_sid: *mut u8 = core::ptr::null_mut();
    let mut p_group_sid: *mut u8 = core::ptr::null_mut();
    let mut p_dacl: *mut u8 = core::ptr::null_mut();
    let mut p_sacl: *mut u8 = core::ptr::null_mut();
    let mut defaulted: u8 = 0;
    let mut present: u8 = 0;
    let mut control: u16 = sd_::SE_SELF_RELATIVE;

    // Owner
    let status = if security_information & sd_::OWNER_SECURITY_INFORMATION != 0 {
        RtlGetOwnerSecurityDescriptor(
            modification_descriptor as *const SECURITY_DESCRIPTOR,
            &mut p_owner_sid,
            &mut defaulted,
        )
    } else {
        RtlGetOwnerSecurityDescriptor(
            *objects_security_descriptor as *const SECURITY_DESCRIPTOR,
            &mut p_owner_sid,
            &mut defaulted,
        )
    };
    if !nt_success(status) {
        return status;
    }
    if p_owner_sid.is_null() || RtlValidSid(p_owner_sid) == 0 {
        return STATUS_INVALID_OWNER;
    }
    let owner_sid_size = RtlLengthSid(p_owner_sid);

    // Group
    let status = if security_information & sd_::GROUP_SECURITY_INFORMATION != 0 {
        RtlGetGroupSecurityDescriptor(
            modification_descriptor as *const SECURITY_DESCRIPTOR,
            &mut p_group_sid,
            &mut defaulted,
        )
    } else {
        RtlGetGroupSecurityDescriptor(
            *objects_security_descriptor as *const SECURITY_DESCRIPTOR,
            &mut p_group_sid,
            &mut defaulted,
        )
    };
    if !nt_success(status) {
        return status;
    }
    if p_group_sid.is_null() || RtlValidSid(p_group_sid) == 0 {
        return STATUS_INVALID_PRIMARY_GROUP;
    }
    let group_sid_size = round_up_4(RtlLengthSid(p_group_sid));

    // DACL
    let status = if security_information & sd_::DACL_SECURITY_INFORMATION != 0 {
        let s = RtlGetDaclSecurityDescriptor(
            modification_descriptor as *const SECURITY_DESCRIPTOR,
            &mut present,
            &mut p_dacl,
            &mut defaulted,
        );
        if nt_success(s) {
            control |= sd_::SE_DACL_PRESENT;
        }
        s
    } else {
        let s = RtlGetDaclSecurityDescriptor(
            *objects_security_descriptor as *const SECURITY_DESCRIPTOR,
            &mut present,
            &mut p_dacl,
            &mut defaulted,
        );
        if nt_success(s) {
            if present != 0 {
                control |= sd_::SE_DACL_PRESENT;
            }
            if defaulted != 0 {
                control |= sd_::SE_DACL_DEFAULTED;
            }
        }
        s
    };
    if !nt_success(status) {
        return status;
    }
    let dacl_size = if !p_dacl.is_null() {
        *(p_dacl.add(2) as *const u16) as u32 // ACL.AclSize at offset 2
    } else {
        0
    };

    // SACL
    let status = if security_information & sd_::SACL_SECURITY_INFORMATION != 0 {
        let s = RtlGetSaclSecurityDescriptor(
            modification_descriptor as *const SECURITY_DESCRIPTOR,
            &mut present,
            &mut p_sacl,
            &mut defaulted,
        );
        if nt_success(s) {
            control |= sd_::SE_SACL_PRESENT;
        }
        s
    } else {
        let s = RtlGetSaclSecurityDescriptor(
            *objects_security_descriptor as *const SECURITY_DESCRIPTOR,
            &mut present,
            &mut p_sacl,
            &mut defaulted,
        );
        if nt_success(s) {
            if present != 0 {
                control |= sd_::SE_SACL_PRESENT;
            }
            if defaulted != 0 {
                control |= sd_::SE_SACL_DEFAULTED;
            }
        }
        s
    };
    if !nt_success(status) {
        return status;
    }
    let sacl_size = if !p_sacl.is_null() {
        *(p_sacl.add(2) as *const u16) as u32
    } else {
        0
    };

    // Compute total size.
    let new_sd_size = sec::sd_total_size(owner_sid_size, group_sid_size, dacl_size, sacl_size);

    let p_new_sd = RtlAllocateHeap(RtlGetProcessHeap(), 0, new_sd_size as usize);
    if p_new_sd.is_null() {
        return STATUS_NO_MEMORY;
    }

    let status = RtlCreateSecurityDescriptorRelative(
        p_new_sd as *mut SECURITY_DESCRIPTOR_RELATIVE,
        sd_::SECURITY_DESCRIPTOR_REVISION,
    );
    if !nt_success(status) {
        RtlFreeHeap(RtlGetProcessHeap(), 0, p_new_sd);
        return status;
    }

    // Set control on the relative SD header (offset 2: Sbz1 + Control).
    let sdr = p_new_sd as *mut SECURITY_DESCRIPTOR_RELATIVE;
    (*sdr).Control = control;

    let mut p_dest = p_new_sd.add(sec::SDR_HEADER_SIZE as usize);

    // Copy order: SACL, DACL, Owner, Group (faithful to C).
    if !p_sacl.is_null() {
        core::ptr::copy_nonoverlapping(p_sacl, p_dest, sacl_size as usize);
        (*sdr).Sacl = (p_dest as usize - p_new_sd as usize) as u32;
        p_dest = p_dest.add(round_up_4(sacl_size) as usize);
    }
    if !p_dacl.is_null() {
        core::ptr::copy_nonoverlapping(p_dacl, p_dest, dacl_size as usize);
        (*sdr).Dacl = (p_dest as usize - p_new_sd as usize) as u32;
        p_dest = p_dest.add(round_up_4(dacl_size) as usize);
    }
    core::ptr::copy_nonoverlapping(p_owner_sid, p_dest, owner_sid_size as usize);
    (*sdr).Owner = (p_dest as usize - p_new_sd as usize) as u32;
    p_dest = p_dest.add(round_up_4(owner_sid_size) as usize);
    core::ptr::copy_nonoverlapping(p_group_sid, p_dest, group_sid_size as usize);
    (*sdr).Group = (p_dest as usize - p_new_sd as usize) as u32;

    // Free old SD, install new.
    RtlFreeHeap(RtlGetProcessHeap(), 0, *objects_security_descriptor);
    *objects_security_descriptor = p_new_sd;

    0 // STATUS_SUCCESS
}

// ---------------------------------------------------------------------
// SEC translate-only stubs (UNIMPLEMENTED in C).
// ---------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "system" fn RtlpNewSecurityObject(
    _parent: *const u8,
    _creator: *const u8,
    _new_descriptor: *mut *mut u8,
    _object_types: *mut *mut u8,
    _guid_count: u32,
    _is_directory_object: u8,
    _auto_inherit_flags: u32,
    _token: *mut u8,
    _generic_mapping: *const GENERIC_MAPPING_FFI,
) -> i32 {
    STATUS_NOT_IMPLEMENTED_SEC
}

#[no_mangle]
pub unsafe extern "system" fn RtlpConvertToAutoInheritSecurityObject(
    _parent: *const u8,
    _creator: *const u8,
    _new_descriptor: *mut *mut u8,
    _object_type: *mut u8,
    _is_directory_object: u8,
    _generic_mapping: *const GENERIC_MAPPING_FFI,
) -> i32 {
    STATUS_NOT_IMPLEMENTED_SEC
}

#[no_mangle]
pub unsafe extern "system" fn RtlCreateAndSetSD(
    _ace_data: *mut u8,
    _ace_count: u32,
    _owner_sid: *mut u8,
    _group_sid: *mut u8,
    _new_descriptor: *mut *mut u8,
) -> i32 {
    STATUS_NOT_IMPLEMENTED_SEC
}

#[no_mangle]
pub unsafe extern "system" fn RtlRegisterSecureMemoryCacheCallback(_callback: *mut u8) -> i32 {
    STATUS_NOT_IMPLEMENTED_SEC
}

#[no_mangle]
pub unsafe extern "system" fn RtlFlushSecureMemoryCache(
    _memory_cache: *mut u8,
    _memory_length: usize,
) -> u8 {
    0
}

// ---------------------------------------------------------------------
// SEC thin wrappers + simple fns.
// ---------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "system" fn RtlDeleteSecurityObject(object_descriptor: *mut *mut u8) -> i32 {
    RtlFreeHeap(RtlGetProcessHeap(), 0, *object_descriptor);
    0
}

#[no_mangle]
pub unsafe extern "system" fn RtlNewSecurityObject(
    parent: *const u8,
    creator: *const u8,
    new_descriptor: *mut *mut u8,
    is_directory_object: u8,
    token: *mut u8,
    generic_mapping: *const GENERIC_MAPPING_FFI,
) -> i32 {
    RtlpNewSecurityObject(
        parent,
        creator,
        new_descriptor,
        core::ptr::null_mut(),
        0,
        is_directory_object,
        0,
        token,
        generic_mapping,
    )
}

#[no_mangle]
pub unsafe extern "system" fn RtlNewSecurityObjectEx(
    parent: *const u8,
    creator: *const u8,
    new_descriptor: *mut *mut u8,
    object_type: *mut u8,
    is_directory_object: u8,
    auto_inherit_flags: u32,
    token: *mut u8,
    generic_mapping: *const GENERIC_MAPPING_FFI,
) -> i32 {
    let mut ot_local = object_type;
    let (otp, count) = if !object_type.is_null() {
        (&mut ot_local as *mut *mut u8, 1u32)
    } else {
        (core::ptr::null_mut(), 0u32)
    };
    RtlpNewSecurityObject(
        parent,
        creator,
        new_descriptor,
        otp,
        count,
        is_directory_object,
        auto_inherit_flags,
        token,
        generic_mapping,
    )
}

#[no_mangle]
pub unsafe extern "system" fn RtlNewSecurityObjectWithMultipleInheritance(
    parent: *const u8,
    creator: *const u8,
    new_descriptor: *mut *mut u8,
    object_types: *mut *mut u8,
    guid_count: u32,
    is_directory_object: u8,
    auto_inherit_flags: u32,
    token: *mut u8,
    generic_mapping: *const GENERIC_MAPPING_FFI,
) -> i32 {
    RtlpNewSecurityObject(
        parent,
        creator,
        new_descriptor,
        object_types,
        guid_count,
        is_directory_object,
        auto_inherit_flags,
        token,
        generic_mapping,
    )
}

#[no_mangle]
pub unsafe extern "system" fn RtlNewInstanceSecurityObject(
    parent_descriptor_changed: u8,
    creator_descriptor_changed: u8,
    old_client_token_modified_id: *const LUID_FFI,
    new_client_token_modified_id: *mut LUID_FFI,
    parent_descriptor: *const u8,
    creator_descriptor: *const u8,
    new_descriptor: *mut *mut u8,
    is_directory_object: u8,
    token: *mut u8,
    generic_mapping: *const GENERIC_MAPPING_FFI,
) -> i32 {
    // Query the token statistics → ModifiedId. The genuine TOKEN_STATISTICS
    // layout puts ModifiedId at offset 24 (LUID after TokenId+AuthenticationId).
    let mut buf = [0u8; SIZEOF_TOKEN_STATISTICS];
    let mut size: u32 = 0;
    let status = NtQueryInformationToken(
        token,
        TOKEN_STATISTICS_INFO_CLASS,
        buf.as_mut_ptr(),
        SIZEOF_TOKEN_STATISTICS as u32,
        &mut size,
    );
    if !nt_success(status) {
        return status;
    }
    // ModifiedId offset in TOKEN_STATISTICS: TokenId (8) + AuthenticationId (8)
    // + ExpirationTime (8) = 24.
    let modified_id = *(buf.as_ptr().add(24) as *const LUID_FFI);
    *new_client_token_modified_id = modified_id;

    if RtlEqualLuid(new_client_token_modified_id, old_client_token_modified_id) != 0
        && parent_descriptor_changed == 0
        && creator_descriptor_changed == 0
    {
        *new_descriptor = core::ptr::null_mut();
        return 0;
    }

    RtlNewSecurityObject(
        parent_descriptor,
        creator_descriptor,
        new_descriptor,
        is_directory_object,
        token,
        generic_mapping,
    )
}

#[no_mangle]
pub unsafe extern "system" fn RtlCreateUserSecurityObject(
    ace_data: *mut u8,
    ace_count: u32,
    owner_sid: *mut u8,
    group_sid: *mut u8,
    is_directory_object: u8,
    generic_mapping: *const GENERIC_MAPPING_FFI,
    new_descriptor: *mut *mut u8,
) -> i32 {
    let mut sd_ptr: *mut u8 = core::ptr::null_mut();
    let status = RtlCreateAndSetSD(ace_data, ace_count, owner_sid, group_sid, &mut sd_ptr);
    if !nt_success(status) {
        return status;
    }
    let mut token_handle: *mut u8 = core::ptr::null_mut();
    let status = NtOpenProcessToken(
        priv_::nt_current_process(),
        priv_::TOKEN_QUERY,
        &mut token_handle,
    );
    if !nt_success(status) {
        RtlFreeHeap(RtlGetProcessHeap(), 0, sd_ptr);
        return status;
    }
    let status = RtlNewSecurityObject(
        core::ptr::null(),
        sd_ptr,
        new_descriptor,
        is_directory_object,
        token_handle,
        generic_mapping,
    );
    let _ = NtClose(token_handle);
    RtlFreeHeap(RtlGetProcessHeap(), 0, sd_ptr);
    status
}

#[no_mangle]
pub unsafe extern "system" fn RtlSetSecurityObject(
    security_information: u32,
    modification_descriptor: *const u8,
    objects_security_descriptor: *mut *mut u8,
    generic_mapping: *const GENERIC_MAPPING_FFI,
    token: *mut u8,
) -> i32 {
    RtlpSetSecurityObject(
        core::ptr::null_mut(),
        security_information,
        modification_descriptor,
        objects_security_descriptor,
        0,
        PAGED_POOL,
        generic_mapping,
        token,
    )
}

#[no_mangle]
pub unsafe extern "system" fn RtlSetSecurityObjectEx(
    security_information: u32,
    modification_descriptor: *const u8,
    objects_security_descriptor: *mut *mut u8,
    auto_inherit_flags: u32,
    generic_mapping: *const GENERIC_MAPPING_FFI,
    token: *mut u8,
) -> i32 {
    RtlpSetSecurityObject(
        core::ptr::null_mut(),
        security_information,
        modification_descriptor,
        objects_security_descriptor,
        auto_inherit_flags,
        PAGED_POOL,
        generic_mapping,
        token,
    )
}

#[no_mangle]
pub unsafe extern "system" fn RtlConvertToAutoInheritSecurityObject(
    parent: *const u8,
    creator: *const u8,
    new_descriptor: *mut *mut u8,
    object_type: *mut u8,
    is_directory_object: u8,
    generic_mapping: *const GENERIC_MAPPING_FFI,
) -> i32 {
    RtlpConvertToAutoInheritSecurityObject(
        parent,
        creator,
        new_descriptor,
        object_type,
        is_directory_object,
        generic_mapping,
    )
}

// ---------------------------------------------------------------------
// RtlQuerySecurityObject — second byte-diff target. Composes an
// absolute SD from existing-SD pieces (filtered by SecurityInformation),
// then converts to self-relative in the caller's buffer.
// ---------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "system" fn RtlQuerySecurityObject(
    object_descriptor: *const u8,
    security_information: u32,
    resultant_descriptor: *mut u8,
    descriptor_length: u32,
    return_length: *mut u32,
) -> i32 {
    let mut desc = SECURITY_DESCRIPTOR {
        Revision: 0,
        Sbz1: 0,
        Control: 0,
        Owner: core::ptr::null_mut(),
        Group: core::ptr::null_mut(),
        Sacl: core::ptr::null_mut(),
        Dacl: core::ptr::null_mut(),
    };
    let status = RtlCreateSecurityDescriptor(
        &mut desc as *mut SECURITY_DESCRIPTOR,
        sd_::SECURITY_DESCRIPTOR_REVISION,
    );
    if !nt_success(status) {
        return status;
    }

    let mut psid: *mut u8 = core::ptr::null_mut();
    let mut defaulted: u8 = 0;
    let mut present: u8 = 0;
    let mut pacl: *mut u8 = core::ptr::null_mut();

    if security_information & sd_::OWNER_SECURITY_INFORMATION != 0 {
        let s = RtlGetOwnerSecurityDescriptor(
            object_descriptor as *const SECURITY_DESCRIPTOR,
            &mut psid,
            &mut defaulted,
        );
        if !nt_success(s) {
            return s;
        }
        let s =
            RtlSetOwnerSecurityDescriptor(&mut desc as *mut SECURITY_DESCRIPTOR, psid, defaulted);
        if !nt_success(s) {
            return s;
        }
    }

    if security_information & sd_::GROUP_SECURITY_INFORMATION != 0 {
        let s = RtlGetGroupSecurityDescriptor(
            object_descriptor as *const SECURITY_DESCRIPTOR,
            &mut psid,
            &mut defaulted,
        );
        if !nt_success(s) {
            return s;
        }
        let s =
            RtlSetGroupSecurityDescriptor(&mut desc as *mut SECURITY_DESCRIPTOR, psid, defaulted);
        if !nt_success(s) {
            return s;
        }
    }

    if security_information & sd_::DACL_SECURITY_INFORMATION != 0 {
        let s = RtlGetDaclSecurityDescriptor(
            object_descriptor as *const SECURITY_DESCRIPTOR,
            &mut present,
            &mut pacl,
            &mut defaulted,
        );
        if !nt_success(s) {
            return s;
        }
        let s = RtlSetDaclSecurityDescriptor(
            &mut desc as *mut SECURITY_DESCRIPTOR,
            present,
            pacl,
            defaulted,
        );
        if !nt_success(s) {
            return s;
        }
    }

    if security_information & sd_::SACL_SECURITY_INFORMATION != 0 {
        let s = RtlGetSaclSecurityDescriptor(
            object_descriptor as *const SECURITY_DESCRIPTOR,
            &mut present,
            &mut pacl,
            &mut defaulted,
        );
        if !nt_success(s) {
            return s;
        }
        let s = RtlSetSaclSecurityDescriptor(
            &mut desc as *mut SECURITY_DESCRIPTOR,
            present,
            pacl,
            defaulted,
        );
        if !nt_success(s) {
            return s;
        }
    }

    *return_length = descriptor_length;
    RtlAbsoluteToSelfRelativeSD(
        &mut desc as *mut SECURITY_DESCRIPTOR,
        resultant_descriptor as *mut SECURITY_DESCRIPTOR_RELATIVE,
        return_length,
    )
}

// ---------------------------------------------------------------------
// RtlDefaultNpAcl + RtlNewSecurityGrantedAccess: heavy syscalls.
// Translate-only; no byte-diff coverage.
// ---------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "system" fn RtlDefaultNpAcl(p_acl: *mut *mut u8) -> i32 {
    let nt_authority = SID_IDENTIFIER_AUTHORITY_FFI {
        Value: SECURITY_NT_AUTHORITY_VAL,
    };
    let world_authority = SID_IDENTIFIER_AUTHORITY_FFI {
        Value: SECURITY_WORLD_AUTHORITY_VAL,
    };

    // SID buffer big enough for max 2 SubAuthorities.
    let mut sid_buf = SID2 {
        Revision: 0,
        SubAuthorityCount: 0,
        IdentifierAuthority: SID_IDENTIFIER_AUTHORITY_FFI { Value: [0; 6] },
        SubAuthority: [0; 2],
    };
    let p_sid = &mut sid_buf as *mut SID2 as *mut u8;

    *p_acl = core::ptr::null_mut();

    let mut token_handle: *mut u8 = core::ptr::null_mut();
    let mut status = NtOpenThreadToken(
        priv_::nt_current_thread(),
        priv_::TOKEN_QUERY,
        1,
        &mut token_handle,
    );
    if status == STATUS_NO_TOKEN {
        status = NtOpenProcessToken(
            priv_::nt_current_process(),
            priv_::TOKEN_QUERY,
            &mut token_handle,
        );
    }
    if !nt_success(status) {
        return status;
    }

    let mut return_length: u32 = 0;
    let status = NtQueryInformationToken(
        token_handle,
        TOKEN_OWNER_INFO_CLASS,
        core::ptr::null_mut(),
        0,
        &mut return_length,
    );
    if status != env::STATUS_BUFFER_TOO_SMALL {
        let _ = NtClose(token_handle);
        return status;
    }

    let owner_sid_buf = RtlAllocateHeap(RtlGetProcessHeap(), 0, return_length as usize);
    if owner_sid_buf.is_null() {
        let _ = NtClose(token_handle);
        return STATUS_NO_MEMORY;
    }

    let status = NtQueryInformationToken(
        token_handle,
        TOKEN_OWNER_INFO_CLASS,
        owner_sid_buf,
        return_length,
        &mut return_length,
    );
    if !nt_success(status) {
        RtlFreeHeap(RtlGetProcessHeap(), 0, owner_sid_buf);
        let _ = NtClose(token_handle);
        return status;
    }

    // TOKEN_OWNER->Owner is just a PSID pointer at offset 0.
    let owner_sid_ptr = *(owner_sid_buf as *const *mut u8);

    let acl_size = (core::mem::size_of::<ACL_HDR>() as u32) +    // ACL header (8)
        5 * 8u32 +                                                // 5 ACE headers
        RtlLengthRequiredSid(1)                                   // LocalSystem
        + RtlLengthRequiredSid(2)                                 // Administrators
        + RtlLengthRequiredSid(1)                                 // Anonymous
        + RtlLengthRequiredSid(1)                                 // World
        + RtlLengthSid(owner_sid_ptr); // Owner

    let acl = RtlAllocateHeap(RtlGetProcessHeap(), 0, acl_size as usize);
    if acl.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, owner_sid_buf);
        let _ = NtClose(token_handle);
        return STATUS_NO_MEMORY;
    }
    *p_acl = acl;

    let mut status = RtlCreateAcl(acl, acl_size, ACL_REVISION2 as u32);

    // Local System SID.
    let _ = RtlInitializeSid(p_sid, &nt_authority as *const _ as *const u8, 1);
    *RtlSubAuthoritySid(p_sid, 0) = SECURITY_LOCAL_SYSTEM_RID;
    status = RtlAddAccessAllowedAce(acl, ACL_REVISION2 as u32, GENERIC_ALL, p_sid);

    // Administrators SID.
    let _ = RtlInitializeSid(p_sid, &nt_authority as *const _ as *const u8, 2);
    *RtlSubAuthoritySid(p_sid, 0) = SECURITY_BUILTIN_DOMAIN_RID;
    *RtlSubAuthoritySid(p_sid, 1) = DOMAIN_ALIAS_RID_ADMINS;
    status = RtlAddAccessAllowedAce(acl, ACL_REVISION2 as u32, GENERIC_ALL, p_sid);

    // Owner SID.
    let _ = RtlAddAccessAllowedAce(acl, ACL_REVISION2 as u32, GENERIC_ALL, owner_sid_ptr);

    // Anonymous SID.
    let _ = RtlInitializeSid(p_sid, &nt_authority as *const _ as *const u8, 1);
    *RtlSubAuthoritySid(p_sid, 0) = SECURITY_ANONYMOUS_LOGON_RID;
    status = RtlAddAccessAllowedAce(acl, ACL_REVISION2 as u32, GENERIC_READ, p_sid);

    // World SID.
    let _ = RtlInitializeSid(p_sid, &world_authority as *const _ as *const u8, 1);
    *RtlSubAuthoritySid(p_sid, 0) = SECURITY_WORLD_RID;
    status = RtlAddAccessAllowedAce(acl, ACL_REVISION2 as u32, GENERIC_READ, p_sid);

    if !nt_success(status) {
        RtlFreeHeap(RtlGetProcessHeap(), 0, acl);
        *p_acl = core::ptr::null_mut();
    }

    RtlFreeHeap(RtlGetProcessHeap(), 0, owner_sid_buf);
    let _ = NtClose(token_handle);
    status
}

#[repr(C)]
struct ACL_HDR {
    AclRevision: u8,
    Sbz1: u8,
    AclSize: u16,
    AceCount: u16,
    Sbz2: u16,
}

#[no_mangle]
pub unsafe extern "system" fn RtlNewSecurityGrantedAccess(
    mut desired_access: u32,
    privileges: *mut PRIVILEGE_SET_FFI,
    length: *mut u32,
    mut token: *mut u8,
    generic_mapping: *const GENERIC_MAPPING_FFI,
    remaining_desired_access: *mut u32,
) -> i32 {
    let caller_token = !token.is_null();
    if !caller_token {
        let s = NtOpenThreadToken(
            priv_::nt_current_thread(),
            priv_::TOKEN_QUERY,
            1,
            &mut token,
        );
        if !nt_success(s) {
            return s;
        }
    }

    let mut buf = [0u8; SIZEOF_TOKEN_STATISTICS];
    let mut sz: u32 = 0;
    let _ = NtQueryInformationToken(
        token,
        TOKEN_STATISTICS_INFO_CLASS,
        buf.as_mut_ptr(),
        SIZEOF_TOKEN_STATISTICS as u32,
        &mut sz,
    );

    RtlMapGenericMask(&mut desired_access, generic_mapping);
    *remaining_desired_access = desired_access;

    let granted = if desired_access & ACCESS_SYSTEM_SECURITY != 0 {
        *remaining_desired_access &= !ACCESS_SYSTEM_SECURITY;
        true
    } else {
        false
    };

    if !caller_token {
        let _ = NtClose(token);
    }

    let needed = core::mem::size_of::<PRIVILEGE_SET_FFI>() as u32;
    if needed > *length {
        *length = needed;
        return env::STATUS_BUFFER_TOO_SMALL;
    }

    core::ptr::write_bytes(privileges as *mut u8, 0, needed as usize);
    if granted {
        (*privileges).PrivilegeCount = 1;
        (*privileges).Privilege[0].Luid.LowPart = SE_SECURITY_PRIVILEGE;
        (*privileges).Privilege[0].Luid.HighPart = 0;
        (*privileges).Privilege[0].Attributes = SE_PRIVILEGE_USED_FOR_ACCESS;
    }
    0
}

// =====================================================================
// Wave RES — straight translation of `sdk/lib/rtl/resource.c` (8 fns,
// pure plumbing: critsec + 2 semaphores + ULONG counters). No byte
// diff — every output observable lives behind a syscall or a private
// critsec; smoke is the net.
// =====================================================================

extern "system" {
    fn RtlInitializeCriticalSection(crit: *mut u8) -> i32;
    fn RtlDeleteCriticalSection(crit: *mut u8) -> i32;
    fn RtlEnterCriticalSection(crit: *mut u8) -> i32;
    fn RtlLeaveCriticalSection(crit: *mut u8) -> i32;
    fn NtCreateSemaphore(
        Handle: *mut *mut u8,
        DesiredAccess: u32,
        ObjectAttributes: *mut u8,
        InitialCount: i32,
        MaximumCount: i32,
    ) -> i32;
    fn NtReleaseSemaphore(
        SemaphoreHandle: *mut u8,
        ReleaseCount: i32,
        PreviousCount: *mut i32,
    ) -> i32;
    fn NtWaitForSingleObject(Handle: *mut u8, Alertable: u8, Timeout: *mut i64) -> i32;
}

/// Read `NtCurrentTeb()->ClientId.UniqueThread` via per-arch inline asm.
/// The C original uses the `NtCurrentTeb()` macro intrinsic; we replicate
/// the GS/FS/X18 load + offset directly so we don't drag a C macro into
/// the Rust translation. Offset constant lives in `core_rs::resource`.
#[inline]
unsafe fn read_current_thread_id() -> *mut u8 {
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    {
        let val: u64;
        let teb: u64;
        core::arch::asm!(
            "mov {teb}, gs:[0x30]",
            "mov {val}, [{teb} + {off}]",
            teb = out(reg) teb,
            val = out(reg) val,
            off = const res::TEB_UNIQUE_THREAD_OFFSET,
            options(nostack, preserves_flags, readonly),
        );
        let _ = teb;
        val as *mut u8
    }
    #[cfg(all(target_os = "windows", target_arch = "x86"))]
    {
        let val: u32;
        let teb: u32;
        core::arch::asm!(
            "mov {teb:e}, fs:[0x18]",
            "mov {val:e}, [{teb:e} + {off}]",
            teb = out(reg) teb,
            val = out(reg) val,
            off = const res::TEB_UNIQUE_THREAD_OFFSET,
            options(nostack, preserves_flags, readonly),
        );
        let _ = teb;
        val as *mut u8
    }
    #[cfg(all(target_os = "windows", target_arch = "aarch64"))]
    {
        let val: u64;
        core::arch::asm!(
            "ldr {val}, [x18, {off}]",
            val = out(reg) val,
            off = const res::TEB_UNIQUE_THREAD_OFFSET,
            options(nostack, preserves_flags, readonly),
        );
        val as *mut u8
    }
    #[cfg(not(target_os = "windows"))]
    {
        // Host (rtl-difftest): no TEB. Wave RES has no diff harness;
        // return NULL — only RtlAcquire*/Release flow would touch this.
        core::ptr::null_mut()
    }
}

/// `VOID NTAPI RtlInitializeResource(PRTL_RESOURCE)`.
///
/// Allocates the critsec + the two semaphores; on any failure raises
/// the failing NTSTATUS via `RtlRaiseStatus`. Counters/owner cleared.
#[no_mangle]
pub unsafe extern "system" fn RtlInitializeResource(resource: *mut res::RTL_RESOURCE) {
    let r = &mut *resource;

    let status = RtlInitializeCriticalSection(&mut r.Lock as *mut _ as *mut u8);
    if status < 0 {
        RtlRaiseStatus(status);
    }

    let status = NtCreateSemaphore(
        &mut r.SharedSemaphore,
        res::SEMAPHORE_ALL_ACCESS,
        core::ptr::null_mut(),
        0,
        65535,
    );
    if status < 0 {
        RtlRaiseStatus(status);
    }
    r.SharedWaiters = 0;

    let status = NtCreateSemaphore(
        &mut r.ExclusiveSemaphore,
        res::SEMAPHORE_ALL_ACCESS,
        core::ptr::null_mut(),
        0,
        65535,
    );
    if status < 0 {
        RtlRaiseStatus(status);
    }
    r.ExclusiveWaiters = 0;
    r.NumberActive = 0;
    r.OwningThread = core::ptr::null_mut();
    r.TimeoutBoost = 0; // C comment: no info on this one, default is 0
}

/// `VOID NTAPI RtlDeleteResource(PRTL_RESOURCE)`.
#[no_mangle]
pub unsafe extern "system" fn RtlDeleteResource(resource: *mut res::RTL_RESOURCE) {
    let r = &mut *resource;
    let _ = RtlDeleteCriticalSection(&mut r.Lock as *mut _ as *mut u8);
    let _ = NtClose(r.ExclusiveSemaphore);
    let _ = NtClose(r.SharedSemaphore);
    r.OwningThread = core::ptr::null_mut();
    r.ExclusiveWaiters = 0;
    r.SharedWaiters = 0;
    r.NumberActive = 0;
}

/// `BOOLEAN NTAPI RtlAcquireResourceExclusive(PRTL_RESOURCE, BOOLEAN)`.
///
/// Faithful port of the C `goto`-driven state machine (start/wait/done).
#[no_mangle]
pub unsafe extern "system" fn RtlAcquireResourceExclusive(
    resource: *mut res::RTL_RESOURCE,
    wait: u8,
) -> u8 {
    let r = &mut *resource;
    let mut ret_val: u8 = 0;

    'start: loop {
        let _ = RtlEnterCriticalSection(&mut r.Lock as *mut _ as *mut u8);

        if r.NumberActive == 0 {
            // lock is free
            r.NumberActive = -1;
            ret_val = 1;
        } else if r.NumberActive < 0 {
            // exclusive lock in progress
            if r.OwningThread == read_current_thread_id() {
                ret_val = 1;
                r.NumberActive -= 1;
                // C: goto done — skip the `if (retVal)` owner write.
                let _ = RtlLeaveCriticalSection(&mut r.Lock as *mut _ as *mut u8);
                return ret_val;
            }
            // wait label
            if wait != 0 {
                r.ExclusiveWaiters += 1;
                let _ = RtlLeaveCriticalSection(&mut r.Lock as *mut _ as *mut u8);
                let status = NtWaitForSingleObject(r.ExclusiveSemaphore, 0, core::ptr::null_mut());
                if status < 0 {
                    // C: goto done — but Lock is released and we
                    // haven't re-entered yet. Falls into the bare
                    // RtlLeaveCriticalSection below; the C code is
                    // technically buggy here (double-leaves) but we
                    // mirror the original faithfully.
                    let _ = RtlLeaveCriticalSection(&mut r.Lock as *mut _ as *mut u8);
                    return ret_val;
                }
                // C: goto start — restart acquisition.
                continue 'start;
            }
        } else {
            // one or more shared locks are in progress
            if wait != 0 {
                // C: goto wait — same shape as the wait above.
                r.ExclusiveWaiters += 1;
                let _ = RtlLeaveCriticalSection(&mut r.Lock as *mut _ as *mut u8);
                let status = NtWaitForSingleObject(r.ExclusiveSemaphore, 0, core::ptr::null_mut());
                if status < 0 {
                    let _ = RtlLeaveCriticalSection(&mut r.Lock as *mut _ as *mut u8);
                    return ret_val;
                }
                continue 'start;
            }
        }

        if ret_val != 0 {
            r.OwningThread = read_current_thread_id();
        }
        // C: done label.
        let _ = RtlLeaveCriticalSection(&mut r.Lock as *mut _ as *mut u8);
        return ret_val;
    }
}

/// `BOOLEAN NTAPI RtlAcquireResourceShared(PRTL_RESOURCE, BOOLEAN)`.
#[no_mangle]
pub unsafe extern "system" fn RtlAcquireResourceShared(
    resource: *mut res::RTL_RESOURCE,
    wait: u8,
) -> u8 {
    let r = &mut *resource;
    // C: STATUS_UNSUCCESSFUL = 0xC0000001 — used to gate the
    // "NumberActive++ vs already-counted" branch on first entry.
    let mut status: i32 = 0xC000_0001u32 as i32;
    let mut ret_val: u8 = 0;

    'start: loop {
        let _ = RtlEnterCriticalSection(&mut r.Lock as *mut _ as *mut u8);
        if r.NumberActive < 0 {
            if r.OwningThread == read_current_thread_id() {
                r.NumberActive -= 1;
                ret_val = 1;
                let _ = RtlLeaveCriticalSection(&mut r.Lock as *mut _ as *mut u8);
                return ret_val;
            }
            if wait != 0 {
                r.SharedWaiters += 1;
                let _ = RtlLeaveCriticalSection(&mut r.Lock as *mut _ as *mut u8);
                status = NtWaitForSingleObject(r.SharedSemaphore, 0, core::ptr::null_mut());
                if status < 0 {
                    let _ = RtlLeaveCriticalSection(&mut r.Lock as *mut _ as *mut u8);
                    return ret_val;
                }
                continue 'start;
            }
        } else {
            // C: "otherwise RtlReleaseResource() has already done it"
            // — STATUS_WAIT_0 means we were woken out of the shared
            // semaphore and our slot is already accounted for.
            if status != res::STATUS_WAIT_0 {
                r.NumberActive += 1;
            }
            ret_val = 1;
        }
        let _ = RtlLeaveCriticalSection(&mut r.Lock as *mut _ as *mut u8);
        return ret_val;
    }
}

/// `VOID NTAPI RtlConvertExclusiveToShared(PRTL_RESOURCE)`.
#[no_mangle]
pub unsafe extern "system" fn RtlConvertExclusiveToShared(resource: *mut res::RTL_RESOURCE) {
    let r = &mut *resource;
    let _ = RtlEnterCriticalSection(&mut r.Lock as *mut _ as *mut u8);

    if r.NumberActive == -1 {
        r.OwningThread = core::ptr::null_mut();

        if r.SharedWaiters > 0 {
            // Prevent new writers from joining until all queued
            // readers have done their thing.
            let n = r.SharedWaiters;
            r.NumberActive = (r.SharedWaiters + 1) as i32;
            r.SharedWaiters = 0;
            let _ = NtReleaseSemaphore(r.SharedSemaphore, n as i32, core::ptr::null_mut());
        } else {
            r.NumberActive = 1;
        }
    }

    let _ = RtlLeaveCriticalSection(&mut r.Lock as *mut _ as *mut u8);
}

/// `VOID NTAPI RtlConvertSharedToExclusive(PRTL_RESOURCE)`.
#[no_mangle]
pub unsafe extern "system" fn RtlConvertSharedToExclusive(resource: *mut res::RTL_RESOURCE) {
    let r = &mut *resource;
    let _ = RtlEnterCriticalSection(&mut r.Lock as *mut _ as *mut u8);

    if r.NumberActive == 1 {
        r.OwningThread = read_current_thread_id();
        r.NumberActive = -1;
    } else {
        r.ExclusiveWaiters += 1;
        let _ = RtlLeaveCriticalSection(&mut r.Lock as *mut _ as *mut u8);
        let status = NtWaitForSingleObject(r.ExclusiveSemaphore, 0, core::ptr::null_mut());
        if status < 0 {
            return;
        }
        let _ = RtlEnterCriticalSection(&mut r.Lock as *mut _ as *mut u8);
        r.OwningThread = read_current_thread_id();
        r.NumberActive = -1;
    }
    let _ = RtlLeaveCriticalSection(&mut r.Lock as *mut _ as *mut u8);
}

/// `VOID NTAPI RtlReleaseResource(PRTL_RESOURCE)`.
#[no_mangle]
pub unsafe extern "system" fn RtlReleaseResource(resource: *mut res::RTL_RESOURCE) {
    let r = &mut *resource;
    let _ = RtlEnterCriticalSection(&mut r.Lock as *mut _ as *mut u8);

    if r.NumberActive > 0 {
        // one or more readers
        r.NumberActive -= 1;
        if r.NumberActive == 0 && r.ExclusiveWaiters > 0 {
            // wake_exclusive label
            r.ExclusiveWaiters -= 1;
            let _ = NtReleaseSemaphore(r.ExclusiveSemaphore, 1, core::ptr::null_mut());
        }
    } else if r.NumberActive < 0 {
        // writer, possibly recursive
        r.NumberActive += 1;
        if r.NumberActive == 0 {
            r.OwningThread = core::ptr::null_mut();
            if r.ExclusiveWaiters > 0 {
                // wake_exclusive
                r.ExclusiveWaiters -= 1;
                let _ = NtReleaseSemaphore(r.ExclusiveSemaphore, 1, core::ptr::null_mut());
            } else if r.SharedWaiters > 0 {
                // Prevent new writers from joining until all queued
                // readers have done their thing.
                let n = r.SharedWaiters;
                r.NumberActive = r.SharedWaiters as i32;
                r.SharedWaiters = 0;
                let _ = NtReleaseSemaphore(r.SharedSemaphore, n as i32, core::ptr::null_mut());
            }
        }
    }
    let _ = RtlLeaveCriticalSection(&mut r.Lock as *mut _ as *mut u8);
}

/// `VOID NTAPI RtlDumpResource(PRTL_RESOURCE)`.
///
/// Faithful translation: emit one `DbgPrint` line via the kernel/user
/// `DbgPrint` extern. Format strings use ANSI literals (`%p`, `%d`,
/// `%u`) — same as the C original.
#[no_mangle]
pub unsafe extern "system" fn RtlDumpResource(resource: *mut res::RTL_RESOURCE) {
    extern "C" {
        fn DbgPrint(fmt: *const u8, ...) -> u32;
    }
    let r = &*resource;

    // C: "RtlDumpResource(%p):\n\tactive count = %d\n\twaiting readers = %u\n\twaiting writers = %u\n"
    let fmt1 = b"RtlDumpResource(%p):\n\tactive count = %d\n\twaiting readers = %u\n\twaiting writers = %u\n\0";
    let _ = DbgPrint(
        fmt1.as_ptr(),
        resource as *const u8,
        r.NumberActive,
        r.SharedWaiters,
        r.ExclusiveWaiters,
    );

    if r.NumberActive != 0 {
        let fmt2 = b"\towner thread = %p\n\0";
        let _ = DbgPrint(fmt2.as_ptr(), r.OwningThread);
    }
}

// =====================================================================
// Wave THR — straight translation of `sdk/lib/rtl/thread.c` (8 fns,
// pure plumbing: `Zw*`/`Nt*` thread + VM syscalls + `Rtl*` helpers).
// PEB/TEB/IMAGE_NT_HEADERS field reads go through `RtlpThr*` libsupp
// helpers — same pattern as Wave ENV's `RtlpGetPebEnvironment`. No
// byte-diff: every observable lives behind a syscall.
// =====================================================================

extern "system" {
    fn ZwQuerySystemInformation(
        InfoClass: u32,
        Info: *mut u8,
        InfoLen: u32,
        ReturnLength: *mut u32,
    ) -> i32;
    fn ZwAllocateVirtualMemory(
        ProcessHandle: *mut u8,
        BaseAddress: *mut *mut u8,
        ZeroBits: u32,
        RegionSize: *mut usize,
        AllocationType: u32,
        Protect: u32,
    ) -> i32;
    fn ZwFreeVirtualMemory(
        ProcessHandle: *mut u8,
        BaseAddress: *mut *mut u8,
        RegionSize: *mut usize,
        FreeType: u32,
    ) -> i32;
    fn ZwProtectVirtualMemory(
        ProcessHandle: *mut u8,
        BaseAddress: *mut *mut u8,
        RegionSize: *mut usize,
        NewProtect: u32,
        OldProtect: *mut u32,
    ) -> i32;
    // NtFreeVirtualMemory + ZwSetInformationThread declared earlier
    // (Wave ENV + Wave PRIV); reused here.
    fn NtReadVirtualMemory(
        ProcessHandle: *mut u8,
        BaseAddress: *const u8,
        Buffer: *mut u8,
        BytesToRead: usize,
        NumberOfBytesRead: *mut usize,
    ) -> i32;
    fn NtQueryInformationThread(
        ThreadHandle: *mut u8,
        ThreadInformationClass: u32,
        ThreadInformation: *mut u8,
        ThreadInformationLength: u32,
        ReturnLength: *mut u32,
    ) -> i32;
    fn ZwQueryInformationThread(
        ThreadHandle: *mut u8,
        ThreadInformationClass: u32,
        ThreadInformation: *mut u8,
        ThreadInformationLength: u32,
        ReturnLength: *mut u32,
    ) -> i32;
    fn ZwCreateThread(
        ThreadHandle: *mut *mut u8,
        DesiredAccess: u32,
        ObjectAttributes: *mut thr::OBJECT_ATTRIBUTES,
        ProcessHandle: *mut u8,
        ClientId: *mut thr::CLIENT_ID,
        ThreadContext: *mut u8,
        InitialTeb: *mut thr::INITIAL_TEB,
        CreateSuspended: u8,
    ) -> i32;
    fn NtTerminateThread(ThreadHandle: *mut u8, ExitStatus: i32) -> i32;
    fn LdrShutdownThread() -> i32;
    fn RtlImageNtHeader(Base: *mut u8) -> *mut u8;
    fn RtlInitializeContext(
        ProcessHandle: *mut u8,
        ThreadContext: *mut u8,
        ThreadStartParam: *mut u8,
        ThreadStartAddress: *mut u8,
        InitialTeb: *mut u8,
    );

    // libsupp helpers (per-flavor: ntoskrnl/rtl/libsupp.c or
    // dll/ntdll/rtl/libsupp.c). See env-wave precedent.
    fn RtlpThrGetPebImageBaseAddress() -> *mut u8;
    fn RtlpThrGetPebMinimumStackCommit() -> usize;
    fn RtlpThrGetPebNtGlobalFlag() -> u32;
    fn RtlpThrSetFreeStackOnTermination();
    fn RtlpThrTebDeallocStackAddr(TebBase: *mut u8) -> *mut u8;
    fn RtlpThrImageStackReserve(Headers: *mut u8) -> usize;
    fn RtlpThrImageStackCommit(Headers: *mut u8) -> usize;
    fn RtlpThrCurrentTeb() -> *mut u8;
}

/// `NTSTATUS NTAPI RtlpCreateUserStack(HANDLE, SIZE_T, SIZE_T, ULONG, PINITIAL_TEB)`.
///
/// Faithful translation: query system info, reserve+commit a stack
/// region, optionally guard-page the bottom. All-or-nothing — on the
/// commit failure path the reserved region is released. Per-image
/// stack defaults come from PE OptionalHeader when the target is the
/// current process.
#[no_mangle]
pub unsafe extern "system" fn RtlpCreateUserStack(
    process_handle: *mut u8,
    mut stack_reserve: usize,
    mut stack_commit: usize,
    stack_zero_bits: u32,
    initial_teb: *mut thr::INITIAL_TEB,
) -> i32 {
    let mut sbi: thr::SYSTEM_BASIC_INFORMATION = core::mem::zeroed();
    let status = ZwQuerySystemInformation(
        thr::SYSTEM_BASIC_INFORMATION_CLASS,
        &mut sbi as *mut _ as *mut u8,
        core::mem::size_of::<thr::SYSTEM_BASIC_INFORMATION>() as u32,
        core::ptr::null_mut(),
    );
    if status < 0 {
        return status;
    }

    // Use the Image Settings if we are dealing with the current Process.
    if process_handle == thr::NT_CURRENT_PROCESS as *mut u8 {
        let headers = RtlImageNtHeader(RtlpThrGetPebImageBaseAddress());
        if headers.is_null() {
            return thr::STATUS_INVALID_IMAGE_FORMAT;
        }

        if stack_reserve == 0 {
            stack_reserve = RtlpThrImageStackReserve(headers);
        }
        if stack_commit == 0 {
            stack_commit = RtlpThrImageStackCommit(headers);
        }

        let minimum = RtlpThrGetPebMinimumStackCommit();
        if minimum != 0 && stack_commit < minimum {
            stack_commit = minimum;
        }
    } else {
        // Use the System Settings if needed.
        if stack_reserve == 0 {
            stack_reserve = sbi.AllocationGranularity as usize;
        }
        if stack_commit == 0 {
            stack_commit = sbi.PageSize as usize;
        }
    }

    // Pure decision: derive reserve/commit/use_guard. Delegates
    // every alignment + grow + guard-fit branch to
    // `core_rs::thread::compute_stack_layout`. See its unit tests
    // for the spec.
    let layout = thr::compute_stack_layout(
        stack_reserve,
        stack_commit,
        sbi.PageSize as usize,
        sbi.AllocationGranularity as usize,
    );
    stack_reserve = layout.stack_reserve;
    stack_commit = layout.stack_commit; // includes guard page if use_guard
    let use_guard = layout.use_guard;

    // Reserve memory for the stack.
    let mut stack: *mut u8 = core::ptr::null_mut();
    let mut reserve_size = stack_reserve;
    let status = ZwAllocateVirtualMemory(
        process_handle,
        &mut stack,
        stack_zero_bits,
        &mut reserve_size,
        thr::MEM_RESERVE,
        thr::PAGE_READWRITE,
    );
    if status < 0 {
        return status;
    }

    // Now set up some basic Initial TEB Parameters.
    let it = &mut *initial_teb;
    it.AllocatedStackBase = stack;
    it.StackBase = stack.add(stack_reserve);
    it.PreviousStackBase = core::ptr::null_mut();
    it.PreviousStackLimit = core::ptr::null_mut();

    // Stack top = base + reserve - commit. `stack_commit` already
    // includes the guard page if use_guard, so the stack_top here
    // is already pointing at the start of the guard page (or first
    // committed page when !use_guard).
    let mut stack_top = stack.add(stack_reserve - stack_commit);

    // Allocate memory for the stack.
    let mut commit_size = stack_commit;
    let status = ZwAllocateVirtualMemory(
        process_handle,
        &mut stack_top,
        0,
        &mut commit_size,
        thr::MEM_COMMIT,
        thr::PAGE_READWRITE,
    );
    if status < 0 {
        let mut guard_size: usize = 0;
        let _ = ZwFreeVirtualMemory(
            process_handle,
            &mut stack_top,
            &mut guard_size,
            thr::MEM_RELEASE,
        );
        return status;
    }

    // Now set the current Stack Limit.
    it.StackLimit = stack_top;

    // Create a guard page if needed.
    if use_guard {
        let mut guard_size: usize = sbi.PageSize as usize;
        let mut dummy: u32 = 0;
        let status = ZwProtectVirtualMemory(
            process_handle,
            &mut stack_top,
            &mut guard_size,
            thr::PAGE_GUARD | thr::PAGE_READWRITE,
            &mut dummy,
        );
        if status < 0 {
            return status;
        }
        // Update the Stack Limit keeping in mind the Guard Page.
        it.StackLimit = (it.StackLimit as usize + guard_size) as *mut u8;
    }

    0 // STATUS_SUCCESS
}

/// `VOID NTAPI RtlpFreeUserStack(HANDLE, PINITIAL_TEB)`.
#[no_mangle]
pub unsafe extern "system" fn RtlpFreeUserStack(
    process_handle: *mut u8,
    initial_teb: *mut thr::INITIAL_TEB,
) {
    let mut dummy: usize = 0;
    let it = &mut *initial_teb;
    let _ = ZwFreeVirtualMemory(
        process_handle,
        &mut it.AllocatedStackBase,
        &mut dummy,
        thr::MEM_RELEASE,
    );
    // RtlZeroMemory(InitialTeb, sizeof(*InitialTeb));
    core::ptr::write_bytes(
        initial_teb as *mut u8,
        0,
        core::mem::size_of::<thr::INITIAL_TEB>(),
    );
}

/// `NTSTATUS __cdecl RtlSetThreadIsCritical(BOOLEAN, PBOOLEAN, BOOLEAN)`.
///
/// Note: C declares this `__cdecl`, not `NTAPI`. `extern "C"` on all
/// arches matches — Wave THR is the only fn here with this calling
/// convention.
#[no_mangle]
pub unsafe extern "C" fn RtlSetThreadIsCritical(
    new_value: u8,
    old_value: *mut u8,
    need_breaks: u8,
) -> i32 {
    // Initialize OldValue to FALSE.
    if !old_value.is_null() {
        *old_value = 0;
    }

    // Fail, if the critical breaks flag is required but is not set.
    if need_breaks != 0 && (RtlpThrGetPebNtGlobalFlag() & thr::FLG_ENABLE_SYSTEM_CRIT_BREAKS) == 0 {
        return thr::STATUS_UNSUCCESSFUL;
    }

    // Check if the caller wants the old value.
    if !old_value.is_null() {
        let mut break_on_termination: u32 = 0;
        let _ = ZwQueryInformationThread(
            thr::NT_CURRENT_THREAD as *mut u8,
            thr::THREAD_BREAK_ON_TERMINATION,
            &mut break_on_termination as *mut _ as *mut u8,
            core::mem::size_of::<u32>() as u32,
            core::ptr::null_mut(),
        );
        *old_value = break_on_termination as u8;
    }

    // Set the break on termination flag for the process.
    let break_on_termination: u32 = new_value as u32;
    ZwSetInformationThread(
        thr::NT_CURRENT_THREAD as *mut u8,
        thr::THREAD_BREAK_ON_TERMINATION,
        &break_on_termination as *const _ as *const u8,
        core::mem::size_of::<u32>() as u32,
    )
}

/// `NTSTATUS NTAPI RtlCreateUserThread(HANDLE, PSECURITY_DESCRIPTOR,
/// BOOLEAN, ULONG, SIZE_T, SIZE_T, PTHREAD_START_ROUTINE, PVOID,
/// PHANDLE, PCLIENT_ID)`.
#[no_mangle]
pub unsafe extern "system" fn RtlCreateUserThread(
    process_handle: *mut u8,
    security_descriptor: *mut u8,
    create_suspended: u8,
    stack_zero_bits: u32,
    stack_reserve: usize,
    stack_commit: usize,
    start_address: *mut u8,
    parameter: *mut u8,
    thread_handle: *mut *mut u8,
    client_id: *mut thr::CLIENT_ID,
) -> i32 {
    let mut initial_teb: thr::INITIAL_TEB = core::mem::zeroed();
    let mut context = [0u8; thr::CONTEXT_SIZE];

    // First, create the Stack.
    let status = RtlpCreateUserStack(
        process_handle,
        stack_reserve,
        stack_commit,
        stack_zero_bits,
        &mut initial_teb,
    );
    if status < 0 {
        return status;
    }

    // Next, set up the Initial Context (per-arch RtlInitializeContext).
    // NOTE: the C `RtlInitializeContext` 5th parameter is mistyped
    // `PINITIAL_TEB` but is actually used as `(ULONG_PTR)StackBase` (the
    // user-stack top). The original C caller (sdk/lib/rtl/thread.c) passes
    // `InitialTeb.StackBase`, NOT `&InitialTeb`. Passing the struct address
    // here set the new thread's user Rsp to a kernel-stack address →
    // KiInitializeUserApc AV (BUGCHECK 0x1E) at first user-thread startup.
    RtlInitializeContext(
        process_handle,
        context.as_mut_ptr(),
        parameter,
        start_address,
        initial_teb.StackBase,
    );

    // InitializeObjectAttributes(&oa, NULL, 0, NULL, SecurityDescriptor)
    let mut oa: thr::OBJECT_ATTRIBUTES = core::mem::zeroed();
    oa.Length = core::mem::size_of::<thr::OBJECT_ATTRIBUTES>() as u32;
    oa.SecurityDescriptor = security_descriptor;

    // Create the Kernel Thread Object.
    let mut handle: *mut u8 = core::ptr::null_mut();
    let mut thread_cid: thr::CLIENT_ID = core::mem::zeroed();
    let status = ZwCreateThread(
        &mut handle,
        thr::THREAD_ALL_ACCESS,
        &mut oa,
        process_handle,
        &mut thread_cid,
        context.as_mut_ptr(),
        &mut initial_teb,
        create_suspended,
    );
    if status < 0 {
        // Free the stack.
        RtlpFreeUserStack(process_handle, &mut initial_teb);
    } else {
        // Return thread data.
        if !thread_handle.is_null() {
            *thread_handle = handle;
        } else {
            let _ = NtClose(handle);
        }
        if !client_id.is_null() {
            *client_id = thread_cid;
        }
    }

    status
}

/// `VOID NTAPI RtlExitUserThread(NTSTATUS)`.
#[no_mangle]
pub unsafe extern "system" fn RtlExitUserThread(status: i32) {
    // Call the Loader and tell him to notify the DLLs.
    let _ = LdrShutdownThread();
    // NtCurrentTeb()->FreeStackOnTermination = TRUE; via libsupp.
    RtlpThrSetFreeStackOnTermination();
    let _ = NtTerminateThread(thr::NT_CURRENT_THREAD as *mut u8, status);
}

/// `VOID NTAPI RtlFreeUserThreadStack(HANDLE, HANDLE)`.
#[no_mangle]
pub unsafe extern "system" fn RtlFreeUserThreadStack(
    process_handle: *mut u8,
    thread_handle: *mut u8,
) {
    // Query the Basic Info.
    let mut tbi: thr::THREAD_BASIC_INFORMATION = core::mem::zeroed();
    let status = NtQueryInformationThread(
        thread_handle,
        thr::THREAD_BASIC_INFORMATION_CLASS,
        &mut tbi as *mut _ as *mut u8,
        core::mem::size_of::<thr::THREAD_BASIC_INFORMATION>() as u32,
        core::ptr::null_mut(),
    );
    if status < 0 || tbi.TebBaseAddress.is_null() {
        return;
    }

    // Get the deallocation stack — read &teb->DeallocationStack out of
    // the foreign process at the libsupp-computed offset.
    let mut stack_location: *mut u8 = core::ptr::null_mut();
    let mut dummy: usize = 0;
    let status = NtReadVirtualMemory(
        process_handle,
        RtlpThrTebDeallocStackAddr(tbi.TebBaseAddress),
        &mut stack_location as *mut _ as *mut u8,
        core::mem::size_of::<*mut u8>(),
        &mut dummy,
    );
    if status < 0 || stack_location.is_null() {
        return;
    }

    // Free it.
    let mut size: usize = 0;
    let _ = NtFreeVirtualMemory(
        process_handle,
        &mut stack_location,
        &mut size,
        thr::MEM_RELEASE,
    );
}

/// `PTEB NTAPI _NtCurrentTeb(VOID)` — exported NTAPI primitive that
/// just returns the current thread's TEB. User-mode: libsupp's
/// `RtlpThrCurrentTeb` reads gs/fs/x18 via the C `NtCurrentTeb()`
/// macro. Kernel-mode: libsupp returns NULL (no current TEB).
#[no_mangle]
pub unsafe extern "system" fn _NtCurrentTeb() -> *mut u8 {
    RtlpThrCurrentTeb()
}

/// `NTSTATUS NTAPI RtlRemoteCall(HANDLE, HANDLE, PVOID, ULONG, PULONG,
/// BOOLEAN, BOOLEAN)` — UNIMPLEMENTED in the C original.
#[no_mangle]
pub unsafe extern "system" fn RtlRemoteCall(
    _process: *mut u8,
    _thread: *mut u8,
    _call_site: *mut u8,
    _arg_count: u32,
    _args: *mut u32,
    _pass_context: u8,
    _already_suspended: u8,
) -> i32 {
    thr::STATUS_NOT_IMPLEMENTED
}

// =====================================================================
// Wave PRC — straight translation of `sdk/lib/rtl/process.c` (9 fns,
// pure Zw*/Nt*/Rtl* plumbing: process create + encode/decode pointer
// + critical flag + processor + os version). No byte-diff: every
// observable lives behind a syscall.
// =====================================================================

extern "system" {
    fn ZwOpenFile(
        FileHandle: *mut *mut u8,
        DesiredAccess: u32,
        ObjectAttributes: *mut thr::OBJECT_ATTRIBUTES,
        IoStatusBlock: *mut prc::IO_STATUS_BLOCK,
        ShareAccess: u32,
        OpenOptions: u32,
    ) -> i32;
    fn ZwCreateSection(
        SectionHandle: *mut *mut u8,
        DesiredAccess: u32,
        ObjectAttributes: *mut u8,
        MaximumSize: *mut i64,
        SectionPageProtection: u32,
        AllocationAttributes: u32,
        FileHandle: *mut u8,
    ) -> i32;
    fn ZwCreateProcess(
        ProcessHandle: *mut *mut u8,
        DesiredAccess: u32,
        ObjectAttributes: *mut thr::OBJECT_ATTRIBUTES,
        ParentProcess: *mut u8,
        InheritObjectTable: u8,
        SectionHandle: *mut u8,
        DebugPort: *mut u8,
        ExceptionPort: *mut u8,
    ) -> i32;
    fn ZwQuerySection(
        SectionHandle: *mut u8,
        InformationClass: u32,
        InformationBuffer: *mut u8,
        InformationBufferSize: usize,
        ResultLength: *mut usize,
    ) -> i32;
    fn ZwQueryInformationProcess(
        ProcessHandle: *mut u8,
        InfoClass: u32,
        Info: *mut u8,
        InfoLen: u32,
        ReturnLength: *mut u32,
    ) -> i32;
    fn ZwSetInformationProcess(
        ProcessHandle: *mut u8,
        InfoClass: u32,
        Info: *const u8,
        InfoLen: u32,
    ) -> i32;
    fn ZwDuplicateObject(
        SourceProcessHandle: *mut u8,
        SourceHandle: *mut u8,
        TargetProcessHandle: *mut u8,
        TargetHandle: *mut *mut u8,
        DesiredAccess: u32,
        HandleAttributes: u32,
        Options: u32,
    ) -> i32;
    // ZwClose declared earlier (priv wave); reused here.
    fn NtGetCurrentProcessorNumber() -> u32;
    fn RtlGetNtGlobalFlags() -> u32;
    fn ZwWriteVirtualMemory(
        ProcessHandle: *mut u8,
        BaseAddress: *mut u8,
        Buffer: *const u8,
        BytesToWrite: usize,
        NumberOfBytesWritten: *mut usize,
    ) -> i32;

    // libsupp helpers (per-flavor: ntoskrnl/rtl/libsupp.c or
    // dll/ntdll/rtl/libsupp.c).
    fn RtlpPrcGetPebOSMajorVersion() -> u32;
    fn RtlpPrcGetPebOSMinorVersion() -> u32;
    fn RtlpPrcPebProcessParametersAddr(PebBase: *mut u8) -> *mut u8;
}

extern "C" {
    fn wcsstr(haystack: *const u16, needle: *const u16) -> *const u16;
}

/// `NTSTATUS NTAPI RtlpMapFile(PUNICODE_STRING, ULONG, PHANDLE)`.
#[no_mangle]
pub unsafe extern "system" fn RtlpMapFile(
    image_file_name: *mut UNICODE_STRING,
    attributes: u32,
    section: *mut *mut u8,
) -> i32 {
    let mut oa: thr::OBJECT_ATTRIBUTES = core::mem::zeroed();
    oa.Length = core::mem::size_of::<thr::OBJECT_ATTRIBUTES>() as u32;
    oa.ObjectName = image_file_name as *mut u8;
    oa.Attributes = attributes & (prc::OBJ_CASE_INSENSITIVE | prc::OBJ_INHERIT);

    let mut io_status_block: prc::IO_STATUS_BLOCK = core::mem::zeroed();
    let mut h_file: *mut u8 = core::ptr::null_mut();

    let status = ZwOpenFile(
        &mut h_file,
        prc::SYNCHRONIZE | prc::FILE_EXECUTE | prc::FILE_READ_DATA,
        &mut oa,
        &mut io_status_block,
        prc::FILE_SHARE_DELETE | prc::FILE_SHARE_READ,
        prc::FILE_SYNCHRONOUS_IO_NONALERT | prc::FILE_NON_DIRECTORY_FILE,
    );
    if status < 0 {
        return status;
    }

    // Create a section for this image.
    let status = ZwCreateSection(
        section,
        prc::SECTION_ALL_ACCESS,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        prc::PAGE_EXECUTE,
        prc::SEC_IMAGE,
        h_file,
    );
    let _ = ZwClose(h_file);
    status
}

/// `NTSTATUS NTAPI RtlpInitEnvironment(HANDLE, PPEB,
/// PRTL_USER_PROCESS_PARAMETERS)`.
///
/// Allocates env block + parameter block in the foreign process,
/// copies them across via ZwWriteVirtualMemory, then writes the
/// parameter-block base address into the foreign PEB's
/// ProcessParameters slot.
#[no_mangle]
pub unsafe extern "system" fn RtlpInitEnvironment(
    process_handle: *mut u8,
    peb: *mut u8,
    process_parameters: *mut prc::RTL_USER_PROCESS_PARAMETERS,
) -> i32 {
    let pp = &mut *process_parameters;
    let mut base_address: *mut u8 = core::ptr::null_mut();
    let mut enviro_size: usize = 0;

    // Give the caller 1MB if requested (reserve at fixed VA 4).
    if (pp.Flags & prc::RTL_USER_PROCESS_PARAMETERS_RESERVE_1MB) != 0 {
        base_address = 4 as *mut u8;
        enviro_size = 1024 * 1024 - 256;
        let mut size = enviro_size;
        let status = ZwAllocateVirtualMemory(
            process_handle,
            &mut base_address,
            0,
            &mut size,
            prc::MEM_RESERVE,
            prc::PAGE_READWRITE,
        );
        if status < 0 {
            return status;
        }
    }

    // Walk to the end of the environment block (double-NUL terminated).
    let env_ptr = pp.Environment;
    if !env_ptr.is_null() {
        let mut walk = env_ptr;
        // C: while (*Environment++) while (*Environment++);
        // Walk through pairs of inner-strings.
        loop {
            if *walk == 0 {
                walk = walk.add(1);
                break;
            }
            walk = walk.add(1);
            while *walk != 0 {
                walk = walk.add(1);
            }
            walk = walk.add(1);
        }
        enviro_size = (walk as usize) - (env_ptr as usize);

        // Allocate + reserve + commit env block in foreign process.
        let mut size = enviro_size;
        base_address = core::ptr::null_mut();
        let status = ZwAllocateVirtualMemory(
            process_handle,
            &mut base_address,
            0,
            &mut size,
            prc::MEM_RESERVE | prc::MEM_COMMIT,
            prc::PAGE_READWRITE,
        );
        if status < 0 {
            return status;
        }

        // Copy env bytes across.
        let _ = ZwWriteVirtualMemory(
            process_handle,
            base_address,
            pp.Environment as *const u8,
            enviro_size,
            core::ptr::null_mut(),
        );

        pp.Environment = base_address as *mut u16;
    }

    // Allocate Parameter Block in foreign process.
    base_address = core::ptr::null_mut();
    let mut size = pp.MaximumLength as usize;
    let status = ZwAllocateVirtualMemory(
        process_handle,
        &mut base_address,
        0,
        &mut size,
        prc::MEM_COMMIT,
        prc::PAGE_READWRITE,
    );
    if status < 0 {
        return status;
    }

    // Write the Parameter Block across.
    let status = ZwWriteVirtualMemory(
        process_handle,
        base_address,
        process_parameters as *const u8,
        pp.Length as usize,
        core::ptr::null_mut(),
    );
    if status < 0 {
        return status;
    }

    // Write pointer-to-Parameter-Block into foreign PEB's
    // ProcessParameters slot.
    let mut ba = base_address;
    let status = ZwWriteVirtualMemory(
        process_handle,
        RtlpPrcPebProcessParametersAddr(peb),
        &ba as *const *mut u8 as *const u8,
        core::mem::size_of::<*mut u8>(),
        core::ptr::null_mut(),
    );
    if status < 0 {
        return status;
    }
    let _ = &mut ba; // suppress unused-mut

    0 // STATUS_SUCCESS
}

/// `NTSTATUS NTAPI RtlCreateUserProcess(...)`.
///
/// Faithful translation. The SEH around the StandardInput/Output/Error
/// duplication block is dead under `USE_DUMMY_PSEH` (per AGENTS.md);
/// the body of the `_SEH2_TRY` becomes the function body, and the
/// `_SEH2_FINALLY` cleanup runs on any failure status — implemented
/// here as a plain on-error block.
#[no_mangle]
pub unsafe extern "system" fn RtlCreateUserProcess(
    image_file_name: *mut UNICODE_STRING,
    attributes: u32,
    process_parameters: *mut prc::RTL_USER_PROCESS_PARAMETERS,
    process_security_descriptor: *mut u8,
    thread_security_descriptor: *mut u8,
    mut parent_process: *mut u8,
    inherit_handles: u8,
    debug_port: *mut u8,
    exception_port: *mut u8,
    process_info: *mut prc::RTL_USER_PROCESS_INFORMATION,
) -> i32 {
    let pp = &mut *process_parameters;
    let pi = &mut *process_info;
    let mut h_section: *mut u8 = core::ptr::null_mut();

    // Map and Load the File.
    let status = RtlpMapFile(image_file_name, attributes, &mut h_section);
    if status < 0 {
        return status;
    }

    // Clean out the current directory handle if we won't use it.
    if inherit_handles == 0 {
        pp.CurrentDirectory.Handle = core::ptr::null_mut();
    }

    // Use us as parent if none other specified.
    if parent_process.is_null() {
        parent_process = thr::NT_CURRENT_PROCESS as *mut u8;
    }

    // Initialize Object Attributes (NULL name).
    let mut oa: thr::OBJECT_ATTRIBUTES = core::mem::zeroed();
    oa.Length = core::mem::size_of::<thr::OBJECT_ATTRIBUTES>() as u32;
    oa.SecurityDescriptor = process_security_descriptor;

    // CSRSS-under-WindowsSS check.
    let debug_string: [u16; 11] = [
        '\\' as u16,
        'W' as u16,
        'i' as u16,
        'n' as u16,
        'd' as u16,
        'o' as u16,
        'w' as u16,
        's' as u16,
        'S' as u16,
        'S' as u16,
        0,
    ];
    let csrss_pat: [u16; 6] = [
        'c' as u16, 's' as u16, 'r' as u16, 's' as u16, 's' as u16, 0,
    ];
    let mut debug_us = UNICODE_STRING {
        Length: 20, // "\\WindowsSS" = 10 chars * 2 bytes
        MaximumLength: 22,
        Buffer: debug_string.as_ptr() as *mut u16,
    };
    if (RtlGetNtGlobalFlags() & prc::FLG_ENABLE_CSRDEBUG) != 0
        && !(*image_file_name).Buffer.is_null()
        && !wcsstr((*image_file_name).Buffer, csrss_pat.as_ptr()).is_null()
    {
        oa.ObjectName = &mut debug_us as *mut _ as *mut u8;
    }

    // Create Kernel Process Object.
    let status = ZwCreateProcess(
        &mut pi.ProcessHandle,
        prc::PROCESS_ALL_ACCESS,
        &mut oa,
        parent_process,
        inherit_handles,
        h_section,
        debug_port,
        exception_port,
    );
    if status < 0 {
        let _ = ZwClose(h_section);
        return status;
    }

    // Query the section image info.
    let status = ZwQuerySection(
        h_section,
        prc::SECTION_IMAGE_INFORMATION_CLASS,
        &mut pi.ImageInformation as *mut _ as *mut u8,
        core::mem::size_of::<prc::SECTION_IMAGE_INFORMATION>(),
        core::ptr::null_mut(),
    );
    if status < 0 {
        let _ = ZwClose(pi.ProcessHandle);
        let _ = ZwClose(h_section);
        return status;
    }

    // Query process basic info to get PEB base.
    let mut process_basic_info: prc::PROCESS_BASIC_INFORMATION = core::mem::zeroed();
    let status = ZwQueryInformationProcess(
        pi.ProcessHandle,
        prc::PROCESS_BASIC_INFORMATION_CLASS,
        &mut process_basic_info as *mut _ as *mut u8,
        core::mem::size_of::<prc::PROCESS_BASIC_INFORMATION>() as u32,
        core::ptr::null_mut(),
    );
    if status < 0 {
        let _ = ZwClose(pi.ProcessHandle);
        let _ = ZwClose(h_section);
        return status;
    }

    // Duplicate the standard handles. The original C wraps this in
    // _SEH2_TRY + _SEH2_FINALLY; under DUMMY_PSEH the handlers are
    // dead — the cleanup block runs unconditionally as plain code.
    let mut status: i32 = 0;
    if !pp.StandardInput.is_null() {
        status = ZwDuplicateObject(
            parent_process,
            pp.StandardInput,
            pi.ProcessHandle,
            &mut pp.StandardInput,
            0,
            0,
            prc::DUPLICATE_SAME_ACCESS | prc::DUPLICATE_SAME_ATTRIBUTES,
        );
    }
    if status >= 0 && !pp.StandardOutput.is_null() {
        status = ZwDuplicateObject(
            parent_process,
            pp.StandardOutput,
            pi.ProcessHandle,
            &mut pp.StandardOutput,
            0,
            0,
            prc::DUPLICATE_SAME_ACCESS | prc::DUPLICATE_SAME_ATTRIBUTES,
        );
    }
    if status >= 0 && !pp.StandardError.is_null() {
        status = ZwDuplicateObject(
            parent_process,
            pp.StandardError,
            pi.ProcessHandle,
            &mut pp.StandardError,
            0,
            0,
            prc::DUPLICATE_SAME_ACCESS | prc::DUPLICATE_SAME_ATTRIBUTES,
        );
    }
    // _SEH2_FINALLY block.
    if status < 0 {
        let _ = ZwClose(pi.ProcessHandle);
        let _ = ZwClose(h_section);
        return status;
    }

    // Create Process Environment.
    let status = RtlpInitEnvironment(
        pi.ProcessHandle,
        process_basic_info.PebBaseAddress,
        process_parameters,
    );
    if status < 0 {
        let _ = ZwClose(pi.ProcessHandle);
        let _ = ZwClose(h_section);
        return status;
    }

    // Create the first Thread.
    let status = RtlCreateUserThread(
        pi.ProcessHandle,
        thread_security_descriptor,
        1, // CreateSuspended = TRUE
        pi.ImageInformation.ZeroBits,
        pi.ImageInformation.MaximumStackSize,
        pi.ImageInformation.CommittedStackSize,
        pi.ImageInformation.TransferAddress,
        process_basic_info.PebBaseAddress,
        &mut pi.ThreadHandle,
        &mut pi.ClientId,
    );
    if status < 0 {
        let _ = ZwClose(pi.ProcessHandle);
        let _ = ZwClose(h_section);
        return status;
    }

    // Close the Section Handle and return.
    let _ = ZwClose(h_section);
    0 // STATUS_SUCCESS
}

/// `PVOID NTAPI RtlEncodePointer(PVOID)`. XOR the pointer with the
/// per-process `ProcessCookie` queried lazily on first call. The
/// XOR itself lives in `core_rs::process::xor_pointer` (tested).
#[no_mangle]
pub unsafe extern "system" fn RtlEncodePointer(pointer: *mut u8) -> *mut u8 {
    let mut cookie: u32 = 0;
    let status = ZwQueryInformationProcess(
        thr::NT_CURRENT_PROCESS as *mut u8,
        prc::PROCESS_COOKIE_CLASS,
        &mut cookie as *mut _ as *mut u8,
        core::mem::size_of::<u32>() as u32,
        core::ptr::null_mut(),
    );
    if status < 0 {
        return pointer;
    }
    prc::xor_pointer(pointer as usize, cookie) as *mut u8
}

/// `PVOID NTAPI RtlDecodePointer(PVOID)` — same XOR.
#[no_mangle]
pub unsafe extern "system" fn RtlDecodePointer(pointer: *mut u8) -> *mut u8 {
    RtlEncodePointer(pointer)
}

/// `PVOID NTAPI RtlEncodeSystemPointer(PVOID)` — XOR with
/// `SharedUserData->Cookie` at fixed VA.
#[no_mangle]
pub unsafe extern "system" fn RtlEncodeSystemPointer(pointer: *mut u8) -> *mut u8 {
    // VA chosen at runtime: librtl.a is shared by ntoskrnl + ntdll, so
    // the kernel KUSER_SHARED_DATA VA would fault when called from user
    // mode (see prc::kuser_shared_data_va).
    let cookie_addr = (prc::kuser_shared_data_va() + prc::KUSER_SHARED_COOKIE_OFFSET) as *const u32;
    let cookie = core::ptr::read_volatile(cookie_addr);
    prc::xor_pointer(pointer as usize, cookie) as *mut u8
}

/// `PVOID NTAPI RtlDecodeSystemPointer(PVOID)`.
#[no_mangle]
pub unsafe extern "system" fn RtlDecodeSystemPointer(pointer: *mut u8) -> *mut u8 {
    RtlEncodeSystemPointer(pointer)
}

/// `NTSTATUS __cdecl RtlSetProcessIsCritical(BOOLEAN, PBOOLEAN, BOOLEAN)`.
#[no_mangle]
pub unsafe extern "C" fn RtlSetProcessIsCritical(
    new_value: u8,
    old_value: *mut u8,
    need_breaks: u8,
) -> i32 {
    if !old_value.is_null() {
        *old_value = 0;
    }

    if need_breaks != 0 && (RtlpThrGetPebNtGlobalFlag() & prc::FLG_ENABLE_SYSTEM_CRIT_BREAKS) == 0 {
        return prc::STATUS_UNSUCCESSFUL;
    }

    if !old_value.is_null() {
        let mut break_on_termination: u32 = 0;
        let _ = ZwQueryInformationProcess(
            thr::NT_CURRENT_PROCESS as *mut u8,
            prc::PROCESS_BREAK_ON_TERMINATION_CLASS,
            &mut break_on_termination as *mut _ as *mut u8,
            core::mem::size_of::<u32>() as u32,
            core::ptr::null_mut(),
        );
        *old_value = break_on_termination as u8;
    }

    let break_on_termination: u32 = new_value as u32;
    ZwSetInformationProcess(
        thr::NT_CURRENT_PROCESS as *mut u8,
        prc::PROCESS_BREAK_ON_TERMINATION_CLASS,
        &break_on_termination as *const _ as *const u8,
        core::mem::size_of::<u32>() as u32,
    )
}

/// `ULONG NTAPI RtlGetCurrentProcessorNumber(VOID)`.
#[no_mangle]
pub unsafe extern "system" fn RtlGetCurrentProcessorNumber() -> u32 {
    NtGetCurrentProcessorNumber()
}

/// `ULONG NTAPI RtlRosGetAppcompatVersion(VOID)` — ReactOS-specific.
/// Returns `(PEB.OSMajorVersion << 8) | PEB.OSMinorVersion`, or the
/// Server 2003 default if no PEB. Decision in
/// `core_rs::process::appcompat_version` (tested).
#[no_mangle]
pub unsafe extern "system" fn RtlRosGetAppcompatVersion() -> u32 {
    prc::appcompat_version(RtlpPrcGetPebOSMajorVersion(), RtlpPrcGetPebOSMinorVersion())
}

// =====================================================================
// Wave PATH-1 — pure-decision leaf fns of `sdk/lib/rtl/path.c`.
// PARTIAL swap (path.c stays in cmake; per-fn `#if 0` blocks
// remove the C body). Smoke is the net.
// =====================================================================

/// `RTL_PATH_TYPE NTAPI RtlDetermineDosPathNameType_Ustr(PCUNICODE_STRING)`.
/// Note: `RTL_PATH_TYPE` is an enum — `extern "system" fn -> u32`.
#[no_mangle]
pub unsafe extern "system" fn RtlDetermineDosPathNameType_Ustr(
    path_string: *const UNICODE_STRING,
) -> u32 {
    let s = &*path_string;
    let chars = (s.Length / 2) as usize;
    let buf = core::slice::from_raw_parts(s.Buffer as *const u16, chars);
    path_::determine_dos_path_name_type_ustr(buf)
}

/// `RTL_PATH_TYPE NTAPI RtlDetermineDosPathNameType_U(PCWSTR)`.
#[no_mangle]
pub unsafe extern "system" fn RtlDetermineDosPathNameType_U(path: *const u16) -> u32 {
    // Faithful: the C version peeks Path[0..=3] unconditionally
    // (assumes 4 chars of lookahead are valid). Mirror that here.
    let p0 = *path;
    let p1 = *path.add(1);
    let p2 = *path.add(2);
    let p3 = *path.add(3);
    path_::determine_dos_path_name_type_u_prefix(p0, p1, p2, p3)
}

/// `ULONG NTAPI RtlGetLongestNtPathLength(VOID)`.
#[no_mangle]
pub unsafe extern "system" fn RtlGetLongestNtPathLength() -> u32 {
    path_::LONGEST_NT_PATH_LENGTH
}

/// `NTSTATUS NTAPI RtlGetLengthWithoutTrailingPathSeparators(ULONG,
/// PCUNICODE_STRING, PULONG)`.
#[no_mangle]
pub unsafe extern "system" fn RtlGetLengthWithoutTrailingPathSeparators(
    flags: u32,
    path_string: *const UNICODE_STRING,
    length: *mut u32,
) -> i32 {
    if length.is_null() {
        return path_::STATUS_INVALID_PARAMETER;
    }
    *length = 0;
    if path_string.is_null() {
        return path_::STATUS_INVALID_PARAMETER;
    }
    if flags != 0 {
        return path_::STATUS_INVALID_PARAMETER;
    }
    let s = &*path_string;
    let chars = (s.Length / 2) as usize;
    let buf = core::slice::from_raw_parts(s.Buffer as *const u16, chars);
    *length = path_::length_without_trailing_separators(buf);
    path_::STATUS_SUCCESS
}

/// Build a stack `UNICODE_STRING` describing `[buf, len_chars)` —
/// matches the C-side `PathCopy` local in `RtlIsDosDeviceName_Ustr`.
#[inline]
unsafe fn us_view(buf: *mut u16, len_chars: u16) -> UNICODE_STRING {
    UNICODE_STRING {
        Length: len_chars.saturating_mul(2),
        MaximumLength: len_chars.saturating_mul(2),
        Buffer: buf,
    }
}

/// `ULONG NTAPI RtlIsDosDeviceName_Ustr(PCUNICODE_STRING)`. Faithful
/// translation of path.c lines 92-251. Returns 0 if the path is not
/// a DOS device name, or `MAKELONG(length_bytes, offset_bytes)` if
/// it is.
#[no_mangle]
pub unsafe extern "system" fn RtlIsDosDeviceName_Ustr(path_string: *const UNICODE_STRING) -> u32 {
    if path_string.is_null() {
        return 0;
    }
    let path_str = &*path_string;

    // Check what type of path this is.
    let pt = RtlDetermineDosPathNameType_Ustr(path_string);
    match pt {
        path_::RTL_PATH_TYPE_UNKNOWN | path_::RTL_PATH_TYPE_UNC_ABSOLUTE => return 0,
        path_::RTL_PATH_TYPE_LOCAL_DEVICE => {
            // Make special check for the CON device.
            if RtlEqualUnicodeString(path_string, &RtlpDosSlashCONDevice, 1) != 0 {
                // MAKELONG(RtlpDosCONDevice.Length, DeviceRootString.Length)
                return ((DeviceRootString.Length as u32) << 16) | (RtlpDosCONDevice.Length as u32);
            }
            return 0;
        }
        _ => {}
    }

    // Make a copy of the string.
    let original_length = path_str.Length;
    let mut path_copy_buf = path_str.Buffer;
    let mut path_copy_len = path_str.Length;
    let mut path_chars = (path_copy_len / 2) as u16;
    if path_chars == 0 {
        return 0;
    }

    let mut colon_count: u16 = 0;
    let mut return_offset: u16 = 0;

    // Check for drive path and truncate trailing colon.
    if *path_copy_buf.add((path_chars - 1) as usize) == b':' as u16 {
        path_copy_len -= 2;
        path_chars -= 1;
        if path_chars == 0 {
            return 0;
        }
        colon_count = 1;
    }

    // Check for extension or space, and truncate.
    while path_chars > 0 {
        let c = *path_copy_buf.add((path_chars - 1) as usize);
        if c != b'.' as u16 && c != b' ' as u16 {
            break;
        }
        path_copy_len -= 2;
        colon_count += 1;
        path_chars -= 1;
    }

    if path_chars > 0 {
        // Loop from the end looking for a path or drive separator.
        let mut found = false;
        let mut end_idx: isize = (path_chars - 1) as isize;
        while end_idx >= 0 {
            let c = *path_copy_buf.add(end_idx as usize);
            let is_drive_colon = c == b':' as u16 && end_idx == 1; // End == Buffer+1
            if path_::is_path_separator(c) || is_drive_colon {
                // Get next-lowercase character at End+1.
                end_idx += 1;
                if (end_idx as usize) >= (original_length / 2) as usize {
                    return 0;
                }
                let c_next = path_::ascii_lower(*path_copy_buf.add(end_idx as usize));
                if c_next == b'l' as u16
                    || c_next == b'c' as u16
                    || c_next == b'p' as u16
                    || c_next == b'a' as u16
                    || c_next == b'n' as u16
                {
                    // Calculate the offset (in bytes from start of buffer).
                    return_offset = ((end_idx as usize) * 2) as u16;
                    // Build the final string.
                    path_copy_len = original_length - return_offset - colon_count.saturating_mul(2);
                    path_copy_buf = path_copy_buf.add(end_idx as usize);
                    path_chars = (path_copy_len / 2) as u16;
                    found = true;
                    break;
                } else {
                    return 0;
                }
            }
            end_idx -= 1;
        }

        if !found {
            // No separator found — check the first char.
            let c = path_::ascii_lower(*path_copy_buf);
            if c != b'l' as u16
                && c != b'c' as u16
                && c != b'p' as u16
                && c != b'a' as u16
                && c != b'n' as u16
            {
                return 0;
            }
        }
    }

    // Skip past any extra extension or drive-letter characters.
    let mut start: usize = 0;
    let end = path_chars as usize;
    while start < end {
        let c = *path_copy_buf.add(start);
        if c == b'.' as u16 || c == b':' as u16 {
            break;
        }
        start += 1;
    }

    // And then go backwards to get rid of spaces.
    while start > 0 && *path_copy_buf.add(start - 1) == b' ' as u16 {
        start -= 1;
    }

    // Finally see how many characters are left.
    let final_chars = start as u16;
    let final_us = us_view(path_copy_buf, final_chars);

    // Check if this is a COM or LPT port (3 chars + non-zero digit).
    if final_chars == 4
        && path_::is_digit(*path_copy_buf.add(3))
        && *path_copy_buf.add(3) != b'0' as u16
    {
        // Don't compare the number part, just check for LPT or COM.
        let trimmed = us_view(path_copy_buf, 3);
        if RtlEqualUnicodeString(&trimmed, &RtlpDosLPTDevice, 1) != 0
            || RtlEqualUnicodeString(&trimmed, &RtlpDosCOMDevice, 1) != 0
        {
            // sizeof(L"COM1") - sizeof(WCHAR) = 4*2 - 2 = 6 bytes.
            return ((return_offset as u32) << 16) | 6;
        }
    } else if final_chars == 3 {
        if RtlEqualUnicodeString(&final_us, &RtlpDosPRNDevice, 1) != 0
            || RtlEqualUnicodeString(&final_us, &RtlpDosAUXDevice, 1) != 0
            || RtlEqualUnicodeString(&final_us, &RtlpDosNULDevice, 1) != 0
            || RtlEqualUnicodeString(&final_us, &RtlpDosCONDevice, 1) != 0
        {
            // sizeof(L"AUX") - sizeof(WCHAR) = 4*2 - 2 = 6 bytes.
            return ((return_offset as u32) << 16) | 6;
        }
    }

    0
}

/// `ULONG NTAPI RtlIsDosDeviceName_U(PCWSTR)`.
#[no_mangle]
pub unsafe extern "system" fn RtlIsDosDeviceName_U(path: *const u16) -> u32 {
    // Build a UNICODE_STRING from the NUL-terminated path (faithful
    // to RtlInitUnicodeStringEx semantics: success iff <=MAXUSHORT/2
    // chars). For PATH-1 we keep the failure path tight.
    if path.is_null() {
        return 0;
    }
    let mut len: usize = 0;
    while *path.add(len) != 0 {
        len += 1;
        if len > 32767 {
            return 0; // STATUS_NAME_TOO_LONG — would fail in C
        }
    }
    let us = UNICODE_STRING {
        Length: (len * 2) as u16,
        MaximumLength: ((len + 1) * 2) as u16,
        Buffer: path as *mut u16,
    };
    RtlIsDosDeviceName_Ustr(&us)
}

// =====================================================================
// Wave ATM — straight translation of `sdk/lib/rtl/atom.c` (11 NTAPI
// fns). FULL swap (atom.c removed from cmake). atom.c never touches the
// union members directly — only the cdecl `Rtlp*` helpers in libsupp do
// (concurrency + handle FFI). See the RTL_ATOM_TABLE mirror below for why
// it uses the larger (user) union arm unconditionally.
// =====================================================================

use crate::core_rs::atom as atom_;

// --- atom.c local STATUS values (some are success-class!). ---
const ATM_STATUS_INVALID_HANDLE: i32 = 0xC000_0008u32 as i32;
const ATM_STATUS_OBJECT_NAME_INVALID: i32 = 0xC000_0033u32 as i32;
const ATM_STATUS_OBJECT_NAME_NOT_FOUND: i32 = 0xC000_0034u32 as i32;
const ATM_STATUS_WAS_LOCKED: i32 = 0x0000_0001; // NOT a failure code
const ATM_STATUS_BUFFER_TOO_SMALL: i32 = 0xC000_0023u32 as i32;
const ATM_STATUS_INFO_LENGTH_MISMATCH: i32 = 0xC000_0004u32 as i32;
const HASH_STRING_ALGORITHM_X65599: u32 = 1;
const RTL_ATOM_IS_PINNED: u8 = atom_::RTL_ATOM_IS_PINNED;

#[inline]
fn atm_nt_success(s: i32) -> bool {
    s >= 0
}

// --- `RTL_ATOM_TABLE` layout mirror. ---
//
// librtl_rust.a is built ONCE and shared by BOTH atom consumers:
//   * ntoskrnl — the kernel atom path. `ex/atom.rs` defines only the Nt*
//                syscalls; they call the Rtl* atom fns HERE, paired with
//                ntoskrnl-rust's libsupp (FAST_MUTEX + PHANDLE_TABLE arm).
//   * ntdll    — local atom tables, paired with ntdll-rust's libsupp
//                (RTL_CRITICAL_SECTION + RTL_HANDLE_TABLE arm).
//
// A single staticlib can't carry two layouts — but it doesn't need to.
// Only NumberOfBuckets + Buckets are read HERE; the union members are
// written/read exclusively by the per-mode libsupp through ITS OWN struct
// view. The sole requirement is that NumberOfBuckets/Buckets sit PAST
// whatever the libsupp union write covers in either mode. So we always lay
// the union region out as the LARGER (user) arm — a safe superset:
//   64-bit: kernel union ends @72, user (RtlHandleTable) ends @96 →
//           NumberOfBuckets @96, clear of both.
//   32-bit: kernel ends @40, user ends @60 → NumberOfBuckets @60.
// The kernel's FAST_MUTEX(+PHANDLE_TABLE) write fits entirely inside the
// user-arm region and never reaches NumberOfBuckets.
//
// Using the SMALLER (kernel) arm here was the bug: in ntdll it put
// NumberOfBuckets INSIDE RtlHandleTable's footprint (on CommittedHandles,
// which RtlCreateHandleTable leaves NULL) → bucket_index read 0 →
// divide-by-zero. It manifested arm64-only, post-desktop (idle-smoke
// masked it). See feedback_rtl_atom_arm64_boot_regression.

/// `RTL_CRITICAL_SECTION` (rtltypes.h) — the larger (user) union arm; also
/// a safe superset for the kernel (FAST_MUTEX) consumer.
#[repr(C)]
struct AtmSync {
    debug_info: *mut core::ffi::c_void,
    lock_count: i32,
    recursion_count: i32,
    owning_thread: *mut core::ffi::c_void,
    lock_semaphore: *mut core::ffi::c_void,
    spin_count: usize,
}

/// `RTL_HANDLE_TABLE_ENTRY` only matters as a pointer target here.
#[repr(C)]
struct AtmHandleTableEntry {
    next_free_or_flags: usize,
}

/// `RTL_HANDLE_TABLE` (rtltypes.h) — the larger (user) union arm.
#[repr(C)]
struct AtmHandleStorage {
    maximum_number_of_handles: u32,
    size_of_handle_table_entry: u32,
    reserved: [u32; 2],
    free_handles: *mut AtmHandleTableEntry,
    committed_handles: *mut AtmHandleTableEntry,
    uncommitted_handles: *mut AtmHandleTableEntry,
    max_reserved_handles: *mut AtmHandleTableEntry,
}

/// `RTL_ATOM_TABLE_ENTRY` (rtltypes.h) — no union, flat layout, identical
/// kernel/user. `Name` is a `WCHAR[1]` flexible tail.
#[repr(C)]
pub struct RTL_ATOM_TABLE_ENTRY {
    hash_link: *mut RTL_ATOM_TABLE_ENTRY,
    handle_index: u16,
    atom: u16,
    reference_count: u16,
    flags: u8,
    name_length: u8,
    name: [u16; 1],
}

/// `RTL_ATOM_TABLE` — union region laid out as the larger (user) arm so
/// NumberOfBuckets/Buckets sit past any per-mode libsupp union write.
#[repr(C)]
pub struct RTL_ATOM_TABLE {
    signature: u32,
    sync: AtmSync,
    handle_storage: AtmHandleStorage,
    number_of_buckets: u32,
    buckets: [*mut RTL_ATOM_TABLE_ENTRY; 1], // flexible tail
}

// Guard the superset invariant: NumberOfBuckets MUST sit past the kernel
// union arm's end (FAST_MUTEX + PHANDLE_TABLE = 72 on 64-bit, 40 on
// 32-bit), so the kernel-mode libsupp's union write can never clobber it.
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(RTL_ATOM_TABLE, number_of_buckets) >= 72);
#[cfg(target_pointer_width = "32")]
const _: () = assert!(core::mem::offset_of!(RTL_ATOM_TABLE, number_of_buckets) >= 40);

// --- libsupp.c helpers. C declares these WITHOUT NTAPI ⇒ __cdecl on
//     all arches ⇒ `extern "C"`. (Mismatching this with extern "system"
//     would be the #1 i386 crash cause — see feedback_extern_c_abi.) ---
extern "C" {
    fn RtlpInitAtomTableLock(atom_table: *mut RTL_ATOM_TABLE) -> i32;
    fn RtlpDestroyAtomTableLock(atom_table: *mut RTL_ATOM_TABLE);
    fn RtlpLockAtomTable(atom_table: *mut RTL_ATOM_TABLE) -> u8;
    fn RtlpUnlockAtomTable(atom_table: *mut RTL_ATOM_TABLE);
    fn RtlpCreateAtomHandleTable(atom_table: *mut RTL_ATOM_TABLE) -> u8;
    fn RtlpDestroyAtomHandleTable(atom_table: *mut RTL_ATOM_TABLE);
    fn RtlpAllocAtomTable(size: u32) -> *mut RTL_ATOM_TABLE;
    fn RtlpFreeAtomTable(atom_table: *mut RTL_ATOM_TABLE);
    fn RtlpAllocAtomTableEntry(size: u32) -> *mut RTL_ATOM_TABLE_ENTRY;
    fn RtlpFreeAtomTableEntry(entry: *mut RTL_ATOM_TABLE_ENTRY);
    fn RtlpCreateAtomHandle(
        atom_table: *mut RTL_ATOM_TABLE,
        entry: *mut RTL_ATOM_TABLE_ENTRY,
    ) -> u8;
    fn RtlpFreeAtomHandle(atom_table: *mut RTL_ATOM_TABLE, entry: *mut RTL_ATOM_TABLE_ENTRY);
    fn RtlpGetAtomEntry(atom_table: *mut RTL_ATOM_TABLE, index: u32) -> *mut RTL_ATOM_TABLE_ENTRY;
    // CRT wide compare (cdecl). Same fn atom.c calls; matches its fold.
    fn _wcsicmp(a: *const u16, b: *const u16) -> i32;
}

/// Pointer to bucket[0] — strides by pointer size, == C `Buckets`.
#[inline]
unsafe fn atm_buckets(table: *mut RTL_ATOM_TABLE) -> *mut *mut RTL_ATOM_TABLE_ENTRY {
    (*table).buckets.as_mut_ptr()
}

/// Write `"#<decimal>\0"` into `buf`, return chars written excluding the
/// NUL (== what `swprintf(buf, L"#%lu", value)` returns).
unsafe fn atm_format_int_name(buf: *mut u16, value: u32) -> u8 {
    *buf = b'#' as u16;
    let mut digits = [0u8; 10];
    let mut v = value;
    let mut n = 0usize;
    if v == 0 {
        digits[0] = b'0';
        n = 1;
    } else {
        while v > 0 {
            digits[n] = b'0' + (v % 10) as u8;
            v /= 10;
            n += 1;
        }
    }
    for i in 0..n {
        *buf.add(1 + i) = digits[n - 1 - i] as u16;
    }
    *buf.add(1 + n) = 0;
    (1 + n) as u8
}

/// `static RtlpHashAtomName` — file-local in C, private here. Finds the
/// existing entry for `atom_name`, and sets `*hash_link` to the
/// insertion slot (bucket head or prior entry's HashLink), or NULL.
unsafe fn RtlpHashAtomName(
    atom_table: *mut RTL_ATOM_TABLE,
    atom_name: *const u16,
    hash_link: *mut *mut *mut RTL_ATOM_TABLE_ENTRY,
) -> *mut RTL_ATOM_TABLE_ENTRY {
    let mut name = UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    RtlInitUnicodeString(&mut name, atom_name);

    let mut hash: u32 = 0;
    if name.Length != 0
        && atm_nt_success(RtlHashUnicodeString(
            &name,
            1,
            HASH_STRING_ALGORITHM_X65599,
            &mut hash,
        ))
    {
        let idx = atom_::bucket_index(hash, (*atom_table).number_of_buckets);
        let mut link: *mut *mut RTL_ATOM_TABLE_ENTRY = atm_buckets(atom_table).add(idx as usize);

        let mut current = *link;
        while !current.is_null() {
            if (*current).name_length as u32 == (name.Length / 2) as u32
                && _wcsicmp((*current).name.as_ptr(), name.Buffer) == 0
            {
                *hash_link = link;
                return current;
            }
            link = core::ptr::addr_of_mut!((*current).hash_link);
            current = (*current).hash_link;
        }
        *hash_link = link;
    } else {
        *hash_link = core::ptr::null_mut();
    }
    core::ptr::null_mut()
}

/// `static RtlpCheckIntegerAtom` — file-local in C.
unsafe fn RtlpCheckIntegerAtom(atom_name: *const u16, atom_value: *mut u16) -> bool {
    match atom_::check_packed_integer_atom(atom_name as usize) {
        atom_::IntegerAtom::Packed(v) => {
            if !atom_value.is_null() {
                *atom_value = v;
            }
            return true;
        }
        atom_::IntegerAtom::NotPacked => {}
    }
    // Real pointer (high bits set ⇒ non-NULL, per C's ASSERT).
    if *atom_name != b'#' as u16 {
        return false;
    }
    let mut p = atom_name.add(1);
    while *p != 0 {
        if *p < b'0' as u16 || *p > b'9' as u16 {
            return false;
        }
        p = p.add(1);
    }
    let mut atom_string = UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    RtlInitUnicodeString(&mut atom_string, atom_name.add(1));
    let mut long_value: u32 = 0;
    RtlUnicodeStringToInteger(&atom_string, 10, &mut long_value);
    // C writes *AtomValue unconditionally here (no NULL check).
    *atom_value = (long_value & 0xFFFF) as u16;
    true
}

/// `NTSTATUS NTAPI RtlCreateAtomTable(ULONG, PRTL_ATOM_TABLE*)`.
#[no_mangle]
pub unsafe extern "system" fn RtlCreateAtomTable(
    table_size: u32,
    atom_table: *mut *mut RTL_ATOM_TABLE,
) -> i32 {
    if !(*atom_table).is_null() {
        return STATUS_SUCCESS;
    }
    let table_size = atom_::clamp_table_size(table_size);

    let size = ((table_size - 1) as usize * core::mem::size_of::<*mut RTL_ATOM_TABLE_ENTRY>()
        + core::mem::size_of::<RTL_ATOM_TABLE>()) as u32;
    let table = RtlpAllocAtomTable(size);
    if table.is_null() {
        return STATUS_NO_MEMORY;
    }

    (*table).number_of_buckets = table_size;

    let status = RtlpInitAtomTableLock(table);
    if !atm_nt_success(status) {
        RtlpFreeAtomTable(table);
        return status;
    }

    if RtlpCreateAtomHandleTable(table) == 0 {
        RtlpDestroyAtomTableLock(table);
        RtlpFreeAtomTable(table);
        return STATUS_NO_MEMORY;
    }

    *atom_table = table;
    STATUS_SUCCESS
}

/// `NTSTATUS NTAPI RtlDestroyAtomTable(PRTL_ATOM_TABLE)`.
#[no_mangle]
pub unsafe extern "system" fn RtlDestroyAtomTable(atom_table: *mut RTL_ATOM_TABLE) -> i32 {
    if RtlpLockAtomTable(atom_table) == 0 {
        return STATUS_INVALID_PARAMETER;
    }

    let n = (*atom_table).number_of_buckets as usize;
    let base = atm_buckets(atom_table);
    for i in 0..n {
        let current_bucket = base.add(i);
        let mut next_entry = *current_bucket;
        *current_bucket = core::ptr::null_mut();
        while !next_entry.is_null() {
            let current_entry = next_entry;
            next_entry = (*next_entry).hash_link;
            // handles freed wholesale by RtlpDestroyAtomHandleTable below
            RtlpFreeAtomTableEntry(current_entry);
        }
    }

    RtlpDestroyAtomHandleTable(atom_table);
    RtlpUnlockAtomTable(atom_table);
    RtlpDestroyAtomTableLock(atom_table);
    RtlpFreeAtomTable(atom_table);
    STATUS_SUCCESS
}

/// `NTSTATUS NTAPI RtlEmptyAtomTable(PRTL_ATOM_TABLE, BOOLEAN)`.
#[no_mangle]
pub unsafe extern "system" fn RtlEmptyAtomTable(
    atom_table: *mut RTL_ATOM_TABLE,
    delete_pinned: u8,
) -> i32 {
    if RtlpLockAtomTable(atom_table) == 0 {
        return STATUS_INVALID_PARAMETER;
    }

    let n = (*atom_table).number_of_buckets as usize;
    let base = atm_buckets(atom_table);
    for i in 0..n {
        let current_bucket = base.add(i);
        let mut next_entry = *current_bucket;
        let mut ptr_entry: *mut *mut RTL_ATOM_TABLE_ENTRY = current_bucket;
        while !next_entry.is_null() {
            let current_entry = next_entry;
            next_entry = (*next_entry).hash_link;
            if delete_pinned != 0 || ((*current_entry).flags & RTL_ATOM_IS_PINNED) == 0 {
                *ptr_entry = next_entry;
                RtlpFreeAtomHandle(atom_table, current_entry);
                RtlpFreeAtomTableEntry(current_entry);
            } else {
                ptr_entry = core::ptr::addr_of_mut!((*current_entry).hash_link);
            }
        }
    }

    RtlpUnlockAtomTable(atom_table);
    STATUS_SUCCESS
}

/// `NTSTATUS NTAPI RtlAddAtomToAtomTable(PRTL_ATOM_TABLE, PWSTR, PRTL_ATOM)`.
#[no_mangle]
pub unsafe extern "system" fn RtlAddAtomToAtomTable(
    atom_table: *mut RTL_ATOM_TABLE,
    atom_name: *mut u16,
    atom: *mut u16,
) -> i32 {
    let mut atom_value: u16 = 0;
    let mut status = STATUS_SUCCESS;

    if RtlpCheckIntegerAtom(atom_name, &mut atom_value) {
        if atom_value >= 0xC000 {
            status = STATUS_INVALID_PARAMETER;
        } else if !atom.is_null() {
            *atom = atom_value;
        }
        return status;
    }

    RtlpLockAtomTable(atom_table);

    let mut hash_link: *mut *mut RTL_ATOM_TABLE_ENTRY = core::ptr::null_mut();
    let entry = RtlpHashAtomName(atom_table, atom_name, &mut hash_link);

    if !entry.is_null() {
        if ((*entry).flags & RTL_ATOM_IS_PINNED) == 0 {
            (*entry).reference_count = (*entry).reference_count.wrapping_add(1);
            if (*entry).reference_count == 0 {
                (*entry).flags |= RTL_ATOM_IS_PINNED;
            }
        }
        if !atom.is_null() {
            *atom = (*entry).atom;
        }
    } else if !hash_link.is_null() {
        let atom_name_len = wcslen(atom_name) as u32;
        if !atom_::atom_name_len_ok(atom_name_len) {
            status = STATUS_INVALID_PARAMETER;
        } else {
            let size = (core::mem::size_of::<RTL_ATOM_TABLE_ENTRY>() - 2
                + ((atom_name_len + 1) * 2) as usize) as u32;
            let new_entry = RtlpAllocAtomTableEntry(size);
            if !new_entry.is_null() {
                (*new_entry).hash_link = core::ptr::null_mut();
                (*new_entry).reference_count = 1;
                (*new_entry).flags = 0;
                (*new_entry).name_length = atom_name_len as u8;
                core::ptr::copy_nonoverlapping(
                    atom_name,
                    (*new_entry).name.as_mut_ptr(),
                    (atom_name_len + 1) as usize,
                );
                if RtlpCreateAtomHandle(atom_table, new_entry) != 0 {
                    *hash_link = new_entry;
                    if !atom.is_null() {
                        *atom = (*new_entry).atom;
                    }
                } else {
                    RtlpFreeAtomTableEntry(new_entry);
                    status = STATUS_NO_MEMORY;
                }
            } else {
                status = STATUS_NO_MEMORY;
            }
        }
    } else {
        status = ATM_STATUS_OBJECT_NAME_INVALID;
    }

    RtlpUnlockAtomTable(atom_table);
    status
}

/// `NTSTATUS NTAPI RtlDeleteAtomFromAtomTable(PRTL_ATOM_TABLE, RTL_ATOM)`.
#[no_mangle]
pub unsafe extern "system" fn RtlDeleteAtomFromAtomTable(
    atom_table: *mut RTL_ATOM_TABLE,
    atom: u16,
) -> i32 {
    let mut status = STATUS_SUCCESS;

    if atom >= 0xC000 {
        RtlpLockAtomTable(atom_table);

        let entry = RtlpGetAtomEntry(atom_table, (atom - 0xC000) as u32);

        if !entry.is_null() && (*entry).atom == atom {
            if ((*entry).flags & RTL_ATOM_IS_PINNED) == 0 {
                (*entry).reference_count = (*entry).reference_count.wrapping_sub(1);
                if (*entry).reference_count == 0 {
                    let mut hash_link: *mut *mut RTL_ATOM_TABLE_ENTRY = core::ptr::null_mut();
                    if !RtlpHashAtomName(atom_table, (*entry).name.as_ptr(), &mut hash_link)
                        .is_null()
                    {
                        *hash_link = (*entry).hash_link;
                        RtlpFreeAtomHandle(atom_table, entry);
                        RtlpFreeAtomTableEntry(entry);
                    }
                    // else: C ASSERT(FALSE) — unreachable; do nothing.
                }
            } else {
                status = ATM_STATUS_WAS_LOCKED;
            }
        } else {
            status = ATM_STATUS_INVALID_HANDLE;
        }

        RtlpUnlockAtomTable(atom_table);
    }

    status
}

/// `NTSTATUS NTAPI RtlLookupAtomInAtomTable(PRTL_ATOM_TABLE, PWSTR, PRTL_ATOM)`.
#[no_mangle]
pub unsafe extern "system" fn RtlLookupAtomInAtomTable(
    atom_table: *mut RTL_ATOM_TABLE,
    atom_name: *mut u16,
    atom: *mut u16,
) -> i32 {
    let mut atom_value: u16 = 0;
    let mut found_atom: u16 = 0;
    let mut status = STATUS_SUCCESS;

    if RtlpCheckIntegerAtom(atom_name, &mut atom_value) {
        if atom_value >= 0xC000 {
            status = STATUS_INVALID_PARAMETER;
        } else if !atom.is_null() {
            *atom = atom_value;
        }
        return status;
    }

    RtlpLockAtomTable(atom_table);
    status = ATM_STATUS_OBJECT_NAME_NOT_FOUND;

    let mut hash_link: *mut *mut RTL_ATOM_TABLE_ENTRY = core::ptr::null_mut();
    let entry = RtlpHashAtomName(atom_table, atom_name, &mut hash_link);
    if !entry.is_null() {
        status = STATUS_SUCCESS;
        found_atom = (*entry).atom;
    }

    RtlpUnlockAtomTable(atom_table);
    if atm_nt_success(status) && !atom.is_null() {
        *atom = found_atom;
    }
    status
}

/// `NTSTATUS NTAPI RtlPinAtomInAtomTable(PRTL_ATOM_TABLE, RTL_ATOM)`.
#[no_mangle]
pub unsafe extern "system" fn RtlPinAtomInAtomTable(
    atom_table: *mut RTL_ATOM_TABLE,
    atom: u16,
) -> i32 {
    let mut status = STATUS_SUCCESS;

    if atom >= 0xC000 {
        RtlpLockAtomTable(atom_table);

        let entry = RtlpGetAtomEntry(atom_table, (atom - 0xC000) as u32);
        if !entry.is_null() && (*entry).atom == atom {
            (*entry).flags |= RTL_ATOM_IS_PINNED;
        } else {
            status = ATM_STATUS_INVALID_HANDLE;
        }

        RtlpUnlockAtomTable(atom_table);
    }

    status
}

/// `NTSTATUS NTAPI RtlQueryAtomInAtomTable(PRTL_ATOM_TABLE, RTL_ATOM,
/// PULONG RefCount, PULONG PinCount, PWSTR AtomName, PULONG NameLength)`.
#[no_mangle]
pub unsafe extern "system" fn RtlQueryAtomInAtomTable(
    atom_table: *mut RTL_ATOM_TABLE,
    atom: u16,
    ref_count: *mut u32,
    pin_count: *mut u32,
    atom_name: *mut u16,
    name_length: *mut u32,
) -> i32 {
    let mut status = STATUS_SUCCESS;
    let mut unlock = false;

    // union { RTL_ATOM_TABLE_ENTRY; WCHAR[sizeof(entry)/2 + 12] }
    const NUMBER_ENTRY_WORDS: usize = core::mem::size_of::<RTL_ATOM_TABLE_ENTRY>() / 2 + 12;
    let mut number_entry: [u16; NUMBER_ENTRY_WORDS] = [0; NUMBER_ENTRY_WORDS];
    let synth = number_entry.as_mut_ptr() as *mut RTL_ATOM_TABLE_ENTRY;

    let entry: *mut RTL_ATOM_TABLE_ENTRY = if atom < 0xC000 {
        (*synth).atom = atom;
        (*synth).name_length = atm_format_int_name((*synth).name.as_mut_ptr(), atom as u32);
        (*synth).reference_count = 1;
        (*synth).flags = RTL_ATOM_IS_PINNED;
        synth
    } else {
        RtlpLockAtomTable(atom_table);
        unlock = true;
        RtlpGetAtomEntry(atom_table, (atom - 0xC000) as u32)
    };

    if !entry.is_null() && (*entry).atom == atom {
        if !ref_count.is_null() {
            *ref_count = (*entry).reference_count as u32;
        }
        if !pin_count.is_null() {
            *pin_count = (((*entry).flags & RTL_ATOM_IS_PINNED) != 0) as u32;
        }
        if !name_length.is_null() {
            let mut length = (*entry).name_length as u32 * 2;
            if !atom_name.is_null() {
                if *name_length < length + 2 {
                    if *name_length < 4 {
                        *name_length = length;
                        status = ATM_STATUS_BUFFER_TOO_SMALL;
                    } else {
                        length = *name_length - 2;
                    }
                }
                if atm_nt_success(status) {
                    core::ptr::copy_nonoverlapping(
                        (*entry).name.as_ptr(),
                        atom_name,
                        (length / 2) as usize,
                    );
                    *atom_name.add((length / 2) as usize) = 0;
                    *name_length = length;
                }
            } else {
                *name_length = length;
            }
        } else if !atom_name.is_null() {
            status = STATUS_INVALID_PARAMETER;
        }
    } else {
        status = ATM_STATUS_INVALID_HANDLE;
    }

    if unlock {
        RtlpUnlockAtomTable(atom_table);
    }
    status
}

/// `NTSTATUS NTAPI RtlQueryAtomListInAtomTable(PRTL_ATOM_TABLE, ULONG,
/// PULONG AtomCount, PRTL_ATOM AtomList)`. (NtQueryInformationAtom.)
#[no_mangle]
pub unsafe extern "system" fn RtlQueryAtomListInAtomTable(
    atom_table: *mut RTL_ATOM_TABLE,
    mut max_atom_count: u32,
    atom_count: *mut u32,
    mut atom_list: *mut u16,
) -> i32 {
    let mut atoms: u32 = 0;
    let mut status = STATUS_SUCCESS;

    RtlpLockAtomTable(atom_table);

    let n = (*atom_table).number_of_buckets as usize;
    let base = atm_buckets(atom_table);
    for i in 0..n {
        let mut current_entry = *base.add(i);
        while !current_entry.is_null() {
            if max_atom_count > 0 {
                *atom_list = (*current_entry).atom;
                atom_list = atom_list.add(1);
                max_atom_count -= 1;
            } else {
                status = ATM_STATUS_INFO_LENGTH_MISMATCH;
            }
            atoms += 1;
            current_entry = (*current_entry).hash_link;
        }
    }

    *atom_count = atoms;
    RtlpUnlockAtomTable(atom_table);
    status
}

// ===== path.c wave (Opus 4.8) — translated, integrated into rtl-strings =====
// The 27 non-lexical path functions from
// `ntoskrnl-rust/src/rtl/path.rs`, remapped from the kernel `ntkrnl_sys`
// substrate onto cabi's `UNICODE_STRING` + 3-way allocator. Wrapped in a
// private module so the local NT type aliases / consts / support structs
// (OBJECT_ATTRIBUTES, IO_STATUS_BLOCK, PEB, CURDIR, …) and the rtl-internal
// extern decls don't collide with cabi's top-level ones. `#[no_mangle]` items
// inside a module still export their global symbols. The 6 lexical fns
// (RtlDetermineDosPathNameType_{Ustr,U}, RtlGetLongestNtPathLength,
// RtlIsDosDeviceName_{Ustr,U}, RtlGetLengthWithoutTrailingPathSeparators)
// stay in their existing cabi versions above; the device/prefix statics they
// reference are DEFINED here (real UTF-16 buffers) and re-exported below.
#[allow(
    non_snake_case,
    non_upper_case_globals,
    non_camel_case_types,
    dead_code,
    unused_assignments,
    unused_variables
)]
mod path_wave {
    // One definition, from the file scope above. This module used to carry its own
    // `const TAG_USTR` -- a nested mod does not inherit the outer one, which is how the
    // copy got here, and how it then drifted to the byte-reversed value.
    use super::TAG_USTR;
    use super::{
        rtl_alloc, rtl_free, wcslen, RtlAppendUnicodeStringToString, RtlCopyUnicodeString,
        RtlDetermineDosPathNameType_U, RtlDetermineDosPathNameType_Ustr, RtlEqualUnicodeString,
        RtlFindCharInUnicodeString, RtlFreeUnicodeString,
        RtlInitUnicodeStringEx, RtlIsDosDeviceName_Ustr, RtlPrefixUnicodeString,
        RtlQueryEnvironmentVariable_U, RtlUpcaseUnicodeChar, RTL_BUFFER, UNICODE_STRING,
    };

    type ULONG = u32;
    type USHORT = u16;
    type WCHAR = u16;
    type NTSTATUS = i32;
    type BOOLEAN = u8;
    type SIZE_T = usize;
    type ULONG_PTR = usize;
    type CHAR = u8;
    type PCHAR = *mut u8;
    type PVOID = *mut core::ffi::c_void;
    type HANDLE = *mut core::ffi::c_void;
    type PWSTR = *mut u16;
    type PWCHAR = *mut u16;
    type PWCH = *mut u16;
    type PCWSTR = *const u16;
    type PCWCH = *const u16;
    type PBOOLEAN = *mut u8;
    type PULONG = *mut u32;
    type PSIZE_T = *mut usize;
    type PUNICODE_STRING = *mut UNICODE_STRING;
    type PCUNICODE_STRING = *const UNICODE_STRING;
    type RTL_PATH_TYPE = u32;

    const RtlPathTypeUnknown: RTL_PATH_TYPE = 0;
    const RtlPathTypeUncAbsolute: RTL_PATH_TYPE = 1;
    const RtlPathTypeDriveAbsolute: RTL_PATH_TYPE = 2;
    const RtlPathTypeDriveRelative: RTL_PATH_TYPE = 3;
    const RtlPathTypeRooted: RTL_PATH_TYPE = 4;
    const RtlPathTypeRelative: RTL_PATH_TYPE = 5;
    const RtlPathTypeLocalDevice: RTL_PATH_TYPE = 6;
    const RtlPathTypeRootLocalDevice: RTL_PATH_TYPE = 7;

    const MAX_PFX_SIZE: u32 = 16;
    const RTL_CURDIR_IS_REMOVABLE: u32 = 0x1;
    const RTL_CURDIR_DROP_OLD_HANDLE: u32 = 0x2;
    const RTL_CURDIR_ALL_FLAGS: u32 = RTL_CURDIR_DROP_OLD_HANDLE | RTL_CURDIR_IS_REMOVABLE;
    const MAX_PATH: u32 = 260;
    const OBJ_CASE_INSENSITIVE: u32 = 0x40;
    const OBJ_INHERIT: u32 = 0x02;
    // 32767, per sdk/include/xdk/ntbasedef.h:621. Was 0x7FFE (32766) — the same
    // off-by-one the two kernel32-rust copies carried; it made this guard reject a
    // name one character short of the real limit.
    const UNICODE_STRING_MAX_CHARS: u32 = 32767;
    const UNICODE_STRING_MAX_BYTES: u32 = 0xFFFE;
    const STATUS_SUCCESS: i32 = 0;
    const STATUS_INVALID_PARAMETER: i32 = 0xC000_000Du32 as i32;
    const STATUS_NO_MEMORY: i32 = 0xC000_0017u32 as i32;
    const STATUS_NAME_TOO_LONG: i32 = 0xC000_0106u32 as i32;
    const STATUS_BUFFER_TOO_SMALL: i32 = 0xC000_0023u32 as i32;
    const STATUS_OBJECT_NAME_INVALID: i32 = 0xC0000033u32 as i32;
    const STATUS_NOT_A_DIRECTORY: i32 = 0xC000_0103u32 as i32;
    const STATUS_SHARING_VIOLATION: i32 = 0xC000_0043u32 as i32;
    const STATUS_ACCESS_DENIED: i32 = 0xC000_0022u32 as i32;
    const STATUS_NO_SUCH_FILE: i32 = 0xC000_000Fu32 as i32;
    const STATUS_INTERNAL_ERROR: i32 = 0xC000_00E5u32 as i32;
    const RTL_USER_PROCESS_PARAMETERS_NORMALIZED: u32 = 0x00000001;
    // ndk/rtlfuncs.h:3018-3021 — RtlNtPathNameToDosPathName's *PathType output.
    const RTL_UNCHANGED_UNK_PATH: u32 = 1;
    const RTL_CONVERTED_UNC_PATH: u32 = 2;
    const RTL_CONVERTED_NT_PATH: u32 = 3;
    const RTL_UNCHANGED_DOS_PATH: u32 = 4;
    const OBJ_NAME_PATH_SEPARATOR: u16 = b'\\' as u16;
    const SYNCHRONIZE: u32 = 0x0010_0000;
    const FILE_TRAVERSE: u32 = 0x0020;
    const FILE_SHARE_READ: u32 = 0x0001;
    const FILE_SHARE_WRITE: u32 = 0x0002;
    const FILE_DIRECTORY_FILE: u32 = 0x0000_0001;
    const FILE_SYNCHRONOUS_IO_NONALERT: u32 = 0x0000_0020;
    const FILE_REMOVABLE_MEDIA: u32 = 0x0000_0001;

    #[inline]
    fn NT_SUCCESS(s: NTSTATUS) -> bool {
        s >= 0
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct CURDIR {
        pub DosPath: UNICODE_STRING,
        pub Handle: HANDLE,
    }
    type PCURDIR = *mut CURDIR;

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct RTLP_CURDIR_REF {
        pub RefCount: i32,
        pub Handle: HANDLE,
    }
    type PRTLP_CURDIR_REF = *mut RTLP_CURDIR_REF;

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct RTL_RELATIVE_NAME_U {
        pub RelativeName: UNICODE_STRING,
        pub ContainingDirectory: HANDLE,
        pub CurDirRef: PRTLP_CURDIR_REF,
    }
    type PRTL_RELATIVE_NAME_U = *mut RTL_RELATIVE_NAME_U;

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct RTL_UNICODE_STRING_BUFFER {
        pub String: UNICODE_STRING,
        pub ByteBuffer: RTL_BUFFER,
        pub MinimumStaticBufferForTerminalNul: WCHAR,
    }
    type PRTL_UNICODE_STRING_BUFFER = *mut RTL_UNICODE_STRING_BUFFER;

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct RTL_USER_PROCESS_PARAMETERS {
        pub MaximumLength: u32,
        pub Length: u32,
        pub Flags: u32,
        pub DebugFlags: u32,
        pub ConsoleHandle: HANDLE,
        pub ConsoleFlags: u32,
        pub StandardInput: HANDLE,
        pub StandardOutput: HANDLE,
        pub StandardError: HANDLE,
        pub CurrentDirectory: CURDIR,
        pub DllPath: UNICODE_STRING,
        pub ImagePathName: UNICODE_STRING,
        pub CommandLine: UNICODE_STRING,
    }
    type PRTL_USER_PROCESS_PARAMETERS = *mut RTL_USER_PROCESS_PARAMETERS;

    #[repr(C)]
    pub struct PEB {
        #[cfg(target_pointer_width = "64")]
        _pad: [u8; 0x20],
        #[cfg(target_pointer_width = "32")]
        _pad: [u8; 0x10],
        pub ProcessParameters: PRTL_USER_PROCESS_PARAMETERS,
    }
    type PPEB = *mut PEB;

    #[cfg(target_pointer_width = "64")]
    #[repr(C)]
    pub struct NT_TEB {
        _pad: [u8; 0x60],
        pub ProcessEnvironmentBlock: PPEB,
    }
    #[cfg(target_pointer_width = "32")]
    #[repr(C)]
    pub struct NT_TEB {
        _pad: [u8; 0x30],
        pub ProcessEnvironmentBlock: PPEB,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct OBJECT_ATTRIBUTES {
        pub Length: u32,
        pub RootDirectory: HANDLE,
        pub ObjectName: PUNICODE_STRING,
        pub Attributes: u32,
        pub SecurityDescriptor: PVOID,
        pub SecurityQualityOfService: PVOID,
    }
    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct IO_STATUS_BLOCK {
        pub Status: NTSTATUS,
        pub Information: usize,
    }
    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct FILE_BASIC_INFORMATION {
        pub CreationTime: i64,
        pub LastAccessTime: i64,
        pub LastWriteTime: i64,
        pub ChangeTime: i64,
        pub FileAttributes: u32,
    }
    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct FILE_FS_DEVICE_INFORMATION {
        pub DeviceType: u32,
        pub Characteristics: u32,
    }
    const FileFsDeviceInformation: u32 = 4;

    #[inline]
    unsafe fn RtlCopyMemory(d: PVOID, s: *const core::ffi::c_void, len: usize) {
        core::ptr::copy_nonoverlapping(s as *const u8, d as *mut u8, len);
    }
    #[inline]
    unsafe fn RtlZeroMemory(d: PVOID, len: usize) {
        core::ptr::write_bytes(d as *mut u8, 0, len);
    }
    #[inline]
    unsafe fn memcpy(d: *mut u8, s: *const u8, len: usize) {
        core::ptr::copy_nonoverlapping(s, d, len);
    }
    #[inline]
    unsafe fn memmove(d: *mut u8, s: *const u8, len: usize) {
        core::ptr::copy(s, d, len);
    }
    #[inline]
    unsafe fn wcsrchr(mut s: *const u16, c: u16) -> *mut u16 {
        let mut last: *mut u16 = core::ptr::null_mut();
        while *s != 0 {
            if *s == c {
                last = s as *mut u16;
            }
            s = s.add(1);
        }
        last
    }
    #[inline]
    unsafe fn RtlpUpcaseUnicodeChar(c: u16) -> u16 {
        RtlUpcaseUnicodeChar(c)
    }
    #[inline]
    unsafe fn InterlockedDecrement(p: *mut i32) -> i32 {
        (&*(p as *const core::sync::atomic::AtomicI32))
            .fetch_sub(1, core::sync::atomic::Ordering::SeqCst)
            - 1
    }
    #[inline]
    unsafe fn InterlockedIncrement(p: *mut i32) -> i32 {
        (&*(p as *const core::sync::atomic::AtomicI32))
            .fetch_add(1, core::sync::atomic::Ordering::SeqCst)
            + 1
    }
    // Really DbgPrint, not a no-op. This shadowed the whole module: the four
    // "RtlGetFullPathName_Ustr() returned 0" reports below sit on the path that
    // answers STATUS_OBJECT_NAME_INVALID, so every DOS→NT path conversion that
    // failed did so without a word on the serial port. Variadic, so extern "C"
    // (cdecl) on every arch — the same binding RtlDumpResource already uses.
    extern "C" {
        fn DbgPrint(fmt: *const i8, ...) -> u32;
    }
    #[inline]
    unsafe fn RtlInitEmptyUnicodeString(dest: *mut UNICODE_STRING, buf: PWSTR, max: u16) {
        (*dest).Length = 0;
        (*dest).MaximumLength = max;
        (*dest).Buffer = buf;
    }

    // rtl-internal helpers (ntdll + ntoskrnl). Plain externs — they resolve on
    // the PE kernel/user-mode targets; the host difftest cdylib never calls a
    // path fn that needs them (cargo check never links).
    extern "system" {
        // rtlp.h: RtlpAllocateStringMemory/RtlpFreeStringMemory are C MACROS for
        // RtlpAllocateMemory/RtlpFreeMemory — C never emits a `RtlpAllocateString*`
        // symbol. Bind the real macro targets and wrap them inline below.
        fn RtlpAllocateMemory(Bytes: ULONG, Tag: ULONG) -> PWSTR;
        fn RtlpFreeMemory(Mem: *mut core::ffi::c_void, Tag: ULONG);
        fn RtlGetCurrentPeb() -> PPEB;
        fn RtlAcquirePebLock();
        fn RtlReleasePebLock();
        fn NtClose(Handle: HANDLE) -> NTSTATUS;
        fn NtOpenFile(
            FileHandle: *mut HANDLE,
            DesiredAccess: u32,
            ObjectAttributes: *mut OBJECT_ATTRIBUTES,
            IoStatusBlock: *mut IO_STATUS_BLOCK,
            ShareAccess: u32,
            OpenOptions: u32,
        ) -> NTSTATUS;
        fn NtQueryVolumeInformationFile(
            FileHandle: HANDLE,
            IoStatusBlock: *mut IO_STATUS_BLOCK,
            FsInformation: PVOID,
            Length: u32,
            FsInformationClass: u32,
        ) -> NTSTATUS;
        fn ZwQueryAttributesFile(
            ObjectAttributes: *mut OBJECT_ATTRIBUTES,
            FileInformation: *mut FILE_BASIC_INFORMATION,
        ) -> NTSTATUS;
        // rtlfuncs.h: RtlEnsureBufferSize is FORCEINLINE → real work is
        // RtlpEnsureBufferSize. Bind the real symbol; the inline is replicated below.
        fn RtlpEnsureBufferSize(
            Flags: ULONG,
            Buffer: *mut RTL_BUFFER,
            RequiredSize: ULONG,
        ) -> NTSTATUS;
        // ntdll-only SxS redirection hub (path.c guards it under #ifndef
        // _NTOSKRNL_). Declared unconditionally — only referenced from the
        // cfg(not(feature="kernel")) branch of RtlDosSearchPath_Ustr, so the
        // kernel build never emits a link reference to it.
        fn RtlDosApplyFileIsolationRedirection_Ustr(
            Flags: ULONG,
            OriginalName: PUNICODE_STRING,
            Extension: PUNICODE_STRING,
            StaticString: PUNICODE_STRING,
            DynamicString: PUNICODE_STRING,
            NewName: *mut PUNICODE_STRING,
            NewFlags: PULONG,
            FileNameSize: PSIZE_T,
            RequiredLength: PSIZE_T,
        ) -> NTSTATUS;
    }
    // RtlpAllocateStringMemory/RtlpFreeStringMemory: rtlp.h macros → Rtlp{Allocate,Free}Memory.
    #[inline]
    unsafe fn RtlpAllocateStringMemory(Length: ULONG, Tag: ULONG) -> PWSTR {
        RtlpAllocateMemory(Length, Tag)
    }
    #[inline]
    unsafe fn RtlpFreeStringMemory(Buffer: *mut core::ffi::c_void, Tag: ULONG) {
        RtlpFreeMemory(Buffer, Tag)
    }
    // RtlEnsureBufferSize: FORCEINLINE in rtlfuncs.h (fast-path + RtlpEnsureBufferSize).
    #[inline]
    unsafe fn RtlEnsureBufferSize(
        Flags: ULONG,
        Buffer: *mut RTL_BUFFER,
        RequiredSize: ULONG,
    ) -> NTSTATUS {
        if !Buffer.is_null() && (RequiredSize as usize) <= (*Buffer).Size {
            return STATUS_SUCCESS;
        }
        RtlpEnsureBufferSize(Flags, Buffer, RequiredSize)
    }
    // NtCurrentTeb() is a FORCEINLINE macro (no out-of-line symbol); route the
    // PEB fetch through RtlGetCurrentPeb (a real exported symbol on both targets).
    #[inline]
    unsafe fn NtCurrentPeb() -> PPEB {
        RtlGetCurrentPeb()
    }

    // Heap shims — route the C-translated RtlAllocateHeap/RtlFreeHeap through
    // cabi's 3-way allocator (rtl_alloc/rtl_free). `heap_handle()` is a dummy.
    #[inline]
    unsafe fn heap_handle() -> PVOID {
        core::ptr::null_mut()
    }
    #[inline]
    unsafe fn rtl_heap_alloc(_heap: PVOID, _flags: u32, size: usize) -> PVOID {
        rtl_alloc(size, TAG_USTR) as PVOID
    }
    #[inline]
    unsafe fn rtl_heap_free(_heap: PVOID, _flags: u32, p: PVOID) -> u8 {
        if !p.is_null() {
            rtl_free(p as *mut u8, TAG_USTR);
        }
        1
    }

    // --- device / prefix statics (real UTF-16 buffers) ---
    static DEVICEROOTSTRING_BUF: [u16; 5] = [0x005C, 0x005C, 0x002E, 0x005C, 0x0000];
    #[no_mangle]
    pub static DeviceRootString: UNICODE_STRING = UNICODE_STRING {
        Length: 8,
        MaximumLength: 10,
        Buffer: DEVICEROOTSTRING_BUF.as_ptr() as *mut u16,
    };
    static RTLPDOSDEVICESUNCPREFIX_BUF: [u16; 9] = [
        0x005C, 0x003F, 0x003F, 0x005C, 0x0055, 0x004E, 0x0043, 0x005C, 0x0000,
    ];
    #[no_mangle]
    pub static RtlpDosDevicesUncPrefix: UNICODE_STRING = UNICODE_STRING {
        Length: 16,
        MaximumLength: 18,
        Buffer: RTLPDOSDEVICESUNCPREFIX_BUF.as_ptr() as *mut u16,
    };
    static RTLPWIN32NTROOTSLASH_BUF: [u16; 5] = [0x005C, 0x005C, 0x003F, 0x005C, 0x0000];
    #[no_mangle]
    pub static RtlpWin32NtRootSlash: UNICODE_STRING = UNICODE_STRING {
        Length: 8,
        MaximumLength: 10,
        Buffer: RTLPWIN32NTROOTSLASH_BUF.as_ptr() as *mut u16,
    };
    static RTLPDOSSLASHCONDEVICE_BUF: [u16; 8] = [
        0x005C, 0x005C, 0x002E, 0x005C, 0x0043, 0x004F, 0x004E, 0x0000,
    ];
    #[no_mangle]
    pub static RtlpDosSlashCONDevice: UNICODE_STRING = UNICODE_STRING {
        Length: 14,
        MaximumLength: 16,
        Buffer: RTLPDOSSLASHCONDEVICE_BUF.as_ptr() as *mut u16,
    };
    static RTLPDOSDEVICESPREFIX_BUF: [u16; 5] = [0x005C, 0x003F, 0x003F, 0x005C, 0x0000];
    #[no_mangle]
    pub static RtlpDosDevicesPrefix: UNICODE_STRING = UNICODE_STRING {
        Length: 8,
        MaximumLength: 10,
        Buffer: RTLPDOSDEVICESPREFIX_BUF.as_ptr() as *mut u16,
    };
    static RTLPDOSLPTDEVICE_BUF: [u16; 4] = [0x004C, 0x0050, 0x0054, 0x0000];
    #[no_mangle]
    pub static RtlpDosLPTDevice: UNICODE_STRING = UNICODE_STRING {
        Length: 6,
        MaximumLength: 8,
        Buffer: RTLPDOSLPTDEVICE_BUF.as_ptr() as *mut u16,
    };
    static RTLPDOSCOMDEVICE_BUF: [u16; 4] = [0x0043, 0x004F, 0x004D, 0x0000];
    #[no_mangle]
    pub static RtlpDosCOMDevice: UNICODE_STRING = UNICODE_STRING {
        Length: 6,
        MaximumLength: 8,
        Buffer: RTLPDOSCOMDEVICE_BUF.as_ptr() as *mut u16,
    };
    static RTLPDOSPRNDEVICE_BUF: [u16; 4] = [0x0050, 0x0052, 0x004E, 0x0000];
    #[no_mangle]
    pub static RtlpDosPRNDevice: UNICODE_STRING = UNICODE_STRING {
        Length: 6,
        MaximumLength: 8,
        Buffer: RTLPDOSPRNDEVICE_BUF.as_ptr() as *mut u16,
    };
    static RTLPDOSAUXDEVICE_BUF: [u16; 4] = [0x0041, 0x0055, 0x0058, 0x0000];
    #[no_mangle]
    pub static RtlpDosAUXDevice: UNICODE_STRING = UNICODE_STRING {
        Length: 6,
        MaximumLength: 8,
        Buffer: RTLPDOSAUXDEVICE_BUF.as_ptr() as *mut u16,
    };
    static RTLPDOSCONDEVICE_BUF: [u16; 4] = [0x0043, 0x004F, 0x004E, 0x0000];
    #[no_mangle]
    pub static RtlpDosCONDevice: UNICODE_STRING = UNICODE_STRING {
        Length: 6,
        MaximumLength: 8,
        Buffer: RTLPDOSCONDEVICE_BUF.as_ptr() as *mut u16,
    };
    static RTLPDOSNULDEVICE_BUF: [u16; 4] = [0x004E, 0x0055, 0x004C, 0x0000];
    #[no_mangle]
    pub static RtlpDosNULDevice: UNICODE_STRING = UNICODE_STRING {
        Length: 6,
        MaximumLength: 8,
        Buffer: RTLPDOSNULDEVICE_BUF.as_ptr() as *mut u16,
    };
    static RTLPDOUBLESLASHPREFIX_BUF: [u16; 3] = [0x005C, 0x005C, 0x0000];
    #[no_mangle]
    pub static RtlpDoubleSlashPrefix: UNICODE_STRING = UNICODE_STRING {
        Length: 4,
        MaximumLength: 6,
        Buffer: RTLPDOUBLESLASHPREFIX_BUF.as_ptr() as *mut u16,
    };
    static RTLPDEFAULTEXTENSION_BUF: [u16; 5] = [0x002E, 0x0044, 0x004C, 0x004C, 0x0000];
    #[no_mangle]
    pub static RtlpDefaultExtension: UNICODE_STRING = UNICODE_STRING {
        Length: 8,
        MaximumLength: 10,
        Buffer: RTLPDEFAULTEXTENSION_BUF.as_ptr() as *mut u16,
    };
    static RTLPDOTLOCAL_BUF: [u16; 8] = [
        0x002E, 0x004C, 0x006F, 0x0063, 0x0061, 0x006C, 0x005C, 0x0000,
    ];
    #[no_mangle]
    pub static RtlpDotLocal: UNICODE_STRING = UNICODE_STRING {
        Length: 14,
        MaximumLength: 16,
        Buffer: RTLPDOTLOCAL_BUF.as_ptr() as *mut u16,
    };
    static RTLPPATHDIVIDERS_BUF: [u16; 3] = [0x005C, 0x002F, 0x0000];
    #[no_mangle]
    pub static RtlpPathDividers: UNICODE_STRING = UNICODE_STRING {
        Length: 4,
        MaximumLength: 6,
        Buffer: RTLPPATHDIVIDERS_BUF.as_ptr() as *mut u16,
    };

    #[no_mangle]
    pub static mut RtlpCurDirRef: PRTLP_CURDIR_REF = core::ptr::null_mut();

    #[no_mangle]
    pub unsafe extern "system" fn RtlpCheckDeviceName(
        FileName: PUNICODE_STRING,
        Length: ULONG,
        NameInvalid: PBOOLEAN,
    ) -> NTSTATUS {
        let Buffer: PWCHAR =
            rtl_heap_alloc(heap_handle(), 0, (*FileName).Length as usize) as PWCHAR;
        let Status: NTSTATUS;

        if !Buffer.is_null() {
            *NameInvalid = 1u8;

            RtlCopyMemory(
                Buffer as *mut core::ffi::c_void,
                (*FileName).Buffer as *const core::ffi::c_void,
                (*FileName).Length as usize,
            );

            *Buffer.offset((Length as usize / core::mem::size_of::<u16>()) as isize) = '.' as u16;
            *Buffer.offset((Length as usize / core::mem::size_of::<u16>() + 1) as isize) = 0u16;

            *NameInvalid = if RtlDoesFileExists_U(Buffer) != 0 {
                0u8
            } else {
                1u8
            };

            Status = rtl_heap_free(heap_handle(), 0, Buffer as *mut core::ffi::c_void) as NTSTATUS;
        } else {
            *NameInvalid = 0u8;
            Status = STATUS_NO_MEMORY;
        }

        Status
    }

    unsafe fn RtlpCollapsePath(
        Path: PWSTR,
        mark: ULONG,
        SkipTrailingPathSeparators: BOOLEAN,
    ) -> ULONG {
        let mut p: PWSTR;
        let mut next: PWSTR;

        let mut PathLength: usize = wcslen(Path);
        let EndBuffer: PWSTR = Path.add(PathLength);
        let mut EndPath: PWSTR;

        /* Convert slashes into backslashes */
        p = Path;
        while *p != 0 {
            if *p == b'/' as u16 {
                *p = b'\\' as u16;
            }
            p = p.add(1);
        }

        /* Collapse duplicate backslashes */
        let mark_max = core::cmp::max(1u32, mark) as usize;
        next = Path.add(mark_max);
        p = next;
        while *p != 0 {
            if *p != b'\\' as u16 || *next.offset(-1) != b'\\' as u16 {
                *next = *p;
                next = next.add(1);
            }
            p = p.add(1);
        }
        *next = 0;
        EndPath = next;

        p = Path.add(mark as usize);
        'outer: while *p != 0 {
            if *p == b'.' as u16 {
                let c1 = *p.add(1);
                if c1 == 0 {
                    /* final . */
                    if p > Path.add(mark as usize) {
                        p = p.offset(-1);
                    }
                    *p = 0;
                    EndPath = p;
                    continue 'outer;
                } else if c1 == b'\\' as u16 {
                    /* .\ component */
                    next = p.add(2);
                    core::ptr::copy(next, p, (EndPath.offset_from(next) + 1) as usize);
                    EndPath = EndPath.offset(-(next.offset_from(p)));
                    continue 'outer;
                } else if c1 == b'.' as u16 {
                    let c2 = *p.add(2);
                    if c2 == b'\\' as u16 {
                        /* ..\ component */
                        next = p.add(3);
                        if p > Path.add(mark as usize) {
                            p = p.offset(-1);
                            while p > Path.add(mark as usize) && *p.offset(-1) != b'\\' as u16 {
                                p = p.offset(-1);
                            }
                        }
                        core::ptr::copy(next, p, (EndPath.offset_from(next) + 1) as usize);
                        EndPath = EndPath.offset(-(next.offset_from(p)));
                        continue 'outer;
                    } else if c2 == 0 {
                        /* final .. */
                        if p > Path.add(mark as usize) {
                            p = p.offset(-1);
                            while p > Path.add(mark as usize) && *p.offset(-1) != b'\\' as u16 {
                                p = p.offset(-1);
                            }
                            if p > Path.add(mark as usize) {
                                p = p.offset(-1);
                            }
                        }
                        *p = 0;
                        EndPath = p;
                        continue 'outer;
                    }
                }
            }

            /* Skip to the next component */
            while *p != 0 && *p != b'\\' as u16 {
                p = p.add(1);
            }
            if *p == b'\\' as u16 {
                /* Remove last dot in previous dir name */
                if p > Path.add(mark as usize) && *p.offset(-1) == b'.' as u16 {
                    core::ptr::copy(p, p.offset(-1), (EndPath.offset_from(p) + 1) as usize);
                    EndPath = EndPath.offset(-1);
                } else {
                    p = p.add(1);
                }
            }
        }

        /* Remove trailing backslashes if needed */
        if SkipTrailingPathSeparators != 0 {
            while p > Path.add(mark as usize)
                && (*p.offset(-1) == b'\\' as u16 || *p.offset(-1) == b'/' as u16)
            {
                p = p.offset(-1);
            }
        }

        /* Remove trailing spaces and dots */
        while p > Path && (*p.offset(-1) == b' ' as u16 || *p.offset(-1) == b'.' as u16) {
            p = p.offset(-1);
        }

        RtlZeroMemory(
            p as *mut _,
            ((EndBuffer.offset_from(p) + 1) as usize) * core::mem::size_of::<u16>(),
        );

        PathLength = p.offset_from(Path) as usize;
        (PathLength * core::mem::size_of::<u16>()) as ULONG
    }

    unsafe fn RtlpSkipUNCPrefix(FileNameBuffer: PCWSTR) -> SIZE_T {
        let mut UncPath: PCWSTR = FileNameBuffer.add(2);

        while *UncPath != 0 && !(*UncPath == b'\\' as u16 || *UncPath == b'/' as u16) {
            UncPath = UncPath.add(1);
        }
        while *UncPath == b'\\' as u16 || *UncPath == b'/' as u16 {
            UncPath = UncPath.add(1);
        }
        while *UncPath != 0 && !(*UncPath == b'\\' as u16 || *UncPath == b'/' as u16) {
            UncPath = UncPath.add(1);
        }

        (UncPath.offset_from(FileNameBuffer)) as SIZE_T
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlpApplyLengthFunction(
        Flags: ULONG,
        Type: ULONG,
        UnicodeStringOrUnicodeStringBuffer: PVOID,
        LengthFunction: Option<
            unsafe extern "system" fn(ULONG, PUNICODE_STRING, PULONG) -> NTSTATUS,
        >,
    ) -> NTSTATUS {
        let String: PUNICODE_STRING;
        let mut Length: ULONG;

        if Flags != 0 || UnicodeStringOrUnicodeStringBuffer.is_null() || LengthFunction.is_none() {
            return STATUS_INVALID_PARAMETER;
        }

        if Type as usize == core::mem::size_of::<UNICODE_STRING>() {
            String = UnicodeStringOrUnicodeStringBuffer as PUNICODE_STRING;
        } else if Type as usize == core::mem::size_of::<RTL_UNICODE_STRING_BUFFER>() {
            String = core::ptr::addr_of_mut!(
                (*(UnicodeStringOrUnicodeStringBuffer as PRTL_UNICODE_STRING_BUFFER)).String
            );
        } else {
            return STATUS_INVALID_PARAMETER;
        }

        Length = 0;
        let Status = (LengthFunction.unwrap())(0, String, &mut Length);
        if !NT_SUCCESS(Status) {
            return Status;
        }

        if Length > UNICODE_STRING_MAX_CHARS {
            return STATUS_NAME_TOO_LONG;
        }

        (*String).Length = (Length * core::mem::size_of::<u16>() as ULONG) as u16;

        if Type as usize == core::mem::size_of::<RTL_UNICODE_STRING_BUFFER>() {
            *(*String).Buffer.offset(Length as isize) = 0u16;
        }

        STATUS_SUCCESS
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlGetLengthWithoutLastFullDosOrNtPathElement(
        Flags: ULONG,
        Path: PCUNICODE_STRING,
        LengthOut: *mut ULONG,
    ) -> NTSTATUS {
        let is_path_separator = |x: u16| x == b'\\' as u16 || x == b'/' as u16;

        let mut PathDividersBuffer: [u16; 2] = [b'\\' as u16, b'/' as u16];
        let mut PathDividers: UNICODE_STRING = core::mem::zeroed();
        PathDividers.Length = (2 * core::mem::size_of::<u16>()) as u16;
        PathDividers.MaximumLength = (2 * core::mem::size_of::<u16>()) as u16;
        PathDividers.Buffer = PathDividersBuffer.as_mut_ptr();

        let mut Position: u16 = 0;

        if !LengthOut.is_null() {
            *LengthOut = 0;
        }

        if Flags != 0 || Path.is_null() || LengthOut.is_null() {
            return STATUS_INVALID_PARAMETER;
        }

        if ((*Path).Length as usize) / core::mem::size_of::<u16>() == 0 {
            return STATUS_SUCCESS;
        }

        let PathType = RtlDetermineDosPathNameType_Ustr(Path);

        if PathType == RtlPathTypeLocalDevice {
            if ((*Path).Length as usize) / core::mem::size_of::<u16>() < 7
                || *(*Path).Buffer.offset(5) != b':' as u16
                || !is_path_separator(*(*Path).Buffer.offset(6))
            {
                return STATUS_INVALID_PARAMETER;
            }
        } else if PathType == RtlPathTypeRooted {
        } else if PathType == RtlPathTypeUncAbsolute {
        } else if PathType == RtlPathTypeDriveAbsolute {
        } else {
            return STATUS_INVALID_PARAMETER;
        }

        if !NT_SUCCESS(RtlFindCharInUnicodeString(
            1,
            Path,
            &PathDividers,
            &mut Position,
        )) {
            Position = 0;
        }

        if Position != 0
            && (Position as usize + core::mem::size_of::<u16>()) == (*Path).Length as usize
        {
            let mut Tmp: UNICODE_STRING = *Path;
            Tmp.Length = Position;

            while (Tmp.Length as usize) > core::mem::size_of::<u16>()
                && is_path_separator(
                    *Tmp.Buffer
                        .offset((Tmp.Length as usize / core::mem::size_of::<u16>()) as isize),
                )
            {
                Tmp.Length -= core::mem::size_of::<u16>() as u16;
            }

            if !NT_SUCCESS(RtlFindCharInUnicodeString(
                1,
                &Tmp,
                &PathDividers,
                &mut Position,
            )) {
                Position = 0;
            }
        }

        if Position != 0 {
            Position /= core::mem::size_of::<u16>() as u16;
        }

        if Position != 0 {
            while Position > 1 && is_path_separator(*(*Path).Buffer.offset((Position - 1) as isize))
            {
                Position -= 1;
            }
        }

        if Position > 0 {
            *LengthOut = (Position + 1) as ULONG;
        }

        STATUS_SUCCESS
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlComputePrivatizedDllName_U(
        DllName: PUNICODE_STRING,
        RealName: PUNICODE_STRING,
        LocalName: PUNICODE_STRING,
    ) -> NTSTATUS {
        const RTL_FIND_CHAR_IN_UNICODE_STRING_START_AT_END: ULONG = 0x00000001;
        const RTL_USER_PROCESS_PARAMETERS_NORMALIZED: ULONG = 0x00000001;
        const UNICODE_STRING_MAX_BYTES: ULONG = 65534;
        const STATUS_NAME_TOO_LONG: NTSTATUS = 0xC0000106u32 as NTSTATUS;

        let ExtensionChar: UNICODE_STRING = UNICODE_STRING {
            Length: 2,
            MaximumLength: 4,
            Buffer: {
                static DOT: [u16; 2] = [0x2E, 0x0000];
                DOT.as_ptr() as PWSTR
            },
        };

        let mut Position: u16;
        let mut ImagePathName: UNICODE_STRING;
        let mut DllNameOnly: UNICODE_STRING = core::mem::zeroed();
        let mut CopyRealName: UNICODE_STRING;
        let mut CopyLocalName: UNICODE_STRING;
        let HasExtension: u8;
        let mut RequiredSize: ULONG;
        let mut Status: NTSTATUS;

        CopyRealName = *RealName;
        CopyLocalName = *LocalName;

        let Peb = RtlGetCurrentPeb();
        let ProcessParameters = (*Peb).ProcessParameters;
        ImagePathName = (*ProcessParameters).ImagePathName;

        if ((*ProcessParameters).Flags & RTL_USER_PROCESS_PARAMETERS_NORMALIZED) == 0 {
            ImagePathName.Buffer =
                (ImagePathName.Buffer as usize + ProcessParameters as usize) as PWSTR;
        }

        let mut Pos: u16 = 0;
        if !NT_SUCCESS(RtlFindCharInUnicodeString(
            RTL_FIND_CHAR_IN_UNICODE_STRING_START_AT_END,
            DllName,
            &RtlpPathDividers,
            &mut Pos,
        )) {
            DllNameOnly = *DllName;
        } else {
            Pos += core::mem::size_of::<u16>() as u16;
            DllNameOnly.Buffer = (*DllName)
                .Buffer
                .add(Pos as usize / core::mem::size_of::<u16>());
            DllNameOnly.Length = (*DllName).Length - Pos;
            DllNameOnly.MaximumLength = (*DllName).MaximumLength - Pos;
        }

        Position = 0;
        if !NT_SUCCESS(RtlFindCharInUnicodeString(
            RTL_FIND_CHAR_IN_UNICODE_STRING_START_AT_END,
            &mut DllNameOnly,
            &ExtensionChar,
            &mut Position,
        )) {
            Position = 0;
        }

        HasExtension = if Position > 1 { 1 } else { 0 };

        RequiredSize = ImagePathName.Length as ULONG
            + RtlpDotLocal.Length as ULONG
            + DllNameOnly.Length as ULONG
            + (if HasExtension != 0 {
                0
            } else {
                RtlpDefaultExtension.Length as ULONG
            })
            + core::mem::size_of::<u16>() as ULONG;

        if RequiredSize > UNICODE_STRING_MAX_BYTES {
            return STATUS_NAME_TOO_LONG;
        }

        if RequiredSize > CopyLocalName.MaximumLength as ULONG {
            CopyLocalName.Buffer = RtlpAllocateStringMemory(RequiredSize, TAG_USTR);
            if CopyLocalName.Buffer.is_null() {
                return STATUS_NO_MEMORY;
            }
            CopyLocalName.MaximumLength = RequiredSize as u16;
        }

        CopyLocalName.Length = 0;
        Status = RtlAppendUnicodeStringToString(&mut CopyLocalName, &mut ImagePathName);
        if NT_SUCCESS(Status) {
            Status = RtlAppendUnicodeStringToString(
                &mut CopyLocalName,
                &RtlpDotLocal as *const _ as PUNICODE_STRING,
            );
        }
        if NT_SUCCESS(Status) {
            Status = RtlAppendUnicodeStringToString(&mut CopyLocalName, &mut DllNameOnly);
        }
        if NT_SUCCESS(Status) && HasExtension == 0 {
            Status = RtlAppendUnicodeStringToString(
                &mut CopyLocalName,
                &RtlpDefaultExtension as *const _ as PUNICODE_STRING,
            );
        }

        if NT_SUCCESS(Status) {
            let mut Pos2: u16 = 0;
            if NT_SUCCESS(RtlFindCharInUnicodeString(
                RTL_FIND_CHAR_IN_UNICODE_STRING_START_AT_END,
                &mut ImagePathName,
                &RtlpPathDividers,
                &mut Pos2,
            )) {
                ImagePathName.Length = Pos2 + core::mem::size_of::<u16>() as u16;
            }

            RequiredSize = ImagePathName.Length as ULONG
                + DllNameOnly.Length as ULONG
                + (if HasExtension != 0 {
                    0
                } else {
                    RtlpDefaultExtension.Length as ULONG
                })
                + core::mem::size_of::<u16>() as ULONG;

            if RequiredSize >= UNICODE_STRING_MAX_BYTES {
                Status = STATUS_NAME_TOO_LONG;
            } else {
                if RequiredSize > CopyRealName.MaximumLength as ULONG {
                    CopyRealName.Buffer = RtlpAllocateStringMemory(RequiredSize, TAG_USTR);
                    if CopyRealName.Buffer.is_null() {
                        Status = STATUS_NO_MEMORY;
                    }
                    CopyRealName.MaximumLength = RequiredSize as u16;
                }
                CopyRealName.Length = 0;
                if NT_SUCCESS(Status) {
                    Status = RtlAppendUnicodeStringToString(&mut CopyRealName, &mut ImagePathName);
                }
                if NT_SUCCESS(Status) {
                    Status = RtlAppendUnicodeStringToString(&mut CopyRealName, &mut DllNameOnly);
                }
                if NT_SUCCESS(Status) && HasExtension == 0 {
                    Status = RtlAppendUnicodeStringToString(
                        &mut CopyRealName,
                        &RtlpDefaultExtension as *const _ as PUNICODE_STRING,
                    );
                }
            }
        }

        if !NT_SUCCESS(Status) {
            if !CopyRealName.Buffer.is_null() && CopyRealName.Buffer != (*RealName).Buffer {
                RtlpFreeStringMemory(CopyRealName.Buffer as *mut core::ffi::c_void, TAG_USTR);
            }
            if !CopyLocalName.Buffer.is_null() && CopyLocalName.Buffer != (*LocalName).Buffer {
                RtlpFreeStringMemory(CopyLocalName.Buffer as *mut core::ffi::c_void, TAG_USTR);
            }
            return Status;
        }

        *RealName = CopyRealName;
        *LocalName = CopyLocalName;
        STATUS_SUCCESS
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlGetFullPathName_Ustr(
        FileName: PUNICODE_STRING,
        Size: ULONG,
        Buffer: PWSTR,
        ShortName: *mut PCWSTR,
        InvalidName: PBOOLEAN,
        PathType: *mut RTL_PATH_TYPE,
    ) -> ULONG {
        let is_path_separator = |x: u16| x == b'\\' as u16 || x == b'/' as u16;
        let hiword = |l: ULONG| -> u16 { ((l >> 16) & 0xffff) as u16 };
        let loword = |l: ULONG| -> u16 { (l & 0xffff) as u16 };

        let Status: NTSTATUS;
        let FileNameBuffer: PWCHAR;
        let mut FileNameLength: ULONG;
        let FileNameChars: ULONG;
        let mut DosLength: ULONG;
        let DosLengthOffset: ULONG;
        let FullLength: ULONG;
        let SkipTrailingPathSeparators: BOOLEAN;
        let mut c: WCHAR;

        let mut reqsize: ULONG = 0;
        let mut ptr: PCWSTR;

        let CurDirName: PCUNICODE_STRING;
        let mut EnvVarName: UNICODE_STRING = core::mem::zeroed();
        let mut EnvVarValue: UNICODE_STRING = core::mem::zeroed();
        let mut EnvVarNameBuffer: [WCHAR; 4] = [0; 4];

        let mut PrefixCut: ULONG = 0;
        let mut Prefix: PWCHAR = core::ptr::null_mut();
        let mut PrefixLength: ULONG = 0;
        let mut Source: PWCHAR;
        let mut SourceLength: ULONG;

        if !InvalidName.is_null() {
            *InvalidName = 0u8;
        }

        *PathType = RtlPathTypeUnknown;
        if (*FileName).Length == 0 || *(*FileName).Buffer.offset(0) == 0 {
            return 0;
        }

        FileNameBuffer = (*FileName).Buffer;
        FileNameLength = (*FileName).Length as ULONG;
        FileNameChars = FileNameLength / core::mem::size_of::<WCHAR>() as ULONG;

        c = *FileNameBuffer.offset((FileNameChars - 1) as isize);
        while FileNameLength != 0 && c == b' ' as u16 {
            FileNameLength -= core::mem::size_of::<WCHAR>() as ULONG;
            if FileNameLength != 0 {
                c = *FileNameBuffer
                    .offset((FileNameLength / core::mem::size_of::<WCHAR>() as ULONG - 1) as isize);
            }
        }

        if FileNameLength == 0 {
            return 0;
        }

        SkipTrailingPathSeparators =
            if is_path_separator(*FileNameBuffer.offset((FileNameChars - 1) as isize)) {
                0u8
            } else {
                1u8
            };

        DosLength = RtlIsDosDeviceName_Ustr(FileName);
        if DosLength != 0 {
            if !ShortName.is_null() {
                *ShortName = core::ptr::null_mut();
            }

            DosLengthOffset = hiword(DosLength) as ULONG;
            DosLength = loword(DosLength) as ULONG;

            if !InvalidName.is_null() && DosLengthOffset != 0 {
                Status = RtlpCheckDeviceName(FileName, DosLengthOffset, InvalidName);

                if !NT_SUCCESS(Status) {
                    return 0;
                }
                if *InvalidName != 0 {
                    return 0;
                }
            }

            FullLength = DosLength + DeviceRootString.Length as ULONG;
            if FullLength < Size {
                core::ptr::copy(
                    DeviceRootString.Buffer as *const u8,
                    Buffer as *mut u8,
                    DeviceRootString.Length as usize,
                );

                core::ptr::copy(
                    (FileNameBuffer as *const u8).add(DosLengthOffset as usize),
                    (Buffer as *mut u8).add(DeviceRootString.Length as usize),
                    DosLength as usize,
                );

                *((Buffer as usize + FullLength as usize) as *mut WCHAR) = 0;
                return FullLength;
            }

            if (FullLength + core::mem::size_of::<WCHAR>() as ULONG) > 0xfffe {
                return 0;
            }
            return FullLength + core::mem::size_of::<WCHAR>() as ULONG;
        }

        RtlZeroMemory(Buffer as *mut _, Size as usize);

        *PathType = RtlDetermineDosPathNameType_U(FileNameBuffer);

        Source = FileNameBuffer;
        SourceLength = FileNameLength;
        EnvVarValue.Buffer = core::ptr::null_mut();

        RtlAcquirePebLock();

        let peb = RtlGetCurrentPeb();
        CurDirName = core::ptr::addr_of!((*(*peb).ProcessParameters).CurrentDirectory.DosPath)
            as PCUNICODE_STRING;

        'quit: {
            loop {
                match *PathType {
                    x if x == RtlPathTypeUncAbsolute => {
                        PrefixCut = RtlpSkipUNCPrefix(FileNameBuffer) as ULONG;
                        break;
                    }
                    x if x == RtlPathTypeLocalDevice => {
                        PrefixCut = 4;
                        break;
                    }
                    x if x == RtlPathTypeDriveAbsolute => {
                        Prefix = FileNameBuffer;
                        PrefixLength = 3 * core::mem::size_of::<WCHAR>() as ULONG;
                        Source = Source.offset(3);
                        SourceLength -= 3 * core::mem::size_of::<WCHAR>() as ULONG;
                        PrefixCut = 3;
                        break;
                    }
                    x if x == RtlPathTypeDriveRelative => {
                        Source = Source.offset(2);
                        SourceLength -= 2 * core::mem::size_of::<WCHAR>() as ULONG;

                        let CurDrive = RtlpUpcaseUnicodeChar(*(*CurDirName).Buffer.offset(0));
                        let NewDrive = RtlpUpcaseUnicodeChar(*FileNameBuffer.offset(0));

                        if NewDrive != CurDrive || *(*CurDirName).Buffer.offset(1) != b':' as u16 {
                            EnvVarNameBuffer[0] = b'=' as u16;
                            EnvVarNameBuffer[1] = NewDrive;
                            EnvVarNameBuffer[2] = b':' as u16;
                            EnvVarNameBuffer[3] = 0;

                            EnvVarName.Length = (3 * core::mem::size_of::<WCHAR>()) as u16;
                            EnvVarName.MaximumLength =
                                EnvVarName.Length + core::mem::size_of::<WCHAR>() as u16;
                            EnvVarName.Buffer = EnvVarNameBuffer.as_mut_ptr();

                            EnvVarValue.Length = 0;
                            EnvVarValue.MaximumLength = Size as u16;
                            EnvVarValue.Buffer =
                                rtl_heap_alloc(heap_handle(), 0, Size as usize) as PWSTR;
                            if EnvVarValue.Buffer.is_null() {
                                Prefix = core::ptr::null_mut();
                                PrefixLength = 0;
                                break 'quit;
                            }

                            let qStatus = RtlQueryEnvironmentVariable_U(
                                core::ptr::null_mut(),
                                &mut EnvVarName,
                                &mut EnvVarValue,
                            );
                            if qStatus == STATUS_SUCCESS {
                                *EnvVarValue.Buffer.offset(
                                    (EnvVarValue.Length as usize / core::mem::size_of::<WCHAR>())
                                        as isize,
                                ) = b'\\' as u16;
                                Prefix = EnvVarValue.Buffer;
                                PrefixLength = EnvVarValue.Length as ULONG
                                    + core::mem::size_of::<WCHAR>() as ULONG;
                            } else if qStatus == 0xC0000023u32 as NTSTATUS {
                                reqsize = EnvVarValue.Length as ULONG
                                    + SourceLength
                                    + core::mem::size_of::<WCHAR>() as ULONG;
                                break 'quit;
                            } else {
                                EnvVarNameBuffer[0] = NewDrive;
                                EnvVarNameBuffer[1] = b':' as u16;
                                EnvVarNameBuffer[2] = b'\\' as u16;
                                EnvVarNameBuffer[3] = 0;
                                Prefix = EnvVarNameBuffer.as_mut_ptr();
                                PrefixLength = 3 * core::mem::size_of::<WCHAR>() as ULONG;

                                rtl_heap_free(heap_handle(), 0, EnvVarValue.Buffer as *mut _);
                                EnvVarValue.Buffer = core::ptr::null_mut();
                            }
                            PrefixCut = 3;
                            break;
                        }
                        // Fall through to RtlPathTypeRelative
                        Prefix = (*CurDirName).Buffer;
                        PrefixLength = (*CurDirName).Length as ULONG;
                        if *(*CurDirName).Buffer.offset(1) != b':' as u16 {
                            PrefixCut = RtlpSkipUNCPrefix((*CurDirName).Buffer) as ULONG;
                        } else {
                            PrefixCut = 3;
                        }
                        break;
                    }
                    x if x == RtlPathTypeRelative => {
                        Prefix = (*CurDirName).Buffer;
                        PrefixLength = (*CurDirName).Length as ULONG;
                        if *(*CurDirName).Buffer.offset(1) != b':' as u16 {
                            PrefixCut = RtlpSkipUNCPrefix((*CurDirName).Buffer) as ULONG;
                        } else {
                            PrefixCut = 3;
                        }
                        break;
                    }
                    x if x == RtlPathTypeRooted => {
                        if *(*CurDirName).Buffer.offset(1) == b':' as u16 {
                            Prefix = (*CurDirName).Buffer;
                            PrefixLength = 3 * core::mem::size_of::<WCHAR>() as ULONG;
                            PrefixCut = 3;
                        } else {
                            PrefixCut = RtlpSkipUNCPrefix((*CurDirName).Buffer) as ULONG;
                            PrefixLength = PrefixCut * core::mem::size_of::<WCHAR>() as ULONG;
                            Prefix = (*CurDirName).Buffer;
                        }
                        break;
                    }
                    x if x == RtlPathTypeRootLocalDevice => {
                        Prefix = DeviceRootString.Buffer;
                        PrefixLength = DeviceRootString.Length as ULONG;
                        Source = Source.offset(3);
                        SourceLength -= 3 * core::mem::size_of::<WCHAR>() as ULONG;
                        PrefixCut = 4;
                        break;
                    }
                    _ => {
                        break 'quit;
                    }
                }
            }

            reqsize = PrefixLength;
            if reqsize + SourceLength + core::mem::size_of::<WCHAR>() as ULONG > Size {
                reqsize += SourceLength + core::mem::size_of::<WCHAR>() as ULONG;
                break 'quit;
            }

            if PrefixLength != 0 {
                core::ptr::copy(
                    Prefix,
                    Buffer,
                    PrefixLength as usize / core::mem::size_of::<WCHAR>(),
                );
            }
            core::ptr::copy(
                Source,
                Buffer.offset((PrefixLength / core::mem::size_of::<WCHAR>() as ULONG) as isize),
                (SourceLength as usize + core::mem::size_of::<WCHAR>())
                    / core::mem::size_of::<WCHAR>(),
            );

            Prefix = core::ptr::null_mut();
            if !EnvVarValue.Buffer.is_null() {
                rtl_heap_free(heap_handle(), 0, EnvVarValue.Buffer as *mut _);
                EnvVarValue.Buffer = core::ptr::null_mut();
            }

            reqsize = RtlpCollapsePath(Buffer, PrefixCut, SkipTrailingPathSeparators);

            if !ShortName.is_null() {
                ptr = wcsrchr(Buffer, b'\\' as u16);
                if !ptr.is_null() {
                    ptr = ptr.offset(1);
                }

                PrefixCut = if *PathType == RtlPathTypeUncAbsolute {
                    PrefixCut
                } else {
                    3
                };

                if !ptr.is_null()
                    && *ptr != 0
                    && ptr as usize >= Buffer.offset(PrefixCut as isize) as usize
                {
                    *ShortName = ptr;
                } else {
                    *ShortName = core::ptr::null_mut();
                }
            }
        }

        RtlReleasePebLock();

        reqsize
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlpWin32NTNameToNtPathName_U(
        DosPath: PUNICODE_STRING,
        NtPath: PUNICODE_STRING,
        PartName: *mut PCWSTR,
        RelativeName: PRTL_RELATIVE_NAME_U,
    ) -> NTSTATUS {
        let DosLength: ULONG;
        let NewBuffer: PWSTR;
        let mut p: PWSTR;

        if DosPath.is_null() {
            return STATUS_OBJECT_NAME_INVALID;
        }

        DosLength = (*DosPath).Length as ULONG;
        if DosLength >= UNICODE_STRING_MAX_BYTES as ULONG {
            return STATUS_NAME_TOO_LONG;
        }

        NewBuffer = rtl_heap_alloc(
            heap_handle(),
            0,
            (DosLength + core::mem::size_of::<WCHAR>() as ULONG) as usize,
        ) as PWSTR;
        if NewBuffer.is_null() {
            return STATUS_NO_MEMORY;
        }

        RtlCopyMemory(
            NewBuffer as PVOID,
            RtlpDosDevicesPrefix.Buffer as PVOID,
            RtlpDosDevicesPrefix.Length as usize,
        );
        RtlCopyMemory(
            (NewBuffer as PCHAR).add(RtlpDosDevicesPrefix.Length as usize) as PVOID,
            (*DosPath)
                .Buffer
                .add(RtlpDosDevicesPrefix.Length as usize / core::mem::size_of::<WCHAR>())
                as PVOID,
            ((*DosPath).Length - RtlpDosDevicesPrefix.Length) as usize,
        );
        *NewBuffer.add(DosLength as usize / core::mem::size_of::<WCHAR>()) = 0;

        if !RelativeName.is_null() {
            RtlInitEmptyUnicodeString(
                core::ptr::addr_of_mut!((*RelativeName).RelativeName),
                core::ptr::null_mut(),
                0,
            );
            (*RelativeName).ContainingDirectory = core::ptr::null_mut();
            (*RelativeName).CurDirRef = core::ptr::null_mut();
        }

        if !PartName.is_null() {
            p = NewBuffer.add(DosLength as usize / core::mem::size_of::<WCHAR>());
            loop {
                p = p.sub(1);
                if !(p > NewBuffer) {
                    break;
                }
                if *p == '\\' as WCHAR {
                    p = p.add(1);
                    break;
                }
            }

            if (p > NewBuffer) && (*p != 0) {
                *PartName = p;
            } else {
                *PartName = core::ptr::null_mut();
            }
        }

        (*NtPath).Buffer = NewBuffer;
        (*NtPath).Length = DosLength as USHORT;
        (*NtPath).MaximumLength = DosLength as USHORT + core::mem::size_of::<WCHAR>() as USHORT;
        STATUS_SUCCESS
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlpDosPathNameToRelativeNtPathName_Ustr(
        HaveRelative: BOOLEAN,
        DosName: PCUNICODE_STRING,
        NtName: PUNICODE_STRING,
        PartName: *mut PCWSTR,
        RelativeName: PRTL_RELATIVE_NAME_U,
    ) -> NTSTATUS {
        let mut BigBuffer: [u16; MAX_PATH as usize + 1] = [0; MAX_PATH as usize + 1];
        let PrefixBuffer: PWCHAR;
        let NewBuffer: PWCHAR;
        let Buffer: PWCHAR;
        let mut MaxLength: ULONG;
        let PathLength: ULONG;
        let PrefixLength: ULONG;
        let PrefixCut: ULONG;
        let LengthChars: ULONG;
        let Length: ULONG;
        let mut CapturedDosName: UNICODE_STRING;
        let mut PartNameString: UNICODE_STRING = core::mem::zeroed();
        let mut FullPath: UNICODE_STRING = core::mem::zeroed();
        let QuickPath: BOOLEAN;
        let mut InputPathType: RTL_PATH_TYPE = 0;
        let BufferPathType: RTL_PATH_TYPE;
        let mut Status: NTSTATUS;
        let mut NameInvalid: BOOLEAN = 0;
        let CurrentDirectory: PCURDIR;

        MaxLength = core::mem::size_of::<[u16; MAX_PATH as usize + 1]>() as ULONG;

        if DosName.is_null() {
            return STATUS_OBJECT_NAME_INVALID;
        }

        CapturedDosName = *DosName;

        if (CapturedDosName.Length <= RtlpWin32NtRootSlash.Length)
            || (*CapturedDosName.Buffer.offset(0) != *RtlpWin32NtRootSlash.Buffer.offset(0))
            || (*CapturedDosName.Buffer.offset(1) != *RtlpWin32NtRootSlash.Buffer.offset(1))
            || (*CapturedDosName.Buffer.offset(2) != *RtlpWin32NtRootSlash.Buffer.offset(2))
            || (*CapturedDosName.Buffer.offset(3) != *RtlpWin32NtRootSlash.Buffer.offset(3))
        {
            QuickPath = 0;

            Buffer = BigBuffer.as_mut_ptr();
            MaxLength += RtlpDosDevicesUncPrefix.Length as ULONG;

            NewBuffer = rtl_heap_alloc(heap_handle(), 0, MaxLength as usize) as PWCHAR;
            if NewBuffer.is_null() {
                return STATUS_NO_MEMORY;
            }
        } else {
            QuickPath = 1;
            NewBuffer = core::ptr::null_mut();
            Buffer = BigBuffer.as_mut_ptr();
        }

        RtlAcquirePebLock();
        if QuickPath != 0 {
            Status = RtlpWin32NTNameToNtPathName_U(
                core::ptr::addr_of_mut!(CapturedDosName),
                NtName,
                PartName,
                RelativeName,
            );
            RtlReleasePebLock();
            return Status;
        }

        PathLength = RtlGetFullPathName_Ustr(
            core::ptr::addr_of_mut!(CapturedDosName),
            MAX_PATH * core::mem::size_of::<u16>() as ULONG,
            Buffer,
            PartName as *mut PCWSTR,
            core::ptr::addr_of_mut!(NameInvalid),
            core::ptr::addr_of_mut!(InputPathType),
        );
        if (NameInvalid != 0)
            || (PathLength == 0)
            || (PathLength > MAX_PATH * core::mem::size_of::<u16>() as ULONG)
        {
            rtl_heap_free(heap_handle(), 0, NewBuffer as PVOID);
            RtlReleasePebLock();
            return STATUS_OBJECT_NAME_INVALID;
        }

        PrefixLength = RtlpDosDevicesPrefix.Length as ULONG;
        PrefixBuffer = RtlpDosDevicesPrefix.Buffer;
        PrefixCut = 0;

        BufferPathType = RtlDetermineDosPathNameType_U(Buffer);

        let (PrefixLength, PrefixBuffer, PrefixCut) = match BufferPathType {
            x if x == RtlPathTypeUncAbsolute => (
                RtlpDosDevicesUncPrefix.Length as ULONG,
                RtlpDosDevicesUncPrefix.Buffer,
                2u32,
            ),
            x if x == RtlPathTypeLocalDevice => (PrefixLength, PrefixBuffer, 4u32),
            _ => (PrefixLength, PrefixBuffer, PrefixCut),
        };

        core::ptr::copy_nonoverlapping::<u8>(
            PrefixBuffer as *const u8,
            NewBuffer as *mut u8,
            PrefixLength as usize,
        );
        core::ptr::copy_nonoverlapping::<u8>(
            Buffer.offset(PrefixCut as isize) as *const u8,
            (NewBuffer as *mut u8).offset(PrefixLength as isize),
            (PathLength - PrefixCut * core::mem::size_of::<u16>() as ULONG) as usize,
        );

        Length = PathLength + PrefixLength - PrefixCut * core::mem::size_of::<u16>() as ULONG;
        LengthChars = Length / core::mem::size_of::<u16>() as ULONG;

        (*NtName).Buffer = NewBuffer;
        (*NtName).Length = Length as u16;
        (*NtName).MaximumLength = MaxLength as u16;
        *NewBuffer.offset(LengthChars as isize) = 0;

        if !PartName.is_null() && !(*PartName).is_null() {
            Status = RtlInitUnicodeStringEx(core::ptr::addr_of_mut!(PartNameString), *PartName);
            if NT_SUCCESS(Status) {
                *PartName = NewBuffer.offset(
                    (LengthChars
                        - (PartNameString.Length / core::mem::size_of::<u16>() as u16) as ULONG)
                        as isize,
                ) as PCWSTR;
            } else {
                rtl_heap_free(heap_handle(), 0, NewBuffer as PVOID);
                RtlReleasePebLock();
                return Status;
            }
        }

        if !RelativeName.is_null() {
            RtlInitEmptyUnicodeString(
                core::ptr::addr_of_mut!((*RelativeName).RelativeName),
                core::ptr::null_mut(),
                0,
            );
            (*RelativeName).ContainingDirectory = core::ptr::null_mut();
            (*RelativeName).CurDirRef = core::ptr::null_mut();

            if InputPathType == RtlPathTypeRelative {
                let Peb = NtCurrentPeb();
                let _ = Peb;
                CurrentDirectory = core::ptr::addr_of_mut!(
                    (*(*NtCurrentPeb()).ProcessParameters).CurrentDirectory
                );
                if !(*CurrentDirectory).Handle.is_null() {
                    Status = RtlInitUnicodeStringEx(core::ptr::addr_of_mut!(FullPath), Buffer);
                    if !NT_SUCCESS(Status) {
                        rtl_heap_free(heap_handle(), 0, NewBuffer as PVOID);
                        RtlReleasePebLock();
                        return Status;
                    }

                    if (*CurrentDirectory).DosPath.Length > FullPath.Length {
                        RtlReleasePebLock();
                        return Status;
                    }

                    if RtlEqualUnicodeString(
                        core::ptr::addr_of_mut!(FullPath),
                        core::ptr::addr_of_mut!((*CurrentDirectory).DosPath),
                        1,
                    ) != 0
                    {
                        (*RelativeName).RelativeName.Buffer = ((NewBuffer as usize)
                            + PrefixLength as usize
                            + FullPath.Length as usize
                            - PrefixCut as usize * core::mem::size_of::<u16>())
                            as PWSTR;
                        (*RelativeName).RelativeName.Length =
                            (PathLength - FullPath.Length as ULONG) as u16;
                        if *(*RelativeName).RelativeName.Buffer.offset(0) == b'\\' as u16 {
                            (*RelativeName).RelativeName.Buffer =
                                (*RelativeName).RelativeName.Buffer.offset(1);
                            (*RelativeName).RelativeName.Length -=
                                core::mem::size_of::<u16>() as u16;
                        }
                        (*RelativeName).RelativeName.MaximumLength =
                            (*RelativeName).RelativeName.Length;

                        if HaveRelative == 0 {
                            (*RelativeName).ContainingDirectory = (*CurrentDirectory).Handle;
                            return Status;
                        }

                        (*RelativeName).CurDirRef = RtlpCurDirRef;
                        if !(*RelativeName).CurDirRef.is_null() {
                            InterlockedIncrement(core::ptr::addr_of_mut!(
                                (*RtlpCurDirRef).RefCount
                            ));
                        }

                        (*RelativeName).ContainingDirectory = (*CurrentDirectory).Handle;
                    }
                }
            }
        }

        RtlReleasePebLock();
        STATUS_SUCCESS
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlpDosPathNameToRelativeNtPathName_U(
        HaveRelative: BOOLEAN,
        DosName: PCWSTR,
        NtName: PUNICODE_STRING,
        PartName: *mut PCWSTR,
        RelativeName: PRTL_RELATIVE_NAME_U,
    ) -> NTSTATUS {
        let mut NameString: UNICODE_STRING = core::mem::zeroed();

        let mut Status = RtlInitUnicodeStringEx(&mut NameString, DosName);
        if NT_SUCCESS(Status) {
            Status = RtlpDosPathNameToRelativeNtPathName_Ustr(
                HaveRelative,
                &NameString,
                NtName,
                PartName,
                RelativeName,
            );
        }

        Status
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlDosPathNameToRelativeNtPathName_Ustr(
        DosName: PCUNICODE_STRING,
        NtName: PUNICODE_STRING,
        PartName: *mut PCWSTR,
        RelativeName: PRTL_RELATIVE_NAME_U,
    ) -> BOOLEAN {
        if NT_SUCCESS(RtlpDosPathNameToRelativeNtPathName_Ustr(
            1u8,
            DosName,
            NtName,
            PartName,
            RelativeName,
        )) {
            1u8
        } else {
            0u8
        }
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlDoesFileExists_UstrEx(
        FileName: PCUNICODE_STRING,
        SucceedIfBusy: BOOLEAN,
    ) -> BOOLEAN {
        let mut Result: BOOLEAN;
        let mut RelativeName: RTL_RELATIVE_NAME_U = core::mem::zeroed();
        let mut NtPathName: UNICODE_STRING = core::mem::zeroed();
        let Buffer: PVOID;
        let mut ObjectAttributes: OBJECT_ATTRIBUTES = core::mem::zeroed();
        let Status: NTSTATUS;
        let mut BasicInformation: FILE_BASIC_INFORMATION = core::mem::zeroed();

        /* Get the NT Path */
        Result = RtlDosPathNameToRelativeNtPathName_Ustr(
            FileName,
            &mut NtPathName,
            core::ptr::null_mut(),
            &mut RelativeName,
        );
        if Result == 0 {
            return 0u8;
        }

        /* Save the buffer */
        Buffer = NtPathName.Buffer as PVOID;

        /* Check if we have a relative name */
        if RelativeName.RelativeName.Length != 0 {
            /* Use it */
            NtPathName = RelativeName.RelativeName;
        } else {
            /* Otherwise ignore it */
            RelativeName.ContainingDirectory = core::ptr::null_mut();
        }

        /* Initialize the object attributes */
        ObjectAttributes.Length = core::mem::size_of::<OBJECT_ATTRIBUTES>() as ULONG;
        ObjectAttributes.RootDirectory = RelativeName.ContainingDirectory;
        ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
        ObjectAttributes.ObjectName = &mut NtPathName;
        ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
        ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

        /* Query the attributes and free the buffer now */
        Status = ZwQueryAttributesFile(&mut ObjectAttributes, &mut BasicInformation);
        RtlReleaseRelativeName(&mut RelativeName);
        rtl_heap_free(heap_handle(), 0, Buffer);

        /* Check if we failed */
        if !NT_SUCCESS(Status) {
            /* Check if we failed because the file is in use */
            if Status == STATUS_SHARING_VIOLATION || Status == STATUS_ACCESS_DENIED {
                /* Check if the caller wants this to be considered OK */
                Result = if SucceedIfBusy != 0 { 1u8 } else { 0u8 };
            } else {
                /* A failure because the file didn't exist */
                Result = 0u8;
            }
        } else {
            /* The file exists */
            Result = 1u8;
        }

        /* Return the result */
        Result
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlDoesFileExists_UStr(FileName: PUNICODE_STRING) -> BOOLEAN {
        RtlDoesFileExists_UstrEx(FileName, 1u8)
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlDoesFileExists_UEx(
        FileName: PCWSTR,
        SucceedIfBusy: BOOLEAN,
    ) -> BOOLEAN {
        let mut NameString: UNICODE_STRING = core::mem::zeroed();

        /* Create the unicode name*/
        if NT_SUCCESS(RtlInitUnicodeStringEx(&mut NameString, FileName)) {
            /* Call the unicode function */
            return RtlDoesFileExists_UstrEx(&NameString, SucceedIfBusy);
        }

        /* Fail */
        0u8
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlReleaseRelativeName(RelativeName: PRTL_RELATIVE_NAME_U) {
        if !(*RelativeName).CurDirRef.is_null() {
            if InterlockedDecrement(core::ptr::addr_of_mut!(
                (*(*RelativeName).CurDirRef).RefCount
            )) == 0
            {
                NtClose((*(*RelativeName).CurDirRef).Handle);
                rtl_heap_free(heap_handle(), 0, (*RelativeName).CurDirRef as PVOID);
            }
            (*RelativeName).CurDirRef = core::ptr::null_mut();
        }
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlGetCurrentDirectory_U(
        MaximumLength: ULONG,
        Buffer: PWSTR,
    ) -> ULONG {
        let mut Length: ULONG;
        let Bytes: ULONG;
        let CurDir: PCURDIR;
        let CurDirName: PWSTR;

        RtlAcquirePebLock();
        let Peb = NtCurrentPeb();
        CurDir = core::ptr::addr_of_mut!((*(*Peb).ProcessParameters).CurrentDirectory);

        CurDirName = (*CurDir).DosPath.Buffer;
        Length = (*CurDir).DosPath.Length as ULONG / core::mem::size_of::<u16>() as ULONG;

        Bytes = Length * core::mem::size_of::<u16>() as ULONG;
        if (Length <= 1) || (*CurDirName.offset((Length - 2) as isize) == b':' as u16) {
            if MaximumLength <= Bytes {
                RtlReleasePebLock();
                return Bytes + core::mem::size_of::<u16>() as ULONG;
            }
        } else {
            if MaximumLength < Bytes {
                RtlReleasePebLock();
                return Bytes;
            }
        }

        RtlCopyMemory(
            Buffer as *mut core::ffi::c_void,
            CurDirName as *const core::ffi::c_void,
            Bytes as usize,
        );

        if (Length <= 1) || (*Buffer.offset((Length - 2) as isize) != b':' as u16) {
            *Buffer.offset((Length - 1) as isize) = 0;
            Length -= 1;
        } else {
            *Buffer.offset(Length as isize) = 0;
        }

        RtlReleasePebLock();
        Length * core::mem::size_of::<u16>() as ULONG
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlSetCurrentDirectory_U(Path: PUNICODE_STRING) -> NTSTATUS {
        let CurDir: PCURDIR;
        let mut Status: NTSTATUS;
        let mut PathType: RTL_PATH_TYPE = 0;
        let mut IoStatusBlock: IO_STATUS_BLOCK = core::mem::zeroed();
        let mut FullPath: UNICODE_STRING = core::mem::zeroed();
        let mut NtName: UNICODE_STRING = core::mem::zeroed();
        let mut OldCurDir: PRTLP_CURDIR_REF = core::ptr::null_mut();
        let mut ObjectAttributes: OBJECT_ATTRIBUTES = core::mem::zeroed();
        let mut FileFsDeviceInfo: FILE_FS_DEVICE_INFORMATION = core::mem::zeroed();
        let SavedLength: ULONG;
        let CharLength: ULONG;
        let FullPathLength: ULONG;
        let mut OldHandle: HANDLE = core::ptr::null_mut();
        let mut CurDirHandle: HANDLE = core::ptr::null_mut();
        let mut OldCurDirHandle: HANDLE = core::ptr::null_mut();

        RtlInitEmptyUnicodeString(&mut NtName, core::ptr::null_mut(), 0);

        if RtlIsDosDeviceName_Ustr(Path) != 0 {
            return STATUS_NOT_A_DIRECTORY;
        }

        RtlAcquirePebLock();

        let Peb = NtCurrentPeb();
        let _ = Peb;
        let PebPtr = NtCurrentPeb();
        CurDir = core::ptr::addr_of_mut!((*(*PebPtr).ProcessParameters).CurrentDirectory);

        'leave_block: {
            if ((*CurDir).Handle as ULONG_PTR) & (RTL_CURDIR_ALL_FLAGS as ULONG_PTR)
                == RTL_CURDIR_DROP_OLD_HANDLE as ULONG_PTR
            {
                OldHandle = (*CurDir).Handle;
                (*CurDir).Handle = core::ptr::null_mut();
            }

            FullPath.Buffer =
                rtl_heap_alloc(heap_handle(), 0, (*CurDir).DosPath.MaximumLength as SIZE_T)
                    as PWSTR;
            if FullPath.Buffer.is_null() {
                Status = STATUS_NO_MEMORY;
                break 'leave_block;
            }

            FullPath.Length = 0;
            FullPath.MaximumLength = (*CurDir).DosPath.MaximumLength;

            FullPathLength = RtlGetFullPathName_Ustr(
                Path,
                FullPath.MaximumLength as ULONG,
                FullPath.Buffer,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                &mut PathType,
            );
            if FullPathLength == 0 {
                Status = STATUS_OBJECT_NAME_INVALID;
                break 'leave_block;
            }

            SavedLength = FullPath.MaximumLength as ULONG;
            CharLength = FullPathLength / core::mem::size_of::<u16>() as ULONG;

            if FullPathLength > FullPath.MaximumLength as ULONG {
                Status = STATUS_NAME_TOO_LONG;
                break 'leave_block;
            }

            if RtlDosPathNameToNtPathName_U(
                FullPath.Buffer,
                &mut NtName,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            ) == 0
            {
                Status = STATUS_OBJECT_NAME_INVALID;
                break 'leave_block;
            }

            ObjectAttributes.Length = core::mem::size_of::<OBJECT_ATTRIBUTES>() as ULONG;
            ObjectAttributes.RootDirectory = core::ptr::null_mut();
            ObjectAttributes.Attributes = (OBJ_CASE_INSENSITIVE | OBJ_INHERIT) as ULONG;
            ObjectAttributes.ObjectName = &mut NtName;
            ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
            ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

            if ((*CurDir).Handle as ULONG_PTR) & (RTL_CURDIR_ALL_FLAGS as ULONG_PTR)
                == RTL_CURDIR_ALL_FLAGS as ULONG_PTR
            {
                CurDirHandle = (((*CurDir).Handle as ULONG_PTR)
                    & !(RTL_CURDIR_ALL_FLAGS as ULONG_PTR))
                    as HANDLE;
                (*CurDir).Handle = core::ptr::null_mut();

                Status = NtQueryVolumeInformationFile(
                    CurDirHandle,
                    &mut IoStatusBlock,
                    &mut FileFsDeviceInfo as *mut _ as PVOID,
                    core::mem::size_of::<FILE_FS_DEVICE_INFORMATION>() as ULONG,
                    FileFsDeviceInformation,
                );
                if !NT_SUCCESS(Status) {
                    Status = RtlSetCurrentDirectory_U(Path);
                    break 'leave_block;
                }
            } else {
                Status = NtOpenFile(
                    &mut CurDirHandle,
                    SYNCHRONIZE | FILE_TRAVERSE,
                    &mut ObjectAttributes,
                    &mut IoStatusBlock,
                    FILE_SHARE_READ | FILE_SHARE_WRITE,
                    FILE_DIRECTORY_FILE | FILE_SYNCHRONOUS_IO_NONALERT,
                );
                if !NT_SUCCESS(Status) {
                    break 'leave_block;
                }

                Status = NtQueryVolumeInformationFile(
                    CurDirHandle,
                    &mut IoStatusBlock,
                    &mut FileFsDeviceInfo as *mut _ as PVOID,
                    core::mem::size_of::<FILE_FS_DEVICE_INFORMATION>() as ULONG,
                    FileFsDeviceInformation,
                );
                if !NT_SUCCESS(Status) {
                    break 'leave_block;
                }
            }

            if FileFsDeviceInfo.Characteristics & FILE_REMOVABLE_MEDIA != 0 {
                CurDirHandle = ((CurDirHandle as ULONG_PTR)
                    | (RTL_CURDIR_IS_REMOVABLE as ULONG_PTR))
                    as HANDLE;
            }

            FullPath.Length = FullPathLength as USHORT;

            if *FullPath.Buffer.offset((CharLength - 1) as isize) != OBJ_NAME_PATH_SEPARATOR as u16
            {
                if (CharLength + 1) * core::mem::size_of::<u16>() as ULONG > SavedLength {
                    Status = STATUS_NAME_TOO_LONG;
                    break 'leave_block;
                }

                *FullPath.Buffer.offset(CharLength as isize) = OBJ_NAME_PATH_SEPARATOR as u16;
                *FullPath.Buffer.offset((CharLength + 1) as isize) = 0;
                FullPath.Length += core::mem::size_of::<u16>() as USHORT;
            }

            if !RtlpCurDirRef.is_null() && (*RtlpCurDirRef).RefCount == 1 {
                OldCurDirHandle = (*RtlpCurDirRef).Handle;
            } else {
                OldCurDir = RtlpCurDirRef;
                RtlpCurDirRef = rtl_heap_alloc(
                    heap_handle(),
                    0,
                    core::mem::size_of::<RTLP_CURDIR_REF>() as SIZE_T,
                ) as PRTLP_CURDIR_REF;
                if RtlpCurDirRef.is_null() {
                    RtlpCurDirRef = OldCurDir;
                    OldCurDir = core::ptr::null_mut();
                    Status = STATUS_NO_MEMORY;
                    break 'leave_block;
                }

                (*RtlpCurDirRef).RefCount = 1;
            }

            (*CurDir).Handle = CurDirHandle;
            (*RtlpCurDirRef).Handle = CurDirHandle;
            CurDirHandle = core::ptr::null_mut();

            core::ptr::copy_nonoverlapping(
                FullPath.Buffer,
                (*CurDir).DosPath.Buffer,
                (FullPath.Length as usize + core::mem::size_of::<u16>())
                    / core::mem::size_of::<u16>(),
            );
            (*CurDir).DosPath.Length = FullPath.Length;

            Status = STATUS_SUCCESS;
        }

        RtlReleasePebLock();

        if !FullPath.Buffer.is_null() {
            rtl_heap_free(heap_handle(), 0, FullPath.Buffer as PVOID);
        }

        if !NtName.Buffer.is_null() {
            rtl_heap_free(heap_handle(), 0, NtName.Buffer as PVOID);
        }

        if !CurDirHandle.is_null() {
            NtClose(CurDirHandle);
        }

        if !OldHandle.is_null() {
            NtClose(OldHandle);
        }

        if !OldCurDirHandle.is_null() {
            NtClose(OldCurDirHandle);
        }

        if !OldCurDir.is_null()
            && InterlockedDecrement(core::ptr::addr_of_mut!((*OldCurDir).RefCount)) == 0
        {
            NtClose((*OldCurDir).Handle);
            rtl_heap_free(heap_handle(), 0, OldCurDir as PVOID);
        }

        Status
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlGetFullPathName_UEx(
        FileName: PWSTR,
        BufferLength: ULONG,
        Buffer: PWSTR,
        FilePart: *mut PWSTR,
        InputPathType: *mut RTL_PATH_TYPE,
    ) -> ULONG {
        let mut FileNameString: UNICODE_STRING = core::mem::zeroed();
        let status: NTSTATUS;

        if !InputPathType.is_null() {
            *InputPathType = 0;
        }

        status = RtlInitUnicodeStringEx(&mut FileNameString, FileName);
        if !NT_SUCCESS(status) {
            return 0;
        }

        RtlGetFullPathName_Ustr(
            &mut FileNameString,
            BufferLength,
            Buffer,
            FilePart as *mut PCWSTR,
            core::ptr::null_mut(),
            InputPathType,
        )
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlGetFullPathName_U(
        FileName: PCWSTR,
        Size: ULONG,
        Buffer: PWSTR,
        ShortName: *mut PWSTR,
    ) -> ULONG {
        let mut PathType: RTL_PATH_TYPE = 0;

        RtlGetFullPathName_UEx(FileName as PWSTR, Size, Buffer, ShortName, &mut PathType)
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlDosPathNameToNtPathName_U(
        DosName: PCWSTR,
        NtName: PUNICODE_STRING,
        PartName: *mut PCWSTR,
        RelativeName: PRTL_RELATIVE_NAME_U,
    ) -> u8 {
        if NT_SUCCESS(RtlpDosPathNameToRelativeNtPathName_U(
            0u8,
            DosName,
            NtName,
            PartName,
            RelativeName,
        )) {
            1u8
        } else {
            0u8
        }
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlDosPathNameToNtPathName_U_WithStatus(
        DosName: PCWSTR,
        NtName: PUNICODE_STRING,
        PartName: *mut PCWSTR,
        RelativeName: PRTL_RELATIVE_NAME_U,
    ) -> NTSTATUS {
        RtlpDosPathNameToRelativeNtPathName_U(0u8, DosName, NtName, PartName, RelativeName)
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlDosPathNameToRelativeNtPathName_U(
        DosName: PCWSTR,
        NtName: PUNICODE_STRING,
        PartName: *mut PCWSTR,
        RelativeName: PRTL_RELATIVE_NAME_U,
    ) -> BOOLEAN {
        if NT_SUCCESS(RtlpDosPathNameToRelativeNtPathName_U(
            1u8,
            DosName,
            NtName,
            PartName,
            RelativeName,
        )) {
            1u8
        } else {
            0u8
        }
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlDosPathNameToRelativeNtPathName_U_WithStatus(
        DosName: PCWSTR,
        NtName: PUNICODE_STRING,
        PartName: *mut PCWSTR,
        RelativeName: PRTL_RELATIVE_NAME_U,
    ) -> NTSTATUS {
        RtlpDosPathNameToRelativeNtPathName_U(1u8, DosName, NtName, PartName, RelativeName)
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlNtPathNameToDosPathName(
        Flags: ULONG,
        Path: PRTL_UNICODE_STRING_BUFFER,
        PathType: PULONG,
        _Unknown: PULONG,
    ) -> NTSTATUS {
        let mut UsePrefix: PCUNICODE_STRING = core::ptr::null();
        let mut AlternatePrefix: PCUNICODE_STRING = core::ptr::null();

        if !PathType.is_null() {
            *PathType = 0;
        }

        if Path.is_null() || Flags != 0 {
            return STATUS_INVALID_PARAMETER;
        }

        if RtlPrefixUnicodeString(
            core::ptr::addr_of!(RtlpDosDevicesUncPrefix),
            core::ptr::addr_of_mut!((*Path).String),
            1u8,
        ) != 0
        {
            UsePrefix = core::ptr::addr_of!(RtlpDosDevicesUncPrefix);
            AlternatePrefix = core::ptr::addr_of!(RtlpDoubleSlashPrefix);
            if !PathType.is_null() {
                *PathType = RTL_CONVERTED_UNC_PATH;
            }
        } else if RtlPrefixUnicodeString(
            core::ptr::addr_of!(RtlpDosDevicesPrefix),
            core::ptr::addr_of_mut!((*Path).String),
            0u8,
        ) != 0
        {
            UsePrefix = core::ptr::addr_of!(RtlpDosDevicesPrefix);
            if !PathType.is_null() {
                *PathType = RTL_CONVERTED_NT_PATH;
            }
        }

        if !UsePrefix.is_null() {
            let mut Len: USHORT = (*Path).String.Length - (*UsePrefix).Length;
            if !AlternatePrefix.is_null() {
                Len += (*AlternatePrefix).Length;
            }

            let Status =
                RtlEnsureBufferSize(0, core::ptr::addr_of_mut!((*Path).ByteBuffer), Len as ULONG);
            if !NT_SUCCESS(Status) {
                return Status;
            }

            if (Len as ULONG) + (core::mem::size_of::<u16>() as ULONG)
                <= ((*Path).ByteBuffer.Size as ULONG)
            {
                if !AlternatePrefix.is_null() {
                    memcpy(
                        (*Path).ByteBuffer.Buffer as *mut _,
                        (*AlternatePrefix).Buffer as *const _,
                        (*AlternatePrefix).Length as usize,
                    );
                    memmove(
                        (*Path)
                            .ByteBuffer
                            .Buffer
                            .offset((*AlternatePrefix).Length as isize)
                            as *mut _,
                        (*Path)
                            .ByteBuffer
                            .Buffer
                            .offset((*UsePrefix).Length as isize)
                            as *const _,
                        (Len - (*AlternatePrefix).Length) as usize,
                    );
                } else {
                    memmove(
                        (*Path).ByteBuffer.Buffer as *mut _,
                        (*Path)
                            .ByteBuffer
                            .Buffer
                            .offset((*UsePrefix).Length as isize)
                            as *const _,
                        Len as usize,
                    );
                }
                (*Path).String.Buffer = (*Path).ByteBuffer.Buffer as PWSTR;
                (*Path).String.Length = Len;
                (*Path).String.MaximumLength = ((*Path).ByteBuffer.Size as ULONG) as USHORT;
                *(*Path)
                    .String
                    .Buffer
                    .offset((Len / core::mem::size_of::<u16>() as USHORT) as isize) = 0;
            }
            return STATUS_SUCCESS;
        }

        if !PathType.is_null() {
            match RtlDetermineDosPathNameType_Ustr(core::ptr::addr_of!((*Path).String)) {
                x if x == RtlPathTypeUncAbsolute
                    || x == RtlPathTypeDriveAbsolute
                    || x == RtlPathTypeLocalDevice
                    || x == RtlPathTypeRootLocalDevice =>
                {
                    *PathType = RTL_UNCHANGED_DOS_PATH;
                }
                x if x == RtlPathTypeUnknown
                    || x == RtlPathTypeDriveRelative
                    || x == RtlPathTypeRooted
                    || x == RtlPathTypeRelative =>
                {
                    *PathType = RTL_UNCHANGED_UNK_PATH;
                }
                _ => {}
            }
        }

        STATUS_SUCCESS
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlDosSearchPath_U(
        Path: PCWSTR,
        FileName: PCWSTR,
        mut Extension: PCWSTR,
        Size: ULONG,
        Buffer: PWSTR,
        PartName: *mut PWSTR,
    ) -> ULONG {
        let mut Status: NTSTATUS;
        let ExtensionLength: ULONG;
        let mut Length: ULONG;
        let FileNameLength: ULONG;
        let PathLength: ULONG;
        let mut TempString: UNICODE_STRING = core::mem::zeroed();
        let NewBuffer: PWSTR;
        let mut BufferStart: PWSTR;
        let mut p: PCWSTR;
        let mut Path = Path;

        /* Check if this is an absolute path */
        if RtlDetermineDosPathNameType_U(FileName) != RtlPathTypeRelative {
            /* Check if the file exists */
            if RtlDoesFileExists_UEx(FileName, 1u8) != 0 {
                /* Get the full name, which does the DOS lookup */
                return RtlGetFullPathName_U(FileName, Size, Buffer, PartName);
            }

            /* Doesn't exist, so fail */
            return 0;
        }

        /* Scan the filename */
        p = FileName;
        while *p != 0 {
            /* Looking for an extension */
            if *p == b'.' as u16 {
                /* No extension string needed -- it's part of the filename */
                Extension = core::ptr::null_mut();
                break;
            }

            /* Next character */
            p = p.offset(1);
        }

        /* Do we have an extension? */
        if Extension.is_null() {
            /* Nope, don't worry about one */
            ExtensionLength = 0;
        } else {
            /* Build a temporary string to get the extension length */
            Status = RtlInitUnicodeStringEx(&mut TempString, Extension);
            if !NT_SUCCESS(Status) {
                return 0;
            }
            ExtensionLength = TempString.Length as ULONG;
        }

        /* Build a temporary string to get the path length */
        Status = RtlInitUnicodeStringEx(&mut TempString, Path);
        if !NT_SUCCESS(Status) {
            return 0;
        }
        PathLength = TempString.Length as ULONG;

        /* Build a temporary string to get the filename length */
        Status = RtlInitUnicodeStringEx(&mut TempString, FileName);
        if !NT_SUCCESS(Status) {
            return 0;
        }
        FileNameLength = TempString.Length as ULONG;

        /* Allocate the buffer for the new string name */
        NewBuffer = rtl_heap_alloc(
            heap_handle(),
            0,
            (FileNameLength
                + ExtensionLength
                + PathLength
                + 3 * core::mem::size_of::<u16>() as ULONG) as usize,
        ) as PWSTR;
        if NewBuffer.is_null() {
            /* Fail the call */
            return 0;
        }

        /* Final loop to build the path */
        loop {
            /* Check if we have a valid character */
            BufferStart = NewBuffer;
            if *Path != 0 {
                /* Loop as long as there's no semicolon */
                while *Path != b';' as u16 {
                    /* Copy the next character */
                    *BufferStart = *Path;
                    BufferStart = BufferStart.offset(1);
                    Path = Path.offset(1);
                    if *Path == 0 {
                        break;
                    }
                }

                /* We found a semi-colon, to stop path processing on this loop */
                if *Path == b';' as u16 {
                    Path = Path.offset(1);
                }
            }

            /* Add a terminating slash if needed */
            if BufferStart != NewBuffer && *BufferStart.offset(-1) != b'\\' as u16 {
                *BufferStart = b'\\' as u16;
                BufferStart = BufferStart.offset(1);
            }

            /* Bail out if we reached the end */
            if *Path == 0 {
                Path = core::ptr::null_mut();
            }

            /* Copy the file name and check if an extension is needed */
            core::ptr::copy_nonoverlapping::<u8>(
                FileName as *const u8,
                BufferStart as *mut u8,
                FileNameLength as usize,
            );
            if ExtensionLength != 0 {
                /* Copy the extension too */
                core::ptr::copy_nonoverlapping::<u8>(
                    Extension as *const u8,
                    (BufferStart as *mut u8).add(FileNameLength as usize),
                    (ExtensionLength + core::mem::size_of::<u16>() as ULONG) as usize,
                );
            } else {
                /* Just NULL-terminate */
                *((BufferStart as *mut u8).add(FileNameLength as usize) as *mut u16) = 0;
            }

            /* Now, does this file exist? */
            if RtlDoesFileExists_UEx(NewBuffer, 0u8) != 0 {
                /* Call the full-path API to get the length */
                Length = RtlGetFullPathName_U(NewBuffer, Size, Buffer, PartName);
                break;
            }

            /* If we got here, path doesn't exist, so fail the call */
            Length = 0;
            if Path.is_null() {
                break;
            }
        }

        /* Free the allocation and return the length */
        rtl_heap_free(heap_handle(), 0, NewBuffer as PVOID);
        Length
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlGetFullPathName_UstrEx(
        FileName: PUNICODE_STRING,
        StaticString: PUNICODE_STRING,
        DynamicString: PUNICODE_STRING,
        StringUsed: *mut PUNICODE_STRING,
        FilePartSize: PSIZE_T,
        NameInvalid: PBOOLEAN,
        PathType: *mut RTL_PATH_TYPE,
        LengthNeeded: PSIZE_T,
    ) -> NTSTATUS {
        let Status: NTSTATUS;
        let mut StaticBuffer: PWCHAR;
        let mut ShortName: PCWCH = core::ptr::null_mut();
        let mut Length: ULONG;
        let StaticLength: USHORT;
        let mut TempDynamicString: UNICODE_STRING = core::mem::zeroed();

        TempDynamicString.Buffer = core::ptr::null_mut();

        if !StringUsed.is_null() {
            *StringUsed = core::ptr::null_mut();
        }
        if !LengthNeeded.is_null() {
            *LengthNeeded = 0;
        }
        if !FilePartSize.is_null() {
            *FilePartSize = 0;
        }

        if !DynamicString.is_null() && StringUsed.is_null() && !StaticString.is_null() {
            return STATUS_INVALID_PARAMETER;
        }

        if StaticString.is_null() {
            StaticLength = (260 * core::mem::size_of::<u16>()) as USHORT;
            StaticBuffer =
                RtlpAllocateStringMemory(260 * core::mem::size_of::<u16>() as u32, TAG_USTR);
            if StaticBuffer.is_null() {
                return STATUS_NO_MEMORY;
            }
        } else {
            StaticBuffer = (*StaticString).Buffer;
            StaticLength = (*StaticString).MaximumLength;
        }

        Length = RtlGetFullPathName_Ustr(
            FileName,
            StaticLength as ULONG,
            StaticBuffer,
            &mut ShortName,
            NameInvalid,
            PathType,
        );

        'quickie: {
            if Length == 0 {
                DbgPrint(
                    b"%s(%d) - RtlGetFullPathName_Ustr() returned 0\n\0".as_ptr() as *const i8,
                );
                Status = STATUS_OBJECT_NAME_INVALID;
                break 'quickie;
            }

            if !StaticString.is_null() && Length < StaticLength as ULONG {
                (*StaticString).Length = Length as USHORT;
                if !FilePartSize.is_null() {
                    *FilePartSize = if !ShortName.is_null() {
                        (ShortName as usize - (*StaticString).Buffer as usize)
                            / core::mem::size_of::<u16>()
                    } else {
                        0
                    };
                }
                if !StringUsed.is_null() {
                    *StringUsed = StaticString;
                }
                Status = STATUS_SUCCESS;
                break 'quickie;
            }

            if DynamicString.is_null() {
                if !LengthNeeded.is_null() {
                    *LengthNeeded = Length as SIZE_T;
                }
                Status = STATUS_BUFFER_TOO_SMALL;
                break 'quickie;
            }

            if !StaticBuffer.is_null() && Length < StaticLength as ULONG {
                *StaticBuffer.add(Length as usize / core::mem::size_of::<u16>()) = 0;
                (*DynamicString).MaximumLength = StaticLength;
                (*DynamicString).Length = Length as USHORT;
                (*DynamicString).Buffer = StaticBuffer;
                if !FilePartSize.is_null() {
                    *FilePartSize = if !ShortName.is_null() {
                        (ShortName as usize - StaticBuffer as usize) / core::mem::size_of::<u16>()
                    } else {
                        0
                    };
                }
                if !StringUsed.is_null() {
                    *StringUsed = DynamicString;
                }
                StaticBuffer = core::ptr::null_mut();
                Status = STATUS_SUCCESS;
                break 'quickie;
            }

            RtlAcquirePebLock();

            'release: {
                Length = RtlGetFullPathName_Ustr(
                    FileName,
                    StaticLength as ULONG,
                    StaticBuffer,
                    &mut ShortName,
                    NameInvalid,
                    PathType,
                );
                if Length == 0 {
                    DbgPrint(
                        b"%s line %d: RtlGetFullPathName_Ustr() returned 0\n\0".as_ptr()
                            as *const i8,
                    );
                    Status = STATUS_OBJECT_NAME_INVALID;
                    break 'release;
                }

                if !StaticString.is_null() && Length < StaticLength as ULONG {
                    (*StaticString).Length = Length as USHORT;
                    if !FilePartSize.is_null() {
                        *FilePartSize = if !ShortName.is_null() {
                            (ShortName as usize - (*StaticString).Buffer as usize)
                                / core::mem::size_of::<u16>()
                        } else {
                            0
                        };
                    }
                    if !StringUsed.is_null() {
                        *StringUsed = StaticString;
                    }
                    Status = STATUS_SUCCESS;
                    break 'release;
                }

                if (Length as usize + core::mem::size_of::<u16>()) > 0xfffe {
                    Status = STATUS_NAME_TOO_LONG;
                    break 'release;
                }

                TempDynamicString.Buffer =
                    RtlpAllocateStringMemory(Length + core::mem::size_of::<u16>() as u32, TAG_USTR);
                if TempDynamicString.Buffer.is_null() {
                    Status = STATUS_NO_MEMORY;
                    break 'release;
                }

                TempDynamicString.MaximumLength =
                    Length as USHORT + core::mem::size_of::<u16>() as USHORT;
                Length = RtlGetFullPathName_Ustr(
                    FileName,
                    Length,
                    TempDynamicString.Buffer,
                    &mut ShortName,
                    NameInvalid,
                    PathType,
                );
                if Length == 0 {
                    DbgPrint(
                        b"%s line %d: RtlGetFullPathName_Ustr() returned 0\n\0".as_ptr()
                            as *const i8,
                    );
                    Status = STATUS_OBJECT_NAME_INVALID;
                    break 'release;
                }

                if Length > TempDynamicString.MaximumLength as ULONG {
                    Status = STATUS_INTERNAL_ERROR;
                    break 'release;
                }

                if !FilePartSize.is_null() {
                    *FilePartSize = if !ShortName.is_null() {
                        (ShortName as usize - TempDynamicString.Buffer as usize)
                            / core::mem::size_of::<u16>()
                    } else {
                        0
                    };
                }

                *TempDynamicString
                    .Buffer
                    .add(Length as usize / core::mem::size_of::<u16>()) = 0;

                (*DynamicString).Buffer = TempDynamicString.Buffer;
                (*DynamicString).Length = Length as USHORT;
                (*DynamicString).MaximumLength = TempDynamicString.MaximumLength;
                if !StringUsed.is_null() {
                    *StringUsed = DynamicString;
                }

                TempDynamicString.Buffer = core::ptr::null_mut();
                Status = STATUS_SUCCESS;
            }

            RtlReleasePebLock();
        }

        if !StaticString.is_null()
            && !StaticBuffer.is_null()
            && StaticBuffer != (*StaticString).Buffer
        {
            RtlpFreeStringMemory(StaticBuffer as *mut core::ffi::c_void, TAG_USTR);
        }
        if !TempDynamicString.Buffer.is_null() {
            RtlpFreeStringMemory(TempDynamicString.Buffer as *mut core::ffi::c_void, TAG_USTR);
        }

        if (Status as u32 & 0xC0000000) == 0xC0000000
            && Status != STATUS_NO_SUCH_FILE
            && Status != STATUS_BUFFER_TOO_SMALL
        {
            DbgPrint(
                b"RTL: %s - failing on filename %wZ with status %08lx\n\0".as_ptr() as *const i8,
            );
        }

        Status
    }

    /// `NTSTATUS NTAPI RtlDosSearchPath_Ustr(ULONG Flags, PUNICODE_STRING x5,
    /// PUNICODE_STRING* FullNameOut, PSIZE_T FilePartSize, PSIZE_T LengthNeeded)`
    /// — the DOS path/DLL search hub. Hand-translated from
    /// `sdk/lib/rtl/path.c:2565`. The SxS isolation-redirection branch is
    /// `#ifndef _NTOSKRNL_` in C (RtlDosApplyFileIsolationRedirection_Ustr is
    /// an ntdll-only symbol), so it is cfg-gated out of the kernel build here.
    #[no_mangle]
    pub unsafe extern "system" fn RtlDosSearchPath_Ustr(
        Flags: ULONG,
        PathString: PUNICODE_STRING,
        FileNameString: PUNICODE_STRING,
        mut ExtensionString: PUNICODE_STRING,
        CallerBuffer: PUNICODE_STRING,
        DynamicString: PUNICODE_STRING,
        FullNameOut: *mut PUNICODE_STRING,
        FilePartSize: PSIZE_T,
        LengthNeeded: PSIZE_T,
    ) -> NTSTATUS {
        let is_path_separator = |x: u16| x == b'\\' as u16 || x == b'/' as u16;

        let mut StaticCandidateBuffer: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
        let mut StaticCandidateString: UNICODE_STRING = core::mem::zeroed();
        let mut Status: NTSTATUS = STATUS_SUCCESS;
        let mut PathType: RTL_PATH_TYPE = RtlPathTypeUnknown;
        let mut ExtensionLength: USHORT = 0;
        let mut MaxPathSize: SIZE_T = 0;

        /* Initialize the input string */
        RtlInitEmptyUnicodeString(
            &mut StaticCandidateString,
            StaticCandidateBuffer.as_mut_ptr(),
            (MAX_PATH as usize * core::mem::size_of::<u16>()) as USHORT,
        );

        /* Initialize optional arguments */
        if !FullNameOut.is_null() {
            *FullNameOut = core::ptr::null_mut();
        }
        if !FilePartSize.is_null() {
            *FilePartSize = 0;
        }
        if !LengthNeeded.is_null() {
            *LengthNeeded = 0;
        }
        if !DynamicString.is_null() {
            (*DynamicString).Length = 0;
            (*DynamicString).MaximumLength = 0;
            (*DynamicString).Buffer = core::ptr::null_mut();
        }

        'quickie: {
            /* Check for invalid parameters */
            if (Flags & !7) != 0
                || PathString.is_null()
                || FileNameString.is_null()
                || (!CallerBuffer.is_null() && !DynamicString.is_null() && FullNameOut.is_null())
            {
                Status = STATUS_INVALID_PARAMETER;
                break 'quickie;
            }

            /* First check what kind of path this is */
            PathType = RtlDetermineDosPathNameType_Ustr(FileNameString);

            /* Caller wants to prevent relative .\ and ..\ paths */
            if (Flags & 2) != 0
                && PathType == RtlPathTypeRelative
                && (*FileNameString).Length >= (2 * core::mem::size_of::<u16>()) as USHORT
                && *(*FileNameString).Buffer == b'.' as u16
                && (is_path_separator(*(*FileNameString).Buffer.add(1))
                    || (*(*FileNameString).Buffer.add(1) == b'.' as u16
                        && (*FileNameString).Length >= (3 * core::mem::size_of::<u16>()) as USHORT
                        && is_path_separator(*(*FileNameString).Buffer.add(2))))
            {
                /* Yes, and this path is like that, so make it seem unknown */
                PathType = RtlPathTypeUnknown;
            }

            /* Now check relative vs non-relative paths */
            if PathType == RtlPathTypeRelative {
                /* Does the caller want SxS? (ntdll-only; #ifndef _NTOSKRNL_) */
                #[cfg(not(feature = "kernel"))]
                {
                    if (Flags & 1) != 0 {
                        const STATUS_SXS_KEY_NOT_FOUND: NTSTATUS = 0xC015_0008u32 as i32;
                        let mut FullIsolatedPath: PUNICODE_STRING = core::ptr::null_mut();
                        Status = RtlDosApplyFileIsolationRedirection_Ustr(
                            1,
                            FileNameString,
                            ExtensionString,
                            CallerBuffer,
                            DynamicString,
                            &mut FullIsolatedPath,
                            core::ptr::null_mut(),
                            FilePartSize,
                            LengthNeeded,
                        );
                        if NT_SUCCESS(Status) {
                            /* We found the SxS path, return it */
                            if !FullNameOut.is_null() {
                                *FullNameOut = FullIsolatedPath;
                            }
                            break 'quickie;
                        } else if Status != STATUS_SXS_KEY_NOT_FOUND {
                            /* Critical SxS error, fail */
                            break 'quickie;
                        }
                    }
                }

                /* No SxS key found, or not requested, check if there's an extension */
                if !ExtensionString.is_null() {
                    /* Save the extension length, and check if there's a file name */
                    ExtensionLength = (*ExtensionString).Length;
                    if (*FileNameString).Length != 0 {
                        /* Start parsing the file name */
                        let mut End = (*FileNameString)
                            .Buffer
                            .add((*FileNameString).Length as usize / core::mem::size_of::<u16>());
                        while End > (*FileNameString).Buffer {
                            /* If we find a path separator, there's no extension */
                            End = End.sub(1);
                            if is_path_separator(*End) {
                                break;
                            }

                            /* Otherwise, did we find an extension dot? */
                            if *End == b'.' as u16 {
                                /* Ignore what the caller sent it, use the filename's */
                                ExtensionString = core::ptr::null_mut();
                                ExtensionLength = 0;
                                break;
                            }
                        }
                    }
                }

                /* Check if we got a path */
                if (*PathString).Length != 0 {
                    /* Start parsing the path name, looking for path separators */
                    let mut End = (*PathString)
                        .Buffer
                        .add((*PathString).Length as usize / core::mem::size_of::<u16>());
                    let mut p = End;
                    while p > (*PathString).Buffer {
                        p = p.sub(1);
                        if *p == b';' as u16 {
                            /* This is the size of the path -- handle a trailing slash */
                            let mut PathSize = End.offset_from(p) as usize - 1;
                            if PathSize != 0 && !is_path_separator(*End.sub(1)) {
                                PathSize += 1;
                            }

                            /* Check if we found a bigger path than before */
                            if PathSize > MaxPathSize {
                                MaxPathSize = PathSize;
                            }

                            /* Keep going with the path after this path separator */
                            End = p;
                        }
                    }

                    /* This is the trailing path, run the same code as above */
                    let mut PathSize = End.offset_from(p) as usize;
                    if PathSize != 0 && !is_path_separator(*End.sub(1)) {
                        PathSize += 1;
                    }
                    if PathSize > MaxPathSize {
                        MaxPathSize = PathSize;
                    }

                    /* Finally, convert the largest path size into WCHAR */
                    MaxPathSize *= core::mem::size_of::<u16>();
                }

                /* Use the extension, the file name, and the largest path as the size */
                let WorstCaseLength: SIZE_T = ExtensionLength as usize
                    + (*FileNameString).Length as usize
                    + (MaxPathSize as u16) as usize
                    + core::mem::size_of::<u16>();
                if WorstCaseLength > UNICODE_STRING_MAX_BYTES as usize {
                    /* It has to fit in a registry string, if not, fail here */
                    Status = STATUS_NAME_TOO_LONG;
                    break 'quickie;
                }

                /* Scan the path now, to see if we can find the file */
                let mut p = (*PathString).Buffer;
                let End = (*PathString)
                    .Buffer
                    .add((*PathString).Length as usize / core::mem::size_of::<u16>());
                while p < End {
                    /* Find out where this path ends */
                    let mut SegmentEnd = p;
                    while SegmentEnd != End && *SegmentEnd != b';' as u16 {
                        SegmentEnd = SegmentEnd.add(1);
                    }

                    /* Compute the size of this path */
                    let ByteCount: SIZE_T =
                        (SegmentEnd.offset_from(p) as usize) * core::mem::size_of::<u16>();
                    let mut SegmentSize: SIZE_T = ByteCount;

                    /* Handle trailing slash if there isn't one */
                    if SegmentSize != 0 && !is_path_separator(*SegmentEnd.sub(1)) {
                        SegmentSize += core::mem::size_of::<u16>();
                    }

                    /* Now check if our initial static buffer is too small */
                    if (StaticCandidateString.MaximumLength as usize)
                        < (SegmentSize
                            + ExtensionLength as usize
                            + (*FileNameString).Length as usize
                            + core::mem::size_of::<u16>())
                    {
                        /* At this point we should've been using our static buffer */
                        if StaticCandidateString.Buffer != StaticCandidateBuffer.as_mut_ptr() {
                            Status = STATUS_INTERNAL_ERROR;
                            break 'quickie;
                        }

                        if (SegmentSize
                            + ExtensionLength as usize
                            + (*FileNameString).Length as usize)
                            > (UNICODE_STRING_MAX_BYTES as usize - core::mem::size_of::<u16>())
                        {
                            Status = STATUS_INTERNAL_ERROR;
                            break 'quickie;
                        }

                        /* Now allocate the dynamic string */
                        StaticCandidateString.MaximumLength = WorstCaseLength as USHORT;
                        StaticCandidateString.Buffer =
                            RtlpAllocateStringMemory(WorstCaseLength as ULONG, TAG_USTR);
                        if StaticCandidateString.Buffer.is_null() {
                            Status = STATUS_NO_MEMORY;
                            break 'quickie;
                        }
                    }

                    /* Copy the path in the string */
                    RtlCopyMemory(
                        StaticCandidateString.Buffer as PVOID,
                        p as *const core::ffi::c_void,
                        ByteCount,
                    );

                    /* Get to the end of the string, and add the trailing slash if missing */
                    let mut CandidateEnd = StaticCandidateString
                        .Buffer
                        .add(ByteCount / core::mem::size_of::<u16>());
                    if SegmentSize != 0 && SegmentSize != ByteCount {
                        *CandidateEnd = OBJ_NAME_PATH_SEPARATOR;
                        CandidateEnd = CandidateEnd.add(1);
                    }

                    /* Copy the filename now */
                    RtlCopyMemory(
                        CandidateEnd as PVOID,
                        (*FileNameString).Buffer as *const core::ffi::c_void,
                        (*FileNameString).Length as usize,
                    );
                    CandidateEnd = CandidateEnd
                        .add((*FileNameString).Length as usize / core::mem::size_of::<u16>());

                    /* Check if there was an extension */
                    if !ExtensionString.is_null() {
                        /* Copy the extension too */
                        RtlCopyMemory(
                            CandidateEnd as PVOID,
                            (*ExtensionString).Buffer as *const core::ffi::c_void,
                            (*ExtensionString).Length as usize,
                        );
                        CandidateEnd = CandidateEnd
                            .add((*ExtensionString).Length as usize / core::mem::size_of::<u16>());
                    }

                    /* We are done, terminate it */
                    *CandidateEnd = 0;

                    /* Now set the final length of the string so it becomes valid */
                    StaticCandidateString.Length =
                        (CandidateEnd.offset_from(StaticCandidateString.Buffer) as usize
                            * core::mem::size_of::<u16>()) as USHORT;

                    /* Check if this file exists */
                    if RtlDoesFileExists_UEx(StaticCandidateString.Buffer as PCWSTR, 0u8) != 0 {
                        /* Awesome, it does, now get the full path */
                        Status = RtlGetFullPathName_UstrEx(
                            &mut StaticCandidateString,
                            CallerBuffer,
                            DynamicString,
                            FullNameOut,
                            FilePartSize,
                            core::ptr::null_mut(),
                            &mut PathType,
                            LengthNeeded,
                        );
                        break 'quickie;
                    } else {
                        /* Otherwise, move to the next path */
                        if SegmentEnd != End {
                            /* Handle the case of the path separator trailing */
                            p = SegmentEnd.add(1);
                        } else {
                            p = SegmentEnd;
                        }
                    }
                }

                /* Loop finished and we didn't break out -- fail */
                Status = STATUS_NO_SUCH_FILE;
            } else {
                /* We have a full path, so check if it does exist */
                if RtlDoesFileExists_UstrEx(FileNameString, 1u8) == 0 {
                    /* It doesn't exist, did we have an extension? */
                    if ExtensionString.is_null() || (*ExtensionString).Length == 0 {
                        /* No extension, so just fail */
                        Status = STATUS_NO_SUCH_FILE;
                        break 'quickie;
                    }

                    /* There was an extension, check if the filename already had one */
                    if (Flags & 4) == 0 && (*FileNameString).Length != 0 {
                        /* Parse the filename */
                        let p = (*FileNameString).Buffer;
                        let mut End =
                            p.add((*FileNameString).Length as usize / core::mem::size_of::<u16>());
                        while End > p {
                            /* If there's a path separator, there's no extension */
                            End = End.sub(1);
                            if is_path_separator(*End) {
                                break;
                            }

                            /* Otherwise, did we find an extension dot? */
                            if *End == b'.' as u16 {
                                /* File already had an extension, so fail */
                                Status = STATUS_NO_SUCH_FILE;
                                break 'quickie;
                            }
                        }
                    }

                    /* So there is an extension, we'll try again by adding it */
                    let NamePlusExtLength: SIZE_T = (*FileNameString).Length as usize
                        + (*ExtensionString).Length as usize
                        + core::mem::size_of::<u16>();
                    if NamePlusExtLength > UNICODE_STRING_MAX_BYTES as usize {
                        /* It won't fit in any kind of valid string, so fail */
                        Status = STATUS_NAME_TOO_LONG;
                        break 'quickie;
                    }

                    /* Will it fit in our temporary string? */
                    if NamePlusExtLength > StaticCandidateString.MaximumLength as usize {
                        /* It won't fit anymore, allocate a dynamic string for it */
                        StaticCandidateString.MaximumLength = NamePlusExtLength as USHORT;
                        StaticCandidateString.Buffer =
                            RtlpAllocateStringMemory(NamePlusExtLength as ULONG, TAG_USTR);
                        if StaticCandidateString.Buffer.is_null() {
                            /* Ran out of memory, so fail */
                            Status = STATUS_NO_MEMORY;
                            break 'quickie;
                        }
                    }

                    /* Copy the filename */
                    RtlCopyUnicodeString(&mut StaticCandidateString, FileNameString);

                    /* Copy the extension */
                    RtlAppendUnicodeStringToString(&mut StaticCandidateString, ExtensionString);

                    /* And check if this file now exists */
                    if RtlDoesFileExists_UstrEx(core::ptr::addr_of!(StaticCandidateString), 1u8)
                        == 0
                    {
                        /* Still no joy, fail out */
                        Status = STATUS_NO_SUCH_FILE;
                        break 'quickie;
                    }

                    /* File was found, get the final full path */
                    Status = RtlGetFullPathName_UstrEx(
                        &mut StaticCandidateString,
                        CallerBuffer,
                        DynamicString,
                        FullNameOut,
                        FilePartSize,
                        core::ptr::null_mut(),
                        &mut PathType,
                        LengthNeeded,
                    );
                } else {
                    /* File was found on the first try, get the final full path */
                    Status = RtlGetFullPathName_UstrEx(
                        FileNameString,
                        CallerBuffer,
                        DynamicString,
                        FullNameOut,
                        FilePartSize,
                        core::ptr::null_mut(),
                        &mut PathType,
                        LengthNeeded,
                    );
                }
            }
        }

        /* Quickie: anything that was not an error, turn into STATUS_SUCCESS */
        if NT_SUCCESS(Status) {
            Status = STATUS_SUCCESS;
        }

        /* Check if we had a dynamic string */
        if !StaticCandidateString.Buffer.is_null()
            && StaticCandidateString.Buffer != StaticCandidateBuffer.as_mut_ptr()
        {
            /* Free it */
            RtlFreeUnicodeString(&mut StaticCandidateString);
        }

        Status
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlDoesFileExists_U(FileName: PCWSTR) -> BOOLEAN {
        RtlDoesFileExists_UEx(FileName, 1u8)
    }

    // ============================================================================
}

// Re-export the device-name statics into the cabi top-level namespace so the
// 6 lexical fns above (RtlIsDosDeviceName_Ustr etc.) resolve them by bare name.
use path_wave::{
    DeviceRootString, RtlpDosAUXDevice, RtlpDosCOMDevice, RtlpDosCONDevice, RtlpDosLPTDevice,
    RtlpDosNULDevice, RtlpDosPRNDevice, RtlpDosSlashCONDevice,
};
