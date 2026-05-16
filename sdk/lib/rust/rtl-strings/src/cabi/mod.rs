//! `#[no_mangle] extern "system"` NTAPI shims — Wave RTL-1. Unsafe
//! counted-string struct reconstruct + the exact legacy `Rtl*`
//! contract. `extern "system"` is NTAPI (cdecl on amd64/arm64,
//! stdcall on i386 — and plain C on the x86-64 host the diff harness
//! links against). The pure decisions live in `core_rs::ustr`.
#![allow(non_snake_case)]

use crate::core_rs::ustr;

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
/// BOOLEAN)` — Wave RTL-1 pins the case-SENSITIVE path (the
/// case-insensitive path needs `RtlUpperChar`, an NLS leaf → a later
/// wave on the shared CP1252 model).
#[no_mangle]
pub unsafe extern "system" fn RtlCompareString(
    s1: *const STRING,
    s2: *const STRING,
    case_insensitive: u8,
) -> i32 {
    if case_insensitive != 0 {
        // Not pinned in Wave RTL-1 (NLS leaf); never fuzzed here.
        return 0;
    }
    ustr::compare(sbytes(s1), sbytes(s2))
}

/// `BOOLEAN NTAPI RtlEqualString(const STRING*, const STRING*,
/// BOOLEAN)` (case-sensitive path).
#[no_mangle]
pub unsafe extern "system" fn RtlEqualString(
    s1: *const STRING,
    s2: *const STRING,
    case_insensitive: u8,
) -> u8 {
    if case_insensitive != 0 {
        return 0;
    }
    // genuine: `if (L1 != L2) return FALSE; return !RtlCompareString`
    if (*s1).Length != (*s2).Length {
        return 0;
    }
    ustr::equal(sbytes(s1), sbytes(s2)) as u8
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
    if case_insensitive != 0 {
        return 0;
    }
    // `if (String2->Length < String1->Length) return FALSE;` is
    // checked before the buffer test in the genuine code.
    if (*string2).Length < (*string1).Length {
        return 0;
    }
    if (*string1).Buffer.is_null() || (*string2).Buffer.is_null() {
        return 0;
    }
    ustr::prefix(sbytes(string1), sbytes(string2)) as u8
}
