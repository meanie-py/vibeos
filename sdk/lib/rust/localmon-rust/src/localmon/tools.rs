// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_0FA88D88CEB2: [u16; 6] = [0x0046u16, 0x0049, 0x004C, 0x0045, 0x003A, 0x0000];
pub static RUSTCONV_WSTR_20E46296C1D4: [u16; 4] = [0x0058u16, 0x0050, 0x0053, 0x0000];
pub static RUSTCONV_WSTR_3DB566BC3DA4: [u16; 4] = [0x0043u16, 0x004F, 0x004D, 0x0000];
pub static RUSTCONV_WSTR_B862926D575C: [u16; 53] = [
    0x0053u16, 0x004F, 0x0046, 0x0054, 0x0057, 0x0041, 0x0052, 0x0045, 0x005C, 0x004D,
    0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0057,
    0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x0020, 0x004E, 0x0054, 0x005C,
    0x0043, 0x0075, 0x0072, 0x0072, 0x0065, 0x006E, 0x0074, 0x0056, 0x0065, 0x0072,
    0x0073, 0x0069, 0x006F, 0x006E, 0x005C, 0x0057, 0x0069, 0x006E, 0x0064, 0x006F,
    0x0077, 0x0073, 0x0000,
];
pub static RUSTCONV_WSTR_BC56FDFF67BB: [u16; 25] = [
    0x0054u16, 0x0072, 0x0061, 0x006E, 0x0073, 0x006D, 0x0069, 0x0073, 0x0073, 0x0069,
    0x006F, 0x006E, 0x0052, 0x0065, 0x0074, 0x0072, 0x0079, 0x0054, 0x0069, 0x006D,
    0x0065, 0x006F, 0x0075, 0x0074, 0x0000,
];
pub static RUSTCONV_WSTR_C372F2CCF22D: [u16; 4] = [0x004Cu16, 0x0050, 0x0054, 0x0000];

use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn DoesPortExist(pwszPortName: PCWSTR) -> BOOL {
    let mut bReturnValue: BOOL = FALSE as BOOL;
    let mut cbNeeded: DWORD = 0;
    let mut dwErrorCode: DWORD = 0;
    let mut dwReturned: DWORD = 0;
    let mut i: DWORD = 0;
    let mut p: PPORT_INFO_1W = core::ptr::null_mut();
    let mut pPortInfo1: PPORT_INFO_1W = core::ptr::null_mut();

    'cleanup: {
        // Determine the required buffer size.
        EnumPortsW(
            core::ptr::null_mut(),
            1,
            core::ptr::null_mut(),
            0,
            core::ptr::addr_of_mut!(cbNeeded),
            core::ptr::addr_of_mut!(dwReturned),
        );
        if GetLastError() != ERROR_INSUFFICIENT_BUFFER {
            dwErrorCode = GetLastError();
            break 'cleanup;
        }

        // Allocate a buffer large enough.
        pPortInfo1 = DllAllocSplMem(cbNeeded) as PPORT_INFO_1W;
        if pPortInfo1.is_null() {
            dwErrorCode = ERROR_NOT_ENOUGH_MEMORY;
            break 'cleanup;
        }

        // Now get the actual port information.
        if EnumPortsW(
            core::ptr::null_mut(),
            1,
            pPortInfo1 as PBYTE,
            cbNeeded,
            core::ptr::addr_of_mut!(cbNeeded),
            core::ptr::addr_of_mut!(dwReturned),
        ) == 0
        {
            dwErrorCode = GetLastError();
            break 'cleanup;
        }

        // We were successful! Loop through all returned ports.
        dwErrorCode = ERROR_SUCCESS;
        p = pPortInfo1;

        i = 0;
        while i < dwReturned {
            // Check if this existing port matches our queried one.
            if _wcsicmp((*p).pName, pwszPortName) == 0 {
                bReturnValue = TRUE as BOOL;
                break 'cleanup;
            }

            p = p.add(1);
            i += 1;
        }
    }

    // Cleanup:
    if !pPortInfo1.is_null() {
        DllFreeSplMem((pPortInfo1).cast());
    }

    SetLastError(dwErrorCode);
    return bReturnValue;
}
#[no_mangle]
pub unsafe extern "C" fn GetLPTTransmissionRetryTimeout() -> DWORD {
    let mut cbBuffer: DWORD = 0;
    let mut dwReturnValue: DWORD = 90;
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut lStatus: LSTATUS = 0;
    let mut wszBuffer: [WCHAR; 6 + 1] = [0; 6 + 1];

    'Cleanup: {
        lStatus = RegOpenKeyExW(HKEY_LOCAL_MACHINE, RUSTCONV_WSTR_B862926D575C.as_ptr(), 0, KEY_READ, &raw mut hKey);
        if lStatus != ERROR_SUCCESS as LSTATUS {
            break 'Cleanup;
        }

        cbBuffer = core::mem::size_of::<[WCHAR; 6 + 1]>() as DWORD;
        lStatus = RegQueryValueExW(hKey, RUSTCONV_WSTR_BC56FDFF67BB.as_ptr(), core::ptr::null_mut(), core::ptr::null_mut(), &raw mut wszBuffer as *mut WCHAR as PBYTE, &raw mut cbBuffer);
        if lStatus != ERROR_SUCCESS as LSTATUS {
            break 'Cleanup;
        }

        dwReturnValue = wcstoul(&raw mut wszBuffer as *mut WCHAR, core::ptr::null_mut(), 10) as DWORD;
    }

    if !hKey.is_null() {
        RegCloseKey(hKey);
    }

    return dwReturnValue;
}
#[no_mangle]
pub unsafe extern "C" fn GetPortNameWithoutColon(pwszPortName: PCWSTR, ppwszPortNameWithoutColon: *mut PWSTR) -> DWORD {
    let mut cchPortNameWithoutColon: DWORD;

    // Compute the string length of pwszPortNameWithoutColon.
    cchPortNameWithoutColon = (wcslen(pwszPortName) as DWORD) - (1 as DWORD);

    // Check if pwszPortName really has a colon as the last character.
    if *pwszPortName.offset(cchPortNameWithoutColon as isize) != 0x003Au16 {
        return ERROR_INVALID_PARAMETER as DWORD;
    }

    // Allocate the output buffer.
    *ppwszPortNameWithoutColon = DllAllocSplMem(
        (cchPortNameWithoutColon + (1 as DWORD))
            * (core::mem::size_of::<u16>() as DWORD),
    ) as PWSTR;
    if (*ppwszPortNameWithoutColon).is_null() {
        return ERROR_NOT_ENOUGH_MEMORY as DWORD;
    }

    // Copy the port name without colon into the buffer.
    // The buffer is already zero-initialized, so no additional null-termination is necessary.
    CopyMemory!(
        *ppwszPortNameWithoutColon as *mut core::ffi::c_void,
        pwszPortName as *const core::ffi::c_void,
        (cchPortNameWithoutColon as usize) * core::mem::size_of::<u16>(),
    );

    return ERROR_SUCCESS as DWORD;
}

