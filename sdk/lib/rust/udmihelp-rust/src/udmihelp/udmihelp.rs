// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_602B7AA459C2: [u16; 11] = [
    0x0053u16, 0x004D, 0x0042, 0x0069, 0x006F, 0x0073, 0x0044, 0x0061, 0x0074, 0x0061,
    0x0000,
];
pub static RUSTCONV_WSTR_990385C876A2: [u16; 13] = [
    0x006Bu16, 0x0065, 0x0072, 0x006E, 0x0065, 0x006C, 0x0033, 0x0032, 0x002E, 0x0064,
    0x006C, 0x006C, 0x0000,
];
pub static RUSTCONV_WSTR_AC1693EFEEAE: [u16; 48] = [
    0x0053u16, 0x0059, 0x0053, 0x0054, 0x0045, 0x004D, 0x005C, 0x0043, 0x0075, 0x0072,
    0x0072, 0x0065, 0x006E, 0x0074, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F,
    0x006C, 0x0053, 0x0065, 0x0074, 0x005C, 0x0053, 0x0065, 0x0072, 0x0076, 0x0069,
    0x0063, 0x0065, 0x0073, 0x005C, 0x006D, 0x0073, 0x0073, 0x006D, 0x0062, 0x0069,
    0x006F, 0x0073, 0x005C, 0x0044, 0x0061, 0x0074, 0x0061, 0x0000,
];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut pGetSystemFirmwareTable: Option<
    unsafe extern "C" fn(DWORD, DWORD, PVOID, DWORD) -> UINT,
> = None;

static mut bInitAPI: BOOL = (FALSE as i32);

