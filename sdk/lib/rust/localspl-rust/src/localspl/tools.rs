use crate::prelude::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn AllocAndRegQueryWSZ(hKey: HKEY, pwszValueName: PCWSTR) -> PWSTR {
    let mut cbNeeded: DWORD = 0;
    let mut lStatus: LONG;
    let mut pwszValue: PWSTR;

    // Determine the size of the required buffer.
    lStatus = RegQueryValueExW(
        hKey,
        pwszValueName as LPCWSTR,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(cbNeeded),
    );
    if lStatus != ERROR_SUCCESS as LONG {
        return core::ptr::null_mut();
    }

    // Allocate it.
    pwszValue = DllAllocSplMem(cbNeeded) as PWSTR;
    if pwszValue.is_null() {
        return core::ptr::null_mut();
    }

    // Now get the actual value.
    lStatus = RegQueryValueExW(
        hKey,
        pwszValueName as LPCWSTR,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        pwszValue as PBYTE,
        core::ptr::addr_of_mut!(cbNeeded),
    );
    if lStatus != ERROR_SUCCESS as LONG {
        DllFreeSplMem(pwszValue as PVOID);
        return core::ptr::null_mut();
    }

    return pwszValue;
}
#[no_mangle]
pub unsafe extern "C" fn DuplicateDevMode(pInput: PDEVMODEW) -> PDEVMODEW {
    let mut pOutput: PDEVMODEW;

    // Allocate a buffer for this DevMode.
    pOutput = DllAllocSplMem(((*pInput).dmSize as DWORD).wrapping_add((*pInput).dmDriverExtra as DWORD)) as PDEVMODEW;
    if pOutput.is_null() {
        return core::ptr::null_mut();
    }

    // Copy it.
    CopyMemory!(
        pOutput as *mut core::ffi::c_void,
        pInput as *const core::ffi::c_void,
        ((*pInput).dmSize as usize) + ((*pInput).dmDriverExtra as usize),
    );

    return pOutput;
}
#[no_mangle]
pub unsafe extern "C" fn copy_servername_from_name(mut name: LPCWSTR, mut target: LPWSTR) -> LONG {
    let mut server: LPCWSTR;
    let mut ptr: LPWSTR;
    let mut buffer: [WCHAR; MAX_COMPUTERNAME_LENGTH as usize + 1] = [0; MAX_COMPUTERNAME_LENGTH as usize + 1];
    let mut len: DWORD;
    let mut serverlen: DWORD;

    if !target.is_null() {
        *target = '\0' as WCHAR;
    }

    if name.is_null() {
        return 0 as LONG;
    }
    if (*name.offset(0 as isize) != '\\' as WCHAR)
        || (*name.offset(1 as isize) != '\\' as WCHAR)
    {
        return 0 as LONG;
    }

    server = name.offset(2 as isize);
    /* skip over both backslash, find separator '\' */
    ptr = wcschr(server, '\\' as WCHAR) as LPWSTR;
    serverlen = if !ptr.is_null() {
        ptr.offset_from(server) as DWORD
    } else {
        lstrlenW(server) as DWORD
    };

    /* servername is empty */
    if serverlen == 0 as DWORD {
        return 0 as LONG;
    }

    if serverlen > MAX_COMPUTERNAME_LENGTH as DWORD {
        return -(serverlen as LONG);
    }

    if !target.is_null() {
        memcpy(
            target as *mut core::ffi::c_void,
            server as *const core::ffi::c_void,
            (serverlen as usize) * core::mem::size_of::<WCHAR>(),
        );
        *target.offset(serverlen as isize) = '\0' as WCHAR;
    }

    len = ARRAYSIZE!(buffer);
    if GetComputerNameW(
        core::ptr::addr_of_mut!(buffer) as LPWSTR,
        core::ptr::addr_of_mut!(len),
    ) != 0
    {
        if (serverlen == len)
            && (_wcsnicmp(
                server,
                core::ptr::addr_of_mut!(buffer) as LPWSTR,
                len as usize,
            ) == 0 as i32)
        {
            /* The requested Servername is our computername */
            return 0 as LONG;
        }
    }
    return serverlen as LONG;
}
