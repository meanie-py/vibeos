//! controlset.c — SCM control-set management (Current/Default/Failed/
//! LastKnownGood select, LKG creation + boot acceptance). Faithful translation.
//! All registry work via advapi32 (winsrv-sys); the static helpers stay private
//! Rust fns, the 3 exported entry points are cdecl `#[no_mangle]`. `swprintf`
//! of the "SYSTEM\\ControlSet%03lu" name is done by a manual formatter
//! (`write_controlset_name`) to avoid the variadic CRT ABI.

use crate::wstr;
use core::ptr::null_mut;
use winsrv_sys::winsrv::*;

use crate::types::HKEY_LOCAL_MACHINE;

/// controlset.c: `static BOOL bBootAccepted = FALSE;`
static mut bBootAccepted: BOOL = 0; // FALSE

/// Format "SYSTEM\\ControlSet%03lu" into `buf` (replaces swprintf — avoids the
/// variadic CRT ABI). Min 3-digit zero-padded decimal, NUL-terminated.
unsafe fn write_controlset_name(buf: &mut [u16; 32], n: u32) {
    const PREFIX: &[u16] = wstr!("SYSTEM\\ControlSet");
    let plen = PREFIX.len() - 1; // drop the macro's trailing NUL
    let mut i = 0usize;
    while i < plen {
        buf[i] = PREFIX[i];
        i += 1;
    }

    let mut digits = [0u8; 10];
    let mut nd = 0usize;
    let mut v = n;
    loop {
        digits[nd] = b'0' + (v % 10) as u8;
        nd += 1;
        v /= 10;
        if v == 0 {
            break;
        }
    }
    while nd < 3 {
        digits[nd] = b'0';
        nd += 1;
    }
    let mut k = nd;
    while k > 0 {
        k -= 1;
        buf[i] = digits[k] as u16;
        i += 1;
    }
    buf[i] = 0;
}

unsafe fn ScmGetControlSetValues(
    pdwCurrentControlSet: PDWORD,
    pdwDefaultControlSet: PDWORD,
    pdwFailedControlSet: PDWORD,
    pdwLastKnownGoodControlSet: PDWORD,
) -> DWORD {
    let mut hSelectKey: HKEY = null_mut();
    let mut dwType: DWORD = 0;
    let mut dwSize: DWORD;
    let mut dwError: DWORD;

    dwError = RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        wstr!("System\\Select").as_ptr(),
        0,
        KEY_READ,
        &mut hSelectKey,
    ) as DWORD;
    if dwError != ERROR_SUCCESS {
        return dwError;
    }

    dwSize = core::mem::size_of::<DWORD>() as DWORD;
    dwError = RegQueryValueExW(
        hSelectKey,
        wstr!("Current").as_ptr(),
        null_mut(),
        &mut dwType,
        pdwCurrentControlSet as *mut u8,
        &mut dwSize,
    ) as DWORD;
    if dwError != ERROR_SUCCESS {
        *pdwCurrentControlSet = 0;
    }

    dwSize = core::mem::size_of::<DWORD>() as DWORD;
    dwError = RegQueryValueExW(
        hSelectKey,
        wstr!("Default").as_ptr(),
        null_mut(),
        &mut dwType,
        pdwDefaultControlSet as *mut u8,
        &mut dwSize,
    ) as DWORD;
    if dwError != ERROR_SUCCESS {
        *pdwDefaultControlSet = 0;
    }

    dwSize = core::mem::size_of::<DWORD>() as DWORD;
    dwError = RegQueryValueExW(
        hSelectKey,
        wstr!("Failed").as_ptr(),
        null_mut(),
        &mut dwType,
        pdwFailedControlSet as *mut u8,
        &mut dwSize,
    ) as DWORD;
    if dwError != ERROR_SUCCESS {
        *pdwFailedControlSet = 0;
    }

    dwSize = core::mem::size_of::<DWORD>() as DWORD;
    dwError = RegQueryValueExW(
        hSelectKey,
        wstr!("LastKnownGood").as_ptr(),
        null_mut(),
        &mut dwType,
        pdwLastKnownGoodControlSet as *mut u8,
        &mut dwSize,
    ) as DWORD;
    if dwError != ERROR_SUCCESS {
        *pdwLastKnownGoodControlSet = 0;
    }

    RegCloseKey(hSelectKey);

    dwError
}

