use crate::prelude::*;
// The crate's own functions: the definition is the declaration.
use crate::reg_reg::RegCloseKey;
use crate::reg_reg::RegOpenKeyExW;
use core::ffi::c_void;

// w! — NUL-terminated UTF-16 array from an ASCII literal (2-aligned 'static;
// a b"...\0" byte string cast to *const u16 is both ANSI-not-wide and
// potentially odd-aligned — see field manual / basesrv init.rs CSDVersion).
macro_rules! w {
    ($s:literal) => {{
        const __W: &[u16] = &{
            const BYTES: &[u8] = $s.as_bytes();
            let mut out = [0u16; { BYTES.len() + 1 }];
            let mut i = 0;
            while i < BYTES.len() {
                out[i] = BYTES[i] as u16;
                i += 1;
            }
            out
        };
        __W
    }};
}


extern "system" {
    fn IsHKCRKey(hKey: HKEY) -> BOOL;
}

// ---- file-scope preamble (translated) ----
// C had `static const UNICODE_STRING HKLM_ClassesPath`. The struct holds a raw
// `*mut u16` Buffer (not Sync), so wrap in a transparent Sync newtype to keep it a
// real `static` (immutable, matching the C `const`). Use sites read `.0`.
#[repr(transparent)]
struct SyncUnicodeString(_UNICODE_STRING);
unsafe impl Sync for SyncUnicodeString {}
pub static HKLM_ClassesPath: SyncUnicodeString = SyncUnicodeString(_UNICODE_STRING {
    Length: 64,
    MaximumLength: 66,
    Buffer: w!("\\Registry\\Machine\\Software\\Classes").as_ptr() as *mut u16,
});

unsafe fn ValueExists(hNormalKey: HKEY, Name: PUNICODE_STRING) -> BOOL {
    let mut kvi: KEY_VALUE_PARTIAL_INFORMATION = core::mem::zeroed();
    let mut total_size: ULONG = core::mem::size_of::<KEY_VALUE_PARTIAL_INFORMATION>() as ULONG;
    let status = NtQueryValueKey(
        hNormalKey as *mut _,
        Name,
        KeyValuePartialInformation,
        core::ptr::addr_of_mut!(kvi) as *mut _,
        total_size,
        core::ptr::addr_of_mut!(total_size),
    );
    (status != STATUS_OBJECT_NAME_NOT_FOUND) as BOOL
}

unsafe fn GetKeyName(hKey: HKEY, KeyName: PUNICODE_STRING) -> LONG {
    let mut InfoName: _UNICODE_STRING = core::mem::zeroed();
    let NameInformation: PKEY_NAME_INFORMATION;
    let mut InfoLength: ULONG;
    let mut Status: NTSTATUS;

    InfoLength = 0;
    Status = NtQueryKey(
        hKey as _,
        KeyNameInformation,
        core::ptr::null_mut(),
        0,
        &mut InfoLength,
    );
    if Status != STATUS_BUFFER_TOO_SMALL {
        return RtlNtStatusToDosError(Status) as LONG;
    }

    NameInformation =
        RtlAllocateHeap(RtlGetProcessHeap(), 0, InfoLength as SIZE_T) as PKEY_NAME_INFORMATION;
    if NameInformation.is_null() {
        return ERROR_NOT_ENOUGH_MEMORY as LONG;
    }

    Status = NtQueryKey(
        hKey as _,
        KeyNameInformation,
        NameInformation as _,
        InfoLength,
        &mut InfoLength,
    );
    if !NT_SUCCESS(Status) {
        RtlFreeHeap(RtlGetProcessHeap(), 0, NameInformation as _);
        return RtlNtStatusToDosError(Status) as LONG;
    }

    InfoName.Length = (*NameInformation).NameLength as u16;
    InfoName.MaximumLength = (*NameInformation).NameLength as u16;
    InfoName.Buffer = core::ptr::addr_of_mut!((*NameInformation).Name) as *mut _;

    Status = RtlDuplicateUnicodeString(
        RTL_DUPLICATE_UNICODE_STRING_NULL_TERMINATE as ULONG,
        &mut InfoName,
        KeyName,
    );
    if !NT_SUCCESS(Status) {
        RtlFreeHeap(RtlGetProcessHeap(), 0, NameInformation as _);
        return RtlNtStatusToDosError(Status) as LONG;
    }

    RtlFreeHeap(RtlGetProcessHeap(), 0, NameInformation as _);

    ERROR_SUCCESS as LONG
}

unsafe fn GetKeySam(hKey: HKEY, RegSam: *mut REGSAM) -> LONG {
    let mut ObjectInfo: OBJECT_BASIC_INFORMATION = core::mem::zeroed();

    let Status = NtQueryObject(
        hKey as HANDLE,
        _OBJECT_INFORMATION_CLASS_ObjectBasicInformation,
        core::ptr::addr_of_mut!(ObjectInfo) as *mut _,
        core::mem::size_of::<OBJECT_BASIC_INFORMATION>() as u32,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        return RtlNtStatusToDosError(Status) as LONG;
    }

    *RegSam = ObjectInfo.GrantedAccess;
    ERROR_SUCCESS as LONG
}

unsafe fn GetFallbackHKCRKey(
    hKey: HKEY,
    MachineKey: *mut HKEY,
    MustCreate: BOOL,
) -> LONG {
    let mut KeyName: _UNICODE_STRING = core::mem::zeroed();
    let mut SubKeyName: LPWSTR;
    let mut ErrorCode: LONG;
    let mut SamDesired: REGSAM = 0;

    /* Get the key name */
    ErrorCode = GetKeyName(hKey, &mut KeyName);
    if ErrorCode != ERROR_SUCCESS as LONG {
        *MachineKey = hKey;
        return ErrorCode;
    }

    /* See if we really need a conversion */
    if RtlPrefixUnicodeString(&HKLM_ClassesPath.0, &KeyName, 1u8) != 0 {
        RtlFreeUnicodeString(&mut KeyName);
        *MachineKey = hKey;
        return ERROR_SUCCESS as LONG;
    }

    SubKeyName = KeyName.Buffer.add(15); /* 15 == wcslen(L"\\Registry\\User\\") */
    /* Skip the user token */
    loop {
        let ch = *SubKeyName;
        SubKeyName = SubKeyName.add(1);
        if ch == '\\' as u16 {
            break;
        }
        if *SubKeyName == 0 {
            ERR!(
                "Key name %S is invalid!\n",
                KeyName.Buffer,
            );
            return ERROR_INTERNAL_ERROR as LONG;
        }
    }

    /* Use the same access mask than the original key */
    ErrorCode = GetKeySam(hKey, &mut SamDesired);
    if ErrorCode != ERROR_SUCCESS as LONG {
        RtlFreeUnicodeString(&mut KeyName);
        *MachineKey = hKey;
        return ErrorCode;
    }

    if MustCreate != 0 {
        ErrorCode = RegCreateKeyExW(
            HKEY_LOCAL_MACHINE,
            SubKeyName,
            0,
            core::ptr::null_mut(),
            0,
            SamDesired,
            core::ptr::null_mut(),
            MachineKey,
            core::ptr::null_mut(),
        );
    } else {
        /* Open the key. */
        ErrorCode = RegOpenKeyExW(
            HKEY_LOCAL_MACHINE,
            SubKeyName,
            0,
            SamDesired,
            MachineKey,
        );
    }

    RtlFreeUnicodeString(&mut KeyName);

    ErrorCode
}

unsafe fn GetPreferredHKCRKey(hKey: HKEY, PreferredKey: *mut HKEY) -> LONG {
    let mut KeyName: _UNICODE_STRING = core::mem::zeroed();
    let SubKeyName: LPWSTR;
    let mut ErrorCode: LONG;
    let mut SamDesired: REGSAM = 0;

    /* Get the key name */
    ErrorCode = GetKeyName(hKey, &mut KeyName);
    if ErrorCode != ERROR_SUCCESS as LONG {
        *PreferredKey = hKey;
        return ErrorCode;
    }

    /* See if we really need a conversion */
    if RtlPrefixUnicodeString(&HKLM_ClassesPath.0, &mut KeyName, 1u8) == 0 {
        RtlFreeUnicodeString(&mut KeyName);
        *PreferredKey = hKey;
        return ERROR_SUCCESS as LONG;
    }

    /* 18 == wcslen(L"\\Registry\\Machine\\") */
    SubKeyName = KeyName.Buffer.add(18);

    /* Use the same access mask than the original key */
    ErrorCode = GetKeySam(hKey, &mut SamDesired);
    if ErrorCode != ERROR_SUCCESS as LONG {
        RtlFreeUnicodeString(&mut KeyName);
        *PreferredKey = hKey;
        return ErrorCode;
    }

    /* Open the key. */
    ErrorCode = RegOpenKeyExW(
        HKEY_CURRENT_USER,
        SubKeyName,
        0,
        SamDesired,
        PreferredKey,
    );

    RtlFreeUnicodeString(&mut KeyName);

    ErrorCode
}

