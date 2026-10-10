use crate::types::*;
use crate::glue::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{IoAttachDeviceToDeviceStackSafe, IoConnectInterrupt, IoCreateDevice, IoCreateSymbolicLink, IoDeleteDevice, IoDeleteSymbolicLink, IoForwardIrpSynchronously, IoGetConfigurationInformation, IoRegisterDeviceInterface, IoSetDeviceInterfaceState, KAFFINITY, KeInitializeDpc, KeInitializeEvent, _EVENT_TYPE_NotificationEvent, _OBJECT_ATTRIBUTES, PCM_PARTIAL_RESOURCE_DESCRIPTOR, PCM_RESOURCE_LIST, PDEVICE_OBJECT, PDRIVER_OBJECT, PIO_STACK_LOCATION, PIRP, RtlInitUnicodeString, ZwClose, ZwCreateKey, ZwSetValueKey};
// IoGetCurrentIrpStackLocation → crate::glue (tier1 FORCEINLINE, not exported on i386)
// KeInitializeSpinLock         → crate::glue (ke FORCEINLINE)
// IoInitializeRemoveLockEx, READ_PORT_UCHAR, WRITE_PORT_UCHAR, ULongToPtr,
//   RtlAssert, KdComPortInUse  → crate::glue inline fns / extern statics
use core::ffi::c_void;