unsafe fn ScmSetLastKnownGoodControlSet(dwControlSet: DWORD) -> DWORD {
    let mut hSelectKey: HKEY = null_mut();
    let dwError: DWORD;

    dwError = RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        wstr!("System\\Select").as_ptr(),
        0,
        KEY_WRITE,
        &mut hSelectKey,
    ) as DWORD;
    if dwError != ERROR_SUCCESS {
        return dwError;
    }

    let dwError = RegSetValueExW(
        hSelectKey,
        wstr!("LastKnownGood").as_ptr(),
        0,
        REG_DWORD,
        &dwControlSet as *const DWORD as *const u8,
        core::mem::size_of::<DWORD>() as DWORD,
    ) as DWORD;

    RegFlushKey(hSelectKey);
    RegCloseKey(hSelectKey);

    dwError
}

unsafe fn ScmGetSetupInProgress() -> DWORD {
    let dwError: DWORD;
    let mut hKey: HKEY = null_mut();
    let mut dwType: DWORD = 0;
    let mut dwSize: DWORD;
    let mut dwSetupInProgress: DWORD = u32::MAX; // (DWORD)-1

    dwError = RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        wstr!("SYSTEM\\Setup").as_ptr(),
        0,
        KEY_QUERY_VALUE,
        &mut hKey,
    ) as DWORD;
    if dwError == ERROR_SUCCESS {
        dwSize = core::mem::size_of::<DWORD>() as DWORD;
        RegQueryValueExW(
            hKey,
            wstr!("SystemSetupInProgress").as_ptr(),
            null_mut(),
            &mut dwType,
            &mut dwSetupInProgress as *mut DWORD as *mut u8,
            &mut dwSize,
        );
        RegCloseKey(hKey);
    }

    dwSetupInProgress
}

unsafe fn ScmCopyControlSet(dwSourceControlSet: DWORD, dwDestinationControlSet: DWORD) -> DWORD {
    let mut szSourceControlSetName: [u16; 32] = [0; 32];
    let mut szDestinationControlSetName: [u16; 32] = [0; 32];
    let mut hSourceControlSetKey: HKEY = null_mut();
    let mut hDestinationControlSetKey: HKEY = null_mut();
    let mut dwDisposition: DWORD = 0;
    let mut dwError: DWORD;

    write_controlset_name(&mut szSourceControlSetName, dwSourceControlSet);
    write_controlset_name(&mut szDestinationControlSetName, dwDestinationControlSet);

    loop {
        dwError = RegOpenKeyExW(
            HKEY_LOCAL_MACHINE,
            szSourceControlSetName.as_ptr(),
            0,
            KEY_READ,
            &mut hSourceControlSetKey,
        ) as DWORD;
        if dwError != ERROR_SUCCESS {
            break;
        }

        dwError = RegCreateKeyExW(
            HKEY_LOCAL_MACHINE,
            szDestinationControlSetName.as_ptr(),
            0,
            null_mut(),
            REG_OPTION_NON_VOLATILE,
            KEY_WRITE,
            null_mut(),
            &mut hDestinationControlSetKey,
            &mut dwDisposition,
        ) as DWORD;
        if dwError != ERROR_SUCCESS {
            break;
        }

        dwError = RegCopyTreeW(hSourceControlSetKey, null_mut(), hDestinationControlSetKey) as DWORD;
        if dwError != ERROR_SUCCESS {
            break;
        }

        RegFlushKey(hDestinationControlSetKey);
        break;
    }

    // done:
    if !hDestinationControlSetKey.is_null() {
        RegCloseKey(hDestinationControlSetKey);
    }
    if !hSourceControlSetKey.is_null() {
        RegCloseKey(hSourceControlSetKey);
    }

    dwError
}

unsafe fn ScmDeleteControlSet(dwControlSet: DWORD) -> DWORD {
    let mut szControlSetName: [u16; 32] = [0; 32];
    let mut hControlSetKey: HKEY = null_mut();
    let dwError: DWORD;

    write_controlset_name(&mut szControlSetName, dwControlSet);

    dwError = RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        szControlSetName.as_ptr(),
        0,
        DELETE | KEY_ENUMERATE_SUB_KEYS | KEY_QUERY_VALUE | KEY_SET_VALUE,
        &mut hControlSetKey,
    ) as DWORD;
    if dwError != ERROR_SUCCESS {
        return dwError;
    }

    let dwError = RegDeleteTreeW(hControlSetKey, null_mut()) as DWORD;

    RegCloseKey(hControlSetKey);

    dwError
}

