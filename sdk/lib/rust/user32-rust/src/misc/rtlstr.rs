use win32k_sys::win32k::*;
// ntdll Rtl* string helpers live in crate::internal (NOT the kernel ntkrnl_sys crate).
use crate::internal::{
    RtlAnsiStringToUnicodeString, RtlInitAnsiString, RtlInitUnicodeString,
};
// wcslen + NT_SUCCESS live in crate::winbase; strlen is CRT (msvcrt).
use crate::winbase::{wcslen, NT_SUCCESS};
#[allow(unused_imports)]
use core::ffi::c_void;

extern "C" {
    fn strlen(s: *const core::ffi::c_char) -> usize;
}

// RtlCreateUnicodeString is an ntdll Rtl* helper (BOOLEAN return); not in crate::internal.
extern "system" {
    fn RtlCreateUnicodeString(DestinationString: PUNICODE_STRING, SourceString: PCWSTR) -> u8;
}

#[no_mangle]
pub unsafe extern "system" fn RtlInitLargeAnsiString(
    DestinationString: PLARGE_ANSI_STRING,
    SourceString: PCSZ,
    Unknown: INT,
) {
    let DestSize: ULONG;

    if !SourceString.is_null() {
        DestSize = strlen(SourceString as *const _) as ULONG;
        (*DestinationString).Length = DestSize;
        (*DestinationString).set_MaximumLength(DestSize + core::mem::size_of::<CHAR>() as ULONG);
    } else {
        (*DestinationString).Length = 0;
        (*DestinationString).set_MaximumLength(0);
    }

    (*DestinationString).Buffer = SourceString as PCHAR;
    (*DestinationString).set_bAnsi(1);
}

#[no_mangle]
pub unsafe extern "system" fn RtlInitLargeUnicodeString(
    DestinationString: PLARGE_UNICODE_STRING,
    SourceString: PCWSTR,
    Unknown: INT,
) {
    let DestSize: ULONG;

    if !SourceString.is_null() {
        DestSize = (wcslen(SourceString) as ULONG) * core::mem::size_of::<WCHAR>() as ULONG;
        (*DestinationString).Length = DestSize;
        (*DestinationString).set_MaximumLength(DestSize + core::mem::size_of::<WCHAR>() as ULONG);
    } else {
        (*DestinationString).Length = 0;
        (*DestinationString).set_MaximumLength(0);
    }

    (*DestinationString).Buffer = SourceString as PWSTR;
    (*DestinationString).set_bAnsi(0);
}

#[no_mangle]
pub unsafe extern "system" fn RtlLargeStringToUnicodeString(
    DestinationString: PUNICODE_STRING,
    SourceString: PLARGE_STRING,
) -> i32 {
    let mut AnsiString: ANSI_STRING = core::mem::zeroed();

    RtlInitUnicodeString(DestinationString, core::ptr::null_mut());
    if !DestinationString.is_null() && !SourceString.is_null() && (*SourceString).bAnsi() != 0 {
        RtlInitAnsiString(&mut AnsiString, (*SourceString).Buffer as LPSTR as PCSZ);
        NT_SUCCESS(RtlAnsiStringToUnicodeString(DestinationString, &mut AnsiString, 1u8)) as i32
    } else if !DestinationString.is_null() && !SourceString.is_null() {
        RtlCreateUnicodeString(DestinationString, (*SourceString).Buffer as PCWSTR) as i32
    } else {
        0
    }
}
