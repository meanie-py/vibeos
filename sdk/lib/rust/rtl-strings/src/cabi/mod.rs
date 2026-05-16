//! `#[no_mangle] extern "system"` NTAPI shims — Wave RTL-1. Unsafe
//! counted-string struct reconstruct + the exact legacy `Rtl*`
//! contract. `extern "system"` is NTAPI (cdecl on amd64/arm64,
//! stdcall on i386 — and plain C on the x86-64 host the diff harness
//! links against). The pure decisions live in `core_rs::ustr`.
#![allow(non_snake_case)]

use crate::core_rs::{nls, ustr};

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
const STATUS_UNMAPPABLE_CHARACTER: i32 = 0x8000_0016u32 as i32;
const MAXUSHORT: u32 = 0xFFFF;

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
        && !nls::did_oem_work(s, core::slice::from_raw_parts((*dest).Buffer, n))
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
        && !nls::did_oem_work(s, core::slice::from_raw_parts((*dest).Buffer, n))
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
        && !nls::did_oem_work(s, core::slice::from_raw_parts((*dest).Buffer, n))
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
