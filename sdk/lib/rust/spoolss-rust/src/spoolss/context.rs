use crate::prelude::*;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn ImpersonatePrinterClient(hToken: HANDLE) -> BOOL {
    let mut cbReturned: DWORD = 0;
    let mut dwErrorCode: DWORD = 0;
    let mut Type: TOKEN_TYPE = 0 as TOKEN_TYPE;
    'Cleanup: {
        // Sanity check
        if hToken.is_null() {
            dwErrorCode = ERROR_INVALID_HANDLE as DWORD;
            break 'Cleanup;
        }

        // Get the type of the supplied token.
        if GetTokenInformation(
            hToken,
            TokenType,
            core::ptr::addr_of_mut!(Type) as PVOID,
            core::mem::size_of::<TOKEN_TYPE>() as DWORD,
            core::ptr::addr_of_mut!(cbReturned),
        ) == 0
        {
            dwErrorCode = GetLastError();
            break 'Cleanup;
        }

        // Check if this is an impersonation token and only set it as the thread token in this case.
        // This is not always an impersonation token, see RevertToPrinterSelf.
        if Type == TokenImpersonation {
            if SetThreadToken(core::ptr::null_mut(), hToken) == 0 {
                dwErrorCode = GetLastError();
                break 'Cleanup;
            }
        }
    }

    // Cleanup:
    if !hToken.is_null() {
        CloseHandle(hToken);
    }

    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn RevertToPrinterSelf() -> HANDLE {
    let mut dwErrorCode: DWORD;
    let mut hReturnValue: HANDLE = core::ptr::null_mut();
    let mut hToken: HANDLE = core::ptr::null_mut();

    'Cleanup: {
        if OpenThreadToken(GetCurrentThread(), TOKEN_IMPERSONATE, (TRUE) as BOOL, &mut hToken) != 0 {
            if SetThreadToken(core::ptr::null_mut(), core::ptr::null_mut()) == 0 {
                dwErrorCode = GetLastError();
                break 'Cleanup;
            }
        } else if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut hToken) == 0 {
            dwErrorCode = GetLastError();
            break 'Cleanup;
        }

        // We were successful, return a token!
        dwErrorCode = ERROR_SUCCESS;
        hReturnValue = hToken;

        // Don't let the cleanup routine close this.
        hToken = core::ptr::null_mut();
    }

    // Cleanup:
    if !hToken.is_null() {
        CloseHandle(hToken);
    }

    SetLastError(dwErrorCode);
    return hReturnValue;
}
