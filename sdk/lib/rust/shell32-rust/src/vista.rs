//! Registry MUI-string helpers — translation of dll/win32/shell32/vista.c.
//!
//! shell32's own reimplementation of `RegLoadMUIStringW`/`RegLoadMUIStringA` (originally
//! copied from advapi32/reg/reg.c). Loads a REG_SZ/REG_EXPAND_SZ value, expands env vars
//! when appropriate, and — when the value is a `@dll,-resId` indirect-string reference —
//! loads the referenced string resource. The private `load_string` helper is a copy of
//! user32's LoadStringW kept here to avoid importing user32.
//!
//! Both exports are WINAPI → `extern "system"` → i386 `_RegLoadMUIStringW@28` /
//! `_RegLoadMUIStringA@28` (7 args × 4 bytes). Debug/trace machinery (FIXME/WINE channel)
//! is dropped (no observable behavior).

use crate::types::*;
// One declaration per external symbol: island.rs.
use crate::island::FindResourceW;
use crate::island::FreeLibrary;
use crate::island::HeapAlloc;
use crate::island::LoadLibraryExW;
use crate::island::LoadResource;
use crate::island::RegQueryValueExW;
use core::mem::size_of;
use core::ptr;

// ── file-local types (candidates for hoisting into types.rs) ──────────────────────────
type USHORT = u16;
use crate::island::HMODULE;
use crate::island::HINSTANCE;
type HRSRC = HANDLE;
type HGLOBAL = HANDLE;
use ntdll_sys::nt::SIZE_T;
type PCSZ = LPCSTR;

/// UNICODE_STRING (ndk/rtlfuncs.h): { USHORT Length; USHORT MaximumLength; PWSTR Buffer; }.
#[repr(C)]
#[derive(Copy, Clone)]
struct UNICODE_STRING {
    Length: USHORT,
    MaximumLength: USHORT,
    Buffer: LPWSTR,
}

// ── constants (pinned from SDK headers) ───────────────────────────────────────────────
const ERROR_INVALID_PARAMETER: LONG = 87; // winerror.h
const ERROR_FILE_NOT_FOUND: LONG = 2; // winerror.h
const ERROR_NOT_ENOUGH_MEMORY: LONG = 8; // winerror.h
const ERROR_BADKEY: LONG = 1010; // winerror.h
const REG_SZ: DWORD = 1; // winnt.h
const REG_EXPAND_SZ: DWORD = 2; // winnt.h
const LOAD_LIBRARY_AS_DATAFILE: DWORD = 0x00000002; // winbase.h
const CP_ACP: UINT = 0; // winnls.h
const RT_STRING: usize = 6; // winuser.h: RT_STRING = MAKEINTRESOURCE(6)

// ── imports (resolved via shell32's existing import libs at link) ──────────────────────
extern "system" {
    // advapi32
// _RegQueryValueExW@24
    // kernel32
    fn GetProcessHeap() -> HANDLE; // _GetProcessHeap@0
// _HeapAlloc@12
    fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: LPVOID) -> BOOL; // _HeapFree@12
    fn ExpandEnvironmentStringsW(lpSrc: LPCWSTR, lpDst: LPWSTR, nSize: DWORD) -> DWORD; // _ExpandEnvironmentStringsW@12
// _FindResourceW@12
// _LoadResource@8
    fn LockResource(hResData: HGLOBAL) -> LPVOID; // _LockResource@4
// _LoadLibraryExW@12
// _FreeLibrary@4
    fn lstrcpynW(lpString1: LPWSTR, lpString2: LPCWSTR, iMaxLength: INT) -> LPWSTR; // _lstrcpynW@12
    fn WideCharToMultiByte(
        CodePage: UINT,
        dwFlags: DWORD,
        lpWideCharStr: LPCWSTR,
        cchWideChar: INT,
        lpMultiByteStr: LPSTR,
        cbMultiByte: INT,
        lpDefaultChar: LPCSTR,
        lpUsedDefaultChar: *mut BOOL,
    ) -> INT; // _WideCharToMultiByte@32
    // ntdll (NTAPI = stdcall on i386)
    fn RtlCreateUnicodeStringFromAsciiz(
        DestinationString: *mut UNICODE_STRING,
        SourceString: PCSZ,
    ) -> BOOLEAN; // __imp__RtlCreateUnicodeStringFromAsciiz@8
    fn RtlFreeUnicodeString(UnicodeString: *mut UNICODE_STRING); // __imp__RtlFreeUnicodeString@4
}

