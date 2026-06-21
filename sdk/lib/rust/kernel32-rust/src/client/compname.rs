use ntdll_sys::nt::*;
use crate::prelude::*;
use crate::substrate::{TRUE, FALSE};
use crate::k32_externs::{RtlCopyMemory, LoadLibraryW, GetProcAddress};
use crate::client::loader::FreeLibrary;
use crate::string::lstring::lstrlenA;
use core::ffi::c_void;

extern "system" {
    fn RtlStringCchLengthW(psz: *const u16, cchMax: usize, pcch: *mut usize) -> NTSTATUS;
    fn RtlStringCchCopyNW(pszDest: *mut u16, cchDest: usize, pszSrc: *const u16, cchSrc: usize) -> NTSTATUS;
}

// windns.h: DNS_NAME_FORMAT enum
pub type DNS_NAME_FORMAT = core::ffi::c_int;
pub const DnsNameDomain: DNS_NAME_FORMAT = 0;
pub const DnsNameDomainLabel: DNS_NAME_FORMAT = 1;
pub const DnsNameHostnameFull: DNS_NAME_FORMAT = 2;
pub const DnsNameHostnameLabel: DNS_NAME_FORMAT = 3;
pub const DnsNameWildcard: DNS_NAME_FORMAT = 4;
pub const DnsNameSrvRecord: DNS_NAME_FORMAT = 5;
pub const DnsNameValidateTld: DNS_NAME_FORMAT = 6;

// ---- file-scope preamble (translated) ----
pub type FN_DnsValidateName_W = Option<unsafe extern "system" fn(LPCWSTR, DNS_NAME_FORMAT) -> NTSTATUS>;
unsafe fn GetComputerNameFromRegistry(
    RegistryKey: LPCWSTR,
    ValueNameStr: LPCWSTR,
    lpBuffer: LPWSTR,
    nSize: LPDWORD,
) -> BOOL {
    let mut KeyInfo: PKEY_VALUE_PARTIAL_INFORMATION = core::ptr::null_mut();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut KeyName: _UNICODE_STRING = core::mem::zeroed();
    let mut ValueName: _UNICODE_STRING = core::mem::zeroed();
    let mut KeyHandle: HANDLE = core::ptr::null_mut();
    let mut KeyInfoSize: ULONG = 0;
    let mut ReturnSize: ULONG = 0;
    let mut Status: NTSTATUS = 0;

    if !lpBuffer.is_null() && *nSize > 0 {
        *lpBuffer = 0;
    }

    RtlInitUnicodeString(&mut KeyName, RegistryKey);

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
    ObjectAttributes.ObjectName = &mut KeyName;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = NtOpenKey(&mut KeyHandle, KEY_READ, &mut ObjectAttributes);
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return FALSE;
    }

    KeyInfoSize = (core::mem::size_of::<KEY_VALUE_PARTIAL_INFORMATION>() as ULONG)
        .wrapping_add((*nSize).wrapping_mul(core::mem::size_of::<WCHAR>() as ULONG));

    KeyInfo = RtlAllocateHeap(RtlGetProcessHeap(), 0, KeyInfoSize as SIZE_T)
        as PKEY_VALUE_PARTIAL_INFORMATION;

    if KeyInfo.is_null() {
        NtClose(KeyHandle);
        SetLastError(ERROR_OUTOFMEMORY);
        return FALSE;
    }

    RtlInitUnicodeString(&mut ValueName, ValueNameStr);

    Status = NtQueryValueKey(
        KeyHandle,
        &mut ValueName,
        KeyValuePartialInformation,
        KeyInfo as PVOID,
        KeyInfoSize,
        &mut ReturnSize,
    );

    NtClose(KeyHandle);

    if !NT_SUCCESS(Status) {
        *nSize = (ReturnSize
            .wrapping_sub(core::mem::offset_of!(KEY_VALUE_PARTIAL_INFORMATION, Data) as ULONG))
            .wrapping_div(core::mem::size_of::<WCHAR>() as ULONG);
    } else if (*KeyInfo).Type != REG_SZ {
        Status = STATUS_UNSUCCESSFUL;
    } else if lpBuffer.is_null()
        || (*nSize as usize)
            < ((*KeyInfo).DataLength as usize).wrapping_div(core::mem::size_of::<WCHAR>() as usize)
    {
        *nSize = (ReturnSize
            .wrapping_sub(core::mem::offset_of!(KEY_VALUE_PARTIAL_INFORMATION, Data) as ULONG))
            .wrapping_div(core::mem::size_of::<WCHAR>() as ULONG);
        Status = STATUS_BUFFER_OVERFLOW;
    } else {
        *nSize = ((*KeyInfo).DataLength as usize)
            .wrapping_div(core::mem::size_of::<WCHAR>() as usize)
            .wrapping_sub(1) as ULONG;
        RtlCopyMemory(
            lpBuffer as *mut core::ffi::c_void,
            &raw mut (*KeyInfo).Data as *const core::ffi::c_void,
            (*KeyInfo).DataLength as SIZE_T,
        );
        *lpBuffer.offset(*nSize as isize) = 0;
        RtlFreeHeap(RtlGetProcessHeap(), 0, KeyInfo as PVOID);
        return TRUE;
    }

    RtlFreeHeap(RtlGetProcessHeap(), 0, KeyInfo as PVOID);
    BaseSetLastNTError(Status);
    return FALSE;
}
unsafe fn SetActiveComputerNameToRegistry(
    RegistryKey: LPCWSTR,
    SubKey: LPCWSTR,
    ValueNameStr: LPCWSTR,
    lpBuffer: LPCWSTR,
) -> BOOL {
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut KeyName: _UNICODE_STRING = core::mem::zeroed();
    let mut ValueName: _UNICODE_STRING = core::mem::zeroed();
    let mut KeyHandle: HANDLE = core::ptr::null_mut();
    let mut SubKeyHandle: HANDLE = core::ptr::null_mut();
    let mut StringLength: SIZE_T;
    let mut Disposition: ULONG = 0;
    let mut Status: NTSTATUS;

    StringLength = wcslen(lpBuffer) as SIZE_T;
    if (StringLength as usize) > ((MAXULONG as usize).wrapping_div(core::mem::size_of::<WCHAR>())).wrapping_sub(1) {
        return FALSE;
    }

    RtlInitUnicodeString(&mut KeyName, RegistryKey);
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
    ObjectAttributes.ObjectName = &mut KeyName;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = NtOpenKey(
        &mut KeyHandle,
        KEY_WRITE,
        &mut ObjectAttributes,
    );
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return FALSE;
    }

    RtlInitUnicodeString(&mut KeyName, SubKey);
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = KeyHandle;
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
    ObjectAttributes.ObjectName = &mut KeyName;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = NtCreateKey(
        &mut SubKeyHandle,
        KEY_WRITE,
        &mut ObjectAttributes,
        0,
        core::ptr::null_mut(),
        REG_OPTION_VOLATILE,
        &mut Disposition,
    );
    if !NT_SUCCESS(Status) {
        NtClose(KeyHandle);
        BaseSetLastNTError(Status);
        return FALSE;
    }

    RtlInitUnicodeString(&mut ValueName, ValueNameStr);

    Status = NtSetValueKey(
        SubKeyHandle,
        &mut ValueName,
        0,
        REG_SZ,
        lpBuffer as PVOID,
        ((StringLength as usize).wrapping_add(1)).wrapping_mul(core::mem::size_of::<WCHAR>()) as ULONG,
    );
    if !NT_SUCCESS(Status) {
        NtClose(SubKeyHandle);
        NtClose(KeyHandle);
        BaseSetLastNTError(Status);
        return FALSE;
    }

    NtFlushKey(SubKeyHandle);
    NtClose(SubKeyHandle);
    NtClose(KeyHandle);

    return TRUE;
}

