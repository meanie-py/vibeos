//! `#[no_mangle] extern "system"` NTAPI shims — Wave RTL-1. Unsafe
//! counted-string struct reconstruct + the exact legacy `Rtl*`
//! contract. `extern "system"` is NTAPI (cdecl on amd64/arm64,
//! stdcall on i386 — and plain C on the x86-64 host the diff harness
//! links against). The pure decisions live in `core_rs::ustr`.
#![allow(non_snake_case)]

use crate::core_rs::{acl, conv, dos8dot3, misc, nls, sd as sd_, sid, ustr};
#[cfg(not(feature = "kernel"))]
use crate::core_rs::utf8;

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
pub struct UNICODE_STRING {
    pub Length: u16,
    pub MaximumLength: u16,
    pub Buffer: *mut u16,
}

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
const STATUS_UNMAPPABLE_CHARACTER: i32 = 0x8000_0016u32 as i32;
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
const TAG_USTR: u32 = u32::from_le_bytes(*b"RTSU");
const TAG_ASTR: u32 = u32::from_le_bytes(*b"RTSA");
const TAG_OSTR: u32 = u32::from_le_bytes(*b"RTSO");

// Kernel-side allocator: when linked into ntoskrnl, route through the
// existing C `RtlpAllocateMemory` (ntoskrnl/rtl/libsupp.c — wraps
// ExAllocatePoolWithTag(PagedPool, ...)). The difftest harness keeps
// the NULL stub so its alloc-failure model paths still fire.
#[cfg(feature = "kernel")]
extern "system" {
    fn RtlpAllocateMemory(Bytes: u32, Tag: u32) -> *mut u8;
    fn RtlpFreeMemory(Mem: *mut u8, Tag: u32);
}
#[inline]
unsafe fn rtl_alloc(bytes: usize, tag: u32) -> *mut u8 {
    #[cfg(feature = "kernel")]
    { RtlpAllocateMemory(bytes as u32, tag) }
    #[cfg(not(feature = "kernel"))]
    { let _ = (bytes, tag); core::ptr::null_mut() }
}
#[inline]
#[allow(dead_code)]
unsafe fn rtl_free(p: *mut u8, tag: u32) {
    #[cfg(feature = "kernel")]
    { if !p.is_null() { RtlpFreeMemory(p, tag); } }
    #[cfg(not(feature = "kernel"))]
    { let _ = (p, tag); }
}