extern "C" {
    // msvcrt (cdecl)
    fn _wtoi(str: *const WCHAR) -> INT; // __imp___wtoi
    fn wcsrchr(str: *const WCHAR, c: WCHAR) -> *mut WCHAR; // __imp__wcsrchr
}

/// load_string [Internal] — copy of user32/resource.c's LoadStringW (avoids importing
/// user32). Helper for RegLoadMUIString.
unsafe fn load_string(hModule: HINSTANCE, mut resId: UINT, pwszBuffer: LPWSTR, cMaxChars: INT) -> INT {
    // Negative values have to be inverted.
    if (resId >> 16) & 0xffff == 0xffff {
        resId = (-(resId as INT)) as UINT;
    }

    // Load the resource into memory and get a pointer to it.
    let name = ((((resId >> 4) & 0xffff) + 1) as usize) as LPCWSTR; // MAKEINTRESOURCEW(LOWORD(resId >> 4) + 1)
    let hResource = FindResourceW((hModule).cast(), name, RT_STRING as LPCWSTR);
    if hResource.is_null() {
        return 0;
    }
    let hMemory = LoadResource((hModule).cast(), hResource);
    if hMemory.is_null() {
        return 0;
    }
    let mut pString = LockResource(hMemory) as *const WCHAR;

    // Strings are length-prefixed. Lowest nibble of resId is an index.
    let mut idxString = (resId & 0xf) as INT;
    while idxString != 0 {
        idxString -= 1;
        pString = pString.add((*pString as usize) + 1);
    }

    // If no buffer is given, return length of the string.
    if pwszBuffer.is_null() {
        return *pString as INT;
    }

    // Else copy over the string, respecting the buffer size.
    let str_len = *pString as INT;
    let n = if str_len < cMaxChars { str_len } else { cMaxChars - 1 };
    if n >= 0 {
        ptr::copy(pString.add(1), pwszBuffer, n as usize);
        pwszBuffer.add(n as usize).write_volatile(0); // L'\0'
    }

    n
}

/// RegLoadMUIStringW [@implemented]
#[no_mangle]
pub unsafe extern "system" fn RegLoadMUIStringW(
    hKey: HKEY,
    pszValue: LPCWSTR,
    pszOutBuf: LPWSTR,
    cbOutBuf: DWORD,
    _pcbData: LPDWORD,
    _Flags: DWORD,
    pszDirectory: LPCWSTR,
) -> LONG {
    let mut dwValueType: DWORD = 0;
    let mut cbData: DWORD = 0;
    let mut pwszTempBuffer: LPWSTR = ptr::null_mut();
    let mut pwszExpandedBuffer: LPWSTR = ptr::null_mut();

    let result: LONG = 'cleanup: {
        // Parameter sanity checks.
        if hKey.is_null() || pszOutBuf.is_null() {
            break 'cleanup ERROR_INVALID_PARAMETER;
        }

        if !pszDirectory.is_null() && *pszDirectory != 0 {
            // FIXME: BaseDir parameter not yet supported!
            break 'cleanup ERROR_INVALID_PARAMETER;
        }

        // Check for value existence and correctness of its type, allocate a buffer and load it.
        let mut result = RegQueryValueExW(
            hKey,
            pszValue,
            ptr::null_mut(),
            &mut dwValueType,
            ptr::null_mut(),
            &mut cbData,
        );
        if result != ERROR_SUCCESS {
            break 'cleanup result;
        }
        if !(dwValueType == REG_SZ || dwValueType == REG_EXPAND_SZ) || cbData == 0 {
            break 'cleanup ERROR_FILE_NOT_FOUND;
        }
        pwszTempBuffer = HeapAlloc(GetProcessHeap(), 0, (cbData as SIZE_T) as _) as LPWSTR;
        if pwszTempBuffer.is_null() {
            break 'cleanup ERROR_NOT_ENOUGH_MEMORY;
        }
        result = RegQueryValueExW(
            hKey,
            pszValue,
            ptr::null_mut(),
            &mut dwValueType,
            pwszTempBuffer as LPBYTE,
            &mut cbData,
        );
        if result != ERROR_SUCCESS {
            break 'cleanup result;
        }

        // Expand environment variables, if appropriate, or copy the original string over.
        if dwValueType == REG_EXPAND_SZ {
            cbData = ExpandEnvironmentStringsW(pwszTempBuffer, ptr::null_mut(), 0)
                .wrapping_mul(size_of::<WCHAR>() as DWORD);
            if cbData == 0 {
                break 'cleanup result;
            }
            pwszExpandedBuffer = HeapAlloc(GetProcessHeap(), 0, (cbData as SIZE_T) as _) as LPWSTR;
            if pwszExpandedBuffer.is_null() {
                break 'cleanup ERROR_NOT_ENOUGH_MEMORY;
            }
            ExpandEnvironmentStringsW(pwszTempBuffer, pwszExpandedBuffer, cbData);
        } else {
            pwszExpandedBuffer = HeapAlloc(GetProcessHeap(), 0, (cbData as SIZE_T) as _) as LPWSTR;
            ptr::copy(pwszTempBuffer as *const u8, pwszExpandedBuffer as *mut u8, cbData as usize);
        }

        // If the value references a resource based string, parse the value and load the
        // string. Else just copy over the original value.
        result = ERROR_SUCCESS;
        if *pwszExpandedBuffer != b'@' as WCHAR {
            // '@' is the prefix for resource based string entries.
            lstrcpynW(pszOutBuf, pwszExpandedBuffer, (cbOutBuf / size_of::<WCHAR>() as DWORD) as INT);
        } else {
            let pComma = wcsrchr(pwszExpandedBuffer, b',' as WCHAR);

            // Format of the expanded value is 'path_to_dll,-resId'
            if pComma.is_null() || *pComma.add(1) != b'-' as WCHAR {
                break 'cleanup ERROR_BADKEY;
            }

            let uiStringId = _wtoi(pComma.add(2)) as UINT;
            pComma.write_volatile(0); // *pComma = L'\0';

            let hModule =
                LoadLibraryExW(pwszExpandedBuffer.add(1), ptr::null_mut(), LOAD_LIBRARY_AS_DATAFILE);
            if hModule.is_null()
                || load_string((hModule).cast(), uiStringId, pszOutBuf, (cbOutBuf / size_of::<WCHAR>() as DWORD) as INT) == 0
            {
                result = ERROR_BADKEY;
            }
            FreeLibrary(hModule);
        }

        result
    };

    HeapFree(GetProcessHeap(), 0, pwszTempBuffer as LPVOID);
    HeapFree(GetProcessHeap(), 0, pwszExpandedBuffer as LPVOID);
    result
}

