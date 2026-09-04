// rustconv-assembly: whole-tu
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const TICKS_PER_DAY: i64 = -864000000000i64;

pub const TICKS_PER_MINUTE: i64 = -600000000i64;

#[no_mangle]
pub unsafe extern "system" fn SetAccountsDomainSid(
    DomainSid: PSID,
    DomainName: LPCWSTR,
) -> NTSTATUS {
    let mut OrigInfo: PPOLICY_ACCOUNT_DOMAIN_INFO = core::ptr::null_mut();
    let mut Info: POLICY_ACCOUNT_DOMAIN_INFO = core::mem::zeroed();
    let mut ObjectAttributes: LSA_OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut PolicyHandle: LSA_HANDLE = core::mem::zeroed();
    let mut ServerHandle: SAM_HANDLE = core::ptr::null_mut();
    let mut DomainHandle: SAM_HANDLE = core::ptr::null_mut();
    let mut DomainNameInfo: DOMAIN_NAME_INFORMATION = core::mem::zeroed();
    let mut DomainNameLength: SIZE_T = 0;
    let mut Status: NTSTATUS;

    // DPRINT removed

    if DomainName != core::ptr::null() {
        DomainNameLength = wcslen(DomainName) as SIZE_T;
        if DomainNameLength > UNICODE_STRING_MAX_CHARS as SIZE_T {
            return STATUS_INVALID_PARAMETER;
        }
    }

    memset(
        core::ptr::addr_of_mut!(ObjectAttributes) as *mut core::ffi::c_void,
        0,
        core::mem::size_of::<LSA_OBJECT_ATTRIBUTES>(),
    );
    ObjectAttributes.Length = core::mem::size_of::<LSA_OBJECT_ATTRIBUTES>() as u32;

    Status = LsaOpenPolicy(
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(ObjectAttributes),
        (POLICY_VIEW_LOCAL_INFORMATION | POLICY_TRUST_ADMIN) as u32,
        core::ptr::addr_of_mut!(PolicyHandle),
    );
    if Status != STATUS_SUCCESS {
        // DPRINT removed
        return Status;
    }

    Status = LsaQueryInformationPolicy(
        PolicyHandle,
        PolicyAccountDomainInformation,
        core::ptr::addr_of_mut!(OrigInfo) as *mut PVOID,
    );
    if Status == STATUS_SUCCESS && OrigInfo != core::ptr::null_mut() {
        if DomainName == core::ptr::null() {
            Info.DomainName.Buffer = (*OrigInfo).DomainName.Buffer;
            Info.DomainName.Length = (*OrigInfo).DomainName.Length;
            Info.DomainName.MaximumLength = (*OrigInfo).DomainName.MaximumLength;
        } else {
            Info.DomainName.Buffer = DomainName as LPWSTR;
            Info.DomainName.Length = (DomainNameLength * core::mem::size_of::<WCHAR>() as SIZE_T) as u16;
            Info.DomainName.MaximumLength = Info.DomainName.Length + core::mem::size_of::<WCHAR>() as u16;
        }

        if DomainSid == core::ptr::null_mut() {
            Info.DomainSid = (*OrigInfo).DomainSid;
        } else {
            Info.DomainSid = DomainSid;
        }
    } else {
        Info.DomainName.Buffer = DomainName as LPWSTR;
        Info.DomainName.Length = (DomainNameLength * core::mem::size_of::<WCHAR>() as SIZE_T) as u16;
        Info.DomainName.MaximumLength = Info.DomainName.Length + core::mem::size_of::<WCHAR>() as u16;
        Info.DomainSid = DomainSid;
    }

    Status = LsaSetInformationPolicy(
        PolicyHandle,
        PolicyAccountDomainInformation,
        core::ptr::addr_of_mut!(Info) as PVOID,
    );
    if Status != STATUS_SUCCESS {
        // DPRINT removed
    }

    if OrigInfo != core::ptr::null_mut() {
        LsaFreeMemory(OrigInfo as PVOID);
    }

    LsaClose(PolicyHandle);

    DomainNameInfo.DomainName.Length = (DomainNameLength * core::mem::size_of::<WCHAR>() as SIZE_T) as u16;
    DomainNameInfo.DomainName.MaximumLength = DomainNameInfo.DomainName.Length + core::mem::size_of::<WCHAR>() as u16;
    DomainNameInfo.DomainName.Buffer = DomainName as LPWSTR;

    Status = SamConnect(
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(ServerHandle),
        (SAM_SERVER_CONNECT | SAM_SERVER_LOOKUP_DOMAIN) as u32,
        core::ptr::null_mut(),
    );
    if NT_SUCCESS(Status) {
        Status = SamOpenDomain(
            ServerHandle,
            (DOMAIN_WRITE_OTHER_PARAMETERS) as u32,
            Info.DomainSid,
            core::ptr::addr_of_mut!(DomainHandle),
        );
        if NT_SUCCESS(Status) {
            Status = SamSetInformationDomain(
                DomainHandle,
                DomainNameInformation,
                core::ptr::addr_of_mut!(DomainNameInfo) as PVOID,
            );
            if !NT_SUCCESS(Status) {
                // DPRINT removed
            }

            SamCloseHandle(DomainHandle);
        } else {
            // DPRINT removed
        }

        SamCloseHandle(ServerHandle);
    }

    Status
}
unsafe extern "C" fn SetPrimaryDomain(
    DomainName: LPCWSTR,
    DomainSid: PSID,
) -> NTSTATUS {
    let mut OrigInfo: PPOLICY_PRIMARY_DOMAIN_INFO = core::ptr::null_mut();
    let mut Info: POLICY_PRIMARY_DOMAIN_INFO = core::mem::zeroed();
    let mut ObjectAttributes: LSA_OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut PolicyHandle: LSA_HANDLE = core::mem::zeroed();
    let mut DomainNameLength: SIZE_T = 0;
    let mut Status: NTSTATUS;

    // DPRINT removed

    if DomainName != core::ptr::null() {
        DomainNameLength = wcslen(DomainName) as SIZE_T;
        if DomainNameLength > UNICODE_STRING_MAX_CHARS as SIZE_T {
            return STATUS_INVALID_PARAMETER;
        }
    }

    memset(
        &mut ObjectAttributes as *mut _ as *mut core::ffi::c_void,
        0,
        core::mem::size_of::<LSA_OBJECT_ATTRIBUTES>(),
    );
    ObjectAttributes.Length = core::mem::size_of::<LSA_OBJECT_ATTRIBUTES>() as u32;

    Status = LsaOpenPolicy(
        core::ptr::null_mut(),
        &mut ObjectAttributes,
        (POLICY_VIEW_LOCAL_INFORMATION | POLICY_TRUST_ADMIN) as u32,
        &mut PolicyHandle,
    );
    if Status != STATUS_SUCCESS {
        // DPRINT removed
        return Status;
    }

    Status = LsaQueryInformationPolicy(
        PolicyHandle,
        PolicyPrimaryDomainInformation,
        &mut OrigInfo as *mut _ as *mut PVOID,
    );
    if Status == STATUS_SUCCESS && OrigInfo != core::ptr::null_mut() {
        if DomainName == core::ptr::null() {
            Info.Name.Buffer = (*OrigInfo).Name.Buffer;
            Info.Name.Length = (*OrigInfo).Name.Length;
            Info.Name.MaximumLength = (*OrigInfo).Name.MaximumLength;
        } else {
            Info.Name.Buffer = DomainName as LPWSTR;
            Info.Name.Length = (DomainNameLength * core::mem::size_of::<WCHAR>() as SIZE_T) as u16;
            Info.Name.MaximumLength = Info.Name.Length + core::mem::size_of::<WCHAR>() as u16;
        }

        if DomainSid == core::ptr::null_mut() {
            Info.Sid = (*OrigInfo).Sid;
        } else {
            Info.Sid = DomainSid;
        }
    } else {
        Info.Name.Buffer = DomainName as LPWSTR;
        Info.Name.Length = (DomainNameLength * core::mem::size_of::<WCHAR>() as SIZE_T) as u16;
        Info.Name.MaximumLength = Info.Name.Length + core::mem::size_of::<WCHAR>() as u16;
        Info.Sid = DomainSid;
    }

    Status = LsaSetInformationPolicy(
        PolicyHandle,
        PolicyPrimaryDomainInformation,
        &mut Info as *mut _ as PVOID,
    );
    if Status != STATUS_SUCCESS {
        // DPRINT removed
    }

    if OrigInfo != core::ptr::null_mut() {
        LsaFreeMemory(OrigInfo as PVOID);
    }

    LsaClose(PolicyHandle);

    Status
}
unsafe extern "C" fn InstallBuiltinAccounts() {
    let mut BuiltinAccounts: [LPWSTR; 10] = [
        w!("S-1-1-0") as LPWSTR,
        w!("S-1-5-4") as LPWSTR,
        w!("S-1-5-6") as LPWSTR,
        w!("S-1-5-19") as LPWSTR,
        w!("S-1-5-20") as LPWSTR,
        w!("S-1-5-32-544") as LPWSTR,
        w!("S-1-5-32-545") as LPWSTR,
        w!("S-1-5-32-547") as LPWSTR,
        w!("S-1-5-32-551") as LPWSTR,
        w!("S-1-5-32-555") as LPWSTR,
    ];
    let mut ObjectAttributes: LSA_OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let mut PolicyHandle: LSA_HANDLE = core::ptr::null_mut();
    let mut AccountHandle: LSA_HANDLE = core::ptr::null_mut();
    let mut AccountSid: PSID = core::ptr::null_mut();
    let mut i: ULONG;

    memset(core::ptr::addr_of_mut!(ObjectAttributes) as *mut c_void, 0, core::mem::size_of::<LSA_OBJECT_ATTRIBUTES>());

    Status = LsaOpenPolicy(
        core::ptr::null_mut(),
        &mut ObjectAttributes,
        (POLICY_CREATE_ACCOUNT) as u32,
        &mut PolicyHandle,
    );
    if !NT_SUCCESS(Status) {
        return;
    }

    i = 0;
    while i < ARRAYSIZE!(BuiltinAccounts) {
        if ConvertStringSidToSid(BuiltinAccounts[i as usize], &mut AccountSid) == 0 {
            GetLastError();
            i = i.wrapping_add(1);
            continue;
        }

        Status = LsaCreateAccount(
            PolicyHandle,
            AccountSid,
            0,
            &mut AccountHandle,
        );
        if NT_SUCCESS(Status) {
            LsaClose(AccountHandle);
        }

        LocalFree(AccountSid);
        i = i.wrapping_add(1);
    }

    LsaClose(PolicyHandle);
}
unsafe extern "C" fn InstallPrivileges(hSecurityInf: HINF) {
    let mut ObjectAttributes: LSA_OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut szPrivilegeString: [u16; 256] = [0; 256];
    let mut szSidString: [u16; 256] = [0; 256];
    let mut InfContext: INFCONTEXT = core::mem::zeroed();
    let mut i: u32;
    let mut AccountSid: PSID = core::ptr::null_mut();
    let mut Status: i32;
    let mut PolicyHandle: LSA_HANDLE = core::ptr::null_mut();
    let mut RightString: LSA_UNICODE_STRING = core::mem::zeroed();
    let mut AccountName: LSA_UNICODE_STRING = core::mem::zeroed();
    let mut ReferencedDomains: PLSA_REFERENCED_DOMAIN_LIST = core::ptr::null_mut();
    let mut Sids: PLSA_TRANSLATED_SID2 = core::ptr::null_mut();

    // DPRINT removed

    memset(
        &mut ObjectAttributes as *mut _ as *mut _,
        0,
        core::mem::size_of::<LSA_OBJECT_ATTRIBUTES>(),
    );

    Status = LsaOpenPolicy(
        core::ptr::null_mut(),
        &mut ObjectAttributes,
        (POLICY_CREATE_ACCOUNT | POLICY_LOOKUP_NAMES) as u32,
        &mut PolicyHandle,
    );
    if !NT_SUCCESS(Status) {
        // DPRINT removed
        'done: loop {
            if !PolicyHandle.is_null() {
                LsaClose(PolicyHandle);
            }
            break;
        }
        return;
    }

    if SetupFindFirstLineW(
        hSecurityInf,
        w!("Privilege Rights") as *const u16,
        core::ptr::null_mut(),
        &mut InfContext,
    ) == 0
    {
        // DPRINT removed
        'done: loop {
            if !PolicyHandle.is_null() {
                LsaClose(PolicyHandle);
            }
            break;
        }
        return;
    }

    loop {
        if SetupGetStringFieldW(
            &mut InfContext,
            0,
            szPrivilegeString.as_mut_ptr(),
            ARRAYSIZE!(szPrivilegeString),
            core::ptr::null_mut(),
        ) == 0
        {
            // DPRINT removed
            'done: loop {
                if !PolicyHandle.is_null() {
                    LsaClose(PolicyHandle);
                }
                break;
            }
            return;
        }
        // DPRINT removed

        i = 0;
        while i < SetupGetFieldCount(&mut InfContext) {
            if SetupGetStringFieldW(
                &mut InfContext,
                i + 1,
                szSidString.as_mut_ptr(),
                ARRAYSIZE!(szSidString),
                core::ptr::null_mut(),
            ) == 0
            {
                // DPRINT removed
                'done: loop {
                    if !PolicyHandle.is_null() {
                        LsaClose(PolicyHandle);
                    }
                    break;
                }
                return;
            }
            // DPRINT removed

            if szSidString[0] == UNICODE_NULL {
                i += 1;
                continue;
            }

            if szSidString[0] == 0x002A {
                // DPRINT removed

                if ConvertStringSidToSid(
                    szSidString.as_ptr().add(1),
                    &mut AccountSid,
                ) == 0
                {
                    // DPRINT removed
                    i += 1;
                    continue;
                }
            } else {
                // DPRINT removed

                ReferencedDomains = core::ptr::null_mut();
                Sids = core::ptr::null_mut();
                RtlInitUnicodeString(&mut AccountName, szSidString.as_ptr() as *mut _);
                Status = LsaLookupNames2(
                    PolicyHandle,
                    0,
                    1,
                    &mut AccountName,
                    &mut ReferencedDomains,
                    &mut Sids,
                );
                if !ReferencedDomains.is_null() {
                    LsaFreeMemory((ReferencedDomains).cast());
                }

                if !NT_SUCCESS(Status) {
                    // DPRINT removed

                    if !Sids.is_null() {
                        LsaFreeMemory((Sids).cast());
                        Sids = core::ptr::null_mut();
                    }

                    i += 1;
                    continue;
                }
            }

            RtlInitUnicodeString(&mut RightString, szPrivilegeString.as_ptr() as *mut _);
            Status = LsaAddAccountRights(
                PolicyHandle,
                if !AccountSid.is_null() {
                    AccountSid
                } else {
                    (*Sids.add(0)).Sid
                },
                &mut RightString,
                1,
            );
            if !NT_SUCCESS(Status) {
                // DPRINT removed
            }

            if !Sids.is_null() {
                LsaFreeMemory((Sids).cast());
                Sids = core::ptr::null_mut();
            }

            if !AccountSid.is_null() {
                LocalFree(AccountSid);
                AccountSid = core::ptr::null_mut();
            }

            i += 1;
        }

        if SetupFindNextLine(core::ptr::addr_of_mut!(InfContext), core::ptr::addr_of_mut!(InfContext)) == 0 {
            break;
        }
    }

    if !PolicyHandle.is_null() {
        LsaClose(PolicyHandle);
    }
}
unsafe extern "C" fn ApplyRegistryValues(hSecurityInf: HINF) {
    let mut szRegistryPath: [u16; (MAX_PATH) as usize] = [0; (MAX_PATH) as usize];
    let mut szRootName: [u16; (MAX_PATH) as usize] = [0; (MAX_PATH) as usize];
    let mut szKeyName: [u16; (MAX_PATH) as usize] = [0; (MAX_PATH) as usize];
    let mut szValueName: [u16; (MAX_PATH) as usize] = [0; (MAX_PATH) as usize];
    let mut InfContext: INFCONTEXT = core::mem::zeroed();
    let mut dwLength: u32;
    let mut dwType: u32 = 0;
    let mut hRootKey: HKEY;
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut Ptr1: PWSTR;
    let mut Ptr2: PWSTR;
    let mut dwError: u32;
    let mut pBuffer: PVOID;

    if SetupFindFirstLineW(
        hSecurityInf,
        w!("Registry Values") as LPCWSTR,
        core::ptr::null_mut(),
        &mut InfContext,
    ) == 0
    {
        return;
    }

    loop {
        if SetupGetStringFieldW(
            &mut InfContext,
            0,
            szRegistryPath.as_mut_ptr(),
            ARRAYSIZE!(szRegistryPath),
            core::ptr::null_mut(),
        ) == 0
        {
            return;
        }

        Ptr1 = wcschr(szRegistryPath.as_ptr(), b'\\' as u16);
        Ptr2 = wcsrchr(szRegistryPath.as_ptr(), b'\\' as u16);
        if !Ptr1.is_null() && !Ptr2.is_null() && Ptr1 != Ptr2 {
            dwLength = ((Ptr1 as usize - szRegistryPath.as_ptr() as usize) / core::mem::size_of::<u16>()) as u32;
            wcsncpy(szRootName.as_mut_ptr(), szRegistryPath.as_ptr(), (dwLength) as usize);
            szRootName[dwLength as usize] = UNICODE_NULL;

            Ptr1 = Ptr1.offset(1);
            dwLength = ((Ptr2 as usize - Ptr1 as usize) / core::mem::size_of::<u16>()) as u32;
            wcsncpy(szKeyName.as_mut_ptr(), Ptr1, (dwLength) as usize);
            szKeyName[dwLength as usize] = UNICODE_NULL;

            Ptr2 = Ptr2.offset(1);
            wcscpy(szValueName.as_mut_ptr(), Ptr2);

            if _wcsicmp(
                szRootName.as_ptr(),
                w!("Machine") as LPCWSTR,
            ) == 0
            {
                hRootKey = HKEY_LOCAL_MACHINE;
            } else {
                break;
            }

            if SetupGetIntField(
                &mut InfContext,
                1,
                &mut dwType as *mut u32 as PINT,
            ) == 0
            {
                GetLastError();
                break;
            }

            if dwType != (REG_SZ) as u32
                && dwType != (REG_EXPAND_SZ) as u32
                && dwType != (REG_BINARY) as u32
                && dwType != (REG_DWORD) as u32
                && dwType != (REG_MULTI_SZ) as u32
            {
                break;
            }

            dwLength = 0;
            match dwType {
                REG_SZ | REG_EXPAND_SZ => {
                    SetupGetStringField(
                        &mut InfContext,
                        2,
                        core::ptr::null_mut(),
                        0,
                        &mut dwLength,
                    );
                    dwLength *= core::mem::size_of::<u16>() as u32;
                }
                REG_BINARY => {
                    SetupGetBinaryField(
                        &mut InfContext,
                        2,
                        core::ptr::null_mut(),
                        0,
                        &mut dwLength,
                    );
                }
                REG_DWORD => {
                    dwLength = core::mem::size_of::<i32>() as u32;
                }
                REG_MULTI_SZ => {
                    SetupGetMultiSzField(
                        &mut InfContext,
                        2,
                        core::ptr::null_mut(),
                        0,
                        &mut dwLength,
                    );
                    dwLength *= core::mem::size_of::<u16>() as u32;
                }
                _ => {}
            }

            if dwLength == 0 {
                break;
            }

            dwError = (RegCreateKeyExW(
                hRootKey,
                szKeyName.as_ptr(),
                0,
                core::ptr::null_mut(),
                REG_OPTION_NON_VOLATILE,
                KEY_WRITE,
                core::ptr::null_mut(),
                &mut hKey,
                core::ptr::null_mut(),
            )) as u32;
            if dwError != ERROR_SUCCESS {
                break;
            }

            pBuffer = HeapAlloc(
                GetProcessHeap(),
                0,
                (dwLength) as SIZE_T,
            );
            if !pBuffer.is_null() {
                match dwType {
                    REG_SZ | REG_EXPAND_SZ => {
                        SetupGetStringField(
                            &mut InfContext,
                            2,
                            (pBuffer).cast(),
                            dwLength / core::mem::size_of::<u16>() as u32,
                            &mut dwLength,
                        );
                        dwLength *= core::mem::size_of::<u16>() as u32;
                    }
                    REG_BINARY => {
                        SetupGetBinaryField(
                            &mut InfContext,
                            2,
                            (pBuffer).cast(),
                            dwLength,
                            &mut dwLength,
                        );
                    }
                    REG_DWORD => {
                        SetupGetIntField(
                            &mut InfContext,
                            2,
                            (pBuffer).cast(),
                        );
                    }
                    REG_MULTI_SZ => {
                        SetupGetMultiSzField(
                            &mut InfContext,
                            2,
                            (pBuffer).cast(),
                            dwLength / core::mem::size_of::<u16>() as u32,
                            &mut dwLength,
                        );
                        dwLength *= core::mem::size_of::<u16>() as u32;
                    }
                    _ => {}
                }

                RegSetValueEx(
                    hKey,
                    szValueName.as_ptr(),
                    0,
                    dwType,
                    pBuffer as LPBYTE,
                    dwLength,
                );

                HeapFree(GetProcessHeap(), 0, pBuffer);
            }

            RegCloseKey(hKey);
        }

        if SetupFindNextLine(&mut InfContext, &mut InfContext) == 0 {
            break;
        }
    }
}
unsafe extern "C" fn ApplyEventlogSettings(
    hSecurityInf: HINF,
    pszSectionName: PWSTR,
    pszLogName: PWSTR,
) {
    let mut InfContext: INFCONTEXT = core::mem::zeroed();
    let mut hServiceKey: HKEY = core::ptr::null_mut();
    let mut hLogKey: HKEY = core::ptr::null_mut();
    let mut dwValue: DWORD;
    let mut dwError: DWORD;
    let mut bValueSet: BOOL;

    // DPRINT removed

    dwError = (RegCreateKeyExW(
        HKEY_LOCAL_MACHINE,
        w!("System\\CurrentControlSet\\Services\\Eventlog") as LPCWSTR,
        0,
        core::ptr::null_mut(),
        REG_OPTION_NON_VOLATILE,
        KEY_WRITE,
        core::ptr::null_mut(),
        &mut hServiceKey,
        core::ptr::null_mut(),
    )) as u32;
    if dwError != ERROR_SUCCESS {
        // DPRINT removed
        return;
    }

    dwError = (RegCreateKeyExW(
        hServiceKey,
        pszLogName,
        0,
        core::ptr::null_mut(),
        REG_OPTION_NON_VOLATILE,
        KEY_WRITE,
        core::ptr::null_mut(),
        &mut hLogKey,
        core::ptr::null_mut(),
    )) as u32;
    if dwError != ERROR_SUCCESS {
        // DPRINT removed
        RegCloseKey(hServiceKey);
        return;
    }

    if SetupFindFirstLineW(
        hSecurityInf,
        pszSectionName,
        w!("MaximumLogSize") as LPCWSTR,
        &mut InfContext,
    ) != 0
    {
        // DPRINT removed
        dwValue = 0;
        SetupGetIntField(
            &mut InfContext,
            1,
            &mut dwValue as *mut DWORD as PINT,
        );

        // DPRINT removed
        if dwValue >= 64 && dwValue <= 4194240 {
            dwValue *= 1024;

            // DPRINT removed
            RegSetValueEx(
                hLogKey,
                w!("MaxSize") as LPCWSTR,
                0,
                (REG_DWORD) as u32,
                &dwValue as *const DWORD as LPBYTE,
                core::mem::size_of::<DWORD>() as DWORD,
            );
        }
    }

    if SetupFindFirstLineW(
        hSecurityInf,
        pszSectionName,
        w!("AuditLogRetentionPeriod") as LPCWSTR,
        &mut InfContext,
    ) != 0
    {
        bValueSet = (FALSE) as i32;
        dwValue = 0;
        SetupGetIntField(
            &mut InfContext,
            1,
            &mut dwValue as *mut DWORD as PINT,
        );
        if dwValue == 0 {
            bValueSet = (TRUE) as i32;
        } else if dwValue == 1 {
            if SetupFindFirstLineW(
                hSecurityInf,
                pszSectionName,
                w!("RetentionDays") as LPCWSTR,
                &mut InfContext,
            ) != 0
            {
                SetupGetIntField(
                    &mut InfContext,
                    1,
                    &mut dwValue as *mut DWORD as PINT,
                );
                dwValue *= 86400;
                bValueSet = (TRUE) as i32;
            }
        } else if dwValue == 2 {
            dwValue = (-1i32) as DWORD;
            bValueSet = (TRUE) as i32;
        }

        if bValueSet != 0 {
            // DPRINT removed
            RegSetValueEx(
                hLogKey,
                w!("Retention") as LPCWSTR,
                0,
                (REG_DWORD) as u32,
                &dwValue as *const DWORD as LPBYTE,
                core::mem::size_of::<DWORD>() as DWORD,
            );
        }
    }

    if SetupFindFirstLineW(
        hSecurityInf,
        pszSectionName,
        w!("RestrictGuestAccess") as LPCWSTR,
        &mut InfContext,
    ) != 0
    {
        dwValue = 0;
        SetupGetIntField(
            &mut InfContext,
            1,
            &mut dwValue as *mut DWORD as PINT,
        );
        if dwValue == 0 || dwValue == 1 {
            // DPRINT removed
            RegSetValueEx(
                hLogKey,
                w!("RestrictGuestAccess") as LPCWSTR,
                0,
                (REG_DWORD) as u32,
                &dwValue as *const DWORD as LPBYTE,
                core::mem::size_of::<DWORD>() as DWORD,
            );
        }
    }

    RegCloseKey(hLogKey);
    RegCloseKey(hServiceKey);
}
unsafe extern "C" fn ApplyPasswordSettings(
    hSecurityInf: HINF,
    pszSectionName: PWSTR,
) {
    let mut InfContext: INFCONTEXT = core::mem::zeroed();
    let mut PasswordInfo: PDOMAIN_PASSWORD_INFORMATION = core::ptr::null_mut();
    let mut OrigInfo: PPOLICY_ACCOUNT_DOMAIN_INFO = core::ptr::null_mut();
    let mut ObjectAttributes: LSA_OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut PolicyHandle: LSA_HANDLE = core::ptr::null_mut();
    let mut ServerHandle: SAM_HANDLE = core::ptr::null_mut();
    let mut DomainHandle: SAM_HANDLE = core::ptr::null_mut();
    let mut nValue: INT = 0;
    let mut Status: NTSTATUS;

    // DPRINT removed

    memset(&mut ObjectAttributes as *mut _ as *mut core::ffi::c_void, 0, core::mem::size_of::<LSA_OBJECT_ATTRIBUTES>());
    ObjectAttributes.Length = core::mem::size_of::<LSA_OBJECT_ATTRIBUTES>() as u32;

    Status = LsaOpenPolicy(
        core::ptr::null_mut(),
        &mut ObjectAttributes,
        (POLICY_VIEW_LOCAL_INFORMATION | POLICY_TRUST_ADMIN) as u32,
        &mut PolicyHandle,
    );
    if Status != STATUS_SUCCESS {
        // DPRINT removed
        return;
    }

    Status = LsaQueryInformationPolicy(
        PolicyHandle,
        PolicyAccountDomainInformation,
        &mut OrigInfo as *mut _ as *mut PVOID,
    );
    if !NT_SUCCESS(Status) {
        // DPRINT removed
        'done: loop {
            if !PasswordInfo.is_null() {
                SamFreeMemory(PasswordInfo as PVOID);
            }
            if !DomainHandle.is_null() {
                SamCloseHandle(DomainHandle);
            }
            if !ServerHandle.is_null() {
                SamCloseHandle(ServerHandle);
            }
            if !OrigInfo.is_null() {
                LsaFreeMemory(OrigInfo as PVOID);
            }
            if !PolicyHandle.is_null() {
                LsaClose(PolicyHandle);
            }
            break;
        }
        return;
    }

    Status = SamConnect(
        core::ptr::null_mut(),
        &mut ServerHandle,
        (SAM_SERVER_CONNECT | SAM_SERVER_LOOKUP_DOMAIN) as u32,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        // DPRINT removed
        'done: loop {
            if !PasswordInfo.is_null() {
                SamFreeMemory(PasswordInfo as PVOID);
            }
            if !DomainHandle.is_null() {
                SamCloseHandle(DomainHandle);
            }
            if !ServerHandle.is_null() {
                SamCloseHandle(ServerHandle);
            }
            if !OrigInfo.is_null() {
                LsaFreeMemory(OrigInfo as PVOID);
            }
            if !PolicyHandle.is_null() {
                LsaClose(PolicyHandle);
            }
            break;
        }
        return;
    }

    Status = SamOpenDomain(
        ServerHandle,
        (DOMAIN_READ_PASSWORD_PARAMETERS | DOMAIN_WRITE_PASSWORD_PARAMS) as u32,
        (*OrigInfo).DomainSid,
        &mut DomainHandle,
    );
    if !NT_SUCCESS(Status) {
        // DPRINT removed
        'done: loop {
            if !PasswordInfo.is_null() {
                SamFreeMemory(PasswordInfo as PVOID);
            }
            if !DomainHandle.is_null() {
                SamCloseHandle(DomainHandle);
            }
            if !ServerHandle.is_null() {
                SamCloseHandle(ServerHandle);
            }
            if !OrigInfo.is_null() {
                LsaFreeMemory(OrigInfo as PVOID);
            }
            if !PolicyHandle.is_null() {
                LsaClose(PolicyHandle);
            }
            break;
        }
        return;
    }

    Status = SamQueryInformationDomain(
        DomainHandle,
        DomainPasswordInformation,
        &mut PasswordInfo as *mut _ as *mut PVOID,
    );
    if !NT_SUCCESS(Status) {
        // DPRINT removed
        'done: loop {
            if !PasswordInfo.is_null() {
                SamFreeMemory(PasswordInfo as PVOID);
            }
            if !DomainHandle.is_null() {
                SamCloseHandle(DomainHandle);
            }
            if !ServerHandle.is_null() {
                SamCloseHandle(ServerHandle);
            }
            if !OrigInfo.is_null() {
                LsaFreeMemory(OrigInfo as PVOID);
            }
            if !PolicyHandle.is_null() {
                LsaClose(PolicyHandle);
            }
            break;
        }
        return;
    }

    // DPRINT removed
    if SetupFindFirstLineW(
        hSecurityInf,
        pszSectionName,
        w!("MaximumPasswordAge") as PWSTR,
        &mut InfContext,
    ) != 0 {
        if SetupGetIntField(&mut InfContext, 1, &mut nValue) != 0 {
            // DPRINT removed
            if nValue == -1 {
                (*PasswordInfo).MaxPasswordAge.QuadPart = 0x8000000000000000u64 as i64;
            } else if (nValue >= 1) && (nValue < 1000) {
                (*PasswordInfo).MaxPasswordAge.QuadPart = (nValue as LONGLONG) * TICKS_PER_DAY;
            }
            // DPRINT removed
        }
    }

    // DPRINT removed
    if SetupFindFirstLineW(
        hSecurityInf,
        pszSectionName,
        w!("MinimumPasswordAge") as PWSTR,
        &mut InfContext,
    ) != 0 {
        if SetupGetIntField(&mut InfContext, 1, &mut nValue) != 0 {
            // DPRINT removed
            if (nValue >= 0) && (nValue < 1000) {
                if (*PasswordInfo).MaxPasswordAge.QuadPart < (nValue as LONGLONG) * TICKS_PER_DAY {
                    (*PasswordInfo).MinPasswordAge.QuadPart = (nValue as LONGLONG) * TICKS_PER_DAY;
                }
            }
            // DPRINT removed
        }
    }

    // DPRINT removed
    if SetupFindFirstLineW(
        hSecurityInf,
        pszSectionName,
        w!("MinimumPasswordLength") as PWSTR,
        &mut InfContext,
    ) != 0 {
        if SetupGetIntField(&mut InfContext, 1, &mut nValue) != 0 {
            // DPRINT removed
            if (nValue >= 0) && (nValue <= 65535) {
                (*PasswordInfo).MinPasswordLength = nValue as u16;
            }
            // DPRINT removed
        }
    }

    // DPRINT removed
    if SetupFindFirstLineW(
        hSecurityInf,
        pszSectionName,
        w!("PasswordHistorySize") as PWSTR,
        &mut InfContext,
    ) != 0 {
        if SetupGetIntField(&mut InfContext, 1, &mut nValue) != 0 {
            // DPRINT removed
            if (nValue >= 0) && (nValue <= 65535) {
                (*PasswordInfo).PasswordHistoryLength = nValue as u16;
            }
            // DPRINT removed
        }
    }

    if SetupFindFirstLineW(
        hSecurityInf,
        pszSectionName,
        w!("PasswordComplexity") as PWSTR,
        &mut InfContext,
    ) != 0 {
        if SetupGetIntField(&mut InfContext, 1, &mut nValue) != 0 {
            if nValue == 0 {
                (*PasswordInfo).PasswordProperties &= (!DOMAIN_PASSWORD_COMPLEX) as u32;
            } else {
                (*PasswordInfo).PasswordProperties |= (DOMAIN_PASSWORD_COMPLEX) as u32;
            }
        }
    }

    if SetupFindFirstLineW(
        hSecurityInf,
        pszSectionName,
        w!("ClearTextPassword") as PWSTR,
        &mut InfContext,
    ) != 0 {
        if SetupGetIntField(&mut InfContext, 1, &mut nValue) != 0 {
            if nValue == 0 {
                (*PasswordInfo).PasswordProperties &= (!DOMAIN_PASSWORD_STORE_CLEARTEXT) as u32;
            } else {
                (*PasswordInfo).PasswordProperties |= (DOMAIN_PASSWORD_STORE_CLEARTEXT) as u32;
            }
        }
    }

    Status = SamSetInformationDomain(
        DomainHandle,
        DomainPasswordInformation,
        PasswordInfo as PVOID,
    );
    if !NT_SUCCESS(Status) {
        // DPRINT removed
    }

    if !PasswordInfo.is_null() {
        SamFreeMemory(PasswordInfo as PVOID);
    }
    if !DomainHandle.is_null() {
        SamCloseHandle(DomainHandle);
    }
    if !ServerHandle.is_null() {
        SamCloseHandle(ServerHandle);
    }
    if !OrigInfo.is_null() {
        LsaFreeMemory(OrigInfo as PVOID);
    }
    if !PolicyHandle.is_null() {
        LsaClose(PolicyHandle);
    }
}
unsafe extern "C" fn ApplyLockoutSettings(
    hSecurityInf: HINF,
    pszSectionName: PWSTR) {
    let mut InfContext: INFCONTEXT = core::mem::zeroed();
    let mut LockoutInfo: PDOMAIN_LOCKOUT_INFORMATION = core::ptr::null_mut();
    let mut OrigInfo: PPOLICY_ACCOUNT_DOMAIN_INFO = core::ptr::null_mut();
    let mut ObjectAttributes: LSA_OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut PolicyHandle: LSA_HANDLE = core::ptr::null_mut();
    let mut ServerHandle: SAM_HANDLE = core::ptr::null_mut();
    let mut DomainHandle: SAM_HANDLE = core::ptr::null_mut();
    let mut nValue: INT = 0;
    let mut Status: NTSTATUS;

    // DPRINT removed

    memset(&mut ObjectAttributes as *mut _ as *mut core::ffi::c_void, 0, core::mem::size_of::<LSA_OBJECT_ATTRIBUTES>());
    ObjectAttributes.Length = core::mem::size_of::<LSA_OBJECT_ATTRIBUTES>() as u32;

    Status = LsaOpenPolicy(core::ptr::null_mut(),
                           &mut ObjectAttributes,
                           (POLICY_VIEW_LOCAL_INFORMATION | POLICY_TRUST_ADMIN) as u32,
                           &mut PolicyHandle);
    if Status != STATUS_SUCCESS
    {
        // DPRINT removed
        return;
    }

    Status = LsaQueryInformationPolicy(PolicyHandle,
                                       PolicyAccountDomainInformation,
                                       &mut OrigInfo as *mut _ as *mut PVOID);
    if !NT_SUCCESS(Status)
    {
        // DPRINT removed
        'done: loop {

    Status = SamConnect(core::ptr::null_mut(),
                        &mut ServerHandle,
                        (SAM_SERVER_CONNECT | SAM_SERVER_LOOKUP_DOMAIN) as u32,
                        core::ptr::null_mut());
    if !NT_SUCCESS(Status)
    {
        // DPRINT removed
        break 'done;
    }

    Status = SamOpenDomain(ServerHandle,
                           (DOMAIN_READ_PASSWORD_PARAMETERS | DOMAIN_WRITE_PASSWORD_PARAMS) as u32,
                           (*OrigInfo).DomainSid,
                           &mut DomainHandle);
    if !NT_SUCCESS(Status)
    {
        // DPRINT removed
        break 'done;
    }

    Status = SamQueryInformationDomain(DomainHandle,
                                       DomainLockoutInformation,
                                       &mut LockoutInfo as *mut _ as *mut PVOID);
    if !NT_SUCCESS(Status)
    {
        // DPRINT removed
        break 'done;
    }

    if SetupFindFirstLineW(hSecurityInf,
                            pszSectionName,
                            w!("LockoutBadCount") as *const u16,
                            &mut InfContext) != 0
    {
        if SetupGetIntField(&mut InfContext, 1, &mut nValue) != 0
        {
            if nValue >= 0
            {
                (*LockoutInfo).LockoutThreshold = nValue as u16;
            }
        }
    }

    if SetupFindFirstLineW(hSecurityInf,
                            pszSectionName,
                            w!("ResetLockoutCount") as *const u16,
                            &mut InfContext) != 0
    {
        if SetupGetIntField(&mut InfContext, 1, &mut nValue) != 0
        {
            if nValue >= 0
            {
                (*LockoutInfo).LockoutObservationWindow.QuadPart = (nValue as i64) * TICKS_PER_MINUTE;
            }
        }
    }

    if SetupFindFirstLineW(hSecurityInf,
                            pszSectionName,
                            w!("LockoutDuration") as *const u16,
                            &mut InfContext) != 0
    {
        if SetupGetIntField(&mut InfContext, 1, &mut nValue) != 0
        {
            if nValue == -1
            {
                (*LockoutInfo).LockoutDuration.QuadPart = 0x8000000000000000u64 as i64;
            }
            else if (nValue >= 0) && (nValue < 100000)
            {
                (*LockoutInfo).LockoutDuration.QuadPart = (nValue as i64) * TICKS_PER_MINUTE;
            }
        }
    }

    Status = SamSetInformationDomain(DomainHandle,
                                     DomainLockoutInformation,
                                     LockoutInfo as PVOID);
    if !NT_SUCCESS(Status)
    {
        // DPRINT removed
        break 'done;
    }

    break 'done;
    }
    }

    if !LockoutInfo.is_null() {
        SamFreeMemory(LockoutInfo as PVOID);
    }

    if !DomainHandle.is_null() {
        SamCloseHandle(DomainHandle);
    }

    if !ServerHandle.is_null() {
        SamCloseHandle(ServerHandle);
    }

    if !OrigInfo.is_null() {
        LsaFreeMemory(OrigInfo as PVOID);
    }

    if !PolicyHandle.is_null() {
        LsaClose(PolicyHandle);
    }
}

