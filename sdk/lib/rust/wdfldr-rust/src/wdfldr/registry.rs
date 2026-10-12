// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
pub static RUSTCONV_WSTR_5D973BA3D129: [u16; 72] = [
    0x005Cu16, 0x0052, 0x0065, 0x0067, 0x0069, 0x0073, 0x0074, 0x0072, 0x0079, 0x005C,
    0x004D, 0x0061, 0x0063, 0x0068, 0x0069, 0x006E, 0x0065, 0x005C, 0x0053, 0x0079,
    0x0073, 0x0074, 0x0065, 0x006D, 0x005C, 0x0043, 0x0075, 0x0072, 0x0072, 0x0065,
    0x006E, 0x0074, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F, 0x006C, 0x0053,
    0x0065, 0x0074, 0x005C, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F, 0x006C,
    0x005C, 0x0057, 0x0064, 0x0066, 0x005C, 0x004B, 0x006D, 0x0064, 0x0066, 0x005C,
    0x0025, 0x0073, 0x005C, 0x0056, 0x0065, 0x0072, 0x0073, 0x0069, 0x006F, 0x006E,
    0x0073, 0x0000,
];
pub static RUSTCONV_WSTR_826330FF4F4A: [u16; 63] = [
    0x005Cu16, 0x0052, 0x0065, 0x0067, 0x0069, 0x0073, 0x0074, 0x0072, 0x0079, 0x005C,
    0x004D, 0x0061, 0x0063, 0x0068, 0x0069, 0x006E, 0x0065, 0x005C, 0x0053, 0x0079,
    0x0073, 0x0074, 0x0065, 0x006D, 0x005C, 0x0043, 0x0075, 0x0072, 0x0072, 0x0065,
    0x006E, 0x0074, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F, 0x006C, 0x0053,
    0x0065, 0x0074, 0x005C, 0x0053, 0x0065, 0x0072, 0x0076, 0x0069, 0x0063, 0x0065,
    0x0073, 0x005C, 0x0057, 0x0064, 0x0066, 0x0025, 0x0030, 0x0032, 0x0064, 0x0030,
    0x0030, 0x0030, 0x0000,
];
pub static RUSTCONV_WSTR_CF50DD39C387: [u16; 8] = [0x0053u16, 0x0065, 0x0072, 0x0076, 0x0069, 0x0063, 0x0065, 0x0000];
pub static RUSTCONV_WSTR_FA8A27D23A1A: [u16; 6] = [0x0053u16, 0x0074, 0x0061, 0x0072, 0x0074, 0x0000];

use crate::prelude::*;
use ntkrnl_sys::ddk::{KEY_VALUE_PARTIAL_INFORMATION, _OBJECT_ATTRIBUTES, PHANDLE, PKEY_VALUE_PARTIAL_INFORMATION, PUNICODE_STRING, PWCH, RtlIntegerToUnicodeString, ZwClose, ZwOpenKey, ZwQueryValueKey};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut regPathLength: SIZE_T = (::core::mem::size_of::<[::core::ffi::c_ushort; 61]>() as SIZE_T)
        .wrapping_sub(::core::mem::size_of::<WCHAR>() as SIZE_T);

