// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
pub static RUSTCONV_WSTR_769464D2088F: [u16; 14] = [
    0x0050u16, 0x0072, 0x006F, 0x0063, 0x0065, 0x0073, 0x0073, 0x006F, 0x0072, 0x0054,
    0x0079, 0x0070, 0x0065, 0x0000,
];
pub static RUSTCONV_WSTR_95A26E22DCB6: [u16; 15] = [
    0x004Eu16, 0x0065, 0x0074, 0x0077, 0x006F, 0x0072, 0x006B, 0x0041, 0x0064, 0x0064,
    0x0072, 0x0065, 0x0073, 0x0073, 0x0000,
];
pub static RUSTCONV_WSTR_96A296D224F2: [u16; 1] = [0x0000u16];
pub static RUSTCONV_WSTR_B733DC901B6C: [u16; 12] = [
    0x0045u16, 0x006E, 0x0076, 0x0069, 0x0072, 0x006F, 0x006E, 0x006D, 0x0065, 0x006E,
    0x0074, 0x0000,
];
pub static RUSTCONV_WSTR_CAECF7D590D8: [u16; 12] = [
    0x004Eu16, 0x0064, 0x0069, 0x0073, 0x0056, 0x0065, 0x0072, 0x0073, 0x0069, 0x006F,
    0x006E, 0x0000,
];

