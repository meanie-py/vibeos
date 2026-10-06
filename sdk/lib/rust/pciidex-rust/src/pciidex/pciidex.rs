// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
pub static RUSTCONV_WSTR_604F52A85CB9: [u16; 12] = [
    0x005Cu16, 0x0044, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065, 0x005C, 0x0049, 0x0064,
    0x0065, 0x0000,
];
pub static RUSTCONV_WSTR_6D9E94643474: [u16; 22] = [
    0x005Cu16, 0x0044, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065, 0x005C, 0x0049, 0x0064,
    0x0065, 0x005C, 0x0050, 0x0063, 0x0069, 0x0049, 0x0064, 0x0065, 0x0025, 0x006C,
    0x0075, 0x0000,
];

use crate::prelude::*;
use ntkrnl_sys::ddk::{BUS_INTERFACE_STANDARD, BusNumber, DeviceNumber, ExInitializeFastMutex, _KWAIT_REASON_Executive, FunctionNumber, GUID, HalGetBusDataByOffset, _IO_STATUS_BLOCK, IoAllocateDriverObjectExtension, IoAllocateIrp, IoAttachDeviceToDeviceStackSafe, IoBuildSynchronousFsdRequest, IoCreateDevice, IoDeleteDevice, IoDetachDevice, IoForwardIrpSynchronously, IoFreeIrp, IoGetAttachedDeviceReference, IoGetDriverObjectExtension, IoInvalidateDeviceState, KEVENT, KTIMER, KeInitializeEvent, KeInitializeSpinLock, KeInitializeTimer, KeSetTimer, KeWaitForSingleObject, LARGE_INTEGER, _EVENT_TYPE_NotificationEvent, _OBJECT_ATTRIBUTES, PCI_COMMON_HEADER, PCI_SLOT_NUMBER, PDEVICE_CAPABILITIES, PDEVICE_OBJECT, PDRIVER_OBJECT, PIO_STACK_LOCATION, PIRP, PPCI_COMMON_HEADER, PUNICODE_STRING, RtlInitUnicodeString, _EVENT_TYPE_SynchronizationEvent, ZwClose, ZwCreateDirectoryObject};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static mut GUID_BUS_INTERFACE_STANDARD: GUID = _GUID {
    Data1: 0x496b8280 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x6f25 as ::core::ffi::c_ushort,
    Data3: 0x11d0 as ::core::ffi::c_ushort,
    Data4: [
        0xbe as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xaf as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2f as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut PciIdeControllerNumber: ULONG = 0 as ULONG;
#[no_mangle]
pub unsafe extern "C" fn PciFindDevice(
    MatchFunction: PATA_PCI_MATCH_FN,
    Context: PVOID,
) -> BOOLEAN {
    let mut BusNumber: ULONG;
    let mut DeviceNumber: ULONG;
    let mut FunctionNumber: ULONG;

    BusNumber = 0;
    while BusNumber < 0xFF {
        DeviceNumber = 0;
        while DeviceNumber < PCI_MAX_DEVICES {
            FunctionNumber = 0;
            while FunctionNumber < PCI_MAX_FUNCTION {
                let mut Buffer: [UCHAR; RTL_SIZEOF_THROUGH_FIELD!(PCI_COMMON_HEADER, RevisionID)] =
                    [0; RTL_SIZEOF_THROUGH_FIELD!(PCI_COMMON_HEADER, RevisionID)];
                let PciConfig: PPCI_COMMON_HEADER = core::ptr::addr_of_mut!(Buffer) as *mut UCHAR as PPCI_COMMON_HEADER;
                let mut PciSlot: PCI_SLOT_NUMBER = core::mem::zeroed();
                let mut BytesRead: ULONG;

                PciSlot.u.AsULONG = 0;
                PciSlot.u.bits.DeviceNumber = DeviceNumber as _;
                PciSlot.u.bits.FunctionNumber = FunctionNumber as _;

                BytesRead = HalGetBusDataByOffset(
                    PCIConfiguration,
                    BusNumber,
                    PciSlot.u.AsULONG,
                    core::ptr::addr_of_mut!(Buffer) as PVOID,
                    0,
                    core::mem::size_of_val(&Buffer) as ULONG,
                );
                if BytesRead != core::mem::size_of_val(&Buffer) as ULONG
                    || ((*PciConfig).VendorID) as u32 == PCI_INVALID_VENDORID
                    || (*PciConfig).VendorID == 0
                {
                    if FunctionNumber == 0 {
                        break;
                    } else {
                        FunctionNumber = FunctionNumber.wrapping_add(1);
                        continue;
                    }
                }

                if (MatchFunction.unwrap_unchecked())(Context, BusNumber, PciSlot, PciConfig) != 0 {
                    return TRUE;
                }

                if !PCI_MULTIFUNCTION_DEVICE(PciConfig) {
                    break;
                }

                FunctionNumber = FunctionNumber.wrapping_add(1);
            }
            DeviceNumber = DeviceNumber.wrapping_add(1);
        }
        BusNumber = BusNumber.wrapping_add(1);
    }

    return FALSE;
}
#[no_mangle]
pub unsafe extern "C" fn PciRead(
    Controller: *mut PATA_CONTROLLER,
    Buffer: *mut core::ffi::c_void,
    ConfigDataOffset: u32,
    BufferLength: u32,
) {
    (((*(*Controller)).GetBusData).unwrap_unchecked())(
        (*(*Controller)).BusInterfaceContext,
        PCI_WHICHSPACE_CONFIG,
        Buffer,
        ConfigDataOffset,
        BufferLength,
    );
}
#[no_mangle]
pub unsafe extern "C" fn PciWrite(
    Controller: *mut PATA_CONTROLLER,
    Buffer: *mut core::ffi::c_void,
    ConfigDataOffset: u32,
    BufferLength: u32,
) {
    (((*(*Controller)).SetBusData).unwrap_unchecked())(
        (*(*Controller)).BusInterfaceContext,
        PCI_WHICHSPACE_CONFIG,
        Buffer,
        ConfigDataOffset,
        BufferLength,
    );
}
#[no_mangle]
pub unsafe extern "C" fn AtaSleep() {
    let mut Timer: KTIMER = core::mem::zeroed();
    let mut DueTime: LARGE_INTEGER = core::mem::zeroed();

    DueTime.QuadPart = ((PORT_TIMER_TICK_MS as i64) * -10000i64) as _;

    KeInitializeTimer(&mut Timer);
    KeSetTimer(&mut Timer, DueTime, core::ptr::null_mut());
    KeWaitForSingleObject(
        &mut Timer as *mut KTIMER as *mut core::ffi::c_void,
        _KWAIT_REASON_Executive,
        (0i32) as i8,
        FALSE,
        core::ptr::null_mut(),
    );
}

#[no_mangle]
pub unsafe extern "system" fn PciIdeXDispatchWmi(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let mut Status: NTSTATUS;

    if IS_FDO!((*DeviceObject).DeviceExtension) {
        let FdoExtension: PFDO_DEVICE_EXTENSION = (*DeviceObject).DeviceExtension as PFDO_DEVICE_EXTENSION;

        IoSkipCurrentIrpStackLocation(Irp);
        Status = IoCallDriver((*FdoExtension).Common.__bindgen_anon_1.LowerDeviceObject, Irp);
    } else {
        Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
        IoCompleteRequest(Irp, (IO_NO_INCREMENT) as CCHAR);
    }

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn PciIdeXUnload(DriverObject: PDRIVER_OBJECT) {
    if (KeGetCurrentIrql() as i32) as u32 > DISPATCH_LEVEL {
        // PAGED_CODE assertion
    }
    NOTHING;
}
#[no_mangle]
pub unsafe extern "C" fn PciIdeXPnpRepeatRequest(
    CommonExt: *mut COMMON_DEVICE_EXTENSION,
    Irp: PIRP,
    DeviceCapabilities: PDEVICE_CAPABILITIES,
) -> NTSTATUS {
    let FdoExt: *mut COMMON_DEVICE_EXTENSION = ((*CommonExt).__bindgen_anon_1.FdoExt).cast();
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

    KeInitializeEvent(
        core::ptr::addr_of_mut!(Event),
        _EVENT_TYPE_NotificationEvent,
        FALSE,
    );

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
        Some(PciIdeXPdoCompletionRoutine),
        core::ptr::addr_of_mut!(Event) as *mut core::ffi::c_void,
        TRUE,
        TRUE,
        TRUE,
    );

    (*SubIrp).IoStatus.__bindgen_anon_1.Status = STATUS_NOT_SUPPORTED;

    Status = IoCallDriver(TopDeviceObject, SubIrp);
    if Status == STATUS_PENDING {
        KeWaitForSingleObject(
            core::ptr::addr_of_mut!(Event) as *mut core::ffi::c_void,
            _KWAIT_REASON_Executive,
            (0i32) as i8,
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
pub unsafe extern "C" fn PciIdeXPnpQueryDeviceUsageNotification(
    CommonExt: PCOMMON_DEVICE_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    let mut IoStack: PIO_STACK_LOCATION;
    let mut Status: NTSTATUS;
    let mut Counter: *mut i32;

    if !IS_FDO!(CommonExt) {
        Status = PciIdeXPnpRepeatRequest(CommonExt, Irp, core::ptr::null_mut());
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
            Counter = &mut (*CommonExt).PageFiles;
        }
        DeviceUsageTypeHibernation => {
            Counter = &mut (*CommonExt).HibernateFiles;
        }
        DeviceUsageTypeDumpFile => {
            Counter = &mut (*CommonExt).DumpFiles;
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
pub unsafe extern "C" fn PciIdeXPnpQueryPnpDeviceState(
    CommonExt: PCOMMON_DEVICE_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    if (*CommonExt).PageFiles != 0
        || (*CommonExt).HibernateFiles != 0
        || (*CommonExt).DumpFiles != 0
    {
        (*Irp).IoStatus.Information |= PNP_DEVICE_NOT_DISABLEABLE as ULONG_PTR;
    }
    if IS_FDO!(CommonExt) != false {
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    }
    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn PciIdeXPdoCompletionRoutine(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Context: PVOID,
) -> NTSTATUS {
    UNREFERENCED_PARAMETER!(DeviceObject);

    if (*Irp).PendingReturned != 0 {
        KeSetEvent((Context).cast(), (IO_NO_INCREMENT) as i32, FALSE);
    }

    return STATUS_MORE_PROCESSING_REQUIRED;
}

unsafe extern "C" fn PciIdeXPnpQueryInterface(
    CommonExt: PCOMMON_DEVICE_EXTENSION,
    Guid: *const GUID,
    Interface: PVOID,
    Version: ULONG,
    Size: ULONG,
) -> NTSTATUS {
    let mut Event: KEVENT = core::mem::zeroed();
    let mut IoStatus: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Irp: PIRP;
    let mut Stack: PIO_STACK_LOCATION;
    let mut Status: NTSTATUS;

    PAGED_CODE!();

    KeInitializeEvent(&mut Event, _EVENT_TYPE_SynchronizationEvent, FALSE);

    Irp = IoBuildSynchronousFsdRequest(
        IRP_MJ_PNP,
        (*CommonExt).__bindgen_anon_1.LowerDeviceObject,
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
        &mut Event,
        &mut IoStatus,
    );
    if Irp.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_NOT_SUPPORTED;
    (*Irp).IoStatus.Information = 0;

    Stack = IoGetNextIrpStackLocation(Irp);
    (*Stack).MinorFunction = IRP_MN_QUERY_INTERFACE;
    (*Stack).Parameters.QueryInterface.InterfaceType = Guid;
    (*Stack).Parameters.QueryInterface.Version = Version as USHORT;
    (*Stack).Parameters.QueryInterface.Size = Size as USHORT;
    (*Stack).Parameters.QueryInterface.Interface = Interface as PINTERFACE;
    (*Stack).Parameters.QueryInterface.InterfaceSpecificData = core::ptr::null_mut();

    Status = IoCallDriver((*CommonExt).__bindgen_anon_1.LowerDeviceObject, Irp);
    if Status == STATUS_PENDING {
        KeWaitForSingleObject(
            &mut Event as *mut KEVENT as *mut core::ffi::c_void,
            _KWAIT_REASON_Executive,
            (0i32) as i8,
            FALSE,
            core::ptr::null_mut(),
        );
        Status = IoStatus.__bindgen_anon_1.Status;
    }

    return Status;
}

unsafe extern "C" fn PciIdeXAddDeviceEx(
    DriverObject: PDRIVER_OBJECT,
    PhysicalDeviceObject: PDEVICE_OBJECT,
    ControllerContext: *mut PVOID,
) -> NTSTATUS {
    let mut DriverExtension: PPCIIDEX_DRIVER_EXTENSION = core::ptr::null_mut();
    let mut FdoExtension: PFDO_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut DeviceExtensionSize: ULONG = 0;
    let mut Fdo: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut DeviceName: _UNICODE_STRING = core::mem::zeroed();
    let mut DeviceNameBuffer: [u16; 24] = [0; 24];
    let mut Status: NTSTATUS = 0;

    PAGED_CODE!();

    Status = RtlStringCbPrintfW!(
        core::ptr::addr_of_mut!(DeviceNameBuffer) as *mut u16,
        core::mem::size_of::<[u16; 24]>() as usize,
        RUSTCONV_WSTR_6D9E94643474.as_ptr(),
        PciIdeControllerNumber,
    );

    RtlInitUnicodeString(
        core::ptr::addr_of_mut!(DeviceName) as *mut _UNICODE_STRING,
        core::ptr::addr_of_mut!(DeviceNameBuffer) as *const u16,
    );

    INFO!(
        b"%s(%p, %p, %p) '%wZ'\n\0".as_ptr() as *const i8,
        __FUNCTION__,
        DriverObject as *mut core::ffi::c_void,
        PhysicalDeviceObject as *mut core::ffi::c_void,
        ControllerContext as *mut core::ffi::c_void,
        core::ptr::addr_of!(DeviceName) as *mut core::ffi::c_void,
    );

    DriverExtension = IoGetDriverObjectExtension(DriverObject, DriverObject as *mut core::ffi::c_void)
        as PPCIIDEX_DRIVER_EXTENSION;

    Status = RtlULongAdd(
        (*DriverExtension).MiniControllerExtensionSize,
        core::mem::size_of::<_FDO_DEVICE_EXTENSION>() as ULONG,
        core::ptr::addr_of_mut!(DeviceExtensionSize),
    );
    if !NT_SUCCESS!(Status) {
        return Status;
    }

    Status = IoCreateDevice(
        DriverObject,
        DeviceExtensionSize,
        core::ptr::addr_of_mut!(DeviceName) as *mut _UNICODE_STRING,
        FILE_DEVICE_BUS_EXTENDER,
        FILE_DEVICE_SECURE_OPEN,
        FALSE,
        core::ptr::addr_of_mut!(Fdo),
    );
    if !NT_SUCCESS!(Status) {
        return Status;
    }

    FdoExtension = (*Fdo).DeviceExtension as PFDO_DEVICE_EXTENSION;

    RtlZeroMemory!(
        FdoExtension as *mut core::ffi::c_void,
        core::mem::size_of::<_FDO_DEVICE_EXTENSION>() as usize,
    );
    (*FdoExtension).Common.Flags = DO_IS_FDO;
    (*FdoExtension).Common.Self_ = Fdo;
    IoInitializeRemoveLock(
        core::ptr::addr_of_mut!((*FdoExtension).Common.RemoveLock),
        TAG_PCIIDEX,
        0,
        0,
    );

    (*FdoExtension).DriverObject = DriverObject;
    (*FdoExtension).Pdo = PhysicalDeviceObject;
    let fresh = PciIdeControllerNumber;
    PciIdeControllerNumber = PciIdeControllerNumber.wrapping_add(1);
    (*FdoExtension).ControllerNumber = fresh;

    ExInitializeFastMutex(core::ptr::addr_of_mut!((*FdoExtension).PdoListSyncMutex));
    InitializeListHead(core::ptr::addr_of_mut!((*FdoExtension).PdoListHead));
    KeInitializeSpinLock(core::ptr::addr_of_mut!((*FdoExtension).Controller.Lock));

    'failure_cleanup: {
        Status = IoAttachDeviceToDeviceStackSafe(
            Fdo,
            PhysicalDeviceObject,
            core::ptr::addr_of_mut!((*FdoExtension).Common.__bindgen_anon_1.LowerDeviceObject),
        );
        if !NT_SUCCESS!(Status) {
            break 'failure_cleanup;
        }

        (*Fdo).AlignmentRequirement = (*(*FdoExtension).Common.__bindgen_anon_1.LowerDeviceObject).AlignmentRequirement;
        (*Fdo).AlignmentRequirement = max((*Fdo).AlignmentRequirement, ATA_MIN_BUFFER_ALIGNMENT);

        if !ControllerContext.is_null() {
            (*FdoExtension).Controller.Flags |= CTRL_FLAG_NON_PNP;
            *ControllerContext = FdoExtension as PVOID;
        } else {
            let mut BusInterface: BUS_INTERFACE_STANDARD = core::mem::zeroed();

            Status = PciIdeXPnpQueryInterface(
                core::ptr::addr_of_mut!((*FdoExtension).Common) as PCOMMON_DEVICE_EXTENSION,
                core::ptr::addr_of!(GUID_BUS_INTERFACE_STANDARD) as *const GUID,
                core::ptr::addr_of_mut!(BusInterface) as PVOID,
                PCI_BUS_INTERFACE_STANDARD_VERSION,
                core::mem::size_of::<BUS_INTERFACE_STANDARD>() as ULONG,
            );
            if !NT_SUCCESS!(Status) {
                break 'failure_cleanup;
            }

            (*FdoExtension).Controller.BusInterfaceContext = BusInterface.Context;
            (*FdoExtension).Controller.SetBusData = BusInterface.SetBusData;
            (*FdoExtension).Controller.GetBusData = BusInterface.GetBusData;
        }

        (*Fdo).Flags &= !DO_DEVICE_INITIALIZING;
        return STATUS_SUCCESS;
    }

    if !(*FdoExtension).Common.__bindgen_anon_1.LowerDeviceObject.is_null() {
        IoDetachDevice((*FdoExtension).Common.__bindgen_anon_1.LowerDeviceObject);
    }
    IoDeleteDevice(Fdo);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn PciIdeXAddDevice(
    DriverObject: PDRIVER_OBJECT,
    PhysicalDeviceObject: PDEVICE_OBJECT,
) -> NTSTATUS {
    PAGED_CODE!();

    return PciIdeXAddDeviceEx(DriverObject, PhysicalDeviceObject, core::ptr::null_mut());
}

unsafe extern "C" fn PciIdeXCreateIdeDirectory() {
    let mut Handle: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut Status: i32 = 0;
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = _OBJECT_ATTRIBUTES {
        Length: 0,
        RootDirectory: core::ptr::null_mut(),
        ObjectName: core::ptr::null_mut(),
        Attributes: 0,
        SecurityDescriptor: core::ptr::null_mut(),
        SecurityQualityOfService: core::ptr::null_mut(),
    };
    let mut DirectoryName: _UNICODE_STRING = _UNICODE_STRING {
        Length: (core::mem::size_of::<[u16; 12]>() - core::mem::size_of::<u16>()) as u16,
        MaximumLength: core::mem::size_of::<[u16; 12]>() as u16,
        Buffer: RUSTCONV_WSTR_604F52A85CB9.as_ptr() as *mut u16,
    };

    if KeGetCurrentIrql() as i32 > 1 {
        // PAGED_CODE assertion
    }

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.ObjectName = &mut DirectoryName;
    ObjectAttributes.Attributes = (OBJ_CASE_INSENSITIVE | OBJ_PERMANENT | OBJ_KERNEL_HANDLE) as u32;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = ZwCreateDirectoryObject(&mut Handle, DIRECTORY_ALL_ACCESS, &mut ObjectAttributes);
    if Status >= 0 {
        ZwClose(Handle);
    }
}

#[no_mangle]
pub unsafe extern "system" fn PciIdeXInitialize(
    DriverObject: PDRIVER_OBJECT,
    RegistryPath: PUNICODE_STRING,
    HwGetControllerProperties: PCONTROLLER_PROPERTIES,
    ExtensionSize: u32,
) -> NTSTATUS {
    let mut DriverExtension: PPCIIDEX_DRIVER_EXTENSION = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    PAGED_CODE!();

    INFO!(
        b"PciIdeXInitialize(%p '%wZ' %p 0x%lx)\n\0".as_ptr() as *const i8,
        DriverObject,
        RegistryPath,
        HwGetControllerProperties,
        ExtensionSize,
    );

    if ExtensionSize == PCIIDEX_GET_CONTROLLER_INTERFACE_SIGNATURE {
        return STATUS_NOT_SUPPORTED;
    }

    Status = IoAllocateDriverObjectExtension(
        DriverObject,
        DriverObject,
        core::mem::size_of::<PPCIIDEX_DRIVER_EXTENSION>() as u32,
        &mut DriverExtension as *mut PPCIIDEX_DRIVER_EXTENSION as *mut core::ffi::c_void,
    );
    if !NT_SUCCESS!(Status) {
        return Status;
    }

    RtlZeroMemory!(
        DriverExtension as *mut core::ffi::c_void,
        core::mem::size_of::<PPCIIDEX_DRIVER_EXTENSION>(),
    );
    (*DriverExtension).MiniControllerExtensionSize = ExtensionSize;
    (*DriverExtension).HwGetControllerProperties = HwGetControllerProperties;

    (*DriverObject).MajorFunction[IRP_MJ_PNP as usize] = Some(PciIdeXDispatchPnp);
    (*DriverObject).MajorFunction[IRP_MJ_POWER as usize] = Some(PciIdeXDispatchPower);
    (*DriverObject).MajorFunction[IRP_MJ_SYSTEM_CONTROL as usize] = Some(PciIdeXDispatchWmi);
    (*(*DriverObject).DriverExtension).AddDevice = Some(PciIdeXAddDevice);
    (*DriverObject).DriverUnload = Some(PciIdeXUnload);

    PciIdeXCreateIdeDirectory();

    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn DriverEntry(
    DriverObject: PDRIVER_OBJECT,
    RegistryPath: PUNICODE_STRING,
) -> NTSTATUS {
    if (KeGetCurrentIrql() as i32) as u32 > APC_LEVEL {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0 as i32 == 0 {
            RtlAssert(
                b"0\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
                b"drivers/storage/ide/pciidex/pciidex.c\0"
                    .as_ptr() as *const i8 as *mut core::ffi::c_void,
                579 as u32,
                core::ptr::null_mut::<i8>(),
            );
        }
    }
    return STATUS_SUCCESS;
}