#[no_mangle]
pub unsafe extern "system" fn GetComputerNameExW(
    NameType: COMPUTER_NAME_FORMAT,
    lpBuffer: LPWSTR,
    nSize: LPDWORD,
) -> BOOL {
    let mut ResultString: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut DomainPart: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut QueryTable: [RTL_QUERY_REGISTRY_TABLE; 2] = [
        RTL_QUERY_REGISTRY_TABLE {
            QueryRoutine: None,
            Flags: 0,
            Name: core::ptr::null(),
            EntryContext: core::ptr::null_mut(),
            DefaultType: 0,
            DefaultData: core::ptr::null_mut(),
            DefaultLength: 0,
        },
        RTL_QUERY_REGISTRY_TABLE {
            QueryRoutine: None,
            Flags: 0,
            Name: core::ptr::null(),
            EntryContext: core::ptr::null_mut(),
            DefaultType: 0,
            DefaultData: core::ptr::null_mut(),
            DefaultLength: 0,
        },
    ];
    let mut Status: NTSTATUS = 0;
    let mut ret: BOOL = 1;
    let mut HostSize: DWORD = 0;

    if nSize.is_null() || (lpBuffer.is_null() && *nSize > 0) {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    match NameType as u32 {
        0 => {
            ret = GetComputerNameFromRegistry(
                w!("\\Registry\\Machine\\System\\CurrentControlSet\\Control\\ComputerName\\ActiveComputerName").as_ptr(),
                w!("ComputerName").as_ptr(),
                lpBuffer,
                nSize,
            );
            if ret == 0 && GetLastError() != ERROR_MORE_DATA {
                ret = GetComputerNameFromRegistry(
                    w!("\\Registry\\Machine\\System\\CurrentControlSet\\Control\\ComputerName\\ComputerName").as_ptr(),
                    w!("ComputerName").as_ptr(),
                    lpBuffer,
                    nSize,
                );
                if ret != 0 {
                    ret = SetActiveComputerNameToRegistry(
                        w!("\\Registry\\Machine\\System\\CurrentControlSet\\Control\\ComputerName").as_ptr(),
                        w!("ActiveComputerName").as_ptr(),
                        w!("ComputerName").as_ptr(),
                        lpBuffer,
                    );
                }
            }
            return ret;
        }
        2 => {
            return GetComputerNameFromRegistry(
                w!("\\Registry\\Machine\\System\\CurrentControlSet\\Services\\Tcpip\\Parameters").as_ptr(),
                w!("Domain").as_ptr(),
                lpBuffer,
                nSize,
            );
        }
        3 => {
            ResultString.Length = 0;
            ResultString.MaximumLength = (*nSize as USHORT).wrapping_mul(core::mem::size_of::<WCHAR>() as USHORT);
            ResultString.Buffer = lpBuffer;

            core::ptr::write_bytes(
                &mut QueryTable as *mut _ as *mut u8,
                0,
                core::mem::size_of::<[RTL_QUERY_REGISTRY_TABLE; 2]>(),
            );
            RtlInitUnicodeString(&mut DomainPart, core::ptr::null());
            QueryTable[0].Name = w!("HostName").as_ptr();
            QueryTable[0].Flags = RTL_QUERY_REGISTRY_DIRECT;
            QueryTable[0].EntryContext = &mut DomainPart as *mut _ as *mut core::ffi::c_void;

            Status = RtlQueryRegistryValues(
                RTL_REGISTRY_ABSOLUTE,
                w!("\\Registry\\Machine\\System\\CurrentControlSet\\Services\\Tcpip\\Parameters").as_ptr(),
                &mut QueryTable as *mut _,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );

            if NT_SUCCESS(Status) {
                Status = RtlAppendUnicodeStringToString(&mut ResultString, &DomainPart);
                HostSize = DomainPart.Length as DWORD;

                if !NT_SUCCESS(Status) {
                    ret = 0;
                }

                RtlAppendUnicodeToString(&mut ResultString, w!(".").as_ptr());
                RtlFreeUnicodeString(&mut DomainPart);

                RtlInitUnicodeString(&mut DomainPart, core::ptr::null());
                QueryTable[0].Name = w!("Domain").as_ptr();
                QueryTable[0].Flags = RTL_QUERY_REGISTRY_DIRECT;
                QueryTable[0].EntryContext = &mut DomainPart as *mut _ as *mut core::ffi::c_void;

                Status = RtlQueryRegistryValues(
                    RTL_REGISTRY_ABSOLUTE,
                    w!("\\Registry\\Machine\\System\\CurrentControlSet\\Services\\Tcpip\\Parameters").as_ptr(),
                    &mut QueryTable as *mut _,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                );

                if NT_SUCCESS(Status) {
                    Status = RtlAppendUnicodeStringToString(&mut ResultString, &DomainPart);
                    if !NT_SUCCESS(Status) || ret == 0 {
                        *nSize = (HostSize as DWORD).wrapping_add(DomainPart.Length as DWORD);
                        SetLastError(ERROR_MORE_DATA);
                        RtlFreeUnicodeString(&mut DomainPart);
                        return 0;
                    }
                    RtlFreeUnicodeString(&mut DomainPart);
                    *nSize = (ResultString.Length as usize)
                        .wrapping_div(core::mem::size_of::<WCHAR>())
                        .wrapping_sub(1) as DWORD;
                    return 1;
                }
            }
            return 0;
        }
        1 => {
            return GetComputerNameFromRegistry(
                w!("\\Registry\\Machine\\System\\CurrentControlSet\\Services\\Tcpip\\Parameters").as_ptr(),
                w!("Hostname").as_ptr(),
                lpBuffer,
                nSize,
            );
        }
        6 => {
            return GetComputerNameFromRegistry(
                w!("\\Registry\\Machine\\System\\CurrentControlSet\\Services\\Tcpip\\Parameters").as_ptr(),
                w!("NV Domain").as_ptr(),
                lpBuffer,
                nSize,
            );
        }
        7 => {
            return GetComputerNameExW(ComputerNameDnsFullyQualified, lpBuffer, nSize);
        }
        5 => {
            return GetComputerNameFromRegistry(
                w!("\\Registry\\Machine\\System\\CurrentControlSet\\Services\\Tcpip\\Parameters").as_ptr(),
                w!("NV Hostname").as_ptr(),
                lpBuffer,
                nSize,
            );
        }
        4 => {
            return GetComputerNameExW(ComputerNameNetBIOS, lpBuffer, nSize);
        }
        8 => {
            return 0;
        }
        _ => {}
    }

    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn GetComputerNameExA(
    NameType: COMPUTER_NAME_FORMAT,
    lpBuffer: LPSTR,
    nSize: LPDWORD,
) -> BOOL {
    let mut UnicodeString: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut AnsiString: ANSI_STRING = ANSI_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut Result: BOOL = 0;
    let mut TempBuffer: PWCHAR = core::ptr::null_mut();

    if nSize.is_null() || (lpBuffer.is_null() && *nSize > 0) {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    if *nSize > 0 {
        TempBuffer = RtlAllocateHeap(
            RtlGetProcessHeap(),
            0,
            (*nSize as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
        ) as PWCHAR;
        if TempBuffer.is_null() {
            SetLastError(ERROR_NOT_ENOUGH_MEMORY);
            return 0;
        }
    }

    AnsiString.MaximumLength = *nSize as USHORT;
    AnsiString.Length = 0;
    AnsiString.Buffer = lpBuffer;

    Result = GetComputerNameExW(NameType, TempBuffer, nSize);

    if Result != 0 {
        UnicodeString.MaximumLength = ((*nSize as USHORT as usize)
            .wrapping_mul(core::mem::size_of::<WCHAR>())
            .wrapping_add(core::mem::size_of::<WCHAR>())) as USHORT;
        UnicodeString.Length = ((*nSize as USHORT as usize)
            .wrapping_mul(core::mem::size_of::<WCHAR>())) as USHORT;
        UnicodeString.Buffer = TempBuffer;

        RtlUnicodeStringToAnsiString(
            &mut AnsiString,
            &UnicodeString as *const _UNICODE_STRING,
            0,
        );
    }

    RtlFreeHeap(RtlGetProcessHeap(), 0, TempBuffer as PVOID);

    return Result;
}

#[no_mangle]
pub unsafe extern "system" fn GetComputerNameA(
    lpBuffer: LPSTR,
    lpnSize: LPDWORD,
) -> BOOL {
    let mut ret: BOOL = 0;
    ret = GetComputerNameExA(ComputerNameNetBIOS, lpBuffer, lpnSize);
    if ret == 0 && GetLastError() == ERROR_MORE_DATA as u32 {
        SetLastError(ERROR_BUFFER_OVERFLOW as u32);
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn GetComputerNameW(
    lpBuffer: LPWSTR,
    lpnSize: LPDWORD,
) -> BOOL {
    let ret: BOOL;
    ret = GetComputerNameExW(ComputerNameNetBIOS, lpBuffer, lpnSize);
    if ret == 0 && GetLastError() == ERROR_MORE_DATA as u32 {
        SetLastError(ERROR_BUFFER_OVERFLOW as u32);
    }
    return ret;
}
unsafe fn BaseVerifyDnsName(lpDnsName: LPCWSTR) -> BOOL {
    let mut hDNSAPI: HINSTANCE = core::ptr::null_mut();
    let mut fnValidate: FN_DnsValidateName_W = None;
    let mut Status: NTSTATUS = 0;
    let mut ret: BOOL = FALSE;

    hDNSAPI = LoadLibraryW(w!("dnsapi.dll").as_ptr());
    if hDNSAPI.is_null() {
        return FALSE;
    }

    fnValidate = core::mem::transmute::<FARPROC, FN_DnsValidateName_W>(
        GetProcAddress(
            hDNSAPI,
            b"DnsValidateName_W\0" as *const u8 as LPCSTR,
        ),
    );
    if fnValidate.is_some() {
        Status = fnValidate.expect("non-null function pointer")(lpDnsName, DnsNameHostnameLabel);
        if Status == STATUS_SUCCESS || Status == DNS_ERROR_NON_RFC_NAME as ::core::ffi::c_long {
            ret = TRUE as BOOL;
        }
    }

    FreeLibrary(hDNSAPI as HMODULE);

    return ret;
}
unsafe fn IsValidComputerName(
    NameType: COMPUTER_NAME_FORMAT,
    lpComputerName: LPCWSTR,
) -> BOOL {
    let mut Length: size_t = 0;
    static s_szInvalidChars: [WCHAR; 46] = [
        0x0022, 0x002F, 0x005C, 0x005B, 0x005D, 0x003A, 0x007C, 0x003C,
        0x003E, 0x002B, 0x003D, 0x003B, 0x002C, 0x003F, 0x0001, 0x0002,
        0x0003, 0x0004, 0x0005, 0x0006, 0x0007, 0x0008, 0x0009, 0x000A,
        0x000B, 0x000C, 0x000D, 0x000E, 0x000F, 0x0010, 0x0011, 0x0012,
        0x0013, 0x0014, 0x0015, 0x0016, 0x0017, 0x0018, 0x0019, 0x001A,
        0x001B, 0x001C, 0x001D, 0x001E, 0x001F, 0x0000,
    ];

    if lpComputerName.is_null() {
        return FALSE;
    }

    if !NT_SUCCESS(RtlStringCchLengthW(
        lpComputerName,
        65,
        &mut Length as *mut size_t,
    )) {
        return FALSE;
    }

    if Length == 0 && NameType != ComputerNamePhysicalDnsDomain {
        return FALSE;
    }

    if Length > 0
        && (*lpComputerName.offset(0) as u16 == ' ' as u16
            || *lpComputerName.offset((Length.wrapping_sub(1)) as isize) as u16 == ' ' as u16)
    {
        return FALSE;
    }

    if wcscspn(
        lpComputerName,
        &s_szInvalidChars as *const WCHAR as *const u16,
    ) < Length
    {
        return FALSE;
    }

    match NameType {
        ComputerNamePhysicalNetBIOS => {
            if Length > MAX_COMPUTERNAME_LENGTH as size_t {
                return FALSE;
            }
            return TRUE;
        }
        ComputerNamePhysicalDnsDomain => {
            if Length != 0 {
                return BaseVerifyDnsName(lpComputerName);
            }
            return TRUE;
        }
        ComputerNamePhysicalDnsHostname => {
            return BaseVerifyDnsName(lpComputerName);
        }
        _ => {
            return FALSE;
        }
    }
}
unsafe fn SetComputerNameToRegistry(
    RegistryKey: LPCWSTR,
    ValueNameStr: LPCWSTR,
    lpBuffer: LPCWSTR,
) -> BOOL {
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut KeyName: _UNICODE_STRING = core::mem::zeroed();
    let mut ValueName: _UNICODE_STRING = core::mem::zeroed();
    let mut KeyHandle: HANDLE = core::ptr::null_mut();
    let mut StringLength: SIZE_T;
    let mut Status: NTSTATUS;

    StringLength = wcslen(lpBuffer) as SIZE_T;
    if (StringLength as usize) > (MAXULONG as usize).wrapping_div(core::mem::size_of::<WCHAR>()).wrapping_sub(1) {
        return FALSE;
    }

    RtlInitUnicodeString(&mut KeyName, RegistryKey);
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
    ObjectAttributes.ObjectName = &mut KeyName;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = NtOpenKey(&mut KeyHandle, KEY_WRITE, &mut ObjectAttributes);
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return FALSE;
    }

    RtlInitUnicodeString(&mut ValueName, ValueNameStr);

    Status = NtSetValueKey(
        KeyHandle,
        &mut ValueName,
        0,
        REG_SZ,
        lpBuffer as PVOID,
        ((StringLength + 1) * core::mem::size_of::<WCHAR>() as SIZE_T) as ULONG,
    );
    if !NT_SUCCESS(Status) {
        NtClose(KeyHandle);
        BaseSetLastNTError(Status);
        return FALSE;
    }

    NtFlushKey(KeyHandle);
    NtClose(KeyHandle);

    SetLastError(ERROR_SUCCESS);
    return TRUE;
}

#[no_mangle]
pub unsafe extern "system" fn SetComputerNameA(
    lpComputerName: LPCSTR,
) -> BOOL {
    return SetComputerNameExA(ComputerNamePhysicalNetBIOS, lpComputerName);
}

#[no_mangle]
pub unsafe extern "system" fn SetComputerNameW(lpComputerName: LPCWSTR) -> BOOL {
    return SetComputerNameExW(ComputerNamePhysicalNetBIOS, lpComputerName);
}

#[no_mangle]
pub unsafe extern "system" fn SetComputerNameExA(
    NameType: COMPUTER_NAME_FORMAT,
    lpBuffer: LPCSTR,
) -> BOOL {
    let mut Buffer: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut bResult: BOOL = 0;

    RtlCreateUnicodeStringFromAsciiz(&raw mut Buffer, lpBuffer as LPSTR);

    bResult = SetComputerNameExW(NameType, Buffer.Buffer);

    RtlFreeUnicodeString(&raw mut Buffer);

    return bResult;
}

#[no_mangle]
pub unsafe extern "system" fn SetComputerNameExW(
    NameType: COMPUTER_NAME_FORMAT,
    lpBuffer: LPCWSTR,
) -> BOOL {
    let mut szShortName: [WCHAR; MAX_COMPUTERNAME_LENGTH as usize + 1] = [0; MAX_COMPUTERNAME_LENGTH as usize + 1];
    let mut ret1: BOOL;
    let mut ret2: BOOL;

    if IsValidComputerName(NameType, lpBuffer) == 0 {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return 0;
    }

    match NameType as u32 {
        6 => {
            return SetComputerNameToRegistry(
                w!("\\Registry\\Machine\\System\\CurrentControlSet\\Services\\Tcpip\\Parameters\0").as_ptr() as LPCWSTR,
                w!("NV Domain\0").as_ptr() as LPCWSTR,
                lpBuffer,
            );
        }
        5 => {
            ret1 = SetComputerNameToRegistry(
                w!("\\Registry\\Machine\\System\\CurrentControlSet\\Services\\Tcpip\\Parameters\0").as_ptr() as LPCWSTR,
                w!("NV Hostname\0").as_ptr() as LPCWSTR,
                lpBuffer,
            );

            RtlStringCchCopyNW(
                &raw mut szShortName as NTSTRSAFE_PWSTR,
                (core::mem::size_of::<[WCHAR; MAX_COMPUTERNAME_LENGTH as usize + 1]>() as size_t)
                    .wrapping_div(core::mem::size_of::<WCHAR>() as size_t),
                lpBuffer as STRSAFE_LPCWSTR,
                MAX_COMPUTERNAME_LENGTH as size_t,
            );
            ret2 = SetComputerNameToRegistry(
                w!("\\Registry\\Machine\\System\\CurrentControlSet\\Control\\ComputerName\\ComputerName\0").as_ptr() as LPCWSTR,
                w!("ComputerName\0").as_ptr() as LPCWSTR,
                &raw mut szShortName as *mut WCHAR as LPCWSTR,
            );
            return (ret1 != 0 && ret2 != 0) as BOOL;
        }
        4 => {
            RtlStringCchCopyNW(
                &raw mut szShortName as NTSTRSAFE_PWSTR,
                (core::mem::size_of::<[WCHAR; MAX_COMPUTERNAME_LENGTH as usize + 1]>() as size_t)
                    .wrapping_div(core::mem::size_of::<WCHAR>() as size_t),
                lpBuffer as STRSAFE_LPCWSTR,
                MAX_COMPUTERNAME_LENGTH as size_t,
            );
            return SetComputerNameToRegistry(
                w!("\\Registry\\Machine\\System\\CurrentControlSet\\Control\\ComputerName\\ComputerName\0").as_ptr() as LPCWSTR,
                w!("ComputerName\0").as_ptr() as LPCWSTR,
                &raw mut szShortName as *mut WCHAR as LPCWSTR,
            );
        }
        _ => {
            SetLastError(ERROR_INVALID_PARAMETER as DWORD);
            return 0;
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn DnsHostnameToComputerNameA(
    Hostname: LPCSTR,
    ComputerName: LPSTR,
    nSize: LPDWORD,
) -> BOOL {
    let mut len: DWORD = 0;

    if Hostname.is_null() || nSize.is_null() {
        return 0;
    }

    len = lstrlenA(Hostname) as DWORD;

    if len > MAX_COMPUTERNAME_LENGTH as DWORD {
        len = MAX_COMPUTERNAME_LENGTH as DWORD;
    }

    if *nSize < len {
        *nSize = len;
        return 0;
    }

    if ComputerName.is_null() {
        return 0;
    }

    memcpy(
        ComputerName as *mut core::ffi::c_void,
        Hostname as *const core::ffi::c_void,
        len as usize,
    );
    *ComputerName.offset((len as DWORD).wrapping_add(1) as isize) = 0i8;
    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn DnsHostnameToComputerNameW(
    hostname: LPCWSTR,
    computername: LPWSTR,
    size: LPDWORD,
) -> BOOL {
    let mut len: DWORD = 0;

    if hostname.is_null() || size.is_null() {
        return FALSE;
    }

    len = lstrlenW(hostname) as DWORD;

    if len > MAX_COMPUTERNAME_LENGTH as DWORD {
        len = MAX_COMPUTERNAME_LENGTH as DWORD;
    }

    if *size < len {
        *size = len;
        return FALSE;
    }

    if computername.is_null() {
        return FALSE;
    }

    memcpy(
        computername as *mut core::ffi::c_void,
        hostname as *const core::ffi::c_void,
        (len as usize).wrapping_mul(core::mem::size_of::<WCHAR>()),
    );

    *computername.offset((len as isize).wrapping_add(1)) = 0 as WCHAR;

    return TRUE;
}
#[no_mangle]
pub unsafe extern "system" fn AddLocalAlternateComputerNameA(
    lpName: LPSTR,
    Status: PNTSTATUS,
) -> DWORD {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED as DWORD);
    if DbgPrint(
        b"(%s:%d) %s() is unimplemented!()\n\0" as *const u8 as *const i8,
        b"dll/win32/kernel32/client/compname.c\0"
            as *const u8 as *const i8,
        765i32,
        b"AddLocalAlternateComputerNameA\0" as *const u8 as *const i8,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const i8,
            b"dll/win32/kernel32/client/compname.c\0"
                as *const u8 as *const i8,
            765i32,
        );
    }
    return 0;
}
#[no_mangle]
pub unsafe extern "system" fn AddLocalAlternateComputerNameW(
    lpName: LPWSTR,
    Status: PNTSTATUS,
) -> DWORD {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED as DWORD);
    if DbgPrint(
        b"(%s:%d) %s() is unimplemented!()\n\0" as *const u8 as LPCSTR,
        b"dll/win32/kernel32/client/compname.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        773i32,
        b"AddLocalAlternateComputerNameW\0" as *const u8 as *const ::core::ffi::c_char,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as LPCSTR,
            b"dll/win32/kernel32/client/compname.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            773i32,
        );
    }
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn EnumerateLocalComputerNamesA(
    pUnknown: PVOID,
    Size: DWORD,
    lpBuffer: LPSTR,
    lpnSize: LPDWORD,
) -> DWORD {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED as DWORD);
    if DbgPrint(
        b"(%s:%d) %s() is unimplemented!()\n\0".as_ptr() as PCSTR,
        b"dll/win32/kernel32/client/compname.c\0"
            .as_ptr() as *const i8,
        781i32,
        b"EnumerateLocalComputerNamesA\0".as_ptr() as *const i8,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
            b"dll/win32/kernel32/client/compname.c\0"
                .as_ptr() as *const i8,
            781i32,
        );
    }
    return ERROR_CALL_NOT_IMPLEMENTED as DWORD;
}

#[no_mangle]
pub unsafe extern "system" fn EnumerateLocalComputerNamesW(
    pUnknown: PVOID,
    Size: DWORD,
    lpBuffer: LPWSTR,
    lpnSize: LPDWORD,
) -> DWORD {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED as DWORD);
    if DbgPrint(
        b"(%s:%d) %s() is unimplemented!()\n\0" as *const u8 as PCSTR,
        b"dll/win32/kernel32/client/compname.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        789 as ::core::ffi::c_int,
        b"EnumerateLocalComputerNamesW\0" as *const u8 as *const ::core::ffi::c_char,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
            b"dll/win32/kernel32/client/compname.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            789 as ::core::ffi::c_int,
        );
    }
    return ERROR_CALL_NOT_IMPLEMENTED as DWORD;
}

#[no_mangle]
pub unsafe extern "system" fn RemoveLocalAlternateComputerNameA(
    lpName: LPSTR,
    _Unknown: DWORD,
) -> DWORD {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED as DWORD);
    if DbgPrint(
        b"(%s:%d) %s() is unimplemented!()\n\0" as *const u8 as *const i8,
        b"dll/win32/kernel32/client/compname.c\0"
            as *const u8 as *const i8,
        797i32,
        b"RemoveLocalAlternateComputerNameA\0" as *const u8 as *const i8,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const i8,
            b"dll/win32/kernel32/client/compname.c\0"
                as *const u8 as *const i8,
            797i32,
        );
    }
    return ERROR_CALL_NOT_IMPLEMENTED as DWORD;
}