unsafe fn GetSubkeyInfoHelper(
    mut hKey: HKEY,
    lpSubKeys: LPDWORD,
    lpMaxSubKeyLen: LPDWORD,
) -> LONG {
    let mut err: LONG = RegQueryInfoKeyW(
        hKey,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        lpSubKeys,
        lpMaxSubKeyLen,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    if err != ERROR_ACCESS_DENIED as LONG {
        return err;
    }

    if RegOpenKeyExW(
        hKey,
        core::ptr::null_mut(),
        0,
        KEY_QUERY_VALUE,
        &mut hKey,
    ) == ERROR_SUCCESS as LONG
    {
        err = RegQueryInfoKeyW(
            hKey,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            lpSubKeys,
            lpMaxSubKeyLen,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
        RegCloseKey(hKey);
    }
    err
}

#[no_mangle]
pub unsafe extern "system" fn CreateHKCRKey(
    mut hKey: HKEY,
    lpSubKey: LPCWSTR,
    Reserved: DWORD,
    lpClass: LPWSTR,
    dwOptions: DWORD,
    samDesired: REGSAM,
    lpSecurityAttributes: LPSECURITY_ATTRIBUTES,
    phkResult: PHKEY,
    lpdwDisposition: LPDWORD,
) -> LONG {
    let mut ErrorCode: LONG;
    let mut QueriedKey: HKEY = core::ptr::null_mut();
    let mut TestKey: HKEY = core::ptr::null_mut();

    /* Remove the HKCR flag while we're working */
    hKey = ((hKey as ULONG_PTR) & !0x2) as HKEY;

    ErrorCode = GetPreferredHKCRKey(hKey, &mut QueriedKey);

    if ErrorCode == ERROR_FILE_NOT_FOUND as LONG {
        /* The current key doesn't exist on HKCU side, so we can only create it in HKLM */
        ErrorCode = RegCreateKeyExW(
            hKey,
            lpSubKey,
            Reserved,
            lpClass,
            dwOptions,
            samDesired,
            lpSecurityAttributes,
            phkResult,
            lpdwDisposition,
        );
        if ErrorCode == ERROR_SUCCESS as LONG {
            MakeHKCRKey(phkResult);
        }
        return ErrorCode;
    }

    if ErrorCode != ERROR_SUCCESS as LONG {
        /* Somehow we failed for another reason (maybe deleted key or whatever) */
        return ErrorCode;
    }

    /* See if the subkey already exists in HKCU. */
    ErrorCode = RegOpenKeyExW(QueriedKey, lpSubKey, 0, READ_CONTROL, &mut TestKey);
    if ErrorCode != ERROR_FILE_NOT_FOUND as LONG {
        if ErrorCode == ERROR_SUCCESS as LONG {
            /* Great. Close the test one and do the real create operation */
            RegCloseKey(TestKey);
            ErrorCode = RegCreateKeyExW(
                QueriedKey,
                lpSubKey,
                Reserved,
                lpClass,
                dwOptions,
                samDesired,
                lpSecurityAttributes,
                phkResult,
                lpdwDisposition,
            );
            if ErrorCode == ERROR_SUCCESS as LONG {
                MakeHKCRKey(phkResult);
            }
        }
        if QueriedKey != hKey {
            RegCloseKey(QueriedKey);
        }

        return ERROR_SUCCESS as LONG;
    }

    if QueriedKey != hKey {
        RegCloseKey(QueriedKey);
    }

    /* So we must do the create operation in HKLM, creating the missing parent keys if needed. */
    ErrorCode = GetFallbackHKCRKey(hKey, &mut QueriedKey, TRUE as BOOL);
    if ErrorCode != ERROR_SUCCESS as LONG {
        return ErrorCode;
    }

    /* Do the key creation */
    ErrorCode = RegCreateKeyExW(
        QueriedKey,
        lpSubKey,
        Reserved,
        lpClass,
        dwOptions,
        samDesired,
        lpSecurityAttributes,
        phkResult,
        lpdwDisposition,
    );

    if QueriedKey != hKey {
        RegCloseKey(QueriedKey);
    }

    if ErrorCode == ERROR_SUCCESS as LONG {
        MakeHKCRKey(phkResult);
    }

    ErrorCode
}

#[no_mangle]
pub unsafe extern "system" fn OpenHKCRKey(
    mut hKey: HKEY,
    lpSubKey: LPCWSTR,
    ulOptions: DWORD,
    samDesired: REGSAM,
    phkResult: PHKEY,
) -> LONG {
    let mut QueriedKey: HKEY = core::ptr::null_mut();
    let mut ErrorCode: LONG;

    /* Remove the HKCR flag while we're working */
    hKey = ((hKey as ULONG_PTR) & !0x2) as HKEY;

    ErrorCode = GetPreferredHKCRKey(hKey, &mut QueriedKey);

    if ErrorCode == ERROR_FILE_NOT_FOUND as LONG {
        /* The key doesn't exist on HKCU side, no chance for a subkey */
        ErrorCode = RegOpenKeyExW(hKey, lpSubKey, ulOptions, samDesired, phkResult);
        if ErrorCode == ERROR_SUCCESS as LONG {
            MakeHKCRKey(phkResult);
        }
        return ErrorCode;
    }

    if ErrorCode != ERROR_SUCCESS as LONG {
        /* Somehow we failed for another reason (maybe deleted key or whatever) */
        return ErrorCode;
    }

    /* Try on the HKCU side */
    ErrorCode = RegOpenKeyExW(QueriedKey, lpSubKey, ulOptions, samDesired, phkResult);
    if ErrorCode == ERROR_SUCCESS as LONG {
        MakeHKCRKey(phkResult);
    }

    /* Close it if we must */
    if QueriedKey != hKey {
        RegCloseKey(QueriedKey);
    }

    /* Anything else than ERROR_FILE_NOT_FOUND means that we found it, even if it is with failures. */
    if ErrorCode != ERROR_FILE_NOT_FOUND as LONG {
        return ErrorCode;
    }

    /* If we're here, we must open from HKLM key. */
    ErrorCode = GetFallbackHKCRKey(hKey, &mut QueriedKey, FALSE as BOOL);
    if ErrorCode != ERROR_SUCCESS as LONG {
        /* Maybe the key doesn't exist in the HKLM view */
        return ErrorCode;
    }

    ErrorCode = RegOpenKeyExW(QueriedKey, lpSubKey, ulOptions, samDesired, phkResult);
    if ErrorCode == ERROR_SUCCESS as LONG {
        MakeHKCRKey(phkResult);
    }

    /* Close it if we must */
    if QueriedKey != hKey {
        RegCloseKey(QueriedKey);
    }

    ErrorCode
}

#[no_mangle]
pub unsafe extern "system" fn DeleteHKCRKey(
    mut hKey: HKEY,
    lpSubKey: LPCWSTR,
    RegSam: REGSAM,
    Reserved: DWORD,
) -> LONG {
    let mut QueriedKey: HKEY = core::ptr::null_mut();
    let mut ErrorCode: LONG;

    /* Remove the HKCR flag while we're working */
    hKey = ((hKey as ULONG_PTR) & !0x2) as HKEY;

    ErrorCode = GetPreferredHKCRKey(hKey, &mut QueriedKey);

    if ErrorCode == ERROR_FILE_NOT_FOUND as LONG {
        /* The key doesn't exist on HKCU side, no chance for a subkey */
        return RegDeleteKeyExW(hKey, lpSubKey, RegSam, Reserved);
    }

    if ErrorCode != ERROR_SUCCESS as LONG {
        /* Somehow we failed for another reason (maybe deleted key or whatever) */
        return ErrorCode;
    }

    ErrorCode = RegDeleteKeyExW(QueriedKey, lpSubKey, RegSam, Reserved);

    /* Close it if we must */
    if QueriedKey != hKey {
        /* The original key is on the machine view */
        RegCloseKey(QueriedKey);
    }

    /* Anything else than ERROR_FILE_NOT_FOUND means that we found it, even if it is with failures. */
    if ErrorCode != ERROR_FILE_NOT_FOUND as LONG {
        return ErrorCode;
    }

    /* If we're here, we must open from HKLM key. */
    ErrorCode = GetFallbackHKCRKey(hKey, &mut QueriedKey, FALSE as BOOL);
    if ErrorCode != ERROR_SUCCESS as LONG {
        /* Maybe the key doesn't exist in the HKLM view */
        return ErrorCode;
    }

    ErrorCode = RegDeleteKeyExW(QueriedKey, lpSubKey, RegSam, Reserved);

    /* Close it if we must */
    if QueriedKey != hKey {
        RegCloseKey(QueriedKey);
    }

    ErrorCode
}

#[no_mangle]
pub unsafe extern "system" fn DeleteHKCRValue(
    mut hKey: HKEY,
    ValueName: PUNICODE_STRING,
) -> LONG {
    let mut hActualKey: HKEY = core::ptr::null_mut();
    let mut ErrorCode: LONG;

    /* Remove the HKCR flag while we're working */
    hKey = ((hKey as ULONG_PTR) & !0x2) as HKEY;

    /* Does the HKCU key and value exist? */
    ErrorCode = GetPreferredHKCRKey(hKey, &mut hActualKey);
    if ErrorCode == ERROR_SUCCESS as LONG {
        if ValueExists(hActualKey, ValueName) == 0 {
            if hActualKey != hKey {
                RegCloseKey(hActualKey);
            }
            ErrorCode = ERROR_FILE_NOT_FOUND as LONG;
        }
    }
    if ErrorCode == ERROR_FILE_NOT_FOUND as LONG {
        ErrorCode = GetFallbackHKCRKey(hKey, &mut hActualKey, 0);
    }

    if ErrorCode == ERROR_SUCCESS as LONG {
        let Status: NTSTATUS = NtDeleteValueKey(hActualKey as HANDLE, ValueName);
        ErrorCode = if NT_SUCCESS(Status) {
            ERROR_SUCCESS as LONG
        } else {
            RtlNtStatusToDosError(Status) as LONG
        };
    }
    if hActualKey != hKey {
        RegCloseKey(hActualKey);
    }
    ErrorCode
}

#[no_mangle]
pub unsafe extern "system" fn QueryHKCRValue(
    mut hKey: HKEY,
    Name: LPCWSTR,
    Reserved: LPDWORD,
    Type: LPDWORD,
    Data: LPBYTE,
    Count: LPDWORD,
) -> LONG {
    let mut QueriedKey: HKEY = core::ptr::null_mut();
    let mut ErrorCode: LONG;

    /* Remove the HKCR flag while we're working */
    hKey = ((hKey as ULONG_PTR) & !0x2) as HKEY;

    ErrorCode = GetPreferredHKCRKey(hKey, &mut QueriedKey);

    if ErrorCode == ERROR_FILE_NOT_FOUND as LONG {
        /* The key doesn't exist on HKCU side, no chance for a value in it */
        return RegQueryValueExW(hKey, Name, Reserved, Type, Data, Count);
    }

    if ErrorCode != ERROR_SUCCESS as LONG {
        /* Somehow we failed for another reason (maybe deleted key or whatever) */
        return ErrorCode;
    }

    ErrorCode = RegQueryValueExW(QueriedKey, Name, Reserved, Type, Data, Count);

    /* Close it if we must */
    if QueriedKey != hKey {
        /* The original key is on the machine view */
        RegCloseKey(QueriedKey);
    }

    /* Anything else than ERROR_FILE_NOT_FOUND means that we found it, even if it is with failures. */
    if ErrorCode != ERROR_FILE_NOT_FOUND as LONG {
        return ErrorCode;
    }

    /* If we're here, we must open from HKLM key. */
    ErrorCode = GetFallbackHKCRKey(hKey, &mut QueriedKey, FALSE as BOOL);
    if ErrorCode != ERROR_SUCCESS as LONG {
        /* Maybe the key doesn't exist in the HKLM view */
        return ErrorCode;
    }

    ErrorCode = RegQueryValueExW(QueriedKey, Name, Reserved, Type, Data, Count);

    /* Close it if we must */
    if QueriedKey != hKey {
        RegCloseKey(QueriedKey);
    }

    ErrorCode
}

#[no_mangle]
pub unsafe extern "system" fn SetHKCRValue(
    mut hKey: HKEY,
    Name: LPCWSTR,
    Reserved: DWORD,
    Type: DWORD,
    Data: *const BYTE,
    DataSize: DWORD,
) -> LONG {
    let mut QueriedKey: HKEY = core::ptr::null_mut();
    let mut ErrorCode: LONG;

    /* Remove the HKCR flag while we're working */
    hKey = ((hKey as ULONG_PTR) & !0x2) as HKEY;

    ErrorCode = GetPreferredHKCRKey(hKey, &mut QueriedKey);

    if ErrorCode == ERROR_FILE_NOT_FOUND as LONG {
        /* The key doesn't exist on HKCU side, no chance to put a value in it */
        return RegSetValueExW(hKey, Name, Reserved, Type, Data, DataSize);
    }

    if ErrorCode != ERROR_SUCCESS as LONG {
        /* Somehow we failed for another reason (maybe deleted key or whatever) */
        return ErrorCode;
    }

    /* Check if the value already exists in the preferred key */
    ErrorCode = RegQueryValueExW(
        QueriedKey,
        Name,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    if ErrorCode != ERROR_FILE_NOT_FOUND as LONG {
        if ErrorCode == ERROR_SUCCESS as LONG {
            /* Yes, so we have the right to modify it */
            ErrorCode = RegSetValueExW(QueriedKey, Name, Reserved, Type, Data, DataSize);
        }
        if QueriedKey != hKey {
            RegCloseKey(QueriedKey);
        }
        return ErrorCode;
    }
    if QueriedKey != hKey {
        RegCloseKey(QueriedKey);
    }

    /* So we must set the value in the HKLM version */
    ErrorCode = GetPreferredHKCRKey(hKey, &mut QueriedKey);
    if ErrorCode == ERROR_FILE_NOT_FOUND as LONG {
        /* No choice: put this in HKCU */
        return RegSetValueExW(hKey, Name, Reserved, Type, Data, DataSize);
    } else if ErrorCode != ERROR_SUCCESS as LONG {
        return ErrorCode;
    }

    ErrorCode = RegSetValueExW(QueriedKey, Name, Reserved, Type, Data, DataSize);

    if QueriedKey != hKey {
        RegCloseKey(QueriedKey);
    }

    ErrorCode
}

#[no_mangle]
pub unsafe extern "system" fn EnumHKCRKey(
    mut hKey: HKEY,
    mut dwIndex: DWORD,
    lpName: LPWSTR,
    lpcbName: LPDWORD,
    lpReserved: LPDWORD,
    lpClass: LPWSTR,
    lpcbClass: LPDWORD,
    lpftLastWriteTime: PFILETIME,
) -> LONG {
    let mut PreferredKey: HKEY = core::ptr::null_mut();
    let mut FallbackKey: HKEY = core::ptr::null_mut();
    let mut NumPreferredSubKeys: DWORD = 0;
    let mut MaxFallbackSubKeyLen: DWORD = 0;
    let mut FallbackIndex: DWORD;
    let mut FallbackSubKeyName: *mut WCHAR = core::ptr::null_mut();
    let mut ErrorCode: LONG;

    /* Remove the HKCR flag while we're working */
    hKey = ((hKey as ULONG_PTR) & !0x2) as HKEY;

    /* Get the preferred key */
    ErrorCode = GetPreferredHKCRKey(hKey, &mut PreferredKey);
    if ErrorCode != ERROR_SUCCESS as LONG {
        if ErrorCode == ERROR_FILE_NOT_FOUND as LONG {
            return RegEnumKeyExW(
                hKey,
                dwIndex,
                lpName,
                lpcbName,
                lpReserved,
                lpClass,
                lpcbClass,
                lpftLastWriteTime,
            );
        }
        return ErrorCode;
    }

    /* Get the fallback key */
    ErrorCode = GetFallbackHKCRKey(hKey, &mut FallbackKey, FALSE as BOOL);
    if ErrorCode != ERROR_SUCCESS as LONG {
        if PreferredKey != hKey {
            RegCloseKey(PreferredKey);
        }
        if ErrorCode == ERROR_FILE_NOT_FOUND as LONG {
            return RegEnumKeyExW(
                hKey,
                dwIndex,
                lpName,
                lpcbName,
                lpReserved,
                lpClass,
                lpcbClass,
                lpftLastWriteTime,
            );
        }
        return ErrorCode;
    }

    'exit: {
        /* Get some info on the HKCU side */
        ErrorCode = GetSubkeyInfoHelper(PreferredKey, &mut NumPreferredSubKeys, core::ptr::null_mut());
        if ErrorCode != ERROR_SUCCESS as LONG {
            break 'exit;
        }

        if dwIndex < NumPreferredSubKeys {
            ErrorCode = RegEnumKeyExW(
                PreferredKey,
                dwIndex,
                lpName,
                lpcbName,
                lpReserved,
                lpClass,
                lpcbClass,
                lpftLastWriteTime,
            );
            break 'exit;
        }

        /* Squash out the indices from HKCU */
        dwIndex -= NumPreferredSubKeys;

        /* Get some info */
        ErrorCode = GetSubkeyInfoHelper(FallbackKey, core::ptr::null_mut(), &mut MaxFallbackSubKeyLen);
        if ErrorCode != ERROR_SUCCESS as LONG {
            break 'exit;
        }

        MaxFallbackSubKeyLen += 1;

        /* Allocate our buffer */
        FallbackSubKeyName = RtlAllocateHeap(
            RtlGetProcessHeap(),
            0,
            ((MaxFallbackSubKeyLen as usize) * core::mem::size_of::<WCHAR>()) as SIZE_T,
        ) as *mut WCHAR;
        if FallbackSubKeyName.is_null() {
            ErrorCode = ERROR_NOT_ENOUGH_MEMORY as LONG;
            break 'exit;
        }

        FallbackIndex = 0;
        loop {
            let mut PreferredSubKey: HKEY = core::ptr::null_mut();
            let mut FallbackSubkeyLen: DWORD = MaxFallbackSubKeyLen;

            /* Try enumerating */
            ErrorCode = RegEnumKeyExW(
                FallbackKey,
                FallbackIndex,
                FallbackSubKeyName,
                &mut FallbackSubkeyLen,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );
            if ErrorCode != ERROR_SUCCESS as LONG {
                break 'exit;
            }
            *FallbackSubKeyName.add(FallbackSubkeyLen as usize) = 0;

            /* See if there is such a value on HKCU side */
            ErrorCode = RegOpenKeyExW(
                PreferredKey,
                FallbackSubKeyName,
                0,
                READ_CONTROL,
                &mut PreferredSubKey,
            );

            if ErrorCode == ERROR_SUCCESS as LONG {
                RegCloseKey(PreferredSubKey);
                dwIndex += 1;
            } else if ErrorCode != ERROR_FILE_NOT_FOUND as LONG {
                break 'exit;
            }

            if FallbackIndex == dwIndex {
                break;
            }

            FallbackIndex += 1;
        }

        /* We can finally enumerate on the fallback side */
        ErrorCode = RegEnumKeyExW(
            FallbackKey,
            dwIndex,
            lpName,
            lpcbName,
            lpReserved,
            lpClass,
            lpcbClass,
            lpftLastWriteTime,
        );
    }

    if PreferredKey != hKey {
        RegCloseKey(PreferredKey);
    }
    if FallbackKey != hKey {
        RegCloseKey(FallbackKey);
    }
    if !FallbackSubKeyName.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, FallbackSubKeyName as PVOID);
    }

    ErrorCode
}
#[no_mangle]
pub unsafe extern "system" fn EnumHKCRValue(
    mut hKey: HKEY,
    mut dwIndex: DWORD,
    lpName: LPWSTR,
    lpcbName: PDWORD,
    lpReserved: PDWORD,
    lpdwType: PDWORD,
    lpData: LPBYTE,
    lpcbData: PDWORD,
) -> LONG {
    let PreferredKey: HKEY;
    let FallbackKey: HKEY;
    let mut NumPreferredValues: DWORD = 0;
    let mut MaxFallbackValueNameLen: DWORD = 0;
    let mut FallbackIndex: DWORD;
    let mut FallbackValueName: *mut WCHAR = core::ptr::null_mut();
    let mut ErrorCode: LONG;

    /* Remove the HKCR flag while we're working */
    hKey = ((hKey as ULONG_PTR) & !0x2) as HKEY;

    /* Get the preferred key */
    let mut PreferredKeyLocal: HKEY = core::ptr::null_mut();
    ErrorCode = GetPreferredHKCRKey(hKey, &mut PreferredKeyLocal);
    if ErrorCode != ERROR_SUCCESS as LONG {
        if ErrorCode == ERROR_FILE_NOT_FOUND as LONG {
            return RegEnumValueW(
                hKey,
                dwIndex,
                lpName,
                lpcbName,
                lpReserved,
                lpdwType,
                lpData,
                lpcbData,
            );
        }
        return ErrorCode;
    }
    PreferredKey = PreferredKeyLocal;

    /* Get the fallback key */
    let mut FallbackKeyLocal: HKEY = core::ptr::null_mut();
    ErrorCode = GetFallbackHKCRKey(hKey, &mut FallbackKeyLocal, 0);
    if ErrorCode != ERROR_SUCCESS as LONG {
        if PreferredKey != hKey {
            RegCloseKey(PreferredKey);
        }
        if ErrorCode == ERROR_FILE_NOT_FOUND as LONG {
            return RegEnumValueW(
                hKey,
                dwIndex,
                lpName,
                lpcbName,
                lpReserved,
                lpdwType,
                lpData,
                lpcbData,
            );
        }
        return ErrorCode;
    }
    FallbackKey = FallbackKeyLocal;

    'exit: loop {
        /* Get some info on the HKCU side */
        ErrorCode = RegQueryInfoKeyW(
            PreferredKey,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            &mut NumPreferredValues,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
        if ErrorCode != ERROR_SUCCESS as LONG {
            break 'exit;
        }

        if dwIndex < NumPreferredValues {
            return RegEnumValueW(
                PreferredKey,
                dwIndex,
                lpName,
                lpcbName,
                lpReserved,
                lpdwType,
                lpData,
                lpcbData,
            );
        }

        /* Squash out the indices from HKCU */
        dwIndex -= NumPreferredValues;

        /* Get some info */
        ErrorCode = RegQueryInfoKeyW(
            FallbackKey,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            &mut MaxFallbackValueNameLen,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
        if ErrorCode != ERROR_SUCCESS as LONG {
            ERR!("Could not query info of key %p (Err: %d)\n", FallbackKey, ErrorCode);
            break 'exit;
        }

        MaxFallbackValueNameLen += 1;
        TRACE!(b"Maxfallbacksubkeylen: %d\n\0".as_ptr() as *const i8, MaxFallbackValueNameLen);

        /* Allocate our buffer */
        FallbackValueName = RtlAllocateHeap(
            RtlGetProcessHeap(),
            0,
            ((MaxFallbackValueNameLen as usize) * core::mem::size_of::<WCHAR>()) as SIZE_T,
        ) as *mut WCHAR;
        if FallbackValueName.is_null() {
            ErrorCode = ERROR_NOT_ENOUGH_MEMORY as LONG;
            break 'exit;
        }

        FallbackIndex = 0;
        loop {
            let mut FallbackValueNameLen: DWORD = MaxFallbackValueNameLen;

            ErrorCode = RegEnumValueW(
                FallbackKey,
                FallbackIndex,
                FallbackValueName,
                &mut FallbackValueNameLen,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );
            if ErrorCode != ERROR_SUCCESS as LONG {
                if ErrorCode != ERROR_NO_MORE_ITEMS as LONG {
                    ERR!("Returning %d.\n", ErrorCode);
                }
                break 'exit;
            }
            *FallbackValueName.add(FallbackValueNameLen as usize) = 0;

            /* See if there is such a value on HKCU side */
            ErrorCode = RegQueryValueExW(
                PreferredKey,
                FallbackValueName,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );

            if ErrorCode == ERROR_SUCCESS as LONG {
                dwIndex += 1;
            } else if ErrorCode != ERROR_FILE_NOT_FOUND as LONG {
                ERR!("Got error %d while querying for %s on HKCU side.\n", ErrorCode, FallbackValueName);
                break 'exit;
            }

            if FallbackIndex == dwIndex {
                break;
            }

            FallbackIndex += 1;
        }

        /* We can finally enumerate on the fallback side */
        ErrorCode = RegEnumValueW(
            FallbackKey,
            dwIndex,
            lpName,
            lpcbName,
            lpReserved,
            lpdwType,
            lpData,
            lpcbData,
        );

        break 'exit;
    }

    if PreferredKey != hKey {
        RegCloseKey(PreferredKey);
    }
    if FallbackKey != hKey {
        RegCloseKey(FallbackKey);
    }
    if !FallbackValueName.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, FallbackValueName as PVOID);
    }

    ErrorCode
}

