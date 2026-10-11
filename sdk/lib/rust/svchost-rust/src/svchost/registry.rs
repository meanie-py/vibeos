use crate::svchost::globals::MemAlloc;
use crate::svchost::globals::MemFree;
use crate::prelude::*;
use core::ffi::c_void;

extern "system" {
    fn RtlAssert(FailedAssertion: PVOID, FileName: PVOID, LineNumber: ULONG, Message: PCHAR);
}
#[no_mangle]
pub unsafe extern "system" fn RegQueryValueWithAlloc(
    hKey: HKEY,
    pszValueName: LPCWSTR,
    dwExpectedType: DWORD,
    ppbData: *mut PBYTE,
    pdwSize: PDWORD,
) -> DWORD {
    let mut dwError: DWORD = 0;
    let mut dwType: DWORD = 0;
    let mut dwBytes: DWORD = 0;
    let mut pbData: PBYTE = core::ptr::null_mut();

    /* Assume failure */
    *ppbData = core::ptr::null_mut();
    *pdwSize = 0;

    /* Query how big and what type the registry data is */
    dwBytes = 0;
    dwError = RegQueryValueExW(
        hKey,
        pszValueName,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(dwType),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(dwBytes),
    ) as DWORD;
    if dwError != ERROR_SUCCESS {
        return dwError;
    }

    /* It if's not the right type, or it's sero bytes, fail*/
    if (dwType != dwExpectedType) || (dwBytes == 0) {
        return ERROR_INVALID_DATA;
    }

    /* Allocate space to hold the data */
    pbData = MemAlloc(0, dwBytes) as PBYTE;
    if pbData.is_null() {
        return ERROR_OUTOFMEMORY;
    }

    /* Now get the real registry data */
    dwError = RegQueryValueExW(
        hKey,
        pszValueName,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(dwType),
        pbData,
        core::ptr::addr_of_mut!(dwBytes),
    ) as DWORD;
    if dwError != ERROR_SUCCESS {
        /* We failed, free the data since it won't be needed */
        MemFree(pbData as LPVOID);
    } else {
        /* It worked, return the data and size back to the caller */
        *ppbData = pbData;
        *pdwSize = dwBytes;
    }

    /* All done */
    return dwError;
}

#[no_mangle]
pub unsafe extern "system" fn RegQueryDword(hKey: HKEY, pszValueName: LPCWSTR, pdwValue: PDWORD) -> DWORD {
    let mut dwError: DWORD = 0;
    let mut cbData: DWORD = 0;
    let mut dwType: DWORD = 0;

    /* Attempt to read 4 bytes */
    cbData = ::core::mem::size_of::<DWORD>() as DWORD;
    dwError = RegQueryValueExW(hKey, pszValueName, core::ptr::null_mut(), core::ptr::addr_of_mut!(dwType), core::ptr::null_mut(), core::ptr::addr_of_mut!(cbData)) as DWORD;

    /* If we didn't get back a DWORD... */
    if (dwError == ERROR_SUCCESS as DWORD) && (dwType != REG_DWORD as DWORD) {
        /* Zero out the output and fail */
        *pdwValue = 0 as DWORD;
        dwError = ERROR_INVALID_DATATYPE as DWORD;
    }

    /* All done! */
    return dwError;
}

#[no_mangle]
pub unsafe extern "system" fn RegQueryString(hKey: HKEY, pszValueName: LPCWSTR, dwExpectedType: DWORD, ppbData: *mut PBYTE) -> DWORD {
    let mut dwSize: DWORD = 0;
    if hKey.is_null() {
        RtlAssert(
            b"hKey\0" as *const u8 as PVOID,
            b"base/services/svchost/registry.c\0" as *const u8 as PVOID,
            115 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };
    if pszValueName.is_null() {
        RtlAssert(
            b"pszValueName\0" as *const u8 as PVOID,
            b"base/services/svchost/registry.c\0" as *const u8 as PVOID,
            116 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };
    return RegQueryValueWithAlloc(
        hKey,
        pszValueName,
        dwExpectedType,
        ppbData,
        core::ptr::addr_of_mut!(dwSize),
    );
}

#[no_mangle]
pub unsafe extern "system" fn RegQueryStringA(hKey: HKEY, pszValueName: LPCWSTR, dwExpectedType: DWORD, ppszData: *mut LPCSTR) -> DWORD {
    let mut dwError: DWORD;
    let mut pbLocalData: LPWSTR = core::ptr::null_mut();
    let mut cchValueName: DWORD;
    let mut cbMultiByte: DWORD;
    let mut pszData: LPSTR;

    /* Assume failure */
    *ppszData = core::ptr::null();

    /* Query the string in Unicode first */
    dwError = RegQueryString(
        hKey,
        pszValueName,
        dwExpectedType,
        core::ptr::addr_of_mut!(pbLocalData) as *mut PBYTE,
    );
    if dwError != ERROR_SUCCESS {
        return dwError;
    }

    /* Get the length of the Unicode string */
    cchValueName = lstrlenW(pbLocalData as LPCWSTR) as DWORD;

    /* See how much space it would take to convert to ANSI */
    cbMultiByte = WideCharToMultiByte(
        CP_ACP as UINT,
        0 as DWORD,
        pbLocalData as LPCWSTR,
        cchValueName.wrapping_add(1) as core::ffi::c_int,
        core::ptr::null_mut(),
        0 as core::ffi::c_int,
        core::ptr::null(),
        core::ptr::null_mut(),
    ) as DWORD;
    if cbMultiByte != 0 {
        /* Allocate the space, assuming failure */
        dwError = ERROR_OUTOFMEMORY;
        pszData = MemAlloc(0 as DWORD, cbMultiByte) as LPSTR;
        if !pszData.is_null() {
            /* What do you know, it worked! */
            dwError = ERROR_SUCCESS;

            /* Now do the real conversion */
            if WideCharToMultiByte(
                CP_ACP as UINT,
                0 as DWORD,
                pbLocalData as LPCWSTR,
                cchValueName.wrapping_add(1) as core::ffi::c_int,
                pszData,
                cbMultiByte as core::ffi::c_int,
                core::ptr::null(),
                core::ptr::null_mut(),
            ) != 0 as core::ffi::c_int
            {
                /* It worked, return the data back to the caller */
                *ppszData = pszData as LPCSTR;
            } else {
                /* It failed, free our buffer and get the error code */
                MemFree(pszData as LPVOID);
                dwError = GetLastError();
            }
        }
    }

    /* Free the original Unicode string and return the error */
    MemFree(pbLocalData as LPVOID);
    return dwError;
}