#[no_mangle]
pub unsafe extern "system" fn RemoveLocalAlternateComputerNameW(
    lpName: LPWSTR,
    _Unknown: DWORD,
) -> DWORD {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED as DWORD);
    if DbgPrint(
        b"(%s:%d) %s() is unimplemented!()\n\0" as *const u8 as *const i8,
        b"dll/win32/kernel32/client/compname.c\0"
            as *const u8 as *const i8,
        805i32,
        b"RemoveLocalAlternateComputerNameW\0" as *const u8 as *const i8,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const i8,
            b"dll/win32/kernel32/client/compname.c\0"
                as *const u8 as *const i8,
            805i32,
        );
    }
    return ERROR_CALL_NOT_IMPLEMENTED as DWORD;
}

#[no_mangle]
pub unsafe extern "system" fn SetLocalPrimaryComputerNameA(
    Unknown1: DWORD,
    Unknown2: DWORD,
) -> BOOL {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED as DWORD);
    if DbgPrint(
        b"(%s:%d) %s() is unimplemented!()\n\0".as_ptr() as PCSTR,
        b"dll/win32/kernel32/client/compname.c\0"
            .as_ptr() as *const ::core::ffi::c_char,
        817i32,
        b"SetLocalPrimaryComputerNameA\0".as_ptr() as *const ::core::ffi::c_char,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
            b"dll/win32/kernel32/client/compname.c\0"
                .as_ptr() as *const ::core::ffi::c_char,
            817i32,
        );
    }
    return 0i32;
}

