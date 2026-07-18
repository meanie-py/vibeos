use crate::prelude::*;
use crate::msgina_externs::*;
use crate::msgina_types::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut __wine_dbch___default: *mut __wine_debug_channel =
    &raw const __wine_dbch_msgina as *mut __wine_debug_channel;

static mut __wine_dbch_msgina: __wine_debug_channel = unsafe {
    __wine_debug_channel {
        flags: !(0 as ::core::ffi::c_int) as ::core::ffi::c_uchar,
        name: ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(
            *b"msgina\0\0\0\0\0\0\0\0\0",
        ),
    }
};
#[no_mangle]
pub unsafe extern "C" fn RegOpenLoggedOnHKCU(
    hUserToken: HANDLE,
    samDesired: REGSAM,
    phkResult: PHKEY,
) -> DWORD {
    let rc: DWORD;

    if !hUserToken.is_null() && ImpersonateLoggedOnUser(hUserToken) == 0 {
        rc = GetLastError();
        // DPRINT removed
        return rc;
    }

    rc = RegOpenCurrentUser(samDesired, phkResult);

    if !hUserToken.is_null() {
        RevertToSelf();
    }

    rc
}
#[no_mangle]
pub unsafe extern "C" fn ReadRegSzValue(
    hKey: HKEY,
    pszValue: PCWSTR,
    pValue: *mut PWSTR,
) -> DWORD {
    let mut cbData: u32 = 0;
    let mut dwType: u32 = 0;

    *pValue = core::ptr::null_mut();

    let mut rc = RegQueryValueExW(
        hKey,
        pszValue,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(dwType),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(cbData),
    );
    if rc != ERROR_SUCCESS {
        return rc;
    }
    if dwType != REG_SZ {
        return ERROR_UNSUPPORTED_TYPE;
    }

    let Value = HeapAlloc(
        GetProcessHeap(),
        0,
        ((cbData as usize + core::mem::size_of::<u16>()) as SIZE_T).try_into().unwrap(),
    ) as PWSTR;
    if Value.is_null() {
        return ERROR_NOT_ENOUGH_MEMORY;
    }

    rc = RegQueryValueExW(
        hKey,
        pszValue,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        Value as PBYTE,
        core::ptr::addr_of_mut!(cbData),
    );
    if rc != ERROR_SUCCESS {
        HeapFree(GetProcessHeap(), 0, Value as *mut core::ffi::c_void);
        return rc;
    }

    *Value.add((cbData as usize) / core::mem::size_of::<u16>()) = 0u16;
    *pValue = Value;

    ERROR_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn ReadRegDwordValue(
    hKey: HKEY,
    pszValue: *const u16,
    pValue: *mut u32,
) -> DWORD {
    let mut buffer: [u16; 12] = [0u16; 12];
    let mut cbData: u32 = core::mem::size_of::<[u16; 12]>() as u32;
    let mut dwType: u32 = 0;
    let rc = RegQueryValueExW(
        hKey,
        pszValue,
        core::ptr::null_mut(),
        &mut dwType,
        buffer.as_mut_ptr() as *mut u8,
        &mut cbData,
    );
    if rc != ERROR_SUCCESS {
        return rc;
    }

    let dwValue: u32;
    if dwType == REG_DWORD {
        if cbData != core::mem::size_of::<u32>() as u32 {
            return ERROR_INVALID_DATA;
        }
        dwValue = core::ptr::read_unaligned(buffer.as_ptr() as *const u32);
    } else if dwType == REG_SZ {
        let char_count = cbData as usize / core::mem::size_of::<u16>();
        if char_count > 0 {
            buffer[char_count - 1] = 0u16;
        } else {
            // buffer must be at least one character; treat as unsupported if empty?
            return ERROR_UNSUPPORTED_TYPE;
        }
        let mut pEnd: *mut u16 = core::ptr::null_mut();
        dwValue = wcstoul(buffer.as_ptr() as *const u16, &mut pEnd, 0);
        if unsafe { *pEnd } != 0 {
            return ERROR_UNSUPPORTED_TYPE;
        }
    } else {
        return ERROR_UNSUPPORTED_TYPE;
    }

    unsafe { *pValue = dwValue; }
    ERROR_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn TestTokenPrivilege(
    hToken: HANDLE,
    Privilege: ULONG,
) -> BOOL {
    let mut PrivilegeLuid: LUID = LUID { LowPart: Privilege, HighPart: 0 };
    let mut hNewToken: HANDLE = core::ptr::null_mut();
    let mut pTokenPriv: *mut TOKEN_PRIVILEGES = core::ptr::null_mut();
    let mut dwLength: u32 = 0;
    let mut ret: BOOL = 0;

    let mut hToken = hToken;
    if hToken.is_null() {
        ret = OpenThreadToken(GetCurrentThread(), TOKEN_QUERY, 0, &mut hNewToken);
        if ret == 0 && GetLastError() == ERROR_NO_TOKEN {
            ret = OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut hNewToken);
        }
        if ret == 0 || hNewToken.is_null() {
            return 0;
        }
        hToken = hNewToken;
    }

    'quit: loop {
        dwLength = 0;
        ret = GetTokenInformation(
            hToken,
            TokenPrivileges,
            core::ptr::null_mut(),
            0,
            &mut dwLength,
        );
        if ret == 0 && GetLastError() != ERROR_INSUFFICIENT_BUFFER {
            ret = 0;
            break 'quit;
        }

        ret = 0;
        pTokenPriv = LocalAlloc(LPTR, dwLength as SIZE_T) as *mut TOKEN_PRIVILEGES;
        if pTokenPriv.is_null() {
            break 'quit;
        }

        if GetTokenInformation(
            hToken,
            TokenPrivileges,
            pTokenPriv as *mut core::ffi::c_void,
            dwLength,
            &mut dwLength,
        ) != 0
        {
            let cPrivs = (*pTokenPriv).PrivilegeCount;
            let privileges_ptr = (*pTokenPriv).Privileges.as_ptr();
            for i in 0..cPrivs {
                if ret != 0 {
                    break;
                }
                let current = &*privileges_ptr.add(i as usize);
                let l1 = &current.Luid;
                let l2 = &PrivilegeLuid;
                if l1.LowPart == l2.LowPart && l1.HighPart == l2.HighPart {
                    ret = 1;
                }
            }
        }

        LocalFree(pTokenPriv as *mut core::ffi::c_void);
        break 'quit;
    }

    if hToken == hNewToken {
        CloseHandle(hNewToken);
    }
    ret
}
#[no_mangle]
pub unsafe extern "C" fn DuplicateString(Str: PCWSTR) -> PWSTR {
    if Str.is_null() {
        return core::ptr::null_mut();
    }
    let len = {
        let mut p = Str;
        while *p != 0 {
            p = p.add(1);
        }
        p.offset_from(Str) as usize
    };
    let cb = (len + 1) * core::mem::size_of::<u16>();
    let NewStr = LocalAlloc(LMEM_FIXED, cb as SIZE_T) as PWSTR;
    if !NewStr.is_null() {
        core::ptr::copy_nonoverlapping(Str as *const u8, NewStr as *mut u8, cb);
    }
    NewStr
}
