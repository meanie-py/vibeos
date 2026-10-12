// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_10E6F6E93A7E: [u16; 13] = [
    0x0053u16, 0x0059, 0x0053, 0x0054, 0x0045, 0x004D, 0x005C, 0x0053, 0x0065, 0x0074,
    0x0075, 0x0070, 0x0000,
];
pub static RUSTCONV_WSTR_903D2CFF46FA: [u16; 23] = [
    0x0053u16, 0x0065, 0x0049, 0x006D, 0x0070, 0x0065, 0x0072, 0x0073, 0x006F, 0x006E,
    0x0061, 0x0074, 0x0065, 0x0050, 0x0072, 0x0069, 0x0076, 0x0069, 0x006C, 0x0065,
    0x0067, 0x0065, 0x0000,
];
pub static RUSTCONV_WSTR_96A296D224F2: [u16; 1] = [0x0000u16];
pub static RUSTCONV_WSTR_B68957AF6AFC: [u16; 22] = [
    0x0053u16, 0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x0053, 0x0065, 0x0074, 0x0075,
    0x0070, 0x0049, 0x006E, 0x0050, 0x0072, 0x006F, 0x0067, 0x0072, 0x0065, 0x0073,
    0x0073, 0x0000,
];
pub static RUSTCONV_WSTR_BA238EA09BC7: [u16; 28] = [
    0x004Eu16, 0x0054, 0x0020, 0x0041, 0x0055, 0x0054, 0x0048, 0x004F, 0x0052, 0x0049,
    0x0054, 0x0059, 0x005C, 0x004E, 0x0065, 0x0074, 0x0077, 0x006F, 0x0072, 0x006B,
    0x0053, 0x0065, 0x0072, 0x0076, 0x0069, 0x0063, 0x0065, 0x0000,
];
pub static RUSTCONV_WSTR_F54B9E4D358B: [u16; 6] = [0x0052u16, 0x0050, 0x0043, 0x0053, 0x0053, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)

unsafe extern "C" fn SetupIsActive() -> BOOL {
    let mut lResult: LONG;
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut dwData: DWORD = 0;
    let mut cbData: DWORD = core::mem::size_of::<DWORD>() as DWORD;
    let mut dwType: DWORD = REG_NONE;

    lResult = RegOpenKeyExW(HKEY_LOCAL_MACHINE, RUSTCONV_WSTR_10E6F6E93A7E.as_ptr(), 0, KEY_QUERY_VALUE, core::ptr::addr_of_mut!(hKey)) as LONG;
    if lResult != ERROR_SUCCESS as LONG {
        return (FALSE) as BOOL;
    }

    lResult = RegQueryValueExW(hKey, RUSTCONV_WSTR_B68957AF6AFC.as_ptr(), core::ptr::null_mut(),
                               core::ptr::addr_of_mut!(dwType), core::ptr::addr_of_mut!(dwData) as LPBYTE, core::ptr::addr_of_mut!(cbData)) as LONG;
    RegCloseKey(hKey);

    if (lResult == ERROR_SUCCESS as LONG) && (dwType == REG_DWORD as DWORD) &&
        (cbData == core::mem::size_of::<DWORD>() as DWORD) && (dwData == 1 as DWORD)
    {
        return (TRUE) as BOOL;
    }

    return (FALSE) as BOOL;
}

unsafe extern "C" fn RunningAsSYSTEM() -> BOOL {
    /* S-1-5-18 -- Local System */
    static mut SystemSid: SID = SID {
        Revision: SID_REVISION as BYTE,
        SubAuthorityCount: 1 as BYTE,
        IdentifierAuthority: _SID_IDENTIFIER_AUTHORITY { Value: [0, 0, 0, 0, 0, 5] }, // SECURITY_NT_AUTHORITY
        SubAuthority: [SECURITY_LOCAL_SYSTEM_RID as DWORD],
    };

    let mut bRet: BOOL = FALSE as BOOL;
    let mut pTokenUser: PTOKEN_USER = core::ptr::null_mut();
    let mut hToken: HANDLE = core::ptr::null_mut();
    let mut cbTokenBuffer: DWORD = 0 as DWORD;

    /* Get the process token */
    if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY as DWORD, core::ptr::addr_of_mut!(hToken)) == 0 {
        return FALSE as BOOL;
    }

    /* Retrieve token's information */
    'quit: {
        if GetTokenInformation(hToken, TokenUser as TOKEN_INFORMATION_CLASS, core::ptr::null_mut(), 0 as DWORD, core::ptr::addr_of_mut!(cbTokenBuffer)) != 0
            || GetLastError() != ERROR_INSUFFICIENT_BUFFER as DWORD
        {
            break 'quit;
        }

        pTokenUser = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY as DWORD, cbTokenBuffer as SIZE_T) as PTOKEN_USER;
        if pTokenUser.is_null() {
            break 'quit;
        }

        if GetTokenInformation(hToken, TokenUser as TOKEN_INFORMATION_CLASS, pTokenUser as PVOID, cbTokenBuffer, core::ptr::addr_of_mut!(cbTokenBuffer)) != 0 {
            /* Compare with SYSTEM SID */
            bRet = EqualSid((*pTokenUser).User.Sid, core::ptr::addr_of_mut!(SystemSid) as PSID);
        }

        HeapFree(GetProcessHeap(), 0 as DWORD, pTokenUser as PVOID);
    }

    /* Quit: */
    CloseHandle(hToken);
    return bRet;
}

