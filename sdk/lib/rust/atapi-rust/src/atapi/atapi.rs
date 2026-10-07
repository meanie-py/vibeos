// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
pub static RUSTCONV_WSTR_01D8877A5F07: [u16; 23] = [
    0x005Cu16, 0x0044, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065, 0x005C, 0x0049, 0x0064,
    0x0065, 0x005C, 0x0049, 0x0064, 0x0065, 0x0050, 0x006F, 0x0072, 0x0074, 0x0025,
    0x006C, 0x0075, 0x0000,
];
pub static RUSTCONV_WSTR_124A192A3DF6: [u16; 58] = [
    0x005Cu16, 0x0052, 0x0065, 0x0067, 0x0069, 0x0073, 0x0074, 0x0072, 0x0079, 0x005C,
    0x004D, 0x0061, 0x0063, 0x0068, 0x0069, 0x006E, 0x0065, 0x005C, 0x0053, 0x0079,
    0x0073, 0x0074, 0x0065, 0x006D, 0x005C, 0x0043, 0x0075, 0x0072, 0x0072, 0x0065,
    0x006E, 0x0074, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F, 0x006C, 0x0053,
    0x0065, 0x0074, 0x005C, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F, 0x006C,
    0x005C, 0x004D, 0x0069, 0x006E, 0x0069, 0x004E, 0x0054, 0x0000,
];
pub static RUSTCONV_WSTR_447B6E0274D3: [u16; 9] = [0x0054u16, 0x0061, 0x0072, 0x0067, 0x0065, 0x0074, 0x0025, 0x0075, 0x0000];
pub static RUSTCONV_WSTR_604F52A85CB9: [u16; 12] = [
    0x005Cu16, 0x0044, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065, 0x005C, 0x0049, 0x0064,
    0x0065, 0x0000,
];

use crate::prelude::*;
use ntkrnl_sys::ddk::{ExAllocatePoolUninitialized, _KWAIT_REASON_Executive, GUID, _IO_STATUS_BLOCK, IoAllocateIrp, IoAllocateWorkItem, IoAttachDeviceToDeviceStack, IoBuildDeviceIoControlRequest, IoBuildSynchronousFsdRequest, IoCreateDevice, IoDeleteDevice, IoDetachDevice, IoForwardIrpSynchronously, IoFreeIrp, IoFreeWorkItem, IoGetAttachedDeviceReference, IoInvalidateDeviceState, IoOpenDeviceRegistryKey, IoQueueWorkItem, KEVENT, KEY_VALUE_PARTIAL_INFORMATION, KeInitializeDpc, KeInitializeEvent, KeInitializeSpinLock, KeWaitForSingleObject, _EVENT_TYPE_NotificationEvent, _OBJECT_ATTRIBUTES, PCSTR, PDEVICE_CAPABILITIES, PDEVICE_OBJECT, PDRIVER_OBJECT, PHANDLE, PIO_STACK_LOCATION, PIO_WORKITEM, PIRP, PKEY_VALUE_PARTIAL_INFORMATION, PUNICODE_STRING, RtlCopyUnicodeString, RtlFreeUnicodeString, RtlInitUnicodeString, STORAGE_EVENT_NOTIFICATION, _KWAIT_REASON_Suspended, ZwClose, ZwCreateDirectoryObject, ZwCreateKey, ZwOpenKey, ZwQueryValueKey, ZwSetValueKey};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_168 {
    pub DeviceType: PCSTR,
    pub GenericType: PCSTR,
    pub PeripheralId: PCSTR,
}

#[no_mangle]
pub static mut AtapDriverRegistryPath: _UNICODE_STRING = _UNICODE_STRING {
    Length: 0,
    MaximumLength: 0,
    Buffer: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
};

#[no_mangle]
pub static mut AtapInPEMode: BOOLEAN = 0;

