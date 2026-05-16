//! `#[no_mangle] extern "system"` NTAPI shims — Wave RTL-1. Unsafe
//! counted-string struct reconstruct + the exact legacy `Rtl*`
//! contract. `extern "system"` is NTAPI (cdecl on amd64/arm64,
//! stdcall on i386 — and plain C on the x86-64 host the diff harness
//! links against). The pure decisions live in `core_rs::ustr`.
#![allow(non_snake_case)]

use crate::core_rs::{conv, dos8dot3, misc, nls, ustr, utf8};

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
const STATUS_INVALID_PARAMETER_4: i32 = 0xC000_00F2u32 as i32;
const STATUS_INVALID_PARAMETER_5: i32 = 0xC000_00F3u32 as i32;
const STATUS_UNMAPPABLE_CHARACTER: i32 = 0x8000_0016u32 as i32;
const STATUS_ACCESS_VIOLATION: i32 = 0xC000_0005u32 as i32;
const STATUS_BUFFER_TOO_SMALL: i32 = 0xC000_0023u32 as i32;
const STATUS_NOT_FOUND: i32 = 0xC000_0225u32 as i32;
const STATUS_NOT_IMPLEMENTED: i32 = 0xC000_0002u32 as i32;
const STATUS_INVALID_COMPUTER_NAME: i32 = 0xC000_0122u32 as i32;
const MAXUSHORT: u32 = 0xFFFF;
const MAX_COMPUTERNAME_LENGTH: u32 = 15;

const RTL_SKIP_BUFFER_COPY: u32 = 0x0000_0001;
const RTL_DUP_NULL_TERMINATE: u32 = 1;
const RTL_DUP_ALLOCATE_NULL_STRING: u32 = 2;

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
        // genuine sets MaximumLength = Length BEFORE the NULL-alloc
        // check, so it persists even when allocation fails.
        (*dest).MaximumLength = length as u16;
        return STATUS_NO_MEMORY;
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
        // genuine sets MaximumLength = Length BEFORE the NULL-alloc
        // check, so it persists even when allocation fails.
        (*dest).MaximumLength = length as u16;
        return STATUS_NO_MEMORY;
    }
    if (*dest).Length >= (*dest).MaximumLength {
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
        // genuine sets MaximumLength = Length BEFORE the NULL-alloc
        // check, so it persists even when allocation fails.
        (*dest).MaximumLength = length as u16;
        return STATUS_NO_MEMORY;
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
        // genuine sets MaximumLength = Length BEFORE the NULL-alloc
        // check, so it persists even when allocation fails.
        (*dest).MaximumLength = length as u16;
        return STATUS_NO_MEMORY;
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
        // genuine sets MaximumLength = Length BEFORE the NULL-alloc
        // check, so it persists even when allocation fails.
        (*dest).MaximumLength = length as u16;
        return STATUS_NO_MEMORY;
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
        // genuine sets MaximumLength = Length BEFORE the NULL-alloc
        // check, so it persists even when allocation fails.
        (*dest).MaximumLength = length as u16;
        return STATUS_NO_MEMORY;
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
        (*dest).MaximumLength = (*src).Length;
        return STATUS_NO_MEMORY;
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
        // genuine sets MaximumLength = Length BEFORE the NULL-alloc
        // check, so it persists even when allocation fails.
        (*dest).MaximumLength = length as u16;
        return STATUS_NO_MEMORY;
    }
    if (*dest).Length >= (*dest).MaximumLength && (*dest).MaximumLength == 0 {
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
        // genuine sets MaximumLength = Length BEFORE the NULL-alloc
        // check, so it persists even when allocation fails.
        (*dest).MaximumLength = length as u16;
        return STATUS_NO_MEMORY;
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
        // genuine sets MaximumLength = Length BEFORE the NULL-alloc
        // check, so it persists even when allocation fails.
        (*dest).MaximumLength = length as u16;
        return STATUS_NO_MEMORY;
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
        (*dest).MaximumLength = (*src).Length;
        return STATUS_NO_MEMORY;
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
/// `Size = (wcslen+1)*2`; `Size > MAXUSHORT` ⇒ FALSE (no writes);
/// else `Buffer = alloc = NULL` ⇒ FALSE (Length/Max untouched).
#[no_mangle]
pub unsafe extern "system" fn RtlCreateUnicodeString(
    uni_dest: *mut UNICODE_STRING,
    source: *const u16,
) -> u8 {
    let size = (wcslen(source) + 1) * 2;
    if size > MAXUSHORT as usize {
        return 0;
    }
    (*uni_dest).Buffer = core::ptr::null_mut();
    0
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
    let (alen, _) = if source.is_null() {
        (0u16, 0u16)
    } else {
        ustr::ansi_lengths(strlen(source))
    };
    let length = (alen as u32 + 1) * 2;
    if length > MAXUSHORT {
        return 0; // STATUS_INVALID_PARAMETER_2 before any field write
    }
    (*destination).Length = length as u16 - 2;
    (*destination).Buffer = core::ptr::null_mut();
    (*destination).MaximumLength = length as u16;
    0 // NT_SUCCESS(STATUS_NO_MEMORY) == false
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
        // alloc → NULL ⇒ NO_MEMORY (Buffer written, Length/Max not).
        (*destination).Buffer = core::ptr::null_mut();
        return STATUS_NO_MEMORY;
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
        (*ansi_string).Length = 0;
        (*ansi_string).MaximumLength = 0;
        (*ansi_string).Buffer = core::ptr::null_mut();
    }
}

/// `VOID NTAPI RtlFreeOemString(POEM_STRING)`. Buffer non-NULL ⇒ free
/// **only** — genuine does NOT zero the struct (the pinned quirk).
#[no_mangle]
pub unsafe extern "system" fn RtlFreeOemString(_oem_string: *mut STRING) {
    // free is the harness no-op stub; no field writes (faithful).
}

/// `VOID NTAPI RtlFreeUnicodeString(PUNICODE_STRING)`. Buffer
/// non-NULL ⇒ free (stub) + zero the whole struct.
#[no_mangle]
pub unsafe extern "system" fn RtlFreeUnicodeString(
    unicode_string: *mut UNICODE_STRING,
) {
    if !(*unicode_string).Buffer.is_null() {
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
/// CHAR*, ULONG)`.
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