unsafe extern "C" fn InitializeAPI() {
    let mut hKernel: HANDLE = core::ptr::null_mut();

    pGetSystemFirmwareTable = None;

    hKernel = GetModuleHandleW(RUSTCONV_WSTR_990385C876A2.as_ptr()) as HANDLE;
    if hKernel.is_null() {
        return;
    }

    pGetSystemFirmwareTable = core::mem::transmute::<
        *mut core::ffi::c_void,
        Option<unsafe extern "C" fn(DWORD, DWORD, PVOID, DWORD) -> UINT>,
    >(core::mem::transmute::<FARPROC, *mut core::ffi::c_void>(
        GetProcAddress(
            hKernel as HINSTANCE,
            b"GetSystemFirmwareTable\0".as_ptr() as LPCSTR,
        ),
    ));
}
#[no_mangle]
pub unsafe extern "C" fn LoadSMBiosData(Strings: *mut PCHAR) -> PVOID {
    let mut pBuffer: PVOID = core::ptr::null_mut();
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut dwType: DWORD = 0;
    let mut dwCheck: DWORD = 0;
    let mut dwBytes: DWORD = 0;

    if bInitAPI == 0 {
        InitializeAPI();
        bInitAPI = (TRUE as i32);
    }

    /* Try using GetSystemFirmwareTable (works on NT 5.2 and higher) */
    if pGetSystemFirmwareTable.is_some() {
        dwBytes = (pGetSystemFirmwareTable.unwrap_unchecked())(
            1381190978 as DWORD,
            0,
            core::ptr::null_mut(),
            0,
        );
        if dwBytes > 0 {
            pBuffer = HeapAlloc(
                GetProcessHeap(),
                0,
                (dwBytes) as SIZE_T,
            );
            if pBuffer.is_null() {
                return core::ptr::null_mut();
            }
            dwCheck = (pGetSystemFirmwareTable.unwrap_unchecked())(
                1381190978 as DWORD,
                0,
                pBuffer,
                dwBytes,
            );
            if dwCheck != dwBytes {
                HeapFree(
                    GetProcessHeap(),
                    0,
                    pBuffer,
                );
                return core::ptr::null_mut();
            }
        }
    }
    if dwBytes == 0 {
        /* Try using registry (works on NT 5.1) */
        if RegOpenKeyW(
            HKEY_LOCAL_MACHINE,
            RUSTCONV_WSTR_AC1693EFEEAE.as_ptr(),
            core::ptr::addr_of_mut!(hKey),
        ) != (ERROR_SUCCESS as i32)
        {
            return core::ptr::null_mut();
        }

        if RegQueryValueExW(
            hKey,
            RUSTCONV_WSTR_602B7AA459C2.as_ptr(),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(dwType),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(dwBytes),
        ) != (ERROR_SUCCESS as i32)
            || dwType != REG_BINARY
        {
            RegCloseKey(hKey);
            return core::ptr::null_mut();
        }

        pBuffer = HeapAlloc(
            GetProcessHeap(),
            0,
            (dwBytes) as SIZE_T,
        );
        if pBuffer.is_null() {
            RegCloseKey(hKey);
            return core::ptr::null_mut();
        }

        if RegQueryValueExW(
            hKey,
            RUSTCONV_WSTR_602B7AA459C2.as_ptr(),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(dwType),
            (pBuffer).cast(),
            core::ptr::addr_of_mut!(dwBytes),
        ) != (ERROR_SUCCESS as i32)
            || dwType != REG_BINARY
        {
            HeapFree(
                GetProcessHeap(),
                0,
                pBuffer,
            );
            RegCloseKey(hKey);
            return core::ptr::null_mut();
        }

        RegCloseKey(hKey);
    }
    ParseSMBiosTables(pBuffer, dwBytes, Strings);
    return pBuffer;
}
#[no_mangle]
pub unsafe extern "C" fn TrimDmiStringW(mut pStr: PWSTR) {
    let mut Length: SIZE_T;
    let mut i: UINT = 0;

    if pStr.is_null() {
        return;
    }

    Length = wcslen(pStr as *const WCHAR) as SIZE_T;
    if Length == 0 {
        return;
    }

    /* Trim leading spaces */
    while (i as SIZE_T) < Length && *pStr.offset(i as isize) <= 0x0020u16 {
        i = i.wrapping_add(1);
    }

    if i > 0 {
        Length = Length.wrapping_sub(i as SIZE_T);
        memmove(
            pStr as *mut core::ffi::c_void,
            pStr.offset(i as isize) as *const core::ffi::c_void,
            ((Length as usize + 1) * core::mem::size_of::<WCHAR>()) as size_t,
        );
    }

    /* Trim trailing spaces */
    while Length != 0
        && *pStr.offset((Length - 1) as isize) <= 0x0020u16
    {
        *pStr.offset((Length - 1) as isize) = 0x0000u16;
        Length = Length.wrapping_sub(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn GetSMBiosStringW(
    mut DmiString: PCSTR,
    mut pBuf: PWSTR,
    mut cchBuf: DWORD,
    mut bTrim: BOOL,
) -> SIZE_T {
    let mut cChars: SIZE_T = 0;
    if DmiString.is_null() {
        if cchBuf >= 1 as DWORD {
            *pBuf = 0 as WCHAR;
        }
        return 0 as SIZE_T;
    }
    cChars = MultiByteToWideChar(
        CP_OEMCP as UINT,
        0 as DWORD,
        DmiString as LPCSTR,
        -(1 as ::core::ffi::c_int),
        pBuf as LPWSTR,
        cchBuf as ::core::ffi::c_int,
    ) as SIZE_T;
    /* NULL-terminate string */
    *pBuf.offset(
        min!(
            cchBuf.wrapping_sub(1 as DWORD) as SIZE_T,
            cChars,
        ) as isize,
    ) = 0x0000u16;
    if bTrim != 0 {
        TrimDmiStringW(pBuf);
    }
    return wcslen(pBuf as *const WCHAR) as SIZE_T;
}
#[no_mangle]
pub unsafe extern "C" fn FreeSMBiosData(mut Buffer: PVOID) {
    if Buffer.is_null() {
        return;
    }
    HeapFree(GetProcessHeap(), 0 as DWORD, Buffer);
}