#[no_mangle]
pub unsafe extern "system" fn QueryInfoHKCRKey(
    mut hKey: HKEY,
    lpClass: LPWSTR,
    lpcchClass: LPDWORD,
    lpReserved: LPDWORD,
    lpcSubKeys: LPDWORD,
    lpcbMaxSubKeyLen: LPDWORD,
    lpcbMaxClassLen: LPDWORD,
    lpcValues: LPDWORD,
    lpcbMaxValueNameLen: LPDWORD,
    lpcbMaxValueLen: LPDWORD,
    lpcbSecurityDescriptor: LPDWORD,
    lpftLastWriteTime: PFILETIME,
) -> LONG {
    let mut PreferredKey: HKEY = core::ptr::null_mut();
    let mut FallbackKey: HKEY = core::ptr::null_mut();
    let retval: LONG;
    let err: LONG;
    let mut OtherSubKeys: DWORD = 0;
    let mut OtherMaxSub: DWORD = 0;
    let mut OtherMaxClass: DWORD = 0;
    let mut OtherValues: DWORD = 0;
    let mut OtherMaxName: DWORD = 0;
    let mut OtherMaxVal: DWORD = 0;

    /* Remove the HKCR flag while we're working */
    hKey = ((hKey as ULONG_PTR) & !0x2) as HKEY;

    retval = GetFallbackHKCRKey(hKey, &mut FallbackKey, FALSE as BOOL);
    let mut retval = retval;
    if retval == ERROR_SUCCESS as LONG {
        retval = RegQueryInfoKeyW(
            FallbackKey,
            lpClass,
            lpcchClass,
            lpReserved,
            &mut OtherSubKeys,
            &mut OtherMaxSub,
            &mut OtherMaxClass,
            &mut OtherValues,
            &mut OtherMaxName,
            &mut OtherMaxVal,
            lpcbSecurityDescriptor,
            lpftLastWriteTime,
        );
        if FallbackKey != hKey {
            RegCloseKey(FallbackKey);
        }
    }

    let mut err = GetPreferredHKCRKey(hKey, &mut PreferredKey);
    if err == ERROR_SUCCESS as LONG {
        err = RegQueryInfoKeyW(
            PreferredKey,
            lpClass,
            lpcchClass,
            lpReserved,
            lpcSubKeys,
            lpcbMaxSubKeyLen,
            lpcbMaxClassLen,
            lpcValues,
            lpcbMaxValueNameLen,
            lpcbMaxValueLen,
            lpcbSecurityDescriptor,
            lpftLastWriteTime,
        );
        if PreferredKey != hKey {
            RegCloseKey(PreferredKey);
        }
    }

    if !lpcSubKeys.is_null() {
        *lpcSubKeys = (if err != 0 { 0 } else { *lpcSubKeys }) + OtherSubKeys;
    }
    if !lpcValues.is_null() {
        *lpcValues = (if err != 0 { 0 } else { *lpcValues }) + OtherValues;
    }
    if !lpcbMaxSubKeyLen.is_null() {
        let a = if err != 0 { 0 } else { *lpcbMaxSubKeyLen };
        *lpcbMaxSubKeyLen = if a > OtherMaxSub { a } else { OtherMaxSub };
    }
    if !lpcbMaxClassLen.is_null() {
        let a = if err != 0 { 0 } else { *lpcbMaxClassLen };
        *lpcbMaxClassLen = if a > OtherMaxClass { a } else { OtherMaxClass };
    }
    if !lpcbMaxValueNameLen.is_null() {
        let a = if err != 0 { 0 } else { *lpcbMaxValueNameLen };
        *lpcbMaxValueNameLen = if a > OtherMaxName { a } else { OtherMaxName };
    }
    if !lpcbMaxValueLen.is_null() {
        let a = if err != 0 { 0 } else { *lpcbMaxValueLen };
        *lpcbMaxValueLen = if a > OtherMaxVal { a } else { OtherMaxVal };
    }

    if err == ERROR_SUCCESS as LONG {
        ERROR_SUCCESS as LONG
    } else {
        retval
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS system libraries
//  * FILE:            lib/advapi32/reg/hkcr.c
//  * PURPOSE:         Registry functions - HKEY_CLASSES_ROOT abstraction
//  * PROGRAMMER:      Jerôme Gardou (jerome.gardou@reactos.org)
//  */
//
// #include <advapi32.h>
//
// #include <ndk/cmfuncs.h>
// #include <pseh/pseh2.h>
//
// #include "reg.h"
//
// WINE_DEFAULT_DEBUG_CHANNEL(reg);
//
// static const UNICODE_STRING HKLM_ClassesPath = RTL_CONSTANT_STRING(L"\\Registry\\Machine\\Software\\Classes");
//
// static
// BOOL
// ValueExists( HKEY hNormalKey,  PUNICODE_STRING Name)
// {
//     KEY_VALUE_PARTIAL_INFORMATION kvi;
//     ULONG total_size = sizeof(kvi);
//     NTSTATUS status;
//
//     ASSERT(!IsHKCRKey(hNormalKey));
//     status = NtQueryValueKey(hNormalKey, Name, KeyValuePartialInformation,
//                              &kvi, total_size, &total_size);
//     return status != STATUS_OBJECT_NAME_NOT_FOUND;
// }
//
// static
// LONG
// GetKeyName(HKEY hKey, PUNICODE_STRING KeyName)
// {
//     UNICODE_STRING InfoName;
//     PKEY_NAME_INFORMATION NameInformation;
//     ULONG InfoLength;
//     NTSTATUS Status;
//
//     /* Get info length */
//     InfoLength = 0;
//     Status = NtQueryKey(hKey, KeyNameInformation, NULL, 0, &InfoLength);
//     if (Status != STATUS_BUFFER_TOO_SMALL)
//     {
//         ERR("NtQueryKey returned unexpected Status: 0x%08x\n", Status);
//         return RtlNtStatusToDosError(Status);
//     }
//
//     /* Get it for real */
//     NameInformation = RtlAllocateHeap(RtlGetProcessHeap(), 0, InfoLength);
//     if (NameInformation == NULL)
//     {
//         ERR("Failed to allocate %lu bytes\n", InfoLength);
//         return ERROR_NOT_ENOUGH_MEMORY;
//     }
//
//     Status = NtQueryKey(hKey, KeyNameInformation, NameInformation, InfoLength, &InfoLength);
//     if (!NT_SUCCESS(Status))
//     {
//         RtlFreeHeap(RtlGetProcessHeap(), 0, NameInformation);
//         ERR("NtQueryKey failed: 0x%08x\n", Status);
//         return RtlNtStatusToDosError(Status);
//     }
//
//     /* Make it a proper UNICODE_STRING */
//     InfoName.Length = NameInformation->NameLength;
//     InfoName.MaximumLength = NameInformation->NameLength;
//     InfoName.Buffer = NameInformation->Name;
//
//     Status = RtlDuplicateUnicodeString(RTL_DUPLICATE_UNICODE_STRING_NULL_TERMINATE, &InfoName, KeyName);
//     if (!NT_SUCCESS(Status))
//     {
//         RtlFreeHeap(RtlGetProcessHeap(), 0, NameInformation);
//         ERR("RtlDuplicateUnicodeString failed: 0x%08x\n", Status);
//         return RtlNtStatusToDosError(Status);
//     }
//
//     RtlFreeHeap(RtlGetProcessHeap(), 0, NameInformation);
//
//     return ERROR_SUCCESS;
// }
//
// static
// LONG
// GetKeySam(
//      HKEY hKey,
//      REGSAM* RegSam)
// {
//     NTSTATUS Status;
//     OBJECT_BASIC_INFORMATION ObjectInfo;
//
//     Status = NtQueryObject(hKey, ObjectBasicInformation, &ObjectInfo, sizeof(ObjectInfo), NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("NtQueryObject failed, Status %x08x\n", Status);
//         return RtlNtStatusToDosError(Status);
//     }
//
//     *RegSam = ObjectInfo.GrantedAccess;
//     return ERROR_SUCCESS;
// }
//
// /*
//  * Gets a HKLM key from an HKCU key.
//  */
// static
// LONG
// GetFallbackHKCRKey(
//      HKEY hKey,
//      HKEY* MachineKey,
//      BOOL MustCreate)
// {
//     UNICODE_STRING KeyName;
//     LPWSTR SubKeyName;
//     LONG ErrorCode;
//     REGSAM SamDesired;
//
//     /* Get the key name */
//     ErrorCode = GetKeyName(hKey, &KeyName);
//     if (ErrorCode != ERROR_SUCCESS)
//     {
//         *MachineKey = hKey;
//         return ErrorCode;
//     }
//
//     /* See if we really need a conversion */
//     if (RtlPrefixUnicodeString(&HKLM_ClassesPath, &KeyName, TRUE))
//     {
//         RtlFreeUnicodeString(&KeyName);
//         *MachineKey = hKey;
//         return ERROR_SUCCESS;
//     }
//
//     SubKeyName = KeyName.Buffer + 15; /* 15 == wcslen(L"\\Registry\\User\\") */
//     /* Skip the user token */
//     while (*SubKeyName++ != L'\\')
//     {
//         if (!*SubKeyName)
//         {
//             ERR("Key name %S is invalid!\n", KeyName.Buffer);
//             return ERROR_INTERNAL_ERROR;
//         }
//     }
//
//     /* Use the same access mask than the original key */
//     ErrorCode = GetKeySam(hKey, &SamDesired);
//     if (ErrorCode != ERROR_SUCCESS)
//     {
//         RtlFreeUnicodeString(&KeyName);
//         *MachineKey = hKey;
//         return ErrorCode;
//     }
//
//     if (MustCreate)
//     {
//         ErrorCode = RegCreateKeyExW(
//             HKEY_LOCAL_MACHINE,
//             SubKeyName,
//             0,
//             NULL,
//             0,
//             SamDesired,
//             NULL,
//             MachineKey,
//             NULL);
//     }
//     else
//     {
//         /* Open the key. */
//         ErrorCode = RegOpenKeyExW(
//             HKEY_LOCAL_MACHINE,
//             SubKeyName,
//             0,
//             SamDesired,
//             MachineKey);
//     }
//
//     RtlFreeUnicodeString(&KeyName);
//
//     return ErrorCode;
// }
//
// /* Get the HKCU key (if it exists) from an HKCR key */
// static
// LONG
// GetPreferredHKCRKey(
//      HKEY hKey,
//      HKEY* PreferredKey)
// {
//     UNICODE_STRING KeyName;
//     LPWSTR SubKeyName;
//     LONG ErrorCode;
//     REGSAM SamDesired;
//
//     /* Get the key name */
//     ErrorCode = GetKeyName(hKey, &KeyName);
//     if (ErrorCode != ERROR_SUCCESS)
//     {
//         *PreferredKey = hKey;
//         return ErrorCode;
//     }
//
//     /* See if we really need a conversion */
//     if (!RtlPrefixUnicodeString(&HKLM_ClassesPath, &KeyName, TRUE))
//     {
//         RtlFreeUnicodeString(&KeyName);
//         *PreferredKey = hKey;
//         return ERROR_SUCCESS;
//     }
//
//     /* 18 == wcslen(L"\\Registry\\Machine\\") */
//     SubKeyName = KeyName.Buffer + 18;
//
//     /* Use the same access mask than the original key */
//     ErrorCode = GetKeySam(hKey, &SamDesired);
//     if (ErrorCode != ERROR_SUCCESS)
//     {
//         RtlFreeUnicodeString(&KeyName);
//         *PreferredKey = hKey;
//         return ErrorCode;
//     }
//
//     /* Open the key. */
//     ErrorCode = RegOpenKeyExW(
//         HKEY_CURRENT_USER,
//         SubKeyName,
//         0,
//         SamDesired,
//         PreferredKey);
//
//     RtlFreeUnicodeString(&KeyName);
//
//     return ErrorCode;
// }
//
// static
// LONG
// GetSubkeyInfoHelper(
//      HKEY hKey,
//      LPDWORD lpSubKeys,
//      LPDWORD lpMaxSubKeyLen)
// {
//     LONG err = RegQueryInfoKeyW(hKey, NULL, NULL, NULL, lpSubKeys, lpMaxSubKeyLen,
//                                 NULL, NULL, NULL, NULL, NULL, NULL);
//     if (err != ERROR_ACCESS_DENIED)
//         return err;
//
//     /* Windows RegEdit only uses KEY_ENUMERATE_SUB_KEYS when enumerating but
//      * KEY_QUERY_VALUE is required to get the info in EnumHKCRKey.
//      */
//     if (RegOpenKeyExW(hKey, NULL, 0, KEY_QUERY_VALUE, &hKey) == ERROR_SUCCESS)
//     {
//         err = RegQueryInfoKeyW(hKey, NULL, NULL, NULL, lpSubKeys, lpMaxSubKeyLen,
//                                NULL, NULL, NULL, NULL, NULL, NULL);
//         RegCloseKey(hKey);
//     }
//     return err;
// }
//
// /* HKCR version of RegCreateKeyExW. */
// LONG
// WINAPI
// CreateHKCRKey(
//      HKEY hKey,
//      LPCWSTR lpSubKey,
//      DWORD Reserved,
//      LPWSTR lpClass,
//      DWORD dwOptions,
//      REGSAM samDesired,
//      LPSECURITY_ATTRIBUTES lpSecurityAttributes,
//      PHKEY phkResult,
//      LPDWORD lpdwDisposition)
// {
//     LONG ErrorCode;
//     HKEY QueriedKey, TestKey;
//
//     ASSERT(IsHKCRKey(hKey));
//
//     /* Remove the HKCR flag while we're working */
//     hKey = (HKEY)(((ULONG_PTR)hKey) & ~0x2);
//
//     ErrorCode = GetPreferredHKCRKey(hKey, &QueriedKey);
//
//     if (ErrorCode == ERROR_FILE_NOT_FOUND)
//     {
//         /* The current key doesn't exist on HKCU side, so we can only create it in HKLM */
//         ErrorCode = RegCreateKeyExW(
//             hKey,
//             lpSubKey,
//             Reserved,
//             lpClass,
//             dwOptions,
//             samDesired,
//             lpSecurityAttributes,
//             phkResult,
//             lpdwDisposition);
//         if (ErrorCode == ERROR_SUCCESS)
//             MakeHKCRKey(phkResult);
//         return ErrorCode;
//     }
//
//     if (ErrorCode != ERROR_SUCCESS)
//     {
//         /* Somehow we failed for another reason (maybe deleted key or whatever) */
//         return ErrorCode;
//     }
//
//     /* See if the subkey already exists in HKCU. */
//     ErrorCode = RegOpenKeyExW(QueriedKey, lpSubKey, 0, READ_CONTROL, &TestKey);
//     if (ErrorCode != ERROR_FILE_NOT_FOUND)
//     {
//         if (ErrorCode == ERROR_SUCCESS)
//         {
//             /* Great. Close the test one and do the real create operation */
//             RegCloseKey(TestKey);
//             ErrorCode = RegCreateKeyExW(
//                 QueriedKey,
//                 lpSubKey,
//                 Reserved,
//                 lpClass,
//                 dwOptions,
//                 samDesired,
//                 lpSecurityAttributes,
//                 phkResult,
//                 lpdwDisposition);
//             if (ErrorCode == ERROR_SUCCESS)
//                 MakeHKCRKey(phkResult);
//         }
//         if (QueriedKey != hKey)
//             RegCloseKey(QueriedKey);
//
//         return ERROR_SUCCESS;
//     }
//
//     if (QueriedKey != hKey)
//         RegCloseKey(QueriedKey);
//
//     /* So we must do the create operation in HKLM, creating the missing parent keys if needed. */
//     ErrorCode = GetFallbackHKCRKey(hKey, &QueriedKey, TRUE);
//     if (ErrorCode != ERROR_SUCCESS)
//         return ErrorCode;
//
//     /* Do the key creation */
//     ErrorCode = RegCreateKeyEx(
//         QueriedKey,
//         lpSubKey,
//         Reserved,
//         lpClass,
//         dwOptions,
//         samDesired,
//         lpSecurityAttributes,
//         phkResult,
//         lpdwDisposition);
//
//     if (QueriedKey != hKey)
//         RegCloseKey(QueriedKey);
//
//     if (ErrorCode == ERROR_SUCCESS)
//         MakeHKCRKey(phkResult);
//
//     return ErrorCode;
// }
//
// /* Same as RegOpenKeyExW, but for HKEY_CLASSES_ROOT subkeys */
// LONG
// WINAPI
// OpenHKCRKey(
//      HKEY hKey,
//      LPCWSTR lpSubKey,
//      DWORD ulOptions,
//      REGSAM samDesired,
//      PHKEY phkResult)
// {
//     HKEY QueriedKey;
//     LONG ErrorCode;
//
//     ASSERT(IsHKCRKey(hKey));
//
//     /* Remove the HKCR flag while we're working */
//     hKey = (HKEY)(((ULONG_PTR)hKey) & ~0x2);
//
//     ErrorCode = GetPreferredHKCRKey(hKey, &QueriedKey);
//
//     if (ErrorCode == ERROR_FILE_NOT_FOUND)
//     {
//         /* The key doesn't exist on HKCU side, no chance for a subkey */
//         ErrorCode = RegOpenKeyExW(hKey, lpSubKey, ulOptions, samDesired, phkResult);
//         if (ErrorCode == ERROR_SUCCESS)
//             MakeHKCRKey(phkResult);
//         return ErrorCode;
//     }
//
//     if (ErrorCode != ERROR_SUCCESS)
//     {
//         /* Somehow we failed for another reason (maybe deleted key or whatever) */
//         return ErrorCode;
//     }
//
//     /* Try on the HKCU side */
//     ErrorCode = RegOpenKeyExW(QueriedKey, lpSubKey, ulOptions, samDesired, phkResult);
//     if (ErrorCode == ERROR_SUCCESS)
//         MakeHKCRKey(phkResult);
//
//     /* Close it if we must */
//     if (QueriedKey != hKey)
//     {
//         RegCloseKey(QueriedKey);
//     }
//
//     /* Anything else than ERROR_FILE_NOT_FOUND means that we found it, even if it is with failures. */
//     if (ErrorCode != ERROR_FILE_NOT_FOUND)
//         return ErrorCode;
//
//     /* If we're here, we must open from HKLM key. */
//     ErrorCode = GetFallbackHKCRKey(hKey, &QueriedKey, FALSE);
//     if (ErrorCode != ERROR_SUCCESS)
//     {
//         /* Maybe the key doesn't exist in the HKLM view */
//         return ErrorCode;
//     }
//
//     ErrorCode = RegOpenKeyExW(QueriedKey, lpSubKey, ulOptions, samDesired, phkResult);
//     if (ErrorCode == ERROR_SUCCESS)
//         MakeHKCRKey(phkResult);
//
//     /* Close it if we must */
//     if (QueriedKey != hKey)
//     {
//         RegCloseKey(QueriedKey);
//     }
//
//     return ErrorCode;
// }
//
// /* HKCR version of RegDeleteKeyExW */
// LONG
// WINAPI
// DeleteHKCRKey(
//      HKEY hKey,
//      LPCWSTR lpSubKey,
//      REGSAM RegSam,
//      DWORD Reserved)
// {
//     HKEY QueriedKey;
//     LONG ErrorCode;
//
//     ASSERT(IsHKCRKey(hKey));
//
//     /* Remove the HKCR flag while we're working */
//     hKey = (HKEY)(((ULONG_PTR)hKey) & ~0x2);
//
//     ErrorCode = GetPreferredHKCRKey(hKey, &QueriedKey);
//
//     if (ErrorCode == ERROR_FILE_NOT_FOUND)
//     {
//         /* The key doesn't exist on HKCU side, no chance for a subkey */
//         return RegDeleteKeyExW(hKey, lpSubKey, RegSam, Reserved);
//     }
//
//     if (ErrorCode != ERROR_SUCCESS)
//     {
//         /* Somehow we failed for another reason (maybe deleted key or whatever) */
//         return ErrorCode;
//     }
//
//     ErrorCode = RegDeleteKeyExW(QueriedKey, lpSubKey, RegSam, Reserved);
//
//     /* Close it if we must */
//     if (QueriedKey != hKey)
//     {
//         /* The original key is on the machine view */
//         RegCloseKey(QueriedKey);
//     }
//
//     /* Anything else than ERROR_FILE_NOT_FOUND means that we found it, even if it is with failures. */
//     if (ErrorCode != ERROR_FILE_NOT_FOUND)
//         return ErrorCode;
//
//     /* If we're here, we must open from HKLM key. */
//     ErrorCode = GetFallbackHKCRKey(hKey, &QueriedKey, FALSE);
//     if (ErrorCode != ERROR_SUCCESS)
//     {
//         /* Maybe the key doesn't exist in the HKLM view */
//         return ErrorCode;
//     }
//
//     ErrorCode = RegDeleteKeyExW(QueriedKey, lpSubKey, RegSam, Reserved);
//
//     /* Close it if we must */
//     if (QueriedKey != hKey)
//     {
//         RegCloseKey(QueriedKey);
//     }
//
//     return ErrorCode;
// }
//
// /* HKCR version of RegDeleteValueA+W */
// LONG
// WINAPI
// DeleteHKCRValue(
//      HKEY hKey,
//      PUNICODE_STRING ValueName)
// {
//     HKEY hActualKey;
//     LONG ErrorCode;
//
//     ASSERT(IsHKCRKey(hKey));
//     /* Remove the HKCR flag while we're working */
//     hKey = (HKEY)(((ULONG_PTR)hKey) & ~0x2);
//
//     /* Does the HKCU key and value exist? */
//     ErrorCode = GetPreferredHKCRKey(hKey, &hActualKey);
//     if (ErrorCode == ERROR_SUCCESS)
//     {
//         if (!ValueExists(hActualKey, ValueName))
//         {
//             if (hActualKey != hKey)
//             {
//                 RegCloseKey(hActualKey);
//             }
//             ErrorCode = ERROR_FILE_NOT_FOUND;
//         }
//     }
//     if (ErrorCode == ERROR_FILE_NOT_FOUND)
//     {
//         ErrorCode = GetFallbackHKCRKey(hKey, &hActualKey, FALSE);
//     }
//
//     if (ErrorCode == ERROR_SUCCESS)
//     {
//         NTSTATUS Status = NtDeleteValueKey(hActualKey, ValueName);
//         ErrorCode = NT_SUCCESS(Status) ? ERROR_SUCCESS : RtlNtStatusToDosError(Status);
//     }
//     if (hActualKey != hKey)
//     {
//         RegCloseKey(hActualKey);
//     }
//     return ErrorCode;
// }
//
// /* HKCR version of RegQueryValueExW */
// LONG
// WINAPI
// QueryHKCRValue(
//      HKEY hKey,
//      LPCWSTR Name,
//      LPDWORD Reserved,
//      LPDWORD Type,
//      LPBYTE Data,
//      LPDWORD Count)
// {
//     HKEY QueriedKey;
//     LONG ErrorCode;
//
//     ASSERT(IsHKCRKey(hKey));
//
//     /* Remove the HKCR flag while we're working */
//     hKey = (HKEY)(((ULONG_PTR)hKey) & ~0x2);
//
//     ErrorCode = GetPreferredHKCRKey(hKey, &QueriedKey);
//
//     if (ErrorCode == ERROR_FILE_NOT_FOUND)
//     {
//         /* The key doesn't exist on HKCU side, no chance for a value in it */
//         return RegQueryValueExW(hKey, Name, Reserved, Type, Data, Count);
//     }
//
//     if (ErrorCode != ERROR_SUCCESS)
//     {
//         /* Somehow we failed for another reason (maybe deleted key or whatever) */
//         return ErrorCode;
//     }
//
//     ErrorCode = RegQueryValueExW(QueriedKey, Name, Reserved, Type, Data, Count);
//
//     /* Close it if we must */
//     if (QueriedKey != hKey)
//     {
//         /* The original key is on the machine view */
//         RegCloseKey(QueriedKey);
//     }
//
//     /* Anything else than ERROR_FILE_NOT_FOUND means that we found it, even if it is with failures. */
//     if (ErrorCode != ERROR_FILE_NOT_FOUND)
//         return ErrorCode;
//
//     /* If we're here, we must open from HKLM key. */
//     ErrorCode = GetFallbackHKCRKey(hKey, &QueriedKey, FALSE);
//     if (ErrorCode != ERROR_SUCCESS)
//     {
//         /* Maybe the key doesn't exist in the HKLM view */
//         return ErrorCode;
//     }
//
//     ErrorCode = RegQueryValueExW(QueriedKey, Name, Reserved, Type, Data, Count);
//
//     /* Close it if we must */
//     if (QueriedKey != hKey)
//     {
//         RegCloseKey(QueriedKey);
//     }
//
//     return ErrorCode;
// }
//
// /* HKCR version of RegSetValueExW */
// LONG
// WINAPI
// SetHKCRValue(
//      HKEY hKey,
//      LPCWSTR Name,
//      DWORD Reserved,
//      DWORD Type,
//      CONST BYTE* Data,
//      DWORD DataSize)
// {
//     HKEY QueriedKey;
//     LONG ErrorCode;
//
//     ASSERT(IsHKCRKey(hKey));
//
//     /* Remove the HKCR flag while we're working */
//     hKey = (HKEY)(((ULONG_PTR)hKey) & ~0x2);
//
//     ErrorCode = GetPreferredHKCRKey(hKey, &QueriedKey);
//
//     if (ErrorCode == ERROR_FILE_NOT_FOUND)
//     {
//         /* The key doesn't exist on HKCU side, no chance to put a value in it */
//         return RegSetValueExW(hKey, Name, Reserved, Type, Data, DataSize);
//     }
//
//     if (ErrorCode != ERROR_SUCCESS)
//     {
//         /* Somehow we failed for another reason (maybe deleted key or whatever) */
//         return ErrorCode;
//     }
//
//     /* Check if the value already exists in the preferred key */
//     ErrorCode = RegQueryValueExW(QueriedKey, Name, NULL, NULL, NULL, NULL);
//     if (ErrorCode != ERROR_FILE_NOT_FOUND)
//     {
//         if (ErrorCode == ERROR_SUCCESS)
//         {
//             /* Yes, so we have the right to modify it */
//             ErrorCode = RegSetValueExW(QueriedKey, Name, Reserved, Type, Data, DataSize);
//         }
//         if (QueriedKey != hKey)
//             RegCloseKey(QueriedKey);
//         return ErrorCode;
//     }
//     if (QueriedKey != hKey)
//         RegCloseKey(QueriedKey);
//
//     /* So we must set the value in the HKLM version */
//     ErrorCode = GetPreferredHKCRKey(hKey, &QueriedKey);
//     if (ErrorCode == ERROR_FILE_NOT_FOUND)
//     {
//         /* No choice: put this in HKCU */
//         return RegSetValueExW(hKey, Name, Reserved, Type, Data, DataSize);
//     }
//     else if (ErrorCode != ERROR_SUCCESS)
//     {
//         return ErrorCode;
//     }
//
//     ErrorCode = RegSetValueExW(QueriedKey, Name, Reserved, Type, Data, DataSize);
//
//     if (QueriedKey != hKey)
//         RegCloseKey(QueriedKey);
//
//     return ErrorCode;
// }
//
// /* HKCR version of RegEnumKeyExW */
// LONG
// WINAPI
// EnumHKCRKey(
//      HKEY hKey,
//      DWORD dwIndex,
//      LPWSTR lpName,
//      LPDWORD lpcbName,
//      LPDWORD lpReserved,
//      LPWSTR lpClass,
//      LPDWORD lpcbClass,
//      PFILETIME lpftLastWriteTime)
// {
//     HKEY PreferredKey, FallbackKey;
//     DWORD NumPreferredSubKeys;
//     DWORD MaxFallbackSubKeyLen;
//     DWORD FallbackIndex;
//     WCHAR* FallbackSubKeyName = NULL;
//     LONG ErrorCode;
//
//     ASSERT(IsHKCRKey(hKey));
//
//     /* Remove the HKCR flag while we're working */
//     hKey = (HKEY)(((ULONG_PTR)hKey) & ~0x2);
//
//     /* Get the preferred key */
//     ErrorCode = GetPreferredHKCRKey(hKey, &PreferredKey);
//     if (ErrorCode != ERROR_SUCCESS)
//     {
//         if (ErrorCode == ERROR_FILE_NOT_FOUND)
//         {
//             /* Only the HKLM key exists */
//             return RegEnumKeyExW(
//                 hKey,
//                 dwIndex,
//                 lpName,
//                 lpcbName,
//                 lpReserved,
//                 lpClass,
//                 lpcbClass,
//                 lpftLastWriteTime);
//         }
//         return ErrorCode;
//     }
//
//     /* Get the fallback key */
//     ErrorCode = GetFallbackHKCRKey(hKey, &FallbackKey, FALSE);
//     if (ErrorCode != ERROR_SUCCESS)
//     {
//         if (PreferredKey != hKey)
//             RegCloseKey(PreferredKey);
//         if (ErrorCode == ERROR_FILE_NOT_FOUND)
//         {
//             /* Only the HKCU key exists */
//             return RegEnumKeyExW(
//                 hKey,
//                 dwIndex,
//                 lpName,
//                 lpcbName,
//                 lpReserved,
//                 lpClass,
//                 lpcbClass,
//                 lpftLastWriteTime);
//         }
//         return ErrorCode;
//     }
//
//     /* Get some info on the HKCU side */
//     ErrorCode = GetSubkeyInfoHelper(PreferredKey, &NumPreferredSubKeys, NULL);
//     if (ErrorCode != ERROR_SUCCESS)
//         goto Exit;
//
//     if (dwIndex < NumPreferredSubKeys)
//     {
//         /* HKCU side takes precedence */
//         ErrorCode = RegEnumKeyExW(
//             PreferredKey,
//             dwIndex,
//             lpName,
//             lpcbName,
//             lpReserved,
//             lpClass,
//             lpcbClass,
//             lpftLastWriteTime);
//         goto Exit;
//     }
//
//     /* Here it gets tricky. We must enumerate the values from the HKLM side,
//      * without reporting those which are present on the HKCU side */
//
//     /* Squash out the indices from HKCU */
//     dwIndex -= NumPreferredSubKeys;
//
//     /* Get some info */
//     ErrorCode = GetSubkeyInfoHelper(FallbackKey, NULL, &MaxFallbackSubKeyLen);
//     if (ErrorCode != ERROR_SUCCESS)
//     {
//         ERR("Could not query info of key %p (Err: %d)\n", FallbackKey, ErrorCode);
//         goto Exit;
//     }
//
//     MaxFallbackSubKeyLen++;
//     TRACE("Maxfallbacksubkeylen: %d\n", MaxFallbackSubKeyLen);
//
//     /* Allocate our buffer */
//     FallbackSubKeyName = RtlAllocateHeap(
//         RtlGetProcessHeap(), 0, MaxFallbackSubKeyLen * sizeof(WCHAR));
//     if (!FallbackSubKeyName)
//     {
//         ErrorCode = ERROR_NOT_ENOUGH_MEMORY;
//         goto Exit;
//     }
//
//     /* We must begin at the very first subkey of the fallback key,
//      * and then see if we meet keys that already are in the preferred key.
//      * In that case, we must bump dwIndex, as otherwise we would enumerate a key we already
//      * saw in a previous call.
//      */
//     FallbackIndex = 0;
//     while (TRUE)
//     {
//         HKEY PreferredSubKey;
//         DWORD FallbackSubkeyLen = MaxFallbackSubKeyLen;
//
//         /* Try enumerating */
//         ErrorCode = RegEnumKeyExW(
//             FallbackKey,
//             FallbackIndex,
//             FallbackSubKeyName,
//             &FallbackSubkeyLen,
//             NULL,
//             NULL,
//             NULL,
//             NULL);
//         if (ErrorCode != ERROR_SUCCESS)
//         {
//             if (ErrorCode != ERROR_NO_MORE_ITEMS)
//                 ERR("Returning %d.\n", ErrorCode);
//             goto Exit;
//         }
//         FallbackSubKeyName[FallbackSubkeyLen] = L'\0';
//
//         /* See if there is such a value on HKCU side */
//         ErrorCode = RegOpenKeyExW(
//             PreferredKey,
//             FallbackSubKeyName,
//             0,
//             READ_CONTROL,
//             &PreferredSubKey);
//
//         if (ErrorCode == ERROR_SUCCESS)
//         {
//             RegCloseKey(PreferredSubKey);
//             /* So we already enumerated it on HKCU side. */
//             dwIndex++;
//         }
//         else if (ErrorCode != ERROR_FILE_NOT_FOUND)
//         {
//             ERR("Got error %d while querying for %s on HKCU side.\n", ErrorCode, FallbackSubKeyName);
//             goto Exit;
//         }
//
//         /* See if we caught up */
//         if (FallbackIndex == dwIndex)
//             break;
//
//         FallbackIndex++;
//     }
//
//     /* We can finally enumerate on the fallback side */
//     ErrorCode = RegEnumKeyExW(
//         FallbackKey,
//         dwIndex,
//         lpName,
//         lpcbName,
//         lpReserved,
//         lpClass,
//         lpcbClass,
//         lpftLastWriteTime);
//
// Exit:
//     if (PreferredKey != hKey)
//         RegCloseKey(PreferredKey);
//     if (FallbackKey != hKey)
//         RegCloseKey(FallbackKey);
//     if (FallbackSubKeyName)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, FallbackSubKeyName);
//
//     return ErrorCode;
// }
//
// /* HKCR version of RegEnumValueW */
// LONG
// WINAPI
// EnumHKCRValue(
//      HKEY hKey,
//      DWORD dwIndex,
//      LPWSTR lpName,
//      PDWORD lpcbName,
//      PDWORD lpReserved,
//      PDWORD lpdwType,
//      LPBYTE lpData,
//      PDWORD lpcbData)
// {
//     HKEY PreferredKey, FallbackKey;
//     DWORD NumPreferredValues;
//     DWORD MaxFallbackValueNameLen;
//     DWORD FallbackIndex;
//     WCHAR* FallbackValueName = NULL;
//     LONG ErrorCode;
//
//     ASSERT(IsHKCRKey(hKey));
//
//     /* Remove the HKCR flag while we're working */
//     hKey = (HKEY)(((ULONG_PTR)hKey) & ~0x2);
//
//     /* Get the preferred key */
//     ErrorCode = GetPreferredHKCRKey(hKey, &PreferredKey);
//     if (ErrorCode != ERROR_SUCCESS)
//     {
//         if (ErrorCode == ERROR_FILE_NOT_FOUND)
//         {
//             /* Only the HKLM key exists */
//             return RegEnumValueW(
//                 hKey,
//                 dwIndex,
//                 lpName,
//                 lpcbName,
//                 lpReserved,
//                 lpdwType,
//                 lpData,
//                 lpcbData);
//         }
//         return ErrorCode;
//     }
//
//     /* Get the fallback key */
//     ErrorCode = GetFallbackHKCRKey(hKey, &FallbackKey, FALSE);
//     if (ErrorCode != ERROR_SUCCESS)
//     {
//         if (PreferredKey != hKey)
//             RegCloseKey(PreferredKey);
//         if (ErrorCode == ERROR_FILE_NOT_FOUND)
//         {
//             /* Only the HKCU key exists */
//             return RegEnumValueW(
//                 hKey,
//                 dwIndex,
//                 lpName,
//                 lpcbName,
//                 lpReserved,
//                 lpdwType,
//                 lpData,
//                 lpcbData);
//         }
//         return ErrorCode;
//     }
//
//     /* Get some info on the HKCU side */
//     ErrorCode = RegQueryInfoKeyW(
//         PreferredKey,
//         NULL,
//         NULL,
//         NULL,
//         NULL,
//         NULL,
//         NULL,
//         &NumPreferredValues,
//         NULL,
//         NULL,
//         NULL,
//         NULL);
//     if (ErrorCode != ERROR_SUCCESS)
//         goto Exit;
//
//     if (dwIndex < NumPreferredValues)
//     {
//         /* HKCU side takes precedence */
//         return RegEnumValueW(
//             PreferredKey,
//             dwIndex,
//             lpName,
//             lpcbName,
//             lpReserved,
//             lpdwType,
//             lpData,
//             lpcbData);
//         goto Exit;
//     }
//
//     /* Here it gets tricky. We must enumerate the values from the HKLM side,
//      * without reporting those which are present on the HKCU side */
//
//     /* Squash out the indices from HKCU */
//     dwIndex -= NumPreferredValues;
//
//     /* Get some info */
//     ErrorCode = RegQueryInfoKeyW(
//         FallbackKey,
//         NULL,
//         NULL,
//         NULL,
//         NULL,
//         NULL,
//         NULL,
//         NULL,
//         &MaxFallbackValueNameLen,
//         NULL,
//         NULL,
//         NULL);
//     if (ErrorCode != ERROR_SUCCESS)
//     {
//         ERR("Could not query info of key %p (Err: %d)\n", FallbackKey, ErrorCode);
//         goto Exit;
//     }
//
//     MaxFallbackValueNameLen++;
//     TRACE("Maxfallbacksubkeylen: %d\n", MaxFallbackValueNameLen);
//
//     /* Allocate our buffer */
//     FallbackValueName = RtlAllocateHeap(
//         RtlGetProcessHeap(), 0, MaxFallbackValueNameLen * sizeof(WCHAR));
//     if (!FallbackValueName)
//     {
//         ErrorCode = ERROR_NOT_ENOUGH_MEMORY;
//         goto Exit;
//     }
//
//     /* We must begin at the very first subkey of the fallback key,
//      * and then see if we meet keys that already are in the preferred key.
//      * In that case, we must bump dwIndex, as otherwise we would enumerate a key we already
//      * saw in a previous call.
//      */
//     FallbackIndex = 0;
//     while (TRUE)
//     {
//         DWORD FallbackValueNameLen = MaxFallbackValueNameLen;
//
//         /* Try enumerating */
//         ErrorCode = RegEnumValueW(
//             FallbackKey,
//             FallbackIndex,
//             FallbackValueName,
//             &FallbackValueNameLen,
//             NULL,
//             NULL,
//             NULL,
//             NULL);
//         if (ErrorCode != ERROR_SUCCESS)
//         {
//             if (ErrorCode != ERROR_NO_MORE_ITEMS)
//                 ERR("Returning %d.\n", ErrorCode);
//             goto Exit;
//         }
//         FallbackValueName[FallbackValueNameLen] = L'\0';
//
//         /* See if there is such a value on HKCU side */
//         ErrorCode = RegQueryValueExW(
//             PreferredKey,
//             FallbackValueName,
//             NULL,
//             NULL,
//             NULL,
//             NULL);
//
//         if (ErrorCode == ERROR_SUCCESS)
//         {
//             /* So we already enumerated it on HKCU side. */
//             dwIndex++;
//         }
//         else if (ErrorCode != ERROR_FILE_NOT_FOUND)
//         {
//             ERR("Got error %d while querying for %s on HKCU side.\n", ErrorCode, FallbackValueName);
//             goto Exit;
//         }
//
//         /* See if we caught up */
//         if (FallbackIndex == dwIndex)
//             break;
//
//         FallbackIndex++;
//     }
//
//     /* We can finally enumerate on the fallback side */
//     ErrorCode = RegEnumValueW(
//         FallbackKey,
//         dwIndex,
//         lpName,
//         lpcbName,
//         lpReserved,
//         lpdwType,
//         lpData,
//         lpcbData);
//
// Exit:
//     if (PreferredKey != hKey)
//         RegCloseKey(PreferredKey);
//     if (FallbackKey != hKey)
//         RegCloseKey(FallbackKey);
//     if (FallbackValueName)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, FallbackValueName);
//
//     return ErrorCode;
// }
//
// /* HKCR version of RegQueryInfoKeyW */
// LONG
// WINAPI
// QueryInfoHKCRKey(
//      HKEY hKey,
//      LPWSTR lpClass,
//      LPDWORD lpcchClass,
//      LPDWORD lpReserved,
//      LPDWORD lpcSubKeys,
//      LPDWORD lpcbMaxSubKeyLen,
//      LPDWORD lpcbMaxClassLen,
//      LPDWORD lpcValues,
//      LPDWORD lpcbMaxValueNameLen,
//      LPDWORD lpcbMaxValueLen,
//      LPDWORD lpcbSecurityDescriptor,
//      PFILETIME lpftLastWriteTime)
// {
//     HKEY PreferredKey, FallbackKey;
//     LONG retval, err;
//     DWORD OtherSubKeys = 0, OtherMaxSub = 0, OtherMaxClass = 0;
//     DWORD OtherValues = 0, OtherMaxName = 0, OtherMaxVal = 0;
//
//     ASSERT(IsHKCRKey(hKey));
//
//     /* Remove the HKCR flag while we're working */
//     hKey = (HKEY)(((ULONG_PTR)hKey) & ~0x2);
//
//     retval = GetFallbackHKCRKey(hKey, &FallbackKey, FALSE);
//     if (retval == ERROR_SUCCESS)
//     {
//         retval = RegQueryInfoKeyW(FallbackKey, lpClass, lpcchClass, lpReserved,
//                                   &OtherSubKeys, &OtherMaxSub, &OtherMaxClass,
//                                   &OtherValues, &OtherMaxName, &OtherMaxVal,
//                                   lpcbSecurityDescriptor, lpftLastWriteTime);
//         if (FallbackKey != hKey)
//             RegCloseKey(FallbackKey);
//     }
//
//     err = GetPreferredHKCRKey(hKey, &PreferredKey);
//     if (err == ERROR_SUCCESS)
//     {
//         err = RegQueryInfoKeyW(PreferredKey, lpClass, lpcchClass, lpReserved,
//                                lpcSubKeys, lpcbMaxSubKeyLen, lpcbMaxClassLen,
//                                lpcValues, lpcbMaxValueNameLen, lpcbMaxValueLen,
//                                lpcbSecurityDescriptor, lpftLastWriteTime);
//         if (PreferredKey != hKey)
//             RegCloseKey(PreferredKey);
//     }
//
//     if (lpcSubKeys)
//         *lpcSubKeys = (err ? 0 : *lpcSubKeys) + OtherSubKeys;
//     if (lpcValues)
//         *lpcValues = (err ? 0 : *lpcValues) + OtherValues;
//     if (lpcbMaxSubKeyLen)
//         *lpcbMaxSubKeyLen = max((err ? 0 : *lpcbMaxSubKeyLen), OtherMaxSub);
//     if (lpcbMaxClassLen)
//         *lpcbMaxClassLen = max((err ? 0 : *lpcbMaxClassLen), OtherMaxClass);
//     if (lpcbMaxValueNameLen)
//         *lpcbMaxValueNameLen = max((err ? 0 : *lpcbMaxValueNameLen), OtherMaxName);
//     if (lpcbMaxValueLen)
//         *lpcbMaxValueLen = max((err ? 0 : *lpcbMaxValueLen), OtherMaxVal);
//
//     return (err == ERROR_SUCCESS) ? ERROR_SUCCESS : retval;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