extern "C" {
    fn DbgPrintEx(ComponentId: u32, Level: u32, Format: *const i8, ...) -> u32;
    fn swprintf(buffer: *mut u16, format: *const u16, ...) -> i32;
    fn InitializeCircularBuffer(Buffer: *mut CIRCULAR_BUFFER, BufferSize: u32) -> NTSTATUS;
    fn FreeCircularBuffer(Buffer: *mut CIRCULAR_BUFFER) -> NTSTATUS;
    fn SerialDetectUartType(ComPortBase: PUCHAR) -> UART_TYPE;
}
unsafe extern "system" {
    fn SerialSetBaudRate( DeviceExtension: PSERIAL_DEVICE_EXTENSION, BaudRate: ULONG, ) -> NTSTATUS;
    fn SerialSetLineControl( DeviceExtension: PSERIAL_DEVICE_EXTENSION, LineControl: *mut SERIAL_LINE_CONTROL, ) -> NTSTATUS;
    fn ForwardIrpAndForget(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn SerialAddDeviceInternal(
    DriverObject: PDRIVER_OBJECT,
    Pdo: PDEVICE_OBJECT,
    UartType: UART_TYPE,
    pComPortNumber: PULONG,
    pFdo: *mut PDEVICE_OBJECT,
) -> NTSTATUS {

    const DPFLTR_SERIAL_ID: u32 = 39; // DPFLTR_SERIAL_ID
    const DPFLTR_TRACE_LEVEL: u32 = 2; // DPFLTR_TRACE_LEVEL
    const DPFLTR_WARNING_LEVEL: u32 = 1; // DPFLTR_WARNING_LEVEL
    const FILE_DEVICE_SERIAL_PORT: u32 = 0x0000001b; // FILE_DEVICE_SERIAL_PORT
    const SERIAL_TAG: u32 = 1819436371; // 'lreS'

    let mut Fdo: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut DeviceExtension: PSERIAL_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut Status: NTSTATUS;
    let mut DeviceNameBuffer: [WCHAR; 32] = [0; 32];
    let mut DeviceName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };

    DbgPrintEx(
        DPFLTR_SERIAL_ID,
        DPFLTR_TRACE_LEVEL,
        b"(%s:%d) SerialAddDeviceInternal()\n\0" as *const u8 as *const i8,
        b"pnp.c\0" as *const u8 as *const i8,
        31,
    );

    if DriverObject.is_null() {
        RtlAssert(
            b"DriverObject\0" as *const u8 as *mut i8 as *mut core::ffi::c_void,
            b"pnp.c\0" as *const u8 as *mut i8 as *mut core::ffi::c_void,
            33,
            core::ptr::null_mut(),
        );
    } else {
    };
    if Pdo.is_null() {
        RtlAssert(
            b"Pdo\0" as *const u8 as *mut i8 as *mut core::ffi::c_void,
            b"pnp.c\0" as *const u8 as *mut i8 as *mut core::ffi::c_void,
            34,
            core::ptr::null_mut(),
        );
    } else {
    };

    swprintf(
        core::ptr::addr_of_mut!(DeviceNameBuffer) as *mut u16,
        core::mem::transmute::<&[u16; 18], *const u16>(&[
            0x5C, 0x44, 0x65, 0x76, 0x69, 0x63, 0x65, 0x5C, 0x53, 0x65, 0x72, 0x69, 0x61, 0x6C,
            0x25, 0x6C, 0x75, 0x00,
        ]),
        (*IoGetConfigurationInformation()).SerialCount,
    );
    RtlInitUnicodeString(
        core::ptr::addr_of_mut!(DeviceName),
        core::ptr::addr_of_mut!(DeviceNameBuffer) as *mut WCHAR,
    );
    Status = IoCreateDevice(
        DriverObject,
        core::mem::size_of::<SERIAL_DEVICE_EXTENSION>() as u32,
        core::ptr::addr_of_mut!(DeviceName),
        FILE_DEVICE_SERIAL_PORT,
        FILE_DEVICE_SECURE_OPEN,
        0,
        core::ptr::addr_of_mut!(Fdo),
    );
    if !NT_SUCCESS(Status) {
        DbgPrintEx(
            DPFLTR_SERIAL_ID,
            DPFLTR_WARNING_LEVEL,
            b"(%s:%d) IoCreateDevice() failed with status 0x%08x\n\0" as *const u8 as *const i8,
            b"pnp.c\0" as *const u8 as *const i8,
            48,
            Status,
        );
        Fdo = core::ptr::null_mut();
    } else {
        DeviceExtension = (*Fdo).DeviceExtension as PSERIAL_DEVICE_EXTENSION;
        RtlZeroMemory(
            DeviceExtension as *mut core::ffi::c_void,
            core::mem::size_of::<SERIAL_DEVICE_EXTENSION>(),
        );
        Status = IoRegisterDeviceInterface(
            Pdo,
            core::ptr::addr_of!(GUID_DEVINTERFACE_COMPORT),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!((*DeviceExtension).SerialInterfaceName),
        );
        if !NT_SUCCESS(Status) {
            DbgPrintEx(
                DPFLTR_SERIAL_ID,
                DPFLTR_WARNING_LEVEL,
                b"(%s:%d) IoRegisterDeviceInterface() failed with status 0x%08x\n\0" as *const u8
                    as *const i8,
                b"pnp.c\0" as *const u8 as *const i8,
                59,
                Status,
            );
        } else {
            let fresh0 = core::ptr::addr_of_mut!((*IoGetConfigurationInformation()).SerialCount);
            let fresh1 = *fresh0;
            *fresh0 = (*fresh0).wrapping_add(1);
            (*DeviceExtension).SerialPortNumber = fresh1;
            if pComPortNumber.is_null() {
                (*DeviceExtension).ComPort =
                    ((*DeviceExtension).SerialPortNumber as core::ffi::c_ulong).wrapping_add(1)
                        as u32;
            } else {
                (*DeviceExtension).ComPort = *pComPortNumber as u32;
            }
            (*DeviceExtension).Pdo = Pdo;
            (*DeviceExtension).PnpState = dsStopped;
            (*DeviceExtension).UartType = UartType;
            Status = InitializeCircularBuffer(
                core::ptr::addr_of_mut!((*DeviceExtension).InputBuffer),
                16,
            );
            if NT_SUCCESS(Status) {
                Status = InitializeCircularBuffer(
                    core::ptr::addr_of_mut!((*DeviceExtension).OutputBuffer),
                    16,
                );
                if NT_SUCCESS(Status) {
                    IoInitializeRemoveLockEx(
                        core::ptr::addr_of_mut!((*DeviceExtension).RemoveLock) as PVOID,
                        SERIAL_TAG,
                        0,
                        0,
                        core::mem::size_of::<IO_REMOVE_LOCK>() as u32,
                    );
                    KeInitializeSpinLock(core::ptr::addr_of_mut!((*DeviceExtension).InputBufferLock) as *mut usize);
                    KeInitializeSpinLock(core::ptr::addr_of_mut!((*DeviceExtension).OutputBufferLock) as *mut usize);
                    KeInitializeEvent(
                        core::ptr::addr_of_mut!((*DeviceExtension).InputBufferNotEmpty),
                        _EVENT_TYPE_NotificationEvent,
                        0,
                    );
                    KeInitializeDpc(
                        core::ptr::addr_of_mut!((*DeviceExtension).ReceivedByteDpc),
                        Some(SerialReceiveByte),
                        DeviceExtension as PVOID,
                    );
                    KeInitializeDpc(
                        core::ptr::addr_of_mut!((*DeviceExtension).SendByteDpc),
                        Some(SerialSendByte),
                        DeviceExtension as PVOID,
                    );
                    KeInitializeDpc(
                        core::ptr::addr_of_mut!((*DeviceExtension).CompleteIrpDpc),
                        Some(SerialCompleteIrp),
                        DeviceExtension as PVOID,
                    );
                    Status = IoAttachDeviceToDeviceStackSafe(
                        Fdo,
                        Pdo,
                        core::ptr::addr_of_mut!((*DeviceExtension).LowerDevice),
                    );
                    if !NT_SUCCESS(Status) {
                        DbgPrintEx(
                            DPFLTR_SERIAL_ID,
                            DPFLTR_WARNING_LEVEL,
                            b"(%s:%d) IoAttachDeviceToDeviceStackSafe() failed with status 0x%08x\n\0"
                                as *const u8 as *const i8,
                            b"pnp.c\0" as *const u8 as *const i8,
                            85,
                            Status,
                        );
                    } else {
                        if (*(*DeviceExtension).LowerDevice).Flags as core::ffi::c_ulong
                            & DO_POWER_PAGABLE as core::ffi::c_ulong
                            != 0
                        {
                            (*Fdo).Flags |= DO_POWER_PAGABLE as core::ffi::c_ulong;
                        }
                        if (*(*DeviceExtension).LowerDevice).Flags as core::ffi::c_ulong
                            & DO_BUFFERED_IO as core::ffi::c_ulong
                            != 0
                        {
                            (*Fdo).Flags |= DO_BUFFERED_IO as core::ffi::c_ulong;
                        }
                        if (*(*DeviceExtension).LowerDevice).Flags as core::ffi::c_ulong
                            & DO_DIRECT_IO as core::ffi::c_ulong
                            != 0
                        {
                            (*Fdo).Flags |= DO_DIRECT_IO as core::ffi::c_ulong;
                        }
                        if (*Fdo).Flags as core::ffi::c_ulong
                            & (DO_BUFFERED_IO | DO_DIRECT_IO) as core::ffi::c_ulong
                            == 0
                        {
                            (*Fdo).Flags |= DO_BUFFERED_IO as core::ffi::c_ulong;
                        }
                        (*Fdo).Flags &= !DO_DEVICE_INITIALIZING as core::ffi::c_ulong;
                        if !pFdo.is_null() {
                            *pFdo = Fdo;
                        }
                        return STATUS_SUCCESS;
                    }
                }
            }
        }
    }
    if !Fdo.is_null() {
        FreeCircularBuffer(core::ptr::addr_of_mut!((*DeviceExtension).InputBuffer));
        FreeCircularBuffer(core::ptr::addr_of_mut!((*DeviceExtension).OutputBuffer));
        IoDeleteDevice(Fdo);
    }
    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn SerialAddDevice(
    DriverObject: PDRIVER_OBJECT,
    Pdo: PDEVICE_OBJECT,
) -> NTSTATUS {
    if Pdo.is_null() {
        return STATUS_SUCCESS;
    }
    return SerialAddDeviceInternal(
        DriverObject,
        Pdo,
        UartUnknown,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
}

#[no_mangle]
pub unsafe extern "system" fn SerialPnpStartDevice(
    mut DeviceObject: PDEVICE_OBJECT,
    mut ResourceList: PCM_RESOURCE_LIST,
    mut ResourceListTranslated: PCM_RESOURCE_LIST,
) -> NTSTATUS {

    const DPFLTR_SERIAL_ID: ::core::ffi::c_int = 39;
    const DPFLTR_WARNING_LEVEL: ULONG = 1;
    const DPFLTR_INFO_LEVEL: ULONG = 3;
    const CmResourceTypePort: ::core::ffi::c_int = 1;
    const CmResourceTypeInterrupt: ::core::ffi::c_int = 2;
    const CM_RESOURCE_INTERRUPT_LATCHED: ::core::ffi::c_int = 1;
    const SR_IER_DATA_RECEIVED: ::core::ffi::c_int = 0x01;
    const SR_IER_THR_EMPTY: ::core::ffi::c_int = 0x02;
    const SR_IER_LSR_CHANGE: ::core::ffi::c_int = 0x04;
    const SR_IER_MSR_CHANGE: ::core::ffi::c_int = 0x08;
    const SR_MCR_DTR: ::core::ffi::c_int = 0x01;
    const SR_MCR_RTS: ::core::ffi::c_int = 0x02;
    const STOP_BIT_1: ::core::ffi::c_int = 0;
    const NO_PARITY: ::core::ffi::c_int = 0;
    const REG_OPTION_VOLATILE: ULONG = 0x1;

    let mut DeviceExtension: PSERIAL_DEVICE_EXTENSION =
        ::core::ptr::null_mut::<_SERIAL_DEVICE_EXTENSION>();
    let mut DeviceNameBuffer: [WCHAR; 32] = [0; 32];
    let mut DeviceName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: ::core::ptr::null_mut::<WCHAR>(),
    };
    let mut LinkNameBuffer: [WCHAR; 32] = [0; 32];
    let mut LinkName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: ::core::ptr::null_mut::<WCHAR>(),
    };
    let mut ComPortBuffer: [WCHAR; 32] = [0; 32];
    let mut ComPort: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: ::core::ptr::null_mut::<WCHAR>(),
    };
    let mut Vector: ULONG = 0 as ULONG;
    let mut i: ULONG = 0;
    let mut IER: UCHAR = 0;
    let mut Dirql: KIRQL = 0;
    let mut Affinity: KAFFINITY = 0 as KAFFINITY;
    let mut InterruptMode: KINTERRUPT_MODE = Latched;
    let mut ShareInterrupt: BOOLEAN = 1 as BOOLEAN;
    let mut objectAttributes: _OBJECT_ATTRIBUTES = _OBJECT_ATTRIBUTES {
        Length: 0,
        RootDirectory: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        ObjectName: ::core::ptr::null_mut::<_UNICODE_STRING>(),
        Attributes: 0,
        SecurityDescriptor: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        SecurityQualityOfService: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    let mut ComPortBase: PUCHAR = ::core::ptr::null_mut::<::core::ffi::c_uchar>();
    let mut KeyName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: ::core::ptr::null_mut::<WCHAR>(),
    };
    let mut hKey: HANDLE = ::core::ptr::null_mut::<::core::ffi::c_void>();
    let mut Status: NTSTATUS = 0;
    DeviceExtension = (*DeviceObject).DeviceExtension as PSERIAL_DEVICE_EXTENSION;
    if DeviceExtension.is_null() {
        RtlAssert(
            b"DeviceExtension\0" as *const u8 as *const ::core::ffi::c_char as PVOID,
            b"drivers/serial/serial/pnp.c\0"
                as *const u8 as *const ::core::ffi::c_char as PVOID,
            163 as ULONG,
            ::core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };
    if ResourceList.is_null() {
        DbgPrintEx(
            DPFLTR_SERIAL_ID as ULONG,
            DPFLTR_WARNING_LEVEL,
            b"(%s:%d) No allocated resources sent to driver\n\0" as *const u8 as PCSTR,
            b"drivers/serial/serial/pnp.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            167 as ::core::ffi::c_int,
        );
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    if (*ResourceList).Count != 1 as ::core::ffi::c_ulong {
        DbgPrintEx(
            DPFLTR_SERIAL_ID as ULONG,
            DPFLTR_WARNING_LEVEL,
            b"(%s:%d) Wrong number of allocated resources sent to driver\n\0" as *const u8 as PCSTR,
            b"drivers/serial/serial/pnp.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            172 as ::core::ffi::c_int,
        );
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    if (*(&raw mut (*ResourceList).List as *mut CM_FULL_RESOURCE_DESCRIPTOR)
        .offset(0 as ::core::ffi::c_int as isize))
    .PartialResourceList
    .Version as ::core::ffi::c_int
        != 1 as ::core::ffi::c_int
        || (*(&raw mut (*ResourceList).List as *mut CM_FULL_RESOURCE_DESCRIPTOR)
            .offset(0 as ::core::ffi::c_int as isize))
        .PartialResourceList
        .Revision as ::core::ffi::c_int
            != 1 as ::core::ffi::c_int
        || (*(&raw mut (*ResourceListTranslated).List as *mut CM_FULL_RESOURCE_DESCRIPTOR)
            .offset(0 as ::core::ffi::c_int as isize))
        .PartialResourceList
        .Version as ::core::ffi::c_int
            != 1 as ::core::ffi::c_int
        || (*(&raw mut (*ResourceListTranslated).List as *mut CM_FULL_RESOURCE_DESCRIPTOR)
            .offset(0 as ::core::ffi::c_int as isize))
        .PartialResourceList
        .Revision as ::core::ffi::c_int
            != 1 as ::core::ffi::c_int
    {
        DbgPrintEx(
            DPFLTR_SERIAL_ID as ULONG,
            DPFLTR_WARNING_LEVEL,
            b"(%s:%d) Revision mismatch: %u.%u != 1.1 or %u.%u != 1.1\n\0" as *const u8 as PCSTR,
            b"drivers/serial/serial/pnp.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            184 as ::core::ffi::c_int,
            (*(&raw mut (*ResourceList).List as *mut CM_FULL_RESOURCE_DESCRIPTOR)
                .offset(0 as ::core::ffi::c_int as isize))
            .PartialResourceList
            .Version as ::core::ffi::c_int,
            (*(&raw mut (*ResourceList).List as *mut CM_FULL_RESOURCE_DESCRIPTOR)
                .offset(0 as ::core::ffi::c_int as isize))
            .PartialResourceList
            .Revision as ::core::ffi::c_int,
            (*(&raw mut (*ResourceListTranslated).List as *mut CM_FULL_RESOURCE_DESCRIPTOR)
                .offset(0 as ::core::ffi::c_int as isize))
            .PartialResourceList
            .Version as ::core::ffi::c_int,
            (*(&raw mut (*ResourceListTranslated).List as *mut CM_FULL_RESOURCE_DESCRIPTOR)
                .offset(0 as ::core::ffi::c_int as isize))
            .PartialResourceList
            .Revision as ::core::ffi::c_int,
        );
        return STATUS_REVISION_MISMATCH;
    }
    (*DeviceExtension).BaudRate = 19200 as ULONG;
    (*DeviceExtension).BaseAddress = 0 as ULONG;
    Dirql = 0 as KIRQL;
    i = 0 as ULONG;
    while i
        < (*(&raw mut (*ResourceList).List as *mut CM_FULL_RESOURCE_DESCRIPTOR)
            .offset(0 as ::core::ffi::c_int as isize))
        .PartialResourceList
        .Count
    {
        let mut PartialDescriptor: PCM_PARTIAL_RESOURCE_DESCRIPTOR =
            (&raw mut (*(&raw mut (*ResourceList).List as *mut CM_FULL_RESOURCE_DESCRIPTOR)
                .offset(0 as ::core::ffi::c_int as isize))
            .PartialResourceList
            .PartialDescriptors as *mut CM_PARTIAL_RESOURCE_DESCRIPTOR)
                .offset(i as isize) as PCM_PARTIAL_RESOURCE_DESCRIPTOR;
        let mut PartialDescriptorTranslated: PCM_PARTIAL_RESOURCE_DESCRIPTOR =
            (&raw mut (*(&raw mut (*ResourceListTranslated).List
                as *mut CM_FULL_RESOURCE_DESCRIPTOR)
                .offset(0 as ::core::ffi::c_int as isize))
            .PartialResourceList
            .PartialDescriptors as *mut CM_PARTIAL_RESOURCE_DESCRIPTOR)
                .offset(i as isize) as PCM_PARTIAL_RESOURCE_DESCRIPTOR;
        match (*PartialDescriptor).Type as ::core::ffi::c_int {
            CmResourceTypePort => {
                if (*PartialDescriptor).u.Port.Length < 7 as ::core::ffi::c_ulong {
                    return STATUS_INSUFFICIENT_RESOURCES;
                }
                if (*DeviceExtension).BaseAddress != 0 as ::core::ffi::c_ulong {
                    return STATUS_UNSUCCESSFUL;
                }
                (*DeviceExtension).BaseAddress = (*PartialDescriptor).u.Port.Start.u.LowPart;
            }
            CmResourceTypeInterrupt => {
                Dirql = (*PartialDescriptorTranslated).u.Interrupt.Level as KIRQL;
                Vector = (*PartialDescriptorTranslated).u.Interrupt.Vector;
                Affinity = (*PartialDescriptorTranslated).u.Interrupt.Affinity;
                if (*PartialDescriptorTranslated).Flags as ::core::ffi::c_int
                    & CM_RESOURCE_INTERRUPT_LATCHED
                    != 0
                {
                    InterruptMode = Latched;
                } else {
                    InterruptMode = LevelSensitive;
                }
                ShareInterrupt = ((*PartialDescriptorTranslated).ShareDisposition
                    as ::core::ffi::c_int
                    == CmResourceShareShared as ::core::ffi::c_int)
                    as ::core::ffi::c_int as BOOLEAN;
            }
            _ => {}
        }
        i = i.wrapping_add(1);
    }
    DbgPrintEx(
        DPFLTR_SERIAL_ID as ULONG,
        DPFLTR_INFO_LEVEL,
        b"(%s:%d) New COM port. Base = 0x%lx, Irql = %u\n\0" as *const u8 as PCSTR,
        b"drivers/serial/serial/pnp.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        217 as ::core::ffi::c_int,
        (*DeviceExtension).BaseAddress,
        Dirql as ::core::ffi::c_int,
    );
    if (*DeviceExtension).BaseAddress == 0 {
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    if Dirql == 0 {
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    ComPortBase = ULongToPtr((*DeviceExtension).BaseAddress as ::core::ffi::c_ulong) as PUCHAR;
    DbgPrintEx(
        DPFLTR_SERIAL_ID as ULONG,
        DPFLTR_INFO_LEVEL,
        b"(%s:%d) Comparing addresses: KdComPortInUse: %p, ComPortBase: %p\n\0" as *const u8
            as PCSTR,
        b"drivers/serial/serial/pnp.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        225 as ::core::ffi::c_int,
        KdComPortInUse,
        ComPortBase,
    );
    if KdComPortInUse == ComPortBase {
        DbgPrintEx(
            DPFLTR_SERIAL_ID as ULONG,
            DPFLTR_INFO_LEVEL,
            b"(%s:%d) Failing IRP_MN_START_DEVICE as this serial port is used for debugging\n\0"
                as *const u8 as PCSTR,
            b"drivers/serial/serial/pnp.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            228 as ::core::ffi::c_int,
        );
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    if (*DeviceExtension).UartType as ::core::ffi::c_uint
        == UartUnknown as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        (*DeviceExtension).UartType = SerialDetectUartType(ComPortBase);
    }
    (*DeviceExtension).MCR = READ_PORT_UCHAR(ComPortBase.offset(4 as ::core::ffi::c_int as isize));
    (*DeviceExtension).MSR = READ_PORT_UCHAR(ComPortBase.offset(6 as ::core::ffi::c_int as isize));
    (*DeviceExtension).WaitMask = 0 as ULONG;
    Status = SerialSetBaudRate(DeviceExtension, (*DeviceExtension).BaudRate);
    if !(Status >= 0 as ::core::ffi::c_long) {
        DbgPrintEx(
            DPFLTR_SERIAL_ID as ULONG,
            DPFLTR_WARNING_LEVEL,
            b"(%s:%d) SerialSetBaudRate() failed with status 0x%08x\n\0" as *const u8 as PCSTR,
            b"drivers/serial/serial/pnp.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            244 as ::core::ffi::c_int,
            Status,
        );
        return Status;
    }
    (*DeviceExtension).SerialLineControl.StopBits = STOP_BIT_1 as UCHAR;
    (*DeviceExtension).SerialLineControl.Parity = NO_PARITY as UCHAR;
    (*DeviceExtension).SerialLineControl.WordLength = 8 as UCHAR;
    Status = SerialSetLineControl(
        DeviceExtension,
        &raw mut (*DeviceExtension).SerialLineControl,
    );
    if !(Status >= 0 as ::core::ffi::c_long) {
        DbgPrintEx(
            DPFLTR_SERIAL_ID as ULONG,
            DPFLTR_WARNING_LEVEL,
            b"(%s:%d) SerialSetLineControl() failed with status 0x%08x\n\0" as *const u8 as PCSTR,
            b"drivers/serial/serial/pnp.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            255 as ::core::ffi::c_int,
            Status,
        );
        return Status;
    }
    if (*DeviceExtension).UartType as ::core::ffi::c_uint
        >= Uart16550A as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        WRITE_PORT_UCHAR(
            ComPortBase.offset(2 as ::core::ffi::c_int as isize),
            (SR_FCR_CLEAR_RCVR | SR_FCR_CLEAR_XMIT) as UCHAR,
        );
    }
    {
        let fmt_dev: [::core::ffi::c_ushort; 18] = ::core::mem::transmute(*b"\\\0D\0e\0v\0i\0c\0e\0\\\0S\0e\0r\0i\0a\0l\0%\0l\0u\0\0\0");
        swprintf(&raw mut DeviceNameBuffer as *mut wchar_t, fmt_dev.as_ptr() as *const wchar_t, (*DeviceExtension).SerialPortNumber);
    }
    {
        let fmt_link: [::core::ffi::c_ushort; 19] = ::core::mem::transmute(*b"\\\0D\0o\0s\0D\0e\0v\0i\0c\0e\0s\0\\\0C\0O\0M\0%\0l\0u\0\0\0");
        swprintf(&raw mut LinkNameBuffer as *mut wchar_t, fmt_link.as_ptr() as *const wchar_t, (*DeviceExtension).ComPort);
    }
    {
        let fmt_com: [::core::ffi::c_ushort; 7] = ::core::mem::transmute(*b"C\0O\0M\0%\0l\0u\0\0\0");
        swprintf(&raw mut ComPortBuffer as *mut wchar_t, fmt_com.as_ptr() as *const wchar_t, (*DeviceExtension).ComPort);
    }
    RtlInitUnicodeString(
        &raw mut DeviceName,
        &raw mut DeviceNameBuffer as *mut WCHAR as PCWSTR,
    );
    RtlInitUnicodeString(
        &raw mut LinkName,
        &raw mut LinkNameBuffer as *mut WCHAR as PCWSTR,
    );
    RtlInitUnicodeString(
        &raw mut ComPort,
        &raw mut ComPortBuffer as *mut WCHAR as PCWSTR,
    );
    Status = IoCreateSymbolicLink(&raw mut LinkName, &raw mut DeviceName);
    if !(Status >= 0 as ::core::ffi::c_long) {
        DbgPrintEx(
            DPFLTR_SERIAL_ID as ULONG,
            DPFLTR_WARNING_LEVEL,
            b"(%s:%d) IoCreateSymbolicLink() failed with status 0x%08x\n\0" as *const u8 as PCSTR,
            b"drivers/serial/serial/pnp.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            277 as ::core::ffi::c_int,
            Status,
        );
        return Status;
    }
    Status = IoConnectInterrupt(
        &raw mut (*DeviceExtension).Interrupt,
        Some(SerialInterruptService),
        DeviceObject as PVOID,
        {
            #[cfg(target_arch = "x86")] { ::core::ptr::null_mut::<u32>() }
            #[cfg(target_arch = "x86_64")] { ::core::ptr::null_mut::<u64>() }
            #[cfg(target_arch = "aarch64")] { ::core::ptr::null_mut::<u64>() }
        },
        Vector,
        Dirql,
        Dirql,
        InterruptMode,
        ShareInterrupt,
        Affinity,
        0 as BOOLEAN,
    );
    if !(Status >= 0 as ::core::ffi::c_long) {
        DbgPrintEx(
            DPFLTR_SERIAL_ID as ULONG,
            DPFLTR_WARNING_LEVEL,
            b"(%s:%d) IoConnectInterrupt() failed with status 0x%08x\n\0" as *const u8 as PCSTR,
            b"drivers/serial/serial/pnp.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            290 as ::core::ffi::c_int,
            Status,
        );
        IoSetDeviceInterfaceState(
            &raw mut (*DeviceExtension).SerialInterfaceName,
            0 as BOOLEAN,
        );
        IoDeleteSymbolicLink(&raw mut LinkName);
        return Status;
    }
    {
        let fmt_key: [::core::ffi::c_ushort; 48] = ::core::mem::transmute(*b"\\\0R\0e\0g\0i\0s\0t\0r\0y\0\\\0M\0a\0c\0h\0i\0n\0e\0\\\0H\0A\0R\0D\0W\0A\0R\0E\0\\\0D\0e\0v\0i\0c\0e\0M\0a\0p\0\\\0S\0E\0R\0I\0A\0L\0C\0O\0M\0M\0\0\0");
        RtlInitUnicodeString(&raw mut KeyName, fmt_key.as_ptr() as PCWSTR);
    }
    objectAttributes.Length = ::core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    objectAttributes.RootDirectory = ::core::ptr::null_mut::<::core::ffi::c_void>() as HANDLE;
    objectAttributes.ObjectName = &raw mut KeyName as PUNICODE_STRING;
    objectAttributes.Attributes =
        (OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE) as ULONG;
    objectAttributes.SecurityDescriptor = ::core::ptr::null_mut::<::core::ffi::c_void>() as PVOID;
    objectAttributes.SecurityQualityOfService = core::ptr::null_mut() as PVOID;
    Status = ZwCreateKey(
        &raw mut hKey,
        KEY_SET_VALUE as ACCESS_MASK,
        &raw mut objectAttributes,
        0 as ULONG,
        ::core::ptr::null_mut::<_UNICODE_STRING>(),
        REG_OPTION_VOLATILE as ULONG,
        ::core::ptr::null_mut::<::core::ffi::c_ulong>(),
    );
    if Status >= 0 as ::core::ffi::c_long {
        ZwSetValueKey(
            hKey,
            &raw mut DeviceName,
            0 as ULONG,
            REG_SZ as ULONG,
            &raw mut ComPortBuffer as *mut WCHAR as PVOID,
            (ComPort.Length as usize).wrapping_add(::core::mem::size_of::<WCHAR>() as usize)
                as ULONG,
        );
        ZwClose(hKey);
    }
    (*DeviceExtension).PnpState = dsStarted;
    IER = READ_PORT_UCHAR(ComPortBase.offset(1 as ::core::ffi::c_int as isize));
    IER = (IER as ::core::ffi::c_int
        | (SR_IER_DATA_RECEIVED | SR_IER_THR_EMPTY | SR_IER_LSR_CHANGE | SR_IER_MSR_CHANGE))
        as UCHAR;
    WRITE_PORT_UCHAR(ComPortBase.offset(1 as ::core::ffi::c_int as isize), IER);
    (*DeviceExtension).MCR =
        ((*DeviceExtension).MCR as ::core::ffi::c_int | (SR_MCR_DTR | SR_MCR_RTS)) as UCHAR;
    WRITE_PORT_UCHAR(
        ComPortBase.offset(4 as ::core::ffi::c_int as isize),
        (*DeviceExtension).MCR,
    );
    IoSetDeviceInterfaceState(
        &raw mut (*DeviceExtension).SerialInterfaceName,
        1 as BOOLEAN,
    );
    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn SerialPnp(
    mut DeviceObject: PDEVICE_OBJECT,
    mut Irp: PIRP,
) -> NTSTATUS {

    let mut DeviceExtension: PSERIAL_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut MinorFunction: ULONG;
    let mut Stack: PIO_STACK_LOCATION;
    let mut Information: ULONG_PTR = 0 as ULONG_PTR;
    let mut Status: NTSTATUS = 0;

    Stack = IoGetCurrentIrpStackLocation(Irp);
    MinorFunction = (*Stack).MinorFunction as ULONG;

    match MinorFunction {
        0 => {
            /* IRP_MN_START_DEVICE */
            /* TRACE_!(SERIAL, "IRP_MJ_PNP / IRP_MN_START_DEVICE\n"); */

            DeviceExtension = (*DeviceObject).DeviceExtension as PSERIAL_DEVICE_EXTENSION;
            Status = STATUS_UNSUCCESSFUL;

            /* ASSERT(DeviceExtension->PnpState == dsStopped); */

            if IoForwardIrpSynchronously((*DeviceExtension).LowerDevice, Irp) != 0 {
                Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
                if NT_SUCCESS(Status) {
                    Status = SerialPnpStartDevice(
                        DeviceObject,
                        (*Stack).Parameters.StartDevice.AllocatedResources,
                        (*Stack).Parameters.StartDevice.AllocatedResourcesTranslated,
                    );
                }
            }
        }
        7 => {
            /* IRP_MN_QUERY_DEVICE_RELATIONS */
            match (*Stack).Parameters.QueryDeviceRelations.Type as u32 {
                0 => {
                    /* BusRelations */
                    /* TRACE_ */
                    return ForwardIrpAndForget(DeviceObject, Irp);
                }
                3 => {
                    /* RemovalRelations */
                    /* TRACE_ */
                    return ForwardIrpAndForget(DeviceObject, Irp);
                }
                _ => {
                    /* TRACE_ Unknown type */
                    return ForwardIrpAndForget(DeviceObject, Irp);
                }
            }
        }
        13 => {
            /* IRP_MN_FILTER_RESOURCE_REQUIREMENTS */
            /* TRACE_ */
            return ForwardIrpAndForget(DeviceObject, Irp);
        }
        0x17 => {
            /* IRP_MN_SURPRISE_REMOVAL — upstream ae50db9f96d ([SERIAL] Implement
             * surprise removal handler, #9202), landed on the C after this file
             * was translated. Without it a yanked port kept its interrupt
             * connected, its \DosDevices\COMn link and its SERIALCOMM value. */
            DeviceExtension = (*DeviceObject).DeviceExtension as PSERIAL_DEVICE_EXTENSION;
            (*DeviceExtension).PnpState = crate::types::SERIAL_DEVICE_STATE_dsSurpriseRemoved;

            IoSetDeviceInterfaceState(
                &raw mut (*DeviceExtension).SerialInterfaceName,
                0 as BOOLEAN,
            );

            if !(*DeviceExtension).Interrupt.is_null() {
                ntkrnl_sys::ddk::IoDisconnectInterrupt((*DeviceExtension).Interrupt);
                (*DeviceExtension).Interrupt = core::ptr::null_mut();
            }

            let mut LinkNameBuffer: [WCHAR; 32] = [0; 32];
            let mut DeviceNameBuffer: [WCHAR; 32] = [0; 32];
            let mut LinkName: _UNICODE_STRING = core::mem::zeroed();
            let mut DeviceName: _UNICODE_STRING = core::mem::zeroed();
            let mut KeyName: _UNICODE_STRING = core::mem::zeroed();
            let mut objectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
            let mut hKey: HANDLE = core::ptr::null_mut();

            let fmt_link: [::core::ffi::c_ushort; 19] = ::core::mem::transmute(*b"\\\0D\0o\0s\0D\0e\0v\0i\0c\0e\0s\0\\\0C\0O\0M\0%\0l\0u\0\0\0");
            swprintf(&raw mut LinkNameBuffer as *mut u16, fmt_link.as_ptr(), (*DeviceExtension).ComPort);
            RtlInitUnicodeString(&raw mut LinkName, &raw mut LinkNameBuffer as *mut WCHAR as PCWSTR);
            IoDeleteSymbolicLink(&raw mut LinkName);

            let fmt_dev: [::core::ffi::c_ushort; 18] = ::core::mem::transmute(*b"\\\0D\0e\0v\0i\0c\0e\0\\\0S\0e\0r\0i\0a\0l\0%\0l\0u\0\0\0");
            swprintf(&raw mut DeviceNameBuffer as *mut u16, fmt_dev.as_ptr(), (*DeviceExtension).SerialPortNumber);
            RtlInitUnicodeString(&raw mut DeviceName, &raw mut DeviceNameBuffer as *mut WCHAR as PCWSTR);
            let fmt_key: [::core::ffi::c_ushort; 48] = ::core::mem::transmute(*b"\\\0R\0e\0g\0i\0s\0t\0r\0y\0\\\0M\0a\0c\0h\0i\0n\0e\0\\\0H\0A\0R\0D\0W\0A\0R\0E\0\\\0D\0e\0v\0i\0c\0e\0M\0a\0p\0\\\0S\0E\0R\0I\0A\0L\0C\0O\0M\0M\0\0\0");
            RtlInitUnicodeString(&raw mut KeyName, fmt_key.as_ptr() as PCWSTR);
            objectAttributes.Length = ::core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
            objectAttributes.ObjectName = &raw mut KeyName as PUNICODE_STRING;
            objectAttributes.Attributes = (OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE) as ULONG;

            if NT_SUCCESS(ntkrnl_sys::ddk::ZwOpenKey(
                &raw mut hKey,
                KEY_SET_VALUE as ACCESS_MASK,
                &raw mut objectAttributes,
            )) {
                ntkrnl_sys::ddk::ZwDeleteValueKey(hKey, &raw mut DeviceName);
                ZwClose(hKey);
            }

            return ForwardIrpAndForget(DeviceObject, Irp);
        }
        _ => {
            /* TRACE_ Unknown minor function */
            return ForwardIrpAndForget(DeviceObject, Irp);
        }
    }

    (*Irp).IoStatus.Information = Information as _;
    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    IoCompleteRequest(Irp, IO_NO_INCREMENT as CCHAR);
    return Status;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         Serial port driver
//  * FILE:            drivers/dd/serial/pnp.c
//  * PURPOSE:         Serial IRP_MJ_PNP operations
//  *
//  * PROGRAMMERS:     Herv� Poussineau (hpoussin@reactos.org)
//  */
// /* FIXME: call IoAcquireRemoveLock/IoReleaseRemoveLock around each I/O operation */
//
// #include "serial.h"
//
// #include <stdio.h>
// #include <ndk/haltypes.h>
// #include <debug.h>
//
// NTSTATUS NTAPI
// SerialAddDeviceInternal(
// 	IN PDRIVER_OBJECT DriverObject,
// 	IN PDEVICE_OBJECT Pdo,
// 	IN UART_TYPE UartType,
// 	IN PULONG pComPortNumber OPTIONAL,
// 	OUT PDEVICE_OBJECT* pFdo OPTIONAL)
// {
// 	PDEVICE_OBJECT Fdo = NULL;
// 	PSERIAL_DEVICE_EXTENSION DeviceExtension = NULL;
// 	NTSTATUS Status;
// 	WCHAR DeviceNameBuffer[32];
// 	UNICODE_STRING DeviceName;
//
// 	TRACE_!(SERIAL, "SerialAddDeviceInternal()\n");
//
// 	ASSERT(DriverObject);
// 	ASSERT(Pdo);
//
// 	/* Create new device object */
// 	swprintf(DeviceNameBuffer, L"\\Device\\Serial%lu", IoGetConfigurationInformation()->SerialCount);
// 	RtlInitUnicodeString(&DeviceName, DeviceNameBuffer);
// 	Status = IoCreateDevice(DriverObject,
// 	                        sizeof(SERIAL_DEVICE_EXTENSION),
// 	                        &DeviceName,
// 	                        FILE_DEVICE_SERIAL_PORT,
// 	                        FILE_DEVICE_SECURE_OPEN,
// 	                        FALSE,
// 	                        &Fdo);
// 	if (!NT_SUCCESS(Status))
// 	{
// 		WARN_(SERIAL, "IoCreateDevice() failed with status 0x%08x\n", Status);
// 		Fdo = NULL;
// 		goto ByeBye;
// 	}
// 	DeviceExtension = (PSERIAL_DEVICE_EXTENSION)Fdo->DeviceExtension;
// 	RtlZeroMemory(DeviceExtension, sizeof(SERIAL_DEVICE_EXTENSION));
//
// 	/* Register device interface */
// 	Status = IoRegisterDeviceInterface(Pdo, &GUID_DEVINTERFACE_COMPORT, NULL, &DeviceExtension->SerialInterfaceName);
// 	if (!NT_SUCCESS(Status))
// 	{
// 		WARN_(SERIAL, "IoRegisterDeviceInterface() failed with status 0x%08x\n", Status);
// 		goto ByeBye;
// 	}
//
// 	DeviceExtension->SerialPortNumber = IoGetConfigurationInformation()->SerialCount++;
// 	if (pComPortNumber == NULL)
// 		DeviceExtension->ComPort = DeviceExtension->SerialPortNumber + 1;
// 	else
// 		DeviceExtension->ComPort = *pComPortNumber;
// 	DeviceExtension->Pdo = Pdo;
// 	DeviceExtension->PnpState = dsStopped;
// 	DeviceExtension->UartType = UartType;
// 	Status = InitializeCircularBuffer(&DeviceExtension->InputBuffer, 16);
// 	if (!NT_SUCCESS(Status)) goto ByeBye;
// 	Status = InitializeCircularBuffer(&DeviceExtension->OutputBuffer, 16);
// 	if (!NT_SUCCESS(Status)) goto ByeBye;
// 	IoInitializeRemoveLock(&DeviceExtension->RemoveLock, SERIAL_TAG, 0, 0);
// 	KeInitializeSpinLock(&DeviceExtension->InputBufferLock);
// 	KeInitializeSpinLock(&DeviceExtension->OutputBufferLock);
// 	KeInitializeEvent(&DeviceExtension->InputBufferNotEmpty, NotificationEvent, FALSE);
// 	KeInitializeDpc(&DeviceExtension->ReceivedByteDpc, SerialReceiveByte, DeviceExtension);
// 	KeInitializeDpc(&DeviceExtension->SendByteDpc, SerialSendByte, DeviceExtension);
// 	KeInitializeDpc(&DeviceExtension->CompleteIrpDpc, SerialCompleteIrp, DeviceExtension);
// 	Status = IoAttachDeviceToDeviceStackSafe(Fdo, Pdo, &DeviceExtension->LowerDevice);
// 	if (!NT_SUCCESS(Status))
// 	{
// 		WARN_(SERIAL, "IoAttachDeviceToDeviceStackSafe() failed with status 0x%08x\n", Status);
// 		goto ByeBye;
// 	}
// 	if (DeviceExtension->LowerDevice->Flags & DO_POWER_PAGABLE)
// 		Fdo->Flags |= DO_POWER_PAGABLE;
// 	if (DeviceExtension->LowerDevice->Flags & DO_BUFFERED_IO)
// 		Fdo->Flags |= DO_BUFFERED_IO;
// 	if (DeviceExtension->LowerDevice->Flags & DO_DIRECT_IO)
// 		Fdo->Flags |= DO_DIRECT_IO;
//
// 	/* Choose default strategy */
// 	if ((Fdo->Flags & (DO_BUFFERED_IO | DO_DIRECT_IO)) == 0)
// 		Fdo->Flags |= DO_BUFFERED_IO;
//
// 	Fdo->Flags &= ~DO_DEVICE_INITIALIZING;
// 	if (pFdo)
// 	{
// 		*pFdo = Fdo;
// 	}
//
// 	return STATUS_SUCCESS;
//
// ByeBye:
// 	if (Fdo)
// 	{
// 		FreeCircularBuffer(&DeviceExtension->InputBuffer);
// 		FreeCircularBuffer(&DeviceExtension->OutputBuffer);
// 		IoDeleteDevice(Fdo);
// 	}
// 	return Status;
// }
//
// NTSTATUS NTAPI
// SerialAddDevice(
// 	IN PDRIVER_OBJECT DriverObject,
// 	IN PDEVICE_OBJECT Pdo)
// {
// 	/* Serial.sys is a legacy driver. AddDevice is called once
// 	 * with a NULL Pdo just after the driver initialization.
// 	 * Detect this case and return success.
// 	 */
// 	if (Pdo == NULL)
// 		return STATUS_SUCCESS;
//
// 	/* We have here a PDO not null. It represents a real serial
// 	 * port. So call the internal AddDevice function.
// 	 */
// 	return SerialAddDeviceInternal(DriverObject, Pdo, UartUnknown, NULL, NULL);
// }
//
// NTSTATUS NTAPI
// SerialPnpStartDevice(
// 	IN PDEVICE_OBJECT DeviceObject,
// 	IN PCM_RESOURCE_LIST ResourceList,
// 	IN PCM_RESOURCE_LIST ResourceListTranslated)
// {
// 	PSERIAL_DEVICE_EXTENSION DeviceExtension;
// 	WCHAR DeviceNameBuffer[32];
// 	UNICODE_STRING DeviceName;
// 	WCHAR LinkNameBuffer[32];
// 	UNICODE_STRING LinkName;
// 	WCHAR ComPortBuffer[32];
// 	UNICODE_STRING ComPort;
// 	ULONG Vector = 0;
// 	ULONG i;
// 	UCHAR IER;
// 	KIRQL Dirql;
// 	KAFFINITY Affinity = 0;
// 	KINTERRUPT_MODE InterruptMode = Latched;
// 	BOOLEAN ShareInterrupt = TRUE;
// 	OBJECT_ATTRIBUTES objectAttributes;
// 	PUCHAR ComPortBase;
// 	UNICODE_STRING KeyName;
// 	HANDLE hKey;
// 	NTSTATUS Status;
//
// 	DeviceExtension = (PSERIAL_DEVICE_EXTENSION)DeviceObject->DeviceExtension;
//
// 	ASSERT(DeviceExtension);
//
// 	if (!ResourceList)
// 	{
// 		WARN_(SERIAL, "No allocated resources sent to driver\n");
// 		return STATUS_INSUFFICIENT_RESOURCES;
// 	}
// 	if (ResourceList->Count != 1)
// 	{
// 		WARN_(SERIAL, "Wrong number of allocated resources sent to driver\n");
// 		return STATUS_INSUFFICIENT_RESOURCES;
// 	}
// 	if (ResourceList->List[0].PartialResourceList.Version != 1
// 	 || ResourceList->List[0].PartialResourceList.Revision != 1
// 	 || ResourceListTranslated->List[0].PartialResourceList.Version != 1
// 	 || ResourceListTranslated->List[0].PartialResourceList.Revision != 1)
// 	{
// 		WARN_(SERIAL, "Revision mismatch: %u.%u != 1.1 or %u.%u != 1.1\n",
// 			ResourceList->List[0].PartialResourceList.Version,
// 			ResourceList->List[0].PartialResourceList.Revision,
// 			ResourceListTranslated->List[0].PartialResourceList.Version,
// 			ResourceListTranslated->List[0].PartialResourceList.Revision);
// 		return STATUS_REVISION_MISMATCH;
// 	}
//
// 	DeviceExtension->BaudRate = 19200;
// 	DeviceExtension->BaseAddress = 0;
// 	Dirql = 0;
// 	for (i = 0; i < ResourceList->List[0].PartialResourceList.Count; i++)
// 	{
// 		PCM_PARTIAL_RESOURCE_DESCRIPTOR PartialDescriptor = &ResourceList->List[0].PartialResourceList.PartialDescriptors[i];
// 		PCM_PARTIAL_RESOURCE_DESCRIPTOR PartialDescriptorTranslated = &ResourceListTranslated->List[0].PartialResourceList.PartialDescriptors[i];
// 		switch (PartialDescriptor->Type)
// 		{
// 			case CmResourceTypePort:
// 				if (PartialDescriptor->u.Port.Length < 7)
// 					return STATUS_INSUFFICIENT_RESOURCES;
// 				if (DeviceExtension->BaseAddress != 0)
// 					return STATUS_UNSUCCESSFUL;
// 				DeviceExtension->BaseAddress = PartialDescriptor->u.Port.Start.u.LowPart;
// 				break;
// 			case CmResourceTypeInterrupt:
// 				Dirql = (KIRQL)PartialDescriptorTranslated->u.Interrupt.Level;
// 				Vector = PartialDescriptorTranslated->u.Interrupt.Vector;
// 				Affinity = PartialDescriptorTranslated->u.Interrupt.Affinity;
// 				if (PartialDescriptorTranslated->Flags & CM_RESOURCE_INTERRUPT_LATCHED)
// 					InterruptMode = Latched;
// 				else
// 					InterruptMode = LevelSensitive;
// 				ShareInterrupt = (PartialDescriptorTranslated->ShareDisposition == CmResourceShareShared);
// 				break;
// 		}
// 	}
// 	INFO_(SERIAL, "New COM port. Base = 0x%lx, Irql = %u\n",
// 		DeviceExtension->BaseAddress, Dirql);
// 	if (!DeviceExtension->BaseAddress)
// 		return STATUS_INSUFFICIENT_RESOURCES;
// 	if (!Dirql)
// 		return STATUS_INSUFFICIENT_RESOURCES;
// 	ComPortBase = ULongToPtr(DeviceExtension->BaseAddress);
//
// 	/* Test if we are trying to start the serial port used for debugging */
//     INFO_(SERIAL, "Comparing addresses: KdComPortInUse: %p, ComPortBase: %p\n", KdComPortInUse, ComPortBase);
// 	if (KdComPortInUse == ComPortBase)
// 	{
// 		INFO_(SERIAL, "Failing IRP_MN_START_DEVICE as this serial port is used for debugging\n");
// 		return STATUS_INSUFFICIENT_RESOURCES;
// 	}
//
// 	if (DeviceExtension->UartType == UartUnknown)
// 		DeviceExtension->UartType = SerialDetectUartType(ComPortBase);
//
// 	/* Get current settings */
// 	DeviceExtension->MCR = READ_PORT_UCHAR(SER_MCR(ComPortBase));
// 	DeviceExtension->MSR = READ_PORT_UCHAR(SER_MSR(ComPortBase));
// 	DeviceExtension->WaitMask = 0;
//
// 	/* Set baud rate */
// 	Status = SerialSetBaudRate(DeviceExtension, DeviceExtension->BaudRate);
// 	if (!NT_SUCCESS(Status))
// 	{
// 		WARN_(SERIAL, "SerialSetBaudRate() failed with status 0x%08x\n", Status);
// 		return Status;
// 	}
//
// 	/* Set line control */
// 	DeviceExtension->SerialLineControl.StopBits = STOP_BIT_1;
// 	DeviceExtension->SerialLineControl.Parity = NO_PARITY;
// 	DeviceExtension->SerialLineControl.WordLength = 8;
// 	Status = SerialSetLineControl(DeviceExtension, &DeviceExtension->SerialLineControl);
// 	if (!NT_SUCCESS(Status))
// 	{
// 		WARN_(SERIAL, "SerialSetLineControl() failed with status 0x%08x\n", Status);
// 		return Status;
// 	}
//
// 	/* Clear receive/transmit buffers */
// 	if (DeviceExtension->UartType >= Uart16550A)
// 	{
// 		/* 16550 UARTs also have FIFO queues, but they are unusable due to a bug */
// 		WRITE_PORT_UCHAR(SER_FCR(ComPortBase),
// 			SR_FCR_CLEAR_RCVR | SR_FCR_CLEAR_XMIT);
// 	}
//
// 	/* Create link \DosDevices\COMX -> \Device\SerialX */
// 	swprintf(DeviceNameBuffer, L"\\Device\\Serial%lu", DeviceExtension->SerialPortNumber);
// 	swprintf(LinkNameBuffer, L"\\DosDevices\\COM%lu", DeviceExtension->ComPort);
// 	swprintf(ComPortBuffer, L"COM%lu", DeviceExtension->ComPort);
// 	RtlInitUnicodeString(&DeviceName, DeviceNameBuffer);
// 	RtlInitUnicodeString(&LinkName, LinkNameBuffer);
// 	RtlInitUnicodeString(&ComPort, ComPortBuffer);
// 	Status = IoCreateSymbolicLink(&LinkName, &DeviceName);
// 	if (!NT_SUCCESS(Status))
// 	{
// 		WARN_(SERIAL, "IoCreateSymbolicLink() failed with status 0x%08x\n", Status);
// 		return Status;
// 	}
//
// 	/* Connect interrupt and enable them */
// 	Status = IoConnectInterrupt(
// 		&DeviceExtension->Interrupt, SerialInterruptService,
// 		DeviceObject, NULL,
// 		Vector, Dirql, Dirql,
// 		InterruptMode, ShareInterrupt,
// 		Affinity, FALSE);
// 	if (!NT_SUCCESS(Status))
// 	{
// 		WARN_(SERIAL, "IoConnectInterrupt() failed with status 0x%08x\n", Status);
// 		IoSetDeviceInterfaceState(&DeviceExtension->SerialInterfaceName, FALSE);
// 		IoDeleteSymbolicLink(&LinkName);
// 		return Status;
// 	}
//
// 	/* Write an entry value under HKLM\HARDWARE\DeviceMap\SERIALCOMM */
// 	/* This step is not mandatory, so don't exit in case of error */
// 	RtlInitUnicodeString(&KeyName, L"\\Registry\\Machine\\HARDWARE\\DeviceMap\\SERIALCOMM");
// 	InitializeObjectAttributes(&objectAttributes, &KeyName, OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE, NULL, NULL);
// 	Status = ZwCreateKey(&hKey, KEY_SET_VALUE, &objectAttributes, 0, NULL, REG_OPTION_VOLATILE, NULL);
// 	if (NT_SUCCESS(Status))
// 	{
// 		/* Key = \Device\Serialx, Value = COMx */
// 		ZwSetValueKey(hKey, &DeviceName, 0, REG_SZ, ComPortBuffer, ComPort.Length + sizeof(WCHAR));
// 		ZwClose(hKey);
// 	}
//
// 	DeviceExtension->PnpState = dsStarted;
//
// 	/* Activate interrupt modes */
// 	IER = READ_PORT_UCHAR(SER_IER(ComPortBase));
// 	IER |= SR_IER_DATA_RECEIVED | SR_IER_THR_EMPTY | SR_IER_LSR_CHANGE | SR_IER_MSR_CHANGE;
// 	WRITE_PORT_UCHAR(SER_IER(ComPortBase), IER);
//
// 	/* Activate DTR, RTS */
// 	DeviceExtension->MCR |= SR_MCR_DTR | SR_MCR_RTS;
// 	WRITE_PORT_UCHAR(SER_MCR(ComPortBase), DeviceExtension->MCR);
//
// 	/* Activate serial interface */
// 	IoSetDeviceInterfaceState(&DeviceExtension->SerialInterfaceName, TRUE);
// 	/* We don't really care if the call succeeded or not... */
//
// 	return STATUS_SUCCESS;
// }
//
// NTSTATUS NTAPI
// SerialPnp(
// 	IN PDEVICE_OBJECT DeviceObject,
// 	IN PIRP Irp)
// {
// 	PSERIAL_DEVICE_EXTENSION DeviceExtension;
// 	ULONG MinorFunction;
// 	PIO_STACK_LOCATION Stack;
// 	ULONG_PTR Information = 0;
// 	NTSTATUS Status;
//
// 	Stack = IoGetCurrentIrpStackLocation(Irp);
// 	MinorFunction = Stack->MinorFunction;
//
// 	switch (MinorFunction)
// 	{
// 		/* FIXME: do all these minor functions
// 		IRP_MN_QUERY_REMOVE_DEVICE 0x1
// 		IRP_MN_REMOVE_DEVICE 0x2
// 		{
// 			TRACE_!(SERIAL, "IRP_MJ_PNP / IRP_MN_REMOVE_DEVICE\n");
// 			IoAcquireRemoveLock
// 			IoReleaseRemoveLockAndWait
// 			pass request to DeviceExtension-LowerDriver
// 			disable interface
// 			IoDeleteDevice(Fdo) and/or IoDetachDevice
// 			break;
// 		}
// 		IRP_MN_CANCEL_REMOVE_DEVICE 0x3
// 		IRP_MN_STOP_DEVICE 0x4
// 		IRP_MN_QUERY_STOP_DEVICE 0x5
// 		IRP_MN_CANCEL_STOP_DEVICE 0x6
// 		IRP_MN_QUERY_DEVICE_RELATIONS / BusRelations (optional) 0x7
// 		IRP_MN_QUERY_DEVICE_RELATIONS / RemovalRelations (optional) 0x7
// 		IRP_MN_QUERY_INTERFACE (optional) 0x8
// 		IRP_MN_QUERY_CAPABILITIES (optional) 0x9
// 		IRP_MN_FILTER_RESOURCE_REQUIREMENTS (optional) 0xd
// 		IRP_MN_QUERY_PNP_DEVICE_STATE (optional) 0x14
// 		IRP_MN_DEVICE_USAGE_NOTIFICATION (required or optional) 0x16
// 		IRP_MN_SURPRISE_REMOVAL 0x17
// 		*/
// 		case IRP_MN_START_DEVICE: /* 0x0 */
// 		{
// 			TRACE_!(SERIAL, "IRP_MJ_PNP / IRP_MN_START_DEVICE\n");
//
// 			/* Call lower driver */
// 			DeviceExtension = DeviceObject->DeviceExtension;
// 			Status = STATUS_UNSUCCESSFUL;
//
// 			ASSERT(DeviceExtension->PnpState == dsStopped);
//
// 			if (IoForwardIrpSynchronously(DeviceExtension->LowerDevice, Irp))
// 			{
// 				Status = Irp->IoStatus.Status;
// 				if (NT_SUCCESS(Status))
// 				{
// 					Status = SerialPnpStartDevice(
// 						DeviceObject,
// 						Stack->Parameters.StartDevice.AllocatedResources,
// 						Stack->Parameters.StartDevice.AllocatedResourcesTranslated);
// 				}
// 			}
//
// 			break;
// 		}
// 		case IRP_MN_QUERY_DEVICE_RELATIONS: /* (optional) 0x7 */
// 		{
// 			switch (Stack->Parameters.QueryDeviceRelations.Type)
// 			{
// 				case BusRelations:
// 				{
// 					TRACE_!(SERIAL, "IRP_MJ_PNP / IRP_MN_QUERY_DEVICE_RELATIONS / BusRelations\n");
// 					return ForwardIrpAndForget(DeviceObject, Irp);
// 				}
// 				case RemovalRelations:
// 				{
// 					TRACE_!(SERIAL, "IRP_MJ_PNP / IRP_MN_QUERY_DEVICE_RELATIONS / RemovalRelations\n");
// 					return ForwardIrpAndForget(DeviceObject, Irp);
// 				}
// 				default:
// 					TRACE_!(SERIAL, "IRP_MJ_PNP / IRP_MN_QUERY_DEVICE_RELATIONS / Unknown type 0x%lx\n",
// 						Stack->Parameters.QueryDeviceRelations.Type);
// 					return ForwardIrpAndForget(DeviceObject, Irp);
// 			}
// 			break;
// 		}
// 		case IRP_MN_FILTER_RESOURCE_REQUIREMENTS: /* (optional) 0xd */
// 		{
// 			TRACE_!(SERIAL, "IRP_MJ_PNP / IRP_MN_FILTER_RESOURCE_REQUIREMENTS\n");
// 			return ForwardIrpAndForget(DeviceObject, Irp);
// 		}
// 		default:
// 		{
// 			TRACE_!(SERIAL, "Unknown minor function 0x%x\n", MinorFunction);
// 			return ForwardIrpAndForget(DeviceObject, Irp);
// 		}
// 	}
//
// 	Irp->IoStatus.Information = Information;
// 	Irp->IoStatus.Status = Status;
// 	IoCompleteRequest(Irp, IO_NO_INCREMENT);
// 	return Status;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