unsafe extern "C" fn RpcSsConfigureAsNetworkService() {
    let mut hSCManager: SC_HANDLE = core::ptr::null_mut();
    let mut hService: SC_HANDLE = core::ptr::null_mut();

    /* Open the service controller */
    hSCManager = OpenSCManagerW(core::ptr::null(), SERVICES_ACTIVE_DATABASEW, SC_MANAGER_CONNECT); // -DUNICODE: the W form
    if hSCManager.is_null() {
        ERR!("OpenSCManager() failed with error 0x%lx\n", GetLastError());
        return;
    }

    /* Open the RPCSS service */
    hService = OpenServiceW(hSCManager, RUSTCONV_WSTR_F54B9E4D358B.as_ptr(), SERVICE_CHANGE_CONFIG);
    if hService.is_null() {
        ERR!("OpenService(\"RPCSS\") failed with error 0x%lx\n", GetLastError());
    }
    if !hService.is_null() {
        /* Use the NetworkService account */
        if ChangeServiceConfigW(
            hService,
            SERVICE_NO_CHANGE,
            SERVICE_NO_CHANGE,
            SERVICE_NO_CHANGE,
            core::ptr::null(),
            core::ptr::null(),
            core::ptr::null_mut(),
            core::ptr::null(),
            RUSTCONV_WSTR_BA238EA09BC7.as_ptr(),
            RUSTCONV_WSTR_96A296D224F2.as_ptr(),
            core::ptr::null(),
        ) == 0
        {
            ERR!("ChangeServiceConfig(\"RPCSS\") failed with error 0x%lx\n", GetLastError());
        }

        CloseServiceHandle(hService);
    }

    CloseServiceHandle(hSCManager);
}

unsafe extern "C" fn AddImpersonatePrivilege() {
    /* S-1-5-6 -- "Service" group */
    static mut ServiceSid: SID = SID {
        Revision: (SID_REVISION as u8),
        SubAuthorityCount: 1,
        // The translation had these two swapped: S-1-5-6 is authority 5 (NT), RID 6 (Service).
        IdentifierAuthority: SID_IDENTIFIER_AUTHORITY { Value: [0, 0, 0, 0, 0, 5] }, // SECURITY_NT_AUTHORITY
        SubAuthority: [SECURITY_SERVICE_RID as u32],
    };

    let mut Status: NTSTATUS = 0;
    let mut PolicyHandle: LSA_HANDLE = core::ptr::null_mut();
    let mut ObjectAttributes: LSA_OBJECT_ATTRIBUTES = LSA_OBJECT_ATTRIBUTES {
        Length: 0,
        RootDirectory: core::ptr::null_mut(),
        ObjectName: core::ptr::null_mut(),
        Attributes: 0,
        SecurityDescriptor: core::ptr::null_mut(),
        SecurityQualityOfService: core::ptr::null_mut(),
    };
    let mut RightString: LSA_UNICODE_STRING = LSA_UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };

    ZeroMemory!(
        core::ptr::addr_of_mut!(ObjectAttributes) as *mut core::ffi::c_void,
        core::mem::size_of::<LSA_OBJECT_ATTRIBUTES>() as usize,
    );
    Status = LsaOpenPolicy(
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(ObjectAttributes) as _,
        POLICY_CREATE_ACCOUNT | POLICY_LOOKUP_NAMES,
        core::ptr::addr_of_mut!(PolicyHandle),
    );
    if !NT_SUCCESS!(Status) {
        ERR!("LsaOpenPolicy() failed with Status 0x%08lx\n", Status);
        return;
    }

    RtlInitUnicodeString(
        core::ptr::addr_of_mut!(RightString),
        RUSTCONV_WSTR_903D2CFF46FA.as_ptr(),
    );
    Status = LsaAddAccountRights(
        PolicyHandle,
        core::ptr::addr_of_mut!(ServiceSid) as PSID,
        core::ptr::addr_of_mut!(RightString),
        1,
    );
    if !NT_SUCCESS!(Status) {
        ERR!(
            "LsaAddAccountRights(\"S-1-5-6\", \"%wZ\") failed with Status 0x%08lx\n",
            Status,
            core::ptr::addr_of_mut!(RightString)
        );
    }

    LsaClose(PolicyHandle);
}
#[no_mangle]
pub unsafe extern "C" fn DoRpcSsSetupConfiguration() {
    if SetupIsActive() != 0 && RunningAsSYSTEM() != 0 {
        RpcSsConfigureAsNetworkService();
        AddImpersonatePrivilege();
    }
}
