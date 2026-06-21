#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(unused_parens, unused_assignments)]

//! kernel32-shared-rust — Rust translation of the `kernel32_shared` static lib
//! (`client/shared_utils.c` + `client/file/fileutils.c`). This .a is linked into
//! BOTH `kernel32.dll` and `kernel32_vista.dll` (the C `kernel32_shared` target is
//! dropped from both link lines). All symbols are `#[no_mangle]` C-ABI providers.
//! See `docs/engineering/kernel32-shared-vista-rust-plan.md`.

use ntdll_sys::nt::*;

// kernel32 exports — provided by kernel32.dll's own body (utils.rs AreFileApisANSI,
// the SetLastError export); kernel32_vista.dll resolves them via its kernel32
// import lib. WINAPI → stdcall on i386 (`_Name@N`).
extern "system" {
    fn AreFileApisANSI() -> BOOL;
    fn SetLastError(dwErrCode: DWORD);
}

// ntdll exports these as forwarders to the Rtlx* implementations (ntdll.spec
// :1216/:1220). In C they are FORCEINLINE macros
// (`NlsMb*CodePageTag ? Rtlx*(s) : (s->Length + 2)/2`); the Rtlx-backed exported
// function returns the IDENTICAL size on both the SBCS fast path and the MBCS
// path, so calling it unconditionally is behaviour-faithful and avoids the
// cross-DLL `__imp_NlsMbCodePageTag` data import (the link uses --disable-auto-import).
extern "system" {
    fn RtlUnicodeStringToAnsiSize(UnicodeString: PCUNICODE_STRING) -> ULONG;
    fn RtlUnicodeStringToOemSize(UnicodeString: PCUNICODE_STRING) -> ULONG;
}

// CRT (resolved via ntdll/crt at the DLL link, same as kernel32-rust).
extern "C" {
    fn wcslen(s: *const WCHAR) -> usize;
    fn strlen(s: *const CHAR) -> usize;
}

const ERROR_FILENAME_EXCED_RANGE: DWORD = 206;
const MAX_PATH_I: INT = 260;

#[inline]
fn NT_SUCCESS(Status: NTSTATUS) -> bool {
    Status >= 0
}

// no_std staticlib floor. Duplicates kernel32-rust's handler when both .a's link
// into one DLL — resolved by the link's --allow-multiple-definition (first wins).
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}

// ===========================================================================
// client/shared_utils.c
// ===========================================================================

/// `BaseSetLastNTError` — convert an NTSTATUS to a Win32 error code and set it.
#[no_mangle]
pub unsafe extern "system" fn BaseSetLastNTError(Status: NTSTATUS) -> DWORD {
    // Convert from NT to Win32, then set
    let dwErrCode: DWORD = RtlNtStatusToDosError(Status);
    SetLastError(dwErrCode);
    dwErrCode
}

// ===========================================================================
// client/file/fileutils.c
// ===========================================================================

#[no_mangle]
pub unsafe extern "C" fn FilenameA2W(NameA: LPCSTR, alloc: BOOL) -> PWCHAR {
    let mut str: ANSI_STRING = core::mem::zeroed();
    let mut strW: UNICODE_STRING = core::mem::zeroed();
    let pstrW: PUNICODE_STRING;
    let Status: NTSTATUS;

    // (NDEBUG: the C ASSERT on StaticUnicodeString.MaximumLength is compiled out.)

    RtlInitAnsiString(&mut str, NameA);
    pstrW = if alloc != 0 {
        &mut strW
    } else {
        &mut (*NtCurrentTeb()).StaticUnicodeString
    };

    if AreFileApisANSI() != 0 {
        Status = RtlAnsiStringToUnicodeString(pstrW, &mut str, alloc as BOOLEAN);
    } else {
        Status = RtlOemStringToUnicodeString(pstrW, &mut str, alloc as BOOLEAN);
    }

    if NT_SUCCESS(Status) {
        return (*pstrW).Buffer;
    }

    if Status == STATUS_BUFFER_OVERFLOW {
        SetLastError(ERROR_FILENAME_EXCED_RANGE);
    } else {
        BaseSetLastNTError(Status);
    }

    core::ptr::null_mut()
}