unsafe extern "C" fn SetLsaAnonymousNameLookup(hSecurityInf: HINF, pszSectionName: PWSTR) {}
unsafe extern "C" fn EnableAccount(
    hSecurityInf: HINF,
    pszSectionName: PWSTR,
    pszValueName: PWSTR,
    DomainHandle: SAM_HANDLE,
    dwAccountRid: DWORD,
) {
    let mut InfContext: INFCONTEXT = core::mem::zeroed();
    let mut UserHandle: SAM_HANDLE = core::ptr::null_mut();
    let mut ControlInfo: PUSER_CONTROL_INFORMATION = core::ptr::null_mut();
    let mut nValue: INT = 0;
    let mut Status: NTSTATUS;

    // DPRINT removed

    if SetupFindFirstLineW(
        hSecurityInf,
        pszSectionName,
        pszValueName,
        &mut InfContext,
    ) == 0
    {
        return;
    }

    if SetupGetIntField(&mut InfContext, 1, &mut nValue) == 0 {
        // DPRINT removed
        'done: loop {
            if !ControlInfo.is_null() {
                SamFreeMemory((ControlInfo).cast());
            }

            if !UserHandle.is_null() {
                SamCloseHandle(UserHandle);
            }
            break;
        }
        return;
    }

    // DPRINT removed

    Status = SamOpenUser(
        DomainHandle,
        (USER_READ_ACCOUNT | USER_WRITE_ACCOUNT) as u32,
        dwAccountRid,
        &mut UserHandle,
    );
    if !NT_SUCCESS(Status) {
        // DPRINT removed
        'done: loop {
            if !ControlInfo.is_null() {
                SamFreeMemory((ControlInfo).cast());
            }

            if !UserHandle.is_null() {
                SamCloseHandle(UserHandle);
            }
            break;
        }
        return;
    }

    Status = SamQueryInformationUser(
        UserHandle,
        UserControlInformation,
        &mut ControlInfo as *mut _ as *mut PVOID,
    );
    if !NT_SUCCESS(Status) {
        // DPRINT removed
        'done: loop {
            if !ControlInfo.is_null() {
                SamFreeMemory((ControlInfo).cast());
            }

            if !UserHandle.is_null() {
                SamCloseHandle(UserHandle);
            }
            break;
        }
        return;
    }

    if nValue == 0 {
        (*ControlInfo).UserAccountControl |= (USER_ACCOUNT_DISABLED) as u32;
    } else {
        (*ControlInfo).UserAccountControl &= (!USER_ACCOUNT_DISABLED) as u32;
    }

    Status = SamSetInformationUser(
        UserHandle,
        UserControlInformation,
        ControlInfo as PVOID,
    );
    if !NT_SUCCESS(Status) {
        // DPRINT removed
    }

    if !ControlInfo.is_null() {
        SamFreeMemory((ControlInfo).cast());
    }

    if !UserHandle.is_null() {
        SamCloseHandle(UserHandle);
    }
}
unsafe extern "C" fn SetNewAccountName(
    hSecurityInf: HINF,
    pszSectionName: PWSTR,
    pszValueName: PWSTR,
    DomainHandle: SAM_HANDLE,
    dwAccountRid: DWORD,
) {
    let mut InfContext: INFCONTEXT = core::mem::zeroed();
    let mut dwLength: DWORD = 0;
    let mut pszName: PWSTR = core::ptr::null_mut();
    let mut UserHandle: SAM_HANDLE = core::ptr::null_mut();
    let mut NameInfo: USER_NAME_INFORMATION = core::mem::zeroed();
    let Status: NTSTATUS;

    // DPRINT removed

    if SetupFindFirstLineW(
        hSecurityInf,
        pszSectionName,
        pszValueName,
        &mut InfContext,
    ) == 0
    {
        return;
    }

    SetupGetStringFieldW(
        &mut InfContext,
        1,
        core::ptr::null_mut(),
        0,
        &mut dwLength,
    );
    if dwLength == 0 {
        return;
    }

    pszName = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        (dwLength * core::mem::size_of::<WCHAR>() as u32) as SIZE_T,
    ) as PWSTR;
    if pszName.is_null() {
        // DPRINT removed
        return;
    }

    'done: loop {
        if SetupGetStringFieldW(
            &mut InfContext,
            1,
            pszName,
            dwLength,
            &mut dwLength,
        ) == 0
        {
            // DPRINT removed
            break 'done;
        }

        // DPRINT removed

        let status_open = SamOpenUser(
            DomainHandle,
            (USER_WRITE_ACCOUNT) as u32,
            dwAccountRid,
            &mut UserHandle,
        );
        if !NT_SUCCESS(status_open) {
            // DPRINT removed
            break 'done;
        }

        NameInfo.UserName.Length = (wcslen(pszName) * core::mem::size_of::<WCHAR>()) as USHORT;
        NameInfo.UserName.MaximumLength = NameInfo.UserName.Length + core::mem::size_of::<WCHAR>() as USHORT;
        NameInfo.UserName.Buffer = pszName;
        NameInfo.FullName.Length = 0;
        NameInfo.FullName.MaximumLength = 0;
        NameInfo.FullName.Buffer = core::ptr::null_mut();

        let status_set = SamSetInformationUser(
            UserHandle,
            UserNameInformation,
            &mut NameInfo as *mut _ as *mut core::ffi::c_void,
        );
        if !NT_SUCCESS(status_set) {
            // DPRINT removed
        }

        break 'done;
    }

    if !UserHandle.is_null() {
        SamCloseHandle(UserHandle);
    }

    if !pszName.is_null() {
        HeapFree(GetProcessHeap(), 0, pszName as *mut core::ffi::c_void);
    }
}
unsafe extern "C" fn ApplyAccountSettings(
    hSecurityInf: HINF,
    pszSectionName: PWSTR,
) {
    let mut OrigInfo: PPOLICY_ACCOUNT_DOMAIN_INFO = core::ptr::null_mut();
    let mut ObjectAttributes: LSA_OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut PolicyHandle: LSA_HANDLE = core::ptr::null_mut();
    let mut ServerHandle: SAM_HANDLE = core::ptr::null_mut();
    let mut DomainHandle: SAM_HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    // DPRINT removed

    memset(
        &mut ObjectAttributes as *mut _ as *mut _,
        0,
        core::mem::size_of::<LSA_OBJECT_ATTRIBUTES>(),
    );
    ObjectAttributes.Length = core::mem::size_of::<LSA_OBJECT_ATTRIBUTES>() as u32;

    Status = LsaOpenPolicy(
        core::ptr::null_mut(),
        &mut ObjectAttributes,
        (POLICY_VIEW_LOCAL_INFORMATION | POLICY_TRUST_ADMIN) as u32,
        &mut PolicyHandle,
    );
    if Status != STATUS_SUCCESS {
        // DPRINT removed
        return;
    }

    Status = LsaQueryInformationPolicy(
        PolicyHandle,
        PolicyAccountDomainInformation,
        &mut OrigInfo as *mut _ as *mut PVOID,
    );
    if !NT_SUCCESS(Status) {
        // DPRINT removed
        'done: loop {
            if !DomainHandle.is_null() {
                SamCloseHandle(DomainHandle);
            }

            if !ServerHandle.is_null() {
                SamCloseHandle(ServerHandle);
            }

            if !OrigInfo.is_null() {
                LsaFreeMemory(OrigInfo as PVOID);
            }

            if !PolicyHandle.is_null() {
                LsaClose(PolicyHandle);
            }
            break 'done;
        }
        return;
    }

    Status = SamConnect(
        core::ptr::null_mut(),
        &mut ServerHandle,
        (SAM_SERVER_CONNECT | SAM_SERVER_LOOKUP_DOMAIN) as u32,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        // DPRINT removed
        'done: loop {
            if !DomainHandle.is_null() {
                SamCloseHandle(DomainHandle);
            }

            if !ServerHandle.is_null() {
                SamCloseHandle(ServerHandle);
            }

            if !OrigInfo.is_null() {
                LsaFreeMemory(OrigInfo as PVOID);
            }

            if !PolicyHandle.is_null() {
                LsaClose(PolicyHandle);
            }
            break 'done;
        }
        return;
    }

    Status = SamOpenDomain(
        ServerHandle,
        (DOMAIN_LOOKUP) as u32,
        (*OrigInfo).DomainSid,
        &mut DomainHandle,
    );
    if !NT_SUCCESS(Status) {
        // DPRINT removed
        'done: loop {
            if !DomainHandle.is_null() {
                SamCloseHandle(DomainHandle);
            }

            if !ServerHandle.is_null() {
                SamCloseHandle(ServerHandle);
            }

            if !OrigInfo.is_null() {
                LsaFreeMemory(OrigInfo as PVOID);
            }

            if !PolicyHandle.is_null() {
                LsaClose(PolicyHandle);
            }
            break 'done;
        }
        return;
    }

    SetLsaAnonymousNameLookup(hSecurityInf, pszSectionName);

    EnableAccount(
        hSecurityInf,
        pszSectionName,
        w!("EnableAdminAccount") as PWSTR,
        DomainHandle,
        DOMAIN_USER_RID_ADMIN,
    );

    EnableAccount(
        hSecurityInf,
        pszSectionName,
        w!("EnableGuestAccount") as PWSTR,
        DomainHandle,
        DOMAIN_USER_RID_GUEST,
    );

    SetNewAccountName(
        hSecurityInf,
        pszSectionName,
        w!("NewAdministratorName") as PWSTR,
        DomainHandle,
        DOMAIN_USER_RID_ADMIN,
    );

    SetNewAccountName(
        hSecurityInf,
        pszSectionName,
        w!("NewGuestName") as PWSTR,
        DomainHandle,
        DOMAIN_USER_RID_GUEST,
    );

    if !DomainHandle.is_null() {
        SamCloseHandle(DomainHandle);
    }

    if !ServerHandle.is_null() {
        SamCloseHandle(ServerHandle);
    }

    if !OrigInfo.is_null() {
        LsaFreeMemory(OrigInfo as PVOID);
    }

    if !PolicyHandle.is_null() {
        LsaClose(PolicyHandle);
    }
}
unsafe extern "C" fn ApplyAuditEvents(
    hSecurityInf: HINF) {
    let mut ObjectAttributes: LSA_OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut InfContext: INFCONTEXT = core::mem::zeroed();
    let mut szOptionName: [u16; 256] = [0; 256];
    let mut nValue: INT = 0;
    let mut PolicyHandle: LSA_HANDLE = core::ptr::null_mut();
    let mut AuditInfo: POLICY_AUDIT_EVENTS_INFO = core::mem::zeroed();
    let mut AuditOptions: PULONG = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    // DPRINT removed

    if SetupFindFirstLineW(hSecurityInf,
                             w!("Event Audit") as *const u16,
                             core::ptr::null_mut(),
                             &mut InfContext) == 0
    {
        // DPRINT removed
        return;
    }

    ZeroMemory!(&mut ObjectAttributes as *mut _ as *mut core::ffi::c_void, core::mem::size_of::<LSA_OBJECT_ATTRIBUTES>());

    Status = LsaOpenPolicy(core::ptr::null_mut(),
                           &mut ObjectAttributes,
                           (POLICY_SET_AUDIT_REQUIREMENTS) as u32,
                           &mut PolicyHandle);
    if !NT_SUCCESS(Status)
    {
        // DPRINT removed
        return;
    }

    AuditOptions = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY,
                             ((AuditCategoryAccountLogon + 1) * (core::mem::size_of::<ULONG>()) as u32) as SIZE_T) as PULONG;
    if AuditOptions == core::ptr::null_mut()
    {
        // DPRINT removed
        'done: loop {
            if !AuditOptions.is_null() {
                HeapFree(GetProcessHeap(), 0, AuditOptions as *mut core::ffi::c_void);
            }
            if !PolicyHandle.is_null() {
                LsaClose(PolicyHandle);
            }
            break 'done;
        }
        return;
    }

    AuditInfo.AuditingMode = (TRUE) as u8;
    AuditInfo.EventAuditingOptions = AuditOptions;
    AuditInfo.MaximumAuditEventCount = AuditCategoryAccountLogon + 1;

    loop {
        if SetupGetStringFieldW(&mut InfContext,
                                  0,
                                  szOptionName.as_mut_ptr(),
                                  ARRAYSIZE!(szOptionName),
                                  core::ptr::null_mut()) == 0
        {
            // DPRINT removed
            if SetupFindNextLine(core::ptr::addr_of_mut!(InfContext), core::ptr::addr_of_mut!(InfContext)) == 0 {
                break;
            }
            continue;
        }

        // DPRINT removed

        if SetupGetIntField(&mut InfContext,
                              1,
                              &mut nValue) == 0
        {
            // DPRINT removed
            if SetupFindNextLine(core::ptr::addr_of_mut!(InfContext), core::ptr::addr_of_mut!(InfContext)) == 0 {
                break;
            }
            continue;
        }

        // DPRINT removed

        if (nValue < (POLICY_AUDIT_EVENT_UNCHANGED) as i32) || (nValue > (POLICY_AUDIT_EVENT_NONE) as i32)
        {
            // DPRINT removed
            if SetupFindNextLine(core::ptr::addr_of_mut!(InfContext), core::ptr::addr_of_mut!(InfContext)) == 0 {
                break;
            }
            continue;
        }

        if _wcsicmp(szOptionName.as_ptr(), w!("AuditSystemEvents") as *const u16) == 0
        {
            *AuditOptions.offset(AuditCategorySystem as isize) = nValue as ULONG;
        }
        else if _wcsicmp(szOptionName.as_ptr(), w!("AuditLogonEvents") as *const u16) == 0
        {
            *AuditOptions.offset(AuditCategoryLogon as isize) = nValue as ULONG;
        }
        else if _wcsicmp(szOptionName.as_ptr(), w!("AuditObjectAccess") as *const u16) == 0
        {
            *AuditOptions.offset(AuditCategoryObjectAccess as isize) = nValue as ULONG;
        }
        else if _wcsicmp(szOptionName.as_ptr(), w!("AuditPrivilegeUse") as *const u16) == 0
        {
            *AuditOptions.offset(AuditCategoryPrivilegeUse as isize) = nValue as ULONG;
        }
        else if _wcsicmp(szOptionName.as_ptr(), w!("AuditProcessTracking") as *const u16) == 0
        {
            *AuditOptions.offset(AuditCategoryDetailedTracking as isize) = nValue as ULONG;
        }
        else if _wcsicmp(szOptionName.as_ptr(), w!("AuditPolicyChange") as *const u16) == 0
        {
            *AuditOptions.offset(AuditCategoryPolicyChange as isize) = nValue as ULONG;
        }
        else if _wcsicmp(szOptionName.as_ptr(), w!("AuditAccountManage") as *const u16) == 0
        {
            *AuditOptions.offset(AuditCategoryAccountManagement as isize) = nValue as ULONG;
        }
        else if _wcsicmp(szOptionName.as_ptr(), w!("AuditDSAccess") as *const u16) == 0
        {
            *AuditOptions.offset(AuditCategoryDirectoryServiceAccess as isize) = nValue as ULONG;
        }
        else if _wcsicmp(szOptionName.as_ptr(), w!("AuditAccountLogon") as *const u16) == 0
        {
            *AuditOptions.offset(AuditCategoryAccountLogon as isize) = nValue as ULONG;
        }
        else
        {
            // DPRINT removed
        }

        if SetupFindNextLine(core::ptr::addr_of_mut!(InfContext), core::ptr::addr_of_mut!(InfContext)) == 0 {
            break;
        }
    }

    Status = LsaSetInformationPolicy(PolicyHandle,
                                     PolicyAuditEventsInformation,
                                     &mut AuditInfo as *mut _ as *mut core::ffi::c_void);
    if Status != STATUS_SUCCESS
    {
        // DPRINT removed
    }

    if !AuditOptions.is_null() {
        HeapFree(GetProcessHeap(), 0, AuditOptions as *mut core::ffi::c_void);
    }

    if !PolicyHandle.is_null() {
        LsaClose(PolicyHandle);
    }
}
#[no_mangle]
pub unsafe extern "C" fn InstallSecurity() {
    let mut hSecurityInf: HINF;
    let pszSecurityInf: PWSTR;

    pszSecurityInf = w!("defltwk.inf") as PWSTR;

    InstallBuiltinAccounts();

    hSecurityInf = SetupOpenInfFileW(
        pszSecurityInf,
        core::ptr::null_mut(),
        (INF_STYLE_WIN4) as u32,
        core::ptr::null_mut(),
    );
    if hSecurityInf != INVALID_HANDLE_VALUE {
        InstallPrivileges(hSecurityInf);
        ApplyRegistryValues(hSecurityInf);

        ApplyEventlogSettings(
            hSecurityInf,
            w!("Application Log") as PWSTR,
            w!("Application") as PWSTR,
        );
        ApplyEventlogSettings(
            hSecurityInf,
            w!("Security Log") as PWSTR,
            w!("Security") as PWSTR,
        );
        ApplyEventlogSettings(
            hSecurityInf,
            w!("System Log") as PWSTR,
            w!("System") as PWSTR,
        );

        ApplyPasswordSettings(
            hSecurityInf,
            w!("System Access") as PWSTR,
        );
        ApplyLockoutSettings(
            hSecurityInf,
            w!("System Access") as PWSTR,
        );
        ApplyAccountSettings(
            hSecurityInf,
            w!("System Access") as PWSTR,
        );

        ApplyAuditEvents(hSecurityInf);

        SetupCloseInfFile(hSecurityInf);
    }

    SetPrimaryDomain(
        w!("WORKGROUP") as LPCWSTR,
        core::ptr::null_mut(),
    );
}
#[no_mangle]
pub unsafe extern "C" fn SetAdministratorPassword(Password: LPCWSTR) -> NTSTATUS {
    let mut OrigInfo: PPOLICY_ACCOUNT_DOMAIN_INFO = core::ptr::null_mut();
    let mut AccountNameInfo: PUSER_ACCOUNT_NAME_INFORMATION = core::ptr::null_mut();
    let mut PasswordInfo: USER_SET_PASSWORD_INFORMATION = core::mem::zeroed();
    let mut ObjectAttributes: LSA_OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut PolicyHandle: LSA_HANDLE = core::ptr::null_mut();
    let mut ServerHandle: SAM_HANDLE = core::ptr::null_mut();
    let mut DomainHandle: SAM_HANDLE = core::ptr::null_mut();
    let mut UserHandle: SAM_HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    'done: {
        memset(&mut ObjectAttributes as *mut _ as *mut core::ffi::c_void, 0, core::mem::size_of::<LSA_OBJECT_ATTRIBUTES>());
        ObjectAttributes.Length = core::mem::size_of::<LSA_OBJECT_ATTRIBUTES>() as u32;

        Status = LsaOpenPolicy(
            core::ptr::null_mut(),
            &mut ObjectAttributes,
            (POLICY_VIEW_LOCAL_INFORMATION | POLICY_TRUST_ADMIN) as u32,
            &mut PolicyHandle,
        );
        if Status != STATUS_SUCCESS {
            return Status;
        }

        Status = LsaQueryInformationPolicy(
            PolicyHandle,
            PolicyAccountDomainInformation,
            &mut OrigInfo as *mut _ as *mut PVOID,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SamConnect(
            core::ptr::null_mut(),
            &mut ServerHandle,
            (SAM_SERVER_CONNECT | SAM_SERVER_LOOKUP_DOMAIN) as u32,
            core::ptr::null_mut(),
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SamOpenDomain(
            ServerHandle,
            (DOMAIN_LOOKUP) as u32,
            (*OrigInfo).DomainSid,
            &mut DomainHandle,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SamOpenUser(
            DomainHandle,
            (USER_FORCE_PASSWORD_CHANGE | USER_READ_GENERAL) as u32,
            DOMAIN_USER_RID_ADMIN,
            &mut UserHandle,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        RtlInitUnicodeString(&mut PasswordInfo.Password, Password);
        PasswordInfo.PasswordExpired = (FALSE) as u8;

        Status = SamSetInformationUser(
            UserHandle,
            UserSetPasswordInformation,
            &mut PasswordInfo as *mut _ as *mut core::ffi::c_void,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SamQueryInformationUser(
            UserHandle,
            UserAccountNameInformation,
            &mut AccountNameInfo as *mut _ as *mut PVOID,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        AdminInfo.Name = RtlAllocateHeap(
            RtlGetProcessHeap(),
            HEAP_ZERO_MEMORY,
            ((*AccountNameInfo).UserName.Length as usize + core::mem::size_of::<WCHAR>()) as SIZE_T,
        ) as LPWSTR;
        if !AdminInfo.Name.is_null() {
            RtlCopyMemory!(
                AdminInfo.Name as *mut core::ffi::c_void,
                (*AccountNameInfo).UserName.Buffer as *const core::ffi::c_void,
                (*AccountNameInfo).UserName.Length as usize,
            );
        }

        AdminInfo.Domain = RtlAllocateHeap(
            RtlGetProcessHeap(),
            HEAP_ZERO_MEMORY,
            ((*OrigInfo).DomainName.Length as usize + core::mem::size_of::<WCHAR>()) as SIZE_T,
        ) as LPWSTR;
        if !AdminInfo.Domain.is_null() {
            RtlCopyMemory!(
                AdminInfo.Domain as *mut core::ffi::c_void,
                (*OrigInfo).DomainName.Buffer as *const core::ffi::c_void,
                (*OrigInfo).DomainName.Length as usize,
            );
        }

        AdminInfo.Password = RtlAllocateHeap(
            RtlGetProcessHeap(),
            0,
            ((wcslen(Password) + 1) * core::mem::size_of::<WCHAR>()) as SIZE_T,
        ) as LPWSTR;
        if !AdminInfo.Password.is_null() {
            wcscpy(AdminInfo.Password, Password);
        }
    }

    if !AccountNameInfo.is_null() {
        SamFreeMemory(AccountNameInfo as *mut core::ffi::c_void);
    }

    if !OrigInfo.is_null() {
        LsaFreeMemory(OrigInfo as *mut core::ffi::c_void);
    }

    if !PolicyHandle.is_null() {
        LsaClose(PolicyHandle);
    }

    if !UserHandle.is_null() {
        SamCloseHandle(UserHandle);
    }

    if !DomainHandle.is_null() {
        SamCloseHandle(DomainHandle);
    }

    if !ServerHandle.is_null() {
        SamCloseHandle(ServerHandle);
    }

    Status
}
#[no_mangle]
pub unsafe extern "C" fn SetAutoAdminLogon() {
    let mut szAutoAdminLogon: [u16; 2] = [0; 2];
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut dwType: u32 = 0;
    let mut dwSize: u32;
    let mut lError: i32;

    lError = RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        w!("SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\Winlogon") as LPCWSTR,
        0,
        KEY_READ | KEY_WRITE,
        &mut hKey,
    );
    if lError != (ERROR_SUCCESS) as i32 {
        return;
    }

    'done: loop {
        dwSize = 2 * core::mem::size_of::<u16>() as u32;
        lError = RegQueryValueExW(
            hKey,
            w!("AutoAdminLogon") as LPCWSTR,
            core::ptr::null_mut(),
            &mut dwType,
            szAutoAdminLogon.as_mut_ptr() as LPBYTE,
            &mut dwSize,
        );
        if lError != (ERROR_SUCCESS) as i32 {
            break 'done;
        }

        if wcscmp(
            szAutoAdminLogon.as_ptr(),
            w!("1") as LPCWSTR,
        ) == 0
        {
            RegSetValueExW(
                hKey,
                w!("DefaultDomainName") as LPCWSTR,
                0,
                (REG_SZ) as u32,
                AdminInfo.Domain as LPBYTE,
                ((wcslen(AdminInfo.Domain) + 1)
                    * (core::mem::size_of::<u16>() as u32) as usize) as u32,
            );

            RegSetValueExW(
                hKey,
                w!("DefaultUserName") as LPCWSTR,
                0,
                (REG_SZ) as u32,
                AdminInfo.Name as LPBYTE,
                ((wcslen(AdminInfo.Name) + 1)
                    * (core::mem::size_of::<u16>() as u32) as usize) as u32,
            );

            RegSetValueExW(
                hKey,
                w!("DefaultPassword") as LPCWSTR,
                0,
                (REG_SZ) as u32,
                AdminInfo.Password as LPBYTE,
                ((wcslen(AdminInfo.Password) + 1)
                    * (core::mem::size_of::<u16>() as u32) as usize) as u32,
            );
        }

        break 'done;
    }

    if !hKey.is_null() {
        RegCloseKey(hKey);
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:         See COPYING in the top level directory
//  * PROJECT:           ReactOS system libraries
//  * PURPOSE:           System setup
//  * FILE:              dll/win32/syssetup/security.c
//  * PROGRAMER:         Eric Kohl
//  */
//
// /* INCLUDES *****************************************************************/
//
// #include "precomp.h"
//
// #include <ntlsa.h>
// #include <ntsecapi.h>
// #include <ntsam.h>
// #include <sddl.h>
//
// #define NDEBUG
// #include <debug.h>
//
// #define TICKS_PER_DAY -864000000000LL
// #define TICKS_PER_MINUTE -600000000LL
//
// /* FUNCTIONS ****************************************************************/
//
// NTSTATUS
// WINAPI
// SetAccountsDomainSid(
//     PSID DomainSid,
//     LPCWSTR DomainName)
// {
//     PPOLICY_ACCOUNT_DOMAIN_INFO OrigInfo = NULL;
//     POLICY_ACCOUNT_DOMAIN_INFO Info;
//     LSA_OBJECT_ATTRIBUTES ObjectAttributes;
//     LSA_HANDLE PolicyHandle;
//
//     SAM_HANDLE ServerHandle = NULL;
//     SAM_HANDLE DomainHandle = NULL;
//     DOMAIN_NAME_INFORMATION DomainNameInfo;
//
//     SIZE_T DomainNameLength = 0;
//     NTSTATUS Status;
//
//     DPRINT("SYSSETUP: SetAccountsDomainSid\n");
//
//     if (DomainName != NULL)
//     {
//         DomainNameLength = wcslen(DomainName) as SIZE_T;
//         if (DomainNameLength > UNICODE_STRING_MAX_CHARS)
//         {
//             return STATUS_INVALID_PARAMETER;
//         }
//     }
//
//     memset(&ObjectAttributes, 0, sizeof(LSA_OBJECT_ATTRIBUTES));
//     ObjectAttributes.Length = sizeof(LSA_OBJECT_ATTRIBUTES);
//
//     Status = LsaOpenPolicy(NULL,
//                            &ObjectAttributes,
//                            POLICY_VIEW_LOCAL_INFORMATION | POLICY_TRUST_ADMIN,
//                            &PolicyHandle);
//     if (Status != STATUS_SUCCESS)
//     {
//         DPRINT("LsaOpenPolicy failed (Status: 0x%08lx)\n", Status);
//         return Status;
//     }
//
//     Status = LsaQueryInformationPolicy(PolicyHandle,
//                                        PolicyAccountDomainInformation,
//                                        (PVOID *)&OrigInfo);
//     if (Status == STATUS_SUCCESS && OrigInfo != NULL)
//     {
//         if (DomainName == NULL)
//         {
//             Info.DomainName.Buffer = OrigInfo->DomainName.Buffer;
//             Info.DomainName.Length = OrigInfo->DomainName.Length;
//             Info.DomainName.MaximumLength = OrigInfo->DomainName.MaximumLength;
//         }
//         else
//         {
//             Info.DomainName.Buffer = (LPWSTR)DomainName;
//             Info.DomainName.Length = DomainNameLength * sizeof(WCHAR);
//             Info.DomainName.MaximumLength = Info.DomainName.Length + sizeof(WCHAR);
//         }
//
//         if (DomainSid == NULL)
//             Info.DomainSid = OrigInfo->DomainSid;
//         else
//             Info.DomainSid = DomainSid;
//     }
//     else
//     {
//         Info.DomainName.Buffer = (LPWSTR)DomainName;
//         Info.DomainName.Length = DomainNameLength * sizeof(WCHAR);
//         Info.DomainName.MaximumLength = Info.DomainName.Length + sizeof(WCHAR);
//         Info.DomainSid = DomainSid;
//     }
//
//     Status = LsaSetInformationPolicy(PolicyHandle,
//                                      PolicyAccountDomainInformation,
//                                      (PVOID)&Info);
//     if (Status != STATUS_SUCCESS)
//     {
//         DPRINT("LsaSetInformationPolicy failed (Status: 0x%08lx)\n", Status);
//     }
//
//     if (OrigInfo != NULL)
//         LsaFreeMemory(OrigInfo);
//
//     LsaClose(PolicyHandle);
//
//     DomainNameInfo.DomainName.Length = DomainNameLength * sizeof(WCHAR);
//     DomainNameInfo.DomainName.MaximumLength = DomainNameInfo.DomainName.Length + sizeof(WCHAR);
//     DomainNameInfo.DomainName.Buffer = (LPWSTR)DomainName;
//
//     Status = SamConnect(NULL,
//                         &ServerHandle,
//                         SAM_SERVER_CONNECT | SAM_SERVER_LOOKUP_DOMAIN,
//                         NULL);
//     if (NT_SUCCESS(Status))
//     {
//         Status = SamOpenDomain(ServerHandle,
//                                DOMAIN_WRITE_OTHER_PARAMETERS,
//                                Info.DomainSid,
//                                &DomainHandle);
//         if (NT_SUCCESS(Status))
//         {
//             Status = SamSetInformationDomain(DomainHandle,
//                                              DomainNameInformation,
//                                              &DomainNameInfo);
//             if (!NT_SUCCESS(Status))
//             {
//                 DPRINT1("SamSetInformationDomain failed (Status: 0x%08lx)\n", Status);
//             }
//
//             SamCloseHandle(DomainHandle);
//         }
//         else
//         {
//             DPRINT1("SamOpenDomain failed (Status: 0x%08lx)\n", Status);
//         }
//
//         SamCloseHandle(ServerHandle);
//     }
//
//     return Status;
// }
//
//
// /* Hack */
// static
// NTSTATUS
// SetPrimaryDomain(LPCWSTR DomainName,
//                  PSID DomainSid)
// {
//     PPOLICY_PRIMARY_DOMAIN_INFO OrigInfo = NULL;
//     POLICY_PRIMARY_DOMAIN_INFO Info;
//     LSA_OBJECT_ATTRIBUTES ObjectAttributes;
//     LSA_HANDLE PolicyHandle;
//     SIZE_T DomainNameLength = 0;
//     NTSTATUS Status;
//
//     DPRINT1("SYSSETUP: SetPrimaryDomain()\n");
//
//     if (DomainName != NULL)
//     {
//         DomainNameLength = wcslen(DomainName) as SIZE_T;
//         if (DomainNameLength > UNICODE_STRING_MAX_CHARS)
//         {
//             return STATUS_INVALID_PARAMETER;
//         }
//     }
//
//     memset(&ObjectAttributes, 0, sizeof(LSA_OBJECT_ATTRIBUTES));
//     ObjectAttributes.Length = sizeof(LSA_OBJECT_ATTRIBUTES);
//
//     Status = LsaOpenPolicy(NULL,
//                            &ObjectAttributes,
//                            POLICY_VIEW_LOCAL_INFORMATION | POLICY_TRUST_ADMIN,
//                            &PolicyHandle);
//     if (Status != STATUS_SUCCESS)
//     {
//         DPRINT("LsaOpenPolicy failed (Status: 0x%08lx)\n", Status);
//         return Status;
//     }
//
//     Status = LsaQueryInformationPolicy(PolicyHandle,
//                                        PolicyPrimaryDomainInformation,
//                                        (PVOID *)&OrigInfo);
//     if (Status == STATUS_SUCCESS && OrigInfo != NULL)
//     {
//         if (DomainName == NULL)
//         {
//             Info.Name.Buffer = OrigInfo->Name.Buffer;
//             Info.Name.Length = OrigInfo->Name.Length;
//             Info.Name.MaximumLength = OrigInfo->Name.MaximumLength;
//         }
//         else
//         {
//             Info.Name.Buffer = (LPWSTR)DomainName;
//             Info.Name.Length = DomainNameLength * sizeof(WCHAR);
//             Info.Name.MaximumLength = Info.Name.Length + sizeof(WCHAR);
//         }
//
//         if (DomainSid == NULL)
//             Info.Sid = OrigInfo->Sid;
//         else
//             Info.Sid = DomainSid;
//     }
//     else
//     {
//         Info.Name.Buffer = (LPWSTR)DomainName;
//         Info.Name.Length = DomainNameLength * sizeof(WCHAR);
//         Info.Name.MaximumLength = Info.Name.Length + sizeof(WCHAR);
//         Info.Sid = DomainSid;
//     }
//
//     Status = LsaSetInformationPolicy(PolicyHandle,
//                                      PolicyPrimaryDomainInformation,
//                                      (PVOID)&Info);
//     if (Status != STATUS_SUCCESS)
//     {
//         DPRINT("LsaSetInformationPolicy failed (Status: 0x%08lx)\n", Status);
//     }
//
//     if (OrigInfo != NULL)
//         LsaFreeMemory(OrigInfo);
//
//     LsaClose(PolicyHandle);
//
//     return Status;
// }
//
//
// static
// VOID
// InstallBuiltinAccounts(VOID)
// {
//     LPWSTR BuiltinAccounts[] = {
//         L"S-1-1-0",         /* Everyone */
//         L"S-1-5-4",         /* Interactive */
//         L"S-1-5-6",         /* Service */
//         L"S-1-5-19",        /* Local Service */
//         L"S-1-5-20",        /* Network Service */
//         L"S-1-5-32-544",    /* Administrators */
//         L"S-1-5-32-545",    /* Users */
//         L"S-1-5-32-547",    /* Power Users */
//         L"S-1-5-32-551",    /* Backup Operators */
//         L"S-1-5-32-555"};   /* Remote Desktop Users */
//     LSA_OBJECT_ATTRIBUTES ObjectAttributes;
//     NTSTATUS Status;
//     LSA_HANDLE PolicyHandle = NULL;
//     LSA_HANDLE AccountHandle = NULL;
//     PSID AccountSid;
//     ULONG i;
//
//     DPRINT("InstallBuiltinAccounts()\n");
//
//     memset(&ObjectAttributes, 0, sizeof(LSA_OBJECT_ATTRIBUTES));
//
//     Status = LsaOpenPolicy(NULL,
//                            &ObjectAttributes,
//                            POLICY_CREATE_ACCOUNT,
//                            &PolicyHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("LsaOpenPolicy failed (Status %08lx)\n", Status);
//         return;
//     }
//
//     for (i = 0; i < ARRAYSIZE!(BuiltinAccounts); i++)
//     {
//         if (!ConvertStringSidToSid(BuiltinAccounts[i], &AccountSid))
//         {
//             DPRINT1("ConvertStringSidToSid(%S) failed: %lu\n", BuiltinAccounts[i], GetLastError());
//             continue;
//         }
//
//         Status = LsaCreateAccount(PolicyHandle,
//                                   AccountSid,
//                                   0,
//                                   &AccountHandle);
//         if (NT_SUCCESS(Status))
//         {
//             LsaClose(AccountHandle);
//         }
//
//         LocalFree(AccountSid);
//     }
//
//     LsaClose(PolicyHandle);
// }
//
//
// static
// VOID
// InstallPrivileges(
//     HINF hSecurityInf)
// {
//     LSA_OBJECT_ATTRIBUTES ObjectAttributes;
//     WCHAR szPrivilegeString[256];
//     WCHAR szSidString[256];
//     INFCONTEXT InfContext;
//     DWORD i;
//     PSID AccountSid = NULL;
//     NTSTATUS Status;
//     LSA_HANDLE PolicyHandle = NULL;
//     LSA_UNICODE_STRING RightString, AccountName;
//     PLSA_REFERENCED_DOMAIN_LIST ReferencedDomains = NULL;
//     PLSA_TRANSLATED_SID2 Sids = NULL;
//
//     DPRINT("InstallPrivileges()\n");
//
//     memset(&ObjectAttributes, 0, sizeof(LSA_OBJECT_ATTRIBUTES));
//
//     Status = LsaOpenPolicy(NULL,
//                            &ObjectAttributes,
//                            POLICY_CREATE_ACCOUNT | POLICY_LOOKUP_NAMES,
//                            &PolicyHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("LsaOpenPolicy failed (Status %08lx)\n", Status);
//         goto done;
//     }
//
//     if (!SetupFindFirstLineW(hSecurityInf,
//                              L"Privilege Rights",
//                              NULL,
//                              &InfContext))
//     {
//         DPRINT1("SetupFindFirstLineW failed\n");
//         goto done;
//     }
//
//     do
//     {
//         /* Retrieve the privilege name */
//         if (!SetupGetStringFieldW(&InfContext,
//                                   0,
//                                   szPrivilegeString,
//                                   ARRAYSIZE!(szPrivilegeString),
//                                   NULL))
//         {
//             DPRINT1("SetupGetStringFieldW() failed\n");
//             goto done;
//         }
//         DPRINT("Privilege: %S\n", szPrivilegeString);
//
//         for (i = 0; i < SetupGetFieldCount(&InfContext); i++)
//         {
//             if (!SetupGetStringFieldW(&InfContext,
//                                       i + 1,
//                                       szSidString,
//                                       ARRAYSIZE!(szSidString),
//                                       NULL))
//             {
//                 DPRINT1("SetupGetStringFieldW() failed\n");
//                 goto done;
//             }
//             DPRINT("SID: %S\n", szSidString);
//
//             if (szSidString[0] == UNICODE_NULL)
//                 continue;
//
//             if (szSidString[0] == '*')
//             {
//                 DPRINT("Account Sid: %S\n", &szSidString[1]);
//
//                 if (!ConvertStringSidToSid(&szSidString[1], &AccountSid))
//                 {
//                     DPRINT1("ConvertStringSidToSid(%S) failed: %lu\n", szSidString, GetLastError());
//                     continue;
//                 }
//             }
//             else
//             {
//                 DPRINT("Account name: %S\n", szSidString);
//
//                 ReferencedDomains = NULL;
//                 Sids = NULL;
//                 RtlInitUnicodeString(&AccountName, szSidString);
//                 Status = LsaLookupNames2(PolicyHandle,
//                                          0,
//                                          1,
//                                          &AccountName,
//                                          &ReferencedDomains,
//                                          &Sids);
//                 if (ReferencedDomains != NULL)
//                 {
//                     LsaFreeMemory(ReferencedDomains);
//                 }
//
//                 if (!NT_SUCCESS(Status))
//                 {
//                     DPRINT1("LsaLookupNames2() failed (Status 0x%08lx)\n", Status);
//
//                     if (Sids != NULL)
//                     {
//                         LsaFreeMemory(Sids);
//                         Sids = NULL;
//                     }
//
//                     continue;
//                 }
//             }
//
//             RtlInitUnicodeString(&RightString, szPrivilegeString);
//             Status = LsaAddAccountRights(PolicyHandle,
//                                          (AccountSid != NULL) ? AccountSid : Sids[0].Sid,
//                                          &RightString,
//                                          1);
//             if (!NT_SUCCESS(Status))
//             {
//                 DPRINT1("LsaAddAccountRights() failed (Status %08lx)\n", Status);
//             }
//
//             if (Sids != NULL)
//             {
//                 LsaFreeMemory(Sids);
//                 Sids = NULL;
//             }
//
//             if (AccountSid != NULL)
//             {
//                 LocalFree(AccountSid);
//                 AccountSid = NULL;
//             }
//         }
//
//     }
//     while (SetupFindNextLine(&InfContext, &InfContext));
//
// done:
//     if (PolicyHandle != NULL)
//         LsaClose(PolicyHandle);
// }
//
//
// static
// VOID
// ApplyRegistryValues(
//     HINF hSecurityInf)
// {
//     WCHAR szRegistryPath[MAX_PATH];
//     WCHAR szRootName[MAX_PATH];
//     WCHAR szKeyName[MAX_PATH];
//     WCHAR szValueName[MAX_PATH];
//     INFCONTEXT InfContext;
//     DWORD dwLength, dwType;
//     HKEY hRootKey, hKey;
//     PWSTR Ptr1, Ptr2;
//     DWORD dwError;
//     PVOID pBuffer;
//
//     DPRINT("ApplyRegistryValues()\n");
//
//     if (!SetupFindFirstLineW(hSecurityInf,
//                              L"Registry Values",
//                              NULL,
//                              &InfContext))
//     {
//         DPRINT1("SetupFindFirstLineW failed\n");
//         return;
//     }
//
//     do
//     {
//         /* Retrieve the privilege name */
//         if (!SetupGetStringFieldW(&InfContext,
//                                   0,
//                                   szRegistryPath,
//                                   ARRAYSIZE!(szRegistryPath),
//                                   NULL))
//         {
//             DPRINT1("SetupGetStringFieldW() failed\n");
//             return;
//         }
//
//         DPRINT("RegistryPath: %S\n", szRegistryPath);
//
//         Ptr1 = wcschr(szRegistryPath, '\\');
//         Ptr2 = wcsrchr(szRegistryPath, '\\');
//         if (Ptr1 != NULL && Ptr2 != NULL && Ptr1 != Ptr2)
//         {
//             dwLength = (DWORD)(((ULONG_PTR)Ptr1 - (ULONG_PTR)szRegistryPath) / sizeof(WCHAR));
//             wcsncpy(szRootName, szRegistryPath, dwLength);
//             szRootName[dwLength] = UNICODE_NULL;
//
//             Ptr1++;
//             dwLength = (DWORD)(((ULONG_PTR)Ptr2 - (ULONG_PTR)Ptr1) / sizeof(WCHAR));
//             wcsncpy(szKeyName, Ptr1, dwLength);
//             szKeyName[dwLength] = UNICODE_NULL;
//
//             Ptr2++;
//             wcscpy(szValueName, Ptr2);
//
//             DPRINT("RootName: %S\n", szRootName);
//             DPRINT("KeyName: %S\n", szKeyName);
//             DPRINT("ValueName: %S\n", szValueName);
//
//             if (_wcsicmp(szRootName, L"Machine") == 0)
//             {
//                 hRootKey = HKEY_LOCAL_MACHINE;
//             }
//             else
//             {
//                 DPRINT1("Unsupported root key %S\n", szRootName);
//                 break;
//             }
//
//             if (!SetupGetIntField(&InfContext,
//                                   1,
//                                   (PINT)&dwType))
//             {
//                 DPRINT1("Failed to get key type (Error %lu)\n", GetLastError());
//                 break;
//             }
//
//             if (dwType != REG_SZ && dwType != REG_EXPAND_SZ && dwType != REG_BINARY &&
//                 dwType != REG_DWORD && dwType != REG_MULTI_SZ)
//             {
//                 DPRINT1("Invalid value type %lu\n", dwType);
//                 break;
//             }
//
//             dwLength = 0;
//             switch (dwType)
//             {
//                 case REG_SZ:
//                 case REG_EXPAND_SZ:
//                     SetupGetStringField(&InfContext,
//                                         2,
//                                         NULL,
//                                         0,
//                                         &dwLength);
//                     dwLength *= sizeof(WCHAR);
//                     break;
//
//                 case REG_BINARY:
//                     SetupGetBinaryField(&InfContext,
//                                         2,
//                                         NULL,
//                                         0,
//                                         &dwLength);
//                     break;
//
//                 case REG_DWORD:
//                     dwLength = sizeof(INT);
//                     break;
//
//                 case REG_MULTI_SZ:
//                     SetupGetMultiSzField(&InfContext,
//                                          2,
//                                          NULL,
//                                          0,
//                                          &dwLength);
//                     dwLength *= sizeof(WCHAR);
//                     break;
//             }
//
//             if (dwLength == 0)
//             {
//                 DPRINT1("Failed to determine the required buffer size!\n");
//                 break;
//             }
//
//             dwError = RegCreateKeyExW(hRootKey,
//                                       szKeyName,
//                                       0,
//                                       NULL,
//                                       REG_OPTION_NON_VOLATILE,
//                                       KEY_WRITE,
//                                       NULL,
//                                       &hKey,
//                                       NULL);
//             if (dwError != ERROR_SUCCESS)
//             {
//                 DPRINT1("Failed to create the key %S (Error %lu)\n", szKeyName, dwError);
//                 break;
//             }
//
//             pBuffer = HeapAlloc(GetProcessHeap(), 0, dwLength);
//             if (pBuffer)
//             {
//                 switch (dwType)
//                 {
//                     case REG_SZ:
//                     case REG_EXPAND_SZ:
//                         SetupGetStringField(&InfContext,
//                                             2,
//                                             pBuffer,
//                                             dwLength / sizeof(WCHAR),
//                                             &dwLength);
//                         dwLength *= sizeof(WCHAR);
//                         break;
//
//                     case REG_BINARY:
//                         SetupGetBinaryField(&InfContext,
//                                             2,
//                                             pBuffer,
//                                             dwLength,
//                                             &dwLength);
//                         break;
//
//                     case REG_DWORD:
//                         SetupGetIntField(&InfContext,
//                                          2,
//                                          pBuffer);
//                         break;
//
//                     case REG_MULTI_SZ:
//                         SetupGetMultiSzField(&InfContext,
//                                              2,
//                                              pBuffer,
//                                              dwLength / sizeof(WCHAR),
//                                              &dwLength);
//                         dwLength *= sizeof(WCHAR);
//                         break;
//                 }
//
//                 RegSetValueEx(hKey,
//                               szValueName,
//                               0,
//                               dwType,
//                               pBuffer,
//                               dwLength);
//
//                 HeapFree(GetProcessHeap(), 0, pBuffer);
//             }
//
//             RegCloseKey(hKey);
//         }
//     }
//     while (SetupFindNextLine(&InfContext, &InfContext));
// }
//
//
// static
// VOID
// ApplyEventlogSettings(
//      HINF hSecurityInf,
//      PWSTR pszSectionName,
//      PWSTR pszLogName)
// {
//     INFCONTEXT InfContext;
//     HKEY hServiceKey = NULL, hLogKey = NULL;
//     DWORD dwValue, dwError;
//     BOOL bValueSet;
//
//     DPRINT("ApplyEventlogSettings(%p %S %S)\n",
//            hSecurityInf, pszSectionName, pszLogName);
//
//     dwError = RegCreateKeyExW(HKEY_LOCAL_MACHINE,
//                               L"System\\CurrentControlSet\\Services\\Eventlog",
//                               0,
//                               NULL,
//                               REG_OPTION_NON_VOLATILE,
//                               KEY_WRITE,
//                               NULL,
//                               &hServiceKey,
//                               NULL);
//     if (dwError != ERROR_SUCCESS)
//     {
//         DPRINT1("Failed to create the Eventlog Service key (Error %lu)\n", dwError);
//         return;
//     }
//
//     dwError = RegCreateKeyExW(hServiceKey,
//                               pszLogName,
//                               0,
//                               NULL,
//                               REG_OPTION_NON_VOLATILE,
//                               KEY_WRITE,
//                               NULL,
//                               &hLogKey,
//                               NULL);
//     if (dwError != ERROR_SUCCESS)
//     {
//         DPRINT1("Failed to create the key %S (Error %lu)\n", pszLogName, dwError);
//         RegCloseKey(hServiceKey);
//         return;
//     }
//
//     if (SetupFindFirstLineW(hSecurityInf,
//                             pszSectionName,
//                             L"MaximumLogSize",
//                             &InfContext))
//     {
//         DPRINT("MaximumLogSize\n");
//         dwValue = 0;
//         SetupGetIntField(&InfContext,
//                          1,
//                          (PINT)&dwValue);
//
//         DPRINT("MaximumLogSize: %lu (kByte)\n", dwValue);
//         if (dwValue >= 64 && dwValue <= 4194240)
//         {
//             dwValue *= 1024;
//
//             DPRINT("MaxSize: %lu\n", dwValue);
//             RegSetValueEx(hLogKey,
//                           L"MaxSize",
//                           0,
//                           REG_DWORD,
//                           (LPBYTE)&dwValue,
//                           sizeof(dwValue));
//         }
//     }
//
//     if (SetupFindFirstLineW(hSecurityInf,
//                             pszSectionName,
//                             L"AuditLogRetentionPeriod",
//                             &InfContext))
//     {
//         bValueSet = FALSE;
//         dwValue = 0;
//         SetupGetIntField(&InfContext,
//                          1,
//                          (PINT)&dwValue);
//         if (dwValue == 0)
//         {
//             bValueSet = TRUE;
//         }
//         else if (dwValue == 1)
//         {
//             if (SetupFindFirstLineW(hSecurityInf,
//                                     pszSectionName,
//                                     L"RetentionDays",
//                                     &InfContext))
//             {
//                 SetupGetIntField(&InfContext,
//                                  1,
//                                  (PINT)&dwValue);
//                 dwValue *= 86400;
//                 bValueSet = TRUE;
//             }
//         }
//         else if (dwValue == 2)
//         {
//             dwValue = (DWORD)-1;
//             bValueSet = TRUE;
//         }
//
//         if (bValueSet)
//         {
//             DPRINT("Retention: %lu\n", dwValue);
//             RegSetValueEx(hLogKey,
//                           L"Retention",
//                           0,
//                           REG_DWORD,
//                           (LPBYTE)&dwValue,
//                           sizeof(dwValue));
//         }
//     }
//
//     if (SetupFindFirstLineW(hSecurityInf,
//                             pszSectionName,
//                             L"RestrictGuestAccess",
//                             &InfContext))
//     {
//         dwValue = 0;
//         SetupGetIntField(&InfContext,
//                          1,
//                          (PINT)&dwValue);
//         if (dwValue == 0 || dwValue == 1)
//         {
//             DPRINT("RestrictGuestAccess: %lu\n", dwValue);
//             RegSetValueEx(hLogKey,
//                           L"RestrictGuestAccess",
//                           0,
//                           REG_DWORD,
//                           (LPBYTE)&dwValue,
//                           sizeof(dwValue));
//         }
//     }
//
//     RegCloseKey(hLogKey);
//     RegCloseKey(hServiceKey);
// }
//
//
// static
// VOID
// ApplyPasswordSettings(
//      HINF hSecurityInf,
//      PWSTR pszSectionName)
// {
//     INFCONTEXT InfContext;
//     PDOMAIN_PASSWORD_INFORMATION PasswordInfo = NULL;
//     PPOLICY_ACCOUNT_DOMAIN_INFO OrigInfo = NULL;
//     LSA_OBJECT_ATTRIBUTES ObjectAttributes;
//     LSA_HANDLE PolicyHandle = NULL;
//     SAM_HANDLE ServerHandle = NULL;
//     SAM_HANDLE DomainHandle = NULL;
//     INT nValue;
//     NTSTATUS Status;
//
//     DPRINT("ApplyPasswordSettings()\n");
//
//     memset(&ObjectAttributes, 0, sizeof(LSA_OBJECT_ATTRIBUTES));
//     ObjectAttributes.Length = sizeof(LSA_OBJECT_ATTRIBUTES);
//
//     Status = LsaOpenPolicy(NULL,
//                            &ObjectAttributes,
//                            POLICY_VIEW_LOCAL_INFORMATION | POLICY_TRUST_ADMIN,
//                            &PolicyHandle);
//     if (Status != STATUS_SUCCESS)
//     {
//         DPRINT1("LsaOpenPolicy() failed (Status: 0x%08lx)\n", Status);
//         return;
//     }
//
//     Status = LsaQueryInformationPolicy(PolicyHandle,
//                                        PolicyAccountDomainInformation,
//                                        (PVOID *)&OrigInfo);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("LsaQueryInformationPolicy() failed (Status: 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     Status = SamConnect(NULL,
//                         &ServerHandle,
//                         SAM_SERVER_CONNECT | SAM_SERVER_LOOKUP_DOMAIN,
//                         NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("SamConnect() failed (Status: 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     Status = SamOpenDomain(ServerHandle,
//                            DOMAIN_READ_PASSWORD_PARAMETERS | DOMAIN_WRITE_PASSWORD_PARAMS,
//                            OrigInfo->DomainSid,
//                            &DomainHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("SamOpenDomain() failed (Status: 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     Status = SamQueryInformationDomain(DomainHandle,
//                                        DomainPasswordInformation,
//                                        (PVOID*)&PasswordInfo);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("SamQueryInformationDomain() failed (Status %08lx)\n", Status);
//         goto done;
//     }
//
//     DPRINT("MaximumPasswordAge (OldValue) : 0x%I64x\n", PasswordInfo->MaxPasswordAge.QuadPart);
//     if (SetupFindFirstLineW(hSecurityInf,
//                             pszSectionName,
//                             L"MaximumPasswordAge",
//                             &InfContext))
//     {
//         if (SetupGetIntField(&InfContext, 1, &nValue))
//         {
//             DPRINT("Value: %ld\n", nValue);
//             if (nValue == -1)
//             {
//                 PasswordInfo->MaxPasswordAge.QuadPart = 0x8000000000000000;
//             }
//             else if ((nValue >= 1) && (nValue < 1000))
//             {
//                 PasswordInfo->MaxPasswordAge.QuadPart = (LONGLONG)nValue * TICKS_PER_DAY;
//             }
//             DPRINT("MaximumPasswordAge (NewValue) : 0x%I64x\n", PasswordInfo->MaxPasswordAge.QuadPart);
//         }
//     }
//
//     DPRINT("MinimumPasswordAge (OldValue) : 0x%I64x\n", PasswordInfo->MinPasswordAge.QuadPart);
//     if (SetupFindFirstLineW(hSecurityInf,
//                             pszSectionName,
//                             L"MinimumPasswordAge",
//                             &InfContext))
//     {
//         if (SetupGetIntField(&InfContext, 1, &nValue))
//         {
//             DPRINT("Wert: %ld\n", nValue);
//             if ((nValue >= 0) && (nValue < 1000))
//             {
//                 if (PasswordInfo->MaxPasswordAge.QuadPart < (LONGLONG)nValue * TICKS_PER_DAY)
//                     PasswordInfo->MinPasswordAge.QuadPart = (LONGLONG)nValue * TICKS_PER_DAY;
//             }
//             DPRINT("MinimumPasswordAge (NewValue) : 0x%I64x\n", PasswordInfo->MinPasswordAge.QuadPart);
//         }
//     }
//
//     DPRINT("MinimumPasswordLength (OldValue) : %lu\n", PasswordInfo->MinPasswordLength);
//     if (SetupFindFirstLineW(hSecurityInf,
//                             pszSectionName,
//                             L"MinimumPasswordLength",
//                             &InfContext))
//     {
//         if (SetupGetIntField(&InfContext, 1, &nValue))
//         {
//             DPRINT("Value: %ld\n", nValue);
//             if ((nValue >= 0) && (nValue <= 65535))
//             {
//                 PasswordInfo->MinPasswordLength = nValue;
//             }
//             DPRINT("MinimumPasswordLength (NewValue) : %lu\n", PasswordInfo->MinPasswordLength);
//         }
//     }
//
//     DPRINT("PasswordHistoryLength (OldValue) : %lu\n", PasswordInfo->PasswordHistoryLength);
//     if (SetupFindFirstLineW(hSecurityInf,
//                             pszSectionName,
//                             L"PasswordHistorySize",
//                             &InfContext))
//     {
//         if (SetupGetIntField(&InfContext, 1, &nValue))
//         {
//             DPRINT("Value: %ld\n", nValue);
//             if ((nValue >= 0) && (nValue <= 65535))
//             {
//                 PasswordInfo->PasswordHistoryLength = nValue;
//             }
//             DPRINT("PasswordHistoryLength (NewValue) : %lu\n", PasswordInfo->PasswordHistoryLength);
//         }
//     }
//
//     if (SetupFindFirstLineW(hSecurityInf,
//                             pszSectionName,
//                             L"PasswordComplexity",
//                             &InfContext))
//     {
//         if (SetupGetIntField(&InfContext, 1, &nValue))
//         {
//             if (nValue == 0)
//             {
//                 PasswordInfo->PasswordProperties &= ~DOMAIN_PASSWORD_COMPLEX;
//             }
//             else
//             {
//                 PasswordInfo->PasswordProperties |= DOMAIN_PASSWORD_COMPLEX;
//             }
//         }
//     }
//
//     if (SetupFindFirstLineW(hSecurityInf,
//                             pszSectionName,
//                             L"ClearTextPassword",
//                             &InfContext))
//     {
//         if (SetupGetIntField(&InfContext, 1, &nValue))
//         {
//             if (nValue == 0)
//             {
//                 PasswordInfo->PasswordProperties &= ~DOMAIN_PASSWORD_STORE_CLEARTEXT;
//             }
//             else
//             {
//                 PasswordInfo->PasswordProperties |= DOMAIN_PASSWORD_STORE_CLEARTEXT;
//             }
//         }
//     }
//
//     /* Windows ignores the RequireLogonToChangePassword option */
//
//     Status = SamSetInformationDomain(DomainHandle,
//                                      DomainPasswordInformation,
//                                      PasswordInfo);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("SamSetInformationDomain() failed (Status %08lx)\n", Status);
//         goto done;
//     }
//
// done:
//     if (PasswordInfo != NULL)
//         SamFreeMemory(PasswordInfo);
//
//     if (DomainHandle != NULL)
//         SamCloseHandle(DomainHandle);
//
//     if (ServerHandle != NULL)
//         SamCloseHandle(ServerHandle);
//
//     if (OrigInfo != NULL)
//         LsaFreeMemory(OrigInfo);
//
//     if (PolicyHandle != NULL)
//         LsaClose(PolicyHandle);
// }
//
//
// static
// VOID
// ApplyLockoutSettings(
//      HINF hSecurityInf,
//      PWSTR pszSectionName)
// {
//     INFCONTEXT InfContext;
//     PDOMAIN_LOCKOUT_INFORMATION LockoutInfo = NULL;
//     PPOLICY_ACCOUNT_DOMAIN_INFO OrigInfo = NULL;
//     LSA_OBJECT_ATTRIBUTES ObjectAttributes;
//     LSA_HANDLE PolicyHandle = NULL;
//     SAM_HANDLE ServerHandle = NULL;
//     SAM_HANDLE DomainHandle = NULL;
//     INT nValue;
//     NTSTATUS Status;
//
//     DPRINT("ApplyLockoutSettings()\n");
//
//     memset(&ObjectAttributes, 0, sizeof(LSA_OBJECT_ATTRIBUTES));
//     ObjectAttributes.Length = sizeof(LSA_OBJECT_ATTRIBUTES);
//
//     Status = LsaOpenPolicy(NULL,
//                            &ObjectAttributes,
//                            POLICY_VIEW_LOCAL_INFORMATION | POLICY_TRUST_ADMIN,
//                            &PolicyHandle);
//     if (Status != STATUS_SUCCESS)
//     {
//         DPRINT1("LsaOpenPolicy() failed (Status: 0x%08lx)\n", Status);
//         return;
//     }
//
//     Status = LsaQueryInformationPolicy(PolicyHandle,
//                                        PolicyAccountDomainInformation,
//                                        (PVOID *)&OrigInfo);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("LsaQueryInformationPolicy() failed (Status: 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     Status = SamConnect(NULL,
//                         &ServerHandle,
//                         SAM_SERVER_CONNECT | SAM_SERVER_LOOKUP_DOMAIN,
//                         NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("SamConnect() failed (Status: 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     Status = SamOpenDomain(ServerHandle,
//                            DOMAIN_READ_PASSWORD_PARAMETERS | DOMAIN_WRITE_PASSWORD_PARAMS,
//                            OrigInfo->DomainSid,
//                            &DomainHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("SamOpenDomain() failed (Status: 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     Status = SamQueryInformationDomain(DomainHandle,
//                                        DomainLockoutInformation,
//                                        (PVOID*)&LockoutInfo);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("SamQueryInformationDomain() failed (Status %08lx)\n", Status);
//         goto done;
//     }
//
//     if (SetupFindFirstLineW(hSecurityInf,
//                             pszSectionName,
//                             L"LockoutBadCount",
//                             &InfContext))
//     {
//         if (SetupGetIntField(&InfContext, 1, &nValue))
//         {
//             if (nValue >= 0)
//             {
//                 LockoutInfo->LockoutThreshold = nValue;
//             }
//         }
//     }
//
//     if (SetupFindFirstLineW(hSecurityInf,
//                             pszSectionName,
//                             L"ResetLockoutCount",
//                             &InfContext))
//     {
//         if (SetupGetIntField(&InfContext, 1, &nValue))
//         {
//             if (nValue >= 0)
//             {
//                 LockoutInfo->LockoutObservationWindow.QuadPart = (LONGLONG)nValue * TICKS_PER_MINUTE;
//             }
//         }
//     }
//
//     if (SetupFindFirstLineW(hSecurityInf,
//                             pszSectionName,
//                             L"LockoutDuration",
//                             &InfContext))
//     {
//         if (SetupGetIntField(&InfContext, 1, &nValue))
//         {
//             if (nValue == -1)
//             {
//                 LockoutInfo->LockoutDuration.QuadPart = 0x8000000000000000LL;
//             }
//             else if ((nValue >= 0) && (nValue < 100000))
//             {
//                 LockoutInfo->LockoutDuration.QuadPart = (LONGLONG)nValue * TICKS_PER_MINUTE;
//             }
//         }
//     }
//
//     Status = SamSetInformationDomain(DomainHandle,
//                                      DomainLockoutInformation,
//                                      LockoutInfo);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("SamSetInformationDomain() failed (Status %08lx)\n", Status);
//         goto done;
//     }
//
// done:
//     if (LockoutInfo != NULL)
//         SamFreeMemory(LockoutInfo);
//
//     if (DomainHandle != NULL)
//         SamCloseHandle(DomainHandle);
//
//     if (ServerHandle != NULL)
//         SamCloseHandle(ServerHandle);
//
//     if (OrigInfo != NULL)
//         LsaFreeMemory(OrigInfo);
//
//     if (PolicyHandle != NULL)
//         LsaClose(PolicyHandle);
// }
//
//
// static
// VOID
// SetLsaAnonymousNameLookup(
//      HINF hSecurityInf,
//      PWSTR pszSectionName)
// {
// }
//
//
// static
// VOID
// EnableAccount(
//      HINF hSecurityInf,
//      PWSTR pszSectionName,
//      PWSTR pszValueName,
//      SAM_HANDLE DomainHandle,
//      DWORD dwAccountRid)
// {
//     INFCONTEXT InfContext;
//     SAM_HANDLE UserHandle = NULL;
//     PUSER_CONTROL_INFORMATION ControlInfo = NULL;
//     INT nValue = 0;
//     NTSTATUS Status;
//
//     DPRINT("EnableAccount()\n");
//
//     if (!SetupFindFirstLineW(hSecurityInf,
//                             pszSectionName,
//                             pszValueName,
//                             &InfContext))
//         return;
//
//     if (!SetupGetIntField(&InfContext, 1, &nValue))
//     {
//         DPRINT1("No valid integer value\n");
//         goto done;
//     }
//
//     DPRINT("Value: %d\n", nValue);
//
//     Status = SamOpenUser(DomainHandle,
//                          USER_READ_ACCOUNT | USER_WRITE_ACCOUNT,
//                          dwAccountRid,
//                          &UserHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("SamOpenUser() failed (Status: 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     Status = SamQueryInformationUser(UserHandle,
//                                      UserControlInformation,
//                                      (PVOID*)&ControlInfo);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("SamQueryInformationUser() failed (Status: 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     if (nValue == 0)
//     {
//         ControlInfo->UserAccountControl |= USER_ACCOUNT_DISABLED;
//     }
//     else
//     {
//         ControlInfo->UserAccountControl &= ~USER_ACCOUNT_DISABLED;
//     }
//
//     Status = SamSetInformationUser(UserHandle,
//                                    UserControlInformation,
//                                    ControlInfo);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("SamSetInformationUser() failed (Status: 0x%08lx)\n", Status);
//     }
//
// done:
//     if (ControlInfo != NULL)
//         SamFreeMemory(ControlInfo);
//
//     if (UserHandle != NULL)
//         SamCloseHandle(UserHandle);
// }
//
//
// static
// VOID
// SetNewAccountName(
//      HINF hSecurityInf,
//      PWSTR pszSectionName,
//      PWSTR pszValueName,
//      SAM_HANDLE DomainHandle,
//      DWORD dwAccountRid)
// {
//     INFCONTEXT InfContext;
//     DWORD dwLength = 0;
//     PWSTR pszName = NULL;
//     SAM_HANDLE UserHandle = NULL;
//     USER_NAME_INFORMATION NameInfo;
//     NTSTATUS Status;
//
//     DPRINT("SetNewAccountName()\n");
//
//     if (!SetupFindFirstLineW(hSecurityInf,
//                             pszSectionName,
//                             pszValueName,
//                             &InfContext))
//         return;
//
//     SetupGetStringFieldW(&InfContext,
//                          1,
//                          NULL,
//                          0,
//                          &dwLength);
//     if (dwLength == 0)
//         return;
//
//     ASSERT(dwLength <= UNICODE_STRING_MAX_CHARS);
//
//     pszName = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, dwLength * sizeof(WCHAR));
//     if (pszName == NULL)
//     {
//         DPRINT1("HeapAlloc() failed\n");
//         return;
//     }
//
//     if (!SetupGetStringFieldW(&InfContext,
//                               1,
//                               pszName,
//                               dwLength,
//                               &dwLength))
//     {
//         DPRINT1("No valid string value\n");
//         goto done;
//     }
//
//     DPRINT("NewAccountName: '%S'\n", pszName);
//
//     Status = SamOpenUser(DomainHandle,
//                          USER_WRITE_ACCOUNT,
//                          dwAccountRid,
//                          &UserHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("SamOpenUser() failed (Status: 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     NameInfo.UserName.Length = (USHORT)wcslen(pszName) * sizeof(WCHAR);
//     NameInfo.UserName.MaximumLength = NameInfo.UserName.Length + sizeof(WCHAR);
//     NameInfo.UserName.Buffer = pszName;
//     NameInfo.FullName.Length = 0;
//     NameInfo.FullName.MaximumLength = 0;
//     NameInfo.FullName.Buffer = NULL;
//
//     Status = SamSetInformationUser(UserHandle,
//                                    UserNameInformation,
//                                    &NameInfo);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("SamSetInformationUser() failed (Status: 0x%08lx)\n", Status);
//     }
//
// done:
//     if (UserHandle != NULL)
//         SamCloseHandle(UserHandle);
//
//     if (pszName != NULL)
//         HeapFree(GetProcessHeap(), 0, pszName);
// }
//
//
// static
// VOID
// ApplyAccountSettings(
//      HINF hSecurityInf,
//      PWSTR pszSectionName)
// {
//     PPOLICY_ACCOUNT_DOMAIN_INFO OrigInfo = NULL;
//     LSA_OBJECT_ATTRIBUTES ObjectAttributes;
//     LSA_HANDLE PolicyHandle = NULL;
//     SAM_HANDLE ServerHandle = NULL;
//     SAM_HANDLE DomainHandle = NULL;
//     NTSTATUS Status;
//
//     DPRINT("ApplyAccountSettings()\n");
//
//     memset(&ObjectAttributes, 0, sizeof(LSA_OBJECT_ATTRIBUTES));
//     ObjectAttributes.Length = sizeof(LSA_OBJECT_ATTRIBUTES);
//
//     Status = LsaOpenPolicy(NULL,
//                            &ObjectAttributes,
//                            POLICY_VIEW_LOCAL_INFORMATION | POLICY_TRUST_ADMIN,
//                            &PolicyHandle);
//     if (Status != STATUS_SUCCESS)
//     {
//         DPRINT1("LsaOpenPolicy() failed (Status: 0x%08lx)\n", Status);
//         return;
//     }
//
//     Status = LsaQueryInformationPolicy(PolicyHandle,
//                                        PolicyAccountDomainInformation,
//                                        (PVOID *)&OrigInfo);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("LsaQueryInformationPolicy() failed (Status: 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     Status = SamConnect(NULL,
//                         &ServerHandle,
//                         SAM_SERVER_CONNECT | SAM_SERVER_LOOKUP_DOMAIN,
//                         NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("SamConnect() failed (Status: 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     Status = SamOpenDomain(ServerHandle,
//                            DOMAIN_LOOKUP,
//                            OrigInfo->DomainSid,
//                            &DomainHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("SamOpenDomain() failed (Status: 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     SetLsaAnonymousNameLookup(hSecurityInf,
//                               pszSectionName);
//
//     EnableAccount(hSecurityInf,
//                   pszSectionName,
//                   L"EnableAdminAccount",
//                   DomainHandle,
//                   DOMAIN_USER_RID_ADMIN);
//
//     EnableAccount(hSecurityInf,
//                   pszSectionName,
//                   L"EnableGuestAccount",
//                   DomainHandle,
//                   DOMAIN_USER_RID_GUEST);
//
//     SetNewAccountName(hSecurityInf,
//                       pszSectionName,
//                       L"NewAdministratorName",
//                       DomainHandle,
//                       DOMAIN_USER_RID_ADMIN);
//
//     SetNewAccountName(hSecurityInf,
//                       pszSectionName,
//                       L"NewGuestName",
//                       DomainHandle,
//                       DOMAIN_USER_RID_GUEST);
//
// done:
//     if (DomainHandle != NULL)
//         SamCloseHandle(DomainHandle);
//
//     if (ServerHandle != NULL)
//         SamCloseHandle(ServerHandle);
//
//     if (OrigInfo != NULL)
//         LsaFreeMemory(OrigInfo);
//
//     if (PolicyHandle != NULL)
//         LsaClose(PolicyHandle);
// }
//
//
// static
// VOID
// ApplyAuditEvents(
//      HINF hSecurityInf)
// {
//     LSA_OBJECT_ATTRIBUTES ObjectAttributes;
//     INFCONTEXT InfContext;
//     WCHAR szOptionName[256];
//     INT nValue;
//     LSA_HANDLE PolicyHandle = NULL;
//     POLICY_AUDIT_EVENTS_INFO AuditInfo;
//     PULONG AuditOptions = NULL;
//     NTSTATUS Status;
//
//     DPRINT("ApplyAuditEvents(%p)\n", hSecurityInf);
//
//     if (!SetupFindFirstLineW(hSecurityInf,
//                              L"Event Audit",
//                              NULL,
//                              &InfContext))
//     {
//         DPRINT1("SetupFindFirstLineW failed\n");
//         return;
//     }
//
//     ZeroMemory!(&ObjectAttributes, sizeof(LSA_OBJECT_ATTRIBUTES));
//
//     Status = LsaOpenPolicy(NULL,
//                            &ObjectAttributes,
//                            POLICY_SET_AUDIT_REQUIREMENTS,
//                            &PolicyHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("LsaOpenPolicy failed (Status %08lx)\n", Status);
//         return;
//     }
//
//     AuditOptions = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY,
//                              (AuditCategoryAccountLogon + 1) * sizeof(ULONG));
//     if (AuditOptions == NULL)
//     {
//         DPRINT1("Failed to allocate the auditiing options array!\n");
//         goto done;
//     }
//
//     AuditInfo.AuditingMode = TRUE;
//     AuditInfo.EventAuditingOptions = AuditOptions;
//     AuditInfo.MaximumAuditEventCount = AuditCategoryAccountLogon + 1;
//
//     do
//     {
//         /* Retrieve the group name */
//         if (!SetupGetStringFieldW(&InfContext,
//                                   0,
//                                   szOptionName,
//                                   ARRAYSIZE!(szOptionName),
//                                   NULL))
//         {
//             DPRINT1("SetupGetStringFieldW() failed\n");
//             continue;
//         }
//
//         DPRINT("Option: '%S'\n", szOptionName);
//
//         if (!SetupGetIntField(&InfContext,
//                               1,
//                               &nValue))
//         {
//             DPRINT1("SetupGetStringFieldW() failed\n");
//             continue;
//         }
//
//         DPRINT("Value: %d\n", nValue);
//
//         if ((nValue < POLICY_AUDIT_EVENT_UNCHANGED) || (nValue > POLICY_AUDIT_EVENT_NONE))
//         {
//             DPRINT1("Invalid audit option!\n");
//             continue;
//         }
//
//         if (_wcsicmp(szOptionName, L"AuditSystemEvents") == 0)
//         {
//             AuditOptions[AuditCategorySystem] = (ULONG)nValue;
//         }
//         else if (_wcsicmp(szOptionName, L"AuditLogonEvents") == 0)
//         {
//             AuditOptions[AuditCategoryLogon] = (ULONG)nValue;
//         }
//         else if (_wcsicmp(szOptionName, L"AuditObjectAccess") == 0)
//         {
//             AuditOptions[AuditCategoryObjectAccess] = (ULONG)nValue;
//         }
//         else if (_wcsicmp(szOptionName, L"AuditPrivilegeUse") == 0)
//         {
//             AuditOptions[AuditCategoryPrivilegeUse] = (ULONG)nValue;
//         }
//         else if (_wcsicmp(szOptionName, L"AuditProcessTracking") == 0)
//         {
//             AuditOptions[AuditCategoryDetailedTracking] = (ULONG)nValue;
//         }
//         else if (_wcsicmp(szOptionName, L"AuditPolicyChange") == 0)
//         {
//             AuditOptions[AuditCategoryPolicyChange] = (ULONG)nValue;
//         }
//         else if (_wcsicmp(szOptionName, L"AuditAccountManage") == 0)
//         {
//             AuditOptions[AuditCategoryAccountManagement] = (ULONG)nValue;
//         }
//         else if (_wcsicmp(szOptionName, L"AuditDSAccess") == 0)
//         {
//             AuditOptions[AuditCategoryDirectoryServiceAccess] = (ULONG)nValue;
//         }
//         else if (_wcsicmp(szOptionName, L"AuditAccountLogon") == 0)
//         {
//             AuditOptions[AuditCategoryAccountLogon] = (ULONG)nValue;
//         }
//         else
//         {
//             DPRINT1("Invalid auditing option '%S'\n", szOptionName);
//         }
//     }
//     while (SetupFindNextLine(&InfContext, &InfContext));
//
//     Status = LsaSetInformationPolicy(PolicyHandle,
//                                      PolicyAuditEventsInformation,
//                                      (PVOID)&AuditInfo);
//     if (Status != STATUS_SUCCESS)
//     {
//         DPRINT1("LsaSetInformationPolicy() failed (Status 0x%08lx)\n", Status);
//     }
//
// done:
//     if (AuditOptions != NULL)
//         HeapFree(GetProcessHeap(), 0, AuditOptions);
//
//     if (PolicyHandle != NULL)
//         LsaClose(PolicyHandle);
// }
//
//
// VOID
// InstallSecurity(VOID)
// {
//     HINF hSecurityInf;
//     PWSTR pszSecurityInf;
//
// //    if (IsServer())
// //        pszSecurityInf = L"defltsv.inf";
// //    else
//         pszSecurityInf = L"defltwk.inf";
//
//     InstallBuiltinAccounts();
//
//     hSecurityInf = SetupOpenInfFileW(pszSecurityInf,
//                                      NULL,
//                                      INF_STYLE_WIN4,
//                                      NULL);
//     if (hSecurityInf != INVALID_HANDLE_VALUE)
//     {
//         InstallPrivileges(hSecurityInf);
//         ApplyRegistryValues(hSecurityInf);
//
//         ApplyEventlogSettings(hSecurityInf, L"Application Log", L"Application");
//         ApplyEventlogSettings(hSecurityInf, L"Security Log", L"Security");
//         ApplyEventlogSettings(hSecurityInf, L"System Log", L"System");
//
//         ApplyPasswordSettings(hSecurityInf, L"System Access");
//         ApplyLockoutSettings(hSecurityInf, L"System Access");
//         ApplyAccountSettings(hSecurityInf, L"System Access");
//
//         ApplyAuditEvents(hSecurityInf);
//
//         SetupCloseInfFile(hSecurityInf);
//     }
//
//     /* Hack */
//     SetPrimaryDomain(L"WORKGROUP", NULL);
// }
//
//
// NTSTATUS
// SetAdministratorPassword(LPCWSTR Password)
// {
//     PPOLICY_ACCOUNT_DOMAIN_INFO OrigInfo = NULL;
//     PUSER_ACCOUNT_NAME_INFORMATION AccountNameInfo = NULL;
//     USER_SET_PASSWORD_INFORMATION PasswordInfo;
//     LSA_OBJECT_ATTRIBUTES ObjectAttributes;
//     LSA_HANDLE PolicyHandle = NULL;
//     SAM_HANDLE ServerHandle = NULL;
//     SAM_HANDLE DomainHandle = NULL;
//     SAM_HANDLE UserHandle = NULL;
//     NTSTATUS Status;
//
//     DPRINT("SYSSETUP: SetAdministratorPassword(%p)\n", Password);
//
//     memset(&ObjectAttributes, 0, sizeof(LSA_OBJECT_ATTRIBUTES));
//     ObjectAttributes.Length = sizeof(LSA_OBJECT_ATTRIBUTES);
//
//     Status = LsaOpenPolicy(NULL,
//                            &ObjectAttributes,
//                            POLICY_VIEW_LOCAL_INFORMATION | POLICY_TRUST_ADMIN,
//                            &PolicyHandle);
//     if (Status != STATUS_SUCCESS)
//     {
//         DPRINT1("LsaOpenPolicy() failed (Status: 0x%08lx)\n", Status);
//         return Status;
//     }
//
//     Status = LsaQueryInformationPolicy(PolicyHandle,
//                                        PolicyAccountDomainInformation,
//                                        (PVOID *)&OrigInfo);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("LsaQueryInformationPolicy() failed (Status: 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     Status = SamConnect(NULL,
//                         &ServerHandle,
//                         SAM_SERVER_CONNECT | SAM_SERVER_LOOKUP_DOMAIN,
//                         NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("SamConnect() failed (Status: 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     Status = SamOpenDomain(ServerHandle,
//                            DOMAIN_LOOKUP,
//                            OrigInfo->DomainSid,
//                            &DomainHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("SamOpenDomain() failed (Status: 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     Status = SamOpenUser(DomainHandle,
//                          USER_FORCE_PASSWORD_CHANGE | USER_READ_GENERAL,
//                          DOMAIN_USER_RID_ADMIN,
//                          &UserHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("SamOpenUser() failed (Status %08lx)\n", Status);
//         goto done;
//     }
//
//     RtlInitUnicodeString(&PasswordInfo.Password, Password);
//     PasswordInfo.PasswordExpired = FALSE;
//
//     Status = SamSetInformationUser(UserHandle,
//                                    UserSetPasswordInformation,
//                                    &PasswordInfo);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("SamSetInformationUser() failed (Status %08lx)\n", Status);
//         goto done;
//     }
//
//     Status = SamQueryInformationUser(UserHandle,
//                                      UserAccountNameInformation,
//                                      (PVOID*)&AccountNameInfo);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("SamQueryInformationUser() failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     AdminInfo.Name = RtlAllocateHeap(RtlGetProcessHeap(),
//                                      HEAP_ZERO_MEMORY,
//                                      AccountNameInfo->UserName.Length + sizeof(WCHAR));
//     if (AdminInfo.Name != NULL)
//         RtlCopyMemory!(AdminInfo.Name,
//                       AccountNameInfo->UserName.Buffer,
//                       AccountNameInfo->UserName.Length);
//
//     AdminInfo.Domain = RtlAllocateHeap(RtlGetProcessHeap(),
//                                        HEAP_ZERO_MEMORY,
//                                        OrigInfo->DomainName.Length + sizeof(WCHAR));
//     if (AdminInfo.Domain != NULL)
//         RtlCopyMemory!(AdminInfo.Domain,
//                       OrigInfo->DomainName.Buffer,
//                       OrigInfo->DomainName.Length);
//
//     AdminInfo.Password = RtlAllocateHeap(RtlGetProcessHeap(),
//                                          0,
//                                          (wcslen(Password) + 1) * sizeof(WCHAR));
//     if (AdminInfo.Password != NULL)
//         wcscpy(AdminInfo.Password, Password);
//
//     DPRINT("Administrator Name: %S\n", AdminInfo.Name);
//     DPRINT("Administrator Domain: %S\n", AdminInfo.Domain);
//     DPRINT("Administrator Password: %S\n", AdminInfo.Password);
//
// done:
//     if (AccountNameInfo != NULL)
//         SamFreeMemory(AccountNameInfo);
//
//     if (OrigInfo != NULL)
//         LsaFreeMemory(OrigInfo);
//
//     if (PolicyHandle != NULL)
//         LsaClose(PolicyHandle);
//
//     if (UserHandle != NULL)
//         SamCloseHandle(UserHandle);
//
//     if (DomainHandle != NULL)
//         SamCloseHandle(DomainHandle);
//
//     if (ServerHandle != NULL)
//         SamCloseHandle(ServerHandle);
//
//     DPRINT1("SYSSETUP: SetAdministratorPassword() done (Status %08lx)\n", Status);
//
//     return Status;
// }
//
//
// VOID
// SetAutoAdminLogon(VOID)
// {
//     WCHAR szAutoAdminLogon[2];
//     HKEY hKey = NULL;
//     DWORD dwType;
//     DWORD dwSize;
//     LONG lError;
//
//     lError = RegOpenKeyExW(HKEY_LOCAL_MACHINE,
//                            L"SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\Winlogon",
//                            0,
//                            KEY_READ | KEY_WRITE,
//                            &hKey);
//     if (lError != ERROR_SUCCESS)
//         return;
//
//     dwSize = 2 * sizeof(WCHAR);
//     lError = RegQueryValueExW(hKey,
//                               L"AutoAdminLogon",
//                               NULL,
//                               &dwType,
//                               (LPBYTE)szAutoAdminLogon,
//                               &dwSize);
//     if (lError != ERROR_SUCCESS)
//         goto done;
//
//     if (wcscmp(szAutoAdminLogon, L"1") == 0)
//     {
//         RegSetValueExW(hKey,
//                        L"DefaultDomainName",
//                        0,
//                        REG_SZ,
//                        (LPBYTE)AdminInfo.Domain,
//                        (wcslen(AdminInfo.Domain) + 1) * sizeof(WCHAR));
//
//         RegSetValueExW(hKey,
//                        L"DefaultUserName",
//                        0,
//                        REG_SZ,
//                        (LPBYTE)AdminInfo.Name,
//                        (wcslen(AdminInfo.Name) + 1) * sizeof(WCHAR));
//
//         RegSetValueExW(hKey,
//                        L"DefaultPassword",
//                        0,
//                        REG_SZ,
//                        (LPBYTE)AdminInfo.Password,
//                        (wcslen(AdminInfo.Password) + 1) * sizeof(WCHAR));
//     }
//
// done:
//     if (hKey != NULL)
//         RegCloseKey(hKey);
// }
//
//
// /* EOF */
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