/// RegLoadMUIStringA [@implemented]
#[no_mangle]
pub unsafe extern "system" fn RegLoadMUIStringA(
    hKey: HKEY,
    pszValue: LPCSTR,
    pszOutBuf: LPSTR,
    cbOutBuf: DWORD,
    pcbData: LPDWORD,
    Flags: DWORD,
    pszDirectory: LPCSTR,
) -> LONG {
    let mut valueW: UNICODE_STRING = core::mem::zeroed();
    let mut baseDirW: UNICODE_STRING = core::mem::zeroed();
    let mut pwszBuffer: LPWSTR = ptr::null_mut();
    let mut cbData: DWORD = cbOutBuf.wrapping_mul(size_of::<WCHAR>() as DWORD);

    // valueW.Buffer = baseDirW.Buffer = pwszBuffer = NULL; (zeroed above)
    let result: LONG = 'cleanup: {
        if RtlCreateUnicodeStringFromAsciiz(&mut valueW, pszValue) == 0
            || RtlCreateUnicodeStringFromAsciiz(&mut baseDirW, pszDirectory) == 0
            || {
                pwszBuffer = HeapAlloc(GetProcessHeap(), 0, (cbData as SIZE_T) as _) as LPWSTR;
                pwszBuffer.is_null()
            }
        {
            break 'cleanup ERROR_NOT_ENOUGH_MEMORY;
        }

        let result = RegLoadMUIStringW(
            hKey,
            valueW.Buffer,
            pwszBuffer,
            cbData,
            ptr::null_mut(),
            Flags,
            baseDirW.Buffer,
        );

        if result == ERROR_SUCCESS {
            cbData = WideCharToMultiByte(
                CP_ACP,
                0,
                pwszBuffer,
                -1,
                pszOutBuf,
                cbOutBuf as INT,
                ptr::null(),
                ptr::null_mut(),
            ) as DWORD;
            if !pcbData.is_null() {
                pcbData.write_volatile(cbData);
            }
        }

        result
    };

    HeapFree(GetProcessHeap(), 0, pwszBuffer as LPVOID);
    RtlFreeUnicodeString(&mut baseDirW);
    RtlFreeUnicodeString(&mut valueW);

    result
}

// ADDED-TO-TYPES?: UNICODE_STRING (ndk/rtlfuncs.h), USHORT, SIZE_T, PCSZ,
// HMODULE/HINSTANCE/HRSRC/HGLOBAL aliases — common across the shell32 waves.