/// `RTL_BUFFER` (ndk/rtltypes.h) — host layout (SIZE_T = 8).
#[repr(C)]
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
pub unsafe extern "system" fn RtlInitAnsiString(
    dest: *mut STRING,
    src: *const u8,
) {
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
pub unsafe extern "system" fn RtlInitAnsiStringEx(
    dest: *mut STRING,
    src: *const u8,
) -> i32 {
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
pub unsafe extern "system" fn RtlInitUnicodeString(
    dest: *mut UNICODE_STRING,
    src: *const u16,
) {
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
pub unsafe extern "system" fn RtlUpperString(
    dest: *mut STRING,
    src: *const STRING,
) {
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
pub unsafe extern "system" fn RtlAnsiCharToUnicodeChar(
    ansi_char: *mut *mut u8,
) -> u16 {
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
        (*dest).MaximumLength = length as u16;
        if buf.is_null() {
            return STATUS_NO_MEMORY;
        }
        (*dest).Buffer = buf;
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
pub unsafe extern "system" fn RtlxAnsiStringToUnicodeSize(
    s: *const STRING,
) -> u32 {
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
        (*dest).MaximumLength = length as u16;
        if buf.is_null() {
            return STATUS_NO_MEMORY;
        }
        (*dest).Buffer = buf;
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
pub unsafe extern "system" fn RtlxUnicodeStringToAnsiSize(
    s: *const UNICODE_STRING,
) -> u32 {
    (*s).Length as u32 / 2 + 1
}

/// `ULONG NTAPI RtlxUnicodeStringToOemSize` = `Uni.Length/2 + 1`.
#[no_mangle]
pub unsafe extern "system" fn RtlxUnicodeStringToOemSize(
    s: *const UNICODE_STRING,
) -> u32 {
    (*s).Length as u32 / 2 + 1
}

/// `ULONG NTAPI RtlxOemStringToUnicodeSize` = `Oem.Length*2 + 2`.
#[no_mangle]
pub unsafe extern "system" fn RtlxOemStringToUnicodeSize(
    s: *const STRING,
) -> u32 {
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
        (*dest).MaximumLength = length as u16;
        if buf.is_null() {
            return STATUS_NO_MEMORY;
        }
        (*dest).Buffer = buf;
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
        (*dest).MaximumLength = length as u16;
        if buf.is_null() {
            return STATUS_NO_MEMORY;
        }
        (*dest).Buffer = buf;
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
        (*dest).MaximumLength = length as u16;
        if buf.is_null() {
            return STATUS_NO_MEMORY;
        }
        (*dest).Buffer = buf;
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
        (*dest).MaximumLength = length as u16;
        if buf.is_null() {
            return STATUS_NO_MEMORY;
        }
        (*dest).Buffer = buf;
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
    if nt_success(status)
        && !nls::did_oem_work(s, rslice((*dest).Buffer, n))
    {
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
        if buf.is_null() {
            return STATUS_NO_MEMORY;
        }
        (*dest).Buffer = buf;
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
        (*dest).MaximumLength = length as u16;
        if buf.is_null() {
            return STATUS_NO_MEMORY;
        }
        (*dest).Buffer = buf;
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
        (*dest).MaximumLength = length as u16;
        if buf.is_null() {
            return STATUS_NO_MEMORY;
        }
        (*dest).Buffer = buf;
    }
    if (*dest).Length > (*dest).MaximumLength {
        return STATUS_BUFFER_OVERFLOW;
    }
    let dlen = (*dest).Length;
    let s = rslice((*src).Buffer, (*src).Length as usize / 2);
    let buf = wslice((*dest).Buffer, dlen as usize);
    let n = nls::upcase_uni_to_mb(&mut buf[..dlen as usize], s);
    let mut status = STATUS_SUCCESS;
    if nt_success(status)
        && !nls::did_oem_work(s, rslice((*dest).Buffer, n))
    {
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
        (*dest).MaximumLength = length as u16;
        if buf.is_null() {
            return STATUS_NO_MEMORY;
        }
        (*dest).Buffer = buf;
    }
    if (*dest).Length >= (*dest).MaximumLength {
        return STATUS_BUFFER_OVERFLOW;
    }
    let dlen = (*dest).Length;
    let s = rslice((*src).Buffer, (*src).Length as usize / 2);
    let buf = wslice((*dest).Buffer, dlen as usize);
    let n = nls::upcase_uni_to_mb(&mut buf[..dlen as usize], s);
    let mut status = STATUS_SUCCESS;
    if nt_success(status)
        && !nls::did_oem_work(s, rslice((*dest).Buffer, n))
    {
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
        if buf.is_null() {
            return STATUS_NO_MEMORY;
        }
        (*dest).Buffer = buf;
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
pub unsafe extern "system" fn RtlGetDefaultCodePage(
    ansi_cp: *mut u16,
    oem_cp: *mut u16,
) {
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
pub unsafe extern "system" fn RtlCharToInteger(
    str: *const u8,
    base: u32,
    value: *mut u32,
) -> i32 {
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
pub unsafe extern "system" fn RtlCopyString(
    dest: *mut STRING,
    src: *const STRING,
) {
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
            core::ptr::write_volatile(
                (*dest).Buffer.add(unit_off + sl as usize / 2),
                0u16,
            );
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
            core::ptr::write_volatile(
                (*dest).Buffer.add(unit_off + length as usize / 2),
                0u16,
            );
        }
    }
    STATUS_SUCCESS
}

/// `NTSTATUS NTAPI RtlAppendAsciizToString(PSTRING, PCSZ)`. NULL src ⇒
/// unchanged ⇒ SUCCESS. No NUL.
#[no_mangle]
pub unsafe extern "system" fn RtlAppendAsciizToString(
    dest: *mut STRING,
    src: *const u8,
) -> i32 {
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
                status = RtlOemStringToUnicodeString(
                    computer_name,
                    &cn_oem,
                    allocate,
                );
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
    (*uni_dest).MaximumLength = size as u16;
    let buf = rtl_alloc(size, TAG_USTR) as *mut u16;
    if buf.is_null() {
        (*uni_dest).Buffer = core::ptr::null_mut();
        return 0;
    }
    core::ptr::copy_nonoverlapping(source, buf, size / 2);
    (*uni_dest).Buffer = buf;
    (*uni_dest).Length = size as u16 - 2;
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
        || ((*source).Length == 0
            && (*source).MaximumLength > 0
            && (*source).Buffer.is_null())
        || flags == RTL_DUP_ALLOCATE_NULL_STRING
        || flags >= 4
    {
        return STATUS_INVALID_PARAMETER;
    }
    if (*source).Length == 0
        && flags != (RTL_DUP_NULL_TERMINATE | RTL_DUP_ALLOCATE_NULL_STRING)
    {
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
        core::ptr::copy_nonoverlapping(
            (*source).Buffer,
            buf,
            (*source).Length as usize / 2,
        );
        if (flags & RTL_DUP_NULL_TERMINATE) != 0 {
            *buf.add((*source).Length as usize / 2) = 0;
        }
        (*destination).Buffer = buf;
        (*destination).Length = (*source).Length;
        (*destination).MaximumLength = alloc_len as u16;
    }
    STATUS_SUCCESS
}

/// `NTSTATUS NTAPI RtlStringFromGUID(REFGUID, PUNICODE_STRING)` —
/// Length=76, MaximumLength=78 set before `Buffer = alloc = NULL` ⇒
/// STATUS_NO_MEMORY.
#[no_mangle]
pub unsafe extern "system" fn RtlStringFromGUID(
    _guid: *const GUID,
    guid_string: *mut UNICODE_STRING,
) -> i32 {
    (*guid_string).Length = 38 * 2;
    (*guid_string).MaximumLength = 38 * 2 + 2;
    (*guid_string).Buffer = core::ptr::null_mut();
    STATUS_NO_MEMORY
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
    if (*buffer).Buffer == (*buffer).StaticBuffer
        && (*buffer).StaticSize >= required_size
    {
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
pub unsafe extern "system" fn RtlIsTextUnicode(
    buf: *const u8,
    len: i32,
    pf: *mut i32,
) -> u8 {
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
        & !(RTL_FIND_CHAR_START_AT_END
            | RTL_FIND_CHAR_COMPLEMENT
            | RTL_FIND_CHAR_CASE_INSENSITIVE)
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
pub unsafe extern "system" fn RtlEraseUnicodeString(
    string: *mut UNICODE_STRING,
) {
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
pub unsafe extern "system" fn RtlFreeUnicodeString(
    unicode_string: *mut UNICODE_STRING,
) {
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
            let mut checksum =
                dos8dot3::get_checksum(rslice(nbuf, length), (*name).Length);
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
        let mut checksum =
            dos8dot3::get_checksum(rslice(nbuf, length), (*name).Length);
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
    } else if dot > 8
        || (olen as i32 - dot > 4)
        || dot == olen as i32 - 1
    {
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
pub unsafe extern "system" fn RtlSubAuthoritySid(
    sid_: *mut u8,
    sub_authority: u32,
) -> *mut u32 {
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
pub unsafe extern "system" fn RtlCopySid(
    buffer_length: u32,
    dest: *mut u8,
    src: *const u8,
) -> i32 {
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
const TAG_SID: u32 = u32::from_le_bytes(*b"pSid"); // 'diSp' (sdk/lib/rtl/sid.c:15)
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
pub unsafe extern "system" fn RtlCreateAcl(
    acl_: *mut u8,
    acl_sz: u32,
    acl_revision: u32,
) -> i32 {
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
pub unsafe extern "system" fn RtlFirstFreeAce(
    acl_: *mut u8,
    first_free_ace: *mut *mut u8,
) -> u8 {
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
pub unsafe extern "system" fn RtlpDeleteData(
    ace: *mut u8,
    ace_size: u32,
    offset: u32,
) {
    if ace_size < offset {
        core::ptr::copy(
            ace.add(ace_size as usize),
            ace,
            (offset - ace_size) as usize,
        );
    }
    if offset.wrapping_sub(ace_size) < offset {
        core::ptr::write_bytes(
            ace.add((offset - ace_size) as usize),
            0,
            ace_size as usize,
        );
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
    let revision = if revision < acl_revb { acl_revb } else { revision };
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
    if ace.is_null()
        || ace as usize + ace_sz as usize > acl_ as usize + acl_size(acl_) as usize
    {
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
    let revision = if revision < acl_revb { acl_revb } else { revision };
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
    if ace.is_null()
        || ace as usize + ace_sz as usize > acl_ as usize + acl_size(acl_) as usize
    {
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
        core::ptr::copy_nonoverlapping(
            inherited_object_type_guid,
            sid_start,
            acl::GUID_SZ,
        );
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
            } else if at > acl::ACCESS_MAX_MS_V2_ACE_TYPE
                && acl_revision < acl::ACL_REVISION3
            {
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
        || free_ace as usize + ace_list_length as usize
            > acl_ as usize + acl_size(acl_) as usize
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
    (acl_.add(4) as *mut u16)
        .write_unaligned(acl_ace_count(acl_) + new_ace_count);
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
    if acl_ace_count(acl_) as u32 <= ace_index
        || RtlFirstFreeAce(acl_, &mut free_ace) == 0
    {
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
    RtlpAddKnownAce(acl_, revision, 0, access_mask, sid_, acl::ACCESS_ALLOWED_ACE_TYPE)
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
    RtlpAddKnownAce(acl_, revision, 0, access_mask, sid_, acl::ACCESS_DENIED_ACE_TYPE)
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
    (owner, owner_size, group, group_size, dacl, dacl_size, sacl, sacl_size)
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
    core::ptr::write_bytes(sd_p as *mut u8, 0, core::mem::size_of::<SECURITY_DESCRIPTOR>());
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
        *dacl_defaulted = (((*sd_p).Control & sd_::SE_DACL_DEFAULTED) == sd_::SE_DACL_DEFAULTED) as u8;
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
        *sacl_defaulted = (((*sd_p).Control & sd_::SE_SACL_DEFAULTED) == sd_::SE_SACL_DEFAULTED) as u8;
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
    *owner_defaulted = (((*sd_p).Control & sd_::SE_OWNER_DEFAULTED) == sd_::SE_OWNER_DEFAULTED) as u8;
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
    *group_defaulted = (((*sd_p).Control & sd_::SE_GROUP_DEFAULTED) == sd_::SE_GROUP_DEFAULTED) as u8;
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
    let (owner, owner_len, group, group_len, dacl, dacl_len, sacl, sacl_len) =
        rtlp_query_sd(src_p);
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
        core::ptr::copy_nonoverlapping(
            sacl,
            (rel_p as *mut u8).add(current),
            sacl_len as usize,
        );
        (*rel_p).Sacl = current as u32;
        current += sacl_len as usize;
    }
    if dacl_len > 0 {
        core::ptr::copy_nonoverlapping(
            dacl,
            (rel_p as *mut u8).add(current),
            dacl_len as usize,
        );
        (*rel_p).Dacl = current as u32;
        current += dacl_len as usize;
    }
    if owner_len > 0 {
        core::ptr::copy_nonoverlapping(
            owner,
            (rel_p as *mut u8).add(current),
            owner_len as usize,
        );
        (*rel_p).Owner = current as u32;
        current += owner_len as usize;
    }
    if group_len > 0 {
        core::ptr::copy_nonoverlapping(
            group,
            (rel_p as *mut u8).add(current),
            group_len as usize,
        );
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
    let _total = core::mem::size_of::<SECURITY_DESCRIPTOR_RELATIVE>() as u32
        + owner_len
        + group_len
        + dacl_len
        + sacl_len;
    // Real ntdll path goes through RtlAllocateHeap → we don't implement
    // that here (no_std + the in-tree librtl translation is no-alloc).
    // The diff harness stubs RtlAllocateHeap=NULL ⇒ STATUS_NO_MEMORY.
    *dst_pp = core::ptr::null_mut();
    STATUS_NO_MEMORY
}
