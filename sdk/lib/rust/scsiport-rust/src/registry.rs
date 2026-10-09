#[allow(unused_imports)] use crate::ddk_shims::*;
#[allow(unused_imports)] use crate::glue::*;
#[allow(unused_imports)] use crate::inlines::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{_OBJECT_ATTRIBUTES, PUNICODE_STRING, RtlInitUnicodeString, ZwClose, ZwCreateKey, ZwOpenKey, ZwSetValueKey, _UNICODE_STRING, wchar_t};
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{ACCESS_RANGE, CONFIGURATION_INFO, INQUIRYDATA, PSCSI_PORT_DEVICE_EXTENSION, PSCSI_PORT_LUN_EXTENSION, SCSI_PORT_DEVICE_EXTENSION, SCSI_PORT_DRIVER_EXTENSION};
// resolve_imports.py: explicit kernel/private-type imports
use ntkrnl_sys::driver_ddk::{UNICODE_NULL};

extern "C" {
    fn DbgPrint(Format: *const u8, ...) -> i32;
    fn swprintf(buffer: *mut u16, format: *const u16, ...) -> i32;
    fn wcslen(s: *const wchar_t) -> usize;
}

// ---- file-scope preamble (translated) ----
pub const NDEBUG: i32 = 1;

const REG_OPTION_VOLATILE: u32 = 1;
const KEY_ALL_ACCESS: u32 = 0xF003F;
#[no_mangle]
pub unsafe extern "C" fn SpiInitOpenKeys(
    ConfigInfo: *mut CONFIGURATION_INFO,
    DriverExtension: *mut SCSI_PORT_DRIVER_EXTENSION,
) {
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut KeyName: _UNICODE_STRING = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let mut parametersKey: HANDLE = core::ptr::null_mut();

    (*DriverExtension).IsLegacyDriver = 1; // TRUE

    // Open the service key
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.ObjectName = core::ptr::addr_of_mut!((*DriverExtension).RegistryPath);
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = ZwOpenKey(
        core::ptr::addr_of_mut!((*ConfigInfo).ServiceKey),
        KEY_READ,
        &mut ObjectAttributes,
    );

    if Status < 0 {
        (*ConfigInfo).ServiceKey = core::ptr::null_mut();
    }

    if !(*ConfigInfo).ServiceKey.is_null() {
        // Parameters key
        let parameters_wide: [u16; 11] = [
            0x50, 0x61, 0x72, 0x61, 0x6d, 0x65, 0x74, 0x65, 0x72, 0x73, 0x0,
        ];
        RtlInitUnicodeString(&mut KeyName, parameters_wide.as_ptr());

        ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
        ObjectAttributes.RootDirectory = (*ConfigInfo).ServiceKey;
        ObjectAttributes.ObjectName = core::ptr::addr_of_mut!(KeyName);
        ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE;
        ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
        ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

        Status = ZwOpenKey(
            core::ptr::addr_of_mut!((*ConfigInfo).DeviceKey),
            KEY_READ,
            &mut ObjectAttributes,
        );

        if Status >= 0 {
            ZwClose((*ConfigInfo).ServiceKey);
            (*ConfigInfo).ServiceKey = (*ConfigInfo).DeviceKey;
            (*ConfigInfo).DeviceKey = core::ptr::null_mut();
        }
    }

    if !(*ConfigInfo).ServiceKey.is_null() {
        // Device key
        let device_wide: [u16; 7] = [
            0x44, 0x65, 0x76, 0x69, 0x63, 0x65, 0x0,
        ];
        RtlInitUnicodeString(&mut KeyName, device_wide.as_ptr());

        ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
        ObjectAttributes.RootDirectory = (*ConfigInfo).ServiceKey;
        ObjectAttributes.ObjectName = core::ptr::addr_of_mut!(KeyName);
        ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE;
        ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
        ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

        ZwOpenKey(
            core::ptr::addr_of_mut!((*ConfigInfo).DeviceKey),
            KEY_READ,
            &mut ObjectAttributes,
        );

        // PnpInterface key
        let pnp_wide: [u16; 13] = [
            0x50, 0x6e, 0x70, 0x49, 0x6e, 0x74, 0x65, 0x72, 0x66, 0x61, 0x63, 0x65, 0x0,
        ];
        RtlInitUnicodeString(&mut KeyName, pnp_wide.as_ptr());

        ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
        ObjectAttributes.RootDirectory = (*ConfigInfo).ServiceKey;
        ObjectAttributes.ObjectName = core::ptr::addr_of_mut!(KeyName);
        ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE;
        ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
        ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

        Status = ZwOpenKey(&mut parametersKey, KEY_READ, &mut ObjectAttributes);

        if Status >= 0 {
            (*DriverExtension).IsLegacyDriver = 0; // FALSE
            ZwClose(parametersKey);
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn RegistryInitAdapterKey(
    DeviceExtension: *mut SCSI_PORT_DEVICE_EXTENSION,
) -> NTSTATUS {
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut KeyName: _UNICODE_STRING = core::mem::zeroed();
    let mut ValueName: _UNICODE_STRING = core::mem::zeroed();
    let mut NameBuffer: [u16; 64] = [0; 64];
    let mut ScsiKey: HANDLE = core::ptr::null_mut();
    let mut ScsiPortKey: HANDLE = core::ptr::null_mut();
    let mut ScsiBusKey: HANDLE = core::ptr::null_mut();
    let mut ScsiInitiatorKey: HANDLE = core::ptr::null_mut();
    let mut BusNumber: u32;
    let mut UlongData: u32 ;
    let mut Status: NTSTATUS;

    if DeviceExtension.is_null() {
        if DbgPrint(
            b"(%s:%d) Invalid parameter\n\0".as_ptr(),
            b"drivers/storage/port/scsiport/registry.c\0".as_ptr() as *const i8,
            146i32,
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0".as_ptr(),
                b"drivers/storage/port/scsiport/registry.c\0".as_ptr() as *const i8,
                146i32,
            );
        }
        return STATUS_INVALID_PARAMETER;
    }

    RtlInitUnicodeString(
        &raw mut KeyName,
        &[
            b'\\' as u16, 0, (b'R' as u16) as u16, 0, (b'e' as u16) as u16, 0, (b'g' as u16) as u16, 0, b'i' as u16, 0, b's' as u16, 0, b't' as u16, 0, b'r' as u16, 0, b'y' as u16, 0,
            b'\\' as u16, 0, b'M' as u16, 0, b'a' as u16, 0, b'c' as u16, 0, b'h' as u16, 0, b'i' as u16, 0, b'n' as u16, 0, b'e' as u16, 0,
            b'\\' as u16, 0, b'H' as u16, 0, b'a' as u16, 0, b'r' as u16, 0, b'd' as u16, 0, b'w' as u16, 0, b'a' as u16, 0, b'r' as u16, 0, b'e' as u16, 0,
            b'\\' as u16, 0, b'D' as u16, 0, b'e' as u16, 0, b'v' as u16, 0, b'i' as u16, 0, b'c' as u16, 0, b'e' as u16, 0, b'M' as u16, 0, b'a' as u16, 0, b'p' as u16, 0,
            b'\\' as u16, 0, b'S' as u16, 0, b'c' as u16, 0, b's' as u16, 0, b'i' as u16, 0, 0, 0,
        ] as *const u16,
    );

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.ObjectName = &raw mut KeyName as PUNICODE_STRING;
    ObjectAttributes.Attributes = (OBJ_CASE_INSENSITIVE | OBJ_OPENIF | OBJ_KERNEL_HANDLE) as u32;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = ZwCreateKey(
        &raw mut ScsiKey,
        KEY_ALL_ACCESS,
        &raw mut ObjectAttributes,
        0,
        core::ptr::null_mut(),
        REG_OPTION_VOLATILE,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    swprintf(
        NameBuffer.as_mut_ptr(),
        &[
            b'S' as u16, 0, (b'c' as u16) as u16, 0, (b's' as u16) as u16, 0, (b'i' as u16) as u16, 0, b' ' as u16, 0,
            b'P' as u16, 0, b'o' as u16, 0, b'r' as u16, 0, b't' as u16, 0, b' ' as u16, 0,
            b'%' as u16, 0, b'l' as u16, 0, b'u' as u16, 0, 0, 0,
        ] as *const u16,
        (*DeviceExtension).PortNumber,
    );
    RtlInitUnicodeString(
        &raw mut KeyName,
        NameBuffer.as_mut_ptr() as *const u16,
    );

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.RootDirectory = ScsiKey;
    ObjectAttributes.ObjectName = &raw mut KeyName as PUNICODE_STRING;
    ObjectAttributes.Attributes = OBJ_KERNEL_HANDLE as u32;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = ZwCreateKey(
        &raw mut ScsiPortKey,
        KEY_ALL_ACCESS,
        &raw mut ObjectAttributes,
        0,
        core::ptr::null_mut(),
        REG_OPTION_VOLATILE,
        core::ptr::null_mut(),
    );
    ZwClose(ScsiKey);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    UlongData = ((*DeviceExtension).PortCapabilities.AdapterUsesPio == 0) as i32 as u32;

    RtlInitUnicodeString(
        &raw mut ValueName,
        &[
            b'D' as u16, 0, (b'M' as u16) as u16, 0, (b'A' as u16) as u16, 0, (b' ' as u16) as u16, 0,
            b'E' as u16, 0, b'n' as u16, 0, b'a' as u16, 0, b'b' as u16, 0, b'l' as u16, 0, b'e' as u16, 0, b'd' as u16, 0, 0, 0,
        ] as *const u16,
    );

    Status = ZwSetValueKey(
        ScsiPortKey,
        &raw mut ValueName,
        0,
        REG_DWORD,
        &raw mut UlongData as *mut c_void,
        core::mem::size_of::<u32>() as u32,
    );
    if !NT_SUCCESS(Status) {
        ZwClose(ScsiPortKey);
        return Status;
    }

    let driverNameU: PUNICODE_STRING = &raw mut (*(*(*(*DeviceExtension).Common.DeviceObject).DriverObject).DriverExtension).ServiceKeyName;
    let driverName: *mut u16 = ExAllocatePoolWithTag(
        PagedPool,
        (*driverNameU).Length as usize + core::mem::size_of::<u16>() as usize,
        TAG_SCSIPORT,
    ) as *mut u16;

    if driverName.is_null() {
        ZwClose(ScsiPortKey);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    RtlCopyMemory(
        driverName as *mut c_void,
        (*driverNameU).Buffer as *const c_void,
        (*driverNameU).Length as usize,
    );
    *driverName.add(((*driverNameU).Length as usize) / core::mem::size_of::<u16>()) = UNICODE_NULL;

    RtlInitUnicodeString(
        &raw mut ValueName,
        &[
            b'D' as u16, 0, (b'r' as u16) as u16, 0, (b'i' as u16) as u16, 0, (b'v' as u16) as u16, 0, b'e' as u16, 0, b'r' as u16, 0, 0, 0,
        ] as *const u16,
    );

    Status = ZwSetValueKey(
        ScsiPortKey,
        &raw mut ValueName,
        0,
        REG_SZ,
        driverName as *mut c_void,
        ((*driverNameU).Length as usize + core::mem::size_of::<u16>() as usize) as u32,
    );
    ExFreePoolWithTag(driverName as *mut c_void, TAG_SCSIPORT);
    if !NT_SUCCESS(Status) {
        ZwClose(ScsiPortKey);
        return Status;
    }

    UlongData = (*(*DeviceExtension).PortConfig).BusInterruptLevel;

    RtlInitUnicodeString(
        &raw mut ValueName,
        &[
            b'I' as u16, 0, (b'n' as u16) as u16, 0, (b't' as u16) as u16, 0, (b'e' as u16) as u16, 0, b'r' as u16, 0, b'r' as u16, 0, b'u' as u16, 0, b'p' as u16, 0, b't' as u16, 0, 0, 0,
        ] as *const u16,
    );

    Status = ZwSetValueKey(
        ScsiPortKey,
        &raw mut ValueName,
        0,
        REG_DWORD,
        &raw mut UlongData as *mut c_void,
        core::mem::size_of::<u32>() as u32,
    );
    if !NT_SUCCESS(Status) {
        ZwClose(ScsiPortKey);
        return Status;
    }

    UlongData = (*((*(*DeviceExtension).PortConfig).AccessRanges as *mut ACCESS_RANGE)).RangeStart.u.LowPart;

    RtlInitUnicodeString(
        &raw mut ValueName,
        &[
            b'I' as u16, 0, (b'O' as u16) as u16, 0, (b'A' as u16) as u16, 0, (b'd' as u16) as u16, 0, b'd' as u16, 0, b'r' as u16, 0, b'e' as u16, 0, b's' as u16, 0, b's' as u16, 0, 0, 0,
        ] as *const u16,
    );

    Status = ZwSetValueKey(
        ScsiPortKey,
        &raw mut ValueName,
        0,
        REG_DWORD,
        &raw mut UlongData as *mut c_void,
        core::mem::size_of::<u32>() as u32,
    );
    if !NT_SUCCESS(Status) {
        ZwClose(ScsiPortKey);
        return Status;
    }

    BusNumber = 0;
    while BusNumber < (*DeviceExtension).NumberOfBuses as u32 {
        swprintf(
            NameBuffer.as_mut_ptr(),
            &[
                b'S' as u16, 0, (b'c' as u16) as u16, 0, (b's' as u16) as u16, 0, (b'i' as u16) as u16, 0, b' ' as u16, 0,
                b'B' as u16, 0, b'u' as u16, 0, b's' as u16, 0, b' ' as u16, 0,
                b'%' as u16, 0, b'l' as u16, 0, b'u' as u16, 0, 0, 0,
            ] as *const u16,
            BusNumber,
        );
        RtlInitUnicodeString(
            &raw mut KeyName,
            NameBuffer.as_mut_ptr() as *const u16,
        );

        ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
        ObjectAttributes.RootDirectory = ScsiPortKey;
        ObjectAttributes.ObjectName = &raw mut KeyName as PUNICODE_STRING;
        ObjectAttributes.Attributes = OBJ_KERNEL_HANDLE as u32;
        ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
        ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

        Status = ZwCreateKey(
            &raw mut ScsiBusKey,
            KEY_ALL_ACCESS,
            &raw mut ObjectAttributes,
            0,
            core::ptr::null_mut(),
            REG_OPTION_VOLATILE,
            core::ptr::null_mut(),
        );
        if !NT_SUCCESS(Status) {
            break;
        }

        swprintf(
            NameBuffer.as_mut_ptr(),
            &[
                b'I' as u16, 0, (b'n' as u16) as u16, 0, (b'i' as u16) as u16, 0, (b't' as u16) as u16, 0, b'i' as u16, 0, b'a' as u16, 0, b't' as u16, 0, b'o' as u16, 0, b'r' as u16, 0, b' ' as u16, 0,
                b'I' as u16, 0, b'd' as u16, 0, b' ' as u16, 0,
                b'%' as u16, 0, b'l' as u16, 0, b'u' as u16, 0, 0, 0,
            ] as *const u16,
            (*(*DeviceExtension).PortConfig).InitiatorBusId[BusNumber as usize] as u32,
        );
        RtlInitUnicodeString(
            &raw mut KeyName,
            NameBuffer.as_mut_ptr() as *const u16,
        );

        ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
        ObjectAttributes.RootDirectory = ScsiBusKey;
        ObjectAttributes.ObjectName = &raw mut KeyName as PUNICODE_STRING;
        ObjectAttributes.Attributes = OBJ_KERNEL_HANDLE as u32;
        ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
        ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

        Status = ZwCreateKey(
            &raw mut ScsiInitiatorKey,
            KEY_ALL_ACCESS,
            &raw mut ObjectAttributes,
            0,
            core::ptr::null_mut(),
            REG_OPTION_VOLATILE,
            core::ptr::null_mut(),
        );
        if !NT_SUCCESS(Status) {
            break;
        }

        ZwClose(ScsiInitiatorKey);
        ScsiInitiatorKey = core::ptr::null_mut();

        let registry_key = &mut (*(*DeviceExtension).Buses.offset(BusNumber as isize)).RegistryMapKey;
        *registry_key = ScsiBusKey;
        ScsiBusKey = core::ptr::null_mut();

        BusNumber = BusNumber.wrapping_add(1);
    }

    if !ScsiInitiatorKey.is_null() {
        ZwClose(ScsiInitiatorKey);
    }
    if !ScsiBusKey.is_null() {
        ZwClose(ScsiBusKey);
    }
    if !ScsiPortKey.is_null() {
        ZwClose(ScsiPortKey);
    }

    Status
}
#[no_mangle]
pub unsafe extern "C" fn RegistryInitLunKey(
    mut LunExtension: PSCSI_PORT_LUN_EXTENSION,
) -> NTSTATUS {
    let mut typeName: PWCHAR ;
    let mut nameBuffer: [WCHAR; 64] = [0u16; 64];
    let mut keyName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut valueName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut objectAttributes: _OBJECT_ATTRIBUTES = _OBJECT_ATTRIBUTES {
        Length: 0,
        RootDirectory: core::ptr::null_mut(),
        ObjectName: core::ptr::null_mut(),
        Attributes: 0,
        SecurityDescriptor: core::ptr::null_mut(),
        SecurityQualityOfService: core::ptr::null_mut(),
    };
    let mut targetKey: HANDLE = core::ptr::null_mut();
    let mut status: NTSTATUS;
    let mut portExt: PSCSI_PORT_DEVICE_EXTENSION =
        (*(*LunExtension).Common.LowerDevice).DeviceExtension as PSCSI_PORT_DEVICE_EXTENSION;
    let mut busKey: HANDLE =
        (*(*portExt).Buses.offset((*LunExtension).PathId as isize)).RegistryMapKey;

    swprintf(
        &raw mut nameBuffer as *mut wchar_t,
        { const W: [u16; 14] = unsafe { core::mem::transmute(*b"T\0a\0r\0g\0e\0t\0 \0I\0d\0 \0%\0l\0u\0\0\0") }; W.as_ptr() } as *const wchar_t,
        (*LunExtension).TargetId as i32,
    );
    RtlInitUnicodeString(
        &raw mut keyName,
        &raw mut nameBuffer as *mut WCHAR as PCWSTR,
    );
    objectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    objectAttributes.RootDirectory = busKey;
    objectAttributes.ObjectName = &raw mut keyName as PUNICODE_STRING;
    objectAttributes.Attributes = 0x200; // OBJ_KERNEL_HANDLE
    objectAttributes.SecurityDescriptor = core::ptr::null_mut();
    objectAttributes.SecurityQualityOfService = core::ptr::null_mut();
    status = ZwCreateKey(
        &raw mut targetKey,
        0xF003F, // KEY_ALL_ACCESS
        &raw mut objectAttributes,
        0,
        core::ptr::null_mut(),
        1, // REG_OPTION_VOLATILE
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(status) {
        return status;
    }
    swprintf(
        &raw mut nameBuffer as *mut wchar_t,
        { const W: [u16; 20] = unsafe { core::mem::transmute(*b"L\0o\0g\0i\0c\0a\0l\0 \0U\0n\0i\0t\0 \0I\0d\0 \0%\0l\0u\0\0\0") }; W.as_ptr() } as *const wchar_t,
        (*LunExtension).Lun as i32,
    );
    RtlInitUnicodeString(
        &raw mut keyName,
        &raw mut nameBuffer as *mut WCHAR as PCWSTR,
    );
    objectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    objectAttributes.RootDirectory = targetKey;
    objectAttributes.ObjectName = &raw mut keyName as PUNICODE_STRING;
    objectAttributes.Attributes = 0x200; // OBJ_KERNEL_HANDLE
    objectAttributes.SecurityDescriptor = core::ptr::null_mut();
    objectAttributes.SecurityQualityOfService = core::ptr::null_mut();
    status = ZwCreateKey(
        &raw mut (*LunExtension).RegistryMapKey,
        0xF003F, // KEY_ALL_ACCESS
        &raw mut objectAttributes,
        0,
        core::ptr::null_mut(),
        1, // REG_OPTION_VOLATILE
        core::ptr::null_mut(),
    );
    if NT_SUCCESS(status) {
        swprintf(
            &raw mut nameBuffer as *mut wchar_t,
            { const W: [u16; 14] = unsafe { core::mem::transmute(*b"%\0.\08\0S\0%\0.\x001\x006\0S\0%\0.\x004\0S\0\0\0") }; W.as_ptr() } as *const wchar_t,
            &raw mut (*LunExtension).InquiryData.VendorId as *mut u8,
            &raw mut (*LunExtension).InquiryData.ProductId as *mut u8,
            &raw mut (*LunExtension).InquiryData.ProductRevisionLevel as *mut u8,
        );
        RtlInitUnicodeString(
            &raw mut valueName,
            { const W: [u16; 11] = unsafe { core::mem::transmute(*b"I\0d\0e\0n\0t\0i\0f\0i\0e\0r\0\0\0") }; W.as_ptr() } as PCWSTR,
        );
        status = ZwSetValueKey(
            (*LunExtension).RegistryMapKey,
            &raw mut valueName,
            0,
            1, // REG_SZ
            &raw mut nameBuffer as *mut WCHAR as *mut core::ffi::c_void,
            (wcslen(&raw mut nameBuffer as *mut WCHAR) as usize)
                .wrapping_add(1)
                .wrapping_mul(core::mem::size_of::<WCHAR>()) as u32,
        );
        if NT_SUCCESS(status) {
            typeName = GetPeripheralTypeW(&raw mut (*LunExtension).InquiryData) as PWCHAR;
            RtlInitUnicodeString(
                &raw mut valueName,
                { const W: [u16; 5] = unsafe { core::mem::transmute(*b"T\0y\0p\0e\0\0\0") }; W.as_ptr() } as PCWSTR,
            );
            status = ZwSetValueKey(
                (*LunExtension).RegistryMapKey,
                &raw mut valueName,
                0,
                1, // REG_SZ
                typeName as *mut core::ffi::c_void,
                (wcslen(typeName) as usize)
                    .wrapping_add(1)
                    .wrapping_mul(core::mem::size_of::<WCHAR>()) as u32,
            );
            if NT_SUCCESS(status) {
                RtlInitUnicodeString(
                    &raw mut valueName,
                    { const W: [u16; 12] = unsafe { core::mem::transmute(*b"I\0n\0q\0u\0i\0r\0y\0D\0a\0t\0a\0\0\0") }; W.as_ptr() } as PCWSTR,
                );
                status = ZwSetValueKey(
                    (*LunExtension).RegistryMapKey,
                    &raw mut valueName,
                    0,
                    3, // REG_BINARY
                    &raw mut (*LunExtension).InquiryData as *mut core::ffi::c_void,
                    36, // INQUIRYDATABUFFERSIZE
                );
            }
        }
    }
    ZwClose(targetKey);
    ZwClose((*LunExtension).RegistryMapKey);
    return status;
}