/// No copy/conversion is done if the dest buffer is too small.
/// Success: TCHARs copied (excl. NUL). Fail: TCHARs required (incl. NUL).
#[no_mangle]
pub unsafe extern "C" fn FilenameU2A_FitOrFail(
    DestA: LPSTR,
    mut destLen: INT, // buffer size in TCHARS incl. nullchar
    SourceU: PUNICODE_STRING,
) -> DWORD {
    let mut ret: DWORD;

    // destLen should never exceed MAX_PATH
    if destLen > MAX_PATH_I {
        destLen = MAX_PATH_I;
    }

    ret = if AreFileApisANSI() != 0 {
        RtlUnicodeStringToAnsiSize(SourceU as PCUNICODE_STRING)
    } else {
        RtlUnicodeStringToOemSize(SourceU as PCUNICODE_STRING)
    };
    // ret incl. nullchar

    if !DestA.is_null() && (ret as INT) <= destLen {
        let mut str: ANSI_STRING = core::mem::zeroed();
        str.Buffer = DestA;
        str.MaximumLength = destLen as USHORT;

        if AreFileApisANSI() != 0 {
            RtlUnicodeStringToAnsiString(&mut str, SourceU as PCUNICODE_STRING, FALSE as BOOLEAN);
        } else {
            RtlUnicodeStringToOemString(&mut str, SourceU as PCUNICODE_STRING, FALSE as BOOLEAN);
        }

        ret = str.Length as DWORD; // SUCCESS: length without terminating 0
    }

    ret
}

/// Success: TCHARs copied (excl. NUL). Fail: TCHARs required (incl. NUL).
#[no_mangle]
pub unsafe extern "C" fn FilenameW2A_FitOrFail(
    DestA: LPSTR,
    destLen: INT,   // buffer size in TCHARS incl. nullchar
    SourceW: LPCWSTR,
    mut sourceLen: INT, // buffer size in TCHARS incl. nullchar
) -> DWORD {
    let mut strW: UNICODE_STRING = core::mem::zeroed();

    if sourceLen < 0 {
        sourceLen = wcslen(SourceW) as INT + 1;
    }

    strW.Buffer = SourceW as PWCHAR;
    strW.MaximumLength = (sourceLen * core::mem::size_of::<WCHAR>() as INT) as USHORT;
    strW.Length = strW.MaximumLength - core::mem::size_of::<WCHAR>() as USHORT;

    FilenameU2A_FitOrFail(DestA, destLen, &mut strW)
}

/// Return: TCHARs copied into dest including NUL.
#[no_mangle]
pub unsafe extern "C" fn FilenameA2W_N(
    dest: LPWSTR,
    destlen: INT, // buffer size in TCHARS incl. nullchar
    src: LPCSTR,
    mut srclen: INT, // buffer size in TCHARS incl. nullchar
) -> DWORD {
    let mut ret: DWORD = 0;

    if srclen < 0 {
        srclen = strlen(src) as INT + 1;
    }

    if AreFileApisANSI() != 0 {
        RtlMultiByteToUnicodeN(
            dest,
            (destlen * core::mem::size_of::<WCHAR>() as INT) as ULONG,
            &mut ret,
            src,
            srclen as ULONG,
        );
    } else {
        RtlOemToUnicodeN(
            dest,
            (destlen * core::mem::size_of::<WCHAR>() as INT) as ULONG,
            &mut ret,
            src,
            srclen as ULONG,
        );
    }

    if ret != 0 {
        *dest.add(ret as usize / core::mem::size_of::<WCHAR>() - 1) = 0;
    }

    ret / core::mem::size_of::<WCHAR>() as DWORD
}

/// Return: TCHARs copied into dest including NUL.
#[no_mangle]
pub unsafe extern "C" fn FilenameW2A_N(
    dest: LPSTR,
    destlen: INT, // buffer size in TCHARS incl. nullchar
    src: LPCWSTR,
    mut srclen: INT, // buffer size in TCHARS incl. nullchar
) -> DWORD {
    let mut ret: DWORD = 0;

    if srclen < 0 {
        srclen = wcslen(src) as INT + 1;
    }

    if AreFileApisANSI() != 0 {
        RtlUnicodeToMultiByteN(
            dest,
            destlen as ULONG,
            &mut ret,
            src,
            (srclen * core::mem::size_of::<WCHAR>() as INT) as ULONG,
        );
    } else {
        RtlUnicodeToOemN(
            dest,
            destlen as ULONG,
            &mut ret,
            src,
            (srclen * core::mem::size_of::<WCHAR>() as INT) as ULONG,
        );
    }

    if ret != 0 {
        *dest.add(ret as usize - 1) = 0;
    }

    ret
}
