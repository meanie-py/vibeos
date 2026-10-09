use crate::types::*;
use crate::glue::*;
use crate::cb;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{DEVICE_RELATIONS, ExQueueWorkItem, _KWAIT_REASON_Executive, IoAcquireCancelSpinLock, IoAttachDeviceToDeviceStack, IoCreateDevice, IoFreeMdl, IoGetCurrentIrpStackLocation, IoReleaseCancelSpinLock, IoSetCancelRoutine, IoSkipCurrentIrpStackLocation, KEY_VALUE_FULL_INFORMATION, KeDelayExecutionThread, KeInitializeDpc, KeInitializeEvent, KeInitializeSemaphore, KeInitializeSpinLock, KeInitializeTimer, KeInsertQueueDpc, KeQueryTimeIncrement, KeReleaseSemaphore, KeSetTimer, KeWaitForSingleObject, LARGE_INTEGER, LIST_ENTRY, _EVENT_TYPE_NotificationEvent, PDEVICE_OBJECT, PDEVICE_RELATIONS, PDMA_ADAPTER, PDMA_OPERATIONS, PDRIVER_OBJECT, PHYSICAL_ADDRESS, PIO_STACK_LOCATION, PIRP, PKEY_VALUE_FULL_INFORMATION, PKINTERRUPT, PMDL, PRKDPC, PRKEVENT, PUNICODE_STRING, PsCreateSystemThread, PsTerminateSystemThread, _KWAIT_REASON_Suspended, ZwSetValueKey, ZwWaitForSingleObject};
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
    fn IoOpenDeviceRegistryKey( DeviceObject: PDEVICE_OBJECT, DevInstKeyType: ULONG, DesiredAccess: ACCESS_MASK, DevInstRegKey: *mut HANDLE, ) -> NTSTATUS;
    fn RtlInitUnicodeString(DestinationString: PUNICODE_STRING, SourceString: PCWSTR);
    fn ExAllocatePoolWithTag(PoolType: POOL_TYPE, NumberOfBytes: SIZE_T, Tag: ULONG) -> PVOID;
    fn ExFreePoolWithTag(P: PVOID, Tag: ULONG);
    fn ZwQueryValueKey( KeyHandle: HANDLE, ValueName: PUNICODE_STRING, KeyValueInformationClass: KEY_VALUE_INFORMATION_CLASS, KeyValueInformation: PVOID, Length: ULONG, ResultLength: *mut ULONG, ) -> NTSTATUS;
    fn ZwClose(Handle: HANDLE) -> NTSTATUS;
    fn RtlAssert( FailedAssertion: PVOID, FileName: PVOID, LineNumber: ULONG, Message: *mut CHAR, );
    fn USBPORT_FdoDeviceControl(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS;
    fn USBPORT_FdoInternalDeviceControl(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS;
    fn USBPORT_FdoPnP(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS;
    fn USBPORT_FdoPower(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS;
    fn USBPORT_PdoDeviceControl(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS;
    fn USBPORT_PdoInternalDeviceControl(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS;
    fn USBPORT_PdoPnP(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS;
    fn USBPORT_PdoPower(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS;
    fn USBPORT_FlushAllEndpoints(FdoDevice: PDEVICE_OBJECT);
    fn USBPORT_EndpointWorker(Endpoint: PUSBPORT_ENDPOINT, Flag: BOOLEAN) -> BOOLEAN;
    fn USBPORT_FlushClosedEndpointList(FdoDevice: PDEVICE_OBJECT);
    fn USB2_InitController(Usb2Extension: PUSB2_HC_EXTENSION);
    fn USBPORT_RootHubPowerAndChirpAllCcPorts(FdoDevice: PDEVICE_OBJECT);
    fn USBPORT_RemoveActiveTransferIrp(FdoDevice: PDEVICE_OBJECT, Irp: PIRP,) -> PIRP;
    fn USBPORT_DoSetPowerD0(FdoDevice: PDEVICE_OBJECT);
    fn USBPORT_DoneSplitTransfer(Transfer: PUSBPORT_TRANSFER);
    fn USBPORT_EndpointHasQueuedTransfers( FdoDevice: PDEVICE_OBJECT, Endpoint: PUSBPORT_ENDPOINT, TransferCount: *mut ULONG, ) -> BOOLEAN;
    fn USBPORT_InvalidateEndpointHandler( FdoDevice: PDEVICE_OBJECT, Endpoint: PUSBPORT_ENDPOINT, Type: ULONG, );
    fn USBPORT_GetEndpointState(Endpoint: PUSBPORT_ENDPOINT) -> u32;
    fn USBPORT_FlushPendingTransfers(Endpoint: PUSBPORT_ENDPOINT);
}
extern "C" {
    fn DbgPrint(Format: PCSTR, ...) -> ULONG;
    fn RtlStringCbPrintfW( pszDest: *mut u16, cbDest: usize, pszFormat: PCWSTR, ... ) -> NTSTATUS;
    fn _InterlockedCompareExchange( Destination: *mut core::ffi::c_long, Exchange: core::ffi::c_long, Comperand: core::ffi::c_long, ) -> core::ffi::c_long;
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static mut USBPORT_MiniPortDrivers: LIST_ENTRY = _LIST_ENTRY {
    Flink: ::core::ptr::null::<_LIST_ENTRY>() as *mut _LIST_ENTRY,
    Blink: ::core::ptr::null::<_LIST_ENTRY>() as *mut _LIST_ENTRY,
};

pub static mut USBPORT_USB1FdoList: LIST_ENTRY = _LIST_ENTRY {
    Flink: ::core::ptr::null::<_LIST_ENTRY>() as *mut _LIST_ENTRY,
    Blink: ::core::ptr::null::<_LIST_ENTRY>() as *mut _LIST_ENTRY,
};

pub static mut USBPORT_USB2FdoList: LIST_ENTRY = _LIST_ENTRY {
    Flink: ::core::ptr::null::<_LIST_ENTRY>() as *mut _LIST_ENTRY,
    Blink: ::core::ptr::null::<_LIST_ENTRY>() as *mut _LIST_ENTRY,
};

pub static mut USBPORT_SpinLock: KSPIN_LOCK = 0;

pub static mut USBPORT_Initialized: BOOLEAN = FALSE as BOOLEAN;

#[no_mangle]
pub unsafe extern "system" fn USBPORT_FindUSB2Controller(
    FdoDevice: PDEVICE_OBJECT,
) -> PDEVICE_OBJECT {
    let mut FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let mut USB2FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let mut OldIrql: KIRQL = 0;
    let mut USB2FdoEntry: PLIST_ENTRY;
    let mut USB2FdoDevice: PDEVICE_OBJECT = core::ptr::null_mut();

    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;

    KeAcquireSpinLock(
        core::ptr::addr_of_mut!(USBPORT_SpinLock),
        core::ptr::addr_of_mut!(OldIrql),
    );

    USB2FdoEntry = USBPORT_USB2FdoList.Flink as PLIST_ENTRY;

    while !USB2FdoEntry.is_null()
        && USB2FdoEntry != core::ptr::addr_of_mut!(USBPORT_USB2FdoList)
    {
        USB2FdoExtension = ((USB2FdoEntry as usize)
            - (core::mem::offset_of!(USBPORT_DEVICE_EXTENSION, ControllerLink))) as *mut USBPORT_DEVICE_EXTENSION
            as PUSBPORT_DEVICE_EXTENSION;

        if (*USB2FdoExtension).BusNumber == (*FdoExtension).BusNumber
            && (*USB2FdoExtension).PciDeviceNumber == (*FdoExtension).PciDeviceNumber
        {
            USB2FdoDevice = (*USB2FdoExtension).CommonExtension.SelfDevice;
            break;
        }

        USB2FdoEntry = (*USB2FdoEntry).Flink as PLIST_ENTRY;
    }

    KeReleaseSpinLock(core::ptr::addr_of_mut!(USBPORT_SpinLock), OldIrql);

    return USB2FdoDevice;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_AddUSB1Fdo(FdoDevice: PDEVICE_OBJECT) {
    let mut FdoExtension: PUSBPORT_DEVICE_EXTENSION = core::ptr::null_mut();

    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;
    (*FdoExtension).Flags |= USBPORT_FLAG_REGISTERED_FDO as u32;

    ExInterlockedInsertTailList(
        core::ptr::addr_of_mut!(USBPORT_USB1FdoList),
        core::ptr::addr_of_mut!((*FdoExtension).ControllerLink),
        (core::ptr::addr_of_mut!(USBPORT_SpinLock) as *mut _),
    );
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_AddUSB2Fdo(FdoDevice: PDEVICE_OBJECT) {
    let mut FdoExtension: PUSBPORT_DEVICE_EXTENSION = core::ptr::null_mut();

    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;
    (*FdoExtension).Flags |= USBPORT_FLAG_REGISTERED_FDO as u32;

    ExInterlockedInsertTailList(
        core::ptr::addr_of_mut!(USBPORT_USB2FdoList),
        core::ptr::addr_of_mut!((*FdoExtension).ControllerLink),
        (core::ptr::addr_of_mut!(USBPORT_SpinLock) as *mut _),
    );
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_RemoveUSBxFdo(FdoDevice: PDEVICE_OBJECT) {
    let mut FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let mut OldIrql: KIRQL = 0;

    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;

    KeAcquireSpinLock(
        core::ptr::addr_of_mut!(USBPORT_SpinLock),
        core::ptr::addr_of_mut!(OldIrql),
    );
    RemoveEntryList(core::ptr::addr_of_mut!((*FdoExtension).ControllerLink));
    KeReleaseSpinLock(core::ptr::addr_of_mut!(USBPORT_SpinLock), OldIrql);

    (*FdoExtension).Flags &= !(USBPORT_FLAG_REGISTERED_FDO as u32);

    (*FdoExtension).ControllerLink.Flink = core::ptr::null_mut();
    (*FdoExtension).ControllerLink.Blink = core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_IsCompanionFdoExtension(
    USB2FdoDevice: PDEVICE_OBJECT,
    USB1FdoExtension: PUSBPORT_DEVICE_EXTENSION,
) -> BOOLEAN {
    let mut USB2FdoExtension: PUSBPORT_DEVICE_EXTENSION = core::ptr::null_mut();

    USB2FdoExtension = (*USB2FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;

    return ((*USB2FdoExtension).BusNumber == (*USB1FdoExtension).BusNumber
        && (*USB2FdoExtension).PciDeviceNumber == (*USB1FdoExtension).PciDeviceNumber)
        as core::ffi::c_int as BOOLEAN;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_FindCompanionControllers(
    USB2FdoDevice: PDEVICE_OBJECT,
    IsObRefer: BOOLEAN,
    IsFDOsReturned: BOOLEAN,
) -> PDEVICE_RELATIONS {
    let mut USB1FdoList: PLIST_ENTRY;
    let mut USB1FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let mut NumControllers: ULONG = 0;
    let mut Entry: *mut PDEVICE_OBJECT;
    let mut ControllersList: PDEVICE_RELATIONS = core::ptr::null_mut();
    let mut OldIrql: KIRQL = 0;

    KeAcquireSpinLock(
        core::ptr::addr_of_mut!(USBPORT_SpinLock),
        core::ptr::addr_of_mut!(OldIrql),
    );

    USB1FdoList = USBPORT_USB1FdoList.Flink as PLIST_ENTRY;

    while !USB1FdoList.is_null() && USB1FdoList != core::ptr::addr_of_mut!(USBPORT_USB1FdoList) {
        USB1FdoExtension = (USB1FdoList as usize).wrapping_sub(
            core::mem::offset_of!(USBPORT_DEVICE_EXTENSION, ControllerLink),
        ) as PUSBPORT_DEVICE_EXTENSION;

        if (*USB1FdoExtension).Flags & USBPORT_FLAG_COMPANION_HC != 0
            && USBPORT_IsCompanionFdoExtension(USB2FdoDevice, USB1FdoExtension) != 0
        {
            NumControllers += 1;
        }

        USB1FdoList = (*USB1FdoExtension).ControllerLink.Flink as PLIST_ENTRY;
    }

    if NumControllers == 0 {
        KeReleaseSpinLock(core::ptr::addr_of_mut!(USBPORT_SpinLock), OldIrql);
        return ControllersList;
    }

    ControllersList = ExAllocatePoolWithTag(
        NonPagedPool,
        (NumControllers as SIZE_T)
            .wrapping_mul(core::mem::size_of::<DEVICE_RELATIONS>() as SIZE_T),
        USB_PORT_TAG as ULONG,
    ) as PDEVICE_RELATIONS;

    if ControllersList.is_null() {
        KeReleaseSpinLock(core::ptr::addr_of_mut!(USBPORT_SpinLock), OldIrql);
        return ControllersList;
    }

    RtlZeroMemory(
        ControllersList as PVOID,
        (NumControllers as SIZE_T)
            .wrapping_mul(core::mem::size_of::<DEVICE_RELATIONS>() as SIZE_T),
    );

    (*ControllersList).Count = NumControllers;

    USB1FdoList = USBPORT_USB1FdoList.Flink as PLIST_ENTRY;

    Entry = (core::ptr::addr_of_mut!((*ControllersList).Objects) as *mut PDEVICE_OBJECT)
        .offset(0);

    while !USB1FdoList.is_null() && USB1FdoList != core::ptr::addr_of_mut!(USBPORT_USB1FdoList) {
        USB1FdoExtension = (USB1FdoList as usize).wrapping_sub(
            core::mem::offset_of!(USBPORT_DEVICE_EXTENSION, ControllerLink),
        ) as PUSBPORT_DEVICE_EXTENSION;

        if (*USB1FdoExtension).Flags & USBPORT_FLAG_COMPANION_HC != 0
            && USBPORT_IsCompanionFdoExtension(USB2FdoDevice, USB1FdoExtension) != 0
        {
            *Entry = (*USB1FdoExtension).CommonExtension.LowerPdoDevice;

            if IsObRefer != 0 {
                ObReferenceObject(
                    (*USB1FdoExtension).CommonExtension.LowerPdoDevice as PVOID,
                );
            }

            if IsFDOsReturned != 0 {
                *Entry = (*USB1FdoExtension).CommonExtension.SelfDevice;
            }

            Entry = Entry.offset(1);
        }

        USB1FdoList = (*USB1FdoExtension).ControllerLink.Flink as PLIST_ENTRY;
    }

    KeReleaseSpinLock(core::ptr::addr_of_mut!(USBPORT_SpinLock), OldIrql);

    return ControllersList;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_NtStatusToMpStatus(NtStatus: NTSTATUS) -> MPSTATUS {
    if NtStatus == STATUS_SUCCESS {
        return MP_STATUS_SUCCESS as MPSTATUS;
    } else {
        return MP_STATUS_UNSUCCESSFUL as MPSTATUS;
    }
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_SetRegistryKeyValue(
    DeviceObject: PDEVICE_OBJECT,
    UseDriverKey: BOOL,
    Type: ULONG,
    ValueNameString: PCWSTR,
    Data: PVOID,
    DataSize: ULONG,
) -> NTSTATUS {
    const PLUGPLAY_REGKEY_DEVICE: ULONG = 0x00000001;
    const PLUGPLAY_REGKEY_DRIVER: ULONG = 0x00000002;

    let mut ValueName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut KeyHandle: HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    if UseDriverKey != 0 {
        Status = IoOpenDeviceRegistryKey(
            DeviceObject,
            PLUGPLAY_REGKEY_DRIVER,
            STANDARD_RIGHTS_ALL as ACCESS_MASK,
            core::ptr::addr_of_mut!(KeyHandle),
        );
    } else {
        Status = IoOpenDeviceRegistryKey(
            DeviceObject,
            PLUGPLAY_REGKEY_DEVICE,
            STANDARD_RIGHTS_ALL as ACCESS_MASK,
            core::ptr::addr_of_mut!(KeyHandle),
        );
    }

    if NT_SUCCESS(Status) {
        RtlInitUnicodeString(core::ptr::addr_of_mut!(ValueName), ValueNameString);

        Status = ZwSetValueKey(
            KeyHandle,
            core::ptr::addr_of_mut!(ValueName),
            0 as ULONG,
            Type,
            Data,
            DataSize,
        );

        ZwClose(KeyHandle);
    }

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_GetRegistryKeyValueFullInfo(
    mut FdoDevice: PDEVICE_OBJECT,
    mut PdoDevice: PDEVICE_OBJECT,
    mut UseDriverKey: BOOL,
    mut SourceString: PCWSTR,
    mut LengthStr: ULONG,
    mut Buffer: PVOID,
    mut BufferLength: ULONG,
) -> NTSTATUS {
    const PLUGPLAY_REGKEY_DEVICE: u32 = 0x00000001;
    const PLUGPLAY_REGKEY_DRIVER: u32 = 0x00000002;
    const USB_PORT_TAG: u32 = 0x70627375; // 'UBrt'

    let mut Status: NTSTATUS;
    let mut KeyValue: PKEY_VALUE_FULL_INFORMATION = core::ptr::null_mut();
    let mut ValueName: _UNICODE_STRING = core::mem::zeroed();
    let mut KeyHandle: HANDLE = core::ptr::null_mut();
    let mut LengthKey: ULONG;

    if UseDriverKey != 0 {
        Status = IoOpenDeviceRegistryKey(
            PdoDevice,
            PLUGPLAY_REGKEY_DRIVER as ULONG,
            STANDARD_RIGHTS_ALL as ACCESS_MASK,
            core::ptr::addr_of_mut!(KeyHandle),
        );
    } else {
        Status = IoOpenDeviceRegistryKey(
            PdoDevice,
            PLUGPLAY_REGKEY_DEVICE as ULONG,
            STANDARD_RIGHTS_ALL as ACCESS_MASK,
            core::ptr::addr_of_mut!(KeyHandle),
        );
    }

    if NT_SUCCESS(Status) {
        RtlInitUnicodeString(core::ptr::addr_of_mut!(ValueName), SourceString);

        LengthKey = (core::mem::size_of::<KEY_VALUE_FULL_INFORMATION>() as ULONG)
            .wrapping_add(LengthStr)
            .wrapping_add(BufferLength);

        KeyValue = ExAllocatePoolWithTag(PagedPool, LengthKey as SIZE_T, USB_PORT_TAG as ULONG)
            as PKEY_VALUE_FULL_INFORMATION;

        if !KeyValue.is_null() {
            RtlZeroMemory(KeyValue as PVOID, LengthKey as SIZE_T);

            Status = ZwQueryValueKey(
                KeyHandle,
                core::ptr::addr_of_mut!(ValueName),
                KeyValueFullInformation,
                KeyValue as PVOID,
                LengthKey,
                core::ptr::addr_of_mut!(LengthKey),
            );

            if NT_SUCCESS(Status) {
                RtlCopyMemory(
                    Buffer,
                    (KeyValue as PUCHAR).offset((*KeyValue).DataOffset as isize) as PVOID,
                    BufferLength as SIZE_T,
                );
            }

            ExFreePoolWithTag(KeyValue as PVOID, USB_PORT_TAG as ULONG);
        }

        ZwClose(KeyHandle);
    }

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_GetMiniportRegistryKeyValue(
    mut MiniPortExtension: PVOID,
    mut UseDriverKey: BOOL,
    mut SourceString: PCWSTR,
    mut LengthStr: SIZE_T,
    mut Buffer: PVOID,
    mut BufferLength: SIZE_T,
) -> MPSTATUS {
    let mut FdoExtension: PUSBPORT_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut FdoDevice: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    FdoExtension = (MiniPortExtension as usize)
        .wrapping_sub(core::mem::size_of::<USBPORT_DEVICE_EXTENSION>())
        as PUSBPORT_DEVICE_EXTENSION;

    FdoDevice = (*FdoExtension).CommonExtension.SelfDevice;

    Status = USBPORT_GetRegistryKeyValueFullInfo(
        FdoDevice,
        (*FdoExtension).CommonExtension.LowerPdoDevice,
        UseDriverKey,
        SourceString,
        LengthStr as ULONG,
        Buffer,
        BufferLength as ULONG,
    );

    return USBPORT_NtStatusToMpStatus(Status);
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_GetSetConfigSpaceData(
    FdoDevice: PDEVICE_OBJECT,
    IsReadData: BOOLEAN,
    Buffer: PVOID,
    Offset: ULONG,
    Length: ULONG,
) -> NTSTATUS {
    const PCI_WHICHSPACE_CONFIG: ULONG = 0;

    let FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let mut BytesReadWrite: ULONG;

    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;

    BytesReadWrite = Length;

    if IsReadData != 0 {
        RtlZeroMemory(Buffer, Length as usize);

        BytesReadWrite = ((*FdoExtension).BusInterface.GetBusData.expect("non-null function pointer"))(
            (*FdoExtension).BusInterface.Context,
            PCI_WHICHSPACE_CONFIG,
            Buffer,
            Offset,
            Length,
        );
    } else {
        BytesReadWrite = ((*FdoExtension).BusInterface.SetBusData.expect("non-null function pointer"))(
            (*FdoExtension).BusInterface.Context,
            PCI_WHICHSPACE_CONFIG,
            Buffer,
            Offset,
            Length,
        );
    }

    if BytesReadWrite == Length {
        return STATUS_SUCCESS;
    }

    return STATUS_UNSUCCESSFUL;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_ReadWriteConfigSpace(
    MiniPortExtension: PVOID,
    IsReadData: BOOLEAN,
    Buffer: PVOID,
    Offset: ULONG,
    Length: ULONG,
) -> MPSTATUS {
    let mut Status: NTSTATUS;
    let mut FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let mut FdoDevice: PDEVICE_OBJECT;

    FdoExtension = (MiniPortExtension as usize)
        .wrapping_sub(core::mem::size_of::<USBPORT_DEVICE_EXTENSION>())
        as PUSBPORT_DEVICE_EXTENSION;

    FdoDevice = (*FdoExtension).CommonExtension.SelfDevice;

    Status = USBPORT_GetSetConfigSpaceData(FdoDevice, IsReadData, Buffer, Offset, Length);

    return USBPORT_NtStatusToMpStatus(Status);
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_USBDStatusToNtStatus(
    Urb: PURB,
    USBDStatus: USBD_STATUS,
) -> NTSTATUS {
    let mut Status: NTSTATUS;

    if (USBDStatus as ::core::ffi::c_long) < 0 {
        // DPRINT1("USBPORT_USBDStatusToNtStatus: Urb - %p, USBDStatus - %x\n", Urb, USBDStatus);
    }

    if !Urb.is_null() {
        (*Urb).__bindgen_anon_1.UrbHeader.Status = USBDStatus;
    }

    match USBDStatus {
        USBD_STATUS_SUCCESS => {
            Status = STATUS_SUCCESS;
        }
        USBD_STATUS_INSUFFICIENT_RESOURCES => {
            Status = STATUS_INSUFFICIENT_RESOURCES;
        }
        USBD_STATUS_DEVICE_GONE => {
            Status = STATUS_DEVICE_NOT_CONNECTED;
        }
        USBD_STATUS_CANCELED => {
            Status = STATUS_CANCELLED;
        }
        USBD_STATUS_NOT_SUPPORTED => {
            Status = STATUS_NOT_SUPPORTED;
        }
        USBD_STATUS_INVALID_URB_FUNCTION
        | USBD_STATUS_INVALID_PARAMETER
        | USBD_STATUS_INVALID_PIPE_HANDLE
        | USBD_STATUS_BAD_START_FRAME => {
            Status = STATUS_INVALID_PARAMETER;
        }
        _ => {
            if (USBDStatus as ::core::ffi::c_long) < 0 {
                Status = STATUS_UNSUCCESSFUL;
            } else {
                Status = STATUS_SUCCESS;
            }
        }
    }

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_Wait(
    MiniPortExtension: PVOID,
    Milliseconds: ULONG,
) -> NTSTATUS {
    let mut Interval: LARGE_INTEGER = core::mem::zeroed();
    Interval.QuadPart -= (10000u32)
        .wrapping_mul(Milliseconds)
        .wrapping_add((KeQueryTimeIncrement() as u32).wrapping_sub(1)) as i64;
    return KeDelayExecutionThread(0i32 as i8, 0, core::ptr::addr_of_mut!(Interval));
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_MiniportInterrupts(
    FdoDevice: PDEVICE_OBJECT,
    IsEnable: BOOLEAN,
) {
    let mut FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let mut Packet: PUSBPORT_REGISTRATION_PACKET;
    let IsLock: BOOLEAN;
    let mut OldIrql: KIRQL = 0;

    DPRINT_INT(
        b"USBPORT_MiniportInterrupts: IsEnable - %p\n\0".as_ptr() as *const i8,
        (IsEnable as ::core::ffi::c_int),
    );

    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;
    Packet = core::ptr::addr_of_mut!((*(*FdoExtension).MiniPortInterface).Packet)
        as PUSBPORT_REGISTRATION_PACKET;

    IsLock = (((*Packet).MiniPortFlags as u32 & USB_MINIPORT_FLAGS_NOT_LOCK_INT as u32) == 0)
        as i32 as BOOLEAN;

    if IsLock != 0 {
        KeAcquireSpinLock(
            (core::ptr::addr_of_mut!((*FdoExtension).MiniportSpinLock) as *mut usize),
            core::ptr::addr_of_mut!(OldIrql),
        );
    }

    if IsEnable != 0 {
        (*FdoExtension).Flags |= USBPORT_FLAG_INTERRUPT_ENABLED as u32;
        ((*Packet).EnableInterrupts.expect("non-null function pointer"))(
            (*FdoExtension).MiniPortExt,
        );
    } else {
        ((*Packet).DisableInterrupts.expect("non-null function pointer"))(
            (*FdoExtension).MiniPortExt,
        );
        (*FdoExtension).Flags &= !(USBPORT_FLAG_INTERRUPT_ENABLED as u32);
    }

    if IsLock != 0 {
        KeReleaseSpinLock(
            (core::ptr::addr_of_mut!((*FdoExtension).MiniportSpinLock) as *mut usize),
            OldIrql,
        );
    }
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_SoftInterruptDpc(
    mut Dpc: PRKDPC,
    mut DeferredContext: PVOID,
    mut SystemArgument1: PVOID,
    mut SystemArgument2: PVOID,
) {
    let mut FdoDevice: PDEVICE_OBJECT;
    let mut FdoExtension: PUSBPORT_DEVICE_EXTENSION;

    DPRINT_INT(b"USBPORT_SoftInterruptDpc: ...\n\0".as_ptr() as *const i8);

    FdoDevice = DeferredContext as PDEVICE_OBJECT;
    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;

    if KeInsertQueueDpc(
        core::ptr::addr_of_mut!((*FdoExtension).IsrDpc),
        core::ptr::null_mut(),
        1 as PVOID,
    ) == 0
    {
        InterlockedDecrement(
            core::ptr::addr_of_mut!((*FdoExtension).IsrDpcCounter) as *mut core::ffi::c_long,
        );
    }
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_SoftInterrupt(FdoDevice: PDEVICE_OBJECT) {
    let mut FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let mut DueTime: LARGE_INTEGER = core::mem::zeroed();

    DPRINT_INT(b"USBPORT_SoftInterrupt: ...\n\0".as_ptr() as *const i8);

    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;

    KeInitializeTimer(core::ptr::addr_of_mut!((*FdoExtension).TimerSoftInterrupt));

    KeInitializeDpc(
        core::ptr::addr_of_mut!((*FdoExtension).SoftInterruptDpc),
        cb!(USBPORT_SoftInterruptDpc),
        FdoDevice as *mut core::ffi::c_void,
    );

    DueTime.QuadPart -= (10000u32).wrapping_add(
        (KeQueryTimeIncrement() as u32).wrapping_sub(1u32),
    ) as i64;

    KeSetTimer(
        core::ptr::addr_of_mut!((*FdoExtension).TimerSoftInterrupt),
        DueTime,
        core::ptr::addr_of_mut!((*FdoExtension).SoftInterruptDpc),
    );
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_InvalidateControllerHandler(
    FdoDevice: PDEVICE_OBJECT,
    Type: ULONG,
) {
    const USBPORT_INVALIDATE_CONTROLLER_RESET: ULONG = 1;
    const USBPORT_INVALIDATE_CONTROLLER_SURPRISE_REMOVE: ULONG = 2;
    const USBPORT_INVALIDATE_CONTROLLER_SOFT_INTERRUPT: ULONG = 3;

    let mut FdoExtension: PUSBPORT_DEVICE_EXTENSION;

    // DPRINT_CORE("USBPORT_InvalidateControllerHandler: Invalidate Type - %x\n", Type);

    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;

    match Type {
        USBPORT_INVALIDATE_CONTROLLER_RESET => {
            DbgPrint(
                b"USBPORT_InvalidateControllerHandler: INVALIDATE_CONTROLLER_RESET UNIMPLEMENTED. FIXME.\n\0"
                    as *const u8 as PCSTR,
            );
        }
        USBPORT_INVALIDATE_CONTROLLER_SURPRISE_REMOVE => {
            DbgPrint(
                b"USBPORT_InvalidateControllerHandler: INVALIDATE_CONTROLLER_SURPRISE_REMOVE UNIMPLEMENTED. FIXME.\n\0"
                    as *const u8 as PCSTR,
            );
        }
        USBPORT_INVALIDATE_CONTROLLER_SOFT_INTERRUPT => {
            if InterlockedIncrement(core::ptr::addr_of_mut!((*FdoExtension).IsrDpcCounter)) != 0 {
                InterlockedDecrement(core::ptr::addr_of_mut!((*FdoExtension).IsrDpcCounter));
            } else {
                USBPORT_SoftInterrupt(FdoDevice);
            }
        }
        _ => {}
    }
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_InvalidateController(
    MiniPortExtension: PVOID,
    Type: ULONG,
) -> ULONG {
    let mut FdoExtension: PUSBPORT_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut FdoDevice: PDEVICE_OBJECT;

    FdoExtension = (MiniPortExtension as usize)
        .wrapping_sub(core::mem::size_of::<USBPORT_DEVICE_EXTENSION>())
        as PUSBPORT_DEVICE_EXTENSION;
    FdoDevice = (*FdoExtension).CommonExtension.SelfDevice;

    USBPORT_InvalidateControllerHandler(FdoDevice, Type);

    return 0 as ULONG;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_NotifyDoubleBuffer(
    mut MiniPortExtension: PVOID,
    mut MiniPortTransfer: PVOID,
    mut Buffer: PVOID,
    mut Length: SIZE_T,
) -> ULONG {
    if DbgPrint(
        b"(%s:%d) USBPORT_NotifyDoubleBuffer: UNIMPLEMENTED. FIXME.\n\0" as *const u8 as PCSTR,
        b"drivers/usb/usbport/usbport.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        695 as ::core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
            b"drivers/usb/usbport/usbport.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            695 as ::core::ffi::c_int,
        );
    }
    return 0 as ULONG;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_WorkerRequestDpc(
    Dpc: PRKDPC,
    DeferredContext: PVOID,
    SystemArgument1: PVOID,
    SystemArgument2: PVOID,
) {
    let mut FdoDevice: PDEVICE_OBJECT;
    let mut FdoExtension: PUSBPORT_DEVICE_EXTENSION;

    FdoDevice = DeferredContext as PDEVICE_OBJECT;
    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;

    if InterlockedIncrement(
        core::ptr::addr_of_mut!((*FdoExtension).IsrDpcHandlerCounter) as *mut core::ffi::c_long,
    ) == 0
    {
        USBPORT_DpcHandler(FdoDevice);
    }

    InterlockedDecrement(
        core::ptr::addr_of_mut!((*FdoExtension).IsrDpcHandlerCounter) as *mut core::ffi::c_long,
    );
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_DoneTransfer(Transfer: PUSBPORT_TRANSFER) {

    let Endpoint: PUSBPORT_ENDPOINT;
    let FdoDevice: PDEVICE_OBJECT;
    let FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let Urb: PURB;
    let Irp: PIRP;
    let mut CancelIrql: KIRQL = 0;
    let mut OldIrql: KIRQL = 0;

    // DPRINT_CORE("USBPORT_DoneTransfer: Transfer - %p\n", Transfer);

    Endpoint = (*Transfer).Endpoint;
    FdoDevice = (*Endpoint).FdoDevice;
    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;

    Urb = (*Transfer).Urb;
    Irp = (*Transfer).Irp;

    KeAcquireSpinLock(
        (core::ptr::addr_of_mut!((*FdoExtension).FlushTransferSpinLock) as *mut usize),
        core::ptr::addr_of_mut!(OldIrql),
    );

    if !Irp.is_null() {
        IoAcquireCancelSpinLock(core::ptr::addr_of_mut!(CancelIrql));
        IoSetCancelRoutine(Irp, None);
        IoReleaseCancelSpinLock(CancelIrql);

        USBPORT_RemoveActiveTransferIrp(FdoDevice, Irp);
    }

    KeReleaseSpinLock(
        (core::ptr::addr_of_mut!((*FdoExtension).FlushTransferSpinLock) as *mut usize),
        OldIrql,
    );

    USBPORT_USBDStatusToNtStatus((*Transfer).Urb, (*Transfer).USBDStatus);
    USBPORT_CompleteTransfer(Urb, (*Urb).__bindgen_anon_1.UrbHeader.Status);

    // DPRINT_CORE("USBPORT_DoneTransfer: exit\n");
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_FlushDoneTransfers(FdoDevice: PDEVICE_OBJECT) {

    let FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let DoneTransferList: PLIST_ENTRY;
    let mut Transfer: PUSBPORT_TRANSFER;
    let mut Endpoint: PUSBPORT_ENDPOINT;
    let mut TransferCount: ULONG = 0;
    let mut OldIrql: KIRQL = 0;
    let mut IsHasTransfers: BOOLEAN;

    DPRINT_CORE(b"USBPORT_FlushDoneTransfers: ...\n\0".as_ptr() as *const i8);

    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;
    DoneTransferList = core::ptr::addr_of_mut!((*FdoExtension).DoneTransferList) as PLIST_ENTRY;

    while 1 != 0 {
        KeAcquireSpinLock(
            (core::ptr::addr_of_mut!((*FdoExtension).DoneTransferSpinLock) as *mut usize),
            core::ptr::addr_of_mut!(OldIrql),
        );

        if IsListEmpty(DoneTransferList) {
            break;
        }

        Transfer = ((*DoneTransferList).Flink as usize)
            .wrapping_sub(
                core::mem::offset_of!(USBPORT_TRANSFER, TransferLink),
            ) as *mut USBPORT_TRANSFER as PUSBPORT_TRANSFER;

        RemoveHeadList(DoneTransferList);
        KeReleaseSpinLock(
            (core::ptr::addr_of_mut!((*FdoExtension).DoneTransferSpinLock) as *mut usize),
            OldIrql,
        );

        if !Transfer.is_null() {
            Endpoint = (*Transfer).Endpoint;

            if (*Transfer).Flags & TRANSFER_FLAG_SPLITED != 0 {
                USBPORT_DoneSplitTransfer(Transfer);
            } else {
                USBPORT_DoneTransfer(Transfer);
            }

            IsHasTransfers = USBPORT_EndpointHasQueuedTransfers(
                FdoDevice,
                Endpoint,
                core::ptr::addr_of_mut!(TransferCount),
            );

            if IsHasTransfers != 0 && TransferCount == 0 {
                USBPORT_InvalidateEndpointHandler(
                    FdoDevice,
                    Endpoint,
                    INVALIDATE_ENDPOINT_WORKER_DPC as ULONG,
                );
            }
        }
    }

    KeReleaseSpinLock(
        (core::ptr::addr_of_mut!((*FdoExtension).DoneTransferSpinLock) as *mut usize),
        OldIrql,
    );
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_TransferFlushDpc(
    Dpc: PRKDPC,
    DeferredContext: PVOID,
    SystemArgument1: PVOID,
    SystemArgument2: PVOID,
) {
    let mut FdoDevice: PDEVICE_OBJECT;

    // DPRINT_CORE("USBPORT_TransferFlushDpc: ...\n");
    FdoDevice = DeferredContext as PDEVICE_OBJECT;
    USBPORT_FlushDoneTransfers(FdoDevice);
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_QueueDoneTransfer(
    Transfer: PUSBPORT_TRANSFER,
    USBDStatus: USBD_STATUS,
) -> BOOLEAN {
    let FdoDevice: PDEVICE_OBJECT;
    let FdoExtension: PUSBPORT_DEVICE_EXTENSION;

    DPRINT_CORE(
        b"USBPORT_QueueDoneTransfer: Transfer - %p, USBDStatus - %p\n\0".as_ptr() as *const i8,
        Transfer,
        USBDStatus,
    );

    FdoDevice = (*(*Transfer).Endpoint).FdoDevice;
    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;

    RemoveEntryList(core::ptr::addr_of_mut!((*Transfer).TransferLink));
    (*Transfer).USBDStatus = USBDStatus;

    ExInterlockedInsertTailList(
        core::ptr::addr_of_mut!((*FdoExtension).DoneTransferList),
        core::ptr::addr_of_mut!((*Transfer).TransferLink),
        core::ptr::addr_of_mut!((*FdoExtension).DoneTransferSpinLock),
    );

    return KeInsertQueueDpc(
        core::ptr::addr_of_mut!((*FdoExtension).TransferFlushDpc),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    ) as BOOLEAN;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_DpcHandler(FdoDevice: PDEVICE_OBJECT) {

    let mut FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let mut Endpoint: PUSBPORT_ENDPOINT;
    let mut Entry: PLIST_ENTRY;
    let mut List: LIST_ENTRY = LIST_ENTRY {
        Flink: core::ptr::null_mut(),
        Blink: core::ptr::null_mut(),
    };
    let mut LockCounter: LONG;

    DPRINT_CORE(b"USBPORT_DpcHandler: ...\n\0".as_ptr() as *const i8);

    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;

    InitializeListHead(core::ptr::addr_of_mut!(List));

    KeAcquireSpinLockAtDpcLevel((core::ptr::addr_of_mut!((*FdoExtension).EndpointListSpinLock) as *mut usize));
    Entry = (*FdoExtension).EndpointList.Flink as PLIST_ENTRY;

    while !Entry.is_null() && Entry != core::ptr::addr_of_mut!((*FdoExtension).EndpointList) {
        Endpoint = (Entry as usize).wrapping_sub(
            core::mem::offset_of!(USBPORT_ENDPOINT, EndpointLink),
        ) as *mut USBPORT_ENDPOINT as PUSBPORT_ENDPOINT;

        LockCounter = InterlockedIncrement(
            core::ptr::addr_of_mut!((*Endpoint).LockCounter) as *mut core::ffi::c_long,
        ) as LONG;

        if USBPORT_GetEndpointState(Endpoint) != USBPORT_ENDPOINT_ACTIVE
            || LockCounter != 0
            || ((*Endpoint).Flags as u32 & ENDPOINT_FLAG_ROOTHUB_EP0 as u32) != 0
        {
            InterlockedDecrement(
                core::ptr::addr_of_mut!((*Endpoint).LockCounter) as *mut core::ffi::c_long,
            );
        } else {
            InsertTailList(
                core::ptr::addr_of_mut!(List),
                core::ptr::addr_of_mut!((*Endpoint).DispatchLink),
            );

            if !(*Endpoint).WorkerLink.Flink.is_null() && !(*Endpoint).WorkerLink.Blink.is_null() {
                RemoveEntryList(core::ptr::addr_of_mut!((*Endpoint).WorkerLink));

                (*Endpoint).WorkerLink.Flink = core::ptr::null_mut();
                (*Endpoint).WorkerLink.Blink = core::ptr::null_mut();
            }
        }

        Entry = (*Endpoint).EndpointLink.Flink as PLIST_ENTRY;
    }

    KeReleaseSpinLockFromDpcLevel((core::ptr::addr_of_mut!((*FdoExtension).EndpointListSpinLock) as *mut usize));

    while !IsListEmpty(core::ptr::addr_of_mut!(List)) {
        Endpoint = (List.Flink as usize).wrapping_sub(
            core::mem::offset_of!(USBPORT_ENDPOINT, DispatchLink),
        ) as *mut USBPORT_ENDPOINT as PUSBPORT_ENDPOINT;

        RemoveEntryList(List.Flink as PLIST_ENTRY);
        (*Endpoint).DispatchLink.Flink = core::ptr::null_mut();
        (*Endpoint).DispatchLink.Blink = core::ptr::null_mut();

        USBPORT_EndpointWorker(Endpoint, 1 as BOOLEAN);
        USBPORT_FlushPendingTransfers(Endpoint);
    }

    KeAcquireSpinLockAtDpcLevel((core::ptr::addr_of_mut!((*FdoExtension).EndpointListSpinLock) as *mut usize));

    if !IsListEmpty(core::ptr::addr_of_mut!((*FdoExtension).WorkerList)) {
        USBPORT_SignalWorkerThread(FdoDevice);
    }

    KeReleaseSpinLockFromDpcLevel((core::ptr::addr_of_mut!((*FdoExtension).EndpointListSpinLock) as *mut usize));

    USBPORT_FlushDoneTransfers(FdoDevice);
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_IsrDpcHandler(
    FdoDevice: PDEVICE_OBJECT,
    IsDpcHandler: BOOLEAN,
) {
    let mut FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let mut Packet: PUSBPORT_REGISTRATION_PACKET;
    let mut Endpoint: PUSBPORT_ENDPOINT;
    let mut List: PLIST_ENTRY;
    let mut FrameNumber: ULONG;

    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;
    Packet = core::ptr::addr_of_mut!((*(*FdoExtension).MiniPortInterface).Packet)
        as PUSBPORT_REGISTRATION_PACKET;

    if InterlockedIncrement(
        core::ptr::addr_of_mut!((*FdoExtension).IsrDpcHandlerCounter) as *mut core::ffi::c_long,
    ) != 0
    {
        KeInsertQueueDpc(
            core::ptr::addr_of_mut!((*FdoExtension).IsrDpc),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
        InterlockedDecrement(
            core::ptr::addr_of_mut!((*FdoExtension).IsrDpcHandlerCounter) as *mut core::ffi::c_long,
        );
        return;
    }

    KeAcquireSpinLockAtDpcLevel((core::ptr::addr_of_mut!((*FdoExtension).EpStateChangeSpinLock) as *mut usize));
    List = (*FdoExtension).EpStateChangeList.Flink as PLIST_ENTRY;
    while List != core::ptr::addr_of_mut!((*FdoExtension).EpStateChangeList) {
        let mut EndpointReady: BOOLEAN = 0;
        let NextList: PLIST_ENTRY;
        let ControllerSuspended: BOOLEAN;

        Endpoint = (List as ULONG_PTR).wrapping_sub(core::mem::offset_of!(
            USBPORT_ENDPOINT,
            StateChangeLink
        ) as ULONG_PTR) as *mut USBPORT_ENDPOINT
            as PUSBPORT_ENDPOINT;

        NextList = (*List).Flink as PLIST_ENTRY;

        ControllerSuspended = (((*FdoExtension).Flags as core::ffi::c_ulong
            & USBPORT_FLAG_HC_SUSPEND as core::ffi::c_ulong
            != 0 as core::ffi::c_ulong)
            || ((*FdoExtension).CommonExtension.DevicePowerState as core::ffi::c_uint
                == PowerDeviceD3 as core::ffi::c_int as core::ffi::c_uint))
            as core::ffi::c_int as BOOLEAN;

        if ControllerSuspended != 0 {
            EndpointReady = 1 as BOOLEAN;
        } else {
            KeAcquireSpinLockAtDpcLevel((core::ptr::addr_of_mut!((*FdoExtension).MiniportSpinLock) as *mut usize));
            FrameNumber = ((*Packet).Get32BitFrameNumber.expect("non-null function pointer"))(
                (*FdoExtension).MiniPortExt,
            );
            KeReleaseSpinLockFromDpcLevel((core::ptr::addr_of_mut!((*FdoExtension).MiniportSpinLock) as *mut usize));

            if FrameNumber > (*Endpoint).FrameNumber
                || ((*Endpoint).Flags as core::ffi::c_ulong
                    & ENDPOINT_FLAG_NUKE as core::ffi::c_ulong
                    != 0)
            {
                EndpointReady = 1 as BOOLEAN;
            }
        }

        if EndpointReady != 0 {
            RemoveEntryList(core::ptr::addr_of_mut!((*Endpoint).StateChangeLink));
            (*Endpoint).StateChangeLink.Flink = core::ptr::null_mut();
            (*Endpoint).StateChangeLink.Blink = core::ptr::null_mut();

            KeAcquireSpinLockAtDpcLevel((core::ptr::addr_of_mut!((*Endpoint).StateChangeSpinLock) as *mut usize));
            (*Endpoint).StateLast = (*Endpoint).StateNext;
            KeReleaseSpinLockFromDpcLevel((core::ptr::addr_of_mut!(
                (*Endpoint).StateChangeSpinLock
            ) as *mut usize));

            KeReleaseSpinLockFromDpcLevel((core::ptr::addr_of_mut!(
                (*FdoExtension).EpStateChangeSpinLock
            ) as *mut usize));

            if IsDpcHandler != 0 {
                USBPORT_InvalidateEndpointHandler(
                    FdoDevice,
                    Endpoint,
                    INVALIDATE_ENDPOINT_ONLY as ULONG,
                );
            } else {
                USBPORT_InvalidateEndpointHandler(
                    FdoDevice,
                    Endpoint,
                    INVALIDATE_ENDPOINT_WORKER_THREAD as ULONG,
                );
            }

            KeAcquireSpinLockAtDpcLevel((core::ptr::addr_of_mut!(
                (*FdoExtension).EpStateChangeSpinLock
            ) as *mut usize));
        } else if ControllerSuspended == 0 {
            KeAcquireSpinLockAtDpcLevel((core::ptr::addr_of_mut!((*FdoExtension).MiniportSpinLock) as *mut usize));
            ((*Packet).InterruptNextSOF.expect("non-null function pointer"))(
                (*FdoExtension).MiniPortExt,
            );
            KeReleaseSpinLockFromDpcLevel((core::ptr::addr_of_mut!((*FdoExtension).MiniportSpinLock) as *mut usize));
        }

        List = NextList;
    }

    KeReleaseSpinLockFromDpcLevel((core::ptr::addr_of_mut!((*FdoExtension).EpStateChangeSpinLock) as *mut usize));

    if IsDpcHandler != 0 {
        USBPORT_DpcHandler(FdoDevice);
    }

    InterlockedDecrement(
        core::ptr::addr_of_mut!((*FdoExtension).IsrDpcHandlerCounter) as *mut core::ffi::c_long,
    );
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_IsrDpc(
    Dpc: PRKDPC,
    DeferredContext: PVOID,
    SystemArgument1: PVOID,
    SystemArgument2: PVOID,
) {
    let mut FdoDevice: PDEVICE_OBJECT;
    let mut FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let mut Packet: PUSBPORT_REGISTRATION_PACKET;
    let mut InterruptEnable: BOOLEAN;

    DPRINT_INT(
        b"USBPORT_IsrDpc: DeferredContext - %p, SystemArgument2 - %p\n\0".as_ptr() as *const i8,
        DeferredContext,
        SystemArgument2,
    );

    FdoDevice = DeferredContext as PDEVICE_OBJECT;
    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;
    Packet = &mut (*(*FdoExtension).MiniPortInterface).Packet as PUSBPORT_REGISTRATION_PACKET;

    if !SystemArgument2.is_null() {
        InterlockedDecrement(&mut (*FdoExtension).IsrDpcCounter as *mut core::ffi::c_long);
    }

    KeAcquireSpinLockAtDpcLevel(core::ptr::addr_of_mut!((*FdoExtension).MiniportInterruptsSpinLock) as *mut usize);
    InterruptEnable = (((*FdoExtension).Flags as u32 & USBPORT_FLAG_INTERRUPT_ENABLED as u32)
        == USBPORT_FLAG_INTERRUPT_ENABLED as u32) as i32 as BOOLEAN;

    ((*Packet).InterruptDpc.expect("non-null function pointer"))(
        (*FdoExtension).MiniPortExt,
        InterruptEnable,
    );

    KeReleaseSpinLockFromDpcLevel(core::ptr::addr_of_mut!((*FdoExtension).MiniportInterruptsSpinLock) as *mut usize);

    if (*FdoExtension).Flags as u32 & USBPORT_FLAG_HC_SUSPEND as u32 != 0
        && (*FdoExtension).TimerFlags as u32 & USBPORT_TMFLAG_WAKE as u32 != 0
    {
        USBPORT_CompletePdoWaitWake(FdoDevice);
    } else {
        USBPORT_IsrDpcHandler(FdoDevice, TRUE as BOOLEAN);
    }

    DPRINT_INT(b"USBPORT_IsrDpc: exit\n\0".as_ptr() as *const i8);
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_InterruptService(
    Interrupt: PKINTERRUPT,
    ServiceContext: PVOID,
) -> BOOLEAN {
    let mut FdoDevice: PDEVICE_OBJECT;
    let mut FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let mut Packet: PUSBPORT_REGISTRATION_PACKET;
    let mut Result: BOOLEAN = FALSE as BOOLEAN;

    FdoDevice = ServiceContext as PDEVICE_OBJECT;
    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;
    Packet = &mut (*(*FdoExtension).MiniPortInterface).Packet as PUSBPORT_REGISTRATION_PACKET;

    DPRINT_INT(
        b"USBPORT_InterruptService: FdoExtension[%p]->Flags - %08X\n\0".as_ptr() as *const i8,
        FdoExtension,
        (*FdoExtension).Flags,
    );

    if (*FdoExtension).Flags & USBPORT_FLAG_INTERRUPT_ENABLED != 0
        && (*FdoExtension).MiniPortFlags & USBPORT_MPFLAG_INTERRUPTS_ENABLED != 0
    {
        Result = ((*Packet).InterruptService).expect("non-null function pointer")(
            (*FdoExtension).MiniPortExt,
        );

        if Result != 0 {
            KeInsertQueueDpc(
                &mut (*FdoExtension).IsrDpc,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );
        }
    }

    DPRINT_INT(
        b"USBPORT_InterruptService: return - %x\n\0".as_ptr() as *const i8,
        (Result as ::core::ffi::c_int),
    );

    return Result;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_SignalWorkerThread(FdoDevice: PDEVICE_OBJECT) {
    let mut FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let mut OldIrql: KIRQL = 0;

    DPRINT_CORE(b"USBPORT_SignalWorkerThread ...\n\0".as_ptr() as *const i8);

    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;

    KeAcquireSpinLock(
        (core::ptr::addr_of_mut!((*FdoExtension).WorkerThreadEventSpinLock) as *mut usize),
        core::ptr::addr_of_mut!(OldIrql),
    );
    KeSetEvent(
        (core::ptr::addr_of_mut!((*FdoExtension).WorkerThreadEvent) as *mut c_void),
        EVENT_INCREMENT as KPRIORITY,
        FALSE as BOOLEAN,
    );
    KeReleaseSpinLock(
        (core::ptr::addr_of_mut!((*FdoExtension).WorkerThreadEventSpinLock) as *mut usize),
        OldIrql,
    );
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_WorkerThreadHandler(FdoDevice: PDEVICE_OBJECT) {

    const USBPORT_FLAG_HC_SUSPEND: u32 = 0x00000100; // FIXED 2026-07-05: translation invented 0x200; verified vs usbport.h/usbmport.h

    let mut FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let mut Packet: PUSBPORT_REGISTRATION_PACKET;
    let mut workerList: PLIST_ENTRY;
    let mut OldIrql: KIRQL = 0;
    let mut Endpoint: PUSBPORT_ENDPOINT;
    let mut list: LIST_ENTRY = LIST_ENTRY {
        Flink: core::ptr::null_mut(),
        Blink: core::ptr::null_mut(),
    };
    let mut Result: BOOLEAN;

    DPRINT_CORE(b"USBPORT_WorkerThreadHandler: ...\n\0".as_ptr() as *const i8);

    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;
    Packet = core::ptr::addr_of_mut!((*(*FdoExtension).MiniPortInterface).Packet) as PUSBPORT_REGISTRATION_PACKET;

    KeAcquireSpinLock((core::ptr::addr_of_mut!((*FdoExtension).MiniportSpinLock) as *mut usize), core::ptr::addr_of_mut!(OldIrql));

    if (*FdoExtension).Flags as u32 & USBPORT_FLAG_HC_SUSPEND == 0 {
        ((*Packet).CheckController.expect("non-null function pointer"))((*FdoExtension).MiniPortExt);
    }

    KeReleaseSpinLock((core::ptr::addr_of_mut!((*FdoExtension).MiniportSpinLock) as *mut usize), OldIrql);

    InitializeListHead(core::ptr::addr_of_mut!(list));

    USBPORT_FlushAllEndpoints(FdoDevice);

    loop {
        KeRaiseIrql(DISPATCH_LEVEL as KIRQL, core::ptr::addr_of_mut!(OldIrql));
        KeAcquireSpinLockAtDpcLevel((core::ptr::addr_of_mut!((*FdoExtension).EndpointListSpinLock) as *mut usize));

        workerList = core::ptr::addr_of_mut!((*FdoExtension).WorkerList) as PLIST_ENTRY;

        if IsListEmpty(workerList) {
            break;
        }

        Endpoint = (((*workerList).Flink as usize)
            .wrapping_sub(core::mem::offset_of!(USBPORT_ENDPOINT, WorkerLink))) as *mut USBPORT_ENDPOINT as PUSBPORT_ENDPOINT;

        DPRINT_CORE(b"USBPORT_WorkerThreadHandler: Endpoint - %p\n\0".as_ptr() as *const i8, Endpoint);

        RemoveHeadList(workerList);
        (*Endpoint).WorkerLink.Blink = core::ptr::null_mut();
        (*Endpoint).WorkerLink.Flink = core::ptr::null_mut();

        KeReleaseSpinLockFromDpcLevel((core::ptr::addr_of_mut!((*FdoExtension).EndpointListSpinLock) as *mut usize));

        Result = USBPORT_EndpointWorker(Endpoint, 0 as BOOLEAN);
        KeAcquireSpinLockAtDpcLevel((core::ptr::addr_of_mut!((*FdoExtension).EndpointListSpinLock) as *mut usize));

        if Result != 0 {
            if (*Endpoint).FlushAbortLink.Flink.is_null() || (*Endpoint).FlushAbortLink.Blink.is_null() {
                InsertTailList(core::ptr::addr_of_mut!(list), core::ptr::addr_of_mut!((*Endpoint).FlushAbortLink));
            }
        }

        while !IsListEmpty(core::ptr::addr_of_mut!(list)) {
            Endpoint = ((list.Flink as usize)
                .wrapping_sub(core::mem::offset_of!(USBPORT_ENDPOINT, FlushAbortLink))) as *mut USBPORT_ENDPOINT as PUSBPORT_ENDPOINT;

            RemoveHeadList(core::ptr::addr_of_mut!(list));

            (*Endpoint).FlushAbortLink.Flink = core::ptr::null_mut();
            (*Endpoint).FlushAbortLink.Blink = core::ptr::null_mut();

            if (*Endpoint).WorkerLink.Flink.is_null() || (*Endpoint).WorkerLink.Blink.is_null() {
                InsertTailList(core::ptr::addr_of_mut!((*FdoExtension).WorkerList), core::ptr::addr_of_mut!((*Endpoint).WorkerLink));

                USBPORT_SignalWorkerThread(FdoDevice);
            }
        }

        KeReleaseSpinLockFromDpcLevel((core::ptr::addr_of_mut!((*FdoExtension).EndpointListSpinLock) as *mut usize));
        KeLowerIrql(OldIrql);
    }

    KeReleaseSpinLockFromDpcLevel((core::ptr::addr_of_mut!((*FdoExtension).EndpointListSpinLock) as *mut usize));
    KeLowerIrql(OldIrql);

    USBPORT_FlushClosedEndpointList(FdoDevice);
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_DoRootHubCallback(FdoDevice: PDEVICE_OBJECT) {
    let mut FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let mut PdoDevice: PDEVICE_OBJECT;
    let mut PdoExtension: PUSBPORT_RHDEVICE_EXTENSION;
    let mut RootHubInitCallback: PRH_INIT_CALLBACK;
    let mut RootHubInitContext: PVOID;

    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;

    PdoDevice = (*FdoExtension).RootHubPdo;

    if !PdoDevice.is_null() {
        PdoExtension = (*PdoDevice).DeviceExtension as PUSBPORT_RHDEVICE_EXTENSION;

        RootHubInitContext = (*PdoExtension).RootHubInitContext;
        RootHubInitCallback = (*PdoExtension).RootHubInitCallback;

        (*PdoExtension).RootHubInitCallback = None;
        (*PdoExtension).RootHubInitContext = core::ptr::null_mut();

        if RootHubInitCallback.is_some() {
            RootHubInitCallback.expect("non-null function pointer")(RootHubInitContext);
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_SynchronizeRootHubCallback(
    FdoDevice: PDEVICE_OBJECT,
    Usb2FdoDevice: PDEVICE_OBJECT,
) {

    const USB_MINIPORT_FLAGS_USB2: ULONG = 0x00000010; // FIXED 2026-07-05: translation invented 0x40; verified vs usbport.h/usbmport.h
    const USBPORT_FLAG_RH_INIT_CALLBACK: ULONG = 0x80000000; // FIXED 2026-07-05: translation invented 0x800; verified vs usbport.h/usbmport.h
    const USBPORT_FLAG_COMPANION_HC: ULONG = 0x01000000;
    const USBPORT_FLAG_PWR_AND_CHIRP_LOCK: ULONG = 0x10000000;
    const USBPORT_FLAG_HC_SUSPEND: ULONG = 0x00000100; // FIXED 2026-07-05: translation invented 0x200; verified vs usbport.h/usbmport.h
    const USBPORT_FLAG_POWER_AND_CHIRP_OK: ULONG = 0x40000000; // FIXED 2026-07-05: translation invented 0x20000000; verified vs usbport.h/usbmport.h
    const USB_PORT_TAG: ULONG = 0x70627375;

    let mut FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let Packet: PUSBPORT_REGISTRATION_PACKET;
    let mut Usb2FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let mut CompanionControllersList: PDEVICE_RELATIONS;
    let mut CompanionFdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let mut Entry: *mut PDEVICE_OBJECT;
    let mut ix: ULONG;

    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;
    Packet = core::ptr::addr_of_mut!((*(*FdoExtension).MiniPortInterface).Packet)
        as PUSBPORT_REGISTRATION_PACKET;

    if Usb2FdoDevice.is_null()
        && ((*Packet).MiniPortFlags as ULONG & USB_MINIPORT_FLAGS_USB2) == 0
    {
        USBPORT_DoRootHubCallback(FdoDevice);

        (*FdoExtension).Flags &= !USBPORT_FLAG_RH_INIT_CALLBACK;
        _InterlockedCompareExchange(
            core::ptr::addr_of_mut!((*FdoExtension).RHInitCallBackLock) as *mut core::ffi::c_long,
            0,
            1,
        );

        return;
    }

    if ((*FdoExtension).Flags as ULONG & USBPORT_FLAG_COMPANION_HC) == 0 {
        KeWaitForSingleObject(
            core::ptr::addr_of_mut!((*FdoExtension).ControllerSemaphore) as PVOID,
            _KWAIT_REASON_Executive,
            0i32 as KPROCESSOR_MODE,
            0 as BOOLEAN,
            core::ptr::null_mut(),
        );

        (*FdoExtension).Flags |= USBPORT_FLAG_PWR_AND_CHIRP_LOCK;

        if ((*FdoExtension).Flags as ULONG
            & (USBPORT_FLAG_HC_SUSPEND | USBPORT_FLAG_POWER_AND_CHIRP_OK))
            == 0
        {
            USBPORT_RootHubPowerAndChirpAllCcPorts(FdoDevice);
            (*FdoExtension).Flags |= USBPORT_FLAG_POWER_AND_CHIRP_OK;
        }

        (*FdoExtension).Flags &= !USBPORT_FLAG_PWR_AND_CHIRP_LOCK;

        KeReleaseSemaphore(
            core::ptr::addr_of_mut!((*FdoExtension).ControllerSemaphore),
            LOW_REALTIME_PRIORITY as KPRIORITY,
            1 as LONG,
            0 as BOOLEAN,
        );

        CompanionControllersList =
            USBPORT_FindCompanionControllers(FdoDevice, 0 as BOOLEAN, 1 as BOOLEAN);

        if !CompanionControllersList.is_null() {
            Entry = core::ptr::addr_of_mut!((*CompanionControllersList).Objects) as *mut PDEVICE_OBJECT;

            ix = 0;
            while ix < (*CompanionControllersList).Count {
                CompanionFdoExtension = (**Entry).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;

                _InterlockedCompareExchange(
                    core::ptr::addr_of_mut!((*CompanionFdoExtension).RHInitCallBackLock)
                        as *mut core::ffi::c_long,
                    0,
                    1,
                );

                Entry = Entry.offset(1);
                ix = ix.wrapping_add(1);
            }

            ExFreePoolWithTag(CompanionControllersList as PVOID, USB_PORT_TAG);
        }

        USBPORT_DoRootHubCallback(FdoDevice);

        (*FdoExtension).Flags &= !USBPORT_FLAG_RH_INIT_CALLBACK;
        _InterlockedCompareExchange(
            core::ptr::addr_of_mut!((*FdoExtension).RHInitCallBackLock) as *mut core::ffi::c_long,
            0,
            1,
        );
    } else {
        Usb2FdoExtension = (*Usb2FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;

        USBPORT_Wait(FdoDevice as PVOID, 50 as ULONG);

        while (*FdoExtension).RHInitCallBackLock != 0 {
            USBPORT_Wait(FdoDevice as PVOID, 10 as ULONG);

            (*Usb2FdoExtension).Flags |= USBPORT_FLAG_RH_INIT_CALLBACK;
            USBPORT_SignalWorkerThread(Usb2FdoDevice);
        }

        USBPORT_DoRootHubCallback(FdoDevice);

        (*FdoExtension).Flags &= !USBPORT_FLAG_RH_INIT_CALLBACK;
    }
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_WorkerThread(StartContext: PVOID) {

    let mut FdoDevice: PDEVICE_OBJECT;
    let mut FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let mut OldTime: LARGE_INTEGER = core::mem::zeroed();
    let mut NewTime: LARGE_INTEGER = core::mem::zeroed();
    let mut OldIrql: KIRQL = 0;

    // DPRINT_CORE("USBPORT_WorkerThread ...\n");

    FdoDevice = StartContext as PDEVICE_OBJECT;
    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;

    (*FdoExtension).WorkerThread = KeGetCurrentThread() as PRKTHREAD;

    loop {
        // KeQuerySystemTime(&OldTime);
        *(core::ptr::addr_of_mut!(OldTime) as *mut ULONG64) = *(SharedSystemTimePtr() as *mut ULONG64);

        KeWaitForSingleObject(
            core::ptr::addr_of_mut!((*FdoExtension).WorkerThreadEvent) as PVOID,
            _KWAIT_REASON_Suspended,
            0i32 as KPROCESSOR_MODE,
            0 as BOOLEAN,
            core::ptr::null_mut(),
        );

        if (*FdoExtension).Flags as core::ffi::c_ulong
            & USBPORT_FLAG_WORKER_THREAD_EXIT as core::ffi::c_ulong
            != 0
        {
            break;
        }

        // KeQuerySystemTime(&NewTime);
        *(core::ptr::addr_of_mut!(NewTime) as *mut ULONG64) = *(SharedSystemTimePtr() as *mut ULONG64);

        KeAcquireSpinLock(
            (core::ptr::addr_of_mut!((*FdoExtension).WorkerThreadEventSpinLock) as *mut usize),
            core::ptr::addr_of_mut!(OldIrql),
        );
        KeClearEvent((core::ptr::addr_of_mut!((*FdoExtension).WorkerThreadEvent) as *mut c_void));
        KeReleaseSpinLock(
            (core::ptr::addr_of_mut!((*FdoExtension).WorkerThreadEventSpinLock) as *mut usize),
            OldIrql,
        );
        // DPRINT_CORE("USBPORT_WorkerThread: run\n");

        if (*FdoExtension).MiniPortFlags as core::ffi::c_ulong
            & USBPORT_MPFLAG_INTERRUPTS_ENABLED as core::ffi::c_ulong
            != 0
        {
            USBPORT_DoSetPowerD0(FdoDevice);

            if (*FdoExtension).Flags as core::ffi::c_ulong
                & USBPORT_FLAG_RH_INIT_CALLBACK as core::ffi::c_ulong
                != 0
            {
                let mut USB2FdoDevice: PDEVICE_OBJECT = core::ptr::null_mut();

                USB2FdoDevice = USBPORT_FindUSB2Controller(FdoDevice);
                USBPORT_SynchronizeRootHubCallback(FdoDevice, USB2FdoDevice);
            }
        }

        USBPORT_WorkerThreadHandler(FdoDevice);

        if !((*FdoExtension).Flags as core::ffi::c_ulong
            & USBPORT_FLAG_WORKER_THREAD_ON as core::ffi::c_ulong
            == 0)
        {
            break;
        }
    }

    PsTerminateSystemThread(0 as NTSTATUS);
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_CreateWorkerThread(FdoDevice: PDEVICE_OBJECT) -> NTSTATUS {
    let mut FdoExtension: PUSBPORT_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;

    (*FdoExtension).Flags &= !(USBPORT_FLAG_WORKER_THREAD_ON as ULONG);

    KeInitializeEvent(
        core::ptr::addr_of_mut!((*FdoExtension).WorkerThreadEvent),
        _EVENT_TYPE_NotificationEvent,
        0 as BOOLEAN,
    );

    Status = PsCreateSystemThread(
        core::ptr::addr_of_mut!((*FdoExtension).WorkerThreadHandle),
        THREAD_ALL_ACCESS as ULONG,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        cb!(USBPORT_WorkerThread),
        FdoDevice as PVOID,
    );

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_StopWorkerThread(FdoDevice: PDEVICE_OBJECT) {
    let mut FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let Status: NTSTATUS;

    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;

    (*FdoExtension).Flags |= USBPORT_FLAG_WORKER_THREAD_EXIT as u32;
    USBPORT_SignalWorkerThread(FdoDevice);
    Status = ZwWaitForSingleObject(
        (*FdoExtension).WorkerThreadHandle,
        0 as BOOLEAN,
        core::ptr::null_mut(),
    );
    if !(Status == STATUS_SUCCESS) {
        DbgPrint(
            b"Assertion failed at %s(%d): %s\n\0".as_ptr() as PCSTR,
            b"drivers/usb/usbport/usbport.c\0".as_ptr() as *const core::ffi::c_char,
            1502 as core::ffi::c_int,
            b"Status == STATUS_SUCCESS\0".as_ptr() as *const core::ffi::c_char,
        );
        __int2c();
    } else {
    };
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_SynchronizeControllersStart(FdoDevice: PDEVICE_OBJECT) {
    let mut FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let PdoDevice: PDEVICE_OBJECT;
    let PdoExtension: PUSBPORT_RHDEVICE_EXTENSION;
    let mut USB2FdoDevice: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut USB2FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let mut IsOn: BOOLEAN;

    DPRINT_TIMER(
        b"USBPORT_SynchronizeControllersStart: FdoDevice - %p\n\0".as_ptr() as *const i8,
        FdoDevice,
    );

    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;

    PdoDevice = (*FdoExtension).RootHubPdo;

    if PdoDevice.is_null() {
        return;
    }

    PdoExtension = (*PdoDevice).DeviceExtension as PUSBPORT_RHDEVICE_EXTENSION;

    if (*PdoExtension).RootHubInitCallback.is_none()
        || ((*FdoExtension).Flags & USBPORT_FLAG_RH_INIT_CALLBACK) != 0
    {
        return;
    }

    DPRINT_TIMER(
        b"USBPORT_SynchronizeControllersStart: Flags - %p\n\0".as_ptr() as *const i8,
        (*FdoExtension).Flags,
    );

    'start: loop {
        if ((*FdoExtension).Flags & USBPORT_FLAG_COMPANION_HC) != 0 {
            IsOn = FALSE as BOOLEAN;

            USB2FdoDevice = USBPORT_FindUSB2Controller(FdoDevice);

            DPRINT_TIMER(
                b"USBPORT_SynchronizeControllersStart: USB2FdoDevice - %p\n\0".as_ptr() as *const i8,
                USB2FdoDevice,
            );

            if !USB2FdoDevice.is_null() {
                USB2FdoExtension = (*USB2FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;

                if ((*USB2FdoExtension).CommonExtension.PnpStateFlags & USBPORT_PNP_STATE_STARTED)
                    != 0
                {
                    IsOn = TRUE as BOOLEAN;
                }
            }

            if ((*FdoExtension).Flags & USBPORT_FLAG_NO_HACTION) == 0 {
                break 'start;
            }

            USB2FdoDevice = core::ptr::null_mut();
        }

        IsOn = TRUE as BOOLEAN;
        break 'start;
    }

    // Start:
    if IsOn != 0
        && InterlockedCompareExchange(
            core::ptr::addr_of_mut!((*FdoExtension).RHInitCallBackLock) as *mut core::ffi::c_long,
            1,
            0,
        ) == 0
    {
        (*FdoExtension).Flags |= USBPORT_FLAG_RH_INIT_CALLBACK;
        USBPORT_SignalWorkerThread(FdoDevice);

        if !USB2FdoDevice.is_null() {
            USB2FdoExtension = (*USB2FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;

            (*USB2FdoExtension).Flags |= USBPORT_FLAG_RH_INIT_CALLBACK;
            USBPORT_SignalWorkerThread(USB2FdoDevice);
        }
    }

    DPRINT_TIMER(b"USBPORT_SynchronizeControllersStart: exit\n\0".as_ptr() as *const i8);
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_TimerDpc(
    Dpc: PRKDPC,
    DeferredContext: PVOID,
    SystemArgument1: PVOID,
    SystemArgument2: PVOID,
) {
    let mut FdoDevice: PDEVICE_OBJECT;
    let mut FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let mut Packet: PUSBPORT_REGISTRATION_PACKET;
    let mut DueTime: LARGE_INTEGER = core::mem::zeroed();
    let mut TimerFlags: ULONG;
    let mut IdleQueueItem: PTIMER_WORK_QUEUE_ITEM;
    let mut OldIrql: KIRQL = 0;
    let mut TimerOldIrql: KIRQL = 0;

    FdoDevice = DeferredContext as PDEVICE_OBJECT;
    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;
    Packet = core::ptr::addr_of_mut!((*(*FdoExtension).MiniPortInterface).Packet)
        as PUSBPORT_REGISTRATION_PACKET;

    KeAcquireSpinLock(
        (core::ptr::addr_of_mut!((*FdoExtension).TimerFlagsSpinLock) as *mut usize),
        core::ptr::addr_of_mut!(TimerOldIrql),
    );

    TimerFlags = (*FdoExtension).TimerFlags;

    if (*FdoExtension).Flags & USBPORT_FLAG_HC_SUSPEND != 0
        && (*FdoExtension).Flags & USBPORT_FLAG_HC_WAKE_SUPPORT != 0
        && TimerFlags & USBPORT_TMFLAG_HC_RESUME == 0
    {
        KeAcquireSpinLock(
            (core::ptr::addr_of_mut!((*FdoExtension).MiniportSpinLock) as *mut usize),
            core::ptr::addr_of_mut!(OldIrql),
        );
        ((*Packet).PollController.expect("non-null function pointer"))(
            (*FdoExtension).MiniPortExt,
        );
        KeReleaseSpinLock(
            (core::ptr::addr_of_mut!((*FdoExtension).MiniportSpinLock) as *mut usize),
            OldIrql,
        );
    }

    USBPORT_SynchronizeControllersStart(FdoDevice);

    if TimerFlags & USBPORT_TMFLAG_HC_SUSPENDED != 0 {
        USBPORT_BadRequestFlush(FdoDevice);
    } else {
        KeAcquireSpinLock(
            (core::ptr::addr_of_mut!((*FdoExtension).MiniportSpinLock) as *mut usize),
            core::ptr::addr_of_mut!(OldIrql),
        );

        if (*FdoExtension).Flags & USBPORT_FLAG_HC_SUSPEND == 0 {
            ((*Packet).CheckController.expect("non-null function pointer"))(
                (*FdoExtension).MiniPortExt,
            );
        }

        KeReleaseSpinLock(
            (core::ptr::addr_of_mut!((*FdoExtension).MiniportSpinLock) as *mut usize),
            OldIrql,
        );

        if (*FdoExtension).Flags & USBPORT_FLAG_HC_POLLING != 0 {
            KeAcquireSpinLock(
                (core::ptr::addr_of_mut!((*FdoExtension).MiniportSpinLock) as *mut usize),
                core::ptr::addr_of_mut!(OldIrql),
            );
            ((*Packet).PollController.expect("non-null function pointer"))(
                (*FdoExtension).MiniPortExt,
            );
            KeReleaseSpinLock(
                (core::ptr::addr_of_mut!((*FdoExtension).MiniportSpinLock) as *mut usize),
                OldIrql,
            );
        }

        USBPORT_IsrDpcHandler(FdoDevice, 0 as BOOLEAN);

        USBPORT_BadRequestFlush(FdoDevice);

        if (*FdoExtension).IdleLockCounter > -1
            && TimerFlags & USBPORT_TMFLAG_IDLE_QUEUEITEM_ON == 0
        {
            IdleQueueItem = ExAllocatePoolWithTag(
                NonPagedPool,
                core::mem::size_of::<TIMER_WORK_QUEUE_ITEM>() as SIZE_T,
                USB_PORT_TAG as ULONG,
            ) as PTIMER_WORK_QUEUE_ITEM;

            if !IdleQueueItem.is_null() {
                core::ptr::write_bytes(
                    IdleQueueItem as *mut u8,
                    0,
                    core::mem::size_of::<TIMER_WORK_QUEUE_ITEM>(),
                );

                (*IdleQueueItem).WqItem.List.Flink = core::ptr::null_mut();
                (*IdleQueueItem).WqItem.WorkerRoutine = cb!(USBPORT_DoIdleNotificationCallback);
                (*IdleQueueItem).WqItem.Parameter = IdleQueueItem as PVOID;

                (*IdleQueueItem).FdoDevice = FdoDevice;
                (*IdleQueueItem).Context = 0;

                (*FdoExtension).TimerFlags |= USBPORT_TMFLAG_IDLE_QUEUEITEM_ON;

                ExQueueWorkItem(
                    core::ptr::addr_of_mut!((*IdleQueueItem).WqItem),
                    CriticalWorkQueue,
                );
            }
        }
    }

    KeReleaseSpinLock(
        (core::ptr::addr_of_mut!((*FdoExtension).TimerFlagsSpinLock) as *mut usize),
        TimerOldIrql,
    );

    if TimerFlags & USBPORT_TMFLAG_TIMER_QUEUED != 0 {
        DueTime.QuadPart -= ((*FdoExtension).TimerValue as u32)
            .wrapping_mul(10000)
            .wrapping_add((KeQueryTimeIncrement() as u32).wrapping_sub(1))
            as i64;

        KeSetTimer(
            core::ptr::addr_of_mut!((*FdoExtension).TimerObject),
            DueTime,
            core::ptr::addr_of_mut!((*FdoExtension).TimerDpc),
        );
    }
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_StartTimer(
    FdoDevice: PDEVICE_OBJECT,
    Time: ULONG,
) -> BOOLEAN {
    let FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let mut DueTime: LARGE_INTEGER = core::mem::zeroed();
    let TimeIncrement: ULONG;
    let Result: BOOLEAN;

    // DPRINT_TIMER("USBPORT_StartTimer: FdoDevice - %p, Time - %x\n", FdoDevice, Time);

    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;

    TimeIncrement = KeQueryTimeIncrement();

    (*FdoExtension).TimerFlags |= USBPORT_TMFLAG_TIMER_QUEUED as ULONG;
    (*FdoExtension).TimerValue = Time;

    KeInitializeTimer(core::ptr::addr_of_mut!((*FdoExtension).TimerObject));
    KeInitializeDpc(
        core::ptr::addr_of_mut!((*FdoExtension).TimerDpc),
        cb!(USBPORT_TimerDpc),
        FdoDevice as PVOID,
    );

    DueTime.QuadPart -= (10000u32)
        .wrapping_mul(Time)
        .wrapping_add(TimeIncrement.wrapping_sub(1)) as i64;

    Result = KeSetTimer(
        core::ptr::addr_of_mut!((*FdoExtension).TimerObject),
        DueTime,
        core::ptr::addr_of_mut!((*FdoExtension).TimerDpc),
    );

    return Result;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_AllocateCommonBuffer(
    FdoDevice: PDEVICE_OBJECT,
    BufferLength: SIZE_T,
) -> PUSBPORT_COMMON_BUFFER_HEADER {
    let mut HeaderBuffer: PUSBPORT_COMMON_BUFFER_HEADER = core::ptr::null_mut();
    let FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let DmaAdapter: PDMA_ADAPTER;
    let DmaOperations: PDMA_OPERATIONS;
    let HeaderSize: SIZE_T;
    let mut Length: ULONG = 0;
    let LengthPadded: ULONG;
    let mut LogicalAddress: PHYSICAL_ADDRESS = core::mem::zeroed();
    let BaseVA: ULONG_PTR;
    let StartBufferVA: ULONG_PTR;
    let StartBufferPA: ULONG;

    if BufferLength == 0 {
        return HeaderBuffer;
    }

    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;

    DmaAdapter = (*FdoExtension).DmaAdapter;
    DmaOperations = (*DmaAdapter).DmaOperations as PDMA_OPERATIONS;

    HeaderSize = core::mem::size_of::<USBPORT_COMMON_BUFFER_HEADER>() as SIZE_T;
    Length = ROUND_TO_PAGES(BufferLength.wrapping_add(HeaderSize)) as ULONG;
    LengthPadded = (Length as SIZE_T).wrapping_sub(BufferLength.wrapping_add(HeaderSize)) as ULONG;

    BaseVA = ((*DmaOperations).AllocateCommonBuffer.expect("non-null function pointer")(
        DmaAdapter,
        Length,
        core::ptr::addr_of_mut!(LogicalAddress),
        1 as BOOLEAN,
    )) as ULONG_PTR;

    if BaseVA == 0 {
        return HeaderBuffer;
    }

    StartBufferVA = BaseVA & !((PAGE_SIZE - 1) as ULONG_PTR);
    StartBufferPA = LogicalAddress.__bindgen_anon_1.LowPart as ULONG & !((PAGE_SIZE - 1) as ULONG);

    HeaderBuffer = StartBufferVA
        .wrapping_add(BufferLength as ULONG_PTR)
        .wrapping_add(LengthPadded as ULONG_PTR) as PUSBPORT_COMMON_BUFFER_HEADER;

    (*HeaderBuffer).Length = Length;
    (*HeaderBuffer).BaseVA = (BaseVA as _);
    (*HeaderBuffer).LogicalAddress = LogicalAddress;

    (*HeaderBuffer).BufferLength = (BufferLength.wrapping_add(LengthPadded as SIZE_T) as _);
    (*HeaderBuffer).VirtualAddress = (StartBufferVA as _);
    (*HeaderBuffer).PhysicalAddress = StartBufferPA;

    RtlZeroMemory(
        StartBufferVA as *mut core::ffi::c_void,
        BufferLength.wrapping_add(LengthPadded as SIZE_T),
    );

    return HeaderBuffer;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_FreeCommonBuffer(
    FdoDevice: PDEVICE_OBJECT,
    HeaderBuffer: PUSBPORT_COMMON_BUFFER_HEADER,
) {
    let mut FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let mut DmaAdapter: PDMA_ADAPTER;
    let mut DmaOperations: PDMA_OPERATIONS;

    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;

    DmaAdapter = (*FdoExtension).DmaAdapter;
    DmaOperations = (*DmaAdapter).DmaOperations as PDMA_OPERATIONS;

    ((*DmaOperations).FreeCommonBuffer.expect("non-null function pointer"))(
        (*FdoExtension).DmaAdapter,
        (*HeaderBuffer).Length,
        (*HeaderBuffer).LogicalAddress,
        (*HeaderBuffer).VirtualAddress as *mut core::ffi::c_void,
        1 as BOOLEAN,
    );
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_FindMiniPort(
    DriverObject: PDRIVER_OBJECT,
) -> PUSBPORT_MINIPORT_INTERFACE {
    let mut OldIrql: KIRQL = 0;
    let mut List: PLIST_ENTRY;
    let mut MiniPortInterface: PUSBPORT_MINIPORT_INTERFACE =
        core::ptr::null_mut();
    let mut IsFound: BOOLEAN = 0;

    KeAcquireSpinLock(
        core::ptr::addr_of_mut!(USBPORT_SpinLock),
        core::ptr::addr_of_mut!(OldIrql),
    );

    List = USBPORT_MiniPortDrivers.Flink;
    while List != core::ptr::addr_of_mut!(USBPORT_MiniPortDrivers) {
        MiniPortInterface = (List as usize).wrapping_sub(
            core::mem::offset_of!(USBPORT_MINIPORT_INTERFACE, DriverLink),
        ) as *mut USBPORT_MINIPORT_INTERFACE;

        if (*MiniPortInterface).DriverObject == DriverObject {
            IsFound = 1;
            break;
        }

        List = (*List).Flink;
    }

    KeReleaseSpinLock(core::ptr::addr_of_mut!(USBPORT_SpinLock), OldIrql);

    if IsFound != 0 {
        return MiniPortInterface;
    } else {
        return core::ptr::null_mut();
    }
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_AddDevice(
    DriverObject: PDRIVER_OBJECT,
    PhysicalDeviceObject: PDEVICE_OBJECT,
) -> NTSTATUS {

    const FILE_DEVICE_CONTROLLER: u32 = 0x00000004;
    const USB_MINIPORT_FLAGS_USB2: u32 = 0x00000010; // FIXED 2026-07-05: translation invented 0x100; verified vs usbport.h/usbmport.h

    let mut Status: NTSTATUS;
    let mut MiniPortInterface: PUSBPORT_MINIPORT_INTERFACE;
    let mut DeviceNumber: ULONG = 0 as ULONG;
    let mut CharDeviceName: [u16; 64] = [0; 64];
    let mut DeviceName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut DeviceObject: PDEVICE_OBJECT;
    let FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let FdoCommonExtension: PUSBPORT_COMMON_DEVICE_EXTENSION;
    let LowerDevice: PDEVICE_OBJECT;
    let mut Length: ULONG;

    MiniPortInterface = USBPORT_FindMiniPort(DriverObject);

    if MiniPortInterface.is_null() {
        return STATUS_UNSUCCESSFUL;
    }

    DeviceObject = core::ptr::null_mut();

    loop {
        RtlStringCbPrintfW(
            CharDeviceName.as_mut_ptr(),
            core::mem::size_of::<[u16; 64]>(),
            w!("\\Device\\USBFDO-%d").as_ptr() as PCWSTR,
            DeviceNumber,
        );

        RtlInitUnicodeString(
            core::ptr::addr_of_mut!(DeviceName),
            CharDeviceName.as_mut_ptr() as PCWSTR,
        );

        Length = (core::mem::size_of::<USBPORT_DEVICE_EXTENSION>())
            .wrapping_add((*MiniPortInterface).Packet.MiniPortExtensionSize as usize)
            .wrapping_add(core::mem::size_of::<USB2_HC_EXTENSION>()) as ULONG;

        Status = IoCreateDevice(
            DriverObject,
            Length,
            core::ptr::addr_of_mut!(DeviceName),
            FILE_DEVICE_CONTROLLER,
            0,
            0,
            core::ptr::addr_of_mut!(DeviceObject),
        );

        if NT_SUCCESS(Status) {
            break;
        }

        if Status == STATUS_OBJECT_NAME_EXISTS || Status == STATUS_OBJECT_NAME_COLLISION {
            DeviceNumber = DeviceNumber.wrapping_add(1);
            continue;
        }

        if !NT_SUCCESS(Status) {
            return Status;
        }
    }

    FdoExtension = (*DeviceObject).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;
    FdoCommonExtension =
        core::ptr::addr_of_mut!((*FdoExtension).CommonExtension) as PUSBPORT_COMMON_DEVICE_EXTENSION;

    RtlZeroMemory(
        FdoExtension as *mut core::ffi::c_void,
        core::mem::size_of::<USBPORT_DEVICE_EXTENSION>(),
    );

    (*FdoCommonExtension).SelfDevice = DeviceObject;
    (*FdoCommonExtension).LowerPdoDevice = PhysicalDeviceObject;
    (*FdoCommonExtension).IsPDO = 0;

    LowerDevice = IoAttachDeviceToDeviceStack(DeviceObject, PhysicalDeviceObject);

    (*FdoCommonExtension).LowerDevice = LowerDevice;

    (*FdoCommonExtension).DevicePowerState = PowerDeviceD3;

    (*FdoExtension).MiniPortExt = (FdoExtension as usize)
        .wrapping_add(core::mem::size_of::<USBPORT_DEVICE_EXTENSION>())
        as PVOID;

    if (*MiniPortInterface).Packet.MiniPortFlags & USB_MINIPORT_FLAGS_USB2 != 0 {
        (*FdoExtension).Usb2Extension = ((*FdoExtension).MiniPortExt as usize)
            .wrapping_add((*MiniPortInterface).Packet.MiniPortExtensionSize as usize)
            as PUSB2_HC_EXTENSION;

        USB2_InitController((*FdoExtension).Usb2Extension);
    } else {
        (*FdoExtension).Usb2Extension = core::ptr::null_mut();
    }

    (*FdoExtension).MiniPortInterface = MiniPortInterface;
    (*FdoExtension).FdoNameNumber = DeviceNumber;

    // Before the PnP manager assigns resources, which follows AddDevice.
    const USB_MINIPORT_VERSION_XHCI: u32 = 4;
    if (*MiniPortInterface).Packet.MiniPortVersion == USB_MINIPORT_VERSION_XHCI {
        crate::msi::opt_in(PhysicalDeviceObject);
    }

    KeInitializeSemaphore(core::ptr::addr_of_mut!((*FdoExtension).DeviceSemaphore), 1, 1);
    KeInitializeSemaphore(core::ptr::addr_of_mut!((*FdoExtension).ControllerSemaphore), 1, 1);

    InitializeListHead(core::ptr::addr_of_mut!((*FdoExtension).EndpointList));
    InitializeListHead(core::ptr::addr_of_mut!((*FdoExtension).DoneTransferList));
    InitializeListHead(core::ptr::addr_of_mut!((*FdoExtension).WorkerList));
    InitializeListHead(core::ptr::addr_of_mut!((*FdoExtension).EpStateChangeList));
    InitializeListHead(core::ptr::addr_of_mut!((*FdoExtension).MapTransferList));
    InitializeListHead(core::ptr::addr_of_mut!((*FdoExtension).DeviceHandleList));
    InitializeListHead(core::ptr::addr_of_mut!((*FdoExtension).IdleIrpList));
    InitializeListHead(core::ptr::addr_of_mut!((*FdoExtension).BadRequestList));
    InitializeListHead(core::ptr::addr_of_mut!((*FdoExtension).EndpointClosedList));

    (*DeviceObject).Flags &= !DO_DEVICE_INITIALIZING;

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_Unload(mut DriverObject: PDRIVER_OBJECT) {
    const BUGCODE_USB_DRIVER: u32 = 0xFE;

    let mut MiniPortInterface: PUSBPORT_MINIPORT_INTERFACE =
        core::ptr::null_mut::<_USBPORT_MINIPORT_INTERFACE>();

    if DbgPrint(
        b"(%s:%d) USBPORT_Unload: FIXME!\n\0" as *const u8 as PCSTR,
        b"drivers/usb/usbport/usbport.c\0"
            as *const u8 as *const i8,
        2009 as i32,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
            b"drivers/usb/usbport/usbport.c\0"
                as *const u8 as *const i8,
            2009 as i32,
        );
    }

    MiniPortInterface = USBPORT_FindMiniPort(DriverObject);

    if MiniPortInterface.is_null() {
        KeBugCheckEx(
            BUGCODE_USB_DRIVER,
            1 as usize,
            0 as usize,
            0 as usize,
            0 as usize,
        );
    }

    if DbgPrint(
        b"(%s:%d) USBPORT_Unload: UNIMPLEMENTED. FIXME.\n\0" as *const u8 as PCSTR,
        b"drivers/usb/usbport/usbport.c\0"
            as *const u8 as *const i8,
        2019 as i32,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
            b"drivers/usb/usbport/usbport.c\0"
                as *const u8 as *const i8,
            2019 as i32,
        );
    }
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_MiniportCompleteTransfer(
    MiniPortExtension: PVOID,
    MiniPortEndpoint: PVOID,
    TransferParameters: PVOID,
    USBDStatus: USBD_STATUS,
    TransferLength: ULONG,
) {
    let mut Transfer: PUSBPORT_TRANSFER;
    let mut ParentTransfer: PUSBPORT_TRANSFER;
    let mut SplitTransfer: PUSBPORT_TRANSFER;
    let mut SplitHead: PLIST_ENTRY;
    let mut Entry: PLIST_ENTRY;
    let mut OldIrql: KIRQL = 0;

    Transfer = (TransferParameters as usize).wrapping_sub(
        core::mem::offset_of!(USBPORT_TRANSFER, TransferParameters),
    ) as *mut USBPORT_TRANSFER;

    (*Transfer).Flags |= TRANSFER_FLAG_COMPLETED;
    (*Transfer).CompletedTransferLen = TransferLength;

    if ((*Transfer).Flags & TRANSFER_FLAG_SPLITED) == 0
        || TransferLength >= (*Transfer).TransferParameters.TransferBufferLength
    {
        USBPORT_QueueDoneTransfer(Transfer, USBDStatus);
        return;
    }

    ParentTransfer = (*Transfer).ParentTransfer as PUSBPORT_TRANSFER;

    KeAcquireSpinLock(
        (core::ptr::addr_of_mut!((*ParentTransfer).TransferSpinLock) as *mut usize),
        core::ptr::addr_of_mut!(OldIrql),
    );

    if IsListEmpty(core::ptr::addr_of_mut!((*ParentTransfer).SplitTransfersList)) {
        // Upstream ReactOS `goto Exit` here skips KeReleaseSpinLock — a leaked
        // spinlock + raised IRQL on the empty-list path (usbport.c:2059). We
        // acquired TransferSpinLock just above; release it before completing.
        // (Divergence from stock ReactOS: fixes the leak the C leaves behind.)
        KeReleaseSpinLock(
            (core::ptr::addr_of_mut!((*ParentTransfer).TransferSpinLock) as *mut usize),
            OldIrql,
        );
        USBPORT_QueueDoneTransfer(Transfer, USBDStatus);
        return;
    }

    SplitHead = core::ptr::addr_of_mut!((*ParentTransfer).SplitTransfersList) as PLIST_ENTRY;
    Entry = (*SplitHead).Flink as PLIST_ENTRY;

    while !Entry.is_null() && !IsListEmpty(SplitHead) {
        SplitTransfer = (Entry as usize).wrapping_sub(
            core::mem::offset_of!(USBPORT_TRANSFER, SplitLink),
        ) as *mut USBPORT_TRANSFER;

        if ((*SplitTransfer).Flags & TRANSFER_FLAG_SUBMITED) == 0 {
            DbgPrint(
                b"USBPORT_MiniportCompleteTransfer: SplitTransfer->Flags - %X\n\0".as_ptr()
                    as PCSTR,
                (*SplitTransfer).Flags,
            );
            //Add TRANSFER_FLAG_xxx
        }

        Entry = (*Entry).Flink as PLIST_ENTRY;
    }

    KeReleaseSpinLock(
        (core::ptr::addr_of_mut!((*ParentTransfer).TransferSpinLock) as *mut usize),
        OldIrql,
    );

    USBPORT_QueueDoneTransfer(Transfer, USBDStatus);
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_AsyncTimerDpc(
    Dpc: PRKDPC,
    DeferredContext: PVOID,
    SystemArgument1: PVOID,
    SystemArgument2: PVOID,
) {
    let mut FdoDevice: PDEVICE_OBJECT;
    let mut FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let mut AsyncCallbackData: PUSBPORT_ASYNC_CALLBACK_DATA;

    AsyncCallbackData = DeferredContext as PUSBPORT_ASYNC_CALLBACK_DATA;
    FdoDevice = (*AsyncCallbackData).FdoDevice;
    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;

    ((*AsyncCallbackData).CallbackFunction.expect("non-null function pointer"))(
        (*FdoExtension).MiniPortExt,
        core::ptr::addr_of_mut!((*AsyncCallbackData).CallbackContext) as PVOID,
    );

    ExFreePoolWithTag(AsyncCallbackData as PVOID, USB_PORT_TAG as ULONG);
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_RequestAsyncCallback(
    MiniPortExtension: PVOID,
    TimerValue: ULONG,
    Buffer: PVOID,
    Length: SIZE_T,
    Callback: ASYNC_TIMER_CALLBACK,
) -> ULONG {
    let mut FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let mut FdoDevice: PDEVICE_OBJECT;
    let mut AsyncCallbackData: PUSBPORT_ASYNC_CALLBACK_DATA;
    let mut DueTime: LARGE_INTEGER = core::mem::zeroed();

    FdoExtension = ((MiniPortExtension as usize)
        - core::mem::size_of::<USBPORT_DEVICE_EXTENSION>())
        as PUSBPORT_DEVICE_EXTENSION;

    FdoDevice = (*FdoExtension).CommonExtension.SelfDevice;

    AsyncCallbackData = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<USBPORT_ASYNC_CALLBACK_DATA>() + Length,
        USB_PORT_TAG,
    ) as PUSBPORT_ASYNC_CALLBACK_DATA;

    if AsyncCallbackData.is_null() {
        return 0;
    }

    RtlZeroMemory(
        AsyncCallbackData as *mut core::ffi::c_void,
        core::mem::size_of::<USBPORT_ASYNC_CALLBACK_DATA>() + Length,
    );

    if Length != 0 {
        RtlCopyMemory(
            core::ptr::addr_of_mut!((*AsyncCallbackData).CallbackContext) as *mut core::ffi::c_void,
            Buffer as *const core::ffi::c_void,
            Length,
        );
    }

    (*AsyncCallbackData).FdoDevice = FdoDevice;
    (*AsyncCallbackData).CallbackFunction = Callback;

    KeInitializeTimer(core::ptr::addr_of_mut!((*AsyncCallbackData).AsyncTimer));

    KeInitializeDpc(
        core::ptr::addr_of_mut!((*AsyncCallbackData).AsyncTimerDpc),
        cb!(USBPORT_AsyncTimerDpc),
        AsyncCallbackData as PVOID,
    );

    DueTime.QuadPart -= (KeQueryTimeIncrement().wrapping_sub(1).wrapping_add(
        (10000 as ULONG).wrapping_mul(TimerValue),
    )) as i64;

    KeSetTimer(
        core::ptr::addr_of_mut!((*AsyncCallbackData).AsyncTimer),
        DueTime,
        core::ptr::addr_of_mut!((*AsyncCallbackData).AsyncTimerDpc),
    );

    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_GetMappedVirtualAddress(
    PhysicalAddress: ULONG,
    MiniPortExtension: PVOID,
    MiniPortEndpoint: PVOID,
) -> PVOID {

    let mut HeaderBuffer: PUSBPORT_COMMON_BUFFER_HEADER =
        core::ptr::null_mut();
    let mut Endpoint: PUSBPORT_ENDPOINT = core::ptr::null_mut();
    let mut Offset: ULONG;
    let mut VirtualAddress: ULONG_PTR;

    // DPRINT_CORE("USBPORT_GetMappedVirtualAddress ...\n");

    Endpoint = (MiniPortEndpoint as ULONG_PTR)
        .wrapping_sub(core::mem::size_of::<USBPORT_ENDPOINT>() as ULONG_PTR)
        as PUSBPORT_ENDPOINT;

    if Endpoint.is_null() {
        if 0 as core::ffi::c_int == 0 {
            RtlAssert(
                b"FALSE\0".as_ptr() as *const i8 as PVOID,
                b"drivers/usb/usbport/usbport.c\0".as_ptr() as *const i8 as PVOID,
                2186 as ULONG,
                core::ptr::null_mut::<CHAR>(),
            );
        } else {
        }
    }

    HeaderBuffer = (*Endpoint).HeaderBuffer;

    Offset = PhysicalAddress.wrapping_sub((*HeaderBuffer).PhysicalAddress);
    VirtualAddress = ((*HeaderBuffer).VirtualAddress as core::ffi::c_ulonglong)
        .wrapping_add(Offset as core::ffi::c_ulonglong) as ULONG_PTR;

    return VirtualAddress as PVOID;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_InvalidateEndpoint(
    MiniPortExtension: PVOID,
    MiniPortEndpoint: PVOID,
) -> ULONG {
    let mut FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let FdoDevice: PDEVICE_OBJECT;
    let mut Endpoint: PUSBPORT_ENDPOINT;

    DPRINT_CORE(b"USBPORT_InvalidateEndpoint: ...\n\0".as_ptr() as *const i8);

    FdoExtension = (MiniPortExtension as usize)
        .wrapping_sub(core::mem::size_of::<USBPORT_DEVICE_EXTENSION>())
        as PUSBPORT_DEVICE_EXTENSION;

    FdoDevice = (*FdoExtension).CommonExtension.SelfDevice;

    if MiniPortEndpoint.is_null() {
        USBPORT_InvalidateEndpointHandler(
            FdoDevice,
            core::ptr::null_mut(),
            INVALIDATE_ENDPOINT_ONLY as ULONG,
        );
        return 0;
    }

    Endpoint = (MiniPortEndpoint as usize)
        .wrapping_sub(core::mem::size_of::<USBPORT_ENDPOINT>())
        as PUSBPORT_ENDPOINT;

    USBPORT_InvalidateEndpointHandler(
        FdoDevice,
        Endpoint,
        INVALIDATE_ENDPOINT_ONLY as ULONG,
    );

    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_CompleteTransfer(Urb: PURB, TransferStatus: USBD_STATUS) {
    let UrbTransfer: *mut _URB_CONTROL_TRANSFER;
    let Transfer: PUSBPORT_TRANSFER;
    let Status: NTSTATUS;
    let Irp: PIRP;
    let mut OldIrql: KIRQL = 0;
    let Event: PRKEVENT;
    let WriteToDevice: BOOLEAN;
    let IsFlushSuccess: BOOLEAN;
    let Mdl: PMDL;
    let CurrentVa: ULONG_PTR;
    let TransferLength: SIZE_T;
    let Endpoint: PUSBPORT_ENDPOINT;
    let FdoDevice: PDEVICE_OBJECT;
    let FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let DmaOperations: PDMA_OPERATIONS;

    UrbTransfer = core::ptr::addr_of_mut!((*Urb).__bindgen_anon_1.UrbControlTransfer);
    Transfer = (*UrbTransfer).hca.Reserved8[0] as PUSBPORT_TRANSFER;
    Endpoint = (*Transfer).Endpoint;

    (*Transfer).USBDStatus = TransferStatus;
    Status = USBPORT_USBDStatusToNtStatus(Urb, TransferStatus);

    (*UrbTransfer).TransferBufferLength = (*Transfer).CompletedTransferLen;

    if ((*Transfer).Flags as u32 & TRANSFER_FLAG_DMA_MAPPED as u32) != 0 {
        FdoDevice = (*Endpoint).FdoDevice;
        FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;
        DmaOperations = (*(*FdoExtension).DmaAdapter).DmaOperations as PDMA_OPERATIONS;

        WriteToDevice = ((*Transfer).Direction == USBPORT_DMA_DIRECTION_TO_DEVICE as u32) as BOOLEAN;
        Mdl = (*UrbTransfer).TransferBufferMDL;
        CurrentVa = MmGetMdlVirtualAddress(Mdl) as ULONG_PTR;
        TransferLength = (*UrbTransfer).TransferBufferLength as SIZE_T;

        // Our xHCI miniport DMAs only into its private common-buffer arena and
        // copies completed IN bytes into MappedSystemVa itself. The S/G mapping
        // above still supplies offsets for SplitTransfer, but receives no DMA.
        // Flushing it here could copy a stale HAL bounce buffer over the data
        // usbxhci just returned (x86 buffers beyond the adapter's address limit).
        // Keep FreeMapRegisters below: the unused mapping still owns registers.
        IsFlushSuccess = if (*(*FdoExtension).MiniPortInterface).Packet.MiniPortVersion
            == USB_MINIPORT_VERSION_XHCI
        {
            1
        } else {
            ((*DmaOperations).FlushAdapterBuffers.expect("non-null function pointer"))(
                (*FdoExtension).DmaAdapter,
                Mdl,
                (*Transfer).MapRegisterBase,
                CurrentVa as PVOID,
                TransferLength as ULONG,
                WriteToDevice,
            )
        };

        if IsFlushSuccess == 0 {
            // DPRINT("USBPORT_CompleteTransfer: no FlushAdapterBuffers !!!\n");
            // ASSERT(FALSE);
        }

        KeRaiseIrql(DISPATCH_LEVEL as KIRQL, core::ptr::addr_of_mut!(OldIrql));

        ((*DmaOperations).FreeMapRegisters.expect("non-null function pointer"))(
            (*FdoExtension).DmaAdapter,
            (*Transfer).MapRegisterBase,
            (*Transfer).NumberOfMapRegisters,
        );

        KeLowerIrql(OldIrql);
    }

    if ((*Urb).__bindgen_anon_1.UrbHeader.UsbdFlags as u32 & USBD_FLAG_ALLOCATED_MDL as u32) != 0 {
        IoFreeMdl((*Transfer).TransferBufferMDL);
        (*Urb).__bindgen_anon_1.UrbHeader.UsbdFlags |= !(USBD_FLAG_ALLOCATED_MDL as u32);
    }

    (*Urb).__bindgen_anon_1.UrbControlTransfer.hca.Reserved8[0] = core::ptr::null_mut();
    (*Urb).__bindgen_anon_1.UrbHeader.UsbdFlags |= !(USBD_FLAG_ALLOCATED_TRANSFER as u32);

    Irp = (*Transfer).Irp;

    if !Irp.is_null() {
        if !NT_SUCCESS(Status) {
            //DbgBreakPoint();
            USBPORT_DumpingURB(Urb);
        }

        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        (*Irp).IoStatus.Information = (0 as _);

        KeRaiseIrql(DISPATCH_LEVEL as KIRQL, core::ptr::addr_of_mut!(OldIrql));
        IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
        KeLowerIrql(OldIrql);
    }

    Event = (*Transfer).Event;

    if !Event.is_null() {
        KeSetEvent((Event as *mut c_void), EVENT_INCREMENT as i32, 0);
    }

    ExFreePoolWithTag(Transfer as PVOID, USB_PORT_TAG as ULONG);
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_MapTransfer(
    FdoDevice: PDEVICE_OBJECT,
    Irp: PIRP,
    MapRegisterBase: PVOID,
    Context: PVOID,
) -> IO_ALLOCATION_ACTION {
    let mut FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let mut DmaAdapter: PDMA_ADAPTER;
    let mut Transfer: PUSBPORT_TRANSFER;
    let mut Urb: PURB;
    let mut Endpoint: PUSBPORT_ENDPOINT;
    let mut Mdl: PMDL;
    let mut CurrentVa: ULONG_PTR;
    let mut sgList: PUSBPORT_SCATTER_GATHER_LIST;
    let mut CurrentLength: SIZE_T;
    let mut ix: ULONG;
    let mut WriteToDevice: BOOLEAN;
    let mut PhAddr: PHYSICAL_ADDRESS = core::mem::zeroed();
    let mut PhAddress: PHYSICAL_ADDRESS = core::mem::zeroed();
    let mut TransferLength: ULONG;
    let mut SgCurrentLength: SIZE_T;
    let mut ElementLength: SIZE_T;
    let mut DeviceHandle: PUSBPORT_DEVICE_HANDLE;
    let mut DmaOperations: PDMA_OPERATIONS;
    let mut USBDStatus: USBD_STATUS;
    let mut List: LIST_ENTRY = LIST_ENTRY {
        Flink: core::ptr::null_mut(),
        Blink: core::ptr::null_mut(),
    };
    let mut transfer: PUSBPORT_TRANSFER;

    DPRINT_CORE(b"USBPORT_MapTransfer: ...\n\0".as_ptr() as *const i8);

    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;
    DmaAdapter = (*FdoExtension).DmaAdapter;
    DmaOperations = (*DmaAdapter).DmaOperations as PDMA_OPERATIONS;

    Transfer = Context as PUSBPORT_TRANSFER;

    Urb = (*Transfer).Urb;
    Endpoint = (*Transfer).Endpoint;
    TransferLength = (*Transfer).TransferParameters.TransferBufferLength;

    Mdl = (*Urb).__bindgen_anon_1.UrbControlTransfer.TransferBufferMDL;
    CurrentVa = MmGetMdlVirtualAddress(Mdl) as ULONG_PTR;

    sgList = core::ptr::addr_of_mut!((*Transfer).SgList) as PUSBPORT_SCATTER_GATHER_LIST;

    (*sgList).Flags = 0;
    (*sgList).CurrentVa = (CurrentVa as _);
    (*sgList).MappedSystemVa = MmGetSystemAddressForMdlSafe(Mdl, NormalPagePriority);
    (*Transfer).MapRegisterBase = MapRegisterBase;

    ix = 0;
    CurrentLength = 0;

    loop {
        WriteToDevice = ((*Transfer).Direction == USBPORT_DMA_DIRECTION_TO_DEVICE as ULONG) as BOOLEAN;
        // ASSERT(Transfer->Direction != 0)

        PhAddress = ((*DmaOperations).MapTransfer.expect("non-null function pointer"))(
            DmaAdapter,
            Mdl,
            MapRegisterBase,
            CurrentVa as PVOID,
            core::ptr::addr_of_mut!(TransferLength),
            WriteToDevice,
        );

        DPRINT_CORE(
            b"USBPORT_MapTransfer: PhAddress.__bindgen_anon_1.LowPart - %p, PhAddress.__bindgen_anon_1.HighPart - %x, TransferLength - %x\n\0"
                .as_ptr() as *const i8,
        );

        PhAddress.u.HighPart = 0;
        SgCurrentLength = TransferLength as SIZE_T;

        loop {
            ElementLength = (PAGE_SIZE as ULONG)
                .wrapping_sub(PhAddress.u.LowPart as ULONG & (PAGE_SIZE - 1) as ULONG)
                as SIZE_T;

            if ElementLength > SgCurrentLength {
                ElementLength = SgCurrentLength;
            }

            DPRINT_CORE(
                b"USBPORT_MapTransfer: PhAddress.__bindgen_anon_1.LowPart - %p, HighPart - %x, ElementLength - %x\n\0"
                    .as_ptr() as *const i8,
            );

            (*(core::ptr::addr_of_mut!((*sgList).SgElement) as *mut USBPORT_SCATTER_GATHER_ELEMENT).add(ix as usize)).SgPhysicalAddress = PhAddress;
            (*(core::ptr::addr_of_mut!((*sgList).SgElement) as *mut USBPORT_SCATTER_GATHER_ELEMENT).add(ix as usize)).SgTransferLength = ElementLength as ULONG;
            (*(core::ptr::addr_of_mut!((*sgList).SgElement) as *mut USBPORT_SCATTER_GATHER_ELEMENT).add(ix as usize)).SgOffset = CurrentLength
                .wrapping_add((TransferLength as SIZE_T).wrapping_sub(SgCurrentLength))
                as ULONG;

            PhAddress.u.LowPart = (PhAddress.u.LowPart as u64)
                .wrapping_add(ElementLength as u64) as ULONG;
            SgCurrentLength = (SgCurrentLength as u64)
                .wrapping_sub(ElementLength as u64) as SIZE_T;

            ix = ix.wrapping_add(1);

            if !(SgCurrentLength != 0) {
                break;
            }
        }

        if PhAddr.QuadPart == PhAddress.QuadPart {
            // DPRINT removed
            // ASSERT(FALSE)
        }

        PhAddr = PhAddress;

        CurrentLength = (CurrentLength as u64)
            .wrapping_add(TransferLength as u64) as SIZE_T;
        CurrentVa = (CurrentVa as u64)
            .wrapping_add(TransferLength as u64) as ULONG_PTR;

        TransferLength = ((*Transfer).TransferParameters.TransferBufferLength as SIZE_T)
            .wrapping_sub(CurrentLength) as ULONG;

        if !(CurrentLength
            != (*Transfer).TransferParameters.TransferBufferLength as SIZE_T)
        {
            break;
        }
    }

    (*sgList).SgElementCount = ix;

    if (*Endpoint).EndpointProperties.DeviceSpeed as u32 == UsbHighSpeed as u32 {
        (*Transfer).Flags |= TRANSFER_FLAG_HIGH_SPEED as ULONG;
    }

    (*Transfer).Flags |= TRANSFER_FLAG_DMA_MAPPED as ULONG;

    if (*Transfer).Flags as ULONG & TRANSFER_FLAG_ISO as ULONG == 0 {
        KeAcquireSpinLock(
            (core::ptr::addr_of_mut!((*Endpoint).EndpointSpinLock) as *mut usize),
            core::ptr::addr_of_mut!((*Endpoint).EndpointOldIrql),
        );

        USBPORT_SplitTransfer(FdoDevice, Endpoint, Transfer, core::ptr::addr_of_mut!(List));

        while !IsListEmpty(core::ptr::addr_of_mut!(List)) {
            transfer = (List.Flink as usize
                - core::mem::offset_of!(USBPORT_TRANSFER, TransferLink))
                as PUSBPORT_TRANSFER;

            RemoveHeadList(core::ptr::addr_of_mut!(List));
            InsertTailList(
                core::ptr::addr_of_mut!((*Endpoint).TransferList),
                core::ptr::addr_of_mut!((*transfer).TransferLink),
            );
        }

        KeReleaseSpinLock(
            (core::ptr::addr_of_mut!((*Endpoint).EndpointSpinLock) as *mut usize),
            (*Endpoint).EndpointOldIrql,
        );
    } else {
        USBDStatus = USBPORT_InitializeIsoTransfer(
            FdoDevice,
            core::ptr::addr_of_mut!((*Urb).__bindgen_anon_1.UrbIsochronousTransfer),
            Transfer,
        );

        if USBDStatus != USBD_STATUS_SUCCESS {
            KeAcquireSpinLock(
                (core::ptr::addr_of_mut!((*Endpoint).EndpointSpinLock) as *mut usize),
                core::ptr::addr_of_mut!((*Endpoint).EndpointOldIrql),
            );

            USBPORT_QueueDoneTransfer(Transfer, USBDStatus);

            KeReleaseSpinLock(
                (core::ptr::addr_of_mut!((*Endpoint).EndpointSpinLock) as *mut usize),
                (*Endpoint).EndpointOldIrql,
            );
        }
    }

    DeviceHandle = (*Urb).__bindgen_anon_1.UrbHeader.UsbdDeviceHandle as PUSBPORT_DEVICE_HANDLE;
    InterlockedDecrement(core::ptr::addr_of_mut!((*DeviceHandle).DeviceHandleLock) as *mut core::ffi::c_long);

    if USBPORT_EndpointWorker(Endpoint, 0 as BOOLEAN) != 0 {
        USBPORT_InvalidateEndpointHandler(
            FdoDevice,
            Endpoint,
            INVALIDATE_ENDPOINT_WORKER_THREAD as ULONG,
        );
    }

    return DeallocateObjectKeepRegisters;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_FlushMapTransfers(FdoDevice: PDEVICE_OBJECT) {

    let mut FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let mut MapTransferList: PLIST_ENTRY;
    let mut Transfer: PUSBPORT_TRANSFER;
    let mut NumMapRegisters: ULONG;
    let mut Mdl: PMDL;
    let mut TransferBufferLength: SIZE_T;
    let mut VirtualAddr: ULONG_PTR;
    let mut OldIrql: KIRQL = 0;
    let mut Status: NTSTATUS;
    let DmaOperations: PDMA_OPERATIONS;

    DPRINT_CORE(b"USBPORT_FlushMapTransfers: ...\n\0".as_ptr() as *const i8);

    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;
    DmaOperations = (*(*FdoExtension).DmaAdapter).DmaOperations as PDMA_OPERATIONS;

    KeRaiseIrql(DISPATCH_LEVEL as KIRQL, core::ptr::addr_of_mut!(OldIrql));

    while TRUE != 0 {
        MapTransferList = core::ptr::addr_of_mut!((*FdoExtension).MapTransferList) as PLIST_ENTRY;

        if IsListEmpty(core::ptr::addr_of_mut!((*FdoExtension).MapTransferList)) {
            KeLowerIrql(OldIrql);
            return;
        }

        Transfer = (((*MapTransferList).Flink as ULONG_PTR)
            .wrapping_sub(
                core::mem::offset_of!(USBPORT_TRANSFER, TransferLink) as ULONG_PTR,
            )) as *mut USBPORT_TRANSFER as PUSBPORT_TRANSFER;

        RemoveHeadList(MapTransferList);

        Mdl = (*(*Transfer).Urb).__bindgen_anon_1.UrbControlTransfer.TransferBufferMDL;
        TransferBufferLength = (*Transfer).TransferParameters.TransferBufferLength as SIZE_T;
        VirtualAddr = MmGetMdlVirtualAddress(Mdl) as ULONG_PTR;

        NumMapRegisters = ((ADDRESS_AND_SIZE_TO_SPAN_PAGES(VirtualAddr as PVOID, (TransferBufferLength as ULONG as u32) as usize)) as u32);

        (*Transfer).NumberOfMapRegisters = NumMapRegisters;

        Status = ((*DmaOperations).AllocateAdapterChannel.expect("non-null function pointer"))(
            (*FdoExtension).DmaAdapter,
            FdoDevice,
            NumMapRegisters,
            cb!(USBPORT_MapTransfer),
            Transfer as PVOID,
        );

        if !NT_SUCCESS(Status) {
            if 0 == 0 {
                RtlAssert(
                    b"FALSE\0".as_ptr() as *const i8 as PVOID,
                    b"drivers/usb/usbport/usbport.c\0".as_ptr() as *const i8 as PVOID,
                    2570 as ULONG,
                    core::ptr::null_mut::<CHAR>(),
                );
            } else {
            };
        }
    }

    #[allow(unreachable_code)]
    KeLowerIrql(OldIrql);
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_AllocateTransfer(
    FdoDevice: PDEVICE_OBJECT,
    Urb: PURB,
    DeviceHandle: PUSBPORT_DEVICE_HANDLE,
    Irp: PIRP,
    Event: PRKEVENT,
) -> USBD_STATUS {
    let mut FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let mut TransferLength: SIZE_T;
    let mut Mdl: PMDL;
    let mut VirtualAddr: ULONG_PTR;
    let mut PagesNeed: ULONG = 0 as ULONG;
    let mut PortTransferLength: SIZE_T;
    let mut FullTransferLength: SIZE_T;
    let mut Transfer: PUSBPORT_TRANSFER;
    let mut PipeHandle: PUSBPORT_PIPE_HANDLE;
    let mut USBDStatus: USBD_STATUS;
    let mut IsoBlockLen: SIZE_T = 0 as SIZE_T;

    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;

    TransferLength = (*Urb).__bindgen_anon_1.UrbControlTransfer.TransferBufferLength as SIZE_T;
    PipeHandle = (*Urb).__bindgen_anon_1.UrbControlTransfer.PipeHandle as PUSBPORT_PIPE_HANDLE;

    if TransferLength != 0 {
        Mdl = (*Urb).__bindgen_anon_1.UrbControlTransfer.TransferBufferMDL;
        VirtualAddr = MmGetMdlVirtualAddress(Mdl) as ULONG_PTR;

        PagesNeed = ADDRESS_AND_SIZE_TO_SPAN_PAGES(VirtualAddr as PVOID, TransferLength) as ULONG;
        if PagesNeed > 0 as ULONG {
            PagesNeed = PagesNeed.wrapping_sub(1);
        }
    }

    if (*Urb).__bindgen_anon_1.UrbHeader.Function as i32 == (URB_FUNCTION_ISOCH_TRANSFER as i32) {
        // DPRINT1("USBPORT_AllocateTransfer: ISOCH_TRANSFER UNIMPLEMENTED. FIXME\n");
        // IsoBlockLen = sizeof(USBPORT_ISO_BLOCK) + ...
    }

    PortTransferLength = (core::mem::size_of::<USBPORT_TRANSFER>() as SIZE_T)
        .wrapping_add(
            (PagesNeed as SIZE_T)
                .wrapping_mul(core::mem::size_of::<USBPORT_SCATTER_GATHER_ELEMENT>() as SIZE_T),
        )
        .wrapping_add(IsoBlockLen);

    FullTransferLength = PortTransferLength.wrapping_add(
        ((*(*FdoExtension).MiniPortInterface).Packet.MiniPortTransferSize as usize),
    );

    Transfer = ExAllocatePoolWithTag(NonPagedPool, FullTransferLength, USB_PORT_TAG as ULONG)
        as PUSBPORT_TRANSFER;

    if Transfer.is_null() {
        // DPRINT1("USBPORT_AllocateTransfer: Transfer not allocated!\n");
        return USBD_STATUS_INSUFFICIENT_RESOURCES;
    }

    RtlZeroMemory(Transfer as PVOID, FullTransferLength);

    (*Transfer).Irp = Irp;
    (*Transfer).Urb = Urb;
    (*Transfer).Endpoint = (*PipeHandle).Endpoint as PUSBPORT_ENDPOINT;
    (*Transfer).Event = Event;
    (*Transfer).PortTransferLength = (PortTransferLength as _);
    (*Transfer).FullTransferLength = (FullTransferLength as _);
    (*Transfer).IsoBlockPtr = core::ptr::null_mut();
    (*Transfer).Period = 0 as ULONG;
    (*Transfer).ParentTransfer = Transfer;

    if IsoBlockLen != 0 {
        (*Transfer).IsoBlockPtr = (Transfer as ULONG_PTR)
            .wrapping_add(PortTransferLength as ULONG_PTR)
            .wrapping_sub(IsoBlockLen as ULONG_PTR) as PVOID
            as PUSBPORT_ISO_BLOCK;

        (*Transfer).Period = (*(*PipeHandle).Endpoint).EndpointProperties.Period as ULONG;
        (*Transfer).Flags |= TRANSFER_FLAG_ISO as ULONG;
    }

    (*Transfer).MiniportTransfer = (Transfer as ULONG_PTR)
        .wrapping_add(PortTransferLength as ULONG_PTR) as PVOID;

    KeInitializeSpinLock(core::ptr::addr_of_mut!((*Transfer).TransferSpinLock));

    (*Urb).__bindgen_anon_1.UrbControlTransfer.hca.Reserved8[0] = Transfer as PVOID;
    (*Urb).__bindgen_anon_1.UrbHeader.UsbdFlags |= USBD_FLAG_ALLOCATED_TRANSFER as ULONG;

    USBDStatus = USBD_STATUS_SUCCESS;

    return USBDStatus;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_Dispatch(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {

    const USBPORT_PNP_STATE_FAILED: u32 = 0x00000004; // FIXED 2026-07-05: translation invented 0x10; verified vs usbport.h/usbmport.h

    let mut DeviceExtension: PUSBPORT_COMMON_DEVICE_EXTENSION;
    let mut IoStack: PIO_STACK_LOCATION;
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    DeviceExtension = (*DeviceObject).DeviceExtension as PUSBPORT_COMMON_DEVICE_EXTENSION;
    IoStack = IoGetCurrentIrpStackLocation(Irp);

    if (*DeviceExtension).PnpStateFlags as u32 & USBPORT_PNP_STATE_FAILED != 0 {
        DbgBreakPoint();
    }

    match (*IoStack).MajorFunction {
        IRP_MJ_DEVICE_CONTROL => {
            if (*DeviceExtension).IsPDO != 0 {
                Status = USBPORT_PdoDeviceControl(DeviceObject, Irp);
            } else {
                Status = USBPORT_FdoDeviceControl(DeviceObject, Irp);
            }
        }
        IRP_MJ_INTERNAL_DEVICE_CONTROL => {
            if (*DeviceExtension).IsPDO != 0 {
                Status = USBPORT_PdoInternalDeviceControl(DeviceObject, Irp);
            } else {
                Status = USBPORT_FdoInternalDeviceControl(DeviceObject, Irp);
            }
        }
        IRP_MJ_POWER => {
            if (*DeviceExtension).IsPDO != 0 {
                Status = USBPORT_PdoPower(DeviceObject, Irp);
            } else {
                Status = USBPORT_FdoPower(DeviceObject, Irp);
            }
        }
        IRP_MJ_SYSTEM_CONTROL => {
            if (*DeviceExtension).IsPDO != 0 {
                Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
                IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
            } else {
                IoSkipCurrentIrpStackLocation(Irp);
                Status = IoCallDriver((*DeviceExtension).LowerDevice, Irp);
            }
        }
        IRP_MJ_PNP => {
            ntkrnl_sys::driver_ddk::DbgPrint(
                b"[USBPORT] Dispatch PNP minor=0x%x IsPDO=%u\n\0".as_ptr() as *const i8,
                (*IoStack).MinorFunction as u32,
                (*DeviceExtension).IsPDO as u32,
            );
            if (*DeviceExtension).IsPDO != 0 {
                Status = USBPORT_PdoPnP(DeviceObject, Irp);
            } else {
                Status = USBPORT_FdoPnP(DeviceObject, Irp);
            }
        }
        IRP_MJ_CREATE | IRP_MJ_CLOSE => {
            (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
            IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
        }
        _ => {
            if (*DeviceExtension).IsPDO != 0 {
            } else {
            }
            Status = STATUS_INVALID_DEVICE_REQUEST;
            (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
            IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
        }
    }

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_GetHciMn() -> ULONG {
    const USBPORT_HCI_MN: u32 = 0x10000001;
    return USBPORT_HCI_MN as ULONG;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_RegisterUSBPortDriver(
    DriverObject: PDRIVER_OBJECT,
    Version: ULONG,
    RegPacket: PUSBPORT_REGISTRATION_PACKET,
) -> NTSTATUS {
    let mut MiniPortInterface: PUSBPORT_MINIPORT_INTERFACE = core::ptr::null_mut();

    if Version < USB10_MINIPORT_INTERFACE_VERSION as ULONG {
        return STATUS_UNSUCCESSFUL;
    }

    if USBPORT_Initialized == 0 {
        InitializeListHead(core::ptr::addr_of_mut!(USBPORT_MiniPortDrivers));
        InitializeListHead(core::ptr::addr_of_mut!(USBPORT_USB1FdoList));
        InitializeListHead(core::ptr::addr_of_mut!(USBPORT_USB2FdoList));

        KeInitializeSpinLock((core::ptr::addr_of_mut!(USBPORT_SpinLock) as *mut _));
        USBPORT_Initialized = 1 as BOOLEAN;
    }

    MiniPortInterface = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<USBPORT_MINIPORT_INTERFACE>() as SIZE_T,
        USB_PORT_TAG as ULONG,
    ) as PUSBPORT_MINIPORT_INTERFACE;
    if MiniPortInterface.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    RtlZeroMemory(
        MiniPortInterface as *mut core::ffi::c_void,
        core::mem::size_of::<USBPORT_MINIPORT_INTERFACE>() as SIZE_T,
    );

    (*MiniPortInterface).DriverObject = DriverObject;
    (*MiniPortInterface).DriverUnload = (*DriverObject).DriverUnload;
    (*MiniPortInterface).Version = Version;

    ExInterlockedInsertTailList(
        core::ptr::addr_of_mut!(USBPORT_MiniPortDrivers),
        core::ptr::addr_of_mut!((*MiniPortInterface).DriverLink),
        (core::ptr::addr_of_mut!(USBPORT_SpinLock) as *mut _),
    );

    (*(*DriverObject).DriverExtension).AddDevice = cb!(USBPORT_AddDevice);
    (*DriverObject).DriverUnload = cb!(USBPORT_Unload);

    (*DriverObject).MajorFunction[IRP_MJ_CREATE as usize] = cb!(USBPORT_Dispatch);
    (*DriverObject).MajorFunction[IRP_MJ_CLOSE as usize] = cb!(USBPORT_Dispatch);
    (*DriverObject).MajorFunction[IRP_MJ_DEVICE_CONTROL as usize] = cb!(USBPORT_Dispatch);
    (*DriverObject).MajorFunction[IRP_MJ_INTERNAL_DEVICE_CONTROL as usize] = cb!(USBPORT_Dispatch);
    (*DriverObject).MajorFunction[IRP_MJ_PNP as usize] = cb!(USBPORT_Dispatch);
    (*DriverObject).MajorFunction[IRP_MJ_POWER as usize] = cb!(USBPORT_Dispatch);
    (*DriverObject).MajorFunction[IRP_MJ_SYSTEM_CONTROL as usize] = cb!(USBPORT_Dispatch);

    (*RegPacket).UsbPortDbgPrint = Some(USBPORT_DbgPrint);
    (*RegPacket).UsbPortTestDebugBreak = Some(USBPORT_TestDebugBreak);
    (*RegPacket).UsbPortAssertFailure = Some(USBPORT_AssertFailure);
    (*RegPacket).UsbPortGetMiniportRegistryKeyValue = cb!(USBPORT_GetMiniportRegistryKeyValue);
    (*RegPacket).UsbPortInvalidateRootHub = Some(USBPORT_InvalidateRootHub);
    (*RegPacket).UsbPortInvalidateEndpoint = Some(USBPORT_InvalidateEndpoint);
    (*RegPacket).UsbPortCompleteTransfer = Some(USBPORT_MiniportCompleteTransfer);
    (*RegPacket).UsbPortCompleteIsoTransfer = Some(USBPORT_CompleteIsoTransfer);
    (*RegPacket).UsbPortLogEntry = Some(USBPORT_LogEntry);
    (*RegPacket).UsbPortGetMappedVirtualAddress = Some(USBPORT_GetMappedVirtualAddress);
    (*RegPacket).UsbPortRequestAsyncCallback = cb!(USBPORT_RequestAsyncCallback);
    (*RegPacket).UsbPortReadWriteConfigSpace = Some(USBPORT_ReadWriteConfigSpace);
    (*RegPacket).UsbPortWait = Some(USBPORT_Wait);
    (*RegPacket).UsbPortInvalidateController = Some(USBPORT_InvalidateController);
    (*RegPacket).UsbPortBugCheck = Some(USBPORT_BugCheck);
    (*RegPacket).UsbPortNotifyDoubleBuffer = cb!(USBPORT_NotifyDoubleBuffer);

    RtlCopyMemory(
        core::ptr::addr_of_mut!((*MiniPortInterface).Packet) as *mut core::ffi::c_void,
        RegPacket as *const core::ffi::c_void,
        core::mem::size_of::<USBPORT_REGISTRATION_PACKET>() as SIZE_T,
    );

    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn DriverEntry(
    DriverObject: PDRIVER_OBJECT,
    RegistryPath: PUNICODE_STRING,
) -> NTSTATUS {
    return STATUS_SUCCESS;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS USB Port Driver
//  * LICENSE:     GPL-2.0+ (https://spdx.org/licenses/GPL-2.0+)
//  * PURPOSE:     USBPort main driver functions
//  * COPYRIGHT:   Copyright 2017 Vadim Galyant <vgal@rambler.ru>
//  */
//
// #include "usbport.h"
//
// #define NDEBUG
// #include <debug.h>
//
// #define NDEBUG_USBPORT_CORE
// #define NDEBUG_USBPORT_INTERRUPT
// #define NDEBUG_USBPORT_TIMER
// #include "usbdebug.h"
//
// LIST_ENTRY USBPORT_MiniPortDrivers = {NULL, NULL};
// LIST_ENTRY USBPORT_USB1FdoList = {NULL, NULL};
// LIST_ENTRY USBPORT_USB2FdoList = {NULL, NULL};
//
// KSPIN_LOCK USBPORT_SpinLock;
// BOOLEAN USBPORT_Initialized = FALSE;
//
// PDEVICE_OBJECT
// NTAPI
// USBPORT_FindUSB2Controller(IN PDEVICE_OBJECT FdoDevice)
// {
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     PUSBPORT_DEVICE_EXTENSION USB2FdoExtension;
//     KIRQL OldIrql;
//     PLIST_ENTRY USB2FdoEntry;
//     PDEVICE_OBJECT USB2FdoDevice = NULL;
//
//     DPRINT("USBPORT_FindUSB2Controller: FdoDevice - %p\n", FdoDevice);
//
//     FdoExtension = FdoDevice->DeviceExtension;
//
//     KeAcquireSpinLock(&USBPORT_SpinLock, &OldIrql);
//
//     USB2FdoEntry = USBPORT_USB2FdoList.Flink;
//
//     while (USB2FdoEntry && USB2FdoEntry != &USBPORT_USB2FdoList)
//     {
//         USB2FdoExtension = CONTAINING_RECORD(USB2FdoEntry,
//                                              USBPORT_DEVICE_EXTENSION,
//                                              ControllerLink);
//
//         if (USB2FdoExtension->BusNumber == FdoExtension->BusNumber &&
//             USB2FdoExtension->PciDeviceNumber == FdoExtension->PciDeviceNumber)
//         {
//             USB2FdoDevice = USB2FdoExtension->CommonExtension.SelfDevice;
//             break;
//         }
//
//         USB2FdoEntry = USB2FdoEntry->Flink;
//     }
//
//     KeReleaseSpinLock(&USBPORT_SpinLock, OldIrql);
//
//     return USB2FdoDevice;
// }
//
// VOID
// NTAPI
// USBPORT_AddUSB1Fdo(IN PDEVICE_OBJECT FdoDevice)
// {
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//
//     DPRINT("USBPORT_AddUSB1Fdo: FdoDevice - %p\n", FdoDevice);
//
//     FdoExtension = FdoDevice->DeviceExtension;
//     FdoExtension->Flags |= USBPORT_FLAG_REGISTERED_FDO;
//
//     ExInterlockedInsertTailList(&USBPORT_USB1FdoList,
//                                 &FdoExtension->ControllerLink,
//                                 &USBPORT_SpinLock);
// }
//
// VOID
// NTAPI
// USBPORT_AddUSB2Fdo(IN PDEVICE_OBJECT FdoDevice)
// {
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//
//     DPRINT("USBPORT_AddUSB2Fdo: FdoDevice - %p\n", FdoDevice);
//
//     FdoExtension = FdoDevice->DeviceExtension;
//     FdoExtension->Flags |= USBPORT_FLAG_REGISTERED_FDO;
//
//     ExInterlockedInsertTailList(&USBPORT_USB2FdoList,
//                                 &FdoExtension->ControllerLink,
//                                 &USBPORT_SpinLock);
// }
//
// VOID
// NTAPI
// USBPORT_RemoveUSBxFdo(IN PDEVICE_OBJECT FdoDevice)
// {
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     KIRQL OldIrql;
//
//     DPRINT("USBPORT_RemoveUSBxFdo: FdoDevice - %p\n", FdoDevice);
//
//     FdoExtension = FdoDevice->DeviceExtension;
//
//     KeAcquireSpinLock(&USBPORT_SpinLock, &OldIrql);
//     RemoveEntryList(&FdoExtension->ControllerLink);
//     KeReleaseSpinLock(&USBPORT_SpinLock, OldIrql);
//
//     FdoExtension->Flags &= ~USBPORT_FLAG_REGISTERED_FDO;
//
//     FdoExtension->ControllerLink.Flink = NULL;
//     FdoExtension->ControllerLink.Blink = NULL;
// }
//
// BOOLEAN
// NTAPI
// USBPORT_IsCompanionFdoExtension(IN PDEVICE_OBJECT USB2FdoDevice,
//                                 IN PUSBPORT_DEVICE_EXTENSION USB1FdoExtension)
// {
//     PUSBPORT_DEVICE_EXTENSION USB2FdoExtension;
//
//     DPRINT("USBPORT_IsCompanionFdoExtension: USB2Fdo - %p, USB1FdoExtension - %p\n",
//            USB2FdoDevice,
//            USB1FdoExtension);
//
//     USB2FdoExtension = USB2FdoDevice->DeviceExtension;
//
//     return USB2FdoExtension->BusNumber == USB1FdoExtension->BusNumber &&
//            USB2FdoExtension->PciDeviceNumber == USB1FdoExtension->PciDeviceNumber;
// }
//
// PDEVICE_RELATIONS
// NTAPI
// USBPORT_FindCompanionControllers(IN PDEVICE_OBJECT USB2FdoDevice,
//                                  IN BOOLEAN IsObRefer,
//                                  IN BOOLEAN IsFDOsReturned)
// {
//     PLIST_ENTRY USB1FdoList;
//     PUSBPORT_DEVICE_EXTENSION USB1FdoExtension;
//     ULONG NumControllers = 0;
//     PDEVICE_OBJECT * Entry;
//     PDEVICE_RELATIONS ControllersList = NULL;
//     KIRQL OldIrql;
//
//     DPRINT("USBPORT_FindCompanionControllers: USB2Fdo - %p, IsObRefer - %x, IsFDOs - %x\n",
//            USB2FdoDevice,
//            IsObRefer,
//            IsFDOsReturned);
//
//     KeAcquireSpinLock(&USBPORT_SpinLock, &OldIrql);
//
//     USB1FdoList = USBPORT_USB1FdoList.Flink;
//
//     while (USB1FdoList && USB1FdoList != &USBPORT_USB1FdoList)
//     {
//         USB1FdoExtension = CONTAINING_RECORD(USB1FdoList,
//                                              USBPORT_DEVICE_EXTENSION,
//                                              ControllerLink);
//
//         if (USB1FdoExtension->Flags & USBPORT_FLAG_COMPANION_HC &&
//             USBPORT_IsCompanionFdoExtension(USB2FdoDevice, USB1FdoExtension))
//         {
//             ++NumControllers;
//         }
//
//         USB1FdoList = USB1FdoExtension->ControllerLink.Flink;
//     }
//
//     DPRINT("USBPORT_FindCompanionControllers: NumControllers - %x\n",
//            NumControllers);
//
//     if (!NumControllers)
//     {
//         goto Exit;
//     }
//
//     ControllersList = ExAllocatePoolWithTag(NonPagedPool,
//                                             NumControllers * sizeof(DEVICE_RELATIONS),
//                                             USB_PORT_TAG);
//
//     if (!ControllersList)
//     {
//         goto Exit;
//     }
//
//     RtlZeroMemory(ControllersList, NumControllers * sizeof(DEVICE_RELATIONS));
//
//     ControllersList->Count = NumControllers;
//
//     USB1FdoList = USBPORT_USB1FdoList.Flink;
//
//     Entry = &ControllersList->Objects[0];
//
//     while (USB1FdoList && USB1FdoList != &USBPORT_USB1FdoList)
//     {
//         USB1FdoExtension = CONTAINING_RECORD(USB1FdoList,
//                                              USBPORT_DEVICE_EXTENSION,
//                                              ControllerLink);
//
//         if (USB1FdoExtension->Flags & USBPORT_FLAG_COMPANION_HC &&
//             USBPORT_IsCompanionFdoExtension(USB2FdoDevice, USB1FdoExtension))
//         {
//             *Entry = USB1FdoExtension->CommonExtension.LowerPdoDevice;
//
//             if (IsObRefer)
//             {
//                 ObReferenceObject(USB1FdoExtension->CommonExtension.LowerPdoDevice);
//             }
//
//             if (IsFDOsReturned)
//             {
//                 *Entry = USB1FdoExtension->CommonExtension.SelfDevice;
//             }
//
//             ++Entry;
//         }
//
//         USB1FdoList = USB1FdoExtension->ControllerLink.Flink;
//     }
//
// Exit:
//
//     KeReleaseSpinLock(&USBPORT_SpinLock, OldIrql);
//
//     return ControllersList;
// }
//
// MPSTATUS
// NTAPI
// USBPORT_NtStatusToMpStatus(NTSTATUS NtStatus)
// {
//     DPRINT("USBPORT_NtStatusToMpStatus: NtStatus - %x\n", NtStatus);
//
//     if (NtStatus == STATUS_SUCCESS)
//     {
//         return MP_STATUS_SUCCESS;
//     }
//     else
//     {
//         return MP_STATUS_UNSUCCESSFUL;
//     }
// }
//
// NTSTATUS
// NTAPI
// USBPORT_SetRegistryKeyValue(IN PDEVICE_OBJECT DeviceObject,
//                             IN BOOL UseDriverKey,
//                             IN ULONG Type,
//                             IN PCWSTR ValueNameString,
//                             IN PVOID Data,
//                             IN ULONG DataSize)
// {
//     UNICODE_STRING ValueName;
//     HANDLE KeyHandle;
//     NTSTATUS Status;
//
//     DPRINT("USBPORT_SetRegistryKeyValue: ValueNameString - %S\n",
//            ValueNameString);
//
//     if (UseDriverKey)
//     {
//         Status = IoOpenDeviceRegistryKey(DeviceObject,
//                                          PLUGPLAY_REGKEY_DRIVER,
//                                          STANDARD_RIGHTS_ALL,
//                                          &KeyHandle);
//     }
//     else
//     {
//         Status = IoOpenDeviceRegistryKey(DeviceObject,
//                                          PLUGPLAY_REGKEY_DEVICE,
//                                          STANDARD_RIGHTS_ALL,
//                                          &KeyHandle);
//     }
//
//     if (NT_SUCCESS(Status))
//     {
//         RtlInitUnicodeString(&ValueName, ValueNameString);
//
//         Status = ZwSetValueKey(KeyHandle,
//                                &ValueName,
//                                0,
//                                Type,
//                                Data,
//                                DataSize);
//
//         ZwClose(KeyHandle);
//     }
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// USBPORT_GetRegistryKeyValueFullInfo(IN PDEVICE_OBJECT FdoDevice,
//                                     IN PDEVICE_OBJECT PdoDevice,
//                                     IN BOOL UseDriverKey,
//                                     IN PCWSTR SourceString,
//                                     IN ULONG LengthStr,
//                                     IN PVOID Buffer,
//                                     IN ULONG BufferLength)
// {
//     NTSTATUS Status;
//     PKEY_VALUE_FULL_INFORMATION KeyValue;
//     UNICODE_STRING ValueName;
//     HANDLE KeyHandle;
//     ULONG LengthKey;
//
//     DPRINT("USBPORT_GetRegistryKeyValue: UseDriverKey - %x, SourceString - %S, LengthStr - %x, Buffer - %p, BufferLength - %x\n",
//            UseDriverKey,
//            SourceString,
//            LengthStr,
//            Buffer,
//            BufferLength);
//
//     if (UseDriverKey)
//     {
//         Status = IoOpenDeviceRegistryKey(PdoDevice,
//                                          PLUGPLAY_REGKEY_DRIVER,
//                                          STANDARD_RIGHTS_ALL,
//                                          &KeyHandle);
//     }
//     else
//     {
//         Status = IoOpenDeviceRegistryKey(PdoDevice,
//                                          PLUGPLAY_REGKEY_DEVICE,
//                                          STANDARD_RIGHTS_ALL,
//                                          &KeyHandle);
//     }
//
//     if (NT_SUCCESS(Status))
//     {
//         RtlInitUnicodeString(&ValueName, SourceString);
//
//         LengthKey = sizeof(KEY_VALUE_FULL_INFORMATION) +
//                     LengthStr +
//                     BufferLength;
//
//         KeyValue = ExAllocatePoolWithTag(PagedPool,
//                                          LengthKey,
//                                          USB_PORT_TAG);
//
//         if (KeyValue)
//         {
//             RtlZeroMemory(KeyValue, LengthKey);
//
//             Status = ZwQueryValueKey(KeyHandle,
//                                      &ValueName,
//                                      KeyValueFullInformation,
//                                      KeyValue,
//                                      LengthKey,
//                                      &LengthKey);
//
//             if (NT_SUCCESS(Status))
//             {
//                 RtlCopyMemory(Buffer,
//                               (PUCHAR)KeyValue + KeyValue->DataOffset,
//                               BufferLength);
//             }
//
//             ExFreePoolWithTag(KeyValue, USB_PORT_TAG);
//         }
//
//         ZwClose(KeyHandle);
//     }
//
//     return Status;
// }
//
// MPSTATUS
// NTAPI
// USBPORT_GetMiniportRegistryKeyValue(IN PVOID MiniPortExtension,
//                                     IN BOOL UseDriverKey,
//                                     IN PCWSTR SourceString,
//                                     IN SIZE_T LengthStr,
//                                     IN PVOID Buffer,
//                                     IN SIZE_T BufferLength)
// {
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     PDEVICE_OBJECT FdoDevice;
//     NTSTATUS Status;
//
//     DPRINT("USBPORT_GetMiniportRegistryKeyValue: MiniPortExtension - %p, UseDriverKey - %x, SourceString - %S, LengthStr - %x, Buffer - %p, BufferLength - %x\n",
//            MiniPortExtension,
//            UseDriverKey,
//            SourceString,
//            LengthStr,
//            Buffer,
//            BufferLength);
//
//     FdoExtension = (PUSBPORT_DEVICE_EXTENSION)((ULONG_PTR)MiniPortExtension -
//                                                sizeof(USBPORT_DEVICE_EXTENSION));
//
//     FdoDevice = FdoExtension->CommonExtension.SelfDevice;
//
//     Status = USBPORT_GetRegistryKeyValueFullInfo(FdoDevice,
//                                                  FdoExtension->CommonExtension.LowerPdoDevice,
//                                                  UseDriverKey,
//                                                  SourceString,
//                                                  LengthStr,
//                                                  Buffer,
//                                                  BufferLength);
//
//     return USBPORT_NtStatusToMpStatus(Status);
// }
//
// NTSTATUS
// NTAPI
// USBPORT_GetSetConfigSpaceData(IN PDEVICE_OBJECT FdoDevice,
//                               IN BOOLEAN IsReadData,
//                               IN PVOID Buffer,
//                               IN ULONG Offset,
//                               IN ULONG Length)
// {
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     ULONG BytesReadWrite;
//
//     DPRINT("USBPORT_GetSetConfigSpaceData ...\n");
//
//     FdoExtension = FdoDevice->DeviceExtension;
//
//     BytesReadWrite = Length;
//
//     if (IsReadData)
//     {
//         RtlZeroMemory(Buffer, Length);
//
//         BytesReadWrite = (*FdoExtension->BusInterface.GetBusData)
//                           (FdoExtension->BusInterface.Context,
//                            PCI_WHICHSPACE_CONFIG,
//                            Buffer,
//                            Offset,
//                            Length);
//     }
//     else
//     {
//         BytesReadWrite = (*FdoExtension->BusInterface.SetBusData)
//                           (FdoExtension->BusInterface.Context,
//                            PCI_WHICHSPACE_CONFIG,
//                            Buffer,
//                            Offset,
//                            Length);
//     }
//
//     if (BytesReadWrite == Length)
//     {
//         return STATUS_SUCCESS;
//     }
//
//     return STATUS_UNSUCCESSFUL;
// }
//
// MPSTATUS
// NTAPI
// USBPORT_ReadWriteConfigSpace(IN PVOID MiniPortExtension,
//                              IN BOOLEAN IsReadData,
//                              IN PVOID Buffer,
//                              IN ULONG Offset,
//                              IN ULONG Length)
// {
//     NTSTATUS Status;
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     PDEVICE_OBJECT FdoDevice;
//
//     DPRINT("USBPORT_ReadWriteConfigSpace: ...\n");
//
//     //FdoExtension->MiniPortExt = (PVOID)((ULONG_PTR)FdoExtension + sizeof(USBPORT_DEVICE_EXTENSION));
//     FdoExtension = (PUSBPORT_DEVICE_EXTENSION)((ULONG_PTR)MiniPortExtension -
//                                                sizeof(USBPORT_DEVICE_EXTENSION));
//
//     FdoDevice = FdoExtension->CommonExtension.SelfDevice;
//
//     Status = USBPORT_GetSetConfigSpaceData(FdoDevice,
//                                            IsReadData,
//                                            Buffer,
//                                            Offset,
//                                            Length);
//
//     return USBPORT_NtStatusToMpStatus(Status);
// }
//
// NTSTATUS
// NTAPI
// USBPORT_USBDStatusToNtStatus(IN PURB Urb,
//                              IN USBD_STATUS USBDStatus)
// {
//     NTSTATUS Status;
//
//     if (USBD_ERROR(USBDStatus))
//     {
//         DPRINT1("USBPORT_USBDStatusToNtStatus: Urb - %p, USBDStatus - %x\n",
//                 Urb,
//                 USBDStatus);
//     }
//
//     if (Urb)
//         Urb->UrbHeader.Status = USBDStatus;
//
//     switch (USBDStatus)
//     {
//         case USBD_STATUS_SUCCESS:
//             Status = STATUS_SUCCESS;
//             break;
//
//         case USBD_STATUS_INSUFFICIENT_RESOURCES:
//             Status = STATUS_INSUFFICIENT_RESOURCES;
//             break;
//
//         case USBD_STATUS_DEVICE_GONE:
//             Status = STATUS_DEVICE_NOT_CONNECTED;
//             break;
//
//         case USBD_STATUS_CANCELED:
//             Status = STATUS_CANCELLED;
//             break;
//
//         case USBD_STATUS_NOT_SUPPORTED:
//             Status = STATUS_NOT_SUPPORTED;
//             break;
//
//         case USBD_STATUS_INVALID_URB_FUNCTION:
//         case USBD_STATUS_INVALID_PARAMETER:
//         case USBD_STATUS_INVALID_PIPE_HANDLE:
//         case USBD_STATUS_BAD_START_FRAME:
//             Status = STATUS_INVALID_PARAMETER;
//             break;
//
//         default:
//             if (USBD_ERROR(USBDStatus))
//                 Status = STATUS_UNSUCCESSFUL;
//             else
//                 Status = STATUS_SUCCESS;
//
//             break;
//     }
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// USBPORT_Wait(IN PVOID MiniPortExtension,
//              IN ULONG Milliseconds)
// {
//     LARGE_INTEGER Interval = {{0, 0}};
//
//     DPRINT("USBPORT_Wait: Milliseconds - %x\n", Milliseconds);
//     Interval.QuadPart -= 10000 * Milliseconds + (KeQueryTimeIncrement() - 1);
//     return KeDelayExecutionThread(KernelMode, FALSE, &Interval);
// }
//
// VOID
// NTAPI
// USBPORT_MiniportInterrupts(IN PDEVICE_OBJECT FdoDevice,
//                            IN BOOLEAN IsEnable)
// {
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     PUSBPORT_REGISTRATION_PACKET Packet;
//     BOOLEAN IsLock;
//     KIRQL OldIrql;
//
//     DPRINT_INT("USBPORT_MiniportInterrupts: IsEnable - %p\n", IsEnable);
//
//     FdoExtension = FdoDevice->DeviceExtension;
//     Packet = &FdoExtension->MiniPortInterface->Packet;
//
//     IsLock = (Packet->MiniPortFlags & USB_MINIPORT_FLAGS_NOT_LOCK_INT) == 0;
//
//     if (IsLock)
//         KeAcquireSpinLock(&FdoExtension->MiniportSpinLock, &OldIrql);
//
//     if (IsEnable)
//     {
//         FdoExtension->Flags |= USBPORT_FLAG_INTERRUPT_ENABLED;
//         Packet->EnableInterrupts(FdoExtension->MiniPortExt);
//     }
//     else
//     {
//         Packet->DisableInterrupts(FdoExtension->MiniPortExt);
//         FdoExtension->Flags &= ~USBPORT_FLAG_INTERRUPT_ENABLED;
//     }
//
//     if (IsLock)
//         KeReleaseSpinLock(&FdoExtension->MiniportSpinLock, OldIrql);
// }
//
// VOID
// NTAPI
// USBPORT_SoftInterruptDpc(IN PRKDPC Dpc,
//                          IN PVOID DeferredContext,
//                          IN PVOID SystemArgument1,
//                          IN PVOID SystemArgument2)
// {
//     PDEVICE_OBJECT FdoDevice;
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//
//     DPRINT_INT("USBPORT_SoftInterruptDpc: ...\n");
//
//     FdoDevice = DeferredContext;
//     FdoExtension = FdoDevice->DeviceExtension;
//
//     if (!KeInsertQueueDpc(&FdoExtension->IsrDpc, NULL, (PVOID)1))
//     {
//         InterlockedDecrement(&FdoExtension->IsrDpcCounter);
//     }
// }
//
// VOID
// NTAPI
// USBPORT_SoftInterrupt(IN PDEVICE_OBJECT FdoDevice)
// {
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     LARGE_INTEGER DueTime = {{0, 0}};
//
//     DPRINT_INT("USBPORT_SoftInterrupt: ...\n");
//
//     FdoExtension = FdoDevice->DeviceExtension;
//
//     KeInitializeTimer(&FdoExtension->TimerSoftInterrupt);
//
//     KeInitializeDpc(&FdoExtension->SoftInterruptDpc,
//                     USBPORT_SoftInterruptDpc,
//                     FdoDevice);
//
//     DueTime.QuadPart -= 10000 + (KeQueryTimeIncrement() - 1);
//
//     KeSetTimer(&FdoExtension->TimerSoftInterrupt,
//                DueTime,
//                &FdoExtension->SoftInterruptDpc);
// }
//
// VOID
// NTAPI
// USBPORT_InvalidateControllerHandler(IN PDEVICE_OBJECT FdoDevice,
//                                     IN ULONG Type)
// {
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//
//     DPRINT_CORE("USBPORT_InvalidateControllerHandler: Invalidate Type - %x\n",
//                 Type);
//
//     FdoExtension = FdoDevice->DeviceExtension;
//
//     switch (Type)
//     {
//         case USBPORT_INVALIDATE_CONTROLLER_RESET:
//             DPRINT1("USBPORT_InvalidateControllerHandler: INVALIDATE_CONTROLLER_RESET UNIMPLEMENTED. FIXME.\n");
//             break;
//
//         case USBPORT_INVALIDATE_CONTROLLER_SURPRISE_REMOVE:
//             DPRINT1("USBPORT_InvalidateControllerHandler: INVALIDATE_CONTROLLER_SURPRISE_REMOVE UNIMPLEMENTED. FIXME.\n");
//             break;
//
//         case USBPORT_INVALIDATE_CONTROLLER_SOFT_INTERRUPT:
//             if (InterlockedIncrement(&FdoExtension->IsrDpcCounter))
//             {
//                 InterlockedDecrement(&FdoExtension->IsrDpcCounter);
//             }
//             else
//             {
//                 USBPORT_SoftInterrupt(FdoDevice);
//             }
//             break;
//     }
// }
//
// ULONG
// NTAPI
// USBPORT_InvalidateController(IN PVOID MiniPortExtension,
//                              IN ULONG Type)
// {
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     PDEVICE_OBJECT FdoDevice;
//
//     DPRINT("USBPORT_InvalidateController: Invalidate Type - %x\n", Type);
//
//     //FdoExtension->MiniPortExt = (PVOID)((ULONG_PTR)FdoExtension + sizeof(USBPORT_DEVICE_EXTENSION));
//     FdoExtension = (PUSBPORT_DEVICE_EXTENSION)((ULONG_PTR)MiniPortExtension -
//                                                sizeof(USBPORT_DEVICE_EXTENSION));
//     FdoDevice = FdoExtension->CommonExtension.SelfDevice;
//
//     USBPORT_InvalidateControllerHandler(FdoDevice, Type);
//
//     return 0;
// }
//
// ULONG
// NTAPI
// USBPORT_NotifyDoubleBuffer(IN PVOID MiniPortExtension,
//                            IN PVOID MiniPortTransfer,
//                            IN PVOID Buffer,
//                            IN SIZE_T Length)
// {
//     DPRINT1("USBPORT_NotifyDoubleBuffer: UNIMPLEMENTED. FIXME.\n");
//     return 0;
// }
//
// VOID
// NTAPI
// USBPORT_WorkerRequestDpc(IN PRKDPC Dpc,
//                          IN PVOID DeferredContext,
//                          IN PVOID SystemArgument1,
//                          IN PVOID SystemArgument2)
// {
//     PDEVICE_OBJECT FdoDevice;
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//
//     DPRINT("USBPORT_WorkerRequestDpc: ...\n");
//
//     FdoDevice = DeferredContext;
//     FdoExtension = FdoDevice->DeviceExtension;
//
//     if (!InterlockedIncrement(&FdoExtension->IsrDpcHandlerCounter))
//     {
//         USBPORT_DpcHandler(FdoDevice);
//     }
//
//     InterlockedDecrement(&FdoExtension->IsrDpcHandlerCounter);
// }
//
// VOID
// NTAPI
// USBPORT_DoneTransfer(IN PUSBPORT_TRANSFER Transfer)
// {
//     PUSBPORT_ENDPOINT          Endpoint;
//     PDEVICE_OBJECT             FdoDevice;
//     PUSBPORT_DEVICE_EXTENSION  FdoExtension;
//     PURB                       Urb;
//     PIRP                       Irp;
//     KIRQL                      CancelIrql;
//     KIRQL                      OldIrql;
//
//     DPRINT_CORE("USBPORT_DoneTransfer: Transfer - %p\n", Transfer);
//
//     Endpoint = Transfer->Endpoint;
//     FdoDevice = Endpoint->FdoDevice;
//     FdoExtension = FdoDevice->DeviceExtension;
//
//     Urb = Transfer->Urb;
//     Irp = Transfer->Irp;
//
//     KeAcquireSpinLock(&FdoExtension->FlushTransferSpinLock, &OldIrql);
//
//     if (Irp)
//     {
//         IoAcquireCancelSpinLock(&CancelIrql);
//         IoSetCancelRoutine(Irp, NULL);
//         IoReleaseCancelSpinLock(CancelIrql);
//
//         USBPORT_RemoveActiveTransferIrp(FdoDevice, Irp);
//     }
//
//     KeReleaseSpinLock(&FdoExtension->FlushTransferSpinLock, OldIrql);
//
//     USBPORT_USBDStatusToNtStatus(Transfer->Urb, Transfer->USBDStatus);
//     USBPORT_CompleteTransfer(Urb, Urb->UrbHeader.Status);
//
//     DPRINT_CORE("USBPORT_DoneTransfer: exit\n");
// }
//
// VOID
// NTAPI
// USBPORT_FlushDoneTransfers(IN PDEVICE_OBJECT FdoDevice)
// {
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     PLIST_ENTRY DoneTransferList;
//     PUSBPORT_TRANSFER Transfer;
//     PUSBPORT_ENDPOINT Endpoint;
//     ULONG TransferCount;
//     KIRQL OldIrql;
//     BOOLEAN IsHasTransfers;
//
//     DPRINT_CORE("USBPORT_FlushDoneTransfers: ...\n");
//
//     FdoExtension = FdoDevice->DeviceExtension;
//     DoneTransferList = &FdoExtension->DoneTransferList;
//
//     while (TRUE)
//     {
//         KeAcquireSpinLock(&FdoExtension->DoneTransferSpinLock, &OldIrql);
//
//         if (IsListEmpty(DoneTransferList))
//             break;
//
//         Transfer = CONTAINING_RECORD(DoneTransferList->Flink,
//                                      USBPORT_TRANSFER,
//                                      TransferLink);
//
//         RemoveHeadList(DoneTransferList);
//         KeReleaseSpinLock(&FdoExtension->DoneTransferSpinLock, OldIrql);
//
//         if (Transfer)
//         {
//             Endpoint = Transfer->Endpoint;
//
//             if ((Transfer->Flags & TRANSFER_FLAG_SPLITED))
//             {
//                 USBPORT_DoneSplitTransfer(Transfer);
//             }
//             else
//             {
//                 USBPORT_DoneTransfer(Transfer);
//             }
//
//             IsHasTransfers = USBPORT_EndpointHasQueuedTransfers(FdoDevice,
//                                                                 Endpoint,
//                                                                 &TransferCount);
//
//             if (IsHasTransfers && !TransferCount)
//             {
//                 USBPORT_InvalidateEndpointHandler(FdoDevice,
//                                                   Endpoint,
//                                                   INVALIDATE_ENDPOINT_WORKER_DPC);
//             }
//         }
//     }
//
//     KeReleaseSpinLock(&FdoExtension->DoneTransferSpinLock, OldIrql);
// }
//
//
// VOID
// NTAPI
// USBPORT_TransferFlushDpc(IN PRKDPC Dpc,
//                          IN PVOID DeferredContext,
//                          IN PVOID SystemArgument1,
//                          IN PVOID SystemArgument2)
// {
//     PDEVICE_OBJECT FdoDevice;
//
//     DPRINT_CORE("USBPORT_TransferFlushDpc: ...\n");
//     FdoDevice = DeferredContext;
//     USBPORT_FlushDoneTransfers(FdoDevice);
// }
//
// BOOLEAN
// NTAPI
// USBPORT_QueueDoneTransfer(IN PUSBPORT_TRANSFER Transfer,
//                           IN USBD_STATUS USBDStatus)
// {
//     PDEVICE_OBJECT FdoDevice;
//     PUSBPORT_DEVICE_EXTENSION  FdoExtension;
//
//     DPRINT_CORE("USBPORT_QueueDoneTransfer: Transfer - %p, USBDStatus - %p\n",
//                 Transfer,
//                 USBDStatus);
//
//     FdoDevice = Transfer->Endpoint->FdoDevice;
//     FdoExtension = FdoDevice->DeviceExtension;
//
//     RemoveEntryList(&Transfer->TransferLink);
//     Transfer->USBDStatus = USBDStatus;
//
//     ExInterlockedInsertTailList(&FdoExtension->DoneTransferList,
//                                 &Transfer->TransferLink,
//                                 &FdoExtension->DoneTransferSpinLock);
//
//     return KeInsertQueueDpc(&FdoExtension->TransferFlushDpc, NULL, NULL);
// }
//
// VOID
// NTAPI
// USBPORT_DpcHandler(IN PDEVICE_OBJECT FdoDevice)
// {
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     PUSBPORT_ENDPOINT Endpoint;
//     PLIST_ENTRY Entry;
//     LIST_ENTRY List;
//     LONG LockCounter;
//
//     DPRINT_CORE("USBPORT_DpcHandler: ...\n");
//
//     FdoExtension = FdoDevice->DeviceExtension;
//
//     InitializeListHead(&List);
//
//     KeAcquireSpinLockAtDpcLevel(&FdoExtension->EndpointListSpinLock);
//     Entry = FdoExtension->EndpointList.Flink;
//
//     while (Entry && Entry != &FdoExtension->EndpointList)
//     {
//         Endpoint = CONTAINING_RECORD(Entry,
//                                      USBPORT_ENDPOINT,
//                                      EndpointLink);
//
//         LockCounter = InterlockedIncrement(&Endpoint->LockCounter);
//
//         if (USBPORT_GetEndpointState(Endpoint) != USBPORT_ENDPOINT_ACTIVE ||
//             LockCounter ||
//             Endpoint->Flags & ENDPOINT_FLAG_ROOTHUB_EP0)
//         {
//             InterlockedDecrement(&Endpoint->LockCounter);
//         }
//         else
//         {
//             InsertTailList(&List, &Endpoint->DispatchLink);
//
//             if (Endpoint->WorkerLink.Flink && Endpoint->WorkerLink.Blink)
//             {
//                 RemoveEntryList(&Endpoint->WorkerLink);
//
//                 Endpoint->WorkerLink.Flink = NULL;
//                 Endpoint->WorkerLink.Blink = NULL;
//             }
//         }
//
//         Entry = Endpoint->EndpointLink.Flink;
//     }
//
//     KeReleaseSpinLockFromDpcLevel(&FdoExtension->EndpointListSpinLock);
//
//     while (!IsListEmpty(&List))
//     {
//         Endpoint = CONTAINING_RECORD(List.Flink,
//                                      USBPORT_ENDPOINT,
//                                      DispatchLink);
//
//         RemoveEntryList(List.Flink);
//         Endpoint->DispatchLink.Flink = NULL;
//         Endpoint->DispatchLink.Blink = NULL;
//
//         USBPORT_EndpointWorker(Endpoint, TRUE);
//         USBPORT_FlushPendingTransfers(Endpoint);
//     }
//
//     KeAcquireSpinLockAtDpcLevel(&FdoExtension->EndpointListSpinLock);
//
//     if (!IsListEmpty(&FdoExtension->WorkerList))
//     {
//         USBPORT_SignalWorkerThread(FdoDevice);
//     }
//
//     KeReleaseSpinLockFromDpcLevel(&FdoExtension->EndpointListSpinLock);
//
//     USBPORT_FlushDoneTransfers(FdoDevice);
// }
//
// VOID
// NTAPI
// USBPORT_IsrDpcHandler(IN PDEVICE_OBJECT FdoDevice,
//                       IN BOOLEAN IsDpcHandler)
// {
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     PUSBPORT_REGISTRATION_PACKET Packet;
//     PUSBPORT_ENDPOINT Endpoint;
//     PLIST_ENTRY List;
//     ULONG FrameNumber;
//
//     ASSERT(KeGetCurrentIrql() == DISPATCH_LEVEL);
//
//     DPRINT_CORE("USBPORT_IsrDpcHandler: IsDpcHandler - %x\n", IsDpcHandler);
//
//     FdoExtension = FdoDevice->DeviceExtension;
//     Packet = &FdoExtension->MiniPortInterface->Packet;
//
//     if (InterlockedIncrement(&FdoExtension->IsrDpcHandlerCounter))
//     {
//         KeInsertQueueDpc(&FdoExtension->IsrDpc, NULL, NULL);
//         InterlockedDecrement(&FdoExtension->IsrDpcHandlerCounter);
//         return;
//     }
//
//     /* Process the state change list.
//      * - Always process the list (don't flush during suspend)
//      * - If controller is suspended/off, mark endpoints as ready immediately
//      * - Don't request interrupts if suspended */
//     KeAcquireSpinLockAtDpcLevel(&FdoExtension->EpStateChangeSpinLock);
//     List = FdoExtension->EpStateChangeList.Flink;
//     while (List != &FdoExtension->EpStateChangeList)
//     {
//         BOOLEAN EndpointReady = FALSE;
//         PLIST_ENTRY NextList;
//         BOOLEAN ControllerSuspended;
//
//         Endpoint = CONTAINING_RECORD(List,
//                                      USBPORT_ENDPOINT,
//                                      StateChangeLink);
//
//         DPRINT_CORE("USBPORT_IsrDpcHandler: Endpoint - %p\n", Endpoint);
//
//         /* Save the next entry before we potentially remove this one */
//         NextList = List->Flink;
//
//         /* Check if controller is suspended/off  */
//         ControllerSuspended = (FdoExtension->Flags & USBPORT_FLAG_HC_SUSPEND) != 0 ||
//                               (FdoExtension->CommonExtension.DevicePowerState == PowerDeviceD3);
//
//         if (ControllerSuspended)
//         {
//             /* Controller is suspended/off - mark endpoint as ready immediately */
//             EndpointReady = TRUE;
//         }
//         else
//         {
//             KeAcquireSpinLockAtDpcLevel(&FdoExtension->MiniportSpinLock);
//             FrameNumber = Packet->Get32BitFrameNumber(FdoExtension->MiniPortExt);
//             KeReleaseSpinLockFromDpcLevel(&FdoExtension->MiniportSpinLock);
//
//             /* Check if the endpoint is ready to be processed */
//             if (FrameNumber > Endpoint->FrameNumber ||
//                 (Endpoint->Flags & ENDPOINT_FLAG_NUKE))
//             {
//                 EndpointReady = TRUE;
//             }
//         }
//
//
//         if (EndpointReady)
//         {
//             /* Endpoint is ready - remove it from the list and process it */
//             RemoveEntryList(&Endpoint->StateChangeLink);
//             Endpoint->StateChangeLink.Flink = NULL;
//             Endpoint->StateChangeLink.Blink = NULL;
//
//             KeAcquireSpinLockAtDpcLevel(&Endpoint->StateChangeSpinLock);
//             Endpoint->StateLast = Endpoint->StateNext;
//             KeReleaseSpinLockFromDpcLevel(&Endpoint->StateChangeSpinLock);
//
//             DPRINT_CORE("USBPORT_IsrDpcHandler: Endpoint->StateLast - %x\n",
//                         Endpoint->StateLast);
//             
//             KeReleaseSpinLockFromDpcLevel(&FdoExtension->EpStateChangeSpinLock);
//
//             if (IsDpcHandler)
//             {
//                 USBPORT_InvalidateEndpointHandler(FdoDevice,
//                                                   Endpoint,
//                                                   INVALIDATE_ENDPOINT_ONLY);
//             }
//             else
//             {
//                 USBPORT_InvalidateEndpointHandler(FdoDevice,
//                                                   Endpoint,
//                                                   INVALIDATE_ENDPOINT_WORKER_THREAD);
//             }
//
//             KeAcquireSpinLockAtDpcLevel(&FdoExtension->EpStateChangeSpinLock);
//         }
//         else if (!ControllerSuspended)
//         {
//             /* Endpoint is not ready yet - leave it in the list and request interrupt.
//              * Don't request interrupts if controller is suspended. */
//             KeAcquireSpinLockAtDpcLevel(&FdoExtension->MiniportSpinLock);
//             Packet->InterruptNextSOF(FdoExtension->MiniPortExt);
//             KeReleaseSpinLockFromDpcLevel(&FdoExtension->MiniportSpinLock);
//         }
//         /* If suspended and not ready, leave it in list but don't request interrupt */
//
//         /* Move to the next entry */
//         List = NextList;
//     }
//
//     KeReleaseSpinLockFromDpcLevel(&FdoExtension->EpStateChangeSpinLock);
//
//     if (IsDpcHandler)
//     {
//         USBPORT_DpcHandler(FdoDevice);
//     }
//
//     InterlockedDecrement(&FdoExtension->IsrDpcHandlerCounter);
// }
//
// VOID
// NTAPI
// USBPORT_IsrDpc(IN PRKDPC Dpc,
//                IN PVOID DeferredContext,
//                IN PVOID SystemArgument1,
//                IN PVOID SystemArgument2)
// {
//     PDEVICE_OBJECT FdoDevice;
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     PUSBPORT_REGISTRATION_PACKET Packet;
//     BOOLEAN InterruptEnable;
//
//     DPRINT_INT("USBPORT_IsrDpc: DeferredContext - %p, SystemArgument2 - %p\n",
//                DeferredContext,
//                SystemArgument2);
//
//     FdoDevice = DeferredContext;
//     FdoExtension = FdoDevice->DeviceExtension;
//     Packet = &FdoExtension->MiniPortInterface->Packet;
//
//     if (SystemArgument2)
//     {
//         InterlockedDecrement(&FdoExtension->IsrDpcCounter);
//     }
//
//     KeAcquireSpinLockAtDpcLevel(&FdoExtension->MiniportInterruptsSpinLock);
//     InterruptEnable = (FdoExtension->Flags & USBPORT_FLAG_INTERRUPT_ENABLED) ==
//                        USBPORT_FLAG_INTERRUPT_ENABLED;
//
//     Packet->InterruptDpc(FdoExtension->MiniPortExt, InterruptEnable);
//
//     KeReleaseSpinLockFromDpcLevel(&FdoExtension->MiniportInterruptsSpinLock);
//
//     if (FdoExtension->Flags & USBPORT_FLAG_HC_SUSPEND &&
//         FdoExtension->TimerFlags & USBPORT_TMFLAG_WAKE)
//     {
//         USBPORT_CompletePdoWaitWake(FdoDevice);
//     }
//     else
//     {
//         USBPORT_IsrDpcHandler(FdoDevice, TRUE);
//     }
//
//     DPRINT_INT("USBPORT_IsrDpc: exit\n");
// }
//
// BOOLEAN
// NTAPI
// USBPORT_InterruptService(IN PKINTERRUPT Interrupt,
//                          IN PVOID ServiceContext)
// {
//     PDEVICE_OBJECT FdoDevice;
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     PUSBPORT_REGISTRATION_PACKET Packet;
//     BOOLEAN Result = FALSE;
//
//     FdoDevice = ServiceContext;
//     FdoExtension = FdoDevice->DeviceExtension;
//     Packet = &FdoExtension->MiniPortInterface->Packet;
//
//     DPRINT_INT("USBPORT_InterruptService: FdoExtension[%p]->Flags - %08X\n",
//            FdoExtension,
//            FdoExtension->Flags);
//
//     if (FdoExtension->Flags & USBPORT_FLAG_INTERRUPT_ENABLED &&
//         FdoExtension->MiniPortFlags & USBPORT_MPFLAG_INTERRUPTS_ENABLED)
//     {
//         Result = Packet->InterruptService(FdoExtension->MiniPortExt);
//
//         if (Result)
//         {
//             KeInsertQueueDpc(&FdoExtension->IsrDpc, NULL, NULL);
//         }
//     }
//
//     DPRINT_INT("USBPORT_InterruptService: return - %x\n", Result);
//
//     return Result;
// }
//
// VOID
// NTAPI
// USBPORT_SignalWorkerThread(IN PDEVICE_OBJECT FdoDevice)
// {
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     KIRQL OldIrql;
//
//     DPRINT_CORE("USBPORT_SignalWorkerThread ...\n");
//
//     FdoExtension = FdoDevice->DeviceExtension;
//
//     KeAcquireSpinLock(&FdoExtension->WorkerThreadEventSpinLock, &OldIrql);
//     KeSetEvent(&FdoExtension->WorkerThreadEvent, EVENT_INCREMENT, FALSE);
//     KeReleaseSpinLock(&FdoExtension->WorkerThreadEventSpinLock, OldIrql);
// }
//
// VOID
// NTAPI
// USBPORT_WorkerThreadHandler(IN PDEVICE_OBJECT FdoDevice)
// {
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     PUSBPORT_REGISTRATION_PACKET Packet;
//     PLIST_ENTRY workerList;
//     KIRQL OldIrql;
//     PUSBPORT_ENDPOINT Endpoint;
//     LIST_ENTRY list;
//     BOOLEAN Result;
//
//     DPRINT_CORE("USBPORT_WorkerThreadHandler: ...\n");
//
//     FdoExtension = FdoDevice->DeviceExtension;
//     Packet = &FdoExtension->MiniPortInterface->Packet;
//
//     KeAcquireSpinLock(&FdoExtension->MiniportSpinLock, &OldIrql);
//
//     if (!(FdoExtension->Flags & USBPORT_FLAG_HC_SUSPEND))
//     {
//         Packet->CheckController(FdoExtension->MiniPortExt);
//     }
//
//     KeReleaseSpinLock(&FdoExtension->MiniportSpinLock, OldIrql);
//
//     InitializeListHead(&list);
//
//     USBPORT_FlushAllEndpoints(FdoDevice);
//
//     while (TRUE)
//     {
//         KeRaiseIrql(DISPATCH_LEVEL, &OldIrql);
//         KeAcquireSpinLockAtDpcLevel(&FdoExtension->EndpointListSpinLock);
//
//         workerList = &FdoExtension->WorkerList;
//
//         if (IsListEmpty(workerList))
//             break;
//
//         Endpoint = CONTAINING_RECORD(workerList->Flink,
//                                      USBPORT_ENDPOINT,
//                                      WorkerLink);
//
//         DPRINT_CORE("USBPORT_WorkerThreadHandler: Endpoint - %p\n", Endpoint);
//
//         RemoveHeadList(workerList);
//         Endpoint->WorkerLink.Blink = NULL;
//         Endpoint->WorkerLink.Flink = NULL;
//
//         KeReleaseSpinLockFromDpcLevel(&FdoExtension->EndpointListSpinLock);
//
//         Result = USBPORT_EndpointWorker(Endpoint, FALSE);
//         KeAcquireSpinLockAtDpcLevel(&FdoExtension->EndpointListSpinLock);
//
//         if (Result)
//         {
//             if (Endpoint->FlushAbortLink.Flink == NULL ||
//                 Endpoint->FlushAbortLink.Blink == NULL)
//             {
//                 InsertTailList(&list, &Endpoint->FlushAbortLink);
//             }
//         }
//
//         while (!IsListEmpty(&list))
//         {
//             Endpoint = CONTAINING_RECORD(list.Flink,
//                                          USBPORT_ENDPOINT,
//                                          FlushAbortLink);
//
//             RemoveHeadList(&list);
//
//             Endpoint->FlushAbortLink.Flink = NULL;
//             Endpoint->FlushAbortLink.Blink = NULL;
//
//             if (Endpoint->WorkerLink.Flink == NULL ||
//                 Endpoint->WorkerLink.Blink == NULL)
//             {
//                 InsertTailList(&FdoExtension->WorkerList,
//                                &Endpoint->WorkerLink);
//
//                 USBPORT_SignalWorkerThread(FdoDevice);
//             }
//         }
//
//         KeReleaseSpinLockFromDpcLevel(&FdoExtension->EndpointListSpinLock);
//         KeLowerIrql(OldIrql);
//     }
//
//     KeReleaseSpinLockFromDpcLevel(&FdoExtension->EndpointListSpinLock);
//     KeLowerIrql(OldIrql);
//
//     USBPORT_FlushClosedEndpointList(FdoDevice);
// }
//
// VOID
// NTAPI
// USBPORT_DoRootHubCallback(IN PDEVICE_OBJECT FdoDevice)
// {
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     PDEVICE_OBJECT PdoDevice;
//     PUSBPORT_RHDEVICE_EXTENSION PdoExtension;
//     PRH_INIT_CALLBACK RootHubInitCallback;
//     PVOID RootHubInitContext;
//
//     FdoExtension = FdoDevice->DeviceExtension;
//
//     DPRINT("USBPORT_DoRootHubCallback: FdoDevice - %p\n", FdoDevice);
//
//     PdoDevice = FdoExtension->RootHubPdo;
//
//     if (PdoDevice)
//     {
//         PdoExtension = PdoDevice->DeviceExtension;
//
//         RootHubInitContext = PdoExtension->RootHubInitContext;
//         RootHubInitCallback = PdoExtension->RootHubInitCallback;
//
//         PdoExtension->RootHubInitCallback = NULL;
//         PdoExtension->RootHubInitContext = NULL;
//
//         if (RootHubInitCallback)
//         {
//             RootHubInitCallback(RootHubInitContext);
//         }
//     }
//
//     DPRINT("USBPORT_DoRootHubCallback: exit\n");
// }
//
// VOID
// NTAPI
// USBPORT_SynchronizeRootHubCallback(IN PDEVICE_OBJECT FdoDevice,
//                                    IN PDEVICE_OBJECT Usb2FdoDevice)
// {
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     PUSBPORT_REGISTRATION_PACKET Packet;
//     PUSBPORT_DEVICE_EXTENSION Usb2FdoExtension;
//     PDEVICE_RELATIONS CompanionControllersList;
//     PUSBPORT_DEVICE_EXTENSION CompanionFdoExtension;
//     PDEVICE_OBJECT * Entry;
//     ULONG ix;
//
//     DPRINT("USBPORT_SynchronizeRootHubCallback: FdoDevice - %p, Usb2FdoDevice - %p\n",
//            FdoDevice,
//            Usb2FdoDevice);
//
//     FdoExtension = FdoDevice->DeviceExtension;
//     Packet = &FdoExtension->MiniPortInterface->Packet;
//
//     if (Usb2FdoDevice == NULL &&
//         !(Packet->MiniPortFlags & USB_MINIPORT_FLAGS_USB2))
//     {
//         /* Not Companion USB11 Controller */
//         USBPORT_DoRootHubCallback(FdoDevice);
//
//         FdoExtension->Flags &= ~USBPORT_FLAG_RH_INIT_CALLBACK;
//         InterlockedCompareExchange(&FdoExtension->RHInitCallBackLock, 0, 1);
//
//         DPRINT("USBPORT_SynchronizeRootHubCallback: exit\n");
//         return;
//     }
//
//     /* USB2 or Companion USB11 */
//
//     DPRINT("USBPORT_SynchronizeRootHubCallback: FdoExtension->Flags - %p\n",
//            FdoExtension->Flags);
//
//     if (!(FdoExtension->Flags & USBPORT_FLAG_COMPANION_HC))
//     {
//         KeWaitForSingleObject(&FdoExtension->ControllerSemaphore,
//                               Executive,
//                               KernelMode,
//                               FALSE,
//                               NULL);
//
//         FdoExtension->Flags |= USBPORT_FLAG_PWR_AND_CHIRP_LOCK;
//
//         if (!(FdoExtension->Flags & (USBPORT_FLAG_HC_SUSPEND |
//                                      USBPORT_FLAG_POWER_AND_CHIRP_OK)))
//         {
//             USBPORT_RootHubPowerAndChirpAllCcPorts(FdoDevice);
//             FdoExtension->Flags |= USBPORT_FLAG_POWER_AND_CHIRP_OK;
//         }
//
//         FdoExtension->Flags &= ~USBPORT_FLAG_PWR_AND_CHIRP_LOCK;
//
//         KeReleaseSemaphore(&FdoExtension->ControllerSemaphore,
//                            LOW_REALTIME_PRIORITY,
//                            1,
//                            FALSE);
//
//         CompanionControllersList = USBPORT_FindCompanionControllers(FdoDevice,
//                                                                     FALSE,
//                                                                     TRUE);
//
//         if (CompanionControllersList)
//         {
//             Entry = &CompanionControllersList->Objects[0];
//
//             for (ix = 0; ix < CompanionControllersList->Count; ++ix)
//             {
//                 CompanionFdoExtension = ((*Entry)->DeviceExtension);
//
//                 InterlockedCompareExchange(&CompanionFdoExtension->RHInitCallBackLock,
//                                            0,
//                                            1);
//
//                 ++Entry;
//             }
//
//             ExFreePoolWithTag(CompanionControllersList, USB_PORT_TAG);
//         }
//
//         USBPORT_DoRootHubCallback(FdoDevice);
//
//         FdoExtension->Flags &= ~USBPORT_FLAG_RH_INIT_CALLBACK;
//         InterlockedCompareExchange(&FdoExtension->RHInitCallBackLock, 0, 1);
//     }
//     else
//     {
//         Usb2FdoExtension = Usb2FdoDevice->DeviceExtension;
//
//         USBPORT_Wait(FdoDevice, 50);
//
//         while (FdoExtension->RHInitCallBackLock)
//         {
//             USBPORT_Wait(FdoDevice, 10);
//
//             Usb2FdoExtension->Flags |= USBPORT_FLAG_RH_INIT_CALLBACK;
//             USBPORT_SignalWorkerThread(Usb2FdoDevice);
//         }
//
//         USBPORT_DoRootHubCallback(FdoDevice);
//
//         FdoExtension->Flags &= ~USBPORT_FLAG_RH_INIT_CALLBACK;
//     }
//
//     DPRINT("USBPORT_SynchronizeRootHubCallback: exit\n");
// }
//
// VOID
// NTAPI
// USBPORT_WorkerThread(IN PVOID StartContext)
// {
//     PDEVICE_OBJECT FdoDevice;
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     LARGE_INTEGER OldTime;
//     LARGE_INTEGER NewTime;
//     KIRQL OldIrql;
//
//     DPRINT_CORE("USBPORT_WorkerThread ...\n");
//
//     FdoDevice = StartContext;
//     FdoExtension = FdoDevice->DeviceExtension;
//
//     FdoExtension->WorkerThread = KeGetCurrentThread();
//
//     do
//     {
//         KeQuerySystemTime(&OldTime);
//
//         KeWaitForSingleObject(&FdoExtension->WorkerThreadEvent,
//                               Suspended,
//                               KernelMode,
//                               FALSE,
//                               NULL);
//
//         if (FdoExtension->Flags & USBPORT_FLAG_WORKER_THREAD_EXIT)
//         {
//             break;
//         }
//
//         KeQuerySystemTime(&NewTime);
//
//         KeAcquireSpinLock(&FdoExtension->WorkerThreadEventSpinLock, &OldIrql);
//         KeClearEvent(&FdoExtension->WorkerThreadEvent);
//         KeReleaseSpinLock(&FdoExtension->WorkerThreadEventSpinLock, OldIrql);
//         DPRINT_CORE("USBPORT_WorkerThread: run\n");
//
//         if (FdoExtension->MiniPortFlags & USBPORT_MPFLAG_INTERRUPTS_ENABLED)
//         {
//             USBPORT_DoSetPowerD0(FdoDevice);
//
//             if (FdoExtension->Flags & USBPORT_FLAG_RH_INIT_CALLBACK)
//             {
//                 PDEVICE_OBJECT USB2FdoDevice = NULL;
//
//                 USB2FdoDevice = USBPORT_FindUSB2Controller(FdoDevice);
//                 USBPORT_SynchronizeRootHubCallback(FdoDevice, USB2FdoDevice);
//             }
//         }
//
//         USBPORT_WorkerThreadHandler(FdoDevice);
//     }
//     while (!(FdoExtension->Flags & USBPORT_FLAG_WORKER_THREAD_ON));
//
//     PsTerminateSystemThread(0);
// }
//
// NTSTATUS
// NTAPI
// USBPORT_CreateWorkerThread(IN PDEVICE_OBJECT FdoDevice)
// {
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     NTSTATUS Status;
//
//     DPRINT("USBPORT_CreateWorkerThread ...\n");
//
//     FdoExtension = FdoDevice->DeviceExtension;
//
//     FdoExtension->Flags &= ~USBPORT_FLAG_WORKER_THREAD_ON;
//
//     KeInitializeEvent(&FdoExtension->WorkerThreadEvent,
//                       NotificationEvent,
//                       FALSE);
//
//     Status = PsCreateSystemThread(&FdoExtension->WorkerThreadHandle,
//                                   THREAD_ALL_ACCESS,
//                                   NULL,
//                                   NULL,
//                                   NULL,
//                                   USBPORT_WorkerThread,
//                                   (PVOID)FdoDevice);
//
//     return Status;
// }
//
// VOID
// NTAPI
// USBPORT_StopWorkerThread(IN PDEVICE_OBJECT FdoDevice)
// {
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     NTSTATUS Status;
//
//     DPRINT("USBPORT_StopWorkerThread ...\n");
//
//     FdoExtension = FdoDevice->DeviceExtension;
//
//     FdoExtension->Flags |= USBPORT_FLAG_WORKER_THREAD_EXIT;
//     USBPORT_SignalWorkerThread(FdoDevice);
//     Status = ZwWaitForSingleObject(FdoExtension->WorkerThreadHandle, FALSE, NULL);
//     NT_ASSERT(Status == STATUS_SUCCESS);
// }
//
// VOID
// NTAPI
// USBPORT_SynchronizeControllersStart(IN PDEVICE_OBJECT FdoDevice)
// {
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     PDEVICE_OBJECT PdoDevice;
//     PUSBPORT_RHDEVICE_EXTENSION PdoExtension;
//     PDEVICE_OBJECT USB2FdoDevice = NULL;
//     PUSBPORT_DEVICE_EXTENSION USB2FdoExtension;
//     BOOLEAN IsOn;
//
//     DPRINT_TIMER("USBPORT_SynchronizeControllersStart: FdoDevice - %p\n",
//                  FdoDevice);
//
//     FdoExtension = FdoDevice->DeviceExtension;
//
//     PdoDevice = FdoExtension->RootHubPdo;
//
//     if (!PdoDevice)
//     {
//         return;
//     }
//
//     PdoExtension = PdoDevice->DeviceExtension;
//
//     if (PdoExtension->RootHubInitCallback == NULL ||
//         FdoExtension->Flags & USBPORT_FLAG_RH_INIT_CALLBACK)
//     {
//         return;
//     }
//
//     DPRINT_TIMER("USBPORT_SynchronizeControllersStart: Flags - %p\n",
//                  FdoExtension->Flags);
//
//     if (FdoExtension->Flags & USBPORT_FLAG_COMPANION_HC)
//     {
//         IsOn = FALSE;
//
//         USB2FdoDevice = USBPORT_FindUSB2Controller(FdoDevice);
//
//         DPRINT_TIMER("USBPORT_SynchronizeControllersStart: USB2FdoDevice - %p\n",
//                      USB2FdoDevice);
//
//         if (USB2FdoDevice)
//         {
//             USB2FdoExtension = USB2FdoDevice->DeviceExtension;
//
//             if (USB2FdoExtension->CommonExtension.PnpStateFlags &
//                 USBPORT_PNP_STATE_STARTED)
//             {
//                 IsOn = TRUE;
//             }
//         }
//
//         if (!(FdoExtension->Flags & USBPORT_FLAG_NO_HACTION))
//         {
//             goto Start;
//         }
//
//         USB2FdoDevice = NULL;
//     }
//
//     IsOn = TRUE;
//
//   Start:
//
//     if (IsOn &&
//         !InterlockedCompareExchange(&FdoExtension->RHInitCallBackLock, 1, 0))
//     {
//         FdoExtension->Flags |= USBPORT_FLAG_RH_INIT_CALLBACK;
//         USBPORT_SignalWorkerThread(FdoDevice);
//
//         if (USB2FdoDevice)
//         {
//             USB2FdoExtension = USB2FdoDevice->DeviceExtension;
//
//             USB2FdoExtension->Flags |= USBPORT_FLAG_RH_INIT_CALLBACK;
//             USBPORT_SignalWorkerThread(USB2FdoDevice);
//         }
//     }
//
//     DPRINT_TIMER("USBPORT_SynchronizeControllersStart: exit\n");
// }
//
// VOID
// NTAPI
// USBPORT_TimerDpc(IN PRKDPC Dpc,
//                  IN PVOID DeferredContext,
//                  IN PVOID SystemArgument1,
//                  IN PVOID SystemArgument2)
// {
//     PDEVICE_OBJECT FdoDevice;
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     PUSBPORT_REGISTRATION_PACKET Packet;
//     LARGE_INTEGER DueTime = {{0, 0}};
//     ULONG TimerFlags;
//     PTIMER_WORK_QUEUE_ITEM IdleQueueItem;
//     KIRQL OldIrql;
//     KIRQL TimerOldIrql;
//
//     DPRINT_TIMER("USBPORT_TimerDpc: Dpc - %p, DeferredContext - %p\n",
//            Dpc,
//            DeferredContext);
//
//     FdoDevice = DeferredContext;
//     FdoExtension = FdoDevice->DeviceExtension;
//     Packet = &FdoExtension->MiniPortInterface->Packet;
//
//     KeAcquireSpinLock(&FdoExtension->TimerFlagsSpinLock, &TimerOldIrql);
//
//     TimerFlags = FdoExtension->TimerFlags;
//
//     DPRINT_TIMER("USBPORT_TimerDpc: Flags - %p, TimerFlags - %p\n",
//                  FdoExtension->Flags,
//                  TimerFlags);
//
//     if (FdoExtension->Flags & USBPORT_FLAG_HC_SUSPEND &&
//         FdoExtension->Flags & USBPORT_FLAG_HC_WAKE_SUPPORT &&
//         !(TimerFlags & USBPORT_TMFLAG_HC_RESUME))
//     {
//         KeAcquireSpinLock(&FdoExtension->MiniportSpinLock, &OldIrql);
//         Packet->PollController(FdoExtension->MiniPortExt);
//         KeReleaseSpinLock(&FdoExtension->MiniportSpinLock, OldIrql);
//     }
//
//     USBPORT_SynchronizeControllersStart(FdoDevice);
//
//     if (TimerFlags & USBPORT_TMFLAG_HC_SUSPENDED)
//     {
//         USBPORT_BadRequestFlush(FdoDevice);
//         goto Exit;
//     }
//
//     KeAcquireSpinLock(&FdoExtension->MiniportSpinLock, &OldIrql);
//
//     if (!(FdoExtension->Flags & USBPORT_FLAG_HC_SUSPEND))
//     {
//         Packet->CheckController(FdoExtension->MiniPortExt);
//     }
//
//     KeReleaseSpinLock(&FdoExtension->MiniportSpinLock, OldIrql);
//
//     if (FdoExtension->Flags & USBPORT_FLAG_HC_POLLING)
//     {
//         KeAcquireSpinLock(&FdoExtension->MiniportSpinLock, &OldIrql);
//         Packet->PollController(FdoExtension->MiniPortExt);
//         KeReleaseSpinLock(&FdoExtension->MiniportSpinLock, OldIrql);
//     }
//
//     USBPORT_IsrDpcHandler(FdoDevice, FALSE);
//
//     DPRINT_TIMER("USBPORT_TimerDpc: USBPORT_TimeoutAllEndpoints UNIMPLEMENTED.\n");
//     //USBPORT_TimeoutAllEndpoints(FdoDevice);
//     DPRINT_TIMER("USBPORT_TimerDpc: USBPORT_CheckIdleEndpoints UNIMPLEMENTED.\n");
//     //USBPORT_CheckIdleEndpoints(FdoDevice);
//
//     USBPORT_BadRequestFlush(FdoDevice);
//
//     if (FdoExtension->IdleLockCounter > -1 &&
//         !(TimerFlags & USBPORT_TMFLAG_IDLE_QUEUEITEM_ON))
//     {
//         IdleQueueItem = ExAllocatePoolWithTag(NonPagedPool,
//                                               sizeof(TIMER_WORK_QUEUE_ITEM),
//                                               USB_PORT_TAG);
//
//         DPRINT("USBPORT_TimerDpc: IdleLockCounter - %x, IdleQueueItem - %p\n",
//                FdoExtension->IdleLockCounter,
//                IdleQueueItem);
//
//         if (IdleQueueItem)
//         {
//             RtlZeroMemory(IdleQueueItem, sizeof(TIMER_WORK_QUEUE_ITEM));
//
//             IdleQueueItem->WqItem.List.Flink = NULL;
//             IdleQueueItem->WqItem.WorkerRoutine = USBPORT_DoIdleNotificationCallback;
//             IdleQueueItem->WqItem.Parameter = IdleQueueItem;
//
//             IdleQueueItem->FdoDevice = FdoDevice;
//             IdleQueueItem->Context = 0;
//
//             FdoExtension->TimerFlags |= USBPORT_TMFLAG_IDLE_QUEUEITEM_ON;
//
//             ExQueueWorkItem(&IdleQueueItem->WqItem, CriticalWorkQueue);
//         }
//     }
//
// Exit:
//
//     KeReleaseSpinLock(&FdoExtension->TimerFlagsSpinLock, TimerOldIrql);
//
//     if (TimerFlags & USBPORT_TMFLAG_TIMER_QUEUED)
//     {
//         DueTime.QuadPart -= FdoExtension->TimerValue * 10000 +
//                             (KeQueryTimeIncrement() - 1);
//
//         KeSetTimer(&FdoExtension->TimerObject,
//                    DueTime,
//                    &FdoExtension->TimerDpc);
//     }
//
//     DPRINT_TIMER("USBPORT_TimerDpc: exit\n");
// }
//
// BOOLEAN
// NTAPI
// USBPORT_StartTimer(IN PDEVICE_OBJECT FdoDevice,
//                    IN ULONG Time)
// {
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     LARGE_INTEGER DueTime = {{0, 0}};
//     ULONG TimeIncrement;
//     BOOLEAN Result;
//
//     DPRINT_TIMER("USBPORT_StartTimer: FdoDevice - %p, Time - %x\n",
//            FdoDevice,
//            Time);
//
//     FdoExtension = FdoDevice->DeviceExtension;
//
//     TimeIncrement = KeQueryTimeIncrement();
//
//     FdoExtension->TimerFlags |= USBPORT_TMFLAG_TIMER_QUEUED;
//     FdoExtension->TimerValue = Time;
//
//     KeInitializeTimer(&FdoExtension->TimerObject);
//     KeInitializeDpc(&FdoExtension->TimerDpc, USBPORT_TimerDpc, FdoDevice);
//
//     DueTime.QuadPart -= 10000 * Time + (TimeIncrement - 1);
//
//     Result = KeSetTimer(&FdoExtension->TimerObject,
//                         DueTime,
//                         &FdoExtension->TimerDpc);
//
//     return Result;
// }
//
// PUSBPORT_COMMON_BUFFER_HEADER
// NTAPI
// USBPORT_AllocateCommonBuffer(IN PDEVICE_OBJECT FdoDevice,
//                              IN SIZE_T BufferLength)
// {
//     PUSBPORT_COMMON_BUFFER_HEADER HeaderBuffer = NULL;
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     PDMA_ADAPTER DmaAdapter;
//     PDMA_OPERATIONS DmaOperations;
//     SIZE_T HeaderSize;
//     ULONG Length = 0;
//     ULONG LengthPadded;
//     PHYSICAL_ADDRESS LogicalAddress;
//     ULONG_PTR BaseVA;
//     ULONG_PTR StartBufferVA;
//     ULONG StartBufferPA;
//
//     DPRINT("USBPORT_AllocateCommonBuffer: FdoDevice - %p, BufferLength - %p\n",
//            FdoDevice,
//            BufferLength);
//
//     if (BufferLength == 0)
//         goto Exit;
//
//     FdoExtension = FdoDevice->DeviceExtension;
//
//     DmaAdapter = FdoExtension->DmaAdapter;
//     DmaOperations = DmaAdapter->DmaOperations;
//
//     HeaderSize = sizeof(USBPORT_COMMON_BUFFER_HEADER);
//     Length = ROUND_TO_PAGES(BufferLength + HeaderSize);
//     LengthPadded = Length - (BufferLength + HeaderSize);
//
//     BaseVA = (ULONG_PTR)DmaOperations->AllocateCommonBuffer(DmaAdapter,
//                                                             Length,
//                                                             &LogicalAddress,
//                                                             TRUE);
//
//     if (!BaseVA)
//         goto Exit;
//
//     StartBufferVA = BaseVA & ~(PAGE_SIZE - 1);
//     StartBufferPA = LogicalAddress.LowPart & ~(PAGE_SIZE - 1);
//
//     HeaderBuffer = (PUSBPORT_COMMON_BUFFER_HEADER)(StartBufferVA +
//                                                    BufferLength +
//                                                    LengthPadded);
//
//     HeaderBuffer->Length = Length;
//     HeaderBuffer->BaseVA = BaseVA;
//     HeaderBuffer->LogicalAddress = LogicalAddress;
//
//     HeaderBuffer->BufferLength = BufferLength + LengthPadded;
//     HeaderBuffer->VirtualAddress = StartBufferVA;
//     HeaderBuffer->PhysicalAddress = StartBufferPA;
//
//     RtlZeroMemory((PVOID)StartBufferVA, BufferLength + LengthPadded);
//
// Exit:
//     return HeaderBuffer;
// }
//
// VOID
// NTAPI
// USBPORT_FreeCommonBuffer(IN PDEVICE_OBJECT FdoDevice,
//                          IN PUSBPORT_COMMON_BUFFER_HEADER HeaderBuffer)
// {
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     PDMA_ADAPTER DmaAdapter;
//     PDMA_OPERATIONS DmaOperations;
//
//     DPRINT("USBPORT_FreeCommonBuffer: ...\n");
//
//     FdoExtension = FdoDevice->DeviceExtension;
//
//     DmaAdapter = FdoExtension->DmaAdapter;
//     DmaOperations = DmaAdapter->DmaOperations;
//
//     DmaOperations->FreeCommonBuffer(FdoExtension->DmaAdapter,
//                                     HeaderBuffer->Length,
//                                     HeaderBuffer->LogicalAddress,
//                                     (PVOID)HeaderBuffer->VirtualAddress,
//                                     TRUE);
// }
//
// PUSBPORT_MINIPORT_INTERFACE
// NTAPI
// USBPORT_FindMiniPort(IN PDRIVER_OBJECT DriverObject)
// {
//     KIRQL OldIrql;
//     PLIST_ENTRY List;
//     PUSBPORT_MINIPORT_INTERFACE MiniPortInterface;
//     BOOLEAN IsFound = FALSE;
//
//     DPRINT("USBPORT_FindMiniPort: ...\n");
//
//     KeAcquireSpinLock(&USBPORT_SpinLock, &OldIrql);
//
//     for (List = USBPORT_MiniPortDrivers.Flink;
//          List != &USBPORT_MiniPortDrivers;
//          List = List->Flink)
//     {
//         MiniPortInterface = CONTAINING_RECORD(List,
//                                               USBPORT_MINIPORT_INTERFACE,
//                                               DriverLink);
//
//         if (MiniPortInterface->DriverObject == DriverObject)
//         {
//             DPRINT("USBPORT_FindMiniPort: find MiniPortInterface - %p\n",
//                    MiniPortInterface);
//
//             IsFound = TRUE;
//             break;
//         }
//     }
//
//     KeReleaseSpinLock(&USBPORT_SpinLock, OldIrql);
//
//     if (IsFound)
//         return MiniPortInterface;
//     else
//         return NULL;
//
// }
//
// NTSTATUS
// NTAPI
// USBPORT_AddDevice(IN PDRIVER_OBJECT DriverObject,
//                   IN PDEVICE_OBJECT PhysicalDeviceObject)
// {
//     NTSTATUS Status;
//     PUSBPORT_MINIPORT_INTERFACE MiniPortInterface;
//     ULONG DeviceNumber = 0;
//     WCHAR CharDeviceName[64];
//     UNICODE_STRING DeviceName;
//     PDEVICE_OBJECT DeviceObject;
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     PUSBPORT_COMMON_DEVICE_EXTENSION FdoCommonExtension;
//     PDEVICE_OBJECT LowerDevice;
//     ULONG Length;
//
//     DPRINT("USBPORT_AddDevice: DriverObject - %p, PhysicalDeviceObject - %p\n",
//            DriverObject,
//            PhysicalDeviceObject);
//
//     MiniPortInterface = USBPORT_FindMiniPort(DriverObject);
//
//     if (!MiniPortInterface)
//     {
//         DPRINT("USBPORT_AddDevice: USBPORT_FindMiniPort not found MiniPortInterface\n");
//         return STATUS_UNSUCCESSFUL;
//     }
//
//     while (TRUE)
//     {
//         /* Construct device name */
//         RtlStringCbPrintfW(CharDeviceName,
//                            sizeof(CharDeviceName),
//                            L"\\Device\\USBFDO-%d",
//                            DeviceNumber);
//
//         RtlInitUnicodeString(&DeviceName, CharDeviceName);
//
//         ASSERT(MiniPortInterface->Packet.MiniPortExtensionSize <=
//                MAXULONG - sizeof(USBPORT_DEVICE_EXTENSION) - sizeof(USB2_HC_EXTENSION));
//         Length = (ULONG)(sizeof(USBPORT_DEVICE_EXTENSION) +
//                          MiniPortInterface->Packet.MiniPortExtensionSize +
//                          sizeof(USB2_HC_EXTENSION));
//
//         /* Create device */
//         Status = IoCreateDevice(DriverObject,
//                                 Length,
//                                 &DeviceName,
//                                 FILE_DEVICE_CONTROLLER,
//                                 0,
//                                 FALSE,
//                                 &DeviceObject);
//
//         /* Check for success */
//         if (NT_SUCCESS(Status)) break;
//
//         /* Is there a device object with that same name */
//         if ((Status == STATUS_OBJECT_NAME_EXISTS) ||
//             (Status == STATUS_OBJECT_NAME_COLLISION))
//         {
//             /* Try the next name */
//             DeviceNumber++;
//             continue;
//         }
//
//         /* Bail out on other errors */
//         if (!NT_SUCCESS(Status))
//         {
//             DPRINT1("USBPORT_AddDevice: failed to create %wZ, Status %x\n",
//                     &DeviceName,
//                     Status);
//
//             return Status;
//         }
//     }
//
//     DPRINT("USBPORT_AddDevice: created device %p <%wZ>, Status %x\n",
//            DeviceObject,
//            &DeviceName,
//            Status);
//
//     FdoExtension = DeviceObject->DeviceExtension;
//     FdoCommonExtension = &FdoExtension->CommonExtension;
//
//     RtlZeroMemory(FdoExtension, sizeof(USBPORT_DEVICE_EXTENSION));
//
//     FdoCommonExtension->SelfDevice = DeviceObject;
//     FdoCommonExtension->LowerPdoDevice = PhysicalDeviceObject;
//     FdoCommonExtension->IsPDO = FALSE;
//
//     LowerDevice = IoAttachDeviceToDeviceStack(DeviceObject,
//                                               PhysicalDeviceObject);
//
//     FdoCommonExtension->LowerDevice = LowerDevice;
//
//     FdoCommonExtension->DevicePowerState = PowerDeviceD3;
//
//     FdoExtension->MiniPortExt = (PVOID)((ULONG_PTR)FdoExtension +
//                                         sizeof(USBPORT_DEVICE_EXTENSION));
//
//     if (MiniPortInterface->Packet.MiniPortFlags & USB_MINIPORT_FLAGS_USB2)
//     {
//         FdoExtension->Usb2Extension =
//         (PUSB2_HC_EXTENSION)((ULONG_PTR)FdoExtension->MiniPortExt +
//                              MiniPortInterface->Packet.MiniPortExtensionSize);
//
//         DPRINT("USBPORT_AddDevice: Usb2Extension - %p\n",
//                FdoExtension->Usb2Extension);
//
//         USB2_InitController(FdoExtension->Usb2Extension);
//     }
//     else
//     {
//         FdoExtension->Usb2Extension = NULL;
//     }
//
//     FdoExtension->MiniPortInterface = MiniPortInterface;
//     FdoExtension->FdoNameNumber = DeviceNumber;
//
//     KeInitializeSemaphore(&FdoExtension->DeviceSemaphore, 1, 1);
//     KeInitializeSemaphore(&FdoExtension->ControllerSemaphore, 1, 1);
//
//     InitializeListHead(&FdoExtension->EndpointList);
//     InitializeListHead(&FdoExtension->DoneTransferList);
//     InitializeListHead(&FdoExtension->WorkerList);
//     InitializeListHead(&FdoExtension->EpStateChangeList);
//     InitializeListHead(&FdoExtension->MapTransferList);
//     InitializeListHead(&FdoExtension->DeviceHandleList);
//     InitializeListHead(&FdoExtension->IdleIrpList);
//     InitializeListHead(&FdoExtension->BadRequestList);
//     InitializeListHead(&FdoExtension->EndpointClosedList);
//
//     DeviceObject->Flags &= ~DO_DEVICE_INITIALIZING;
//
//     return Status;
// }
//
// VOID
// NTAPI
// USBPORT_Unload(IN PDRIVER_OBJECT DriverObject)
// {
//     PUSBPORT_MINIPORT_INTERFACE MiniPortInterface;
//
//     DPRINT1("USBPORT_Unload: FIXME!\n");
//
//     MiniPortInterface = USBPORT_FindMiniPort(DriverObject);
//
//     if (!MiniPortInterface)
//     {
//         DPRINT("USBPORT_Unload: CRITICAL ERROR!!! Not found MiniPortInterface\n");
//         KeBugCheckEx(BUGCODE_USB_DRIVER, 1, 0, 0, 0);
//     }
//
//     DPRINT1("USBPORT_Unload: UNIMPLEMENTED. FIXME.\n");
//     //MiniPortInterface->DriverUnload(DriverObject); // Call MiniPort _HCI_Unload
// }
//
// VOID
// NTAPI
// USBPORT_MiniportCompleteTransfer(IN PVOID MiniPortExtension,
//                                  IN PVOID MiniPortEndpoint,
//                                  IN PVOID TransferParameters,
//                                  IN USBD_STATUS USBDStatus,
//                                  IN ULONG TransferLength)
// {
//     PUSBPORT_TRANSFER Transfer;
//     PUSBPORT_TRANSFER ParentTransfer;
//     PUSBPORT_TRANSFER SplitTransfer;
//     PLIST_ENTRY SplitHead;
//     PLIST_ENTRY Entry;
//     KIRQL OldIrql;
//
//     DPRINT_CORE("USBPORT_MiniportCompleteTransfer: USBDStatus - %x, TransferLength - %x\n",
//                 USBDStatus,
//                 TransferLength);
//
//     Transfer = CONTAINING_RECORD(TransferParameters,
//                                  USBPORT_TRANSFER,
//                                  TransferParameters);
//
//     Transfer->Flags |= TRANSFER_FLAG_COMPLETED;
//     Transfer->CompletedTransferLen = TransferLength;
//
//     if (((Transfer->Flags & TRANSFER_FLAG_SPLITED) == 0) ||
//         TransferLength >= Transfer->TransferParameters.TransferBufferLength)
//     {
//         goto Exit;
//     }
//
//     ParentTransfer = Transfer->ParentTransfer;
//
//     KeAcquireSpinLock(&ParentTransfer->TransferSpinLock, &OldIrql);
//
//     if (IsListEmpty(&ParentTransfer->SplitTransfersList))
//     {
//         goto Exit;
//     }
//
//     SplitHead = &ParentTransfer->SplitTransfersList;
//     Entry = SplitHead->Flink;
//
//     while (Entry && !IsListEmpty(SplitHead))
//     {
//         SplitTransfer = CONTAINING_RECORD(Entry,
//                                           USBPORT_TRANSFER,
//                                           SplitLink);
//
//         if (!(SplitTransfer->Flags & TRANSFER_FLAG_SUBMITED))
//         {
//             DPRINT1("USBPORT_MiniportCompleteTransfer: SplitTransfer->Flags - %X\n",
//                     SplitTransfer->Flags);
//             //Add TRANSFER_FLAG_xxx
//         }
//
//         Entry = Entry->Flink;
//     }
//
//     KeReleaseSpinLock(&ParentTransfer->TransferSpinLock, OldIrql);
//
// Exit:
//     USBPORT_QueueDoneTransfer(Transfer, USBDStatus);
// }
//
// VOID
// NTAPI
// USBPORT_AsyncTimerDpc(IN PRKDPC Dpc,
//                       IN PVOID DeferredContext,
//                       IN PVOID SystemArgument1,
//                       IN PVOID SystemArgument2)
// {
//     PDEVICE_OBJECT FdoDevice;
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     PUSBPORT_ASYNC_CALLBACK_DATA AsyncCallbackData;
//
//     DPRINT("USBPORT_AsyncTimerDpc: ...\n");
//
//     AsyncCallbackData = DeferredContext;
//     FdoDevice = AsyncCallbackData->FdoDevice;
//     FdoExtension = FdoDevice->DeviceExtension;
//
//     (*AsyncCallbackData->CallbackFunction)(FdoExtension->MiniPortExt,
//                                            &AsyncCallbackData->CallbackContext);
//
//     ExFreePoolWithTag(AsyncCallbackData, USB_PORT_TAG);
// }
//
// ULONG
// NTAPI
// USBPORT_RequestAsyncCallback(IN PVOID MiniPortExtension,
//                              IN ULONG TimerValue,
//                              IN PVOID Buffer,
//                              IN SIZE_T Length,
//                              IN ASYNC_TIMER_CALLBACK * Callback)
// {
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     PDEVICE_OBJECT FdoDevice;
//     PUSBPORT_ASYNC_CALLBACK_DATA AsyncCallbackData;
//     LARGE_INTEGER DueTime = {{0, 0}};
//
//     DPRINT("USBPORT_RequestAsyncCallback: ...\n");
//
//     FdoExtension = (PUSBPORT_DEVICE_EXTENSION)((ULONG_PTR)MiniPortExtension -
//                                                sizeof(USBPORT_DEVICE_EXTENSION));
//
//     FdoDevice = FdoExtension->CommonExtension.SelfDevice;
//
//     AsyncCallbackData = ExAllocatePoolWithTag(NonPagedPool,
//                                               sizeof(USBPORT_ASYNC_CALLBACK_DATA) + Length,
//                                               USB_PORT_TAG);
//
//     if (!AsyncCallbackData)
//     {
//         DPRINT1("USBPORT_RequestAsyncCallback: Not allocated AsyncCallbackData!\n");
//         return 0;
//     }
//
//     RtlZeroMemory(AsyncCallbackData,
//                   sizeof(USBPORT_ASYNC_CALLBACK_DATA) + Length);
//
//     if (Length)
//     {
//         RtlCopyMemory(&AsyncCallbackData->CallbackContext, Buffer, Length);
//     }
//
//     AsyncCallbackData->FdoDevice = FdoDevice;
//     AsyncCallbackData->CallbackFunction = Callback;
//
//     KeInitializeTimer(&AsyncCallbackData->AsyncTimer);
//
//     KeInitializeDpc(&AsyncCallbackData->AsyncTimerDpc,
//                     USBPORT_AsyncTimerDpc,
//                     AsyncCallbackData);
//
//     DueTime.QuadPart -= (KeQueryTimeIncrement() - 1) + 10000 * TimerValue;
//
//     KeSetTimer(&AsyncCallbackData->AsyncTimer,
//                DueTime,
//                &AsyncCallbackData->AsyncTimerDpc);
//
//     return 0;
// }
//
// PVOID
// NTAPI
// USBPORT_GetMappedVirtualAddress(IN ULONG PhysicalAddress,
//                                 IN PVOID MiniPortExtension,
//                                 IN PVOID MiniPortEndpoint)
// {
//     PUSBPORT_COMMON_BUFFER_HEADER HeaderBuffer;
//     PUSBPORT_ENDPOINT Endpoint;
//     ULONG Offset;
//     ULONG_PTR VirtualAddress;
//
//     DPRINT_CORE("USBPORT_GetMappedVirtualAddress ...\n");
//
//     Endpoint = (PUSBPORT_ENDPOINT)((ULONG_PTR)MiniPortEndpoint -
//                                    sizeof(USBPORT_ENDPOINT));
//
//     if (!Endpoint)
//     {
//         ASSERT(FALSE);
//     }
//
//     HeaderBuffer = Endpoint->HeaderBuffer;
//
//     Offset = PhysicalAddress - HeaderBuffer->PhysicalAddress;
//     VirtualAddress = HeaderBuffer->VirtualAddress + Offset;
//
//     return (PVOID)VirtualAddress;
// }
//
// ULONG
// NTAPI
// USBPORT_InvalidateEndpoint(IN PVOID MiniPortExtension,
//                            IN PVOID MiniPortEndpoint)
// {
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     PDEVICE_OBJECT FdoDevice;
//     PUSBPORT_ENDPOINT Endpoint;
//
//     DPRINT_CORE("USBPORT_InvalidateEndpoint: ...\n");
//
//     FdoExtension = (PUSBPORT_DEVICE_EXTENSION)((ULONG_PTR)MiniPortExtension -
//                                                sizeof(USBPORT_DEVICE_EXTENSION));
//
//     FdoDevice = FdoExtension->CommonExtension.SelfDevice;
//
//     if (!MiniPortEndpoint)
//     {
//         USBPORT_InvalidateEndpointHandler(FdoDevice,
//                                           NULL,
//                                           INVALIDATE_ENDPOINT_ONLY);
//         return 0;
//     }
//
//     Endpoint = (PUSBPORT_ENDPOINT)((ULONG_PTR)MiniPortEndpoint -
//                                    sizeof(USBPORT_ENDPOINT));
//
//     USBPORT_InvalidateEndpointHandler(FdoDevice,
//                                       Endpoint,
//                                       INVALIDATE_ENDPOINT_ONLY);
//
//     return 0;
// }
//
// VOID
// NTAPI
// USBPORT_CompleteTransfer(IN PURB Urb,
//                          IN USBD_STATUS TransferStatus)
// {
//     struct _URB_CONTROL_TRANSFER *UrbTransfer;
//     PUSBPORT_TRANSFER Transfer;
//     NTSTATUS Status;
//     PIRP Irp;
//     KIRQL OldIrql;
//     PRKEVENT Event;
//     BOOLEAN WriteToDevice;
//     BOOLEAN IsFlushSuccess;
//     PMDL Mdl;
//     ULONG_PTR CurrentVa;
//     SIZE_T TransferLength;
//     PUSBPORT_ENDPOINT Endpoint;
//     PDEVICE_OBJECT FdoDevice;
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     PDMA_OPERATIONS DmaOperations;
//
//     DPRINT("USBPORT_CompleteTransfer: Urb - %p, TransferStatus - %X\n",
//            Urb,
//            TransferStatus);
//
//     UrbTransfer = &Urb->UrbControlTransfer;
//     Transfer = UrbTransfer->hca.Reserved8[0];
//
//     Transfer->USBDStatus = TransferStatus;
//     Status = USBPORT_USBDStatusToNtStatus(Urb, TransferStatus);
//
//     UrbTransfer->TransferBufferLength = Transfer->CompletedTransferLen;
//
//     if (Transfer->Flags & TRANSFER_FLAG_DMA_MAPPED)
//     {
//         Endpoint = Transfer->Endpoint;
//         FdoDevice = Endpoint->FdoDevice;
//         FdoExtension = FdoDevice->DeviceExtension;
//         DmaOperations = FdoExtension->DmaAdapter->DmaOperations;
//
//         WriteToDevice = Transfer->Direction == USBPORT_DMA_DIRECTION_TO_DEVICE;
//         Mdl = UrbTransfer->TransferBufferMDL;
//         CurrentVa = (ULONG_PTR)MmGetMdlVirtualAddress(Mdl);
//         TransferLength = UrbTransfer->TransferBufferLength;
//
//         IsFlushSuccess = DmaOperations->FlushAdapterBuffers(FdoExtension->DmaAdapter,
//                                                             Mdl,
//                                                             Transfer->MapRegisterBase,
//                                                             (PVOID)CurrentVa,
//                                                             TransferLength,
//                                                             WriteToDevice);
//
//         if (!IsFlushSuccess)
//         {
//             DPRINT("USBPORT_CompleteTransfer: no FlushAdapterBuffers !!!\n");
//             ASSERT(FALSE);
//         }
//
//         KeRaiseIrql(DISPATCH_LEVEL, &OldIrql);
//
//         DmaOperations->FreeMapRegisters(FdoExtension->DmaAdapter,
//                                         Transfer->MapRegisterBase,
//                                         Transfer->NumberOfMapRegisters);
//
//         KeLowerIrql(OldIrql);
//     }
//
//     if (Urb->UrbHeader.UsbdFlags & USBD_FLAG_ALLOCATED_MDL)
//     {
//         IoFreeMdl(Transfer->TransferBufferMDL);
//         Urb->UrbHeader.UsbdFlags |= ~USBD_FLAG_ALLOCATED_MDL;
//     }
//
//     Urb->UrbControlTransfer.hca.Reserved8[0] = NULL;
//     Urb->UrbHeader.UsbdFlags |= ~USBD_FLAG_ALLOCATED_TRANSFER;
//
//     Irp = Transfer->Irp;
//
//     if (Irp)
//     {
//         if (!NT_SUCCESS(Status))
//         {
//             //DbgBreakPoint();
//             DPRINT1("USBPORT_CompleteTransfer: Irp - %p complete with Status - %lx\n",
//                     Irp,
//                     Status);
//
//             USBPORT_DumpingURB(Urb);
//         }
//
//         Irp->IoStatus.Status = Status;
//         Irp->IoStatus.Information = 0;
//
//         KeRaiseIrql(DISPATCH_LEVEL, &OldIrql);
//         IoCompleteRequest(Irp, IO_NO_INCREMENT);
//         KeLowerIrql(OldIrql);
//     }
//
//     Event = Transfer->Event;
//
//     if (Event)
//     {
//         KeSetEvent(Event, EVENT_INCREMENT, FALSE);
//     }
//
//     ExFreePoolWithTag(Transfer, USB_PORT_TAG);
//
//     DPRINT_CORE("USBPORT_CompleteTransfer: exit\n");
// }
//
// IO_ALLOCATION_ACTION
// NTAPI
// USBPORT_MapTransfer(IN PDEVICE_OBJECT FdoDevice,
//                     IN PIRP Irp,
//                     IN PVOID MapRegisterBase,
//                     IN PVOID Context)
// {
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     PDMA_ADAPTER DmaAdapter;
//     PUSBPORT_TRANSFER Transfer;
//     PURB Urb;
//     PUSBPORT_ENDPOINT Endpoint;
//     PMDL Mdl;
//     ULONG_PTR CurrentVa;
//     PUSBPORT_SCATTER_GATHER_LIST sgList;
//     SIZE_T CurrentLength;
//     ULONG ix;
//     BOOLEAN WriteToDevice;
//     PHYSICAL_ADDRESS PhAddr = {{0, 0}};
//     PHYSICAL_ADDRESS PhAddress = {{0, 0}};
//     ULONG TransferLength;
//     SIZE_T SgCurrentLength;
//     SIZE_T ElementLength;
//     PUSBPORT_DEVICE_HANDLE DeviceHandle;
//     PDMA_OPERATIONS DmaOperations;
//     USBD_STATUS USBDStatus;
//     LIST_ENTRY List;
//     PUSBPORT_TRANSFER transfer;
//
//     DPRINT_CORE("USBPORT_MapTransfer: ...\n");
//
//     FdoExtension = FdoDevice->DeviceExtension;
//     DmaAdapter = FdoExtension->DmaAdapter;
//     DmaOperations = DmaAdapter->DmaOperations;
//
//     Transfer = Context;
//
//     Urb = Transfer->Urb;
//     Endpoint = Transfer->Endpoint;
//     TransferLength = Transfer->TransferParameters.TransferBufferLength;
//
//     Mdl = Urb->UrbControlTransfer.TransferBufferMDL;
//     CurrentVa = (ULONG_PTR)MmGetMdlVirtualAddress(Mdl);
//
//     sgList = &Transfer->SgList;
//
//     sgList->Flags = 0;
//     sgList->CurrentVa = CurrentVa;
//     sgList->MappedSystemVa = MmGetSystemAddressForMdlSafe(Mdl,
//                                                           NormalPagePriority);
//     Transfer->MapRegisterBase = MapRegisterBase;
//
//     ix = 0;
//     CurrentLength = 0;
//
//     do
//     {
//         WriteToDevice = Transfer->Direction == USBPORT_DMA_DIRECTION_TO_DEVICE;
//         ASSERT(Transfer->Direction != 0);
//
//         PhAddress = DmaOperations->MapTransfer(DmaAdapter,
//                                                Mdl,
//                                                MapRegisterBase,
//                                                (PVOID)CurrentVa,
//                                                &TransferLength,
//                                                WriteToDevice);
//
//         DPRINT_CORE("USBPORT_MapTransfer: PhAddress.LowPart - %p, PhAddress.HighPart - %x, TransferLength - %x\n",
//                PhAddress.LowPart,
//                PhAddress.HighPart,
//                TransferLength);
//
//         PhAddress.HighPart = 0;
//         SgCurrentLength = TransferLength;
//
//         do
//         {
//             ElementLength = PAGE_SIZE - (PhAddress.LowPart & (PAGE_SIZE - 1));
//
//             if (ElementLength > SgCurrentLength)
//                 ElementLength = SgCurrentLength;
//
//             DPRINT_CORE("USBPORT_MapTransfer: PhAddress.LowPart - %p, HighPart - %x, ElementLength - %x\n",
//                    PhAddress.LowPart,
//                    PhAddress.HighPart,
//                    ElementLength);
//
//             sgList->SgElement[ix].SgPhysicalAddress = PhAddress;
//             sgList->SgElement[ix].SgTransferLength = ElementLength;
//             sgList->SgElement[ix].SgOffset = CurrentLength +
//                                              (TransferLength - SgCurrentLength);
//
//             PhAddress.LowPart += ElementLength;
//             SgCurrentLength -= ElementLength;
//
//             ++ix;
//         }
//         while (SgCurrentLength);
//
//         if (PhAddr.QuadPart == PhAddress.QuadPart)
//         {
//             DPRINT1("USBPORT_MapTransfer: PhAddr == PhAddress\n");
//             ASSERT(FALSE);
//         }
//
//         PhAddr = PhAddress;
//
//         CurrentLength += TransferLength;
//         CurrentVa += TransferLength;
//
//         TransferLength = Transfer->TransferParameters.TransferBufferLength -
//                          CurrentLength;
//     }
//     while (CurrentLength != Transfer->TransferParameters.TransferBufferLength);
//
//     sgList->SgElementCount = ix;
//
//     if (Endpoint->EndpointProperties.DeviceSpeed == UsbHighSpeed)
//     {
//         Transfer->Flags |= TRANSFER_FLAG_HIGH_SPEED;
//     }
//
//     Transfer->Flags |= TRANSFER_FLAG_DMA_MAPPED;
//
//     if ((Transfer->Flags & TRANSFER_FLAG_ISO) == 0)
//     {
//         KeAcquireSpinLock(&Endpoint->EndpointSpinLock,
//                           &Endpoint->EndpointOldIrql);
//
//         USBPORT_SplitTransfer(FdoDevice, Endpoint, Transfer, &List);
//
//         while (!IsListEmpty(&List))
//         {
//             transfer = CONTAINING_RECORD(List.Flink,
//                                          USBPORT_TRANSFER,
//                                          TransferLink);
//
//             RemoveHeadList(&List);
//             InsertTailList(&Endpoint->TransferList, &transfer->TransferLink);
//         }
//
//         KeReleaseSpinLock(&Endpoint->EndpointSpinLock,
//                           Endpoint->EndpointOldIrql);
//     }
//     else
//     {
//         USBDStatus = USBPORT_InitializeIsoTransfer(FdoDevice,
//                                                    &Urb->UrbIsochronousTransfer,
//                                                    Transfer);
//
//         if (USBDStatus != USBD_STATUS_SUCCESS)
//         {
//             KeAcquireSpinLock(&Endpoint->EndpointSpinLock,
//                               &Endpoint->EndpointOldIrql);
//
//             USBPORT_QueueDoneTransfer(Transfer, USBDStatus);
//
//             KeReleaseSpinLock(&Endpoint->EndpointSpinLock,
//                               Endpoint->EndpointOldIrql);
//         }
//     }
//
//     DeviceHandle = Urb->UrbHeader.UsbdDeviceHandle;
//     InterlockedDecrement(&DeviceHandle->DeviceHandleLock);
//
//     if (USBPORT_EndpointWorker(Endpoint, 0))
//     {
//         USBPORT_InvalidateEndpointHandler(FdoDevice,
//                                           Endpoint,
//                                           INVALIDATE_ENDPOINT_WORKER_THREAD);
//     }
//
//     return DeallocateObjectKeepRegisters;
// }
//
// VOID
// NTAPI
// USBPORT_FlushMapTransfers(IN PDEVICE_OBJECT FdoDevice)
// {
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     PLIST_ENTRY MapTransferList;
//     PUSBPORT_TRANSFER Transfer;
//     ULONG NumMapRegisters;
//     PMDL Mdl;
//     SIZE_T TransferBufferLength;
//     ULONG_PTR VirtualAddr;
//     KIRQL OldIrql;
//     NTSTATUS Status;
//     PDMA_OPERATIONS DmaOperations;
//
//     DPRINT_CORE("USBPORT_FlushMapTransfers: ...\n");
//
//     FdoExtension = FdoDevice->DeviceExtension;
//     DmaOperations = FdoExtension->DmaAdapter->DmaOperations;
//
//     KeRaiseIrql(DISPATCH_LEVEL, &OldIrql);
//
//     while (TRUE)
//     {
//         MapTransferList = &FdoExtension->MapTransferList;
//
//         if (IsListEmpty(&FdoExtension->MapTransferList))
//         {
//             KeLowerIrql(OldIrql);
//             return;
//         }
//
//         Transfer = CONTAINING_RECORD(MapTransferList->Flink,
//                                      USBPORT_TRANSFER,
//                                      TransferLink);
//
//         RemoveHeadList(MapTransferList);
//
//         Mdl = Transfer->Urb->UrbControlTransfer.TransferBufferMDL;
//         TransferBufferLength = Transfer->TransferParameters.TransferBufferLength;
//         VirtualAddr = (ULONG_PTR)MmGetMdlVirtualAddress(Mdl);
//
//         NumMapRegisters = ADDRESS_AND_SIZE_TO_SPAN_PAGES(VirtualAddr,
//                                                          TransferBufferLength);
//
//         Transfer->NumberOfMapRegisters = NumMapRegisters;
//
//         Status = DmaOperations->AllocateAdapterChannel(FdoExtension->DmaAdapter,
//                                                        FdoDevice,
//                                                        NumMapRegisters,
//                                                        USBPORT_MapTransfer,
//                                                        Transfer);
//
//         if (!NT_SUCCESS(Status))
//             ASSERT(FALSE);
//     }
//
//     KeLowerIrql(OldIrql);
// }
//
// USBD_STATUS
// NTAPI
// USBPORT_AllocateTransfer(IN PDEVICE_OBJECT FdoDevice,
//                          IN PURB Urb,
//                          IN PUSBPORT_DEVICE_HANDLE DeviceHandle,
//                          IN PIRP Irp,
//                          IN PRKEVENT Event)
// {
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     SIZE_T TransferLength;
//     PMDL Mdl;
//     ULONG_PTR VirtualAddr;
//     ULONG PagesNeed = 0;
//     SIZE_T PortTransferLength;
//     SIZE_T FullTransferLength;
//     PUSBPORT_TRANSFER Transfer;
//     PUSBPORT_PIPE_HANDLE PipeHandle;
//     USBD_STATUS USBDStatus;
//     SIZE_T IsoBlockLen = 0;
//
//     DPRINT_CORE("USBPORT_AllocateTransfer: FdoDevice - %p, Urb - %p, DeviceHandle - %p, Irp - %p, Event - %p\n",
//            FdoDevice,
//            Urb,
//            DeviceHandle,
//            Irp,
//            Event);
//
//     FdoExtension = FdoDevice->DeviceExtension;
//
//     TransferLength = Urb->UrbControlTransfer.TransferBufferLength;
//     PipeHandle = Urb->UrbControlTransfer.PipeHandle;
//
//     if (TransferLength)
//     {
//         Mdl = Urb->UrbControlTransfer.TransferBufferMDL;
//         VirtualAddr = (ULONG_PTR)MmGetMdlVirtualAddress(Mdl);
//
//         PagesNeed = ADDRESS_AND_SIZE_TO_SPAN_PAGES(VirtualAddr,
//                                                    TransferLength);
//         if (PagesNeed > 0)
//         {
//             PagesNeed--;
//         }
//     }
//
//     if (Urb->UrbHeader.Function == URB_FUNCTION_ISOCH_TRANSFER)
//     {
//         DPRINT1("USBPORT_AllocateTransfer: ISOCH_TRANSFER UNIMPLEMENTED. FIXME\n");
//
//         //IsoBlockLen = sizeof(USBPORT_ISO_BLOCK) +
//         //              Urb->UrbIsochronousTransfer.NumberOfPackets *
//         //              sizeof(USBPORT_ISO_BLOCK_PACKET);
//     }
//
//     PortTransferLength = sizeof(USBPORT_TRANSFER) +
//                          PagesNeed * sizeof(USBPORT_SCATTER_GATHER_ELEMENT) +
//                          IsoBlockLen;
//
//     FullTransferLength = PortTransferLength +
//                          FdoExtension->MiniPortInterface->Packet.MiniPortTransferSize;
//
//     Transfer = ExAllocatePoolWithTag(NonPagedPool,
//                                      FullTransferLength,
//                                      USB_PORT_TAG);
//
//     if (!Transfer)
//     {
//         DPRINT1("USBPORT_AllocateTransfer: Transfer not allocated!\n");
//         return USBD_STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     RtlZeroMemory(Transfer, FullTransferLength);
//
//     Transfer->Irp = Irp;
//     Transfer->Urb = Urb;
//     Transfer->Endpoint = PipeHandle->Endpoint;
//     Transfer->Event = Event;
//     Transfer->PortTransferLength = PortTransferLength;
//     Transfer->FullTransferLength = FullTransferLength;
//     Transfer->IsoBlockPtr = NULL;
//     Transfer->Period = 0;
//     Transfer->ParentTransfer = Transfer;
//
//     if (IsoBlockLen)
//     {
//         Transfer->IsoBlockPtr = (PVOID)((ULONG_PTR)Transfer +
//                                  PortTransferLength - IsoBlockLen);
//
//         Transfer->Period = PipeHandle->Endpoint->EndpointProperties.Period;
//         Transfer->Flags |= TRANSFER_FLAG_ISO;
//     }
//
//     Transfer->MiniportTransfer = (PVOID)((ULONG_PTR)Transfer +
//                                                     PortTransferLength);
//
//     KeInitializeSpinLock(&Transfer->TransferSpinLock);
//
//     Urb->UrbControlTransfer.hca.Reserved8[0] = Transfer;
//     Urb->UrbHeader.UsbdFlags |= USBD_FLAG_ALLOCATED_TRANSFER;
//
//     USBDStatus = USBD_STATUS_SUCCESS;
//
//     DPRINT_CORE("USBPORT_AllocateTransfer: return USBDStatus - %x\n",
//                 USBDStatus);
//
//     return USBDStatus;
// }
//
// NTSTATUS
// NTAPI
// USBPORT_Dispatch(IN PDEVICE_OBJECT DeviceObject,
//                  IN PIRP Irp)
// {
//     PUSBPORT_COMMON_DEVICE_EXTENSION DeviceExtension;
//     PIO_STACK_LOCATION IoStack;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     DeviceExtension = DeviceObject->DeviceExtension;
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//
//     if (DeviceExtension->PnpStateFlags & USBPORT_PNP_STATE_FAILED)
//     {
//         DPRINT1("USBPORT_Dispatch: USBPORT_PNP_STATE_FAILED\n");
//         DbgBreakPoint();
//     }
//
//     switch (IoStack->MajorFunction)
//     {
//         case IRP_MJ_DEVICE_CONTROL:
//             if (DeviceExtension->IsPDO)
//             {
//                 DPRINT("USBPORT_Dispatch: PDO IRP_MJ_DEVICE_CONTROL. Major - %d, Minor - %d\n",
//                        IoStack->MajorFunction,
//                        IoStack->MinorFunction);
//
//                 Status = USBPORT_PdoDeviceControl(DeviceObject, Irp);
//             }
//             else
//             {
//                 DPRINT("USBPORT_Dispatch: FDO IRP_MJ_DEVICE_CONTROL. Major - %d, Minor - %d\n",
//                        IoStack->MajorFunction,
//                        IoStack->MinorFunction);
//
//                 Status = USBPORT_FdoDeviceControl(DeviceObject, Irp);
//             }
//
//             break;
//
//         case IRP_MJ_INTERNAL_DEVICE_CONTROL:
//             if (DeviceExtension->IsPDO)
//             {
//                 DPRINT("USBPORT_Dispatch: PDO IRP_MJ_INTERNAL_DEVICE_CONTROL. Major - %d, Minor - %d\n",
//                        IoStack->MajorFunction,
//                        IoStack->MinorFunction);
//
//                 Status = USBPORT_PdoInternalDeviceControl(DeviceObject, Irp);
//             }
//             else
//             {
//                 DPRINT("USBPORT_Dispatch: FDO IRP_MJ_INTERNAL_DEVICE_CONTROL. Major - %d, Minor - %d\n",
//                        IoStack->MajorFunction,
//                        IoStack->MinorFunction);
//
//                 Status = USBPORT_FdoInternalDeviceControl(DeviceObject, Irp);
//             }
//
//             break;
//
//         case IRP_MJ_POWER:
//             if (DeviceExtension->IsPDO)
//             {
//                 DPRINT("USBPORT_Dispatch: PDO IRP_MJ_POWER. Major - %d, Minor - %d\n",
//                        IoStack->MajorFunction,
//                        IoStack->MinorFunction);
//
//                 Status = USBPORT_PdoPower(DeviceObject, Irp);
//             }
//             else
//             {
//                 DPRINT("USBPORT_Dispatch: FDO IRP_MJ_POWER. Major - %d, Minor - %d\n",
//                        IoStack->MajorFunction,
//                        IoStack->MinorFunction);
//
//                 Status = USBPORT_FdoPower(DeviceObject, Irp);
//             }
//
//             break;
//
//         case IRP_MJ_SYSTEM_CONTROL:
//             if (DeviceExtension->IsPDO)
//             {
//                 DPRINT("USBPORT_Dispatch: PDO IRP_MJ_SYSTEM_CONTROL. Major - %d, Minor - %d\n",
//                        IoStack->MajorFunction,
//                        IoStack->MinorFunction);
//
//                 Status = Irp->IoStatus.Status;
//                 IoCompleteRequest(Irp, IO_NO_INCREMENT);
//             }
//             else
//             {
//                 DPRINT("USBPORT_Dispatch: FDO IRP_MJ_SYSTEM_CONTROL. Major - %d, Minor - %d\n",
//                        IoStack->MajorFunction,
//                        IoStack->MinorFunction);
//
//                 IoSkipCurrentIrpStackLocation(Irp);
//                 Status = IoCallDriver(DeviceExtension->LowerDevice, Irp);
//             }
//
//             break;
//
//         case IRP_MJ_PNP:
//             if (DeviceExtension->IsPDO)
//             {
//                 DPRINT("USBPORT_Dispatch: PDO IRP_MJ_PNP. Major - %d, Minor - %d\n",
//                        IoStack->MajorFunction,
//                        IoStack->MinorFunction);
//
//                 Status = USBPORT_PdoPnP(DeviceObject, Irp);
//             }
//             else
//             {
//                 DPRINT("USBPORT_Dispatch: FDO IRP_MJ_PNP. Major - %d, Minor - %d\n",
//                        IoStack->MajorFunction,
//                        IoStack->MinorFunction);
//
//                 Status = USBPORT_FdoPnP(DeviceObject, Irp);
//             }
//
//             break;
//
//         case IRP_MJ_CREATE:
//         case IRP_MJ_CLOSE:
//             DPRINT("USBPORT_Dispatch: IRP_MJ_CREATE | IRP_MJ_CLOSE\n");
//             Irp->IoStatus.Status = Status;
//             IoCompleteRequest(Irp, IO_NO_INCREMENT);
//             break;
//
//         default:
//             if (DeviceExtension->IsPDO)
//             {
//                 DPRINT("USBPORT_Dispatch: PDO unhandled IRP_MJ_???. Major - %d, Minor - %d\n",
//                        IoStack->MajorFunction,
//                        IoStack->MinorFunction);
//             }
//             else
//             {
//                 DPRINT("USBPORT_Dispatch: FDO unhandled IRP_MJ_???. Major - %d, Minor - %d\n",
//                        IoStack->MajorFunction,
//                        IoStack->MinorFunction);
//             }
//
//             Status = STATUS_INVALID_DEVICE_REQUEST;
//             Irp->IoStatus.Status = Status;
//             IoCompleteRequest(Irp, IO_NO_INCREMENT);
//             break;
//     }
//
//     DPRINT("USBPORT_Dispatch: Status - %x\n", Status);
//     return Status;
// }
//
// ULONG
// NTAPI
// USBPORT_GetHciMn(VOID)
// {
//     return USBPORT_HCI_MN;
// }
//
// NTSTATUS
// NTAPI
// USBPORT_RegisterUSBPortDriver(IN PDRIVER_OBJECT DriverObject,
//                               IN ULONG Version,
//                               IN PUSBPORT_REGISTRATION_PACKET RegPacket)
// {
//     PUSBPORT_MINIPORT_INTERFACE MiniPortInterface;
//
//     DPRINT("USBPORT_RegisterUSBPortDriver: DriverObject - %p, Version - %p, RegPacket - %p\n",
//            DriverObject,
//            Version,
//            RegPacket);
//
//     DPRINT("USBPORT_RegisterUSBPortDriver: sizeof(USBPORT_MINIPORT_INTERFACE) - %x\n",
//            sizeof(USBPORT_MINIPORT_INTERFACE));
//
//     DPRINT("USBPORT_RegisterUSBPortDriver: sizeof(USBPORT_DEVICE_EXTENSION)   - %x\n",
//            sizeof(USBPORT_DEVICE_EXTENSION));
//
//     if (Version < USB10_MINIPORT_INTERFACE_VERSION)
//     {
//         return STATUS_UNSUCCESSFUL;
//     }
//
//     if (!USBPORT_Initialized)
//     {
//         InitializeListHead(&USBPORT_MiniPortDrivers);
//         InitializeListHead(&USBPORT_USB1FdoList);
//         InitializeListHead(&USBPORT_USB2FdoList);
//
//         KeInitializeSpinLock(&USBPORT_SpinLock);
//         USBPORT_Initialized = TRUE;
//     }
//
//     MiniPortInterface = ExAllocatePoolWithTag(NonPagedPool,
//                                               sizeof(USBPORT_MINIPORT_INTERFACE),
//                                               USB_PORT_TAG);
//     if (!MiniPortInterface)
//     {
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     RtlZeroMemory(MiniPortInterface, sizeof(USBPORT_MINIPORT_INTERFACE));
//
//     MiniPortInterface->DriverObject = DriverObject;
//     MiniPortInterface->DriverUnload = DriverObject->DriverUnload;
//     MiniPortInterface->Version = Version;
//
//     ExInterlockedInsertTailList(&USBPORT_MiniPortDrivers,
//                                 &MiniPortInterface->DriverLink,
//                                 &USBPORT_SpinLock);
//
//     DriverObject->DriverExtension->AddDevice = USBPORT_AddDevice;
//     DriverObject->DriverUnload = USBPORT_Unload;
//
//     DriverObject->MajorFunction[IRP_MJ_CREATE] = USBPORT_Dispatch;
//     DriverObject->MajorFunction[IRP_MJ_CLOSE] = USBPORT_Dispatch;
//     DriverObject->MajorFunction[IRP_MJ_DEVICE_CONTROL] = USBPORT_Dispatch;
//     DriverObject->MajorFunction[IRP_MJ_INTERNAL_DEVICE_CONTROL] = USBPORT_Dispatch;
//     DriverObject->MajorFunction[IRP_MJ_PNP] = USBPORT_Dispatch;
//     DriverObject->MajorFunction[IRP_MJ_POWER] = USBPORT_Dispatch;
//     DriverObject->MajorFunction[IRP_MJ_SYSTEM_CONTROL] = USBPORT_Dispatch;
//
//     RegPacket->UsbPortDbgPrint = USBPORT_DbgPrint;
//     RegPacket->UsbPortTestDebugBreak = USBPORT_TestDebugBreak;
//     RegPacket->UsbPortAssertFailure = USBPORT_AssertFailure;
//     RegPacket->UsbPortGetMiniportRegistryKeyValue = USBPORT_GetMiniportRegistryKeyValue;
//     RegPacket->UsbPortInvalidateRootHub = USBPORT_InvalidateRootHub;
//     RegPacket->UsbPortInvalidateEndpoint = USBPORT_InvalidateEndpoint;
//     RegPacket->UsbPortCompleteTransfer = USBPORT_MiniportCompleteTransfer;
//     RegPacket->UsbPortCompleteIsoTransfer = USBPORT_CompleteIsoTransfer;
//     RegPacket->UsbPortLogEntry = USBPORT_LogEntry;
//     RegPacket->UsbPortGetMappedVirtualAddress = USBPORT_GetMappedVirtualAddress;
//     RegPacket->UsbPortRequestAsyncCallback = USBPORT_RequestAsyncCallback;
//     RegPacket->UsbPortReadWriteConfigSpace = USBPORT_ReadWriteConfigSpace;
//     RegPacket->UsbPortWait = USBPORT_Wait;
//     RegPacket->UsbPortInvalidateController = USBPORT_InvalidateController;
//     RegPacket->UsbPortBugCheck = USBPORT_BugCheck;
//     RegPacket->UsbPortNotifyDoubleBuffer = USBPORT_NotifyDoubleBuffer;
//
//     RtlCopyMemory(&MiniPortInterface->Packet,
//                   RegPacket,
//                   sizeof(USBPORT_REGISTRATION_PACKET));
//
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// DriverEntry(IN PDRIVER_OBJECT DriverObject,
//             IN PUNICODE_STRING RegistryPath)
// {
//     return STATUS_SUCCESS;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