#[no_mangle]
pub unsafe extern "system" fn SetLocalPrimaryComputerNameW(
    Unknown1: DWORD,
    Unknown2: DWORD,
) -> BOOL {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED as DWORD);
    if DbgPrint(
        b"(%s:%d) %s() is unimplemented!()\n\0" as *const u8 as PCSTR,
        b"dll/win32/kernel32/client/compname.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        829 as ::core::ffi::c_int,
        b"SetLocalPrimaryComputerNameW\0" as *const u8 as *const ::core::ffi::c_char,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
            b"dll/win32/kernel32/client/compname.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            829 as ::core::ffi::c_int,
        );
    }
    return 0;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  *  ReactOS kernel
//  *  Copyright (C) 2003 ReactOS Team
//  *
//  *  This program is free software; you can redistribute it and/or modify
//  *  it under the terms of the GNU General Public License as published by
//  *  the Free Software Foundation; either version 2 of the License, or
//  *  (at your option) any later version.
//  *
//  *  This program is distributed in the hope that it will be useful,
//  *  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  *  GNU General Public License for more details.
//  *
//  *  You should have received a copy of the GNU General Public License
//  *  along with this program; if not, write to the Free Software
//  *  Foundation, Inc., 675 Mass Ave, Cambridge, MA 02139, USA.
//  */
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS system libraries
//  * PURPOSE:         Computer name functions
//  * FILE:            dll/win32/kernel32/client/compname.c
//  * PROGRAMERS:      Eric Kohl
//  *                  Katayama Hirofumi MZ
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <k32.h>
// #include <windns.h>
//
// #define NDEBUG
// #include <debug.h>
//
// typedef NTSTATUS (WINAPI *FN_DnsValidateName_W)(LPCWSTR, DNS_NAME_FORMAT);
//
// /* FUNCTIONS *****************************************************************/
//
// static
// BOOL
// GetComputerNameFromRegistry(LPWSTR RegistryKey,
//                             LPWSTR ValueNameStr,
//                             LPWSTR lpBuffer,
//                             LPDWORD nSize)
// {
//     PKEY_VALUE_PARTIAL_INFORMATION KeyInfo;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     UNICODE_STRING KeyName;
//     UNICODE_STRING ValueName;
//     HANDLE KeyHandle;
//     ULONG KeyInfoSize;
//     ULONG ReturnSize;
//     NTSTATUS Status;
//
//     if (lpBuffer != NULL && *nSize > 0)
//         lpBuffer[0] = 0;
//
//     RtlInitUnicodeString(&KeyName, RegistryKey);
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &KeyName,
//                                OBJ_CASE_INSENSITIVE,
//                                NULL,
//                                NULL);
//
//     Status = NtOpenKey(&KeyHandle,
//                        KEY_READ,
//                        &ObjectAttributes);
//     if (!NT_SUCCESS(Status))
//     {
//         BaseSetLastNTError (Status);
//         return FALSE;
//     }
//
//     KeyInfoSize = sizeof(KEY_VALUE_PARTIAL_INFORMATION) + *nSize * sizeof(WCHAR);
//     KeyInfo = RtlAllocateHeap(RtlGetProcessHeap(), 0, KeyInfoSize);
//     if (KeyInfo == NULL)
//     {
//         NtClose(KeyHandle);
//         SetLastError(ERROR_OUTOFMEMORY);
//         return FALSE;
//     }
//
//     RtlInitUnicodeString(&ValueName, ValueNameStr);
//
//     Status = NtQueryValueKey(KeyHandle,
//                              &mut ValueName,
//                              KeyValuePartialInformation,
//                              KeyInfo,
//                              KeyInfoSize,
//                              &ReturnSize);
//
//     NtClose(KeyHandle);
//
//     if (!NT_SUCCESS(Status))
//     {
//         *nSize = (ReturnSize - FIELD_OFFSET(KEY_VALUE_PARTIAL_INFORMATION, Data)) / sizeof(WCHAR);
//         goto failed;
//     }
//
//     if (KeyInfo->Type != REG_SZ)
//     {
//         Status = STATUS_UNSUCCESSFUL;
//         goto failed;
//     }
//
//     if (!lpBuffer || *nSize < (KeyInfo->DataLength / sizeof(WCHAR)))
//     {
//         *nSize = (ReturnSize - FIELD_OFFSET(KEY_VALUE_PARTIAL_INFORMATION, Data)) / sizeof(WCHAR);
//         Status = STATUS_BUFFER_OVERFLOW;
//         goto failed;
//     }
//
//     *nSize = KeyInfo->DataLength / sizeof(WCHAR) - 1;
//     RtlCopyMemory(lpBuffer, KeyInfo->Data, KeyInfo->DataLength);
//     lpBuffer[*nSize] = 0;
//
//     RtlFreeHeap(RtlGetProcessHeap(), 0, KeyInfo);
//
//     return TRUE;
//
// failed:
//     RtlFreeHeap(RtlGetProcessHeap(), 0, KeyInfo);
//     BaseSetLastNTError(Status);
//     return FALSE;
// }
//
//
// static
// BOOL
// SetActiveComputerNameToRegistry(LPCWSTR RegistryKey,
//                                 LPCWSTR SubKey,
//                                 LPCWSTR ValueNameStr,
//                                 LPCWSTR lpBuffer)
// {
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     UNICODE_STRING KeyName;
//     UNICODE_STRING ValueName;
//     HANDLE KeyHandle, SubKeyHandle;
//     SIZE_T StringLength;
//     ULONG Disposition;
//     NTSTATUS Status;
//
//     StringLength = wcslen(lpBuffer);
//     if (StringLength > ((MAXULONG / sizeof(WCHAR)) - 1))
//     {
//         return FALSE;
//     }
//
//     RtlInitUnicodeString(&KeyName, RegistryKey);
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &KeyName,
//                                OBJ_CASE_INSENSITIVE,
//                                NULL,
//                                NULL);
//
//     Status = NtOpenKey(&KeyHandle,
//                        KEY_WRITE,
//                        &ObjectAttributes);
//     if (!NT_SUCCESS(Status))
//     {
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     RtlInitUnicodeString(&KeyName, SubKey);
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &KeyName,
//                                OBJ_CASE_INSENSITIVE,
//                                KeyHandle,
//                                NULL);
//
//     Status = NtCreateKey(&SubKeyHandle,
//                          KEY_WRITE,
//                          &ObjectAttributes,
//                          0,
//                          NULL,
//                          REG_OPTION_VOLATILE,
//                          &Disposition);
//     if (!NT_SUCCESS(Status))
//     {
//         NtClose(KeyHandle);
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     RtlInitUnicodeString(&ValueName, ValueNameStr);
//
//     Status = NtSetValueKey(SubKeyHandle,
//                            &mut ValueName,
//                            0,
//                            REG_SZ,
//                            (PVOID)lpBuffer,
//                            (StringLength + 1) * sizeof(WCHAR));
//     if (!NT_SUCCESS(Status))
//     {
//         NtClose(SubKeyHandle);
//         NtClose(KeyHandle);
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     NtFlushKey(SubKeyHandle);
//     NtClose(SubKeyHandle);
//     NtClose(KeyHandle);
//
//     return TRUE;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// GetComputerNameExW(COMPUTER_NAME_FORMAT NameType,
//                    LPWSTR lpBuffer,
//                    LPDWORD nSize)
// {
//     UNICODE_STRING ResultString;
//     UNICODE_STRING DomainPart;
//     RTL_QUERY_REGISTRY_TABLE QueryTable[2];
//     NTSTATUS Status;
//     BOOL ret = TRUE;
//     DWORD HostSize;
//
//     if ((nSize == NULL) ||
//         (lpBuffer == NULL && *nSize > 0))
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     switch (NameType)
//     {
//         case ComputerNameNetBIOS:
//             ret = GetComputerNameFromRegistry(L"\\Registry\\Machine\\System\\CurrentControlSet"
//                                                L"\\Control\\ComputerName\\ActiveComputerName",
//                                                L"ComputerName",
//                                                lpBuffer,
//                                                nSize);
//             if ((ret == FALSE) &&
//                 (GetLastError() != ERROR_MORE_DATA))
//             {
//                 ret = GetComputerNameFromRegistry(L"\\Registry\\Machine\\System\\CurrentControlSet"
//                                                   L"\\Control\\ComputerName\\ComputerName",
//                                                   L"ComputerName",
//                                                   lpBuffer,
//                                                   nSize);
//                 if (ret)
//                 {
//                     ret = SetActiveComputerNameToRegistry(L"\\Registry\\Machine\\System\\CurrentControlSet"
//                                                           L"\\Control\\ComputerName",
//                                                           L"ActiveComputerName",
//                                                           L"ComputerName",
//                                                           lpBuffer);
//                 }
//             }
//             return ret;
//
//         case ComputerNameDnsDomain:
//             return GetComputerNameFromRegistry(L"\\Registry\\Machine\\System\\CurrentControlSet"
//                                                L"\\Services\\Tcpip\\Parameters",
//                                                L"Domain",
//                                                lpBuffer,
//                                                nSize);
//
//         case ComputerNameDnsFullyQualified:
//             ResultString.Length = 0;
//             ResultString.MaximumLength = (USHORT)*nSize * sizeof(WCHAR);
//             ResultString.Buffer = lpBuffer;
//
//             RtlZeroMemory(QueryTable, sizeof(QueryTable));
//             RtlInitUnicodeString(&DomainPart, NULL);
//
//             QueryTable[0].Name = L"HostName";
//             QueryTable[0].Flags = RTL_QUERY_REGISTRY_DIRECT;
//             QueryTable[0].EntryContext = &DomainPart;
//
//             Status = RtlQueryRegistryValues(RTL_REGISTRY_ABSOLUTE,
//                                             L"\\Registry\\Machine\\System"
//                                             L"\\CurrentControlSet\\Services\\Tcpip"
//                                             L"\\Parameters",
//                                             QueryTable,
//                                             NULL,
//                                             NULL);
//
//             if (NT_SUCCESS(Status))
//             {
//                 Status = RtlAppendUnicodeStringToString(&ResultString, &DomainPart);
//                 HostSize = DomainPart.Length;
//
//                 if (!NT_SUCCESS(Status))
//                 {
//                     ret = FALSE;
//                 }
//
//                 RtlAppendUnicodeToString(&ResultString, L".");
//                 RtlFreeUnicodeString(&DomainPart);
//
//                 RtlInitUnicodeString(&DomainPart, NULL);
//                 QueryTable[0].Name = L"Domain";
//                 QueryTable[0].Flags = RTL_QUERY_REGISTRY_DIRECT;
//                 QueryTable[0].EntryContext = &DomainPart;
//
//                 Status = RtlQueryRegistryValues(RTL_REGISTRY_ABSOLUTE,
//                                                 L"\\Registry\\Machine\\System"
//                                                 L"\\CurrentControlSet\\Services\\Tcpip"
//                                                 L"\\Parameters",
//                                                 QueryTable,
//                                                 NULL,
//                                                 NULL);
//
//                 if (NT_SUCCESS(Status))
//                 {
//                     Status = RtlAppendUnicodeStringToString(&ResultString, &DomainPart);
//                     if ((!NT_SUCCESS(Status)) || (!ret))
//                     {
//                         *nSize = HostSize + DomainPart.Length;
//                         SetLastError(ERROR_MORE_DATA);
//                         RtlFreeUnicodeString(&DomainPart);
//                         return FALSE;
//                     }
//                     RtlFreeUnicodeString(&DomainPart);
//                     *nSize = ResultString.Length / sizeof(WCHAR) - 1;
//                     return TRUE;
//                 }
//             }
//             return FALSE;
//
//         case ComputerNameDnsHostname:
//             return GetComputerNameFromRegistry(L"\\Registry\\Machine\\System\\CurrentControlSet"
//                                                L"\\Services\\Tcpip\\Parameters",
//                                                L"Hostname",
//                                                lpBuffer,
//                                                nSize);
//
//         case ComputerNamePhysicalDnsDomain:
//             return GetComputerNameFromRegistry(L"\\Registry\\Machine\\System\\CurrentControlSet"
//                                                L"\\Services\\Tcpip\\Parameters",
//                                                L"NV Domain",
//                                                lpBuffer,
//                                                nSize);
//
//         /* XXX Redo this */
//         case ComputerNamePhysicalDnsFullyQualified:
//             return GetComputerNameExW(ComputerNameDnsFullyQualified,
//                                       lpBuffer,
//                                       nSize);
//
//         case ComputerNamePhysicalDnsHostname:
//             return GetComputerNameFromRegistry(L"\\Registry\\Machine\\System\\CurrentControlSet"
//                                                L"\\Services\\Tcpip\\Parameters",
//                                                L"NV Hostname",
//                                                lpBuffer,
//                                                nSize);
//
//         /* XXX Redo this */
//         case ComputerNamePhysicalNetBIOS:
//             return GetComputerNameExW(ComputerNameNetBIOS,
//                                       lpBuffer,
//                                       nSize);
//
//         case ComputerNameMax:
//             return FALSE;
//     }
//
//     return FALSE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// GetComputerNameExA(COMPUTER_NAME_FORMAT NameType,
//                    LPSTR lpBuffer,
//                    LPDWORD nSize)
// {
//     UNICODE_STRING UnicodeString;
//     ANSI_STRING AnsiString;
//     BOOL Result;
//     PWCHAR TempBuffer = NULL;
//
//     if ((nSize == NULL) ||
//         (lpBuffer == NULL && *nSize > 0))
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     if (*nSize > 0)
//     {
//         TempBuffer = RtlAllocateHeap(RtlGetProcessHeap(), 0, *nSize * sizeof(WCHAR));
//         if (!TempBuffer)
//         {
//             SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//             return FALSE;
//         }
//     }
//
//     AnsiString.MaximumLength = (USHORT)*nSize;
//     AnsiString.Length = 0;
//     AnsiString.Buffer = lpBuffer;
//
//     Result = GetComputerNameExW(NameType, TempBuffer, nSize);
//
//     if (Result)
//     {
//         UnicodeString.MaximumLength = (USHORT)*nSize * sizeof(WCHAR) + sizeof(WCHAR);
//         UnicodeString.Length = (USHORT)*nSize * sizeof(WCHAR);
//         UnicodeString.Buffer = TempBuffer;
//
//         RtlUnicodeStringToAnsiString(&AnsiString,
//                                      &UnicodeString,
//                                      FALSE);
//     }
//
//     RtlFreeHeap(RtlGetProcessHeap(), 0, TempBuffer);
//
//     return Result;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// GetComputerNameA(LPSTR lpBuffer, LPDWORD lpnSize)
// {
//     BOOL ret;
//
//     ret = GetComputerNameExA(ComputerNameNetBIOS, lpBuffer, lpnSize);
//     if (!ret && GetLastError() == ERROR_MORE_DATA)
//         SetLastError(ERROR_BUFFER_OVERFLOW);
//
//     return ret;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// GetComputerNameW(LPWSTR lpBuffer, LPDWORD lpnSize)
// {
//     BOOL ret;
//
//     ret = GetComputerNameExW(ComputerNameNetBIOS, lpBuffer, lpnSize);
//     if (!ret && GetLastError() == ERROR_MORE_DATA)
//         SetLastError(ERROR_BUFFER_OVERFLOW);
//
//     return ret;
// }
//
// static
// BOOL
// BaseVerifyDnsName(LPCWSTR lpDnsName)
// {
//     HINSTANCE hDNSAPI;
//     FN_DnsValidateName_W fnValidate;
//     NTSTATUS Status;
//     BOOL ret = FALSE;
//
//     hDNSAPI = LoadLibraryW(L"dnsapi.dll");
//     if (hDNSAPI == NULL)
//         return FALSE;
//
//     fnValidate = (FN_DnsValidateName_W)GetProcAddress(hDNSAPI, "DnsValidateName_W");
//     if (fnValidate)
//     {
//         Status = (*fnValidate)(lpDnsName, DnsNameHostnameLabel);
//         if (Status == STATUS_SUCCESS || Status == DNS_ERROR_NON_RFC_NAME)
//             ret = TRUE;
//     }
//
//     FreeLibrary(hDNSAPI);
//
//     return ret;
// }
//
// /*
//  * @implemented
//  */
// static
// BOOL
// IsValidComputerName(COMPUTER_NAME_FORMAT NameType,
//                     LPCWSTR lpComputerName)
// {
//     size_t Length;
//     static const WCHAR s_szInvalidChars[] =
//         L"\"/\\[]:|<>+=;,?"
//         L"\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0A\x0B\x0C\x0D\x0E\x0F"
//         L"\x10\x11\x12\x13\x14\x15\x16\x17\x18\x19\x1A\x1B\x1C\x1D\x1E\x1F";
//
//     if (lpComputerName == NULL)
//         return FALSE;
//
// #define MAX_COMPUTER_NAME_EX 64
//     /* Get string length */
//     if (!NT_SUCCESS(RtlStringCchLengthW(lpComputerName, MAX_COMPUTER_NAME_EX + 1, &Length)))
//         return FALSE;
// #undef MAX_COMPUTER_NAME_EX
//
//     /* An empty name is invalid, except a DNS name */
//     if (Length == 0 && NameType != ComputerNamePhysicalDnsDomain)
//         return FALSE;
//
//     /* Leading or trailing spaces are invalid */
//     if (Length > 0 &&
//         (lpComputerName[0] == L' ' || lpComputerName[Length - 1] == L' '))
//     {
//         return FALSE;
//     }
//
//     /* Check whether the name contains any invalid character */
//     if (wcscspn(lpComputerName, s_szInvalidChars) < Length)
//         return FALSE;
//
//     switch (NameType)
//     {
//         case ComputerNamePhysicalNetBIOS:
//             if (Length > MAX_COMPUTERNAME_LENGTH)
//                 return FALSE;
//             return TRUE;
//
//         case ComputerNamePhysicalDnsDomain:
//             /* An empty DNS name is valid */
//             if (Length != 0)
//                 return BaseVerifyDnsName(lpComputerName);
//             return TRUE;
//
//         case ComputerNamePhysicalDnsHostname:
//             return BaseVerifyDnsName(lpComputerName);
//
//         default:
//             return FALSE;
//     }
// }
//
// static
// BOOL
// SetComputerNameToRegistry(LPCWSTR RegistryKey,
//                           LPCWSTR ValueNameStr,
//                           LPCWSTR lpBuffer)
// {
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     UNICODE_STRING KeyName;
//     UNICODE_STRING ValueName;
//     HANDLE KeyHandle;
//     SIZE_T StringLength;
//     NTSTATUS Status;
//
//     StringLength = wcslen(lpBuffer);
//     if (StringLength > ((MAXULONG / sizeof(WCHAR)) - 1))
//     {
//         return FALSE;
//     }
//
//     RtlInitUnicodeString(&KeyName, RegistryKey);
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &KeyName,
//                                OBJ_CASE_INSENSITIVE,
//                                NULL,
//                                NULL);
//
//     Status = NtOpenKey(&KeyHandle,
//                        KEY_WRITE,
//                        &ObjectAttributes);
//     if (!NT_SUCCESS(Status))
//     {
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     RtlInitUnicodeString(&ValueName, ValueNameStr);
//
//     Status = NtSetValueKey(KeyHandle,
//                            &mut ValueName,
//                            0,
//                            REG_SZ,
//                            (PVOID)lpBuffer,
//                            (StringLength + 1) * sizeof(WCHAR));
//     if (!NT_SUCCESS(Status))
//     {
//         NtClose(KeyHandle);
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     NtFlushKey(KeyHandle);
//     NtClose(KeyHandle);
//
//     SetLastError(ERROR_SUCCESS);
//     return TRUE;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// SetComputerNameA(LPCSTR lpComputerName)
// {
//     return SetComputerNameExA(ComputerNamePhysicalNetBIOS, lpComputerName);
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// SetComputerNameW(LPCWSTR lpComputerName)
// {
//     return SetComputerNameExW(ComputerNamePhysicalNetBIOS, lpComputerName);
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// SetComputerNameExA(COMPUTER_NAME_FORMAT NameType,
//                    LPCSTR lpBuffer)
// {
//     UNICODE_STRING Buffer;
//     BOOL bResult;
//
//     RtlCreateUnicodeStringFromAsciiz(&Buffer, (LPSTR)lpBuffer);
//
//     bResult = SetComputerNameExW(NameType, Buffer.Buffer);
//
//     RtlFreeUnicodeString(&Buffer);
//
//     return bResult;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// SetComputerNameExW(COMPUTER_NAME_FORMAT NameType,
//                    LPCWSTR lpBuffer)
// {
//     WCHAR szShortName[MAX_COMPUTERNAME_LENGTH + 1];
//     BOOL ret1, ret2;
//
//     if (!IsValidComputerName(NameType, lpBuffer))
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     switch (NameType)
//     {
//         case ComputerNamePhysicalDnsDomain:
//             return SetComputerNameToRegistry(L"\\Registry\\Machine\\System\\CurrentControlSet"
//                                              L"\\Services\\Tcpip\\Parameters",
//                                              L"NV Domain",
//                                              lpBuffer);
//
//         case ComputerNamePhysicalDnsHostname:
//             ret1 = SetComputerNameToRegistry(L"\\Registry\\Machine\\System\\CurrentControlSet"
//                                              L"\\Services\\Tcpip\\Parameters",
//                                              L"NV Hostname",
//                                              lpBuffer);
//
//             RtlStringCchCopyNW(szShortName, ARRAYSIZE(szShortName), lpBuffer, MAX_COMPUTERNAME_LENGTH);
//             ret2 = SetComputerNameToRegistry(L"\\Registry\\Machine\\System\\CurrentControlSet"
//                                              L"\\Control\\ComputerName\\ComputerName",
//                                              L"ComputerName",
//                                              szShortName);
//             return (ret1 && ret2);
//
//         case ComputerNamePhysicalNetBIOS:
//             RtlStringCchCopyNW(szShortName, ARRAYSIZE(szShortName), lpBuffer, MAX_COMPUTERNAME_LENGTH);
//             return SetComputerNameToRegistry(L"\\Registry\\Machine\\System\\CurrentControlSet"
//                                              L"\\Control\\ComputerName\\ComputerName",
//                                              L"ComputerName",
//                                              szShortName);
//
//         default:
//             SetLastError(ERROR_INVALID_PARAMETER);
//             return FALSE;
//     }
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// DnsHostnameToComputerNameA(LPCSTR Hostname,
//                            LPSTR ComputerName,
//                            LPDWORD nSize)
// {
//     DWORD len;
//
//     DPRINT("(%s, %p, %p)\n", Hostname, ComputerName, nSize);
//
//     if (!Hostname || !nSize)
//         return FALSE;
//
//     len = lstrlenA(Hostname);
//
//     if (len > MAX_COMPUTERNAME_LENGTH)
//         len = MAX_COMPUTERNAME_LENGTH;
//
//     if (*nSize < len)
//     {
//         *nSize = len;
//         return FALSE;
//     }
//
//     if (!ComputerName) return FALSE;
//
//     memcpy(ComputerName, Hostname, len);
//     ComputerName[len + 1] = 0;
//     return TRUE;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// DnsHostnameToComputerNameW(LPCWSTR hostname,
//                            LPWSTR computername,
//                            LPDWORD size)
// {
//     DWORD len;
//
//     DPRINT("(%s, %p, %p): stub\n", hostname, computername, size);
//
//     if (!hostname || !size) return FALSE;
//     len = lstrlenW(hostname);
//
//     if (len > MAX_COMPUTERNAME_LENGTH)
//         len = MAX_COMPUTERNAME_LENGTH;
//
//     if (*size < len)
//     {
//         *size = len;
//         return FALSE;
//     }
//     if (!computername) return FALSE;
//
//     memcpy(computername, hostname, len * sizeof(WCHAR));
//     computername[len + 1] = 0;
//     return TRUE;
// }
//
// DWORD
// WINAPI
// AddLocalAlternateComputerNameA(LPSTR lpName, PNTSTATUS Status)
// {
//     STUB;
//     return 0;
// }
//
// DWORD
// WINAPI
// AddLocalAlternateComputerNameW(LPWSTR lpName, PNTSTATUS Status)
// {
//     STUB;
//     return 0;
// }
//
// DWORD
// WINAPI
// EnumerateLocalComputerNamesA(PVOID pUnknown, DWORD Size, LPSTR lpBuffer, LPDWORD lpnSize)
// {
//     STUB;
//     return ERROR_CALL_NOT_IMPLEMENTED;
// }
//
// DWORD
// WINAPI
// EnumerateLocalComputerNamesW(PVOID pUnknown, DWORD Size, LPWSTR lpBuffer, LPDWORD lpnSize)
// {
//     STUB;
//     return ERROR_CALL_NOT_IMPLEMENTED;
// }
//
// DWORD
// WINAPI
// RemoveLocalAlternateComputerNameA(LPSTR lpName, DWORD Unknown)
// {
//     STUB;
//     return ERROR_CALL_NOT_IMPLEMENTED;
// }
//
// DWORD
// WINAPI
// RemoveLocalAlternateComputerNameW(LPWSTR lpName, DWORD Unknown)
// {
//     STUB;
//     return ERROR_CALL_NOT_IMPLEMENTED;
// }
//
// /*
//  * @unimplemented
//  */
// BOOL
// WINAPI
// SetLocalPrimaryComputerNameA(IN DWORD Unknown1,
//                              IN DWORD Unknown2)
// {
//     STUB;
//     return FALSE;
// }
//
// /*
//  * @unimplemented
//  */
// BOOL
// WINAPI
// SetLocalPrimaryComputerNameW(IN DWORD Unknown1,
//                              IN DWORD Unknown2)
// {
//     STUB;
//     return FALSE;
// }
//
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