#[no_mangle]
pub unsafe extern "system" fn GetVersionRegistryHandle(BindInfo: PWDF_BIND_INFO, HandleRegKey: PHANDLE) -> NTSTATUS {
    let mut status: NTSTATUS;
    let mut objectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut handle: HANDLE = core::ptr::null_mut();
    let mut keyHandle: HANDLE = core::ptr::null_mut();
    // DECLARE_UNICODE_STRING_SIZE(string, COMMENT_TOKEN_ECHO____)
    let mut string_buffer = [0 as WCHAR; 256];
    let mut string: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: core::mem::size_of_val(&string_buffer) as USHORT,
        Buffer: string_buffer.as_mut_ptr(),
    };

    'end: {
        status = RtlUnicodeStringPrintf!(
            core::ptr::addr_of_mut!(string),
            RUSTCONV_WSTR_5D973BA3D129.as_ptr(),
            (*BindInfo).Component,
        ); // Component name for kmdf driver - 'KmdfLibrary'

        if !NT_SUCCESS!(status) {
            __DBGPRINT!(("ERROR: RtlUnicodeStringPrintf failed with Status 0x%x\n", status));
            break 'end;
        }

        __DBGPRINT!(("Component path %wZ\n", core::ptr::addr_of_mut!(string)));

        // InitializeObjectAttributes(&objectAttributes, &string, OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE, NULL, NULL)
        objectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
        objectAttributes.RootDirectory = core::ptr::null_mut();
        objectAttributes.ObjectName = core::ptr::addr_of_mut!(string) as PUNICODE_STRING;
        objectAttributes.Attributes =
            (OBJ_KERNEL_HANDLE | OBJ_CASE_INSENSITIVE) as ULONG;
        objectAttributes.SecurityDescriptor = core::ptr::null_mut();
        objectAttributes.SecurityQualityOfService = core::ptr::null_mut();
        status = ZwOpenKey(
            core::ptr::addr_of_mut!(keyHandle),
            KEY_QUERY_VALUE as ACCESS_MASK,
            core::ptr::addr_of_mut!(objectAttributes),
        );

        if !NT_SUCCESS!(status) {
            __DBGPRINT!((
                "ERROR: ZwOpenKey (%wZ) failed with Status 0x%x\n",
                core::ptr::addr_of_mut!(string),
                status,
            ));
            break 'end;
        }

        status = RtlIntegerToUnicodeString(
            (*BindInfo).Version.Major as ULONG,
            10 as ULONG,
            core::ptr::addr_of_mut!(string),
        );

        if !NT_SUCCESS!(status) {
            __DBGPRINT!(("ERROR: ConvertUlongToWString failed with Status 0x%x\n", status));
            break 'end;
        }

        // InitializeObjectAttributes(&objectAttributes, &string, OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE, keyHandle, NULL)
        objectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
        objectAttributes.RootDirectory = keyHandle;
        objectAttributes.ObjectName = core::ptr::addr_of_mut!(string) as PUNICODE_STRING;
        objectAttributes.Attributes =
            (OBJ_KERNEL_HANDLE | OBJ_CASE_INSENSITIVE) as ULONG;
        objectAttributes.SecurityDescriptor = core::ptr::null_mut();
        objectAttributes.SecurityQualityOfService = core::ptr::null_mut();
        status = ZwOpenKey(
            core::ptr::addr_of_mut!(handle),
            KEY_READ as ACCESS_MASK,
            core::ptr::addr_of_mut!(objectAttributes),
        );

        if !NT_SUCCESS!(status) {
            __DBGPRINT!((
                "ERROR: ZwOpenKey (%wZ) failed with Status 0x%x\n",
                string,
                status,
            ));
        }
    }

    // end:
    *HandleRegKey = handle;

    if keyHandle != core::ptr::null_mut() {
        ZwClose(keyHandle);
    }

    return status;
}
#[no_mangle]
pub unsafe extern "system" fn GetDefaultServiceName(
    BindInfo: PWDF_BIND_INFO,
    RegistryPath: PUNICODE_STRING,
) -> NTSTATUS {
    let mut status: NTSTATUS = 0;
    let regPathLength_local: SIZE_T =
        (core::mem::size_of_val(&[
    0x005Cu16, 0x0052, 0x0065, 0x0067, 0x0069, 0x0073, 0x0074, 0x0072, 0x0079, 0x005C,
    0x004D, 0x0061, 0x0063, 0x0068, 0x0069, 0x006E, 0x0065, 0x005C, 0x0053, 0x0079,
    0x0073, 0x0074, 0x0065, 0x006D, 0x005C, 0x0043, 0x0075, 0x0072, 0x0072, 0x0065,
    0x006E, 0x0074, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F, 0x006C, 0x0053,
    0x0065, 0x0074, 0x005C, 0x0053, 0x0065, 0x0072, 0x0076, 0x0069, 0x0063, 0x0065,
    0x0073, 0x005C, 0x0057, 0x0064, 0x0066, 0x0030, 0x0031, 0x0030, 0x0030, 0x0030,
    0x0000,
]) - core::mem::size_of::<WCHAR>()) as SIZE_T;

    if RegistryPath.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    (*RegistryPath).Buffer = ExAllocatePoolWithTag(
        PagedPool,
        regPathLength_local,
        WDFLDR_TAG,
    ) as PWCHAR;

    if (*RegistryPath).Buffer.is_null() {
        status = STATUS_INSUFFICIENT_RESOURCES;
        __DBGPRINT!("ERROR: ExAllocatePoolWithTag failed with status 0x%x\n", status);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    (*RegistryPath).Length = 0;
    (*RegistryPath).MaximumLength = regPathLength_local as USHORT;

    status = RtlUnicodeStringPrintf!(
        RegistryPath,
        RUSTCONV_WSTR_826330FF4F4A.as_ptr(),
        (*BindInfo).Version.Major,
    );

    if NT_SUCCESS!(status) {
        __DBGPRINT!(
            "Couldn't find control Key -- using default service name %wZ\n",
            RegistryPath,
        );
        status = STATUS_SUCCESS;
    } else {
        __DBGPRINT!("ERROR: RtlUnicodeStringCopyString failed with Status 0x%x\n", status);

        ExFreePoolWithTag((*RegistryPath).Buffer as PVOID, WDFLDR_TAG);
        (*RegistryPath).Length = 0;
        (*RegistryPath).Buffer = core::ptr::null_mut();
    }

    return status;
}
#[no_mangle]
pub unsafe extern "C" fn GetVersionServicePath(BindInfo: PWDF_BIND_INFO, ServicePath: PUNICODE_STRING) -> NTSTATUS {
    let mut status: NTSTATUS;
    let mut pKeyVal: PKEY_VALUE_PARTIAL_INFORMATION = core::ptr::null_mut();
    let mut handleRegKey: HANDLE = core::ptr::null_mut();
    let mut ValueName: _UNICODE_STRING = RTL_CONSTANT_STRING!(RUSTCONV_WSTR_CF50DD39C387.as_ptr());

    status = GetVersionRegistryHandle(BindInfo, core::ptr::addr_of_mut!(handleRegKey));

    if !NT_SUCCESS!(status) {
        __DBGPRINT!(("ERROR: GetVersionRegistryHandle failed with Status 0x%x\n", status));
    } else {
        // get service name
        status = FxLdrQueryData(
            handleRegKey,
            core::ptr::addr_of_mut!(ValueName),
            WDFLDR_TAG,
            core::ptr::addr_of_mut!(pKeyVal),
        );

        if !NT_SUCCESS!(status) {
            __DBGPRINT!(("ERROR: QueryData failed with status 0x%x\n", status));
        } else {
            status = BuildServicePath(pKeyVal, ServicePath);
        }
    }

    if !NT_SUCCESS!(status) {
        status = GetDefaultServiceName(BindInfo, ServicePath);

        if !NT_SUCCESS!(status) {
            __DBGPRINT!(("ERROR: GetVersionServicePath failed with Status 0x%x\n", status));
        }
    } else {
        __DBGPRINT!(("(%wZ)\n", ServicePath));
    }

    if handleRegKey != core::ptr::null_mut() {
        ZwClose(handleRegKey);
    }
    if pKeyVal != core::ptr::null_mut() {
        ExFreePoolWithTag(pKeyVal as PVOID, WDFLDR_TAG);
    }

    return status;
}
#[no_mangle]
pub unsafe extern "C" fn ServiceCheckBootStart(Service: PUNICODE_STRING) -> BOOLEAN {
    let mut status: NTSTATUS;
    let mut objectAttributes: _OBJECT_ATTRIBUTES = _OBJECT_ATTRIBUTES {
        Length: core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG,
        RootDirectory: core::ptr::null_mut(),
        ObjectName: Service,
        Attributes: (OBJ_KERNEL_HANDLE | OBJ_CASE_INSENSITIVE) as ULONG,
        SecurityDescriptor: core::ptr::null_mut(),
        SecurityQualityOfService: core::ptr::null_mut(),
    };
    let mut keyHandle: HANDLE = core::ptr::null_mut();
    let mut result: BOOLEAN = FALSE;
    let mut value: ULONG = 0;
    let mut valueName: _UNICODE_STRING = RTL_CONSTANT_STRING!(RUSTCONV_WSTR_FA8A27D23A1A.as_ptr());

    status = ZwOpenKey(&mut keyHandle, KEY_READ, &mut objectAttributes);

    if status != STATUS_OBJECT_NAME_NOT_FOUND {
        if NT_SUCCESS!(status) {
            status = FxLdrQueryUlong(keyHandle, &mut valueName, &mut value);
            if NT_SUCCESS!(status) {
                result = (value == 0) as BOOLEAN;
            }
        } else {
            __DBGPRINT!(
                b"ZwOpenKey(%wZ) failed: %08X\n\0" as *const u8 as PCSTR,
                Service,
                status,
            );
        }
    }

    if !keyHandle.is_null() {
        ZwClose(keyHandle);
    }

    return result;
}
#[no_mangle]
pub unsafe extern "C" fn FxLdrQueryUlong(
    KeyHandle: HANDLE,
    ValueName: PUNICODE_STRING,
    Value: PULONG,
) -> NTSTATUS {
    let mut status: NTSTATUS;
    let mut resultLength: ULONG = 0;
    let mut keyValue: KEY_VALUE_PARTIAL_INFORMATION = core::mem::zeroed();

    keyValue.DataLength = 0;
    keyValue.TitleIndex = 0;
    keyValue.Type = 0;
    keyValue.Data[0 as usize] = 0;
    status = ZwQueryValueKey(
        KeyHandle,
        ValueName,
        KeyValuePartialInformation,
        core::ptr::addr_of_mut!(keyValue) as PVOID,
        core::mem::size_of::<KEY_VALUE_PARTIAL_INFORMATION>() as ULONG,
        core::ptr::addr_of_mut!(resultLength),
    );
    if NT_SUCCESS!(status) {
        if keyValue.Type != REG_DWORD
            || keyValue.DataLength != core::mem::size_of::<ULONG>() as ULONG
        {
            status = STATUS_INVALID_BUFFER_SIZE;
        } else {
            *Value = keyValue.Data[0 as usize] as ULONG;
            status = STATUS_SUCCESS;
        }
    } else {
        __DBGPRINT!(("ERROR: ZwQueryValueKey failed with Status 0x%x\n", status));
    }

    return status;
}
#[no_mangle]
pub unsafe extern "C" fn FxLdrQueryData(
    KeyHandle: HANDLE,
    ValueName: PUNICODE_STRING,
    Tag: ULONG,
    KeyValPartialInfo: *mut PKEY_VALUE_PARTIAL_INFORMATION,
) -> NTSTATUS {
    let mut pKeyInfo: PKEY_VALUE_PARTIAL_INFORMATION = core::ptr::null_mut();
    let mut status: NTSTATUS;
    let mut resultLength: ULONG = 0; // written by the query before it is read

    *KeyValPartialInfo = core::ptr::null_mut();
    loop {
        status = ZwQueryValueKey(
            KeyHandle,
            ValueName,
            KeyValuePartialInformation,
            core::ptr::null_mut(),
            0,
            core::ptr::addr_of_mut!(resultLength),
        );
        if status != STATUS_BUFFER_TOO_SMALL {
            if !NT_SUCCESS!(status) {
                __DBGPRINT!("ERROR: ZwQueryValueKey failed with status 0x%x\n", status);
            }

            return status;
        }

        status = RtlULongAdd(
            resultLength,
            0xC,
            core::ptr::addr_of_mut!(resultLength),
        );
        if !NT_SUCCESS!(status) {
            __DBGPRINT!(
                "ERROR: Computing length of data under %wZ failed with status 0x%x\n",
                ValueName,
                status,
            );

            return status;
        }

        pKeyInfo = ExAllocatePoolWithTag(PagedPool, resultLength as SIZE_T, Tag)
            as PKEY_VALUE_PARTIAL_INFORMATION;

        if pKeyInfo.is_null() {
            break;
        }

        RtlZeroMemory!(pKeyInfo as PVOID, resultLength as SIZE_T);
        status = ZwQueryValueKey(
            KeyHandle,
            ValueName,
            KeyValuePartialInformation,
            pKeyInfo as PVOID,
            resultLength,
            core::ptr::addr_of_mut!(resultLength),
        );

        if NT_SUCCESS!(status) {
            *KeyValPartialInfo = pKeyInfo;
            return status;
        }

        ExFreePoolWithTag(pKeyInfo as PVOID, WDFLDR_TAG);

        if status != STATUS_BUFFER_TOO_SMALL {
            __DBGPRINT!(
                "ERROR: ZwQueryValueKey (%wZ) failed with Status 0x%x\n",
                ValueName,
                status,
            );

            return status;
        }
    }

    __DBGPRINT!(
        "ERROR: ExAllocatePoolWithTag failed with Status 0x%x\n",
        STATUS_INSUFFICIENT_RESOURCES,
    );

    return STATUS_INSUFFICIENT_RESOURCES;
}
#[no_mangle]
pub unsafe extern "C" fn BuildServicePath(KeyValueInformation: PKEY_VALUE_PARTIAL_INFORMATION, ServicePath: PUNICODE_STRING) -> NTSTATUS {
    let mut status: NTSTATUS;
    let mut buffer: PWCHAR;
    let mut lastChar: PWCHAR;
    let mut name: _UNICODE_STRING = core::mem::zeroed();
    let regPath: [u16; 56] = [
    0x005Cu16, 0x0052, 0x0065, 0x0067, 0x0069, 0x0073, 0x0074, 0x0072, 0x0079, 0x005C,
    0x004D, 0x0061, 0x0063, 0x0068, 0x0069, 0x006E, 0x0065, 0x005C, 0x0053, 0x0079,
    0x0073, 0x0074, 0x0065, 0x006D, 0x005C, 0x0043, 0x0075, 0x0072, 0x0072, 0x0065,
    0x006E, 0x0074, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F, 0x006C, 0x0053,
    0x0065, 0x0074, 0x005C, 0x0053, 0x0065, 0x0072, 0x0076, 0x0069, 0x0063, 0x0065,
    0x0073, 0x005C, 0x0025, 0x0077, 0x005A, 0x0000,
];

    if (*KeyValueInformation).Type != REG_SZ &&
        (*KeyValueInformation).Type != REG_EXPAND_SZ
    {
        status = STATUS_OBJECT_TYPE_MISMATCH;
        __DBGPRINT!("ERROR: BuildServicePath failed with status 0x%x\n", status);
        return status;
    }

    if (*KeyValueInformation).DataLength == 0 ||
        (*KeyValueInformation).DataLength > 0xFFFF
    {
        status = STATUS_INVALID_PARAMETER;
        __DBGPRINT!("ERROR: BuildServicePath failed with status 0x%x\n", status);
        return status;
    }

    name.Buffer = core::ptr::addr_of_mut!((*KeyValueInformation).Data) as PWCH;
    name.Length = (*KeyValueInformation).DataLength as USHORT;
    name.MaximumLength = (*KeyValueInformation).DataLength as USHORT;

    lastChar = (core::ptr::addr_of_mut!((*KeyValueInformation).Data) as PWCHAR)
        .add(((*KeyValueInformation).DataLength / core::mem::size_of::<WCHAR>() as ULONG) as usize);
    if (*KeyValueInformation).DataLength >= core::mem::size_of::<WCHAR>() as ULONG
        && *lastChar == UNICODE_NULL
    {
        name.Length = ((*KeyValueInformation).DataLength as USHORT)
            - core::mem::size_of::<WCHAR>() as USHORT;
    }

    buffer = ExAllocatePoolZero(
        PagedPool,
        (name.Length as usize) + core::mem::size_of_val(&regPath),
        WDFLDR_TAG,
    ) as PWCHAR;
    if !buffer.is_null() {
        (*ServicePath).Length = 0;
        (*ServicePath).MaximumLength =
            ((name.Length as usize) + core::mem::size_of_val(&[
    0x005Cu16, 0x0052, 0x0065, 0x0067, 0x0069, 0x0073, 0x0074, 0x0072, 0x0079, 0x005C,
    0x004D, 0x0061, 0x0063, 0x0068, 0x0069, 0x006E, 0x0065, 0x005C, 0x0053, 0x0079,
    0x0073, 0x0074, 0x0065, 0x006D, 0x005C, 0x0043, 0x0075, 0x0072, 0x0072, 0x0065,
    0x006E, 0x0074, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F, 0x006C, 0x0053,
    0x0065, 0x0074, 0x005C, 0x0053, 0x0065, 0x0072, 0x0076, 0x0069, 0x0063, 0x0065,
    0x0073, 0x005C, 0x0000,
])) as USHORT;//106;
        (*ServicePath).Buffer = buffer;
        status = RtlUnicodeStringPrintf!(
            ServicePath,
            core::ptr::addr_of!(regPath) as *const u16,
            core::ptr::addr_of_mut!(name),
        );

        if !NT_SUCCESS!(status) {
            ExFreePoolWithTag(buffer as PVOID, WDFLDR_TAG);
            (*ServicePath).Length = 0;
            (*ServicePath).Buffer = core::ptr::null_mut();
        }
    } else {
        status = STATUS_INSUFFICIENT_RESOURCES;
        __DBGPRINT!("ERROR: ExAllocatePoolWithTag failed with Status 0x%x\n", status);
    }

    return status;
}