#[no_mangle]
pub unsafe extern "C" fn ScmCreateLastKnownGoodControlSet() -> DWORD {
    let mut dwCurrentControlSet: DWORD = 0;
    let mut dwDefaultControlSet: DWORD = 0;
    let mut dwFailedControlSet: DWORD = 0;
    let mut dwLastKnownGoodControlSet: DWORD = 0;
    let mut dwNewControlSet: DWORD;
    let mut dwError: DWORD;

    dwError = ScmGetControlSetValues(
        &mut dwCurrentControlSet,
        &mut dwDefaultControlSet,
        &mut dwFailedControlSet,
        &mut dwLastKnownGoodControlSet,
    );
    if dwError != ERROR_SUCCESS {
        return dwError;
    }

    /* First boot after setup? */
    if ScmGetSetupInProgress() == 0 && dwCurrentControlSet == dwLastKnownGoodControlSet {
        /* Search for a new control set number */
        dwNewControlSet = 1;
        while dwNewControlSet < 1000 {
            if dwNewControlSet != dwCurrentControlSet
                && dwNewControlSet != dwDefaultControlSet
                && dwNewControlSet != dwFailedControlSet
                && dwNewControlSet != dwLastKnownGoodControlSet
            {
                break;
            }
            dwNewControlSet += 1;
        }

        if dwNewControlSet >= 1000 {
            return ERROR_NO_MORE_ITEMS;
        }

        dwError = ScmCopyControlSet(dwCurrentControlSet, dwNewControlSet);
        if dwError != ERROR_SUCCESS {
            return dwError;
        }

        dwError = ScmSetLastKnownGoodControlSet(dwNewControlSet);
        if dwError == ERROR_SUCCESS {
            /* Accept the boot here to prevent creating another control set
             * when a user is about to get logged on */
            bBootAccepted = 1; // TRUE
        }
    }

    dwError
}

#[no_mangle]
pub unsafe extern "C" fn ScmAcceptBoot() -> DWORD {
    let mut dwCurrentControlSet: DWORD = 0;
    let mut dwDefaultControlSet: DWORD = 0;
    let mut dwFailedControlSet: DWORD = 0;
    let mut dwLastKnownGoodControlSet: DWORD = 0;
    let mut dwNewControlSet: DWORD;
    let mut dwError: DWORD;

    if bBootAccepted != 0 {
        return ERROR_BOOT_ALREADY_ACCEPTED;
    }

    dwError = ScmGetControlSetValues(
        &mut dwCurrentControlSet,
        &mut dwDefaultControlSet,
        &mut dwFailedControlSet,
        &mut dwLastKnownGoodControlSet,
    );
    if dwError != ERROR_SUCCESS {
        return dwError;
    }

    /* Search for a new control set number */
    dwNewControlSet = 1;
    while dwNewControlSet < 1000 {
        if dwNewControlSet != dwCurrentControlSet
            && dwNewControlSet != dwDefaultControlSet
            && dwNewControlSet != dwFailedControlSet
            && dwNewControlSet != dwLastKnownGoodControlSet
        {
            break;
        }
        dwNewControlSet += 1;
    }

    if dwNewControlSet >= 1000 {
        return ERROR_NO_MORE_ITEMS;
    }

    dwError = ScmCopyControlSet(dwCurrentControlSet, dwNewControlSet);
    if dwError != ERROR_SUCCESS {
        return dwError;
    }

    /* Delete the current LKG control set, if not used anywhere else */
    if dwLastKnownGoodControlSet != dwCurrentControlSet
        && dwLastKnownGoodControlSet != dwDefaultControlSet
        && dwLastKnownGoodControlSet != dwFailedControlSet
    {
        ScmDeleteControlSet(dwLastKnownGoodControlSet);
    }

    dwError = ScmSetLastKnownGoodControlSet(dwNewControlSet);
    if dwError != ERROR_SUCCESS {
        return dwError;
    }

    bBootAccepted = 1; // TRUE

    ERROR_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn ScmRunLastKnownGood() -> DWORD {
    if bBootAccepted != 0 {
        return ERROR_BOOT_ALREADY_ACCEPTED;
    }

    /* FIXME */

    ERROR_SUCCESS
}