static mut AtapGenericDeviceNames: [C2RustUnnamed_168; 11] = [
    C2RustUnnamed_168 {
        DeviceType: b"Disk\0" as *const u8 as PCSTR,
        GenericType: b"GenDisk\0" as *const u8 as PCSTR,
        PeripheralId: b"DiskPeripheral\0" as *const u8 as PCSTR,
    },
    C2RustUnnamed_168 {
        DeviceType: b"Sequential\0" as *const u8 as PCSTR,
        GenericType: b"GenSequential\0" as *const u8 as PCSTR,
        PeripheralId: b"TapePeripheral\0" as *const u8 as PCSTR,
    },
    C2RustUnnamed_168 {
        DeviceType: b"Printer\0" as *const u8 as PCSTR,
        GenericType: b"GenPrinter\0" as *const u8 as PCSTR,
        PeripheralId: b"PrinterPeripheral\0" as *const u8 as PCSTR,
    },
    C2RustUnnamed_168 {
        DeviceType: b"Processor\0" as *const u8 as PCSTR,
        GenericType: b"GenProcessor\0" as *const u8 as PCSTR,
        PeripheralId: b"ProcessorPeripheral\0" as *const u8 as PCSTR,
    },
    C2RustUnnamed_168 {
        DeviceType: b"Worm\0" as *const u8 as PCSTR,
        GenericType: b"GenWorm\0" as *const u8 as PCSTR,
        PeripheralId: b"WormPeripheral\0" as *const u8 as PCSTR,
    },
    C2RustUnnamed_168 {
        DeviceType: b"CdRom\0" as *const u8 as PCSTR,
        GenericType: b"GenCdRom\0" as *const u8 as PCSTR,
        PeripheralId: b"CdRomPeripheral\0" as *const u8 as PCSTR,
    },
    C2RustUnnamed_168 {
        DeviceType: b"Scanner\0" as *const u8 as PCSTR,
        GenericType: b"GenScanner\0" as *const u8 as PCSTR,
        PeripheralId: b"ScannerPeripheral\0" as *const u8 as PCSTR,
    },
    C2RustUnnamed_168 {
        DeviceType: b"Optical\0" as *const u8 as PCSTR,
        GenericType: b"GenOptical\0" as *const u8 as PCSTR,
        PeripheralId: b"OpticalDiskPeripheral\0" as *const u8 as PCSTR,
    },
    C2RustUnnamed_168 {
        DeviceType: b"Changer\0" as *const u8 as PCSTR,
        GenericType: b"GenChanger\0" as *const u8 as PCSTR,
        PeripheralId: b"MediumChangerPeripheral\0" as *const u8 as PCSTR,
    },
    C2RustUnnamed_168 {
        DeviceType: b"Net\0" as *const u8 as PCSTR,
        GenericType: b"GenNet\0" as *const u8 as PCSTR,
        PeripheralId: b"CommunicationsPeripheral\0" as *const u8 as PCSTR,
    },
    C2RustUnnamed_168 {
        DeviceType: b"Other\0" as *const u8 as PCSTR,
        GenericType: b"IdeOther\0" as *const u8 as PCSTR,
        PeripheralId: b"OtherPeripheral\0" as *const u8 as PCSTR,
    },
];