use crate::prelude::*;
use ntkrnl_sys::ddk::{ExAllocatePool, ExFreePool, KEY_BASIC_INFORMATION, KEY_VALUE_PARTIAL_INFORMATION, _OBJECT_ATTRIBUTES, RtlAppendUnicodeToString, RtlCopyUnicodeString, RtlIntegerToUnicodeString, RtlUnicodeStringToInteger, ZwClose, ZwEnumerateKey, ZwOpenKey, ZwQueryValueKey, ZwSetValueKey};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub const PARAMETERS_KEY: [::core::ffi::c_ushort; 11] = unsafe {
    ::core::mem::transmute::<[u8; 22], [::core::ffi::c_ushort; 11]>(
        *b"P\0a\0r\0a\0m\0e\0t\0e\0r\0s\0\0\0",
    )
};
#[no_mangle]
pub unsafe extern "system" fn NdisWriteConfiguration(
    Status: PNDIS_STATUS,
    ConfigurationHandle: NDIS_HANDLE,
    Keyword: PNDIS_STRING,
    ParameterValue: PNDIS_CONFIGURATION_PARAMETER,
) {
    let mut ParameterType: ULONG;
    let mut DataSize: ULONG;
    let mut Data: PVOID;
    let mut Buff: [WCHAR; 11] = [0; 11];

    NDIS_DbgPrint!(MAX_TRACE, b"Called.\n\0" as *const u8 as PCSTR);

    NDIS_DbgPrint!(
        MID_TRACE,
        b"Parameter type: %d\n\0" as *const u8 as PCSTR,
        (*ParameterValue).ParameterType,
    );

    /* reset parameter type to standard reg types */
    match (*ParameterValue).ParameterType {
        NdisParameterHexInteger | NdisParameterInteger => {
            let mut Str: _UNICODE_STRING = core::mem::zeroed();

            Str.Buffer = core::ptr::addr_of_mut!(Buff) as PWSTR;
            Str.MaximumLength = core::mem::size_of::<[WCHAR; 11]>() as USHORT;
            Str.Length = 0;

            ParameterType = REG_SZ;
            if !NT_SUCCESS!(RtlIntegerToUnicodeString(
                (*ParameterValue).ParameterData.IntegerData,
                if (*ParameterValue).ParameterType == NdisParameterInteger {
                    10
                } else {
                    16
                },
                core::ptr::addr_of_mut!(Str),
            )) {
                NDIS_DbgPrint!(
                    MIN_TRACE,
                    b"RtlIntegerToUnicodeString failed (%x)\n\0" as *const u8 as PCSTR,
                    *Status,
                );
                *Status = NDIS_STATUS_FAILURE as _;
                return;
            }
            Data = Str.Buffer as PVOID;
            DataSize = Str.Length as ULONG;
        }
        NdisParameterString | NdisParameterMultiString => {
            ParameterType = if (*ParameterValue).ParameterType == NdisParameterString {
                REG_SZ
            } else {
                REG_MULTI_SZ
            };
            Data = (*ParameterValue).ParameterData.StringData.Buffer as PVOID;
            DataSize = (*ParameterValue).ParameterData.StringData.Length as ULONG;
        }

        /* New (undocumented) addition to 2k ddk */
        NdisParameterBinary => {
            ParameterType = REG_BINARY;
            Data = (*ParameterValue).ParameterData.BinaryData.Buffer as PVOID;
            DataSize = (*ParameterValue).ParameterData.BinaryData.Length as ULONG;
        }

        _ => {
            *Status = NDIS_STATUS_NOT_SUPPORTED as _;
            return;
        }
    }

    *Status = ZwSetValueKey(
        (*(ConfigurationHandle as PMINIPORT_CONFIGURATION_CONTEXT)).Handle,
        Keyword,
        0,
        ParameterType,
        Data,
        DataSize,
    ) as _;

    if *Status != STATUS_SUCCESS as _ {
        NDIS_DbgPrint!(
            MIN_TRACE,
            b"ZwSetValueKey failed (%x)\n\0" as *const u8 as PCSTR,
            *Status,
        );
        *Status = NDIS_STATUS_FAILURE as _;
    } else {
        *Status = NDIS_STATUS_SUCCESS as _;
    }
}
#[no_mangle]
pub unsafe extern "system" fn NdisCloseConfiguration(ConfigurationHandle: NDIS_HANDLE) {
    let mut ConfigurationContext: PMINIPORT_CONFIGURATION_CONTEXT =
        ConfigurationHandle as PMINIPORT_CONFIGURATION_CONTEXT;
    let mut Resource: PMINIPORT_RESOURCE = core::ptr::null_mut();
    let mut ParameterValue: PNDIS_CONFIGURATION_PARAMETER = core::ptr::null_mut();
    let mut CurrentEntry: PLIST_ENTRY = core::ptr::null_mut();

    loop {
        CurrentEntry = ExInterlockedRemoveHeadList(
            core::ptr::addr_of_mut!((*ConfigurationContext).ResourceListHead),
            core::ptr::addr_of_mut!((*ConfigurationContext).ResourceLock),
        );
        if CurrentEntry.is_null() {
            break;
        }
        Resource = CONTAINING_RECORD!(CurrentEntry, MINIPORT_RESOURCE, ListEntry);

        // Emulates C switch fall-through: case FACT_0001 falls into case FACT_0005.
        let mut fall_to_free: bool = false;
        match (*Resource).ResourceType {
            MINIPORT_RESOURCE_TYPE_REGISTRY_DATA => {
                ParameterValue = (*Resource).Resource as PNDIS_CONFIGURATION_PARAMETER;

                match (*ParameterValue).ParameterType {
                    NdisParameterBinary => {
                        ExFreePool((*ParameterValue).ParameterData.BinaryData.Buffer);
                    }
                    NdisParameterString | NdisParameterMultiString => {
                        ExFreePool(
                            (*ParameterValue).ParameterData.StringData.Buffer as PVOID,
                        );
                    }
                    _ => {}
                }

                // Fall through to free NDIS_CONFIGURATION_PARAMETER struct
                fall_to_free = true;
            }
            MINIPORT_RESOURCE_TYPE_MEMORY => {
                fall_to_free = true;
            }
            _ => {
                NDIS_DbgPrint!(
                    MIN_TRACE,
                    b"Unknown resource type: %d\n\0".as_ptr() as PCSTR,
                    (*Resource).ResourceType,
                );
            }
        }

        if fall_to_free {
            NDIS_DbgPrint!(
                MAX_TRACE,
                b"freeing 0x%x\n\0".as_ptr() as PCSTR,
                (*Resource).Resource,
            );
            ExFreePool((*Resource).Resource as PVOID);
        }

        ExFreePool(Resource as PVOID);
    }

    ZwClose((*ConfigurationContext).Handle as HANDLE);
}
#[no_mangle]
pub unsafe extern "system" fn NdisOpenConfiguration(
    Status: PNDIS_STATUS,
    ConfigurationHandle: PNDIS_HANDLE,
    WrapperConfigurationContext: NDIS_HANDLE,
) {
    let mut KeyHandle: HANDLE = core::ptr::null_mut();
    let mut ConfigurationContext: PMINIPORT_CONFIGURATION_CONTEXT = core::ptr::null_mut();
    let WrapperContext: PNDIS_WRAPPER_CONTEXT = WrapperConfigurationContext as PNDIS_WRAPPER_CONTEXT;
    let RootKeyHandle: HANDLE = (*WrapperContext).RegistryHandle;
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut NoName: _UNICODE_STRING = RTL_CONSTANT_STRING!(RUSTCONV_WSTR_96A296D224F2.as_ptr());

    NDIS_DbgPrint!(MAX_TRACE, b"Called\n\0".as_ptr());

    *ConfigurationHandle = core::ptr::null_mut();

    InitializeObjectAttributes!(
        &raw mut ObjectAttributes,
        &raw mut NoName,
        OBJ_KERNEL_HANDLE,
        RootKeyHandle,
        core::ptr::null_mut(),
    );
    *Status = ZwOpenKey(
        &raw mut KeyHandle,
        KEY_ALL_ACCESS,
        &raw mut ObjectAttributes,
    ) as _;
    if !NT_SUCCESS!(*Status) {
        NDIS_DbgPrint!(
            MIN_TRACE,
            b"Failed to open registry configuration for this miniport\n\0".as_ptr(),
        );
        *Status = NDIS_STATUS_FAILURE as _;
        return;
    }

    ConfigurationContext = ExAllocatePool(
        NonPagedPool,
        core::mem::size_of::<MINIPORT_CONFIGURATION_CONTEXT>() as SIZE_T,
    ) as PMINIPORT_CONFIGURATION_CONTEXT;
    if ConfigurationContext.is_null() {
        NDIS_DbgPrint!(MIN_TRACE, b"Insufficient resources.\n\0".as_ptr());
        ZwClose(KeyHandle);
        *Status = NDIS_STATUS_RESOURCES as _;
        return;
    }

    KeInitializeSpinLock(&raw mut (*ConfigurationContext).ResourceLock);
    InitializeListHead(&raw mut (*ConfigurationContext).ResourceListHead);

    (*ConfigurationContext).Handle = KeyHandle as NDIS_HANDLE;

    *ConfigurationHandle = ConfigurationContext as NDIS_HANDLE as PVOID;
    *Status = NDIS_STATUS_SUCCESS as _;

    NDIS_DbgPrint!(MAX_TRACE, b"returning success\n\0".as_ptr());
}
#[no_mangle]
pub unsafe extern "system" fn NdisOpenProtocolConfiguration(
    Status: PNDIS_STATUS,
    ConfigurationHandle: PNDIS_HANDLE,
    ProtocolSection: PNDIS_STRING,
) {
    let mut KeyAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut KeyNameU: _UNICODE_STRING = core::mem::zeroed();
    let mut KeyHandle: HANDLE = core::ptr::null_mut();
    let mut ConfigurationContext: PMINIPORT_CONFIGURATION_CONTEXT = core::ptr::null_mut();

    KeyNameU.Length = 0;
    KeyNameU.MaximumLength = ((*ProtocolSection).Length as usize)
        .wrapping_add(core::mem::size_of_val(&PARAMETERS_KEY))
        .wrapping_add(core::mem::size_of_val(&UNICODE_NULL)) as USHORT;
    KeyNameU.Buffer = ExAllocatePool(PagedPool, KeyNameU.MaximumLength as SIZE_T) as PWCH;
    if KeyNameU.Buffer.is_null() {
        NDIS_DbgPrint!(MIN_TRACE, b"Insufficient resources.\n\0".as_ptr() as PCSTR);
        *ConfigurationHandle = core::ptr::null_mut();
        *Status = NDIS_STATUS_FAILURE;
        return;
    }

    RtlCopyUnicodeString(core::ptr::addr_of_mut!(KeyNameU), ProtocolSection as PCUNICODE_STRING);
    RtlAppendUnicodeToString(core::ptr::addr_of_mut!(KeyNameU), PARAMETERS_KEY.as_ptr() as PCWSTR);

    // InitializeObjectAttributes expanded inline
    KeyAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    KeyAttributes.RootDirectory = core::ptr::null_mut();
    KeyAttributes.ObjectName = core::ptr::addr_of_mut!(KeyNameU) as PUNICODE_STRING;
    KeyAttributes.Attributes = (OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE) as ULONG;
    KeyAttributes.SecurityDescriptor = core::ptr::null_mut();
    KeyAttributes.SecurityQualityOfService = core::ptr::null_mut();

    *Status = ZwOpenKey(
        core::ptr::addr_of_mut!(KeyHandle),
        KEY_ALL_ACCESS as ACCESS_MASK,
        core::ptr::addr_of_mut!(KeyAttributes),
    ) as NDIS_STATUS;

    ExFreePool(KeyNameU.Buffer as PVOID);

    if *Status != NDIS_STATUS_SUCCESS {
        NDIS_DbgPrint!(
            MIN_TRACE,
            b"ZwOpenKey failed (%x)\n\0".as_ptr() as PCSTR,
            *Status,
        );
        *ConfigurationHandle = core::ptr::null_mut();
        *Status = NDIS_STATUS_FAILURE;
        return;
    }

    ConfigurationContext = ExAllocatePool(
        NonPagedPool,
        core::mem::size_of::<MINIPORT_CONFIGURATION_CONTEXT>() as SIZE_T,
    ) as PMINIPORT_CONFIGURATION_CONTEXT;
    if ConfigurationContext.is_null() {
        NDIS_DbgPrint!(MIN_TRACE, b"Insufficient resources.\n\0".as_ptr() as PCSTR);
        ZwClose(KeyHandle);
        *ConfigurationHandle = core::ptr::null_mut();
        *Status = NDIS_STATUS_FAILURE;
        return;
    }

    KeInitializeSpinLock(core::ptr::addr_of_mut!((*ConfigurationContext).ResourceLock));
    InitializeListHead(core::ptr::addr_of_mut!((*ConfigurationContext).ResourceListHead));

    (*ConfigurationContext).Handle = KeyHandle as NDIS_HANDLE;

    *ConfigurationHandle = ConfigurationContext as NDIS_HANDLE as PVOID;
    *Status = NDIS_STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn UnicodeToHexByte(chr: WCHAR) -> UCHAR {
    match chr {
        0x0030u16 => return 0 as UCHAR,
        0x0031u16 => return 1 as UCHAR,
        0x0032u16 => return 2 as UCHAR,
        0x0033u16 => return 3 as UCHAR,
        0x0034u16 => return 4 as UCHAR,
        0x0035u16 => return 5 as UCHAR,
        0x0036u16 => return 6 as UCHAR,
        0x0037u16 => return 7 as UCHAR,
        0x0038u16 => return 8 as UCHAR,
        0x0039u16 => return 9 as UCHAR,
        0x0041u16 | 0x0061u16 => return 10 as UCHAR,
        0x0042u16 | 0x0062u16 => return 11 as UCHAR,
        0x0043u16 | 0x0063u16 => return 12 as UCHAR,
        0x0044u16 | 0x0064u16 => return 13 as UCHAR,
        0x0045u16 | 0x0065u16 => return 14 as UCHAR,
        0x0046u16 | 0x0066u16 => return 15 as UCHAR,
        _ => {}
    }
    return 0xFF as UCHAR;
}
#[no_mangle]
pub unsafe extern "C" fn IsValidNumericString(String: PNDIS_STRING, Base: ULONG) -> BOOLEAN {
    let mut i: ULONG = 0;
    if (*String).Length == 0 {
        NDIS_DbgPrint!(MIN_TRACE, b"Got an empty string; not sure what to do here\n\0".as_ptr() as PCSTR);
        return FALSE;
    }
    i = 0;
    while (i as usize) < ((*String).Length as usize) / core::mem::size_of::<WCHAR>() {
        if !(*(*String).Buffer.offset(i as isize) == UNICODE_NULL) {
            if UnicodeToHexByte(*(*String).Buffer.offset(i as isize)) as ULONG >= Base {
                return FALSE;
            }
        }
        i = i.wrapping_add(1);
    }
    return TRUE;
}
#[no_mangle]
pub unsafe extern "system" fn NdisReadConfiguration(Status: PNDIS_STATUS, ParameterValue: *mut PNDIS_CONFIGURATION_PARAMETER, ConfigurationHandle: NDIS_HANDLE, Keyword: PNDIS_STRING, ParameterType: NDIS_PARAMETER_TYPE) {
    let mut KeyInformation: *mut KEY_VALUE_PARTIAL_INFORMATION = core::ptr::null_mut();
    let mut KeyDataLength: ULONG = 0;
    let mut MiniportResource: PMINIPORT_RESOURCE = core::ptr::null_mut();
    let mut ConfigurationContext: PMINIPORT_CONFIGURATION_CONTEXT =
        ConfigurationHandle as PMINIPORT_CONFIGURATION_CONTEXT;
    let mut Buffer: PVOID = core::ptr::null_mut();

    *Status = NDIS_STATUS_FAILURE;

    NDIS_DbgPrint!(MAX_TRACE, b"requested read of %wZ\n\0" as *const u8 as PCSTR, Keyword);
    NDIS_DbgPrint!(MID_TRACE, b"requested parameter type: %d\n\0" as *const u8 as PCSTR, ParameterType);

    if ConfigurationContext.is_null() {
        NDIS_DbgPrint!(
            MIN_TRACE,
            b"invalid parameter ConfigurationContext (0x%x)\n\0" as *const u8 as PCSTR,
            ConfigurationContext,
        );
        return;
    }

    if wcsncmp((*Keyword).Buffer as *const u16, RUSTCONV_WSTR_B733DC901B6C.as_ptr(), ((*Keyword).Length as usize) / core::mem::size_of::<WCHAR>()) == 0
        && wcslen(RUSTCONV_WSTR_B733DC901B6C.as_ptr()) as usize == ((*Keyword).Length as usize) / core::mem::size_of::<WCHAR>()
    {
        *ParameterValue = ExAllocatePool(
            PagedPool,
            core::mem::size_of::<NDIS_CONFIGURATION_PARAMETER>() as SIZE_T,
        ) as PNDIS_CONFIGURATION_PARAMETER;
        if (*ParameterValue).is_null() {
            NDIS_DbgPrint!(MIN_TRACE, b"Insufficient resources.\n\0" as *const u8 as PCSTR);
            *Status = NDIS_STATUS_RESOURCES;
            return;
        }

        MiniportResource = ExAllocatePool(
            PagedPool,
            core::mem::size_of::<MINIPORT_RESOURCE>() as SIZE_T,
        ) as PMINIPORT_RESOURCE;
        if MiniportResource.is_null() {
            NDIS_DbgPrint!(MIN_TRACE, b"Insufficient resources.\n\0" as *const u8 as PCSTR);
            ExFreePool(*ParameterValue as PVOID);
            *ParameterValue = core::ptr::null_mut();
            *Status = NDIS_STATUS_RESOURCES;
            return;
        }

        (*MiniportResource).ResourceType = MINIPORT_RESOURCE_TYPE_REGISTRY_DATA;
        (*MiniportResource).Resource = *ParameterValue as PVOID;

        NDIS_DbgPrint!(
            MID_TRACE,
            b"inserting 0x%x into the resource list\n\0" as *const u8 as PCSTR,
            (*MiniportResource).Resource,
        );

        ExInterlockedInsertTailList(
            core::ptr::addr_of_mut!((*ConfigurationContext).ResourceListHead),
            core::ptr::addr_of_mut!((*MiniportResource).ListEntry),
            core::ptr::addr_of_mut!((*ConfigurationContext).ResourceLock),
        );

        (**ParameterValue).ParameterType = NdisParameterInteger;
        (**ParameterValue).ParameterData.IntegerData = NdisEnvironmentWindowsNt;
        *Status = NDIS_STATUS_SUCCESS;

        return;
    }

    if wcsncmp((*Keyword).Buffer as *const u16, RUSTCONV_WSTR_769464D2088F.as_ptr(), ((*Keyword).Length as usize) / core::mem::size_of::<WCHAR>()) == 0
        && wcslen(RUSTCONV_WSTR_769464D2088F.as_ptr()) as usize == ((*Keyword).Length as usize) / core::mem::size_of::<WCHAR>()
    {
        *ParameterValue = ExAllocatePool(
            PagedPool,
            core::mem::size_of::<NDIS_CONFIGURATION_PARAMETER>() as SIZE_T,
        ) as PNDIS_CONFIGURATION_PARAMETER;
        if (*ParameterValue).is_null() {
            NDIS_DbgPrint!(MIN_TRACE, b"Insufficient resources.\n\0" as *const u8 as PCSTR);
            *Status = NDIS_STATUS_RESOURCES;
            return;
        }

        MiniportResource = ExAllocatePool(
            PagedPool,
            core::mem::size_of::<MINIPORT_RESOURCE>() as SIZE_T,
        ) as PMINIPORT_RESOURCE;
        if MiniportResource.is_null() {
            NDIS_DbgPrint!(MIN_TRACE, b"Insufficient resources.\n\0" as *const u8 as PCSTR);
            ExFreePool(*ParameterValue as PVOID);
            *ParameterValue = core::ptr::null_mut();
            *Status = NDIS_STATUS_RESOURCES;
            return;
        }

        (*MiniportResource).ResourceType = MINIPORT_RESOURCE_TYPE_REGISTRY_DATA;
        (*MiniportResource).Resource = *ParameterValue as PVOID;
        NDIS_DbgPrint!(
            MID_TRACE,
            b"inserting 0x%x into the resource list\n\0" as *const u8 as PCSTR,
            (*MiniportResource).Resource,
        );
        ExInterlockedInsertTailList(
            core::ptr::addr_of_mut!((*ConfigurationContext).ResourceListHead),
            core::ptr::addr_of_mut!((*MiniportResource).ListEntry),
            core::ptr::addr_of_mut!((*ConfigurationContext).ResourceLock),
        );

        (**ParameterValue).ParameterType = NdisParameterInteger;
        (**ParameterValue).ParameterData.IntegerData = NdisProcessorX86; /* XXX non-portable */
        *Status = NDIS_STATUS_SUCCESS;

        return;
    }

    if wcsncmp((*Keyword).Buffer as *const u16, RUSTCONV_WSTR_CAECF7D590D8.as_ptr(), ((*Keyword).Length as usize) / core::mem::size_of::<WCHAR>()) == 0
        && wcslen(RUSTCONV_WSTR_CAECF7D590D8.as_ptr()) as usize == ((*Keyword).Length as usize) / core::mem::size_of::<WCHAR>()
    {
        *ParameterValue = ExAllocatePool(
            PagedPool,
            core::mem::size_of::<NDIS_CONFIGURATION_PARAMETER>() as SIZE_T,
        ) as PNDIS_CONFIGURATION_PARAMETER;
        if (*ParameterValue).is_null() {
            NDIS_DbgPrint!(MIN_TRACE, b"Insufficient resources.\n\0" as *const u8 as PCSTR);
            *Status = NDIS_STATUS_RESOURCES;
            return;
        }

        MiniportResource = ExAllocatePool(
            PagedPool,
            core::mem::size_of::<MINIPORT_RESOURCE>() as SIZE_T,
        ) as PMINIPORT_RESOURCE;
        if MiniportResource.is_null() {
            NDIS_DbgPrint!(MIN_TRACE, b"Insufficient resources.\n\0" as *const u8 as PCSTR);
            ExFreePool(*ParameterValue as PVOID);
            *ParameterValue = core::ptr::null_mut();
            *Status = NDIS_STATUS_RESOURCES;
            return;
        }

        (*MiniportResource).ResourceType = MINIPORT_RESOURCE_TYPE_REGISTRY_DATA;
        (*MiniportResource).Resource = *ParameterValue as PVOID;
        NDIS_DbgPrint!(
            MID_TRACE,
            b"inserting 0x%x into the resource list\n\0" as *const u8 as PCSTR,
            (*MiniportResource).Resource,
        );
        ExInterlockedInsertTailList(
            core::ptr::addr_of_mut!((*ConfigurationContext).ResourceListHead),
            core::ptr::addr_of_mut!((*MiniportResource).ListEntry),
            core::ptr::addr_of_mut!((*ConfigurationContext).ResourceLock),
        );

        (**ParameterValue).ParameterType = NdisParameterInteger;
        (**ParameterValue).ParameterData.IntegerData = NDIS_VERSION;
        *Status = NDIS_STATUS_SUCCESS;

        NDIS_DbgPrint!(
            MAX_TRACE,
            b"ParameterType = 0x%x, ParameterValue = 0x%x\n\0" as *const u8 as PCSTR,
            (**ParameterValue).ParameterType,
            (**ParameterValue).ParameterData.IntegerData,
        );
        return;
    }

    /* figure out how much buffer i should allocate */
    *Status = ZwQueryValueKey(
        (*ConfigurationContext).Handle as HANDLE,
        Keyword as PUNICODE_STRING,
        KeyValuePartialInformation,
        core::ptr::null_mut(),
        0 as ULONG,
        core::ptr::addr_of_mut!(KeyDataLength),
    ) as NDIS_STATUS;
    if *Status != STATUS_BUFFER_OVERFLOW && *Status != STATUS_BUFFER_TOO_SMALL && *Status != STATUS_SUCCESS {
        NDIS_DbgPrint!(
            MID_TRACE,
            b"ZwQueryValueKey #1 failed for %wZ, status 0x%x\n\0" as *const u8 as PCSTR,
            Keyword,
            *Status,
        );
        *Status = NDIS_STATUS_FAILURE;
        return;
    }

    /* allocate it */
    KeyInformation = ExAllocatePool(
        PagedPool,
        (KeyDataLength as SIZE_T) + (core::mem::size_of::<KEY_VALUE_PARTIAL_INFORMATION>() as SIZE_T),
    ) as *mut KEY_VALUE_PARTIAL_INFORMATION;
    if KeyInformation.is_null() {
        NDIS_DbgPrint!(MIN_TRACE, b"Insufficient resources.\n\0" as *const u8 as PCSTR);
        *Status = NDIS_STATUS_RESOURCES;
        return;
    }

    /* grab the value */
    *Status = ZwQueryValueKey(
        (*ConfigurationContext).Handle as HANDLE,
        Keyword as PUNICODE_STRING,
        KeyValuePartialInformation,
        KeyInformation as PVOID,
        KeyDataLength + (core::mem::size_of::<KEY_VALUE_PARTIAL_INFORMATION>() as ULONG),
        core::ptr::addr_of_mut!(KeyDataLength),
    ) as NDIS_STATUS;
    if *Status != STATUS_SUCCESS {
        ExFreePool(KeyInformation as PVOID);
        NDIS_DbgPrint!(
            MIN_TRACE,
            b"ZwQueryValueKey #2 failed for %wZ, status 0x%x\n\0" as *const u8 as PCSTR,
            Keyword,
            *Status,
        );
        *Status = NDIS_STATUS_FAILURE;
        return;
    }

    MiniportResource = ExAllocatePool(
        PagedPool,
        core::mem::size_of::<MINIPORT_RESOURCE>() as SIZE_T,
    ) as PMINIPORT_RESOURCE;
    if MiniportResource.is_null() {
        NDIS_DbgPrint!(MIN_TRACE, b"Insufficient resources.\n\0" as *const u8 as PCSTR);
        ExFreePool(KeyInformation as PVOID);
        *Status = NDIS_STATUS_RESOURCES;
        return;
    }

    *ParameterValue = ExAllocatePool(
        PagedPool,
        core::mem::size_of::<NDIS_CONFIGURATION_PARAMETER>() as SIZE_T,
    ) as PNDIS_CONFIGURATION_PARAMETER;
    if (*ParameterValue).is_null() {
        NDIS_DbgPrint!(MIN_TRACE, b"Insufficient resources.\n\0" as *const u8 as PCSTR);
        ExFreePool(MiniportResource as PVOID);
        ExFreePool(KeyInformation as PVOID);
        *Status = NDIS_STATUS_RESOURCES;
        return;
    }

    core::ptr::write_bytes(*ParameterValue as *mut u8, 0, core::mem::size_of::<NDIS_CONFIGURATION_PARAMETER>());

    if (*KeyInformation).Type == REG_BINARY {
        NDIS_DbgPrint!(MAX_TRACE, b"NdisParameterBinary\n\0" as *const u8 as PCSTR);

        (**ParameterValue).ParameterType = NdisParameterBinary;

        Buffer = ExAllocatePool(PagedPool, (*KeyInformation).DataLength as SIZE_T);
        if Buffer.is_null() {
            NDIS_DbgPrint!(MIN_TRACE, b"Insufficient resources.\n\0" as *const u8 as PCSTR);
            ExFreePool(MiniportResource as PVOID);
            ExFreePool(KeyInformation as PVOID);
            *Status = NDIS_STATUS_RESOURCES;
            return;
        }

        core::ptr::copy_nonoverlapping(
            core::ptr::addr_of_mut!((*KeyInformation).Data) as *const u8,
            Buffer as *mut u8,
            (*KeyInformation).DataLength as usize,
        );

        (**ParameterValue).ParameterData.BinaryData.Buffer = Buffer;
        (**ParameterValue).ParameterData.BinaryData.Length = (*KeyInformation).DataLength as USHORT;
    } else if (*KeyInformation).Type == REG_MULTI_SZ {
        NDIS_DbgPrint!(MAX_TRACE, b"NdisParameterMultiString\n\0" as *const u8 as PCSTR);

        (**ParameterValue).ParameterType = NdisParameterMultiString;

        Buffer = ExAllocatePool(PagedPool, (*KeyInformation).DataLength as SIZE_T);
        if Buffer.is_null() {
            NDIS_DbgPrint!(MIN_TRACE, b"Insufficient resources.\n\0" as *const u8 as PCSTR);
            ExFreePool(MiniportResource as PVOID);
            ExFreePool(KeyInformation as PVOID);
            *Status = NDIS_STATUS_RESOURCES;
            return;
        }

        core::ptr::copy_nonoverlapping(
            core::ptr::addr_of_mut!((*KeyInformation).Data) as *const u8,
            Buffer as *mut u8,
            (*KeyInformation).DataLength as usize,
        );

        (**ParameterValue).ParameterData.StringData.Buffer = Buffer as PWCH;
        (**ParameterValue).ParameterData.StringData.Length = (*KeyInformation).DataLength as USHORT;
    } else if (*KeyInformation).Type == REG_DWORD {
        NDIS_DbgPrint!(MAX_TRACE, b"NdisParameterInteger\n\0" as *const u8 as PCSTR);
        (**ParameterValue).ParameterType = NdisParameterInteger;
        (**ParameterValue).ParameterData.IntegerData =
            *((core::ptr::addr_of_mut!((*KeyInformation).Data) as *mut UCHAR)
                .offset(0 as isize) as *mut ULONG);
    } else if (*KeyInformation).Type == REG_SZ {
        let mut str: _UNICODE_STRING = core::mem::zeroed();
        let mut Base: ULONG = 0;

        if ParameterType == NdisParameterInteger {
            Base = 10;
        } else if ParameterType == NdisParameterHexInteger {
            Base = 16;
        } else {
            Base = 0;
        }

        str.MaximumLength = (*KeyInformation).DataLength as USHORT;
        str.Length = str.MaximumLength;
        str.Buffer = core::ptr::addr_of_mut!((*KeyInformation).Data) as *mut UCHAR as PWCHAR;

        if Base != 0 && IsValidNumericString(core::ptr::addr_of_mut!(str), Base) != 0 {
            *Status = RtlUnicodeStringToInteger(
                core::ptr::addr_of_mut!(str) as PCUNICODE_STRING,
                Base,
                core::ptr::addr_of_mut!((**ParameterValue).ParameterData.IntegerData),
            ) as NDIS_STATUS;

            NDIS_DbgPrint!(MAX_TRACE, b"NdisParameter(Hex)Integer\n\0" as *const u8 as PCSTR);

            /* MSDN documents that this is returned for all integers, regardless of the ParameterType passed in */
            (**ParameterValue).ParameterType = NdisParameterInteger;
        } else {
            NDIS_DbgPrint!(MAX_TRACE, b"NdisParameterString\n\0" as *const u8 as PCSTR);

            (**ParameterValue).ParameterType = NdisParameterString;

            Buffer = ExAllocatePool(PagedPool, (*KeyInformation).DataLength as SIZE_T);
            if Buffer.is_null() {
                NDIS_DbgPrint!(MIN_TRACE, b"Insufficient resources.\n\0" as *const u8 as PCSTR);
                ExFreePool(MiniportResource as PVOID);
                ExFreePool(KeyInformation as PVOID);
                *Status = NDIS_STATUS_RESOURCES;
                return;
            }

            core::ptr::copy_nonoverlapping(
                core::ptr::addr_of_mut!((*KeyInformation).Data) as *const u8,
                Buffer as *mut u8,
                (*KeyInformation).DataLength as usize,
            );

            (**ParameterValue).ParameterData.StringData.Buffer = Buffer as PWCH;
            (**ParameterValue).ParameterData.StringData.Length = (*KeyInformation).DataLength as USHORT;
        }
    } else {
        NDIS_DbgPrint!(
            MIN_TRACE,
            b"Invalid type for NdisReadConfiguration (%d)\n\0" as *const u8 as PCSTR,
            (*KeyInformation).Type,
        );
        NDIS_DbgPrint!(MIN_TRACE, b"Requested type: %d\n\0" as *const u8 as PCSTR, ParameterType);
        NDIS_DbgPrint!(MIN_TRACE, b"Registry entry: %wZ\n\0" as *const u8 as PCSTR, Keyword);
        *Status = NDIS_STATUS_FAILURE;
        ExFreePool(KeyInformation as PVOID);
        return;
    }

    if ((**ParameterValue).ParameterType != ParameterType)
        && !((ParameterType == NdisParameterHexInteger) && ((**ParameterValue).ParameterType == NdisParameterInteger))
    {
        NDIS_DbgPrint!(
            MIN_TRACE,
            b"Parameter type mismatch! (Requested: %d | Received: %d)\n\0" as *const u8 as PCSTR,
            ParameterType,
            (**ParameterValue).ParameterType,
        );
        NDIS_DbgPrint!(MIN_TRACE, b"Registry entry: %wZ\n\0" as *const u8 as PCSTR, Keyword);
    }

    (*MiniportResource).ResourceType = MINIPORT_RESOURCE_TYPE_REGISTRY_DATA;
    (*MiniportResource).Resource = *ParameterValue as PVOID;

    ExInterlockedInsertTailList(
        core::ptr::addr_of_mut!((*ConfigurationContext).ResourceListHead),
        core::ptr::addr_of_mut!((*MiniportResource).ListEntry),
        core::ptr::addr_of_mut!((*ConfigurationContext).ResourceLock),
    );

    ExFreePool(KeyInformation as PVOID);

    *Status = NDIS_STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "system" fn NdisReadNetworkAddress(
    Status: PNDIS_STATUS,
    NetworkAddress: *mut PVOID,
    NetworkAddressLength: PUINT,
    ConfigurationHandle: NDIS_HANDLE,
) {
    let mut ConfigurationContext: PMINIPORT_CONFIGURATION_CONTEXT =
        ConfigurationHandle as PMINIPORT_CONFIGURATION_CONTEXT;
    let mut MiniportResource: PMINIPORT_RESOURCE = core::ptr::null_mut();
    let mut ParameterValue: PNDIS_CONFIGURATION_PARAMETER = core::ptr::null_mut();
    let mut Keyword: NDIS_STRING = core::mem::zeroed();
    let mut IntArray: *mut UINT = core::ptr::null_mut();
    let mut i: UINT = 0;
    let mut j: UINT = 0;
    let mut Buff: [WCHAR; 11] = [0; 11];
    let mut str: NDIS_STRING = core::mem::zeroed();

    NdisInitUnicodeString(core::ptr::addr_of_mut!(Keyword), RUSTCONV_WSTR_95A26E22DCB6.as_ptr());
    NdisReadConfiguration(
        Status,
        core::ptr::addr_of_mut!(ParameterValue),
        ConfigurationHandle,
        core::ptr::addr_of_mut!(Keyword),
        NdisParameterString,
    );
    if *Status != NDIS_STATUS_SUCCESS {
        NDIS_DbgPrint!(
            MID_TRACE,
            b"NdisReadConfiguration failed (%x)\n\0".as_ptr() as PCSTR,
            *Status,
        );
        *Status = NDIS_STATUS_FAILURE;
        return;
    }

    if (*ParameterValue).ParameterType as u32 == NdisParameterInteger {
        NDIS_DbgPrint!(
            MAX_TRACE,
            b"Read integer data %lx\n\0".as_ptr() as PCSTR,
            (*ParameterValue).ParameterData.IntegerData,
        );

        str.Buffer = core::ptr::addr_of_mut!(Buff) as PWSTR;
        str.MaximumLength = core::mem::size_of::<[WCHAR; 11]>() as USHORT;
        str.Length = 0;

        *Status = RtlIntegerToUnicodeString(
            (*ParameterValue).ParameterData.IntegerData,
            10,
            core::ptr::addr_of_mut!(str),
        ) as NDIS_STATUS;

        if *Status != NDIS_STATUS_SUCCESS {
            NDIS_DbgPrint!(
                MIN_TRACE,
                b"RtlIntegerToUnicodeString failed (%x)\n\0".as_ptr() as PCSTR,
                *Status,
            );
            *Status = NDIS_STATUS_FAILURE;
            return;
        }

        NDIS_DbgPrint!(
            MAX_TRACE,
            b"Converted integer data into %wZ\n\0".as_ptr() as PCSTR,
            core::ptr::addr_of_mut!(str),
        );
    } else {
        str = (*ParameterValue).ParameterData.StringData;
    }

    while j < str.Length as UINT && *str.Buffer.offset(j as isize) != '\0' as WCHAR {
        j = j.wrapping_add(1);
    }

    *NetworkAddressLength =
        (j.wrapping_add(1) >> 1) as UINT;

    if *NetworkAddressLength == 0 {
        NDIS_DbgPrint!(
            MIN_TRACE,
            b"Empty NetworkAddress registry entry.\n\0".as_ptr() as PCSTR,
        );
        *Status = NDIS_STATUS_FAILURE;
        return;
    }

    IntArray = ExAllocatePool(
        PagedPool,
        (*NetworkAddressLength as SIZE_T).wrapping_mul(core::mem::size_of::<UINT>() as SIZE_T),
    ) as *mut UINT;
    if IntArray.is_null() {
        NDIS_DbgPrint!(
            MIN_TRACE,
            b"Insufficient resources.\n\0".as_ptr() as PCSTR,
        );
        *Status = NDIS_STATUS_RESOURCES;
        return;
    }

    MiniportResource = ExAllocatePool(
        PagedPool,
        core::mem::size_of::<MINIPORT_RESOURCE>() as SIZE_T,
    ) as PMINIPORT_RESOURCE;
    if MiniportResource.is_null() {
        NDIS_DbgPrint!(
            MIN_TRACE,
            b"Insufficient resources.\n\0".as_ptr() as PCSTR,
        );
        ExFreePool(IntArray as PVOID);
        *Status = NDIS_STATUS_RESOURCES;
        return;
    }

    (*MiniportResource).ResourceType = MINIPORT_RESOURCE_TYPE_MEMORY;
    (*MiniportResource).Resource = IntArray as PVOID;
    NDIS_DbgPrint!(
        MID_TRACE,
        b"inserting 0x%x into the resource list\n\0".as_ptr() as PCSTR,
        (*MiniportResource).Resource,
    );
    ExInterlockedInsertTailList(
        core::ptr::addr_of_mut!((*ConfigurationContext).ResourceListHead),
        core::ptr::addr_of_mut!((*MiniportResource).ListEntry),
        core::ptr::addr_of_mut!((*ConfigurationContext).ResourceLock),
    );

    /* convert from string to bytes */
    i = 0;
    while i < *NetworkAddressLength {
        *IntArray.offset(i as isize) = ((UnicodeToHexByte(
            *str.Buffer.offset((2 as UINT).wrapping_mul(i) as isize),
        ) as UINT)
            << 4)
            .wrapping_add(UnicodeToHexByte(*str.Buffer.offset(
                (2 as UINT)
                    .wrapping_mul(i)
                    .wrapping_add(1 as UINT) as isize,
            )) as UINT);
        i = i.wrapping_add(1);
    }

    *NetworkAddress = IntArray as PVOID;

    *Status = NDIS_STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "system" fn NdisOpenConfigurationKeyByIndex(
    Status: PNDIS_STATUS,
    ConfigurationHandle: NDIS_HANDLE,
    Index: ULONG,
    KeyName: PNDIS_STRING,
    KeyHandle: PNDIS_HANDLE,
) {
    let mut KeyInformation: *mut KEY_BASIC_INFORMATION = core::ptr::null_mut();
    let mut KeyInformationLength: ULONG = 0;
    let mut KeyAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut RegKeyHandle: NDIS_HANDLE = core::ptr::null_mut();
    let mut ConfigurationContext: PMINIPORT_CONFIGURATION_CONTEXT = core::ptr::null_mut();

    *KeyHandle = core::ptr::null_mut();

    *Status = ZwEnumerateKey(
        ConfigurationHandle,
        Index,
        KeyBasicInformation,
        core::ptr::null_mut(),
        0,
        &raw mut KeyInformationLength,
    );
    if *Status != STATUS_BUFFER_TOO_SMALL
        && *Status != STATUS_BUFFER_OVERFLOW
        && *Status != STATUS_SUCCESS
    {
        NDIS_DbgPrint!(
            MIN_TRACE,
            "ZwEnumerateKey failed (%x)\n\0".as_ptr() as PCSTR,
            *Status,
        );
        *Status = NDIS_STATUS_FAILURE;
        return;
    }

    KeyInformation = ExAllocatePool(
        PagedPool,
        ((KeyInformationLength as usize)
            .wrapping_add(core::mem::size_of::<KEY_BASIC_INFORMATION>() as usize)) as SIZE_T,
    ) as *mut KEY_BASIC_INFORMATION;
    if KeyInformation.is_null() {
        NDIS_DbgPrint!(
            MIN_TRACE,
            "Insufficient resources.\n\0".as_ptr() as PCSTR,
        );
        *Status = NDIS_STATUS_FAILURE;
        return;
    }

    *Status = ZwEnumerateKey(
        ConfigurationHandle,
        Index,
        KeyBasicInformation,
        KeyInformation as PVOID,
        (KeyInformationLength as usize)
            .wrapping_add(core::mem::size_of::<KEY_BASIC_INFORMATION>() as usize)
            as ULONG,
        &raw mut KeyInformationLength,
    );

    if *Status != STATUS_SUCCESS {
        NDIS_DbgPrint!(
            MIN_TRACE,
            "ZwEnumerateKey failed (%x)\n\0".as_ptr() as PCSTR,
            *Status,
        );
        ExFreePool(KeyInformation as PVOID);
        *Status = NDIS_STATUS_FAILURE;
        return;
    }

    /* should i fail instead if the passed-in string isn't long enough? */
    wcsncpy(
        (*KeyName).Buffer as *mut WCHAR,
        &raw mut (*KeyInformation).Name as *mut WCHAR,
        ((*KeyName).MaximumLength as usize)
            .wrapping_div(core::mem::size_of::<WCHAR>() as usize),
    );
    (*KeyName).Length = min(
        (*KeyInformation).NameLength as ULONG,
        (*KeyName).MaximumLength as ULONG,
    ) as USHORT;

    // InitializeObjectAttributes expanded inline
    KeyAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    KeyAttributes.RootDirectory = ConfigurationHandle as HANDLE;
    KeyAttributes.ObjectName = KeyName as PUNICODE_STRING;
    KeyAttributes.Attributes = (OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE) as ULONG;
    KeyAttributes.SecurityDescriptor = core::ptr::null_mut();
    KeyAttributes.SecurityQualityOfService = core::ptr::null_mut();

    *Status = ZwOpenKey(
        &raw mut RegKeyHandle,
        KEY_ALL_ACCESS,
        &raw mut KeyAttributes,
    );

    ExFreePool(KeyInformation as PVOID);

    if *Status != STATUS_SUCCESS {
        NDIS_DbgPrint!(
            MIN_TRACE,
            "ZwOpenKey failed (%x)\n\0".as_ptr() as PCSTR,
            *Status,
        );
        *Status = NDIS_STATUS_FAILURE;
        return;
    }

    ConfigurationContext = ExAllocatePool(
        NonPagedPool,
        core::mem::size_of::<MINIPORT_CONFIGURATION_CONTEXT>() as SIZE_T,
    ) as PMINIPORT_CONFIGURATION_CONTEXT;
    if ConfigurationContext.is_null() {
        NDIS_DbgPrint!(
            MIN_TRACE,
            "Insufficient resources.\n\0".as_ptr() as PCSTR,
        );
        ZwClose(RegKeyHandle as HANDLE);
        *Status = NDIS_STATUS_FAILURE;
        return;
    }

    KeInitializeSpinLock(&raw mut (*ConfigurationContext).ResourceLock);
    InitializeListHead(&raw mut (*ConfigurationContext).ResourceListHead);

    (*ConfigurationContext).Handle = RegKeyHandle;

    *KeyHandle = ConfigurationContext as NDIS_HANDLE;

    *Status = NDIS_STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "system" fn NdisOpenConfigurationKeyByName(
    Status: PNDIS_STATUS,
    ConfigurationHandle: NDIS_HANDLE,
    KeyName: PNDIS_STRING,
    KeyHandle: PNDIS_HANDLE,
) {
    let mut ConfigurationContext: PMINIPORT_CONFIGURATION_CONTEXT = core::ptr::null_mut();
    let mut KeyAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut RegKeyHandle: NDIS_HANDLE = core::ptr::null_mut();

    *KeyHandle = core::ptr::null_mut();

    // InitializeObjectAttributes(&KeyAttributes, KeyName, OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE, ConfigurationHandle, NULL);
    KeyAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    KeyAttributes.RootDirectory = ConfigurationHandle as HANDLE;
    KeyAttributes.ObjectName = KeyName as PUNICODE_STRING;
    KeyAttributes.Attributes = (OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE) as ULONG;
    KeyAttributes.SecurityDescriptor = 0 as PVOID;
    KeyAttributes.SecurityQualityOfService = core::ptr::null_mut();

    *Status = ZwOpenKey(
        &raw mut RegKeyHandle as PHANDLE,
        KEY_ALL_ACCESS as ACCESS_MASK,
        &raw mut KeyAttributes,
    ) as NDIS_STATUS;

    if *Status as NTSTATUS != STATUS_SUCCESS as NTSTATUS {
        NDIS_DbgPrint!(
            MIN_TRACE,
            b"ZwOpenKey failed (%x)\n\0".as_ptr() as *const i8,
            *Status,
        );
        *Status = NDIS_STATUS_FAILURE as NDIS_STATUS;
        return;
    }

    ConfigurationContext = ExAllocatePool(
        NonPagedPool as POOL_TYPE,
        core::mem::size_of::<MINIPORT_CONFIGURATION_CONTEXT>() as SIZE_T,
    ) as PMINIPORT_CONFIGURATION_CONTEXT;
    if ConfigurationContext.is_null() {
        NDIS_DbgPrint!(
            MIN_TRACE,
            b"Insufficient resources.\n\0".as_ptr() as *const i8,
        );
        ZwClose(RegKeyHandle as HANDLE);
        *Status = NDIS_STATUS_FAILURE as NDIS_STATUS;
        return;
    }

    KeInitializeSpinLock(&raw mut (*ConfigurationContext).ResourceLock);
    InitializeListHead(&raw mut (*ConfigurationContext).ResourceListHead);

    (*ConfigurationContext).Handle = RegKeyHandle;

    *KeyHandle = ConfigurationContext as NDIS_HANDLE;

    *Status = NDIS_STATUS_SUCCESS as NDIS_STATUS;
}
