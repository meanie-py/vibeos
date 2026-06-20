//! NT6+ Firmware API — firmware.c.

use crate::common::*;

#[no_mangle]
pub unsafe extern "system" fn GetFirmwareEnvironmentVariableExW(
    lpName: LPCWSTR,
    lpGuid: LPCWSTR,
    pBuffer: PVOID,
    nSize: DWORD,
    pdwAttribubutes: *mut DWORD,
) -> DWORD {
    if lpName.is_null() || lpGuid.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }
    let mut VariableName: UNICODE_STRING = core::mem::zeroed();
    let mut Namespace: UNICODE_STRING = core::mem::zeroed();
    let mut VendorGuid: GUID = core::mem::zeroed();
    RtlInitUnicodeString(&mut VariableName, lpName);
    RtlInitUnicodeString(&mut Namespace, lpGuid);
    let Status = RtlGUIDFromString(&mut Namespace, &mut VendorGuid);
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return 0;
    }

    let mut Length: ULONG = nSize;
    let Status = NtQuerySystemEnvironmentValueEx(
        &mut VariableName,
        &mut VendorGuid,
        pBuffer,
        &mut Length,
        pdwAttribubutes,
    );
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return 0;
    }

    Length
}

#[no_mangle]
pub unsafe extern "system" fn GetFirmwareEnvironmentVariableExA(
    lpName: LPCSTR,
    lpGuid: LPCSTR,
    pBuffer: PVOID,
    nSize: DWORD,
    pdwAttribubutes: *mut DWORD,
) -> DWORD {
    if lpName.is_null() || lpGuid.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }
    let mut VariableName: UNICODE_STRING = core::mem::zeroed();
    let mut Namespace: UNICODE_STRING = core::mem::zeroed();
    let mut AnsiVariableName: ANSI_STRING = core::mem::zeroed();
    let mut AnsiNamespace: ANSI_STRING = core::mem::zeroed();

    RtlInitString(&mut AnsiVariableName, lpName);
    let Status = RtlAnsiStringToUnicodeString(&mut VariableName, &mut AnsiVariableName, TRUE as BOOLEAN);
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return 0;
    }
    RtlInitString(&mut AnsiNamespace, lpGuid);
    let Status = RtlAnsiStringToUnicodeString(&mut Namespace, &mut AnsiNamespace, TRUE as BOOLEAN);
    if !NT_SUCCESS(Status) {
        RtlFreeUnicodeString(&mut VariableName);
        BaseSetLastNTError(Status);
        return 0;
    }

    let Length = GetFirmwareEnvironmentVariableExW(
        VariableName.Buffer,
        Namespace.Buffer,
        pBuffer,
        nSize,
        pdwAttribubutes,
    );

    RtlFreeUnicodeString(&mut Namespace);
    RtlFreeUnicodeString(&mut VariableName);
    Length
}

#[no_mangle]
pub unsafe extern "system" fn SetFirmwareEnvironmentVariableExW(
    lpName: LPCWSTR,
    lpGuid: LPCWSTR,
    pValue: PVOID,
    nSize: DWORD,
    dwAttributes: DWORD,
) -> BOOL {
    if lpName.is_null() || lpGuid.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE as BOOL;
    }
    let mut VariableName: UNICODE_STRING = core::mem::zeroed();
    let mut Namespace: UNICODE_STRING = core::mem::zeroed();
    let mut VendorGuid: GUID = core::mem::zeroed();
    RtlInitUnicodeString(&mut VariableName, lpName);
    RtlInitUnicodeString(&mut Namespace, lpGuid);
    let Status = RtlGUIDFromString(&mut Namespace, &mut VendorGuid);
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return FALSE as BOOL;
    }

    let Status =
        NtSetSystemEnvironmentValueEx(&mut VariableName, &mut VendorGuid, pValue, nSize, dwAttributes);
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return FALSE as BOOL;
    }

    TRUE as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn SetFirmwareEnvironmentVariableExA(
    lpName: LPCSTR,
    lpGuid: LPCSTR,
    pValue: PVOID,
    nSize: DWORD,
    dwAttributes: DWORD,
) -> BOOL {
    if lpName.is_null() || lpGuid.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE as BOOL;
    }
    let mut VariableName: UNICODE_STRING = core::mem::zeroed();
    let mut Namespace: UNICODE_STRING = core::mem::zeroed();
    let mut AnsiVariableName: ANSI_STRING = core::mem::zeroed();
    let mut AnsiNamespace: ANSI_STRING = core::mem::zeroed();

    RtlInitString(&mut AnsiVariableName, lpName);
    let Status = RtlAnsiStringToUnicodeString(&mut VariableName, &mut AnsiVariableName, TRUE as BOOLEAN);
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return FALSE as BOOL;
    }
    RtlInitString(&mut AnsiNamespace, lpGuid);
    let Status = RtlAnsiStringToUnicodeString(&mut Namespace, &mut AnsiNamespace, TRUE as BOOLEAN);
    if !NT_SUCCESS(Status) {
        RtlFreeUnicodeString(&mut VariableName);
        BaseSetLastNTError(Status);
        return FALSE as BOOL;
    }

    let Result = SetFirmwareEnvironmentVariableExW(
        VariableName.Buffer,
        Namespace.Buffer,
        pValue,
        nSize,
        dwAttributes,
    );

    RtlFreeUnicodeString(&mut Namespace);
    RtlFreeUnicodeString(&mut VariableName);
    Result
}

#[no_mangle]
pub unsafe extern "system" fn GetFirmwareType(FirmwareType: PFIRMWARE_TYPE) -> BOOL {
    if FirmwareType.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE as BOOL;
    }

    let mut BootInfo: SYSTEM_BOOT_ENVIRONMENT_INFORMATION = core::mem::zeroed();
    let Status = NtQuerySystemInformation(
        SystemBootEnvironmentInformation,
        &mut BootInfo as *mut SYSTEM_BOOT_ENVIRONMENT_INFORMATION as PVOID,
        core::mem::size_of::<SYSTEM_BOOT_ENVIRONMENT_INFORMATION>() as ULONG,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return FALSE as BOOL;
    }

    *FirmwareType = BootInfo.FirmwareType;
    TRUE as BOOL
}