unsafe extern "C" fn _IsNEPort(pwszPortName: PCWSTR) -> BOOL {
    let mut p: PCWSTR = pwszPortName;

    // First character needs to be 'N' (uppercase or lowercase)
    if *p != 0x004Eu16 && *p != 0x006Eu16 {
        return FALSE as BOOL;
    }

    // Next character needs to be 'E' (uppercase or lowercase)
    p = p.add(1);
    if *p != 0x0045u16 && *p != 0x0065u16 {
        return FALSE as BOOL;
    }

    // An optional hyphen may follow now.
    p = p.add(1);
    if *p == 0x002Du16 {
        p = p.add(1);
    }

    // Now an arbitrary number of digits may follow.
    while *p >= 0x0030u16 && *p <= 0x0039u16 {
        p = p.add(1);
    }

    // Finally, the virtual Ne port must be terminated by a colon.
    if *p != ':' as u16 {
        return FALSE as BOOL;
    }

    // If this is the end of the string, we have a virtual Ne port.
    p = p.add(1);
    return (*p == 0x0000u16) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn GetTypeFromName(name: LPCWSTR) -> DWORD {
    let mut hfile: HANDLE = core::ptr::null_mut();

    if (wcsncmp(name, RUSTCONV_WSTR_C372F2CCF22D.as_ptr(), (ARRAYSIZE!(RUSTCONV_WSTR_C372F2CCF22D) - 1) as _)) == 0 {
        return PORT_IS_LPT;
    }

    if (wcsncmp(name, RUSTCONV_WSTR_3DB566BC3DA4.as_ptr(), (ARRAYSIZE!(RUSTCONV_WSTR_3DB566BC3DA4) - 1) as _)) == 0 {
        return PORT_IS_COM;
    }

    if (lstrcmpW(name, RUSTCONV_WSTR_0FA88D88CEB2.as_ptr())) == 0 {
        return PORT_IS_FILE;
    }

    // if (name[0] == '/') return PORT_IS_UNIXNAME;
    // if (name[0] == '|') return PORT_IS_PIPE;

    if _IsNEPort(name) != 0 {
        return PORT_IS_VNET;
    }

    if (wcsncmp(name, RUSTCONV_WSTR_20E46296C1D4.as_ptr(), (ARRAYSIZE!(RUSTCONV_WSTR_20E46296C1D4) - 1) as _)) == 0 {
        return PORT_IS_XPS;
    }

    /* Must be a file or a directory. Does the file exist ? */
    hfile = CreateFileW(name, GENERIC_WRITE, 0, core::ptr::null_mut(), OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL, core::ptr::null_mut());

    if hfile == INVALID_HANDLE_VALUE {
        /* Can we create the file? */
        hfile = CreateFileW(name, GENERIC_WRITE, 0, core::ptr::null_mut(), OPEN_ALWAYS, FILE_FLAG_DELETE_ON_CLOSE, core::ptr::null_mut());
    }

    if hfile != INVALID_HANDLE_VALUE {
        CloseHandle(hfile);
        return PORT_IS_FILENAME;
    }

    /* We can't use the name. use GetLastError() for the reason */
    return PORT_IS_UNKNOWN;
}