#[repr(C)]
pub struct _ATAP_GENERIC_DEVICE_NAME {
    pub DeviceType: PCSTR,
    pub GenericType: PCSTR,
    pub PeripheralId: PCSTR,
}
#[no_mangle]
pub unsafe extern "C" fn AtaTypeCodeToName(
    DevExt: *mut ATAPORT_DEVICE_EXTENSION,
    Type: DEVICE_TYPE_NAME,
) -> PCSTR {
    let mut DeviceType: u32;

    PAGED_CODE!();

    DeviceType = (*DevExt).InquiryData.DeviceType() as u32;
    DeviceType = min(
        DeviceType,
        RTL_NUMBER_OF!(AtapGenericDeviceNames) - 1,
    );

    match Type {
        GetDeviceType => return AtapGenericDeviceNames[DeviceType as usize].DeviceType,
        GetPeripheralId => return AtapGenericDeviceNames[DeviceType as usize].PeripheralId,
        GetGenericType => {
            if (*DevExt).Device.DeviceFlags & DEVICE_IS_SUPER_FLOPPY != 0 {
                return b"GenSFloppy\0".as_ptr() as PCSTR;
            } else {
                return AtapGenericDeviceNames[DeviceType as usize].GenericType;
            }
        }
        _ => {
            ASSERT!(FALSE);
            UNREACHABLE!();
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn AtaOpenRegistryKey(
    KeyHandle: PHANDLE,
    RootKey: HANDLE,
    KeyName: PUNICODE_STRING,
    Create: BOOLEAN,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut Disposition: ULONG = core::mem::zeroed();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();

    PAGED_CODE!();

    InitializeObjectAttributes!(
        &mut ObjectAttributes,
        KeyName,
        OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
        RootKey,
        core::ptr::null_mut(),
    );

    if Create != 0 {
        Status = ZwCreateKey(
            KeyHandle,
            KEY_ALL_ACCESS,
            &mut ObjectAttributes,
            0,
            core::ptr::null_mut(),
            REG_OPTION_VOLATILE,
            &mut Disposition,
        );
    } else {
        Status = ZwOpenKey(KeyHandle, KEY_ALL_ACCESS, &mut ObjectAttributes);
    }

    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn AtaGetRegistryKey(
    ChanExt: PATAPORT_CHANNEL_EXTENSION,
    TargetId: UCHAR,
    KeyName: PCWSTR,
    KeyValue: PULONG,
    DefaultValue: ULONG,
) {
    unsafe {
        let mut Buffer: [u8; 20] = [0; 20];
        let mut HwKeyHandle: HANDLE = core::ptr::null_mut();
        let mut TargetKeyHandle: HANDLE = core::ptr::null_mut();
        let mut ValueName: _UNICODE_STRING = core::mem::zeroed();
        let mut TargetKeyName: _UNICODE_STRING = core::mem::zeroed();
        let mut Status: NTSTATUS = 0;
        let mut TargetKeyBuffer: [u16; 9] = [0; 9];
        let mut ResultLength: ULONG = 0;
        let PartialInfo: PKEY_VALUE_PARTIAL_INFORMATION =
            core::ptr::addr_of_mut!(Buffer) as PKEY_VALUE_PARTIAL_INFORMATION;

        PAGED_CODE!();

        Status = IoOpenDeviceRegistryKey(
            (*ChanExt).Pdo,
            PLUGPLAY_REGKEY_DEVICE,
            KEY_ALL_ACCESS,
            core::ptr::addr_of_mut!(HwKeyHandle),
        );
        if !NT_SUCCESS!(Status) {
            return;
        }

        'cleanup: {
            Status = RtlStringCbPrintfW!(
                core::ptr::addr_of_mut!(TargetKeyBuffer) as *mut u16,
                core::mem::size_of_val(&TargetKeyBuffer),
                RUSTCONV_WSTR_447B6E0274D3.as_ptr(),
                TargetId as i32,
            );

            RtlInitUnicodeString(
                core::ptr::addr_of_mut!(TargetKeyName),
                core::ptr::addr_of!(TargetKeyBuffer) as *const u16,
            );

            Status = AtaOpenRegistryKey(
                core::ptr::addr_of_mut!(TargetKeyHandle),
                HwKeyHandle,
                core::ptr::addr_of_mut!(TargetKeyName),
                TRUE,
            );
            if !NT_SUCCESS!(Status) {
                break 'cleanup;
            }

            RtlInitUnicodeString(
                core::ptr::addr_of_mut!(ValueName),
                KeyName,
            );

            Status = ZwQueryValueKey(
                TargetKeyHandle,
                core::ptr::addr_of_mut!(ValueName),
                KeyValuePartialInformation,
                (PartialInfo).cast(),
                core::mem::size_of_val(&Buffer) as u32,
                core::ptr::addr_of_mut!(ResultLength),
            );

            ZwClose(TargetKeyHandle);

            if !NT_SUCCESS!(Status)
                || (*PartialInfo).Type != REG_DWORD
                || (*PartialInfo).DataLength as usize != core::mem::size_of::<u32>()
            {
                *KeyValue = DefaultValue;
                break 'cleanup;
            }

            *KeyValue = *((*PartialInfo).Data.as_mut_ptr() as *mut u32);
        }

        ZwClose(HwKeyHandle);
    }
}
#[no_mangle]
pub unsafe extern "C" fn AtaSetRegistryKey(
    ChanExt: PATAPORT_CHANNEL_EXTENSION,
    TargetId: UCHAR,
    KeyName: PCWSTR,
    mut KeyValue: ULONG) {
    let mut HwKeyHandle: HANDLE = core::ptr::null_mut();
    let mut TargetKeyHandle: HANDLE = core::ptr::null_mut();
    let mut ValueName: _UNICODE_STRING = core::mem::zeroed();
    let mut TargetKeyName: _UNICODE_STRING = core::mem::zeroed();
    let mut Status: NTSTATUS = 0;
    let mut TargetKeyBuffer: [WCHAR; 9] = [0; 9];

    PAGED_CODE!();

    Status = IoOpenDeviceRegistryKey((*ChanExt).Pdo,
                                     PLUGPLAY_REGKEY_DEVICE,
                                     KEY_ALL_ACCESS,
                                     core::ptr::addr_of_mut!(HwKeyHandle));
    if !NT_SUCCESS!(Status) {
        return;
    }

    'cleanup: {
        Status = RtlStringCbPrintfW!(TargetKeyBuffer.as_mut_ptr(),
                                        core::mem::size_of::<[WCHAR; 9]>(),
                                        RUSTCONV_WSTR_447B6E0274D3.as_ptr(),
                                        TargetId as core::ffi::c_int);

        RtlInitUnicodeString(core::ptr::addr_of_mut!(TargetKeyName),
                               TargetKeyBuffer.as_ptr());

        Status = AtaOpenRegistryKey(core::ptr::addr_of_mut!(TargetKeyHandle),
                                        HwKeyHandle,
                                        core::ptr::addr_of_mut!(TargetKeyName),
                                        TRUE);
        if !NT_SUCCESS!(Status) {
            break 'cleanup;
        }

        RtlInitUnicodeString(core::ptr::addr_of_mut!(ValueName), KeyName);

        Status = ZwSetValueKey(TargetKeyHandle,
                                       core::ptr::addr_of_mut!(ValueName),
                                       0,
                                       REG_DWORD,
                                       (core::ptr::addr_of_mut!(KeyValue)).cast(),
                                       core::mem::size_of::<ULONG>() as ULONG);

        ZwClose(TargetKeyHandle);

        if !NT_SUCCESS!(Status) {
        }
    }

    ZwClose(HwKeyHandle);
}
#[no_mangle]
pub unsafe extern "C" fn AtaSetPortRegistryKey(
    ChanExt: *mut ATAPORT_CHANNEL_EXTENSION,
    KeyName: PCWSTR,
    mut KeyValue: u32,
) {
    let mut HwKeyHandle: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut ValueName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut Status: i32 = 0;

    PAGED_CODE!();

    Status = IoOpenDeviceRegistryKey(
        (*ChanExt).Pdo,
        PLUGPLAY_REGKEY_DEVICE,
        KEY_ALL_ACCESS,
        &mut HwKeyHandle,
    );
    if !NT_SUCCESS!(Status) {
        return;
    }

    RtlInitUnicodeString(&mut ValueName, KeyName);
    Status = ZwSetValueKey(
        HwKeyHandle,
        &mut ValueName,
        0,
        REG_DWORD,
        &mut KeyValue as *mut u32 as *mut core::ffi::c_void,
        core::mem::size_of::<u32>() as u32,
    );
    ZwClose(HwKeyHandle);
    if !NT_SUCCESS!(Status) {
    }
}

#[no_mangle]
pub unsafe extern "system" fn AtaDispatchCreateClose(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    PAGED_CODE!();

    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    IoCompleteRequest(Irp, (IO_NO_INCREMENT) as CCHAR);

    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn AtaUnload(DriverObject: PDRIVER_OBJECT) {
    PAGED_CODE!();
    RtlFreeUnicodeString(&raw mut AtapDriverRegistryPath);
}

#[no_mangle]
pub unsafe extern "system" fn AtaStorageNotificationWorker(
    DeviceObject: PDEVICE_OBJECT,
    Context: PVOID,
) {
    unsafe {
        let mut WorkItem: PIO_WORKITEM = Context as PIO_WORKITEM;
        let mut TopDeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
        let mut Irp: PIRP = core::ptr::null_mut();
        let mut Event: KEVENT = core::mem::zeroed();
        let mut IoStatus: _IO_STATUS_BLOCK = core::mem::zeroed();
        let mut EventObject: STORAGE_EVENT_NOTIFICATION = core::mem::zeroed();
        let mut Status: NTSTATUS = 0;

        TopDeviceObject = IoGetAttachedDeviceReference(DeviceObject);

        KeInitializeEvent(
            core::ptr::addr_of_mut!(Event),
            _EVENT_TYPE_NotificationEvent,
            FALSE,
        );

        EventObject.Version = STORAGE_EVENT_NOTIFICATION_VERSION_V1;
        EventObject.Size = core::mem::size_of::<STORAGE_EVENT_NOTIFICATION>() as u32;
        EventObject.Events = (STORAGE_EVENT_ALL) as _;

        Irp = IoBuildDeviceIoControlRequest(
            IOCTL_STORAGE_EVENT_NOTIFICATION,
            TopDeviceObject,
            core::ptr::addr_of_mut!(EventObject) as PVOID,
            core::mem::size_of::<STORAGE_EVENT_NOTIFICATION>() as u32,
            core::ptr::null_mut(),
            0,
            FALSE,
            core::ptr::addr_of_mut!(Event),
            core::ptr::addr_of_mut!(IoStatus),
        );

        if Irp.is_null() {
            if DbgPrint(
                b"(%s:%d) %s IoBuildDeviceIoControlRequest() failed\n\0".as_ptr() as PCSTR,
                b"drivers/storage/ide/atapi/atapi.c\0".as_ptr() as PCSTR,
                330 as i32,
                b"AtaStorageNotificationWorker\0".as_ptr() as PCSTR,
            ) != 0
            {
                DbgPrint(
                    b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
                    b"drivers/storage/ide/atapi/atapi.c\0".as_ptr() as PCSTR,
                    330 as i32,
                );
            }
        } else {
            Status = IoCallDriver(DeviceObject, Irp);
            if Status == STATUS_PENDING {
                KeWaitForSingleObject(
                    core::ptr::addr_of_mut!(Event) as PVOID,
                    _KWAIT_REASON_Suspended,
                    (0i32) as KPROCESSOR_MODE,
                    FALSE,
                    core::ptr::null_mut(),
                );
                Status = IoStatus.__bindgen_anon_1.Status;
            }
        }

        ObDereferenceObject((TopDeviceObject).cast());
        IoFreeWorkItem(WorkItem);
    }
}

#[no_mangle]
pub unsafe extern "system" fn AtaStorageNotificationlDpc(
    Dpc: PKDPC,
    DeferredContext: *mut core::ffi::c_void,
    SystemArgument1: *mut core::ffi::c_void,
    SystemArgument2: *mut core::ffi::c_void,
) {
    let PortData: *mut ATAPORT_PORT_DATA = DeferredContext as *mut ATAPORT_PORT_DATA;
    let DeviceBitmap: u32 = PtrToUlong!(SystemArgument1);
    let mut ChanExt: *mut ATAPORT_CHANNEL_EXTENSION;
    let mut i: u32;

    UNREFERENCED_PARAMETER!(Dpc);
    UNREFERENCED_PARAMETER!(SystemArgument2);

    ChanExt = CONTAINING_RECORD!(PortData, ATAPORT_CHANNEL_EXTENSION, PortData);

    i = 0;
    while i < ATA_MAX_DEVICE {
        let mut DevExt: *mut ATAPORT_DEVICE_EXTENSION;
        let mut WorkItem: PIO_WORKITEM;

        if (DeviceBitmap & (1 << i)) == 0 {
            i = i.wrapping_add(1);
            continue;
        }

        DevExt = (AtaFdoFindDeviceByPath(
            (ChanExt).cast(),
            AtaMarshallScsiAddress((*PortData).PortNumber, i, 0),
            (AtaStorageNotificationlDpc as *mut core::ffi::c_void),
        )).cast();
        if DevExt.is_null() {
            i = i.wrapping_add(1);
            continue;
        }

        WorkItem = IoAllocateWorkItem((*DevExt).Common.Self_);
        if WorkItem.is_null() {
        } else {
            IoQueueWorkItem(
                WorkItem,
                Some(AtaStorageNotificationWorker),
                CriticalWorkQueue,
                (WorkItem).cast(),
            );
        }

        IoReleaseRemoveLock(&mut (*DevExt).Common.RemoveLock, (AtaStorageNotificationlDpc as *mut core::ffi::c_void));

        i = i.wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn AtaPnpQueryInterface(
    CommonExt: PATAPORT_COMMON_EXTENSION,
    Guid: *const GUID,
    Interface: *mut core::ffi::c_void,
    Version: u32,
    Size: u32,
) -> i32 {
    let mut Event: KEVENT = core::mem::zeroed();
    let mut Irp: PIRP = core::ptr::null_mut();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut IoStack: PIO_STACK_LOCATION = core::ptr::null_mut();
    let mut Status: i32 = 0;

    PAGED_CODE!();

    KeInitializeEvent(
        (&mut Event as *mut KEVENT as *mut core::ffi::c_void).cast(),
        _EVENT_TYPE_NotificationEvent,
        FALSE,
    );

    Irp = IoBuildSynchronousFsdRequest(
        IRP_MJ_PNP,
        (*CommonExt).__bindgen_anon_1.LowerDeviceObject,
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
        (&mut Event as *mut KEVENT as *mut core::ffi::c_void).cast(),
        &mut IoStatusBlock,
    );
    if Irp.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_NOT_SUPPORTED;

    IoStack = IoGetNextIrpStackLocation(Irp);
    (*IoStack).MinorFunction = IRP_MN_QUERY_INTERFACE;
    (*IoStack).Parameters.QueryInterface.InterfaceType = Guid;
    (*IoStack).Parameters.QueryInterface.Size = Size as u16;
    (*IoStack).Parameters.QueryInterface.Version = Version as u16;
    (*IoStack).Parameters.QueryInterface.Interface = (Interface).cast();
    (*IoStack).Parameters.QueryInterface.InterfaceSpecificData = core::ptr::null_mut();

    Status = IoCallDriver((*CommonExt).__bindgen_anon_1.LowerDeviceObject, Irp);
    if Status == STATUS_PENDING {
        KeWaitForSingleObject(
            &mut Event as *mut KEVENT as *mut core::ffi::c_void,
            _KWAIT_REASON_Executive,
            (0i32) as KPROCESSOR_MODE,
            FALSE,
            core::ptr::null_mut(),
        );
        Status = IoStatusBlock.__bindgen_anon_1.Status;
    }

    Status
}
#[no_mangle]
pub unsafe extern "C" fn AtaPnpRepeatRequest(
    CommonExt: *mut ATAPORT_COMMON_EXTENSION,
    Irp: PIRP,
    DeviceCapabilities: PDEVICE_CAPABILITIES,
) -> NTSTATUS {
    let FdoExt = (*CommonExt).__bindgen_anon_1.FdoExt;
    let mut TopDeviceObject: PDEVICE_OBJECT;
    let mut IoStack: PIO_STACK_LOCATION;
    let mut SubStack: PIO_STACK_LOCATION;
    let mut SubIrp: PIRP;
    let mut Event: KEVENT = core::mem::zeroed();
    let mut Status: NTSTATUS;

    PAGED_CODE!();

    TopDeviceObject = IoGetAttachedDeviceReference((*FdoExt).Self_);

    SubIrp = IoAllocateIrp((*TopDeviceObject).StackSize, FALSE);
    if SubIrp.is_null() {
        ObDereferenceObject(TopDeviceObject as *mut core::ffi::c_void);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    KeInitializeEvent(&mut Event, _EVENT_TYPE_NotificationEvent, FALSE);

    IoStack = IoGetCurrentIrpStackLocation(Irp);
    SubStack = IoGetNextIrpStackLocation(SubIrp);
    RtlCopyMemory!(
        SubStack as *mut core::ffi::c_void,
        IoStack as *const core::ffi::c_void,
        core::mem::size_of::<IO_STACK_LOCATION>(),
    );

    if !DeviceCapabilities.is_null() {
        (*SubStack).Parameters.DeviceCapabilities.Capabilities = DeviceCapabilities;
    }

    IoSetCompletionRoutine(
        SubIrp,
        Some(AtaPdoCompletionRoutine),
        &mut Event as *mut KEVENT as *mut core::ffi::c_void,
        TRUE,
        TRUE,
        TRUE,
    );

    (*SubIrp).IoStatus.__bindgen_anon_1.Status = STATUS_NOT_SUPPORTED;

    Status = IoCallDriver(TopDeviceObject, SubIrp);
    if Status == STATUS_PENDING {
        KeWaitForSingleObject(
            &mut Event as *mut KEVENT as *mut core::ffi::c_void,
            _KWAIT_REASON_Executive,
            (0i32) as KPROCESSOR_MODE,
            FALSE,
            core::ptr::null_mut(),
        );
    }

    ObDereferenceObject(TopDeviceObject as *mut core::ffi::c_void);

    Status = (*SubIrp).IoStatus.__bindgen_anon_1.Status;
    IoFreeIrp(SubIrp);

    Status
}
#[no_mangle]
pub unsafe extern "C" fn AtaPnpQueryDeviceUsageNotification(
    CommonExt: *mut ATAPORT_COMMON_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    let mut IoStack: PIO_STACK_LOCATION;
    let mut Status: NTSTATUS;
    let mut Counter: *mut core::ffi::c_long;

    PAGED_CODE!();

    if !IS_FDO!(CommonExt) {
        Status = AtaPnpRepeatRequest(CommonExt, Irp, core::ptr::null_mut());
    } else {
        if ((NT_VERIFY!(IoForwardIrpSynchronously((*CommonExt).__bindgen_anon_1.LowerDeviceObject, Irp))) == 0) {
            return STATUS_UNSUCCESSFUL;
        }
        Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
    }
    if !NT_SUCCESS!(Status) {
        return Status;
    }

    IoStack = IoGetCurrentIrpStackLocation(Irp);
    match (*IoStack).Parameters.UsageNotification.Type {
        DeviceUsageTypePaging => {
            Counter = core::ptr::addr_of_mut!((*CommonExt).PageFiles);
        }
        DeviceUsageTypeHibernation => {
            Counter = core::ptr::addr_of_mut!((*CommonExt).HibernateFiles);
        }
        DeviceUsageTypeDumpFile => {
            Counter = core::ptr::addr_of_mut!((*CommonExt).DumpFiles);
        }
        _ => {
            return Status;
        }
    }

    IoAdjustPagingPathCount(Counter, (*IoStack).Parameters.UsageNotification.InPath);

    if !IS_FDO!(CommonExt) {
        IoInvalidateDeviceState((*CommonExt).Self_);
    }

    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn AtaPnpQueryPnpDeviceState(
    CommonExt: *mut ATAPORT_COMMON_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    if (*CommonExt).PageFiles != 0 || (*CommonExt).HibernateFiles != 0 || (*CommonExt).DumpFiles != 0 {
        (*Irp).IoStatus.Information |= (PNP_DEVICE_NOT_DISABLEABLE) as ULONG_PTR;
    }
    if IS_FDO!(CommonExt) {
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    }
    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn AtaPnpInitializeCommonExtension(
    CommonExt: *mut ATAPORT_COMMON_EXTENSION,
    SelfDeviceObject: PDEVICE_OBJECT,
    Flags: u32,
) {
    (*CommonExt).Flags = Flags;
    (*CommonExt).Self_ = SelfDeviceObject;
    (*CommonExt).DevicePowerState = (PowerDeviceD0) as _;
    (*CommonExt).SystemPowerState = PowerSystemWorking;

    IoInitializeRemoveLock(
        &raw mut (*CommonExt).RemoveLock,
        ATAPORT_TAG,
        0,
        0,
    );
}

#[no_mangle]
pub unsafe extern "system" fn AtaAddChannel(
    DriverObject: PDRIVER_OBJECT,
    PhysicalDeviceObject: PDEVICE_OBJECT,
) -> NTSTATUS {
    let mut Status: NTSTATUS = 0;
    let mut Fdo: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut DeviceName: _UNICODE_STRING = core::mem::zeroed();
    let mut ChanExt: *mut ATAPORT_CHANNEL_EXTENSION = core::ptr::null_mut();
    let mut DeviceNameBuffer: [u16; 25] = [0; 25];
    DECLARE_PAGED_WSTRING!(FdoFormat, RUSTCONV_WSTR_01D8877A5F07.as_ptr());
    static mut AtapFdoNumber: u32 = 0;

    PAGED_CODE!();

    Status = RtlStringCbPrintfW!(
        DeviceNameBuffer.as_mut_ptr(),
        core::mem::size_of::<[u16; 25]>(),
        FdoFormat,
        AtapFdoNumber,
    );

    RtlInitUnicodeString(
        core::ptr::addr_of_mut!(DeviceName),
        DeviceNameBuffer.as_ptr(),
    );

    INFO!(
        b"%s(%p, %p) '%wZ'\n\0".as_ptr() as PCSTR,
        __FUNCTION__,
        DriverObject as *mut core::ffi::c_void,
        PhysicalDeviceObject as *mut core::ffi::c_void,
        core::ptr::addr_of!(DeviceName),
    );

    Status = IoCreateDevice(
        DriverObject,
        (core::mem::size_of::<ATAPORT_CHANNEL_EXTENSION>()) as ULONG,
        core::ptr::addr_of_mut!(DeviceName),
        FILE_DEVICE_CONTROLLER,
        FILE_DEVICE_SECURE_OPEN,
        FALSE,
        core::ptr::addr_of_mut!(Fdo),
    );
    if !NT_SUCCESS!(Status) {
        return Status;
    }

    ChanExt = (*Fdo).DeviceExtension as *mut ATAPORT_CHANNEL_EXTENSION;

    RtlZeroMemory!(ChanExt as *mut core::ffi::c_void, core::mem::size_of::<ATAPORT_CHANNEL_EXTENSION>());
    AtaPnpInitializeCommonExtension(
        core::ptr::addr_of_mut!((*ChanExt).Common),
        Fdo,
        DO_IS_FDO,
    );

    (*ChanExt).Common.__bindgen_anon_1.LowerDeviceObject = IoAttachDeviceToDeviceStack(Fdo, PhysicalDeviceObject);
    if (*ChanExt).Common.__bindgen_anon_1.LowerDeviceObject.is_null() {
        Status = STATUS_DEVICE_REMOVED;
        if !(*ChanExt).Common.__bindgen_anon_1.LowerDeviceObject.is_null() {
            IoDetachDevice((*ChanExt).Common.__bindgen_anon_1.LowerDeviceObject);
        }
        IoDeleteDevice(Fdo);
        return Status;
    }

    let previous = AtapFdoNumber;
    AtapFdoNumber = AtapFdoNumber.wrapping_add(1);
    (*ChanExt).DeviceObjectNumber = previous;
    (*ChanExt).Pdo = PhysicalDeviceObject;

    (*Fdo).AlignmentRequirement = (*((*ChanExt).Common.__bindgen_anon_1.LowerDeviceObject)).AlignmentRequirement;
    (*Fdo).AlignmentRequirement = max((*Fdo).AlignmentRequirement, ATA_MIN_BUFFER_ALIGNMENT);

    KeInitializeSpinLock(core::ptr::addr_of_mut!((*ChanExt).PdoListLock));

    (*Fdo).Flags &= !DO_DEVICE_INITIALIZING;

    return STATUS_SUCCESS;
}

unsafe extern "C" fn AtaInPEMode() -> i32 {
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = _OBJECT_ATTRIBUTES {
        Length: 0,
        RootDirectory: core::ptr::null_mut(),
        ObjectName: core::ptr::null_mut(),
        Attributes: 0,
        SecurityDescriptor: core::ptr::null_mut(),
        SecurityQualityOfService: core::ptr::null_mut(),
    };
    let mut KeyHandle: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut Status: i32 = 0;
    static mut KeyName_buffer: [u16; 58] = [0; 58];
    let mut KeyName: _UNICODE_STRING = _UNICODE_STRING {
        Length: (core::mem::size_of::<[u16; 58]>() - core::mem::size_of::<u16>()) as u16,
        MaximumLength: core::mem::size_of::<[u16; 58]>() as u16,
        Buffer: RUSTCONV_WSTR_124A192A3DF6.as_ptr() as *mut u16,
    };
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.ObjectName = &mut KeyName;
    ObjectAttributes.Attributes = (OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE) as u32;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();
    Status = ZwOpenKey(&mut KeyHandle, KEY_READ, &mut ObjectAttributes);
    if Status >= 0 {
        ZwClose(KeyHandle);
        return (TRUE) as _;
    }
    return (FALSE) as _;
}

// HAND-PORTED (atapi.c AtaCreateIdeDirectory): the banked body called the declaring macro
// as an expression and `port fix` could not repair it in two rounds. Re-applied by
// `rustconv port` after every reassembly (port.apply_hand_bodies).
unsafe extern "C" fn AtaCreateIdeDirectory() {
    let mut Handle: HANDLE = core::ptr::null_mut();
    let mut ObjectAttributes: OBJECT_ATTRIBUTES = core::mem::zeroed();
    DECLARE_PAGED_UNICODE_STRING!(DirectoryName, RUSTCONV_WSTR_604F52A85CB9);

    PAGED_CODE!();

    InitializeObjectAttributes!(
        &mut ObjectAttributes,
        &DirectoryName as *const UNICODE_STRING as PUNICODE_STRING,
        OBJ_CASE_INSENSITIVE | OBJ_PERMANENT | OBJ_KERNEL_HANDLE,
        core::ptr::null_mut(),
        core::ptr::null_mut()
    );

    let Status = ZwCreateDirectoryObject(&mut Handle, DIRECTORY_ALL_ACCESS, &mut ObjectAttributes);
    if NT_SUCCESS!(Status) {
        /* We don't need a handle for a permanent object */
        ZwClose(Handle);
    }
    // Directory creation failures are ignored: another driver may already have created it,
    // and a fatal error surfaces later from IoCreateDevice().
}

#[no_mangle]
pub unsafe extern "system" fn DriverEntry(
    DriverObject: PDRIVER_OBJECT,
    RegistryPath: PUNICODE_STRING,
) -> i32 {
    INFO!(b"ATAPI driver entry\n\0".as_ptr() as *const i8);

    if DriverObject.is_null() {
        return STATUS_NOT_IMPLEMENTED;
    }

    AtapDriverRegistryPath.MaximumLength = ((*RegistryPath).Length as usize)
        .wrapping_add(core::mem::size_of_val(&UNICODE_NULL)) as u16;
    AtapDriverRegistryPath.Buffer = ExAllocatePoolUninitialized(
        NonPagedPool,
        (AtapDriverRegistryPath.MaximumLength as usize) as _,
        ATAPORT_TAG,
    ) as *mut u16;
    if AtapDriverRegistryPath.Buffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    RtlCopyUnicodeString(
        core::ptr::addr_of_mut!(AtapDriverRegistryPath),
        RegistryPath,
    );
    *AtapDriverRegistryPath.Buffer.offset(
        ((*RegistryPath).Length as usize / core::mem::size_of::<u16>()) as isize,
    ) = UNICODE_NULL;

    (*DriverObject).MajorFunction[IRP_MJ_CREATE as usize] =
        Some(AtaDispatchCreateClose as unsafe extern "system" fn(PDEVICE_OBJECT, PIRP) -> i32);
    (*DriverObject).MajorFunction[IRP_MJ_CLOSE as usize] =
        Some(AtaDispatchCreateClose as unsafe extern "system" fn(PDEVICE_OBJECT, PIRP) -> i32);
    (*DriverObject).MajorFunction[IRP_MJ_DEVICE_CONTROL as usize] =
        Some(AtaDispatchDeviceControl as unsafe extern "system" fn(PDEVICE_OBJECT, PIRP) -> i32);
    (*DriverObject).MajorFunction[IRP_MJ_SCSI as usize] =
        Some(AtaDispatchScsi as unsafe extern "system" fn(PDEVICE_OBJECT, PIRP) -> i32);
    (*DriverObject).MajorFunction[IRP_MJ_POWER as usize] =
        Some(AtaDispatchPower as unsafe extern "system" fn(PDEVICE_OBJECT, PIRP) -> i32);
    (*DriverObject).MajorFunction[IRP_MJ_SYSTEM_CONTROL as usize] =
        Some(AtaDispatchWmi as unsafe extern "system" fn(PDEVICE_OBJECT, PIRP) -> i32);
    (*DriverObject).MajorFunction[IRP_MJ_PNP as usize] =
        Some(AtaDispatchPnp as unsafe extern "system" fn(PDEVICE_OBJECT, PIRP) -> i32);
    (*(*DriverObject).DriverExtension).AddDevice =
        Some(AtaAddChannel as unsafe extern "system" fn(PDRIVER_OBJECT, PDEVICE_OBJECT) -> i32);
    (*DriverObject).DriverUnload =
        Some(AtaUnload as unsafe extern "system" fn(PDRIVER_OBJECT));

    KeInitializeDpc(
        core::ptr::addr_of_mut!(AtapCompletionDpc),
        Some(AtaReqCompletionDpc as unsafe extern "system" fn(PKDPC, *mut core::ffi::c_void, *mut core::ffi::c_void, *mut core::ffi::c_void)),
        core::ptr::null_mut(),
    );
    InitializeSListHead(core::ptr::addr_of_mut!(AtapCompletionQueueList));

    AtaCreateIdeDirectory();

    AtapInPEMode = (AtaInPEMode()) as u8;

    return STATUS_SUCCESS;
}
